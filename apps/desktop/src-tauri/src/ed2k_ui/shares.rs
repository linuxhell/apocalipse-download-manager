//! What the user chose to share on ED2K, upload limits and statistics. Nothing is shared unless
//! the user picked it: entries are explicit files or folders and are stored in `shared.json`.

use super::{save_file, Ed2kState};
use nibiru_xtreme::ed2k::{CachedShare, ShareRegistry, UploadConfig, UploadManager};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tauri::{Manager, State};

const MAX_ENTRIES: usize = 200;
const MAX_STORE_BYTES: u64 = 1024 * 1024;
const MAX_CACHE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_ERRORS: usize = 20;
/// How often the shared list is offered again to the connected server.
pub const REPUBLISH_EVERY: Duration = Duration::from_secs(15 * 60);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShareEntry {
    pub id: String,
    /// `file` or `directory`.
    pub kind: String,
    pub path: String,
    pub recursive: bool,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Stored {
    entries: Vec<ShareEntry>,
    /// Upload priority per canonical file path (0 very low .. 4 release).
    priorities: BTreeMap<String, u8>,
    upload_slots: usize,
    /// Total upload limit in KiB/s; zero means unlimited.
    upload_kib: u64,
}

impl Default for Stored {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            priorities: BTreeMap::new(),
            upload_slots: 3,
            upload_kib: 0,
        }
    }
}

#[derive(Default)]
struct Scan {
    running: AtomicBool,
    done: AtomicUsize,
    total: AtomicUsize,
    current: Mutex<String>,
    errors: Mutex<Vec<String>>,
}

pub struct Shares {
    dir: PathBuf,
    registry: ShareRegistry,
    uploads: UploadManager,
    stored: Mutex<Stored>,
    scan: Scan,
    started: AtomicBool,
    rebuild_lock: tokio::sync::Mutex<()>,
    published: Mutex<Option<(u64, usize)>>,
}

fn valid_entry(entry: &ShareEntry) -> bool {
    uuid::Uuid::parse_str(&entry.id).is_ok()
        && matches!(entry.kind.as_str(), "file" | "directory")
        && Path::new(&entry.path).is_absolute()
}

impl Shares {
    pub fn load(dir: &Path) -> Arc<Self> {
        let stored = std::fs::metadata(dir.join("shared.json"))
            .ok()
            .filter(|m| m.len() <= MAX_STORE_BYTES)
            .and_then(|_| std::fs::read(dir.join("shared.json")).ok())
            .and_then(|bytes| serde_json::from_slice::<Stored>(&bytes).ok())
            .map(|mut stored| {
                stored.entries.retain(valid_entry);
                stored.entries.truncate(MAX_ENTRIES);
                stored.upload_slots = stored.upload_slots.clamp(1, 64);
                stored.upload_kib = stored.upload_kib.min(1_000_000);
                stored.priorities.retain(|_, p| *p <= 4);
                stored
            })
            .unwrap_or_default();
        let uploads = UploadManager::new(upload_config(&stored));
        Arc::new(Self {
            dir: dir.to_owned(),
            registry: ShareRegistry::default(),
            uploads,
            stored: Mutex::new(stored),
            scan: Scan::default(),
            started: AtomicBool::new(false),
            rebuild_lock: tokio::sync::Mutex::new(()),
            published: Mutex::new(None),
        })
    }

    pub fn registry(&self) -> ShareRegistry {
        self.registry.clone()
    }

    pub fn uploads(&self) -> UploadManager {
        self.uploads.clone()
    }

    fn persist(&self) -> Result<(), String> {
        let stored = self.stored.lock().map_err(|e| e.to_string())?;
        save_file(
            &self.dir.join("shared.json"),
            &serde_json::to_vec(&*stored).map_err(|e| e.to_string())?,
        )
    }

