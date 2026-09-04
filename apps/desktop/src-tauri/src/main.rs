#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use apocalipse_core::{
    classify_url, partial_path, plan_download, Capabilities, DownloadEngine, DownloadEvent,
    DownloadId, DownloadKind, DownloadRequest, DownloadState, DownloadTask,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io::{Read, Write}, net::{TcpListener, TcpStream}, path::{Path, PathBuf}, process::{Command, Stdio}, sync::Mutex, time::{Duration, Instant}};
use tauri::{image::Image, menu::{Menu, MenuItem}, tray::TrayIconBuilder, Manager, State};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tokio::{io::AsyncReadExt, sync::{mpsc, oneshot}};

struct AppState {
    queue: Mutex<Vec<DownloadTask>>,
    queue_path: PathBuf,
    workers: Mutex<HashMap<DownloadId, oneshot::Sender<()>>>,
    settings: Mutex<UserSettings>,
    settings_path: PathBuf,
    bridge_last_seen: Mutex<Option<Instant>>,
    bridge_pending: Mutex<Vec<BridgeDownload>>,
    request_identities: Mutex<HashMap<DownloadId, RequestIdentity>>,
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
    #[serde(default)]
    ffmpeg_path: Option<PathBuf>,
    #[serde(default)]
    yt_dlp_path: Option<PathBuf>,
    #[serde(default)]
    n_m3u8dl_re_path: Option<PathBuf>,
    #[serde(default)]
    aria2_path: Option<PathBuf>,
    #[serde(default)]
    user_agent: Option<String>,
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
            ffmpeg_path: None,
            yt_dlp_path: None,
            n_m3u8dl_re_path: None,
            aria2_path: None,
            user_agent: None,
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
#[serde(rename_all = "camelCase")]
struct MediaFormat { selection: String, label: String }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MediaInspection {
    title: String,
    thumbnail: Option<String>,
    duration: Option<f64>,
    suggested_file_name: String,
    formats: Vec<MediaFormat>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolStatus { id: String, path: String, found: bool, version: Option<String> }

fn configured_tool(path: &Option<PathBuf>, fallback: &str) -> PathBuf {
    path.clone().filter(|value| !value.as_os_str().is_empty()).unwrap_or_else(|| PathBuf::from(fallback))
}

fn http_origin(url: &str) -> Option<&str> {
    let scheme_end = url.find("://")? + 3;
    let path_start = url[scheme_end..].find('/').map(|index| scheme_end + index).unwrap_or(url.len());
    Some(&url[..path_start])
}

fn version_line(executable: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new(executable).args(args).output().ok()?;
    if !output.status.success() { return None; }
    let text = if output.stdout.is_empty() { String::from_utf8_lossy(&output.stderr) } else { String::from_utf8_lossy(&output.stdout) };
    text.lines().next().map(str::trim).filter(|line| !line.is_empty()).map(str::to_owned)
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
struct UserAgentSetting { user_agent: String }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BridgePairing {
    token: String,
    port: u16,
    connected: bool,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct BridgeDownload {
    url: String,
    file_name: Option<String>,
    page_url: Option<String>,
    duration: Option<f64>,
    cookie_header: Option<String>,
    user_agent: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadContext {
    referer: Option<String>,
    known_duration: Option<f64>,
    cookie_header: Option<String>,
    user_agent: Option<String>,
}

#[derive(Clone)]
struct RequestIdentity {
    cookie_header: Option<String>,
    user_agent: Option<String>,
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

#[tauri::command]
async fn inspect_media_formats(state: State<'_, AppState>, url: String) -> Result<MediaInspection, String> {
    let executable = configured_tool(&state.settings.lock().map_err(|error| error.to_string())?.yt_dlp_path, "yt-dlp");
    let output = tokio::process::Command::new(executable)
        .args(["--dump-single-json", "--no-playlist", "--skip-download", "--no-warnings"])
        .arg(&url).output().await.map_err(|error| format!("yt_dlp_unavailable: {error}"))?;
    if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned()); }
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())?;
    let title = value.get("title").and_then(|item| item.as_str()).unwrap_or("media").to_owned();
    let thumbnail = value.get("thumbnail").and_then(|item| item.as_str()).map(str::to_owned);
    let duration = value.get("duration").and_then(|item| item.as_f64());
    let mut formats = value.get("formats").and_then(|item| item.as_array()).into_iter().flatten().filter_map(|format| {
        let id = format.get("format_id")?.as_str()?;
        let vcodec = format.get("vcodec").and_then(|item| item.as_str()).unwrap_or("none");
        let acodec = format.get("acodec").and_then(|item| item.as_str()).unwrap_or("none");
        if vcodec == "none" { return None; }
        let height = format.get("height").and_then(|item| item.as_u64()).map(|value| format!("{value}p")).unwrap_or_else(|| "video".into());
        let fps = format.get("fps").and_then(|item| item.as_f64()).map(|value| format!(" · {} fps", value.round())).unwrap_or_default();
        let extension = format.get("ext").and_then(|item| item.as_str()).unwrap_or("");
        let size = format.get("filesize").or_else(|| format.get("filesize_approx")).and_then(|item| item.as_u64()).map(|value| format!(" · {:.1} MB", value as f64 / 1_048_576.0)).unwrap_or_default();
        let selection = if acodec == "none" { format!("{id}+bestaudio/best") } else { id.to_owned() };
        Some(MediaFormat { selection, label: format!("{height}{fps} · {extension}{size}") })
    }).collect::<Vec<_>>();
    formats.reverse();
    formats.truncate(120);
    let safe_title = title.chars().map(|character| if "<>:\"/\\|?*".contains(character) { '_' } else { character }).collect::<String>();
    Ok(MediaInspection { title, thumbnail, duration, suggested_file_name: format!("{safe_title}.mp4"), formats })
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
    update_task(&app, id, true, |item| {
        item.state = DownloadState::Downloading;
        item.progress_percent = Some(0.0);
    });
    let directory = task.destination.parent().unwrap_or_else(|| Path::new("."));
    let file_name = task.destination.file_name().and_then(|value| value.to_str()).unwrap_or("download");
    let tools = app.state::<AppState>().settings.lock().map(|settings| (
        configured_tool(&settings.ffmpeg_path, "ffmpeg"),
        configured_tool(&settings.yt_dlp_path, "yt-dlp"),
        configured_tool(&settings.n_m3u8dl_re_path, if cfg!(windows) { "N_m3u8DL-RE.exe" } else { "N_m3u8DL-RE" }),
        configured_tool(&settings.aria2_path, if cfg!(windows) { "aria2c.exe" } else { "aria2c" }),
        settings.connections_per_download.clamp(1, 32),
    )).unwrap_or_else(|_| ("ffmpeg".into(), "yt-dlp".into(), "N_m3u8DL-RE".into(), "aria2c".into(), 8));
    let identity = app.state::<AppState>().request_identities.lock().ok()
        .and_then(|identities| identities.get(&task.id).cloned());
    let configured_user_agent = app.state::<AppState>().settings.lock().ok()
        .and_then(|settings| settings.user_agent.clone());
    let user_agent = configured_user_agent.as_deref()
        .or_else(|| identity.as_ref().and_then(|value| value.user_agent.as_deref()))
        .unwrap_or("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/152.0.0.0 Safari/537.36");
    let mut command = match kind {
        DownloadKind::MediaPage => {
            let mut command = tokio::process::Command::new(&tools.1);
            let selection = task.format_selection.as_deref().unwrap_or("bestvideo+bestaudio/best");
            command.arg("--no-playlist");
            if task.source.contains("youtube.com/") || task.source.contains("youtu.be/") {
                command.args(["--cookies-from-browser", "chrome", "--retries", "10", "--fragment-retries", "10", "--retry-sleep", "fragment:exp=1:8"]);
            }
            if let Some(referer) = task.referer.as_deref() { command.args(["--referer", referer]); }
            if let Some(audio_format) = selection.strip_prefix("audio:") {
                command.args(["-f", "bestaudio/best", "-x", "--audio-format", audio_format]);
            } else {
                command.args(["-f", selection, "--merge-output-format", "mp4"]);
            }
            command.arg("-P").arg(directory).arg("-o").arg(file_name).arg(&task.source);
            command
        }
        DownloadKind::Hls => {
            if let Some(audio_format) = task.format_selection.as_deref().and_then(|value| value.strip_prefix("audio:")) {
                let mut command = tokio::process::Command::new(&tools.0);
                command.args(["-y", "-i"]).arg(&task.source).arg("-vn");
                match audio_format {
                    "mp3" => { command.args(["-c:a", "libmp3lame", "-q:a", "2"]); }
                    "wav" => { command.args(["-c:a", "pcm_s16le"]); }
                    "flac" => { command.args(["-c:a", "flac"]); }
                    "opus" => { command.args(["-c:a", "libopus", "-b:a", "192k"]); }
                    _ => { command.args(["-c:a", "aac", "-b:a", "256k"]); }
                }
                command.arg(&task.destination);
                command
            } else {
            let mut command = tokio::process::Command::new(&tools.2);
            let stem = task.destination.file_stem().and_then(|value| value.to_str()).unwrap_or("download");
            command.arg(&task.source).arg("--save-dir").arg(directory)
                .args(["--save-name", stem, "--auto-select", "--concurrent-download", "--download-retry-count", "10", "--http-request-timeout", "30"])
                .arg("--thread-count").arg(tools.4.to_string())
                .arg("--ffmpeg-binary-path").arg(&tools.0);
            if let Some(referer) = task.referer.as_deref() {
                command.arg("-H").arg(format!("Referer: {referer}"));
                if let Some(origin) = http_origin(referer) {
                    command.arg("-H").arg(format!("Origin: {origin}"));
                }
            }
            command.arg("-H").arg(format!("User-Agent: {user_agent}"));
            if let Some(cookie) = identity.as_ref().and_then(|value| value.cookie_header.as_deref()) {
                command.arg("-H").arg(format!("Cookie: {cookie}"));
            }
            if task.source.contains("hdsex.org") || task.referer.as_deref().is_some_and(|url| url.contains("hdsex.org")) {
                command.arg("--append-url-params=true");
            }
            // Some video hosts expose completed VOD playlists without ENDLIST and therefore
            // look live. Treat known finite CDN captures as VOD so the task does not wait forever.
            if task.known_duration.is_some_and(|duration| duration > 0.0)
                || task.source.contains("growcdnssedge.com")
            {
                command.arg("--live-perform-as-vod");
            }
            command.args([
                "--check-segments-count=true",
                "--del-after-done=true",
                "--write-meta-json=false",
                "--no-log=true",
                "--no-ansi-color=true",
                "--disable-update-check=true",
                "--mux-after-done=format=mp4:muxer=ffmpeg",
            ]);
            command
            }
        }
        DownloadKind::Torrent | DownloadKind::Magnet => {
            let mut command = tokio::process::Command::new(&tools.3);
            command.arg(format!("--dir={}", directory.display())).arg(&task.source);
            command
        }
        _ => return,
    };
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.as_std_mut().creation_flags(0x08000000);
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    command.kill_on_drop(true);
    let result = match command.spawn() {
        Ok(mut child) => {
            let mut stdout = child.stdout.take();
            let mut stderr = child.stderr.take();
            let output_app = app.clone();
            let error_app = app.clone();
            let output = tokio::spawn(async move { match stdout.take() { Some(stream) => read_process_tail(stream, Some((output_app, id))).await, None => Vec::new() } });
            let errors = tokio::spawn(async move { match stderr.take() { Some(stream) => read_process_tail(stream, Some((error_app, id))).await, None => Vec::new() } });
            let status = tokio::select! {
                biased;
                _ = &mut cancellation => { let _ = child.kill().await; return; }
                status = child.wait() => status,
            };
            let mut text = String::from_utf8_lossy(&output.await.unwrap_or_default()).into_owned();
            text.push_str(&String::from_utf8_lossy(&errors.await.unwrap_or_default()));
            status.map_err(|error| error.to_string()).and_then(|status| {
                if status.success() { Ok(()) } else {
                    Err(external_error_detail(&text, status.code()))
                }
            })
        },
        Err(error) => Err(format!("external_engine_unavailable: {error}")),
    };
    match result {
        Ok(()) => update_task(&app, id, true, |item| {
            item.progress_percent = Some(100.0);
            item.state = DownloadState::Completed;
        }),
        Err(message) => update_task(&app, id, true, |item| item.state = DownloadState::Failed { message }),
    }
    if let Ok(mut workers) = app.state::<AppState>().workers.lock() {
        workers.remove(&id);
    }
    start_next_queued(&app);
}

async fn read_process_tail(
    mut stream: impl tokio::io::AsyncRead + Unpin,
    progress: Option<(tauri::AppHandle, DownloadId)>,
) -> Vec<u8> {
    let mut tail = Vec::new();
    let mut chunk = [0_u8; 4096];
    loop {
        match stream.read(&mut chunk).await {
            Ok(0) | Err(_) => break,
            Ok(count) => {
                if let Some(percent) = parse_external_progress(&String::from_utf8_lossy(&chunk[..count])) {
                    if let Some((app, id)) = progress.as_ref() {
                        update_task(app, *id, false, |task| {
                            task.progress_percent = Some(task.progress_percent.unwrap_or(0.0).max(percent));
                        });
                    }
                }
                tail.extend_from_slice(&chunk[..count]);
                if tail.len() > 65_536 { tail.drain(..tail.len() - 65_536); }
            }
        }
    }
    tail
}

fn parse_external_progress(text: &str) -> Option<f64> {
    text.match_indices('%').filter_map(|(end, _)| {
        let prefix = &text[..end];
        let start = prefix.char_indices().rev()
            .take_while(|(_, character)| character.is_ascii_digit() || *character == '.')
            .last().map(|(index, _)| index)?;
        prefix[start..].parse::<f64>().ok()
    }).filter(|value| (0.0..=100.0).contains(value)).last()
}

fn external_error_detail(output: &str, exit_code: Option<i32>) -> String {
    let lines = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    let diagnostic = lines
        .iter()
        .copied()
        .filter(|line| {
            let lowered = line.to_ascii_lowercase();
            !lowered.starts_with("at ")
                && !lowered.contains("end of stack trace")
                && (lowered.contains("error")
                    || lowered.contains("exception")
                    || lowered.contains("failed")
                    || lowered.contains("forbidden")
                    || lowered.contains("unauthorized")
                    || lowered.contains("status code")
                    || lowered.contains("timed out")
                    || lowered.contains("not supported"))
        })
        .rev()
        .take(12)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>();
    if !diagnostic.is_empty() {
        return diagnostic.join("\n");
    }
    let fallback = lines
        .into_iter()
        .filter(|line| {
            let lowered = line.to_ascii_lowercase();
            !lowered.starts_with("at ") && !lowered.contains("end of stack trace")
        })
        .rev()
        .take(30)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n");
    if fallback.is_empty() {
        format!("external_engine_exit_{exit_code:?}")
    } else {
        fallback
    }
}

fn suggested_name(source: &str) -> String {
    source.split(['/', '\\']).next_back().and_then(|part| part.split(['?', '#']).next())
        .filter(|part| !part.is_empty()).unwrap_or("download").chars()
        .map(|character| if "<>:\"/\\|?*".contains(character) { '_' } else { character }).collect()
}

fn suggested_download_name(source: &str) -> String {
    let name = suggested_name(source);
    if classify_url(source) == Some(DownloadKind::Hls) {
        let stem = Path::new(&name)
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("stream");
        format!("{stem}.mp4")
    } else {
        name
    }
}

fn validate_file_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() || name == "." || name == ".." || name.chars().any(|character| "<>:\"/\\|?*".contains(character)) {
        return Err("invalid_file_name".to_owned());
    }
    Ok(name.to_owned())
}

fn append_source_extension(file_name: String, source: &str, kind: DownloadKind) -> String {
    let has_extension = Path::new(&file_name)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| {
            (1..=10).contains(&value.len())
                && value.chars().all(|character| character.is_ascii_alphanumeric())
        });
    if kind != DownloadKind::Http || has_extension { return file_name; }
    let source_name = suggested_name(source);
    let extension = Path::new(&source_name).extension().and_then(|value| value.to_str())
        .filter(|value| (1..=10).contains(&value.len()) && value.chars().all(|character| character.is_ascii_alphanumeric()));
    extension.map(|extension| format!("{file_name}.{extension}")).unwrap_or(file_name)
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
    suggested_download_name(&url)
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
fn pick_executable(initial_path: Option<String>) -> Option<String> {
    let mut dialog = rfd::FileDialog::new();
    if let Some(path) = initial_path.filter(|value| !value.trim().is_empty()) {
        if let Some(parent) = Path::new(&path).parent() { dialog = dialog.set_directory(parent); }
    }
    dialog.pick_file().map(|path| path.to_string_lossy().into_owned())
}

#[tauri::command]
fn get_tool_statuses(state: State<'_, AppState>) -> Result<Vec<ToolStatus>, String> {
    let settings = state.settings.lock().map_err(|error| error.to_string())?;
    let definitions = [
        ("ffmpeg", configured_tool(&settings.ffmpeg_path, if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" }), ["-version"].as_slice()),
        ("yt-dlp", configured_tool(&settings.yt_dlp_path, if cfg!(windows) { "yt-dlp.exe" } else { "yt-dlp" }), ["--version"].as_slice()),
        ("n-m3u8dl-re", configured_tool(&settings.n_m3u8dl_re_path, if cfg!(windows) { "N_m3u8DL-RE.exe" } else { "N_m3u8DL-RE" }), ["--version"].as_slice()),
        ("aria2", configured_tool(&settings.aria2_path, if cfg!(windows) { "aria2c.exe" } else { "aria2c" }), ["--version"].as_slice()),
    ];
    Ok(definitions.into_iter().map(|(id, executable, args)| {
        let version = version_line(&executable, args);
        ToolStatus { id: id.to_owned(), path: executable.to_string_lossy().into_owned(), found: version.is_some(), version }
    }).collect())
}

fn optional_path(value: String) -> Option<PathBuf> {
    let value = value.trim();
    (!value.is_empty()).then(|| PathBuf::from(value))
}

#[tauri::command]
fn set_tool_paths(state: State<'_, AppState>, ffmpeg: String, yt_dlp: String, n_m3u8dl_re: String, aria2: String) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.ffmpeg_path = optional_path(ffmpeg);
    settings.yt_dlp_path = optional_path(yt_dlp);
    settings.n_m3u8dl_re_path = optional_path(n_m3u8dl_re);
    settings.aria2_path = optional_path(aria2);
    save_settings(&state, &settings)
}

#[tauri::command]
fn enqueue_download(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    url: String,
    destination_directory: Option<String>,
    file_name: Option<String>,
    format_selection: Option<String>,
    context: Option<DownloadContext>,
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
    let proposed = file_name.unwrap_or_else(|| suggested_name(&url));
    let file_name = validate_file_name(&append_source_extension(proposed, &url, kind))?;
    remember_download_directory(&state, &download_dir)?;
    let mut task = DownloadTask::new(&url, unique_destination(&download_dir, &file_name));
    task.format_selection = format_selection.filter(|value| !value.trim().is_empty());
    if let Some(context) = context {
        task.referer = context
            .referer
            .filter(|url| url.starts_with("https://") || url.starts_with("http://"));
        task.known_duration = context
            .known_duration
            .filter(|duration| duration.is_finite() && *duration > 0.0);
        let cookie_header = context.cookie_header.filter(|value| value.len() <= 16_384 && !value.contains('\r') && !value.contains('\n'));
        let user_agent = context.user_agent.filter(|value| value.len() <= 1024 && !value.contains('\r') && !value.contains('\n'));
        if cookie_header.is_some() || user_agent.is_some() {
            state.request_identities.lock().map_err(|error| error.to_string())?
                .insert(task.id, RequestIdentity { cookie_header, user_agent });
        }
    }
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
fn get_user_agent(state: State<'_, AppState>) -> Result<UserAgentSetting, String> {
    let value = state.settings.lock().map_err(|error| error.to_string())?.user_agent.clone().unwrap_or_default();
    Ok(UserAgentSetting { user_agent: value })
}

#[tauri::command]
fn set_user_agent(state: State<'_, AppState>, user_agent: String) -> Result<UserAgentSetting, String> {
    let value = user_agent.trim();
    if value.len() > 1024 || value.contains('\r') || value.contains('\n') {
        return Err("invalid_user_agent".to_owned());
    }
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.user_agent = (!value.is_empty()).then(|| value.to_owned());
    save_settings(&state, &settings)?;
    Ok(UserAgentSetting { user_agent: settings.user_agent.clone().unwrap_or_default() })
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
        .is_some_and(|seen| seen.elapsed() < Duration::from_secs(75));
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
            (value.starts_with("chrome-extension://") || value.starts_with("moz-extension://") || value == "null").then_some(value)
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
    let cors = origin.map(|value| format!("Access-Control-Allow-Origin: {value}\r\nVary: Origin\r\n")).unwrap_or_else(|| "Access-Control-Allow-Origin: *\r\n".to_owned());
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\n{cors}Access-Control-Allow-Headers: Authorization, Content-Type\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}

fn queue_from_bridge(app: &tauri::AppHandle, request: BridgeDownload) -> Result<(), String> {
    let state = app.state::<AppState>();
    classify_url(&request.url).ok_or_else(|| "unsupported_url".to_owned())?;
    state.bridge_pending.lock().map_err(|error| error.to_string())?.push(request);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
    Ok(())
}

#[tauri::command]
fn take_bridge_download(state: State<'_, AppState>) -> Result<Option<BridgeDownload>, String> {
    let mut pending = state.bridge_pending.lock().map_err(|error| error.to_string())?;
    Ok((!pending.is_empty()).then(|| pending.remove(0)))
}

fn handle_bridge_connection(app: &tauri::AppHandle, mut stream: TcpStream) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
    let mut buffer = vec![0_u8; 65_536];
    let count = match stream.read(&mut buffer) { Ok(count) => count, Err(_) => return };
    let request = String::from_utf8_lossy(&buffer[..count]);
    let Some((headers, body)) = request.split_once("\r\n\r\n") else { return };
    let origin = bridge_origin(headers);
    let has_origin = headers.lines().any(|line| line.split_once(':').is_some_and(|(name, _)| name.eq_ignore_ascii_case("origin")));
    if has_origin && origin.is_none() {
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
        match serde_json::from_str::<BridgeDownload>(body).map_err(|error| error.to_string()).and_then(|request| queue_from_bridge(app, request)) {
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
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    }
    let removed = state.queue.lock().map_err(|error| error.to_string())?
        .iter().filter(|task| ids.contains(&task.id)).cloned().collect::<Vec<_>>();
    if delete_files {
        for task in &removed {
            for path in download_paths(task) {
                remove_file_with_retry(&path).await?;
            }
        }
    }
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    queue.retain(|task| !ids.contains(&task.id));
    if let Ok(mut identities) = state.request_identities.lock() {
        identities.retain(|id, _| !ids.contains(id));
    }
    save_queue(&state, &queue)?;
    Ok(removed.len())
}

fn download_paths(task: &DownloadTask) -> Vec<PathBuf> {
    let partial = partial_path(&task.destination);
    let mut paths = vec![task.destination.clone(), partial];
    let stem = task.destination.file_stem().and_then(|value| value.to_str());
    if let (Some(parent), Some(stem)) = (task.destination.parent(), stem) {
        for extension in ["mp4", "mkv", "ts", "webm", "m4a", "mp3", "wav", "flac", "opus", "aac"] {
            let candidate = parent.join(format!("{stem}.{extension}"));
            if !paths.contains(&candidate) { paths.push(candidate); }
        }
    }
    paths
}

async fn remove_file_with_retry(path: &Path) -> Result<(), String> {
    for attempt in 0..5 {
        match tokio::fs::remove_file(path).await {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied && attempt < 4 => {
                tokio::time::sleep(std::time::Duration::from_millis(100 * (attempt + 1))).await;
            }
            Err(error) => return Err(format!("{}: {error}", path.display())),
        }
    }
    Ok(())
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
                bridge_pending: Mutex::new(Vec::new()),
                request_identities: Mutex::new(HashMap::new()),
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
            inspect_media_formats,
            list_downloads,
            enqueue_download,
            default_download_directory,
            set_default_download_directory,
            pick_directory,
            pick_executable,
            get_tool_statuses,
            set_tool_paths,
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
            get_user_agent,
            set_user_agent,
            get_bridge_pairing,
            regenerate_bridge_token,
            copy_bridge_token,
            list_download_directories,
            remove_download_directory,
            clear_download_directories,
            take_bridge_download
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Apocalipse Download Manager");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_paths_include_hls_output_and_partial_file() {
        let task = DownloadTask::new(
            "https://edge-hls.growcdnssedge.com/hls/157651625/master/157651625.m3u8",
            PathBuf::from("C:/Downloads/157651625.mp4"),
        );
        let paths = download_paths(&task);
        assert!(paths.contains(&PathBuf::from("C:/Downloads/157651625.mp4")));
        assert!(paths.contains(&partial_path(&task.destination)));
    }

    #[tokio::test]
    async fn removing_a_missing_file_is_already_successful() {
        let path = std::env::temp_dir().join(format!("apocalipse-missing-{}.mp4", uuid::Uuid::new_v4()));
        assert!(remove_file_with_retry(&path).await.is_ok());
    }

    #[test]
    fn reads_simplified_external_engine_progress() {
        assert_eq!(parse_external_progress("Vid Kbps: 23%\r"), Some(23.0));
        assert_eq!(parse_external_progress("Aud Kbps: 7.5%"), Some(7.5));
    }

    #[test]
    fn uses_latest_valid_percentage_in_a_progress_chunk() {
        assert_eq!(parse_external_progress("Vid: 42% Aud: 41%"), Some(41.0));
        assert_eq!(parse_external_progress("HTTP 403%"), None);
    }
}
