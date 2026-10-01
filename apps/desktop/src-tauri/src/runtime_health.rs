//! Health must remain queryable even when neither log sink can write.
use serde_json::{json, Value};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
    sync::Mutex,
};

#[derive(Default)]
struct State {
    write_errors: u64,
    rotation_errors: u64,
    lock_errors: u64,
    last_log_error: Option<String>,
    last_successful_write: Option<String>,
    write_failed: bool,
    rotation_failed: bool,
    persistence_errors: u64,
    last_persistence_error: Option<String>,
    queue_degraded: bool,
    settings_degraded: bool,
}

#[derive(Default)]
pub(super) struct RuntimeHealth(Mutex<State>);

impl RuntimeHealth {
    pub(super) fn snapshot(&self) -> Value {
        let state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        json!({
            "generalLog": {
                "status": if state.write_failed { "unavailable" } else if state.rotation_failed || state.lock_errors > 0 { "degraded" } else if state.last_successful_write.is_none() { "unknown" } else { "healthy" },
                "logWriteErrors": state.write_errors,
                "logRotationErrors": state.rotation_errors,
                "logLockErrors": state.lock_errors,
                "lastLogError": state.last_log_error,
                "lastSuccessfulWrite": state.last_successful_write,
            },
            "persistence": {
                "status": if state.queue_degraded || state.settings_degraded { "degraded" } else { "healthy" },
                "persistenceErrors": state.persistence_errors,
                "lastPersistenceError": state.last_persistence_error,
                "queueDegraded": state.queue_degraded,
                "settingsDegraded": state.settings_degraded,
            }
        })
    }

    pub(super) fn lock_recovered(&self) {
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        state.lock_errors += 1;
        state.last_log_error = Some("log_lock_poisoned_recovered".into());
    }

    pub(super) fn persistence(&self, component: &str, result: &Result<(), String>) {
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        if component == "queue" {
            state.queue_degraded = result.is_err();
        } else {
            state.settings_degraded = result.is_err();
        }
        if result.is_err() {
            state.persistence_errors += 1;
            // OS errors can contain private paths. Keep only the operation here.
            state.last_persistence_error = Some(format!("{component}_save_failed"));
        }
    }

    fn failure(&self, operation: &str, error: &io::Error, rotation: bool) {
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        if rotation {
            state.rotation_errors += 1;
            state.rotation_failed = true;
        } else {
            state.write_errors += 1;
            state.write_failed = true;
        }
        state.last_log_error = Some(format!("{operation}:{:?}", error.kind()));
    }

    /// Caller serializes append/rotate/read with the log lock. Never logs recursively.
    pub(super) fn append(&self, path: &Path, record: &Value) {
        if let Some(parent) = path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                self.failure("create_directory", &error, false);
                return;
            }
        }
        match fs::metadata(path) {
            Ok(metadata) if metadata.len() > 8 * 1024 * 1024 => {
                let rotated = path.with_extension("log.1");
                let rotation = (|| -> io::Result<()> {
                    match fs::remove_file(&rotated) {
                        Ok(()) => {}
                        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                        Err(error) => return Err(error),
                    }
                    fs::rename(path, rotated)
                })();
                if let Err(error) = rotation {
                    self.failure("rotate", &error, true);
                    // Still try to append to the original, retaining the current event.
                } else {
                    self.0
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .rotation_failed = false;
                }
            }
            Err(error) if error.kind() != io::ErrorKind::NotFound => {
                self.failure("metadata", &error, false);
                return;
            }
            _ => {}
        }
        let write = (|| -> io::Result<()> {
            let mut file = OpenOptions::new().create(true).append(true).open(path)?;
            writeln!(file, "{record}")
        })();
        if let Err(error) = write {
            self.failure("append", &error, false);
        } else {
            let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
            state.write_failed = false;
            state.last_successful_write = Some(chrono::Utc::now().to_rfc3339());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_write_and_recovery_remain_observable() {
        let directory = tempfile::tempdir().unwrap();
        let blocker = directory.path().join("blocker");
        fs::write(&blocker, b"file instead of directory").unwrap();
        let health = RuntimeHealth::default();
        health.append(&blocker.join("general.log"), &json!({"event":"test"}));
        assert_eq!(health.snapshot()["generalLog"]["status"], "unavailable");
        assert_eq!(health.snapshot()["generalLog"]["logWriteErrors"], 1);
        health.append(&directory.path().join("general.log"), &json!({}));
        let snapshot = health.snapshot();
        assert_eq!(snapshot["generalLog"]["status"], "healthy");
        assert_eq!(snapshot["generalLog"]["logWriteErrors"], 1);
        assert!(snapshot["generalLog"]["lastSuccessfulWrite"].is_string());
    }

    #[test]
    fn rotation_failure_preserves_current_event_and_reports_degradation() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("general.log");
        fs::File::create(&path)
            .unwrap()
            .set_len(8 * 1024 * 1024 + 1)
            .unwrap();
        fs::create_dir(path.with_extension("log.1")).unwrap();
        let health = RuntimeHealth::default();
        health.append(&path, &json!({"event":"retained"}));
        assert_eq!(health.snapshot()["generalLog"]["status"], "degraded");
        assert_eq!(health.snapshot()["generalLog"]["logRotationErrors"], 1);
        assert!(fs::read_to_string(&path)
            .unwrap()
            .ends_with("{\"event\":\"retained\"}\n"));
        fs::remove_dir(path.with_extension("log.1")).unwrap();
        health.append(&path, &json!({"event":"recovered"}));
        assert_eq!(health.snapshot()["generalLog"]["status"], "healthy");
    }

    #[test]
    fn persistence_recovery_is_per_component_and_never_exposes_error_text() {
        let health = RuntimeHealth::default();
        health.persistence("queue", &Err("private path".into()));
        health.persistence("settings", &Ok(()));
        assert_eq!(health.snapshot()["persistence"]["status"], "degraded");
        health.persistence("queue", &Ok(()));
        assert_eq!(health.snapshot()["persistence"]["status"], "healthy");
        assert_eq!(health.snapshot()["persistence"]["persistenceErrors"], 1);
        assert!(!health.snapshot().to_string().contains("private path"));
        health.lock_recovered();
        assert_eq!(health.snapshot()["generalLog"]["logLockErrors"], 1);
    }
}