    fn read_cache(&self) -> Vec<CachedShare> {
        let path = self.dir.join("shared-cache.json");
        std::fs::metadata(&path)
            .ok()
            .filter(|m| m.len() <= MAX_CACHE_BYTES)
            .and_then(|_| std::fs::read(&path).ok())
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    fn note_error(&self, message: String) {
        if let Ok(mut errors) = self.scan.errors.lock() {
            if errors.len() < MAX_ERRORS {
                errors.push(message);
            }
        }
    }

    /// Hashes (or restores from cache) everything the user chose, then drops files that are no
    /// longer wanted. Files already registered keep being served while this runs.
    pub async fn rebuild(&self) {
        let _serial = self.rebuild_lock.lock().await;
        self.scan.running.store(true, Ordering::Release);
        self.scan.done.store(0, Ordering::Release);
        self.scan.total.store(0, Ordering::Release);
        if let Ok(mut errors) = self.scan.errors.lock() {
            errors.clear();
        }
        let (entries, priorities) = {
            let stored = self.stored.lock().expect("share store lock");
            (stored.entries.clone(), stored.priorities.clone())
        };
        let mut wanted: BTreeSet<PathBuf> = BTreeSet::new();
        for entry in entries.iter().filter(|e| e.enabled) {
            let path = PathBuf::from(&entry.path);
            if entry.kind == "file" {
                wanted.insert(path);
            } else {
                match ShareRegistry::scan_directory(path, entry.recursive).await {
                    Ok(files) => wanted.extend(files),
                    Err(error) => self.note_error(format!("{}: {error}", entry.path)),
                }
            }
        }
        self.scan.total.store(wanted.len(), Ordering::Release);
        let mut cache = self.read_cache();
        cache.extend(self.registry.export_cache());
        let mut kept: HashMap<String, String> = HashMap::new();
        for path in &wanted {
            if let Ok(mut current) = self.scan.current.lock() {
                *current = path.to_string_lossy().into_owned();
            }
            match self.registry.add_file_cached(path.clone(), &cache).await {
                Ok(info) => {
                    kept.insert(info.path.clone(), info.hash.clone());
                    if let Some(priority) = priorities.get(&info.path) {
                        if let Ok(hash) = parse_hash(&info.hash) {
                            let _ = self.registry.set_priority(hash, *priority);
                        }
                    }
                }
                Err(error) => self.note_error(format!("{}: {error}", path.display())),
            }
            self.scan.done.fetch_add(1, Ordering::AcqRel);
        }
        // Remove files that were unshared, and stale hashes of files that changed.
        if let Ok(listed) = self.registry.list() {
            for info in listed {
                if kept.get(&info.path) != Some(&info.hash) {
                    if let Ok(hash) = parse_hash(&info.hash) {
                        let _ = self.registry.remove(hash);
                    }
                }
            }
        }
        if let Ok(bytes) = serde_json::to_vec(&self.registry.export_cache()) {
            let _ = save_file(&self.dir.join("shared-cache.json"), &bytes);
        }
        if let Ok(mut current) = self.scan.current.lock() {
            current.clear();
        }
        self.scan.running.store(false, Ordering::Release);
    }

    /// Starts the first scan, once, and offers the result to the server when it finishes.
    pub fn ensure_started(self: &Arc<Self>, app: &tauri::AppHandle) {
        if !self.started.swap(true, Ordering::AcqRel) {
            rebuild_in_background(app.clone());
        }
    }

    pub fn add_entry(
        &self,
        kind: &str,
        path: &Path,
        recursive: bool,
    ) -> Result<ShareEntry, String> {
        let canonical = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
        if canonical.parent().is_none() {
            return Err("ed2k_share_root_refused".into());
        }
        let metadata = std::fs::metadata(&canonical).map_err(|e| e.to_string())?;
        match kind {
            "file" if metadata.is_file() => {}
            "directory" if metadata.is_dir() => {}
            _ => return Err("ed2k_share_invalid_path".into()),
        }
        let canonical = canonical.to_string_lossy().into_owned();
        let entry = {
            let mut stored = self.stored.lock().map_err(|e| e.to_string())?;
            if let Some(existing) = stored
                .entries
                .iter_mut()
                .find(|e| e.kind == kind && e.path == canonical)
            {
                existing.recursive = recursive;
                existing.enabled = true;
                existing.clone()
            } else {
                if stored.entries.len() >= MAX_ENTRIES {
                    return Err("ed2k_share_too_many".into());
                }
                let entry = ShareEntry {
                    id: uuid::Uuid::new_v4().to_string(),
                    kind: kind.to_owned(),
                    path: canonical,
                    recursive,
                    enabled: true,
                };
                stored.entries.push(entry.clone());
                entry
            }
        };
        self.persist()?;
        Ok(entry)
    }

    pub fn remove_entry(&self, id: &str) -> Result<(), String> {
        {
            let mut stored = self.stored.lock().map_err(|e| e.to_string())?;
            let before = stored.entries.len();
            stored.entries.retain(|e| e.id != id);
            if stored.entries.len() == before {
                return Err("ed2k_share_not_found".into());
            }
        }
        self.persist()
    }

    pub fn set_enabled(&self, id: &str, enabled: bool) -> Result<(), String> {
        {
            let mut stored = self.stored.lock().map_err(|e| e.to_string())?;
            stored
                .entries
                .iter_mut()
                .find(|e| e.id == id)
                .ok_or("ed2k_share_not_found")?
                .enabled = enabled;
        }
        self.persist()
    }

    pub fn set_priority(&self, hash: &str, priority: u8) -> Result<(), String> {
        if priority > 4 {
            return Err("invalid_priority".into());
        }
        let bytes = parse_hash(hash)?;
        let info = self
            .registry
            .list()
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|f| f.hash == hash)
            .ok_or("ed2k_share_not_found")?;
        self.registry
            .set_priority(bytes, priority)
            .map_err(|e| e.to_string())?;
        self.stored
            .lock()
            .map_err(|e| e.to_string())?
            .priorities
            .insert(info.path, priority);
        self.persist()
    }

