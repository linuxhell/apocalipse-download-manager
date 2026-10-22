//! What the user chooses to share on ED2K, as in aMule's "Shared files": folders (with or
//! without subfolders) and single files. Nothing is shared until it is added here; hidden files
//! and the application's own data are never offered.
use super::{save_file, Ed2kState};
use nibiru_xtreme::ed2k::{CachedShare, ShareRegistry, UploadConfig, UploadManager};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};
use tauri::{Manager, State};

const DEFAULT_SLOTS: usize = 3;
const MAX_SLOTS: usize = 20;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ShareFolder {
    pub path: String,
    pub recursive: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ShareConfig {
    pub folders: Vec<ShareFolder>,
    pub files: Vec<String>,
    /// Files inside a shared folder that the user chose not to share.
    pub excluded: Vec<String>,
    /// Upload priority (0 very low … 4 release) by hash.
    pub priorities: HashMap<String, u8>,
    /// Upload limit in KiB/s; 0 is unlimited.
    pub upload_kib: u32,
    pub slots: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct Progress {
    pub done: usize,
    pub total: usize,
    pub current: String,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Published {
    pub files: usize,
    pub error: Option<String>,
}

pub struct Shares {
    pub registry: ShareRegistry,
    pub uploads: UploadManager,
    config: Mutex<ShareConfig>,
    config_path: PathBuf,
    cache_path: PathBuf,
    app_data: PathBuf,
    loaded: AtomicBool,
    /// Set by every change; the background run repeats until it is clear.
    dirty: AtomicBool,
    running: AtomicBool,
    progress: Mutex<Option<Progress>>,
    published: Mutex<Published>,
    last_error: Mutex<Option<String>>,
}

fn upload_config(config: &ShareConfig) -> UploadConfig {
    UploadConfig {
        max_active: if config.slots == 0 {
            DEFAULT_SLOTS
        } else {
            config.slots.clamp(1, MAX_SLOTS)
        },
        bytes_per_second: (config.upload_kib > 0).then(|| u64::from(config.upload_kib) * 1024),
        ..UploadConfig::default()
    }
}

pub(super) fn parse_hash(hex: &str) -> Option<[u8; 16]> {
    if hex.len() != 32 || !hex.is_ascii() {
        return None;
    }
    let mut hash = [0_u8; 16];
    for (index, byte) in hash.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(hash)
}

fn key(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// Whether `file`, found while scanning `root`, may be offered: hidden files or folders (any path
/// component starting with a dot), the application's own data and explicit exclusions never are.
pub fn eligible(file: &Path, root: &Path, app_data: &Path, excluded: &[String]) -> bool {
    let Ok(relative) = file.strip_prefix(root) else {
        return false;
    };
    if relative
        .components()
        .any(|part| part.as_os_str().to_string_lossy().starts_with('.'))
    {
        return false;
    }
    if file.starts_with(app_data) {
        return false;
    }
    !excluded.iter().any(|path| path == &key(file))
}

/// A folder that is a filesystem root, or that contains the application's data, is refused.
pub fn folder_allowed(folder: &Path, app_data: &Path) -> Result<(), String> {
    if folder.parent().is_none() || app_data.starts_with(folder) {
        return Err("share_folder_too_broad".into());
    }
    Ok(())
}

impl Shares {
    pub fn new(root: &Path) -> Self {
        let config_path = root.join("shares.json");
        let config: ShareConfig = std::fs::read(&config_path)
            .ok()
            .filter(|bytes| bytes.len() <= 4 << 20)
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        let uploads = UploadManager::new(upload_config(&config));
        Self {
            registry: ShareRegistry::default(),
            uploads,
            config: Mutex::new(config),
            config_path,
            cache_path: root.join("share-cache.json"),
            app_data: root
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| root.to_path_buf()),
            loaded: AtomicBool::new(false),
            dirty: AtomicBool::new(false),
            running: AtomicBool::new(false),
            progress: Mutex::new(None),
            published: Mutex::new(Published::default()),
            last_error: Mutex::new(None),
        }
    }

    fn app_data(&self) -> PathBuf {
        std::fs::canonicalize(&self.app_data).unwrap_or_else(|_| self.app_data.clone())
    }

    fn config(&self) -> ShareConfig {
        self.config
            .lock()
            .map(|config| config.clone())
            .unwrap_or_default()
    }

    fn edit<T>(&self, change: impl FnOnce(&mut ShareConfig) -> T) -> Result<T, String> {
        let mut config = self.config.lock().map_err(|e| e.to_string())?;
        let result = change(&mut config);
        let bytes = serde_json::to_vec_pretty(&*config).map_err(|e| e.to_string())?;
        save_file(&self.config_path, &bytes)?;
        Ok(result)
    }

    fn cache(&self) -> Vec<CachedShare> {
        std::fs::read(&self.cache_path)
            .ok()
            .filter(|bytes| bytes.len() <= 64 << 20)
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    fn save_cache(&self) {
        if let Ok(bytes) = serde_json::to_vec(&self.registry.export_cache()) {
            let _ = save_file(&self.cache_path, &bytes);
        }
    }

    fn apply_priorities(&self) {
        let priorities = self.config().priorities;
        for (hash, priority) in priorities {
            if let Some(hash) = parse_hash(&hash) {
                let _ = self.registry.set_priority(hash, priority);
            }
        }
    }

    /// Every file the current choices call for, found without hashing.
    async fn wanted_files(&self) -> Vec<PathBuf> {
        let config = self.config();
        let app_data = self.app_data();
        let mut found: Vec<PathBuf> = Vec::new();
        for folder in &config.folders {
            let Ok(root) = std::fs::canonicalize(&folder.path) else {
                continue;
            };
            let Ok(files) = ShareRegistry::scan_directory(root.clone(), folder.recursive).await
            else {
                continue;
            };
            found.extend(
                files
                    .into_iter()
                    .filter(|file| eligible(file, &root, &app_data, &config.excluded)),
            );
        }
        for file in &config.files {
            if let Ok(path) = std::fs::canonicalize(file) {
                found.push(path);
            }
        }
        found.sort();
        found.dedup();
        found
    }

    /// Makes the registry match the choices: hashes what is new (reusing stored hashes for
    /// unchanged files), drops what is no longer wanted and restores priorities.
    async fn sync(&self) {
        let wanted = self.wanted_files().await;
        let cache = self.cache();
        let keep: Vec<String> = wanted.iter().map(|path| key(path)).collect();
        if let Ok(current) = self.registry.list() {
            for info in current {
                if !keep.contains(&info.path) {
                    if let Some(hash) = parse_hash(&info.hash) {
                        let _ = self.registry.remove(hash);
                    }
                }
            }
        }
        let known: Vec<String> = self
            .registry
            .list()
            .map(|files| files.into_iter().map(|info| info.path).collect())
            .unwrap_or_default();
        let pending: Vec<PathBuf> = wanted
            .into_iter()
            .filter(|path| !known.contains(&key(path)))
            .collect();
        let total = pending.len();
        let mut failure = None;
        for (index, path) in pending.into_iter().enumerate() {
            if let Ok(mut progress) = self.progress.lock() {
                *progress = Some(Progress {
                    done: index,
                    total,
                    current: path
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                });
            }
            if let Err(error) = self.registry.add_file_cached(path, &cache).await {
                failure = Some(error.to_string());
            }
        }
        if let Ok(mut progress) = self.progress.lock() {
            *progress = None;
        }
        if let Ok(mut last) = self.last_error.lock() {
            *last = failure;
        }
        self.apply_priorities();
        self.save_cache();
    }
}

/// Runs `sync` in the background, one run at a time, then publishes to the connected server.
/// Callers set `dirty` first; a change that arrives while a run is busy is picked up by that run.
fn spawn_sync(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let state = app.state::<Ed2kState>();
        if state.shares.running.swap(true, Ordering::AcqRel) {
            return;
        }
        loop {
            state.shares.dirty.store(false, Ordering::Release);
            state.shares.sync().await;
            if !state.shares.dirty.load(Ordering::Acquire) {
                break;
            }
        }
        state.shares.running.store(false, Ordering::Release);
        if state.shares.dirty.load(Ordering::Acquire) {
            // A change landed between the last check and releasing the lock.
            spawn_sync(app.clone());
        }
        super::publish_shares(&state).await;
    });
}

fn request_sync(app: tauri::AppHandle, state: &Ed2kState) {
    state.shares.dirty.store(true, Ordering::Release);
    spawn_sync(app);
}

/// Loads the saved choices once, the first time the ED2K window asks.
pub fn ensure_loaded(app: &tauri::AppHandle, state: &Ed2kState) {
    if state.shares.loaded.swap(true, Ordering::AcqRel) {
        return;
    }
    let config = state.shares.config();
    if !config.folders.is_empty() || !config.files.is_empty() {
        request_sync(app.clone(), state);
    }
}

pub fn overview(state: &Ed2kState) -> serde_json::Value {
    let shares = &state.shares;
    let config = shares.config();
    let snapshot = shares.uploads.snapshot();
    let files: Vec<serde_json::Value> = shares
        .registry
        .list()
        .unwrap_or_default()
        .into_iter()
        .map(|info| {
            let stats = snapshot.files.iter().find(|stats| stats.hash == info.hash);
            let explicit = config.files.iter().any(|path| path == &info.path);
            serde_json::json!({
                "path": info.path, "name": info.name, "size": info.size, "hash": info.hash,
                "link": info.link, "priority": info.priority, "explicit": explicit,
                "requests": stats.map_or(0, |s| s.requests),
                "accepted": stats.map_or(0, |s| s.accepted),
                "uploaded": stats.map_or(0, |s| s.uploaded),
            })
        })
        .collect();
    serde_json::json!({
        "folders": config.folders, "files": files,
        "upload_kib": config.upload_kib,
        "slots": if config.slots == 0 { DEFAULT_SLOTS } else { config.slots },
        "uploads": snapshot,
        "work": shares.progress.lock().ok().and_then(|p| p.clone()),
        "published": shares.published.lock().map(|p| p.clone()).unwrap_or_default(),
        "error": shares.last_error.lock().ok().and_then(|e| e.clone()),
    })
}

pub fn set_published(state: &Ed2kState, files: usize, error: Option<String>) {
    if let Ok(mut published) = state.shares.published.lock() {
        *published = Published { files, error };
    }
}

fn canonical_folder(state: &Ed2kState, path: &Path) -> Result<PathBuf, String> {
    let root = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
    if !std::fs::metadata(&root)
        .map(|m| m.is_dir())
        .unwrap_or(false)
    {
        return Err("share_not_a_folder".into());
    }
    folder_allowed(&root, &state.shares.app_data())?;
    Ok(root)
}

#[tauri::command]
pub async fn ed2k_share_add_folder(
    app: tauri::AppHandle,
    state: State<'_, Ed2kState>,
    recursive: bool,
) -> Result<Option<String>, String> {
    state.ready()?;
    let picked = tauri::async_runtime::spawn_blocking(|| rfd::FileDialog::new().pick_folder())
        .await
        .map_err(|e| e.to_string())?;
    let Some(path) = picked else {
        return Ok(None);
    };
    let root = canonical_folder(&state, &path)?;
    let folder = ShareFolder {
        path: key(&root),
        recursive,
    };
    state.shares.edit(|config| {
        config
            .folders
            .retain(|existing| existing.path != folder.path);
        config.folders.push(folder.clone());
        config
            .excluded
            .retain(|file| !Path::new(file).starts_with(&root));
    })?;
    request_sync(app, &state);
    Ok(Some(folder.path))
}

#[tauri::command]
pub async fn ed2k_share_add_files(
    app: tauri::AppHandle,
    state: State<'_, Ed2kState>,
) -> Result<usize, String> {
    state.ready()?;
    let picked = tauri::async_runtime::spawn_blocking(|| rfd::FileDialog::new().pick_files())
        .await
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let mut added = 0;
    for path in picked {
        let Ok(path) = std::fs::canonicalize(path) else {
            continue;
        };
        let metadata = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.len() == 0 || path.starts_with(state.shares.app_data()) {
            continue;
        }
        let name = key(&path);
        state.shares.edit(|config| {
            if !config.files.contains(&name) {
                config.files.push(name.clone());
            }
            config.excluded.retain(|file| file != &name);
        })?;
        added += 1;
    }
    if added > 0 {
        request_sync(app, &state);
    }
    Ok(added)
}

#[tauri::command]
pub async fn ed2k_share_remove_folder(
    app: tauri::AppHandle,
    state: State<'_, Ed2kState>,
    path: String,
) -> Result<(), String> {
    state.ready()?;
    state
        .shares
        .edit(|config| config.folders.retain(|folder| folder.path != path))?;
    request_sync(app, &state);
    Ok(())
}

/// Stops sharing one file: a single file is forgotten, a file found through a folder is excluded.
#[tauri::command]
pub async fn ed2k_share_remove_file(
    app: tauri::AppHandle,
    state: State<'_, Ed2kState>,
    hash: String,
) -> Result<(), String> {
    state.ready()?;
    let hash_bytes = parse_hash(&hash).ok_or("invalid_share_hash")?;
    let info = state
        .shares
        .registry
        .list()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|info| info.hash == hash)
        .ok_or("share_not_found")?;
    state.shares.edit(|config| {
        if config.files.contains(&info.path) {
            config.files.retain(|path| path != &info.path);
        }
        // Still covered by a folder? Then it must be excluded explicitly.
        if config
            .folders
            .iter()
            .any(|folder| Path::new(&info.path).starts_with(&folder.path))
            && !config.excluded.contains(&info.path)
        {
            config.excluded.push(info.path.clone());
        }
        config.priorities.remove(&hash);
    })?;
    state
        .shares
        .registry
        .remove(hash_bytes)
        .map_err(|e| e.to_string())?;
    state.shares.save_cache();
    request_sync(app, &state);
    Ok(())
}

#[tauri::command]
pub async fn ed2k_share_set_priority(
    state: State<'_, Ed2kState>,
    hash: String,
    priority: u8,
) -> Result<(), String> {
    state.ready()?;
    let hash_bytes = parse_hash(&hash).ok_or("invalid_share_hash")?;
    if priority > 4 {
        return Err("invalid_share_priority".into());
    }
    state
        .shares
        .registry
        .set_priority(hash_bytes, priority)
        .map_err(|e| e.to_string())?;
    state.shares.edit(|config| {
        config.priorities.insert(hash, priority);
    })
}

#[tauri::command]
pub async fn ed2k_share_rescan(
    app: tauri::AppHandle,
    state: State<'_, Ed2kState>,
) -> Result<(), String> {
    state.ready()?;
    // Files changed on disk are hashed again by the registry; new ones in folders are found by sync.
    let _ = state.shares.registry.rescan().await;
    request_sync(app, &state);
    Ok(())
}

#[tauri::command]
pub async fn ed2k_share_set_limits(
    state: State<'_, Ed2kState>,
    upload_kib: u32,
    slots: usize,
) -> Result<(), String> {
    state.ready()?;
    if upload_kib > 10_000_000 || !(1..=MAX_SLOTS).contains(&slots) {
        return Err("invalid_upload_limits".into());
    }
    state.shares.edit(|config| {
        config.upload_kib = upload_kib;
        config.slots = slots;
    })?;
    state
        .shares
        .uploads
        .set_config(upload_config(&state.shares.config()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, bytes: &[u8]) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }

    #[test]
    fn hidden_files_app_data_and_exclusions_are_never_offered() {
        let root = Path::new("/share");
        let data = Path::new("/share/appdata");
        let excluded = vec!["/share/skip.bin".to_owned()];
        assert!(eligible(Path::new("/share/a/b.iso"), root, data, &excluded));
        assert!(!eligible(
            Path::new("/share/.secret/b.iso"),
            root,
            data,
            &excluded
        ));
        assert!(!eligible(
            Path::new("/share/a/.hidden"),
            root,
            data,
            &excluded
        ));
        assert!(!eligible(
            Path::new("/share/appdata/settings.json"),
            root,
            data,
            &excluded
        ));
        assert!(!eligible(
            Path::new("/share/skip.bin"),
            root,
            data,
            &excluded
        ));
        assert!(!eligible(
            Path::new("/elsewhere/x.iso"),
            root,
            data,
            &excluded
        ));
    }

    #[test]
    fn a_folder_holding_the_application_data_or_a_filesystem_root_is_refused() {
        let data = Path::new("/home/u/.local/share/app");
        assert!(folder_allowed(Path::new("/home/u/Videos"), data).is_ok());
        assert!(folder_allowed(Path::new("/home/u"), data).is_err());
        assert!(folder_allowed(Path::new("/"), data).is_err());
    }

    #[test]
    fn hashes_round_trip_and_garbage_is_rejected() {
        let hash = parse_hash("0123456789abcdef0123456789abcdef").unwrap();
        assert_eq!(hash[0], 0x01);
        assert_eq!(hash[15], 0xef);
        assert!(parse_hash("short").is_none());
        assert!(parse_hash("zz23456789abcdef0123456789abcdef").is_none());
        assert!(parse_hash("0123456789abcdef0123456789abcdeé").is_none());
    }

    #[test]
    fn choices_priorities_and_limits_survive_a_restart() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("app/ed2k-kad")).unwrap();
        let first = Shares::new(&root.path().join("app/ed2k-kad"));
        first
            .edit(|config| {
                config.folders.push(ShareFolder {
                    path: "/media/videos".into(),
                    recursive: true,
                });
                config.files.push("/media/one.iso".into());
                config.priorities.insert("ab".repeat(16), 4);
                config.upload_kib = 512;
                config.slots = 5;
            })
            .unwrap();
        let second = Shares::new(&root.path().join("app/ed2k-kad"));
        let config = second.config();
        assert_eq!(config.folders.len(), 1);
        assert!(config.folders[0].recursive);
        assert_eq!(config.files, vec!["/media/one.iso".to_owned()]);
        assert_eq!(config.priorities.get(&"ab".repeat(16)), Some(&4));
        let limits = second.uploads.config();
        assert_eq!(
            (limits.max_active, limits.bytes_per_second),
            (5, Some(512 * 1024))
        );
    }

    #[tokio::test]
    async fn sync_shares_only_the_chosen_visible_files_and_follows_changes() {
        let root = tempfile::tempdir().unwrap();
        let data = root.path().join("app/ed2k-kad");
        std::fs::create_dir_all(&data).unwrap();
        let shared = root.path().join("shared");
        write(&shared.join("movie.mkv"), &vec![7u8; 300_000]);
        write(&shared.join("sub/song.mp3"), &vec![9u8; 200_000]);
        write(&shared.join(".hidden/private.txt"), b"private");
        write(&shared.join("empty.bin"), b"");
        let single = root.path().join("single.iso");
        write(&single, &vec![3u8; 100_000]);

        let shares = Shares::new(&data);
        let shared_key = key(&std::fs::canonicalize(&shared).unwrap());
        shares
            .edit(|config| {
                config.folders.push(ShareFolder {
                    path: shared_key.clone(),
                    recursive: false,
                });
                config
                    .files
                    .push(key(&std::fs::canonicalize(&single).unwrap()));
            })
            .unwrap();
        shares.sync().await;
        let names = |shares: &Shares| -> Vec<String> {
            shares
                .registry
                .list()
                .unwrap()
                .into_iter()
                .map(|i| i.name)
                .collect()
        };
        assert_eq!(
            names(&shares),
            vec!["movie.mkv".to_owned(), "single.iso".to_owned()],
            "no subfolder, hidden or empty file, and nothing that was not chosen"
        );

        shares
            .edit(|config| config.folders[0].recursive = true)
            .unwrap();
        shares.sync().await;
        assert_eq!(
            names(&shares),
            vec![
                "movie.mkv".to_owned(),
                "single.iso".to_owned(),
                "song.mp3".to_owned()
            ]
        );

        let song = key(&std::fs::canonicalize(shared.join("sub/song.mp3")).unwrap());
        shares.edit(|config| config.excluded.push(song)).unwrap();
        shares.sync().await;
        assert!(
            !names(&shares).contains(&"song.mp3".to_owned()),
            "an excluded file is dropped"
        );

        shares.edit(|config| config.folders.clear()).unwrap();
        shares.sync().await;
        assert_eq!(
            names(&shares),
            vec!["single.iso".to_owned()],
            "removing a folder stops sharing its files"
        );

        let hash = shares.registry.list().unwrap()[0].hash.clone();
        shares
            .edit(|config| {
                config.priorities.insert(hash.clone(), 4);
            })
            .unwrap();
        shares.apply_priorities();
        assert_eq!(shares.registry.list().unwrap()[0].priority, 4);
        assert!(
            shares.cache_path.exists(),
            "hashes are kept so a restart does not hash again"
        );
    }

    #[tokio::test]
    async fn a_chosen_file_is_served_to_another_client_and_an_unchosen_one_is_not() {
        use nibiru_xtreme::ed2k::{self, FileLink, FileSource};
        use std::sync::{atomic::AtomicBool, Arc};
        let root = tempfile::tempdir().unwrap();
        let data = root.path().join("app/ed2k-kad");
        let state = Ed2kState::new(data).unwrap();
        let shared = root.path().join("shared");
        let bytes: Vec<u8> = (0..300_000u32).map(|i| (i % 251) as u8).collect();
        write(&shared.join("chosen.bin"), &bytes);
        write(&shared.join(".hidden/secret.bin"), &vec![5u8; 100_000]);
        state
            .shares
            .edit(|config| {
                config.folders.push(ShareFolder {
                    path: key(&std::fs::canonicalize(&shared).unwrap()),
                    recursive: true,
                })
            })
            .unwrap();
        state.shares.sync().await;
        let port = state.incoming_port().await.unwrap();
        let source = FileSource {
            client_id: 0x0100007f,
            host: Some(std::net::Ipv4Addr::LOCALHOST),
            port,
            requires_callback: false,
            user_hash: None,
            crypt: 0,
            buddy: None,
        };
        let link = FileLink::parse(&state.shares.registry.list().unwrap()[0].link).unwrap();
        let out = root.path().join("downloaded");
        std::fs::create_dir_all(&out).unwrap();
        let got = ed2k::download_from_sources(
            &link,
            &out,
            std::slice::from_ref(&source),
            [9; 16],
            0x0100007f,
            Arc::new(AtomicBool::new(false)),
            |_| {},
        )
        .await
        .unwrap();
        assert_eq!(std::fs::read(got).unwrap(), bytes);
        assert_eq!(state.shares.uploads.snapshot().total_uploaded, 300_000);

        // A file that exists on disk but was never chosen is not offered to anyone.
        let hidden = ed2k::hash_shared_file(&shared.join(".hidden/secret.bin")).unwrap();
        let out = root.path().join("refused");
        std::fs::create_dir_all(&out).unwrap();
        let refused = ed2k::download_from_sources(
            &hidden.link,
            &out,
            &[source],
            [9; 16],
            0x0100007f,
            Arc::new(AtomicBool::new(false)),
            |_| {},
        )
        .await;
        assert!(refused.is_err(), "an unchosen file must not be served");
        assert!(std::fs::read_dir(&out).unwrap().all(|e| {
            std::fs::metadata(e.unwrap().path()).map_or(true, |m| m.len() != 100_000)
        }));
    }
}
