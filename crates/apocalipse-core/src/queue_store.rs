//! Transactional queue storage. SQLite is authoritative after the first import;
//! the original JSON remains untouched and exports are explicit snapshots.
use crate::DownloadTask;
use anyhow::{ensure, Context, Result};
use rusqlite::{params, Connection, OpenFlags};
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    time::Duration,
};

const APPLICATION_ID: i32 = 0x41444d51;
const SCHEMA: i32 = 1;
const MAX_IMPORT: u64 = 256 * 1024 * 1024;

struct State {
    connection: Connection,
    revision: i64,
    rows: HashMap<String, (usize, String)>,
}

pub struct QueueStore {
    path: PathBuf,
    state: Mutex<State>,
}

impl QueueStore {
    /// Uses SQLite's online backup protocol, including committed WAL contents.
    /// Copying only the main database file could omit committed transactions.
    pub fn copy_database(source: &Path, destination: &Path) -> Result<()> {
        ensure!(!destination.exists(), "destination queue already exists");
        let connection = Connection::open_with_flags(source, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let app: i32 = connection.query_row("PRAGMA application_id", [], |r| r.get(0))?;
        ensure!(app == APPLICATION_ID, "source is not an ADM queue database");
        let parent = destination
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let temporary = tempfile::NamedTempFile::new_in(parent)?;
        connection.backup(
            rusqlite::MAIN_DB,
            temporary.path(),
            rusqlite::backup::NO_PROGRESS,
        )?;
        temporary.as_file().sync_all()?;
        temporary
            .persist_noclobber(destination)
            .map_err(|e| e.error)?;
        Ok(())
    }
    /// `migrate` handles retired task representations before deserialization.
    /// Failure never substitutes an empty queue for an existing database.
    pub fn open(json_path: &Path, migrate: impl FnOnce(&mut serde_json::Value)) -> Result<Self> {
        let path = json_path.with_extension("sqlite3");
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent)?;
        }
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&path) {
            Ok(file) => drop(file),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
        ensure!(
            !fs::symlink_metadata(&path)?.file_type().is_symlink(),
            "queue database is a symbolic link"
        );
        let mut connection = Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .context("cannot open queue database; existing files were preserved")?;
        connection.busy_timeout(Duration::from_secs(5))?;
        let version: i32 = connection.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        let app: i32 = connection.query_row("PRAGMA application_id", [], |r| r.get(0))?;
        ensure!(
            version <= SCHEMA,
            "queue database was created by a newer ADM version"
        );
        let tables: i64 = connection.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            [],
            |r| r.get(0),
        )?;
        ensure!(
            app == APPLICATION_ID || app == 0 && tables == 0,
            "file is not an ADM queue database"
        );
        connection.pragma_update(None, "synchronous", "FULL")?;
        // WAL is selected only on a positively identified local filesystem.
        // Other platforms/storage retain SQLite's portable transactional journal.
        let desired = if local_wal_storage(&path) {
            "WAL"
        } else {
            "DELETE"
        };
        if connection
            .pragma_update(None, "journal_mode", desired)
            .is_err()
        {
            connection.pragma_update(None, "journal_mode", "DELETE")?;
        }
        connection.pragma_update(None, "wal_autocheckpoint", 512)?;
        let check: String = connection.query_row("PRAGMA quick_check(1)", [], |r| r.get(0))?;
        ensure!(
            check == "ok",
            "queue database integrity check failed; files were preserved"
        );
        let transaction = connection.transaction()?;
        transaction.execute_batch("CREATE TABLE IF NOT EXISTS queue_state (id INTEGER PRIMARY KEY CHECK(id=1), revision INTEGER NOT NULL); CREATE TABLE IF NOT EXISTS tasks (id TEXT PRIMARY KEY NOT NULL, ordinal INTEGER NOT NULL, body TEXT NOT NULL);")?;
        let initialized: i64 =
            transaction.query_row("SELECT count(*) FROM queue_state", [], |r| r.get(0))?;
        if initialized == 0 {
            if json_path.exists() {
                ensure!(
                    fs::metadata(json_path)?.len() <= MAX_IMPORT,
                    "legacy queue exceeds import limit"
                );
                let bytes = fs::read(json_path)?;
                let mut json: serde_json::Value = serde_json::from_slice(&bytes)
                    .context("legacy queue is invalid; original JSON was preserved")?;
                migrate(&mut json);
                let tasks: Vec<DownloadTask> = serde_json::from_value(json)
                    .context("legacy tasks could not be decoded; original JSON was preserved")?;
                let rows = encode(&tasks)?;
                let mut backup = json_path.as_os_str().to_os_string();
                backup.push(".pre-sqlite");
                let backup = PathBuf::from(backup);
                if !backup.exists() {
                    write_new(&backup, &bytes)?;
                }
                for (id, (ordinal, body)) in rows {
                    transaction.execute(
                        "INSERT INTO tasks(id,ordinal,body) VALUES(?1,?2,?3)",
                        params![id, ordinal as i64, body],
                    )?;
                }
            }
            transaction.execute("INSERT INTO queue_state VALUES(1,0)", [])?;
        }
        transaction.pragma_update(None, "application_id", APPLICATION_ID)?;
        transaction.pragma_update(None, "user_version", SCHEMA)?;
        transaction.commit()?;
        let revision =
            connection.query_row("SELECT revision FROM queue_state WHERE id=1", [], |r| {
                r.get(0)
            })?;
        let rows = read_rows(&connection)?;
        Ok(Self {
            path,
            state: Mutex::new(State {
                connection,
                revision,
                rows,
            }),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<Vec<DownloadTask>> {
        let state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("queue database lock poisoned"))?;
        let mut rows = state.rows.values().collect::<Vec<_>>();
        rows.sort_by_key(|(ordinal, _)| *ordinal);
        rows.into_iter()
            .map(|(_, body)| {
                serde_json::from_str(body).context("stored queue task could not be decoded")
            })
            .collect()
    }

    pub fn save(&self, tasks: &[DownloadTask]) -> Result<()> {
        let next = encode(tasks)?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("queue database lock poisoned"))?;
        let State {
            connection,
            revision,
            rows,
        } = &mut *state;
        let transaction = connection.transaction()?;
        let updated = transaction.execute(
            "UPDATE queue_state SET revision=revision+1 WHERE id=1 AND revision=?1",
            [*revision],
        )?;
        ensure!(
            updated == 1,
            "queue changed in another process; reopen it before saving"
        );
        for id in rows.keys().filter(|id| !next.contains_key(*id)) {
            transaction.execute("DELETE FROM tasks WHERE id=?1", [id])?;
        }
        for (id, (ordinal, body)) in &next {
            if !rows
                .get(id)
                .is_some_and(|(old_ordinal, old_body)| old_ordinal == ordinal && old_body == body)
            {
                transaction.execute("INSERT INTO tasks(id,ordinal,body) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET ordinal=excluded.ordinal,body=excluded.body", params![id, *ordinal as i64, body])?;
            }
        }
        transaction.commit()?;
        *revision += 1;
        *rows = next;
        Ok(())
    }

    pub fn export_json(&self, destination: &Path) -> Result<()> {
        ensure!(
            destination != self.path,
            "export cannot replace the queue database"
        );
        ensure!(
            destination.canonicalize().ok() != self.path.canonicalize().ok(),
            "export cannot replace the queue database"
        );
        if destination.parent().and_then(|p| p.canonicalize().ok())
            == self.path.parent().and_then(|p| p.canonicalize().ok())
        {
            let database = self.path.file_name().unwrap().to_string_lossy();
            let filename = destination
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            ensure!(
                ![
                    format!("{database}-wal"),
                    format!("{database}-shm"),
                    format!("{database}-journal")
                ]
                .iter()
                .any(|name| name.eq_ignore_ascii_case(&filename)),
                "export cannot replace a queue journal"
            );
        }
        let bytes = serde_json::to_vec_pretty(&self.load()?)?;
        let parent = destination
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        temporary.write_all(&bytes)?;
        temporary.as_file().sync_all()?;
        temporary.persist(destination).map_err(|e| e.error)?;
        Ok(())
    }
}