    pub fn set_upload_limits(&self, slots: usize, kib: u64) -> Result<(), String> {
        if !(1..=64).contains(&slots) || kib > 1_000_000 {
            return Err("invalid_upload_limits".into());
        }
        let config = {
            let mut stored = self.stored.lock().map_err(|e| e.to_string())?;
            stored.upload_slots = slots;
            stored.upload_kib = kib;
            upload_config(&stored)
        };
        self.uploads.set_config(UploadConfig {
            max_active: config.max_active,
            bytes_per_second: config.bytes_per_second,
            ..self.uploads.config()
        });
        self.persist()
    }

    pub fn file_path(&self, hash: &str) -> Result<PathBuf, String> {
        self.registry
            .list()
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|f| f.hash == hash)
            .map(|f| PathBuf::from(f.path))
            .ok_or_else(|| "ed2k_share_not_found".into())
    }

    pub fn note_published(&self, count: usize) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if let Ok(mut published) = self.published.lock() {
            *published = Some((now, count));
        }
    }

    pub fn clear_published(&self) {
        if let Ok(mut published) = self.published.lock() {
            *published = None;
        }
    }

    pub fn snapshot(&self) -> Result<serde_json::Value, String> {
        let stored = self.stored.lock().map_err(|e| e.to_string())?.clone();
        let files = self.registry.list().map_err(|e| e.to_string())?;
        let uploads = self.uploads.snapshot();
        let stats: HashMap<_, _> = uploads.files.iter().map(|f| (f.hash.clone(), f)).collect();
        let listed: Vec<_> = files
            .iter()
            .map(|f| {
                let s = stats.get(&f.hash);
                serde_json::json!({
                    "name": f.name, "size": f.size, "hash": f.hash, "link": f.link, "path": f.path,
                    "parts": f.parts, "priority": f.priority,
                    "requests": s.map_or(0, |s| s.requests), "accepted": s.map_or(0, |s| s.accepted),
                    "uploaded": s.map_or(0, |s| s.uploaded),
                })
            })
            .collect();
        let entries: Vec<_> = stored
            .entries
            .iter()
            .map(|e| {
                let count = if e.kind == "file" {
                    files.iter().filter(|f| f.path == e.path).count()
                } else {
                    let prefix = Path::new(&e.path);
                    files
                        .iter()
                        .filter(|f| {
                            let parent = Path::new(&f.path).parent();
                            if e.recursive {
                                Path::new(&f.path).starts_with(prefix)
                            } else {
                                parent == Some(prefix)
                            }
                        })
                        .count()
                };
                serde_json::json!({
                    "id": e.id, "kind": e.kind, "path": e.path, "recursive": e.recursive,
                    "enabled": e.enabled, "files": count,
                })
            })
            .collect();
        let published = self
            .published
            .lock()
            .map_err(|e| e.to_string())?
            .map(|(at, count)| serde_json::json!({"at": at, "count": count}));
        Ok(serde_json::json!({
            "entries": entries,
            "files": listed,
            "scan": {
                "running": self.scan.running.load(Ordering::Acquire),
                "done": self.scan.done.load(Ordering::Acquire),
                "total": self.scan.total.load(Ordering::Acquire),
                "current": self.scan.current.lock().map_err(|e| e.to_string())?.clone(),
                "errors": self.scan.errors.lock().map_err(|e| e.to_string())?.clone(),
            },
            "uploads": uploads,
            "settings": {"upload_slots": stored.upload_slots, "upload_kib": stored.upload_kib},
            "published": published,
        }))
    }
}

