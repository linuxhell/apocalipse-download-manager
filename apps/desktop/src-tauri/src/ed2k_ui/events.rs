//! The ED2K window's event log: what the network layer and the downloads did and why, kept in
//! memory (newest 2000) so the window can show, filter, copy and save it.
use serde::Serialize;
use serde_json::Value;
use std::{collections::VecDeque, sync::Mutex};

const CAPACITY: usize = 2000;

#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    pub id: u64,
    /// Milliseconds since the Unix epoch.
    pub at: u64,
    /// `info`, `warn` or `error`.
    pub level: &'static str,
    /// `server`, `kad`, `upnp`, `download`, `share` or `network`.
    pub area: &'static str,
    /// A stable name the window translates; `fields` fill its placeholders.
    pub code: &'static str,
    pub fields: Value,
}

#[derive(Default)]
pub struct Events {
    items: Mutex<(u64, VecDeque<Entry>)>,
}

impl Events {
    pub fn push(&self, level: &'static str, area: &'static str, code: &'static str, fields: Value) {
        let at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_millis() as u64);
        let mut guard = self.items.lock().unwrap_or_else(|e| e.into_inner());
        guard.0 += 1;
        let id = guard.0;
        if guard.1.len() == CAPACITY {
            guard.1.pop_front();
        }
        guard.1.push_back(Entry {
            id,
            at,
            level,
            area,
            code,
            fields,
        });
    }
    pub fn info(&self, area: &'static str, code: &'static str, fields: Value) {
        self.push("info", area, code, fields);
    }
    pub fn warn(&self, area: &'static str, code: &'static str, fields: Value) {
        self.push("warn", area, code, fields);
    }
    pub fn error(&self, area: &'static str, code: &'static str, fields: Value) {
        self.push("error", area, code, fields);
    }
    /// Entries newer than `after`, oldest first.
    pub fn since(&self, after: u64) -> Vec<Entry> {
        let guard = self.items.lock().unwrap_or_else(|e| e.into_inner());
        guard.1.iter().filter(|e| e.id > after).cloned().collect()
    }
}

#[tauri::command]
pub fn ed2k_events(state: tauri::State<'_, super::Ed2kState>, after: Option<u64>) -> Vec<Entry> {
    state.events.since(after.unwrap_or(0))
}

/// Saves the text the window shows. With `ask`, a save dialog (starting in the ED2K data folder)
/// lets the user choose where; an empty answer means the dialog was cancelled. Without it the file
/// goes into the ED2K data folder. Returns the path written.
#[tauri::command]
pub async fn ed2k_save_log(
    state: tauri::State<'_, super::Ed2kState>,
    text: String,
    ask: Option<bool>,
) -> Result<String, String> {
    if text.len() > 4 * 1024 * 1024 {
        return Err("log_too_large".into());
    }
    let name = format!(
        "ed2k-log-{}.txt",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    );
    let path = if ask.unwrap_or(false) {
        let start = state.root.clone();
        let chosen = tokio::task::spawn_blocking(move || {
            rfd::FileDialog::new()
                .set_directory(start)
                .set_file_name(name)
                .add_filter("Text", &["txt", "log"])
                .save_file()
        })
        .await
        .map_err(|e| e.to_string())?;
        match chosen {
            Some(path) => path,
            None => return Ok(String::new()),
        }
    } else {
        state.root.join(name)
    };
    let parent = path.parent().ok_or("invalid_log_path")?.to_owned();
    std::fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
    super::save_file(&path, text.as_bytes())?;
    Ok(path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn entries_are_numbered_and_only_newer_ones_are_returned() {
        let events = Events::default();
        events.info("server", "a", json!({}));
        events.warn("upnp", "b", json!({"error": "x"}));
        assert_eq!(events.since(0).len(), 2);
        let newer = events.since(1);
        assert_eq!(newer.len(), 1);
        assert_eq!(
            (newer[0].id, newer[0].level, newer[0].area),
            (2, "warn", "upnp")
        );
        assert!(events.since(2).is_empty());
    }

    #[test]
    fn the_log_keeps_only_the_newest_entries() {
        let events = Events::default();
        for _ in 0..CAPACITY + 50 {
            events.info("network", "tick", json!({}));
        }
        let all = events.since(0);
        assert_eq!(all.len(), CAPACITY);
        assert_eq!(all.first().unwrap().id, 51);
        assert_eq!(all.last().unwrap().id, (CAPACITY + 50) as u64);
    }
}
