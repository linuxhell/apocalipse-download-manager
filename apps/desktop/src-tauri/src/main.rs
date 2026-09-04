#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use apocalipse_core::{
    classify_url, partial_path, plan_download, Capabilities, DownloadEngine, DownloadEvent,
    DownloadId, DownloadKind, DownloadRequest, DownloadState, DownloadTask,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io::{Read, Write}, net::{TcpListener, TcpStream}, path::{Path, PathBuf}, process::Command, sync::Mutex, time::{Duration, Instant}};
use tauri::{image::Image, menu::{Menu, MenuItem}, tray::TrayIconBuilder, Manager, State};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tokio::sync::{mpsc, oneshot};

struct AppState {
    queue: Mutex<Vec<DownloadTask>>,
    queue_path: PathBuf,
    workers: Mutex<HashMap<DownloadId, oneshot::Sender<()>>>,
    settings: Mutex<UserSettings>,
    settings_path: PathBuf,
    bridge_last_seen: Mutex<Option<Instant>>,
}

#[derive(Clone, Deserialize, Serialize)]
struct UserSettings {
    download_directory: Option<PathBuf>,
    #[serde(default)]
    capture_clipboard: bool,
    #[serde(default = "default_max_active")]
    max_active_downloads: usize,
    #[serde(default = "default_connections")]
    connections_per_download: usize,
    #[serde(default = "default_bridge_token")]
    bridge_token: String,
    #[serde(default)]
    recent_download_directories: Vec<PathBuf>,
}

const fn default_max_active() -> usize { 3 }
const fn default_connections() -> usize { 8 }
fn default_bridge_token() -> String { uuid::Uuid::new_v4().simple().to_string() }

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            download_directory: None,
            capture_clipboard: false,
            max_active_downloads: default_max_active(),
            connections_per_download: default_connections(),
            bridge_token: default_bridge_token(),
            recent_download_directories: Vec::new(),
        }
    }
}

#[derive(Serialize)]
struct PlanResponse {
    primary: String,
    fallbacks: Vec<String>,
    reason: String,
}

#[derive(Serialize)]
struct AutostartStatus {
    enabled: bool,
}