fn upload_config(stored: &Stored) -> UploadConfig {
    UploadConfig {
        max_active: stored.upload_slots.clamp(1, 64),
        bytes_per_second: (stored.upload_kib > 0).then(|| stored.upload_kib * 1024),
        ..UploadConfig::default()
    }
}

fn parse_hash(value: &str) -> Result<[u8; 16], String> {
    if value.len() != 32 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid_ed2k_hash".into());
    }
    let mut hash = [0u8; 16];
    for (i, byte) in hash.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[i * 2..i * 2 + 2], 16).map_err(|e| e.to_string())?;
    }
    Ok(hash)
}

/// Rebuilds the shared list in the background and then offers it to the connected server.
fn rebuild_in_background(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let state = app.state::<Ed2kState>();
        state.shares.rebuild().await;
        let _ = state.publish_shares().await;
    });
}

#[tauri::command]
pub async fn ed2k_shared_list(
    app: tauri::AppHandle,
    state: State<'_, Ed2kState>,
) -> Result<serde_json::Value, String> {
    state.ready()?;
    state.shares.ensure_started(&app);
    state.shares.snapshot()
}

async fn pick(kind: &'static str) -> Result<Option<PathBuf>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dialog = rfd::FileDialog::new();
        if kind == "file" {
            dialog.pick_file()
        } else {
            dialog.pick_folder()
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ed2k_shared_add(
    app: tauri::AppHandle,
    state: State<'_, Ed2kState>,
    kind: String,
    recursive: bool,
) -> Result<Option<ShareEntry>, String> {
    state.ready()?;
    let kind: &'static str = match kind.as_str() {
        "file" => "file",
        "directory" => "directory",
        _ => return Err("invalid_share_kind".into()),
    };
    let Some(path) = pick(kind).await? else {
        return Ok(None);
    };
    let entry = state.shares.add_entry(kind, &path, recursive)?;
    rebuild_in_background(app);
    Ok(Some(entry))
}

#[tauri::command]
pub async fn ed2k_shared_remove(
    app: tauri::AppHandle,
    state: State<'_, Ed2kState>,
    id: String,
) -> Result<(), String> {
    state.ready()?;
    state.shares.remove_entry(&id)?;
    rebuild_in_background(app);
    Ok(())
}

#[tauri::command]
pub async fn ed2k_shared_set_enabled(
    app: tauri::AppHandle,
    state: State<'_, Ed2kState>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    state.ready()?;
    state.shares.set_enabled(&id, enabled)?;
    rebuild_in_background(app);
    Ok(())
}

#[tauri::command]
pub async fn ed2k_shared_rescan(
    app: tauri::AppHandle,
    state: State<'_, Ed2kState>,
) -> Result<(), String> {
    state.ready()?;
    rebuild_in_background(app);
    Ok(())
}

#[tauri::command]
pub async fn ed2k_shared_set_priority(
    state: State<'_, Ed2kState>,
    hash: String,
    priority: u8,
) -> Result<(), String> {
    state.ready()?;
    state.shares.set_priority(&hash, priority)
}

#[tauri::command]
pub async fn ed2k_upload_settings(
    state: State<'_, Ed2kState>,
    slots: usize,
    kib: u64,
) -> Result<(), String> {
    state.ready()?;
    state.shares.set_upload_limits(slots, kib)
}

/// Opens the folder of a shared file. The path comes from the registry, never from the UI.
#[tauri::command]
pub fn ed2k_shared_open_folder(state: State<'_, Ed2kState>, hash: String) -> Result<(), String> {
    state.ready()?;
    reveal(&state.shares.file_path(&hash)?)
}

fn reveal(file: &Path) -> Result<(), String> {
    use std::process::Command;
    let directory = file.parent().ok_or("download_directory_not_found")?;
    #[cfg(target_os = "windows")]
    let result = Command::new("explorer.exe")
        .arg("/select,")
        .arg(file)
        .spawn();
    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg("-R").arg(file).spawn();
    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open").arg(directory).spawn();
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let result: std::io::Result<std::process::Child> = {
        let _ = directory;
        Err(std::io::ErrorKind::Unsupported.into())
    };
    result.map(|_| ()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shares() -> (tempfile::TempDir, Arc<Shares>) {
        let dir = tempfile::tempdir().unwrap();
        let shares = Shares::load(dir.path());
        (dir, shares)
    }

    #[test]
    fn choices_persist_and_corrupt_stores_are_ignored() {
        let (dir, shares) = shares();
        let data = tempfile::tempdir().unwrap();
        let file = data.path().join("a.bin");
        std::fs::write(&file, b"abc").unwrap();
        let entry = shares.add_entry("file", &file, false).unwrap();
        shares.set_upload_limits(5, 128).unwrap();
        let again = Shares::load(dir.path());
        let stored = again.stored.lock().unwrap();
        assert_eq!(stored.entries, vec![entry]);
        assert_eq!((stored.upload_slots, stored.upload_kib), (5, 128));
        drop(stored);
        assert_eq!(again.uploads.config().max_active, 5);
        assert_eq!(again.uploads.config().bytes_per_second, Some(128 * 1024));

        std::fs::write(dir.path().join("shared.json"), b"{ not json").unwrap();
        assert!(Shares::load(dir.path())
            .stored
            .lock()
            .unwrap()
            .entries
            .is_empty());
    }

    #[test]
    fn entries_are_validated_deduplicated_and_bounded() {
        let (_dir, shares) = shares();
        let data = tempfile::tempdir().unwrap();
        let file = data.path().join("a.bin");
        std::fs::write(&file, b"abc").unwrap();
        // A file cannot be added as a directory, nor the reverse.
        assert!(shares.add_entry("directory", &file, false).is_err());
        assert!(shares.add_entry("file", data.path(), false).is_err());
        assert!(shares
            .add_entry("file", &data.path().join("missing"), false)
            .is_err());
        // The filesystem root is refused outright.
        assert_eq!(
            shares
                .add_entry("directory", Path::new(std::path::MAIN_SEPARATOR_STR), true)
                .unwrap_err(),
            "ed2k_share_root_refused"
        );
        let first = shares.add_entry("directory", data.path(), false).unwrap();
        let second = shares.add_entry("directory", data.path(), true).unwrap();
        assert_eq!(first.id, second.id, "same folder is not added twice");
        assert!(second.recursive);
        shares.set_enabled(&first.id, false).unwrap();
        assert!(!shares.stored.lock().unwrap().entries[0].enabled);
        shares.remove_entry(&first.id).unwrap();
        assert_eq!(
            shares.remove_entry(&first.id).unwrap_err(),
            "ed2k_share_not_found"
        );
    }

    #[test]
    fn invalid_limits_and_hashes_are_rejected() {
        let (_dir, shares) = shares();
        assert!(shares.set_upload_limits(0, 0).is_err());
        assert!(shares.set_upload_limits(65, 0).is_err());
        assert!(shares.set_upload_limits(3, 2_000_000).is_err());
        assert!(shares.set_priority("zz", 2).is_err());
        assert!(shares.set_priority(&"a".repeat(32), 9).is_err());
        assert_eq!(
            shares.set_priority(&"a".repeat(32), 2).unwrap_err(),
            "ed2k_share_not_found"
        );
        assert!(parse_hash("a448017aaf21d8525fc10ae87aa6729d").is_ok());
    }

    #[tokio::test]
    async fn rebuild_hashes_chosen_items_reuses_the_cache_and_follows_removals() {
        let (dir, shares) = shares();
        let data = tempfile::tempdir().unwrap();
        std::fs::create_dir(data.path().join("sub")).unwrap();
        std::fs::write(data.path().join("a.bin"), vec![1u8; 4000]).unwrap();
        std::fs::write(data.path().join("sub").join("b.bin"), vec![2u8; 5000]).unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("never.bin"), b"private").unwrap();

        let entry = shares.add_entry("directory", data.path(), false).unwrap();
        shares.rebuild().await;
        let snapshot = shares.snapshot().unwrap();
        assert_eq!(
            snapshot["files"].as_array().unwrap().len(),
            1,
            "non-recursive: only a.bin"
        );
        assert_eq!(snapshot["entries"][0]["files"], 1);
        let link = snapshot["files"][0]["link"].as_str().unwrap();
        assert!(link.starts_with("ed2k://|file|a%2Ebin|4000|"), "{link}");
        assert!(!snapshot["scan"]["running"].as_bool().unwrap());
        assert!(dir.path().join("shared-cache.json").exists());

        // Make it recursive: the subfolder is now included; nothing outside the folder is.
        shares.add_entry("directory", data.path(), true).unwrap();
        shares.rebuild().await;
        let snapshot = shares.snapshot().unwrap();
        assert_eq!(snapshot["files"].as_array().unwrap().len(), 2);
        assert!(snapshot["files"]
            .as_array()
            .unwrap()
            .iter()
            .all(|f| !f["path"].as_str().unwrap().contains("never")));

        // A second instance restores the same hashes from the cache without error.
        let reloaded = Shares::load(dir.path());
        reloaded.rebuild().await;
        assert_eq!(
            reloaded.snapshot().unwrap()["files"]
                .as_array()
                .unwrap()
                .len(),
            2
        );

        // Priority is remembered per path and survives a rebuild.
        let hash = snapshot["files"][0]["hash"].as_str().unwrap().to_owned();
        shares.set_priority(&hash, 4).unwrap();
        shares.rebuild().await;
        let after = shares.snapshot().unwrap();
        let file = after["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["hash"] == hash.as_str())
            .unwrap();
        assert_eq!(file["priority"], 4);

        // Removing the entry stops sharing everything it covered.
        shares.remove_entry(&entry.id).unwrap();
        shares.rebuild().await;
        assert!(shares.snapshot().unwrap()["files"]
            .as_array()
            .unwrap()
            .is_empty());
        assert!(shares.registry().list().unwrap().is_empty());
    }

    #[tokio::test]
    async fn a_vanished_file_is_reported_without_stopping_the_scan() {
        let (_dir, shares) = shares();
        let data = tempfile::tempdir().unwrap();
        let keep = data.path().join("keep.bin");
        let gone = data.path().join("gone.bin");
        std::fs::write(&keep, b"keep").unwrap();
        std::fs::write(&gone, b"gone").unwrap();
        shares.add_entry("file", &keep, false).unwrap();
        shares.add_entry("file", &gone, false).unwrap();
        std::fs::remove_file(&gone).unwrap();
        shares.rebuild().await;
        let snapshot = shares.snapshot().unwrap();
        assert_eq!(snapshot["files"].as_array().unwrap().len(), 1);
        let errors = snapshot["scan"]["errors"].as_array().unwrap();
        assert_eq!(errors.len(), 1, "the missing file is reported, not fatal");
        assert!(errors[0].as_str().unwrap().contains("gone.bin"));
    }
}