/// Both legacy text/CSV lists and exported JSON queues can be imported as new
/// downloads. File destinations and execution states are never replayed blindly.
pub fn import_sources(content: &str) -> Result<Vec<String>> {
    let values = if content.trim_start().starts_with('[') {
        let values: Vec<serde_json::Value> = serde_json::from_str(content)?;
        values
            .into_iter()
            .filter_map(|value| match value {
                serde_json::Value::String(url) => Some(url),
                serde_json::Value::Object(mut task) => task
                    .remove("source")
                    .and_then(|url| url.as_str().map(str::to_owned)),
                _ => None,
            })
            .collect::<Vec<_>>()
    } else {
        content
            .split(|character: char| {
                character.is_whitespace() || character == ',' || character == ';'
            })
            .map(|value| value.trim_matches(['\"', '\'', '[', ']']).to_owned())
            .collect()
    };
    let mut seen = std::collections::HashSet::new();
    Ok(values
        .into_iter()
        .filter(|url| crate::classify_url(url).is_some() && seen.insert(url.clone()))
        .take(500)
        .collect())
}

fn encode(tasks: &[DownloadTask]) -> Result<HashMap<String, (usize, String)>> {
    let mut rows = HashMap::with_capacity(tasks.len());
    for (ordinal, task) in tasks.iter().enumerate() {
        let id = task.id.to_string();
        ensure!(
            rows.insert(id, (ordinal, serde_json::to_string(task)?))
                .is_none(),
            "duplicate queue task identifier"
        );
    }
    Ok(rows)
}