#[derive(Serialize)]
struct ClipboardStatus {
    enabled: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TransferLimits {
    max_active_downloads: usize,
    connections_per_download: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BridgePairing {
    token: String,
    port: u16,
    connected: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BridgeDownload {
    url: String,
    file_name: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DestinationChoice {
    path: String,
    is_default: bool,
    available: bool,
}

const BRIDGE_PORT: u16 = 17654;

#[tauri::command]
fn inspect_url(url: String) -> Result<PlanResponse, String> {
    let capabilities = Capabilities {
        aria2: true,
        yt_dlp: true,
        n_m3u8dl_re: true,
        torrent: true,
        amule: false,
    };
    let plan = plan_download(&url, capabilities).ok_or_else(|| "unsupported_url".to_owned())?;
    Ok(PlanResponse {
        primary: format!("{:?}", plan.primary),
        fallbacks: plan.fallbacks.iter().map(|engine| format!("{engine:?}")).collect(),
        reason: plan.reason.to_owned(),
    })
}

fn load_queue(path: &Path) -> Vec<DownloadTask> {
    fs::read(path).ok().and_then(|data| serde_json::from_slice(&data).ok()).unwrap_or_default()
}

fn save_queue(state: &AppState, queue: &[DownloadTask]) -> Result<(), String> {
    if let Some(parent) = state.queue_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let data = serde_json::to_vec_pretty(queue).map_err(|error| error.to_string())?;
    fs::write(&state.queue_path, data).map_err(|error| error.to_string())
}

fn load_settings(path: &Path) -> UserSettings {
    fs::read(path).ok().and_then(|data| serde_json::from_slice(&data).ok()).unwrap_or_default()
}

fn save_settings(state: &AppState, settings: &UserSettings) -> Result<(), String> {
    if let Some(parent) = state.settings_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let data = serde_json::to_vec_pretty(settings).map_err(|error| error.to_string())?;
    fs::write(&state.settings_path, data).map_err(|error| error.to_string())
}

fn remember_download_directory(state: &AppState, directory: &Path) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.recent_download_directories.retain(|path| path != directory);
    settings.recent_download_directories.insert(0, directory.to_path_buf());
    settings.recent_download_directories.truncate(20);
    save_settings(state, &settings)
}

fn configured_download_directory(app: &tauri::AppHandle, state: &AppState) -> Result<PathBuf, String> {
    if let Some(path) = state.settings.lock().map_err(|error| error.to_string())?.download_directory.clone() {
        return Ok(path);
    }
    app.path().download_dir().map_err(|error| error.to_string())
}

fn update_task(app: &tauri::AppHandle, id: DownloadId, persist: bool, update: impl FnOnce(&mut DownloadTask)) {
    let state = app.state::<AppState>();
    let mut queue = match state.queue.lock() {
        Ok(queue) => queue,
        Err(_) => return,
    };
    if let Some(task) = queue.iter_mut().find(|task| task.id == id) {
        update(task);
        if persist {
            let _ = save_queue(&state, &queue);
        }
    }
}

async fn run_download(
    app: tauri::AppHandle,
    id: DownloadId,
    request: DownloadRequest,
    mut cancellation: oneshot::Receiver<()>,
) {
    update_task(&app, id, true, |task| task.state = DownloadState::Inspecting);
    let engine = match DownloadEngine::new() {
        Ok(engine) => engine,
        Err(error) => {
            update_task(&app, id, true, |task| task.state = DownloadState::Failed { message: error.to_string() });
            return;
        }
    };
    let (events, mut receiver) = mpsc::channel(64);
    let mut download = Box::pin(engine.download(request, events));
    let mut was_cancelled = false;
    loop {
        tokio::select! {
            biased;
            _ = &mut cancellation => {
                was_cancelled = true;
                break;
            },
            result = &mut download => {
                match result {
                    Ok(()) => update_task(&app, id, true, |task| task.state = DownloadState::Completed),
                    Err(error) => update_task(&app, id, true, |task| task.state = DownloadState::Failed { message: error.to_string() }),
                }
                break;
            }
            event = receiver.recv() => match event {
                Some(DownloadEvent::Started { resumed_at, total }) => update_task(&app, id, true, |task| {
                    task.state = DownloadState::Downloading;
                    task.received = resumed_at;
                    task.total = total;
                }),
                Some(DownloadEvent::Progress { received, total }) => update_task(&app, id, false, |task| {
                    task.received = received;
                    task.total = total;
                }),
                Some(DownloadEvent::Completed { bytes }) => update_task(&app, id, true, |task| {
                    task.received = bytes;
                    task.total = Some(bytes);
                    task.state = DownloadState::Completed;
                }),
                None => break,
            }
        }
    }
    if !was_cancelled {
        if let Ok(mut workers) = app.state::<AppState>().workers.lock() {
            workers.remove(&id);
        }
        start_next_queued(&app);
    }
}

async fn run_external_download(
    app: tauri::AppHandle,
    id: DownloadId,
    task: DownloadTask,
    kind: DownloadKind,
    mut cancellation: oneshot::Receiver<()>,
) {
    update_task(&app, id, true, |item| item.state = DownloadState::Downloading);
    let directory = task.destination.parent().unwrap_or_else(|| Path::new("."));
    let file_name = task.destination.file_name().and_then(|value| value.to_str()).unwrap_or("download");
    let mut command = match kind {
        DownloadKind::MediaPage => {
            let mut command = tokio::process::Command::new("yt-dlp");
            command.args(["--no-playlist", "-f", "bestvideo+bestaudio/best", "--merge-output-format", "mp4", "-P"])
                .arg(directory).arg("-o").arg(file_name).arg(&task.source);
            command
        }
        DownloadKind::Hls => {
            let executable = if cfg!(windows) { "N_m3u8DL-RE.exe" } else { "N_m3u8DL-RE" };
            let mut command = tokio::process::Command::new(executable);
            let stem = task.destination.file_stem().and_then(|value| value.to_str()).unwrap_or("download");
            command.arg(&task.source).arg("--save-dir").arg(directory).args(["--save-name", stem, "--auto-select"]);
            command
        }
        DownloadKind::Torrent | DownloadKind::Magnet => {
            let mut command = tokio::process::Command::new("aria2c");
            command.arg(format!("--dir={}", directory.display())).arg(&task.source);
            command
        }
        _ => return,
    };
    command.kill_on_drop(true);
    let result = match command.spawn() {
        Ok(mut child) => tokio::select! {
            biased;
            _ = &mut cancellation => {
                let _ = child.kill().await;
                return;
            }
            status = child.wait() => status.map_err(|error| error.to_string()).and_then(|status| {
                if status.success() { Ok(()) } else { Err(format!("external_engine_exit_{:?}", status.code())) }
            }),
        },
        Err(error) => Err(format!("external_engine_unavailable: {error}")),
    };
    match result {
        Ok(()) => update_task(&app, id, true, |item| item.state = DownloadState::Completed),
        Err(message) => update_task(&app, id, true, |item| item.state = DownloadState::Failed { message }),
    }
    if let Ok(mut workers) = app.state::<AppState>().workers.lock() {
        workers.remove(&id);
    }
    start_next_queued(&app);
}

fn suggested_name(source: &str) -> String {
    source.split(['/', '\\']).next_back().and_then(|part| part.split(['?', '#']).next())
        .filter(|part| !part.is_empty()).unwrap_or("download").chars()
        .map(|character| if "<>:\"/\\|?*".contains(character) { '_' } else { character }).collect()
}

fn validate_file_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() || name == "." || name == ".." || name.chars().any(|character| "<>:\"/\\|?*".contains(character)) {
        return Err("invalid_file_name".to_owned());
    }
    Ok(name.to_owned())
}

fn unique_destination(directory: &Path, file_name: &str) -> PathBuf {
    let original = directory.join(file_name);
    if !original.exists() && !partial_path(&original).exists() {
        return original;
    }
    let path = Path::new(file_name);
    let stem = path.file_stem().and_then(|value| value.to_str()).unwrap_or("download");
    let extension = path.extension().and_then(|value| value.to_str());
    for index in 1..10_000 {
        let candidate_name = match extension {
            Some(extension) => format!("{stem} ({index}).{extension}"),
            None => format!("{stem} ({index})"),
        };
        let candidate = directory.join(candidate_name);
        if !candidate.exists() && !partial_path(&candidate).exists() {
            return candidate;
        }
    }
    directory.join(format!("{stem}-10000"))
}

#[tauri::command]
fn suggest_download_name(url: String) -> String {
    suggested_name(&url)
}

#[tauri::command]
fn list_downloads(state: State<'_, AppState>) -> Result<Vec<DownloadTask>, String> {
    state.queue.lock().map(|queue| queue.clone()).map_err(|error| error.to_string())
}

#[tauri::command]
fn default_download_directory(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    configured_download_directory(&app, &state).map(|path| path.to_string_lossy().into_owned())
}

#[tauri::command]
fn set_default_download_directory(state: State<'_, AppState>, path: String) -> Result<String, String> {
    let path = PathBuf::from(path.trim());
    if !path.is_absolute() {
        return Err("destination_must_be_absolute".to_owned());
    }
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.download_directory = Some(path.clone());
    save_settings(&state, &settings)?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
fn list_download_directories(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<Vec<DestinationChoice>, String> {
    let default = configured_download_directory(&app, &state)?;
    let recent = state.settings.lock().map_err(|error| error.to_string())?.recent_download_directories.clone();
    let mut paths = vec![default.clone()];
    for path in recent {
        if !paths.contains(&path) { paths.push(path); }
    }
    Ok(paths.into_iter().map(|path| DestinationChoice {
        is_default: path == default,
        available: path.is_dir(),
        path: path.to_string_lossy().into_owned(),
    }).collect())
}

#[tauri::command]
fn remove_download_directory(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let target = PathBuf::from(path);
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.recent_download_directories.retain(|item| item != &target);
    save_settings(&state, &settings)
}

#[tauri::command]
fn clear_download_directories(state: State<'_, AppState>) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.recent_download_directories.clear();
    save_settings(&state, &settings)
}

#[tauri::command]
fn pick_directory(initial_directory: Option<String>) -> Option<String> {
    let mut dialog = rfd::FileDialog::new();
    if let Some(path) = initial_directory.filter(|path| !path.trim().is_empty()) {
        dialog = dialog.set_directory(path);
    }
    dialog.pick_folder().map(|path| path.to_string_lossy().into_owned())
}

#[tauri::command]
fn enqueue_download(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    url: String,
    destination_directory: Option<String>,
    file_name: Option<String>,
) -> Result<DownloadTask, String> {
    inspect_url(url.clone())?;
    let kind = classify_url(&url).ok_or_else(|| "unsupported_url".to_owned())?;
    let download_dir = match destination_directory.filter(|path| !path.trim().is_empty()) {
        Some(path) => {
            let path = PathBuf::from(path);
            if !path.is_absolute() {
                return Err("destination_must_be_absolute".to_owned());
            }
            path
        }
        None => configured_download_directory(&app, &state)?,
    };
    let file_name = validate_file_name(&file_name.unwrap_or_else(|| suggested_name(&url)))?;
    remember_download_directory(&state, &download_dir)?;
    let task = DownloadTask::new(&url, unique_destination(&download_dir, &file_name));
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    queue.push(task.clone());
    save_queue(&state, &queue)?;
    drop(queue);
    start_download(&app, &state, task.clone(), kind)?;
    Ok(task)
}

fn start_download(app: &tauri::AppHandle, state: &AppState, task: DownloadTask, kind: DownloadKind) -> Result<(), String> {
    let mut workers = state.workers.lock().map_err(|error| error.to_string())?;
    if workers.contains_key(&task.id) {
        return Err("download_already_running".to_owned());
    }
    let limits = state.settings.lock().map_err(|error| error.to_string())?.clone();
    if workers.len() >= limits.max_active_downloads.clamp(1, 20) {
        return Ok(());
    }
    let (cancel, cancelled) = oneshot::channel();
    workers.insert(task.id, cancel);
    drop(workers);
    if kind == DownloadKind::Http {
        let request = DownloadRequest {
            url: task.source,
            destination: task.destination,
            overwrite: false,
            connections: limits.connections_per_download.clamp(1, 32),
        };
        tauri::async_runtime::spawn(run_download(app.clone(), task.id, request, cancelled));
    } else {
        tauri::async_runtime::spawn(run_external_download(app.clone(), task.id, task, kind, cancelled));
    }
    Ok(())
}

fn start_next_queued(app: &tauri::AppHandle) {
    let state = app.state::<AppState>();
    let maximum = state.settings.lock().map(|settings| settings.max_active_downloads.clamp(1, 20)).unwrap_or(0);
    let available = state.workers.lock().map(|workers| maximum.saturating_sub(workers.len())).unwrap_or(0);
    if available == 0 { return; }
    let queued = state.queue.lock().map(|queue| queue.iter().filter(|task| task.state == DownloadState::Queued).take(available).cloned().collect::<Vec<_>>()).unwrap_or_default();
    for task in queued {
        if let Some(kind) = classify_url(&task.source) {
            let _ = start_download(app, &state, task, kind);
        }
    }
}

#[tauri::command]
fn pause_download(app: tauri::AppHandle, state: State<'_, AppState>, id: DownloadId) -> Result<(), String> {
    let cancel = state.workers.lock().map_err(|error| error.to_string())?.remove(&id)
        .ok_or_else(|| "download_not_running".to_owned())?;
    let _ = cancel.send(());
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    let task = queue.iter_mut().find(|task| task.id == id).ok_or_else(|| "download_not_found".to_owned())?;
    task.state = DownloadState::Paused;
    save_queue(&state, &queue)?;
    drop(queue);
    start_next_queued(&app);
    Ok(())
}

#[tauri::command]
fn resume_download(app: tauri::AppHandle, state: State<'_, AppState>, id: DownloadId) -> Result<(), String> {
    let task = {
        let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
        let task = queue.iter_mut().find(|task| task.id == id).ok_or_else(|| "download_not_found".to_owned())?;
        match task.state {
            DownloadState::Paused | DownloadState::Failed { .. } => {
                task.state = DownloadState::Queued;
                let task = task.clone();
                save_queue(&state, &queue)?;
                task
            },
            _ => return Err("download_not_resumable".to_owned()),
        }
    };
    let kind = classify_url(&task.source).ok_or_else(|| "unsupported_url".to_owned())?;
    start_download(&app, &state, task, kind)
}

#[tauri::command]
fn get_clipboard_monitor(state: State<'_, AppState>) -> Result<ClipboardStatus, String> {
    let enabled = state.settings.lock().map_err(|error| error.to_string())?.capture_clipboard;
    Ok(ClipboardStatus { enabled })
}

#[tauri::command]
fn set_clipboard_monitor(state: State<'_, AppState>, enabled: bool) -> Result<ClipboardStatus, String> {
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.capture_clipboard = enabled;
    save_settings(&state, &settings)?;
    Ok(ClipboardStatus { enabled })
}

#[tauri::command]
fn get_transfer_limits(state: State<'_, AppState>) -> Result<TransferLimits, String> {
    let settings = state.settings.lock().map_err(|error| error.to_string())?;
    Ok(TransferLimits {
        max_active_downloads: settings.max_active_downloads,
        connections_per_download: settings.connections_per_download,
    })
}

#[tauri::command]
fn set_transfer_limits(state: State<'_, AppState>, max_active_downloads: usize, connections_per_download: usize) -> Result<TransferLimits, String> {
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.max_active_downloads = max_active_downloads.clamp(1, 20);
    settings.connections_per_download = connections_per_download.clamp(1, 32);
    save_settings(&state, &settings)?;
    Ok(TransferLimits {
        max_active_downloads: settings.max_active_downloads,
        connections_per_download: settings.connections_per_download,
    })
}

#[tauri::command]
fn read_clipboard_link(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<Option<String>, String> {
    if !state.settings.lock().map_err(|error| error.to_string())?.capture_clipboard {
        return Ok(None);
    }
    let value = app.clipboard().read_text().map_err(|error| error.to_string())?;
    let value = value.trim();
    Ok(classify_url(value).map(|_| value.to_owned()))
}

#[tauri::command]
fn get_bridge_pairing(state: State<'_, AppState>) -> Result<BridgePairing, String> {
    let token = state.settings.lock().map_err(|error| error.to_string())?.bridge_token.clone();
    let connected = state.bridge_last_seen.lock().map_err(|error| error.to_string())?
        .is_some_and(|seen| seen.elapsed() < Duration::from_secs(15));
    Ok(BridgePairing { token, port: BRIDGE_PORT, connected })
}

#[tauri::command]
fn regenerate_bridge_token(state: State<'_, AppState>) -> Result<BridgePairing, String> {
    let token = {
        let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
        settings.bridge_token = default_bridge_token();
        save_settings(&state, &settings)?;
        settings.bridge_token.clone()
    };
    if let Ok(mut seen) = state.bridge_last_seen.lock() { *seen = None; }
    Ok(BridgePairing { token, port: BRIDGE_PORT, connected: false })
}

#[tauri::command]
fn copy_bridge_token(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let token = state.settings.lock().map_err(|error| error.to_string())?.bridge_token.clone();
    app.clipboard().write_text(token).map_err(|error| error.to_string())
}

fn bridge_origin(headers: &str) -> Option<&str> {
    headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if name.eq_ignore_ascii_case("origin") {
            let value = value.trim();
            (value.starts_with("chrome-extension://") || value.starts_with("moz-extension://")).then_some(value)
        } else { None }
    })
}

fn bridge_authorized(headers: &str, expected: &str) -> bool {
    headers.lines().any(|line| {
        line.split_once(':').is_some_and(|(name, value)| {
            name.eq_ignore_ascii_case("authorization") && value.trim() == format!("Bearer {expected}")
        })
    })
}

fn bridge_response(stream: &mut TcpStream, status: &str, origin: Option<&str>, body: &str) {
    let cors = origin.map(|value| format!("Access-Control-Allow-Origin: {value}\r\nVary: Origin\r\n")).unwrap_or_default();
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\n{cors}Access-Control-Allow-Headers: Authorization, Content-Type\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}

fn enqueue_from_bridge(app: &tauri::AppHandle, request: BridgeDownload) -> Result<(), String> {
    let state = app.state::<AppState>();
    let kind = classify_url(&request.url).ok_or_else(|| "unsupported_url".to_owned())?;
    let directory = configured_download_directory(app, &state)?;
    let proposed = request.file_name.filter(|name| !name.trim().is_empty()).unwrap_or_else(|| suggested_name(&request.url));
    let file_name = validate_file_name(&suggested_name(&proposed))?;
    remember_download_directory(&state, &directory)?;
    let task = DownloadTask::new(&request.url, unique_destination(&directory, &file_name));
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    queue.push(task.clone());
    save_queue(&state, &queue)?;
    drop(queue);
    start_download(app, &state, task, kind)
}

fn handle_bridge_connection(app: &tauri::AppHandle, mut stream: TcpStream) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
    let mut buffer = vec![0_u8; 65_536];
    let count = match stream.read(&mut buffer) { Ok(count) => count, Err(_) => return };
    let request = String::from_utf8_lossy(&buffer[..count]);
    let Some((headers, body)) = request.split_once("\r\n\r\n") else { return };
    let origin = bridge_origin(headers);
    if origin.is_none() {
        bridge_response(&mut stream, "403 Forbidden", None, "{\"ok\":false}");
        return;
    }
    let first = headers.lines().next().unwrap_or_default();
    if first.starts_with("OPTIONS ") {
        bridge_response(&mut stream, "204 No Content", origin, "");
        return;
    }
    let state = app.state::<AppState>();
    let token = match state.settings.lock() { Ok(settings) => settings.bridge_token.clone(), Err(_) => return };
    if !bridge_authorized(headers, &token) {
        bridge_response(&mut stream, "401 Unauthorized", origin, "{\"ok\":false}");
        return;
    }
    if let Ok(mut seen) = state.bridge_last_seen.lock() { *seen = Some(Instant::now()); }
    if first.starts_with("GET /v1/health ") {
        bridge_response(&mut stream, "200 OK", origin, "{\"ok\":true}");
    } else if first.starts_with("POST /v1/download ") {
        match serde_json::from_str::<BridgeDownload>(body).map_err(|error| error.to_string()).and_then(|request| enqueue_from_bridge(app, request)) {
            Ok(()) => bridge_response(&mut stream, "202 Accepted", origin, "{\"ok\":true}"),
            Err(_) => bridge_response(&mut stream, "400 Bad Request", origin, "{\"ok\":false}"),
        }
    } else {
        bridge_response(&mut stream, "404 Not Found", origin, "{\"ok\":false}");
    }
}

fn run_extension_bridge(app: tauri::AppHandle) {
    let Ok(listener) = TcpListener::bind(("127.0.0.1", BRIDGE_PORT)) else { return };
    for stream in listener.incoming().flatten() {
        handle_bridge_connection(&app, stream);
    }
}

#[tauri::command]
fn reveal_download(state: State<'_, AppState>, id: DownloadId) -> Result<(), String> {
    let queue = state.queue.lock().map_err(|error| error.to_string())?;
    let task = queue.iter().find(|task| task.id == id).ok_or_else(|| "download_not_found".to_owned())?;
    let target = if task.destination.exists() { task.destination.clone() } else { partial_path(&task.destination) };
    #[cfg(target_os = "windows")]
    let result = Command::new("explorer.exe").arg(format!("/select,{}", target.display())).spawn();
    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg("-R").arg(&target).spawn();
    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open").arg(target.parent().unwrap_or(&target)).spawn();
    result.map(|_| ()).map_err(|error| error.to_string())
}

#[cfg(target_os = "windows")]
fn autostart_enabled(_: &tauri::AppHandle) -> Result<bool, String> {
    Command::new("reg.exe")
        .args(["QUERY", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run", "/v", "ApocalipseDownloadManager"])
        .status().map(|status| status.success()).map_err(|error| error.to_string())
}

#[cfg(target_os = "windows")]
fn configure_autostart(_: &tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let mut command = Command::new("reg.exe");
    command.args(if enabled {
        vec!["ADD".into(), r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run".into(), "/v".into(), "ApocalipseDownloadManager".into(), "/t".into(), "REG_SZ".into(), "/d".into(), format!("\"{}\" --hidden", executable.display()), "/f".into()]
    } else {
        vec!["DELETE".into(), r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run".into(), "/v".into(), "ApocalipseDownloadManager".into(), "/f".into()]
    });
    let status = command.status().map_err(|error| error.to_string())?;
    if enabled && !status.success() { return Err("autostart_update_failed".to_owned()); }
    Ok(())
}

#[cfg(target_os = "linux")]
fn autostart_entry(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path().config_dir().map(|path| path.join("autostart/apocalipse-download-manager.desktop")).map_err(|error| error.to_string())
}

#[cfg(target_os = "linux")]
fn autostart_enabled(app: &tauri::AppHandle) -> Result<bool, String> {
    Ok(autostart_entry(app)?.is_file())
}

#[cfg(target_os = "linux")]
fn configure_autostart(app: &tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let entry = autostart_entry(app)?;
    if enabled {
        let executable = std::env::current_exe().map_err(|error| error.to_string())?;
        if let Some(parent) = entry.parent() { fs::create_dir_all(parent).map_err(|error| error.to_string())?; }
        let escaped = executable.to_string_lossy().replace('"', "\\\"");
        fs::write(entry, format!("[Desktop Entry]\nType=Application\nName=Apocalipse Download Manager\nExec=\"{escaped}\" --hidden\nTerminal=false\nX-GNOME-Autostart-enabled=true\n" )).map_err(|error| error.to_string())?;
    } else if entry.exists() {
        fs::remove_file(entry).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn autostart_entry(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path().home_dir().map(|path| path.join("Library/LaunchAgents/com.linuxhell.apocalipse.plist")).map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
fn autostart_enabled(app: &tauri::AppHandle) -> Result<bool, String> {
    Ok(autostart_entry(app)?.is_file())
}

#[cfg(target_os = "macos")]
fn configure_autostart(app: &tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let entry = autostart_entry(app)?;
    if enabled {
        let executable = std::env::current_exe().map_err(|error| error.to_string())?;
        if let Some(parent) = entry.parent() { fs::create_dir_all(parent).map_err(|error| error.to_string())?; }
        let escaped = executable.to_string_lossy().replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
        let plist = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\"><dict><key>Label</key><string>com.linuxhell.apocalipse</string><key>ProgramArguments</key><array><string>{escaped}</string><string>--hidden</string></array><key>RunAtLoad</key><true/></dict></plist>\n");
        fs::write(entry, plist).map_err(|error| error.to_string())?;
    } else if entry.exists() {
        fs::remove_file(entry).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_autostart(app: tauri::AppHandle) -> Result<AutostartStatus, String> {
    Ok(AutostartStatus { enabled: autostart_enabled(&app)? })
}

#[tauri::command]
fn set_autostart(app: tauri::AppHandle, enabled: bool) -> Result<AutostartStatus, String> {
    configure_autostart(&app, enabled)?;
    get_autostart(app)
}

#[tauri::command]
async fn remove_downloads(state: State<'_, AppState>, ids: Vec<DownloadId>, delete_files: bool) -> Result<usize, String> {
    if ids.is_empty() {
        return Ok(0);
    }
    let mut cancelled_active = false;
    if let Ok(mut workers) = state.workers.lock() {
        for id in &ids {
            if let Some(cancel) = workers.remove(id) {
                let _ = cancel.send(());
                cancelled_active = true;
            }
        }
    }
    if cancelled_active {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    let mut removed = Vec::new();
    queue.retain(|task| {
        if ids.contains(&task.id) {
            removed.push(task.clone());
            false
        } else {
            true
        }
    });
    if delete_files {
        for task in &removed {
            let partial = partial_path(&task.destination);
            for path in [&task.destination, &partial] {
                if path.is_file() {
                    fs::remove_file(path).map_err(|error| format!("{}: {error}", path.display()))?;
                }
            }
        }
    }
    save_queue(&state, &queue)?;
    Ok(removed.len())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let queue_path = app.path().app_data_dir()?.join("queue.json");
            let settings_path = app.path().app_data_dir()?.join("settings.json");
            app.manage(AppState {
                queue: Mutex::new(load_queue(&queue_path)),
                queue_path,
                workers: Mutex::new(HashMap::new()),
                settings: Mutex::new(load_settings(&settings_path)),
                settings_path,
                bridge_last_seen: Mutex::new(None),
            });
            let bridge_app = app.handle().clone();
            std::thread::Builder::new().name("apocalipse-extension-bridge".into())
                .spawn(move || run_extension_bridge(bridge_app))?;
            let show = MenuItem::with_id(app, "show", "Show Apocalipse", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            // The detailed application artwork loses definition at the 16–24 px sizes used by
            // system trays. Keep a simplified, high-contrast asset specifically for this role.
            let icon = Image::new_owned(
                include_bytes!("../icons/tray.rgba").to_vec(),
                32,
                32,
            );
            TrayIconBuilder::new()
                .icon(icon)
                .tooltip("Apocalipse Download Manager")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            if std::env::args().any(|argument| argument == "--hidden") {
                if let Some(window) = app.get_webview_window("main") { window.hide()?; }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            inspect_url,
            list_downloads,
            enqueue_download,
            default_download_directory,
            set_default_download_directory,
            pick_directory,
            suggest_download_name,
            remove_downloads,
            pause_download,
            resume_download,
            reveal_download,
            get_autostart,
            set_autostart,
            get_clipboard_monitor,
            set_clipboard_monitor,
            read_clipboard_link,
            get_transfer_limits,
            set_transfer_limits,
            get_bridge_pairing,
            regenerate_bridge_token,
            copy_bridge_token,
            list_download_directories,
            remove_download_directory,
            clear_download_directories
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Apocalipse Download Manager");
}
