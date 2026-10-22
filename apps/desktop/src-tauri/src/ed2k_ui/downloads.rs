use super::{
    retry::{self, Failure, Outcome},
    save_file, Ed2kState,
};
use nibiru_xtreme::ed2k::{self, FileLink, TransferStage};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tauri::{Manager, State};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Download {
    pub id: String,
    pub file: FileLink,
    pub directory: PathBuf,
    pub status: String,
    pub verified: u64,
    #[serde(skip)]
    verified_parts: Vec<bool>,
    #[serde(default, skip_deserializing)]
    pub preview_ready: bool,
    pub peer: Option<String>,
    pub queue_rank: Option<u32>,
    pub error: Option<String>,
    /// Why the download is waiting for its next pass (a short code the window translates).
    #[serde(default, skip_deserializing)]
    pub stall: Option<String>,
    /// Passes that ended without finishing since the task was started.
    #[serde(default, skip_deserializing)]
    pub attempts: u32,
    /// When the next pass starts, in Unix seconds.
    #[serde(default, skip_deserializing)]
    pub retry_at: Option<u64>,
    /// Bytes of parts being received that are not verified yet.
    #[serde(default, skip_deserializing)]
    pub in_flight: u64,
}
impl Download {
    fn apply_progress(&mut self, progress: ed2k::TransferProgress) {
        self.status = match progress.stage {
            TransferStage::Connecting => "connecting",
            TransferStage::Queued => "waiting_peer",
            TransferStage::Checking => "checking",
            TransferStage::Downloading => "downloading",
            TransferStage::Completed => "completed",
        }
        .into();
        // An automatic retry re-hashes the same partial file. Its initial Checking event
        // has an empty map; it does not invalidate parts authenticated in this process.
        // The next non-Checking event publishes the full result, including invalidated parts.
        // Explicit resume and application restart still begin with an empty map.
        if !matches!(progress.stage, TransferStage::Checking) {
            self.verified_parts = progress.verified_parts;
        }
        let unchecked = progress.verified == 0
            && matches!(
                progress.stage,
                TransferStage::Connecting | TransferStage::Checking
            );
        if !unchecked {
            self.verified = progress.verified;
        }
        self.in_flight = progress.in_flight;
        self.peer = progress.peer;
        self.queue_rank = progress.queue_rank;
    }
}