fn read_rows(connection: &Connection) -> Result<HashMap<String, (usize, String)>> {
    let mut statement = connection.prepare("SELECT id,ordinal,body FROM tasks ORDER BY ordinal")?;
    let mut rows = HashMap::new();
    for row in statement.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, String>(2)?,
        ))
    })? {
        let (id, ordinal, body) = row?;
        let task: DownloadTask = serde_json::from_str(&body)?;
        ensure!(
            id == task.id.to_string() && ordinal >= 0,
            "queue row identity is invalid"
        );
        rows.insert(id, (ordinal as usize, body));
    }
    Ok(rows)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(bytes)?;
    file.as_file().sync_all()?;
    file.persist_noclobber(path).map_err(|e| e.error)?;
    Ok(())
}

fn local_wal_storage(path: &Path) -> bool {
    #[cfg(target_os = "linux")]
    {
        let Ok(path) = path.canonicalize() else {
            return false;
        };
        let Ok(mounts) = fs::read_to_string("/proc/self/mountinfo") else {
            return false;
        };
        let mut best = (0, false);
        for line in mounts.lines() {
            let Some((fields, properties)) = line.split_once(" - ") else {
                continue;
            };
            let Some(mount) = fields.split_whitespace().nth(4) else {
                continue;
            };
            let mount = mount
                .replace("\\040", " ")
                .replace("\\011", "\t")
                .replace("\\134", "\\");
            let mount = Path::new(&mount);
            if path.starts_with(mount) && mount.as_os_str().len() >= best.0 {
                best = (
                    mount.as_os_str().len(),
                    matches!(
                        properties.split_whitespace().next(),
                        Some(
                            "ext2"
                                | "ext3"
                                | "ext4"
                                | "xfs"
                                | "btrfs"
                                | "f2fs"
                                | "tmpfs"
                                | "overlay"
                                | "zfs"
                                | "bcachefs"
                        )
                    ),
                );
            }
        }
        best.1
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let Ok(path) = path.canonicalize() else {
            return false;
        };
        let Some(root) = path.ancestors().last() else {
            return false;
        };
        let wide: Vec<u16> = root.as_os_str().encode_wide().chain(Some(0)).collect();
        // SAFETY: the owned UTF-16 root is NUL-terminated and lives throughout
        // this synchronous query. Remote/unknown drive types use DELETE mode.
        matches!(
            unsafe { windows_sys::Win32::Storage::FileSystem::GetDriveTypeW(wide.as_ptr()) },
            2 | 3 | 6
        )
    }
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::ffi::OsStrExt;
        let Ok(path) = std::ffi::CString::new(path.as_os_str().as_bytes()) else {
            return false;
        };
        let mut info = std::mem::MaybeUninit::<libc::statfs>::uninit();
        // SAFETY: pointers reference a terminated path and writable statfs.
        if unsafe { libc::statfs(path.as_ptr(), info.as_mut_ptr()) } != 0 {
            return false;
        }
        // SAFETY: a successful statfs initialized the complete output structure.
        unsafe { info.assume_init().f_flags & libc::MNT_LOCAL as u32 != 0 }
    }
    #[cfg(not(any(target_os = "linux", windows, target_os = "macos")))]
    {
        let _ = path;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn task(name: &str) -> DownloadTask {
        DownloadTask::new(format!("https://example.test/{name}"), name)
    }
    #[test]
    fn migrates_once_preserves_original_and_exports_current_order() {
        let dir = tempfile::tempdir().unwrap();
        let json = dir.path().join("queue.json");
        let original = vec![task("first")];
        let bytes = serde_json::to_vec(&original).unwrap();
        fs::write(&json, &bytes).unwrap();
        let db = QueueStore::open(&json, |_| {}).unwrap();
        assert_eq!(db.load().unwrap()[0].id, original[0].id);
        let next = vec![task("second"), original[0].clone()];
        db.save(&next).unwrap();
        assert_eq!(fs::read(&json).unwrap(), bytes);
        assert_eq!(
            fs::read(dir.path().join("queue.json.pre-sqlite")).unwrap(),
            bytes
        );
        let export = dir.path().join("export.json");
        db.export_json(&export).unwrap();
        let restored: Vec<DownloadTask> =
            serde_json::from_slice(&fs::read(export).unwrap()).unwrap();
        assert_eq!(restored[0].id, next[0].id);
        drop(db);
        let db = QueueStore::open(&json, |_| panic!("must not repeat migration")).unwrap();
        assert_eq!(db.load().unwrap().len(), 2);
    }
    #[test]
    fn failed_transaction_preserves_all_previous_rows_and_can_retry() {
        let dir = tempfile::tempdir().unwrap();
        let json = dir.path().join("queue.json");
        let db = QueueStore::open(&json, |_| {}).unwrap();
        let first = task("first");
        db.save(std::slice::from_ref(&first)).unwrap();
        db.state.lock().unwrap().connection.execute_batch("CREATE TRIGGER reject_new BEFORE INSERT ON tasks BEGIN SELECT RAISE(ABORT,'injected write failure'); END;").unwrap();
        assert!(db.save(&[task("second")]).is_err());
        assert_eq!(db.load().unwrap()[0].id, first.id);
        db.state
            .lock()
            .unwrap()
            .connection
            .execute_batch("DROP TRIGGER reject_new")
            .unwrap();
        db.save(&[]).unwrap();
        assert!(db.load().unwrap().is_empty());
    }
    #[test]
    fn duplicate_identifiers_and_stale_writers_cannot_erase_data() {
        let dir = tempfile::tempdir().unwrap();
        let json = dir.path().join("queue.json");
        let db = QueueStore::open(&json, |_| {}).unwrap();
        let stale = QueueStore::open(&json, |_| {}).unwrap();
        let first = task("first");
        db.save(std::slice::from_ref(&first)).unwrap();
        assert!(stale.save(&[]).is_err());
        assert!(db.save(&[first.clone(), first.clone()]).is_err());
        assert_eq!(db.load().unwrap()[0].id, first.id);
    }
    #[test]
    fn corrupt_database_and_future_schema_fail_closed() {
        let dir = tempfile::tempdir().unwrap();
        let json = dir.path().join("queue.json");
        let path = json.with_extension("sqlite3");
        fs::write(&path, b"not sqlite").unwrap();
        assert!(QueueStore::open(&json, |_| {}).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"not sqlite");
        fs::remove_file(&path).unwrap();
        let db = QueueStore::open(&json, |_| {}).unwrap();
        db.state
            .lock()
            .unwrap()
            .connection
            .pragma_update(None, "user_version", SCHEMA + 1)
            .unwrap();
        drop(db);
        assert!(QueueStore::open(&json, |_| {}).is_err());
    }
    #[test]
    fn invalid_legacy_json_is_not_overwritten_or_silently_imported() {
        let dir = tempfile::tempdir().unwrap();
        let json = dir.path().join("queue.json");
        fs::write(&json, b"[{broken").unwrap();
        assert!(QueueStore::open(&json, |_| {}).is_err());
        assert_eq!(fs::read(&json).unwrap(), b"[{broken");
    }
    #[test]
    fn backups_include_committed_wal_and_exports_cannot_replace_live_database() {
        let dir = tempfile::tempdir().unwrap();
        let json = dir.path().join("queue.json");
        let db = QueueStore::open(&json, |_| {}).unwrap();
        let first = task("first");
        db.save(std::slice::from_ref(&first)).unwrap();
        assert!(db.export_json(db.path()).is_err());
        let copy = dir.path().join("copy.sqlite3");
        QueueStore::copy_database(db.path(), &copy).unwrap();
        let restored = QueueStore::open(&dir.path().join("copy.json"), |_| {}).unwrap();
        assert_eq!(restored.load().unwrap()[0].id, first.id);
    }
    #[test]
    fn text_and_json_exports_import_only_valid_unique_sources() {
        let first = task("first");
        let json = serde_json::to_string(&vec![first.clone(), first]).unwrap();
        assert_eq!(
            import_sources(&json).unwrap(),
            vec!["https://example.test/first"]
        );
        assert_eq!(
            import_sources("https://example.test/a,https://example.test/a\nfile:///etc/passwd")
                .unwrap(),
            vec!["https://example.test/a"]
        );
        assert!(import_sources("[broken").is_err());
    }
}