struct Inner {
    path: PathBuf,
    tasks: Mutex<BTreeMap<String, Download>>,
    active: Mutex<BTreeMap<String, Arc<AtomicBool>>>,
    slots: Arc<tokio::sync::Semaphore>,
    /// Wakes downloads that are waiting for sources (a server or Kad connected, or "retry now").
    wake: tokio::sync::Notify,
}
#[derive(Clone)]
pub struct Downloads(Arc<Inner>);
impl Downloads {
    pub fn empty(path: PathBuf) -> Self {
        Self(Arc::new(Inner {
            path,
            tasks: Mutex::new(BTreeMap::new()),
            active: Mutex::new(BTreeMap::new()),
            slots: Arc::new(tokio::sync::Semaphore::new(2)),
            wake: tokio::sync::Notify::new(),
        }))
    }
    /// Starts the next pass of every download that is waiting for sources.
    pub fn wake(&self) {
        self.0.wake.notify_waiters();
    }
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let queue = Self::empty(path);
        match std::fs::metadata(&queue.0.path) {
            Ok(metadata) => {
                if metadata.len() > 1024 * 1024 {
                    return Err("ed2k_download_queue_too_large".into());
                }
                let bytes = std::fs::read(&queue.0.path).map_err(|e| e.to_string())?;
                let tasks: Vec<Download> =
                    serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                if tasks.len() > 200 {
                    return Err("ed2k_download_queue_full".into());
                }
                let mut stored = queue.0.tasks.lock().map_err(|e| e.to_string())?;
                for mut task in tasks {
                    uuid::Uuid::parse_str(&task.id).map_err(|e| e.to_string())?;
                    if FileLink::parse(&task.file.link()).map_err(|e| e.to_string())? != task.file
                        || !task.directory.is_absolute()
                        || task.verified > task.file.size
                    {
                        return Err("invalid_ed2k_download_record".into());
                    }
                    if task.status != "completed" {
                        task.status = "paused".into();
                    }
                    task.peer = None;
                    task.queue_rank = None;
                    if stored.insert(task.id.clone(), task).is_some() {
                        return Err("duplicate_ed2k_task".into());
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
        Ok(queue)
    }
    pub fn list(&self) -> Result<Vec<Download>, String> {
        Ok(self
            .0
            .tasks
            .lock()
            .map_err(|e| e.to_string())?
            .values()
            .cloned()
            .map(|mut task| {
                task.preview_ready =
                    task.status == "completed" || task.verified_parts.first() == Some(&true);
                task
            })
            .collect())
    }
    fn preview_snapshot(&self, id: &str) -> Option<super::preview::Snapshot> {
        let tasks = self.0.tasks.lock().ok()?;
        let task = tasks.get(id)?;
        let complete = task.status == "completed";
        let path = if complete {
            task.directory.join(&task.file.name)
        } else {
            task.directory.join(format!(
                ".nibiru-ed2k-{}-{}.part",
                task.file.hash_hex(),
                task.file.size
            ))
        };
        Some(super::preview::Snapshot {
            path,
            size: task.file.size,
            parts: if complete {
                vec![true; task.file.part_count()]
            } else {
                task.verified_parts.clone()
            },
            part_size: 9_728_000,
        })
    }
    fn persist(&self, tasks: &BTreeMap<String, Download>) -> Result<(), String> {
        let records: Vec<_> = tasks.values().collect();
        save_file(
            &self.0.path,
            &serde_json::to_vec(&records).map_err(|e| e.to_string())?,
        )
    }
    fn update(&self, id: &str, edit: impl FnOnce(&mut Download)) -> Result<(), String> {
        let mut tasks = self.0.tasks.lock().map_err(|e| e.to_string())?;
        let task = tasks.get_mut(id).ok_or("ed2k_task_not_found")?;
        edit(task);
        self.persist(&tasks)
    }
    fn start(&self, app: tauri::AppHandle, id: String) -> Result<(), String> {
        let mut active = self.0.active.lock().map_err(|e| e.to_string())?;
        if active.contains_key(&id) {
            return Err("ed2k_task_still_running".into());
        }
        let task = self
            .0
            .tasks
            .lock()
            .map_err(|e| e.to_string())?
            .get(&id)
            .cloned()
            .ok_or("ed2k_task_not_found")?;
        if task.status == "completed" {
            return Err("ed2k_task_already_completed".into());
        }
        let cancel = Arc::new(AtomicBool::new(false));
        self.update(&id, |task| {
            task.verified_parts.clear();
            task.preview_ready = false;
            task.status = "queued".into();
            task.error = None;
            task.peer = None;
            task.queue_rank = None;
            task.stall = None;
            task.attempts = 0;
            task.retry_at = None;
        })?;
        active.insert(id.clone(), cancel.clone());
        drop(active);
        let queue = self.clone();
        tauri::async_runtime::spawn(async move {
            let log = |level: &str, event: &str, detail: &str| {
                super::super::diagnostic_log(
                    &app.state::<super::super::AppState>(),
                    level,
                    event,
                    detail,
                );
            };
            log(
                "INFO",
                "ed2k.download_started",
                &format!("task={id} verified={}", task.verified),
            );
            let outcome = retry::run(
                |_| attempt_once(&queue, &app, &id, &task, &cancel),
                |pause| wait_or_wake(&queue, &cancel, pause),
                |waiting| {
                    let retry_at = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(0, |now| (now + waiting.wait).as_secs());
                    let _ = queue.update(&id, |task| {
                        task.status = "waiting_sources".into();
                        task.in_flight = 0;
                        task.stall = Some(waiting.code.to_owned());
                        task.attempts = waiting.attempt;
                        task.retry_at = Some(retry_at);
                        task.error = Some(waiting.detail.clone());
                        task.peer = None;
                        task.queue_rank = None;
                    });
                    app.state::<Ed2kState>().events.warn(
                        "download",
                        "waiting",
                        serde_json::json!({
                            "name": task.file.name, "attempt": waiting.attempt,
                            "reason": waiting.code, "next_in_s": waiting.wait.as_secs(),
                            "detail": waiting.detail,
                        }),
                    );
                    log(
                        "WARN",
                        "ed2k.waiting_sources",
                        &format!(
                            "task={id} attempt={} reason={} next_in_s={} detail={}",
                            waiting.attempt,
                            waiting.code,
                            waiting.wait.as_secs(),
                            waiting.detail
                        ),
                    );
                },
                &cancel,
            )
            .await;
            log(
                if matches!(outcome, Outcome::Failed(_)) {
                    "ERROR"
                } else {
                    "INFO"
                },
                "ed2k.download_ended",
                &format!("task={id} outcome={outcome:?}"),
            );
            app.state::<Ed2kState>().events.push(
                match &outcome {
                    Outcome::Failed(_) => "error",
                    _ => "info",
                },
                "download",
                match &outcome {
                    Outcome::Completed => "completed",
                    Outcome::Cancelled => "paused",
                    Outcome::Failed(_) => "failed",
                },
                serde_json::json!({
                    "name": task.file.name,
                    "error": match &outcome { Outcome::Failed(error) => error.as_str(), _ => "" },
                }),
            );
            let _ = queue.update(&id, |task| {
                task.peer = None;
                task.queue_rank = None;
                task.stall = None;
                task.retry_at = None;
                task.in_flight = 0;
                match outcome {
                    Outcome::Completed => {
                        task.status = "completed".into();
                        task.verified = task.file.size;
                        task.error = None;
                    }
                    Outcome::Cancelled => {
                        task.status = "paused".into();
                        task.error = None;
                    }
                    Outcome::Failed(error) => {
                        task.status = "error".into();
                        task.error = Some(error);
                    }
                }
            });
            if let Ok(mut active) = queue.0.active.lock() {
                active.remove(&id);
            }
        });
        Ok(())
    }
}

async fn wait_cancelled(cancel: &AtomicBool) {
    while !cancel.load(Ordering::Acquire) {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

/// Sleeps until the next pass is due, or earlier when a server or Kad connects, the user asks to
/// retry now, or the task is paused.
async fn wait_or_wake(queue: &Downloads, cancel: &AtomicBool, pause: std::time::Duration) {
    let wake = queue.0.wake.notified();
    tokio::pin!(wake);
    wake.as_mut().enable();
    tokio::select! {
        _ = tokio::time::sleep(pause) => {}
        _ = &mut wake => {}
        _ = wait_cancelled(cancel) => {}
    }
}

/// One pass: ask the connected server and Kad for sources, then fetch what is missing. Nothing is
/// thrown away: the engine starts from the verified parts already on disk.
async fn attempt_once(
    queue: &Downloads,
    app: &tauri::AppHandle,
    id: &str,
    task: &Download,
    cancel: &Arc<AtomicBool>,
) -> Result<(), Failure> {
    let _permit = tokio::select! {
        result = queue.0.slots.clone().acquire_owned() => result.map_err(|e| Failure::Fatal(e.to_string()))?,
        _ = wait_cancelled(cancel) => return Err(Failure::Cancelled),
    };
    if cancel.load(Ordering::Acquire) {
        return Err(Failure::Cancelled);
    }
    let state = app.state::<Ed2kState>();
    // Sources come from the connected server and from Kad, so a download also works when only
    // one of them is available.
    let server = state
        .server
        .read()
        .await
        .as_ref()
        .filter(|connection| connection.is_connected())
        .cloned();
    let kad = state.net.kad_session().await;
    if server.is_none() && kad.is_none() {
        return Err(Failure::NotConnected);
    }
    let _ = queue.update(id, |task| {
        task.status = "connecting".into();
        task.error = None;
        task.stall = None;
        task.retry_at = None;
    });
    let before = queue
        .list()
        .ok()
        .and_then(|tasks| tasks.into_iter().find(|t| t.id == id))
        .map_or(0, |t| t.verified);
    let hash = task.file.hash_hex();
    let mut sources: Vec<ed2k::FileSource> = Vec::new();
    let mut error = "ed2k_no_sources".to_owned();
    let mut answered = false;
    if let Some(connection) = &server {
        for attempt in 0..3 {
            if cancel.load(Ordering::Acquire) {
                return Err(Failure::Cancelled);
            }
            let found = tokio::select! {
                result = connection.sources(&hash) => result,
                _ = wait_cancelled(cancel) => return Err(Failure::Cancelled),
            };
            match found {
                Ok(found) => {
                    state.events.info(
                        "download",
                        "server_sources",
                        serde_json::json!({"name": task.file.name, "count": found.len()}),
                    );
                    sources = found;
                    answered = true;
                    break;
                }
                Err(reason) => {
                    state.events.warn(
                        "download",
                        "server_sources_failed",
                        serde_json::json!({
                            "name": task.file.name, "try": attempt + 1, "error": reason.to_string(),
                        }),
                    );
                    error = reason.to_string();
                }
            }
            if attempt < 2 {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
    // Kad takes seconds: it runs while the transfer starts with what the server gave, and what it
    // finds joins the transfer as it arrives.
    let mut late_sources = None;
    let mut kad_task = None;
    if let Some(kad) = kad.clone() {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        late_sources = Some(receiver);
        let (hash_bytes, size, name) = (task.file.hash, task.file.size, task.file.name.clone());
        let app = app.clone();
        kad_task = Some(tauri::async_runtime::spawn(async move {
            let found = tokio::time::timeout(
                std::time::Duration::from_secs(25),
                kad.sources(hash_bytes, size),
            )
            .await;
            let state = app.state::<Ed2kState>();
            match &found {
                Ok(Ok(list)) => state.events.info(
                    "download",
                    "kad_sources",
                    serde_json::json!({"name": name, "count": list.len()}),
                ),
                Ok(Err(reason)) => state.events.warn(
                    "download",
                    "kad_sources_failed",
                    serde_json::json!({"name": name, "error": reason.to_string()}),
                ),
                Err(_) => state.events.warn(
                    "download",
                    "kad_sources_failed",
                    serde_json::json!({"name": name, "error": "timeout"}),
                ),
            }
            if let Ok(Ok(list)) = found {
                for source in list {
                    let _ = sender.send(source);
                }
            }
            // The sender goes away here: the transfer knows no more sources are coming.
        }));
    }
    if !answered && kad.is_none() {
        return Err(Failure::Sources(error));
    }
    tokio::fs::create_dir_all(&task.directory)
        .await
        .map_err(|e| Failure::Fatal(e.to_string()))?;
    // A Low-ID source is reached through a server that gave us a High ID, through Kad, or both.
    let listener = state
        .incoming
        .lock()
        .await
        .as_ref()
        .map(|listener| (listener.callbacks(), listener.status().tcp_port));
    let callbacks = listener.and_then(|(registry, port)| {
        let by_server = server
            .as_ref()
            .filter(|connection| !connection.is_low_id())
            .map(|connection| ed2k::CallbackClient::new(connection.clone(), registry.clone()));
        match (by_server, kad.clone()) {
            (Some(client), Some(kad)) => Some(client.with_kad(kad, port)),
            (Some(client), None) => Some(client),
            (None, Some(kad)) => Some(ed2k::CallbackClient::kad_only(registry, kad, port)),
            (None, None) => None,
        }
    });
    let options = ed2k::TransferOptions {
        identity: state.user_hash,
        client_id: server.as_ref().map_or(0, |connection| connection.client_id),
        callbacks,
        obfuscation: state.net.prefs().obfuscation,
        events: Some(peer_events(app.clone(), task.file.name.clone())),
        late_sources,
    };
    state.events.info(
        "download",
        "attempt",
        serde_json::json!({
            "name": task.file.name, "sources": sources.len(), "verified": before,
            "size": task.file.size, "server": server.is_some(), "kad": kad.is_some(),
        }),
    );
    let result = ed2k::download_with_options(
        &task.file,
        &task.directory,
        &sources,
        options,
        cancel.clone(),
        |progress| {
            let _ = queue.update(id, |task| {
                task.apply_progress(progress);
            });
        },
    )
    .await;
    if let Some(task) = kad_task {
        task.abort();
    }
    match result {
        Ok(_) => Ok(()),
        Err(_) if cancel.load(Ordering::Acquire) => Err(Failure::Cancelled),
        Err(error) => match error.downcast_ref::<ed2k::Stalled>() {
            Some(report) => {
                let after = queue
                    .list()
                    .ok()
                    .and_then(|tasks| tasks.into_iter().find(|t| t.id == id))
                    .map_or(0, |t| t.verified);
                Err(Failure::Stalled {
                    report: report.clone(),
                    progressed: after > before,
                })
            }
            None => Err(Failure::Fatal(format!("{error:#}"))),
        },
    }
}

#[tauri::command]
pub async fn ed2k_add_download(
    app: tauri::AppHandle,
    state: State<'_, Ed2kState>,
    link: String,
    directory: Option<String>,
) -> Result<Option<String>, String> {
    state.ready()?;
    let file = FileLink::parse(link.trim()).map_err(|e| e.to_string())?;
    // Use the directory already chosen by the user in ADM preferences.
    let directory = match directory.filter(|value| !value.trim().is_empty()) {
        Some(value) => {
            let path = PathBuf::from(value);
            if !path.is_absolute() {
                return Err("invalid_download_directory".into());
            }
            std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
            std::fs::canonicalize(path).map_err(|e| e.to_string())?
        }
        None => super::super::configured_download_directory(
            &app,
            &app.state::<super::super::AppState>(),
        )?,
    };
    let id = uuid::Uuid::new_v4().to_string();
    {
        let mut tasks = state.downloads.0.tasks.lock().map_err(|e| e.to_string())?;
        if tasks.len() >= 200 {
            return Err("ed2k_download_queue_full".into());
        }
        if tasks
            .values()
            .any(|task| task.file.hash == file.hash && task.directory == directory)
        {
            return Err("ed2k_download_already_added".into());
        }
        if directory.join(&file.name).exists() {
            return Err("destination_already_exists".into());
        }
        tasks.insert(
            id.clone(),
            Download {
                id: id.clone(),
                file,
                directory,
                status: "paused".into(),
                verified: 0,
                verified_parts: Vec::new(),
                preview_ready: false,
                peer: None,
                queue_rank: None,
                error: None,
                stall: None,
                attempts: 0,
                retry_at: None,
                in_flight: 0,
            },
        );
        if let Err(error) = state.downloads.persist(&tasks) {
            tasks.remove(&id);
            return Err(error);
        }
    }
    // The record is durable before network work starts. If disconnected, retain
    // a paused record so users can connect and resume without losing the link.
    if state
        .server
        .read()
        .await
        .as_ref()
        .is_some_and(|s| s.is_connected())
    {
        state.downloads.start(app, id.clone())?;
    }
    Ok(Some(id))
}
#[tauri::command]
pub async fn ed2k_download_action(
    app: tauri::AppHandle,
    state: State<'_, Ed2kState>,
    id: String,
    action: String,
) -> Result<(), String> {
    state.ready()?;
    match action.as_str() {
        "resume" => state.downloads.start(app, id),
        // Skip the wait: ask for sources and try again now.
        "retry" => {
            state.downloads.wake();
            Ok(())
        }
        "pause" => {
            let active = state.downloads.0.active.lock().map_err(|e| e.to_string())?;
            if let Some(cancel) = active.get(&id) {
                cancel.store(true, Ordering::Release);
                state
                    .downloads
                    .update(&id, |task| task.status = "pausing".into())
            } else {
                state.downloads.update(&id, |task| {
                    if task.status != "completed" {
                        task.status = "paused".into();
                    }
                })
            }
        }
        "remove" => {
            let running = state
                .downloads
                .0
                .active
                .lock()
                .map_err(|e| e.to_string())?
                .contains_key(&id);
            if running {
                return Err("pause_ed2k_task_before_removing".into());
            }
            // An unfinished download leaves its partial file, saved hashes and lock next to the
            // destination; removing the entry removes them too.
            let unfinished = state
                .downloads
                .list()?
                .into_iter()
                .find(|task| task.id == id)
                .filter(|task| task.status != "completed");
            if let Some(task) = unfinished {
                tokio::task::spawn_blocking(move || discard_leftovers(&task))
                    .await
                    .map_err(|e| e.to_string())??;
            }
            let active = state.downloads.0.active.lock().map_err(|e| e.to_string())?;
            if active.contains_key(&id) {
                return Err("pause_ed2k_task_before_removing".into());
            }
            let mut tasks = state.downloads.0.tasks.lock().map_err(|e| e.to_string())?;
            let task = tasks.remove(&id).ok_or("ed2k_task_not_found")?;
            if let Err(error) = state.downloads.persist(&tasks) {
                tasks.insert(id, task);
                return Err(error);
            }
            Ok(())
        }
        _ => Err("invalid_ed2k_download_action".into()),
    }
}

/// Open verified ED2K media with the same player selected in ADM settings.
#[tauri::command]
pub async fn ed2k_preview_download(
    app: tauri::AppHandle,
    state: State<'_, Ed2kState>,
    id: String,
) -> Result<(), String> {
    state.ready()?;
    let task = state
        .downloads
        .list()?
        .into_iter()
        .find(|task| task.id == id)
        .ok_or("ed2k_task_not_found")?;
    if !super::super::is_previewable_video_path(std::path::Path::new(&task.file.name)) {
        return Err("media_not_available".into());
    }
    if !task.preview_ready {
        return Err("waiting_for_verified_ed2k_part".into());
    }
    let app_state = app.state::<super::super::AppState>();
    let player = app_state
        .settings
        .lock()
        .map_err(|e| e.to_string())?
        .media_player_path
        .clone();
    let queue = state.downloads.clone();
    let extension = std::path::Path::new(&task.file.name)
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or("mkv")
        .to_ascii_lowercase();
    let url = super::preview::start(Arc::new(move || queue.preview_snapshot(&id)), &extension)
        .await
        .map_err(|e| e.to_string())?;
    super::super::launch_media_target(player, std::ffi::OsStr::new(&url))
}

/// The files an unfinished download keeps next to its destination: the partial data, the
/// authenticated hashes and the writer lock.
fn leftovers(task: &Download) -> [PathBuf; 3] {
    let stem = format!(".nibiru-ed2k-{}-{}", task.file.hash_hex(), task.file.size);
    ["part", "hashset", "lock"].map(|extension| task.directory.join(format!("{stem}.{extension}")))
}

/// Deletes exactly those files. Anything else in the folder is left alone, and a folder (or link)
/// at one of those names stops the removal instead of being deleted.
fn discard_leftovers(task: &Download) -> Result<(), String> {
    for path in leftovers(task) {
        match std::fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.is_file() => {
                // The engine may still be letting go of the file right after a pause.
                let mut last = None;
                for _ in 0..20 {
                    match std::fs::remove_file(&path) {
                        Ok(()) => {
                            last = None;
                            break;
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                            last = None;
                            break;
                        }
                        Err(error) => {
                            last = Some(error);
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                    }
                }
                if let Some(error) = last {
                    return Err(format!("cannot_remove_ed2k_leftover: {error}"));
                }
            }
            Ok(_) => return Err("ed2k_leftover_is_not_a_file".into()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(())
}

/// Turns what the engine does with each peer into log entries, so a stalled download can be
/// explained afterwards.
fn peer_events(app: tauri::AppHandle, name: String) -> ed2k::EventSink {
    use ed2k::TransferEvent::*;
    use serde_json::json;
    Arc::new(move |event| {
        let state = app.state::<Ed2kState>();
        let events = &state.events;
        let seconds = |ms: u64| ms as f64 / 1000.0;
        match event {
            Connecting { peer } => events.info("peer", "peer_connecting", json!({"name": name, "peer": peer})),
            PeerFailed { peer, kind, error } => events.warn("peer", "peer_failed", json!({"name": name, "peer": peer, "kind": kind, "error": error})),
            PeerReady { peer, offers, wanted } => events.info("peer", "peer_ready", json!({"name": name, "peer": peer, "offers": offers, "wanted": wanted})),
            PeerHasNothingNew { peer } => events.warn("peer", "peer_nothing_new", json!({"name": name, "peer": peer})),
            Queued { peer, rank } => events.info("peer", "peer_queued", json!({"name": name, "peer": peer, "rank": rank})),
            QueueFailed { peer, kind, error } => events.warn("peer", "peer_queue_failed", json!({"name": name, "peer": peer, "kind": kind, "error": error})),
            SlotGranted { peer, waited_ms } => events.info("peer", "peer_slot", json!({"name": name, "peer": peer, "waited_s": seconds(waited_ms)})),
            PartStarted { peer, index, shadow } => events.info("peer", if shadow { "part_helping" } else { "part_started" }, json!({"name": name, "peer": peer, "part": index + 1})),
            PartSlow { peer, index, received, elapsed_ms } => events.warn("peer", "part_slow", json!({"name": name, "peer": peer, "part": index + 1, "received": received, "elapsed_s": seconds(elapsed_ms)})),
            PartDone { peers, peer, index, bytes, elapsed_ms } => events.info("peer", "part_done", json!({"name": name, "peer": peer, "peers": peers, "part": index + 1, "bytes": bytes, "elapsed_s": seconds(elapsed_ms)})),
            PartFailed { peer, index, error, elapsed_ms } => events.warn("peer", "part_failed", json!({"name": name, "peer": peer, "part": index + 1, "error": error, "elapsed_s": seconds(elapsed_ms)})),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn queue_restores_unfinished_tasks_paused_and_keeps_verified_counts() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("downloads.json");
        let queue = Downloads::empty(path.clone());
        let id = uuid::Uuid::new_v4().to_string();
        let file = FileLink::parse("ed2k://|file|test.bin|123|0123456789abcdef0123456789abcdef|/")
            .unwrap();
        let task = Download {
            id: id.clone(),
            file,
            directory: root.path().to_owned(),
            status: "downloading".into(),
            verified: 12,
            verified_parts: vec![true],
            preview_ready: true,
            peer: Some("1.2.3.4".into()),
            queue_rank: None,
            error: None,
            stall: None,
            attempts: 0,
            retry_at: None,
            in_flight: 0,
        };
        // Automatic source retries must not make a playable prefix disappear while rechecking.
        let mut retry = task.clone();
        let progress = |stage, parts| ed2k::TransferProgress {
            stage,
            verified: 0,
            verified_parts: parts,
            total: 123,
            peer: None,
            queue_rank: None,
            in_flight: 0,
        };
        retry.apply_progress(progress(TransferStage::Checking, vec![false]));
        assert_eq!(retry.verified_parts, vec![true]);
        retry.apply_progress(progress(TransferStage::Connecting, vec![false]));
        assert_eq!(
            retry.verified_parts,
            vec![false],
            "a failed recheck must revoke preview access"
        );
        {
            let mut tasks = queue.0.tasks.lock().unwrap();
            tasks.insert(id.clone(), task);
            queue.persist(&tasks).unwrap();
        }
        let loaded = Downloads::load(path).unwrap();
        let tasks = loaded.list().unwrap();
        assert_eq!(tasks[0].status, "paused");
        assert_eq!(tasks[0].verified, 12);
        assert!(tasks[0].peer.is_none());
        assert!(tasks[0].verified_parts.is_empty());
        assert!(!tasks[0].preview_ready);
    }
    #[test]
    fn removing_an_unfinished_download_deletes_only_its_own_leftovers() {
        let root = tempfile::tempdir().unwrap();
        let file = FileLink::parse("ed2k://|file|test.bin|123|0123456789abcdef0123456789abcdef|/")
            .unwrap();
        let task = Download {
            id: "t".into(),
            file,
            directory: root.path().to_owned(),
            status: "paused".into(),
            verified: 0,
            verified_parts: Vec::new(),
            preview_ready: false,
            peer: None,
            queue_rank: None,
            error: None,
            stall: None,
            attempts: 0,
            retry_at: None,
            in_flight: 0,
        };
        let [part, hashset, lock] = leftovers(&task);
        for path in [&part, &hashset, &lock] {
            std::fs::write(path, b"x").unwrap();
        }
        let neighbour = root.path().join(".nibiru-ed2k-other-123.part");
        let finished = root.path().join("test.bin");
        std::fs::write(&neighbour, b"keep").unwrap();
        std::fs::write(&finished, b"keep").unwrap();
        discard_leftovers(&task).unwrap();
        assert!(!part.exists() && !hashset.exists() && !lock.exists());
        assert!(neighbour.exists() && finished.exists());
        discard_leftovers(&task).unwrap();
        std::fs::create_dir(&part).unwrap();
        std::fs::write(part.join("keep"), b"user file").unwrap();
        assert_eq!(
            discard_leftovers(&task).unwrap_err(),
            "ed2k_leftover_is_not_a_file"
        );
        assert!(part.join("keep").exists());
    }
}
