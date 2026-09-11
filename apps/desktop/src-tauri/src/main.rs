#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod diagnostics_v3;
mod prepared_preview;
mod tiktok_preview;

use apocalipse_core::{
    classify_url, cleanup_chunk_artifacts, contextual_media_page, partial_path, plan_download,
    BandwidthLimiter, Capabilities, DownloadEngine, DownloadEvent, DownloadId, DownloadKind,
    DownloadRequest, DownloadState, DownloadTask,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs,
    fs::OpenOptions,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State,
};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{mpsc, oneshot},
};

struct AppState {
    queue: Mutex<Vec<DownloadTask>>,
    queue_path: PathBuf,
    workers: Mutex<HashMap<DownloadId, oneshot::Sender<()>>>,
    settings: Mutex<UserSettings>,
    settings_path: PathBuf,
    bridge_last_seen: Mutex<Option<Instant>>,
    bridge_pending: Mutex<Vec<BridgeDownload>>,
    blob_uploads: Mutex<HashMap<uuid::Uuid, BlobUpload>>,
    recording_stops: Mutex<HashSet<DownloadId>>,
    request_identities: Mutex<HashMap<DownloadId, RequestIdentity>>,
    log_path: PathBuf,
    log_write_lock: Mutex<()>,
    diagnostics: diagnostics_v3::Diagnostics,
    global_bandwidth_limiter: Arc<BandwidthLimiter>,
    download_bandwidth_limiters: Mutex<HashMap<DownloadId, Arc<BandwidthLimiter>>>,
    tray_show: MenuItem<tauri::Wry>,
    tray_quit: MenuItem<tauri::Wry>,
}

fn host_from_url(url: &str) -> Option<String> {
    let scheme = url.find("://")? + 3;
    let authority = url[scheme..].split(['/', '?', '#']).next()?;
    let host = authority
        .rsplit('@')
        .next()?
        .split(':')
        .next()?
        .trim()
        .to_ascii_lowercase();
    (!host.is_empty()).then_some(host)
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
    #[serde(default = "default_true")]
    adaptive_efficiency: bool,
    #[serde(default)]
    global_bandwidth_limit: u64,
    #[serde(default = "default_bridge_token")]
    bridge_token: String,
    #[serde(default)]
    recent_download_directories: Vec<PathBuf>,
    #[serde(default)]
    ffmpeg_path: Option<PathBuf>,
    #[serde(default)]
    yt_dlp_path: Option<PathBuf>,
    #[serde(default)]
    qjs_path: Option<PathBuf>,
    #[serde(default)]
    n_m3u8dl_re_path: Option<PathBuf>,
    #[serde(default)]
    aria2_path: Option<PathBuf>,
    #[serde(default)]
    media_player_path: Option<PathBuf>,
    #[serde(default)]
    user_agent: Option<String>,
    #[serde(default)]
    log_editor_path: Option<PathBuf>,
    #[serde(default)]
    proxy_enabled: bool,
    #[serde(default)]
    proxy_url: Option<String>,
    #[serde(default)]
    proxy_username: Option<String>,
    #[serde(default)]
    proxy_password: Option<String>,
    #[serde(default)]
    website_credentials: Vec<WebsiteCredential>,
    #[serde(default)]
    dns_enabled: bool,
    #[serde(default)]
    dns_servers: Vec<std::net::IpAddr>,
    #[serde(default)]
    associations: HashMap<String, bool>,
    #[serde(default = "default_link_password")]
    link_password: String,
    #[serde(default = "default_language")]
    language: String,
    #[serde(default = "default_theme")]
    theme: String,
}

fn default_language() -> String {
    "en".to_owned()
}

fn default_theme() -> String {
    "void".to_owned()
}

fn tray_labels(language: &str) -> (&'static str, &'static str) {
    match language {
        "pt-BR" => ("Mostrar Apocalipse", "Sair"),
        "zh-CN" => ("显示 Apocalipse", "退出"),
        _ => ("Show Apocalipse", "Quit"),
    }
}

const fn default_max_active() -> usize {
    3
}
const fn default_connections() -> usize {
    8
}
const fn default_true() -> bool {
    true
}
fn default_bridge_token() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}
fn default_link_password() -> String {
    uuid::Uuid::new_v4().simple().to_string()[..8].to_ascii_uppercase()
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            download_directory: None,
            capture_clipboard: false,
            max_active_downloads: default_max_active(),
            connections_per_download: default_connections(),
            adaptive_efficiency: true,
            global_bandwidth_limit: 0,
            bridge_token: default_bridge_token(),
            recent_download_directories: Vec::new(),
            ffmpeg_path: None,
            yt_dlp_path: None,
            qjs_path: None,
            n_m3u8dl_re_path: None,
            aria2_path: None,
            media_player_path: None,
            user_agent: None,
            log_editor_path: None,
            proxy_enabled: false,
            proxy_url: None,
            proxy_username: None,
            proxy_password: None,
            website_credentials: Vec::new(),
            dns_enabled: false,
            dns_servers: Vec::new(),
            associations: HashMap::new(),
            link_password: default_link_password(),
            language: default_language(),
            theme: default_theme(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct WebsiteCredential {
    host: String,
    username: String,
    password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WebsiteCredentialSummary {
    host: String,
    username: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProxySetting {
    enabled: bool,
    url: String,
    username: String,
    has_password: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DnsSetting {
    enabled: bool,
    servers: Vec<String>,
}

#[derive(Serialize)]
struct PlanResponse {
    primary: String,
    fallbacks: Vec<String>,
    reason: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MediaFormat {
    selection: String,
    label: String,
}

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
struct TorrentFileInfo {
    index: usize,
    path: String,
    size: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TorrentInspection {
    name: String,
    files: Vec<TorrentFileInfo>,
    total_size: u64,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkFileEntry {
    name: String,
    path: String,
    size: u64,
    directory: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkListRequest {
    password: String,
    path: String,
}

#[derive(Deserialize)]
struct MobileAddRequest {
    url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkIdentity {
    id: String,
    password: String,
    port: u16,
}

fn safe_link_path(path: &str) -> Result<PathBuf, String> {
    let path = Path::new(path);
    if !path.is_absolute() {
        return Err("remote_path_must_be_absolute".to_owned());
    }
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Prefix(value) => result.push(value.as_os_str()),
            std::path::Component::RootDir => result.push(std::path::MAIN_SEPARATOR.to_string()),
            std::path::Component::Normal(value) => result.push(value),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => return Err("invalid_remote_path".to_owned()),
        }
    }
    Ok(result)
}

fn link_roots() -> Vec<LinkFileEntry> {
    #[cfg(windows)]
    {
        (b'A'..=b'Z')
            .filter_map(|letter| {
                let path = format!("{}:\\", letter as char);
                Path::new(&path).exists().then(|| LinkFileEntry {
                    name: path.clone(),
                    path,
                    size: 0,
                    directory: true,
                })
            })
            .collect()
    }
    #[cfg(not(windows))]
    {
        vec![LinkFileEntry {
            name: "/".to_owned(),
            path: "/".to_owned(),
            size: 0,
            directory: true,
        }]
    }
}

fn list_link_directory(path: &str) -> Result<Vec<LinkFileEntry>, String> {
    if path.trim().is_empty() {
        return Ok(link_roots());
    }
    let directory = safe_link_path(path)?;
    let mut entries = fs::read_dir(&directory)
        .map_err(|error| error.to_string())?
        .flatten()
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = directory.join(&name).to_string_lossy().into_owned();
            Some(LinkFileEntry {
                name,
                path,
                size: if metadata.is_file() {
                    metadata.len()
                } else {
                    0
                },
                directory: metadata.is_dir(),
            })
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| (!entry.directory, entry.name.to_ascii_lowercase()));
    Ok(entries)
}

#[tauri::command]
fn get_link_identity(state: State<'_, AppState>) -> Result<LinkIdentity, String> {
    let settings = state.settings.lock().map_err(|error| error.to_string())?;
    let ip = local_link_ip();
    Ok(LinkIdentity {
        id: format!("{ip}:{LINK_PORT}"),
        password: settings.link_password.clone(),
        port: LINK_PORT,
    })
}

#[tauri::command]
fn regenerate_link_password(state: State<'_, AppState>) -> Result<String, String> {
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.link_password = default_link_password();
    save_settings(&state, &settings)?;
    Ok(settings.link_password.clone())
}

#[tauri::command]
fn list_local_link_files(path: String) -> Result<Vec<LinkFileEntry>, String> {
    list_link_directory(&path)
}

#[tauri::command]
async fn list_remote_link_files(
    id: String,
    password: String,
    path: String,
) -> Result<Vec<LinkFileEntry>, String> {
    let address = if id.starts_with("http://") {
        id
    } else {
        format!("http://{id}")
    };
    reqwest::Client::new()
        .post(format!("{address}/v1/link/list"))
        .json(&LinkListRequest { password, path })
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?
        .json()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn download_remote_link_file(
    id: String,
    password: String,
    path: String,
) -> Result<String, String> {
    let file_name = Path::new(&path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("download");
    let Some(destination) = rfd::FileDialog::new().set_file_name(file_name).save_file() else {
        return Err("cancelled".to_owned());
    };
    let address = if id.starts_with("http://") {
        id
    } else {
        format!("http://{id}")
    };
    let encoded = url::form_urlencoded::byte_serialize(path.as_bytes()).collect::<String>();
    let response = reqwest::Client::new()
        .get(format!("{address}/v1/link/file?path={encoded}"))
        .bearer_auth(password)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    let mut file = tokio::fs::File::create(&destination)
        .await
        .map_err(|error| error.to_string())?;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        file.write_all(&chunk.map_err(|error| error.to_string())?)
            .await
            .map_err(|error| error.to_string())?;
    }
    Ok(destination.to_string_lossy().into_owned())
}

#[tauri::command]
async fn upload_remote_link_file(
    id: String,
    password: String,
    remote_directory: String,
    local_path: String,
) -> Result<String, String> {
    let source = PathBuf::from(local_path);
    if !source.is_file() {
        return Err("selected_local_file_not_found".to_owned());
    }
    if remote_directory.trim().is_empty() {
        return Err("select_remote_directory".to_owned());
    }
    tokio::task::spawn_blocking(move || {
        let name = source.file_name().and_then(|value| value.to_str()).ok_or_else(|| "invalid_file_name".to_owned())?;
        let remote_path = format!("{}/{}", remote_directory.trim_end_matches(['/', '\\']), name);
        let encoded = url::form_urlencoded::byte_serialize(remote_path.as_bytes()).collect::<String>();
        let parsed = url::Url::parse(&if id.starts_with("http://") { id } else { format!("http://{id}") }).map_err(|error| error.to_string())?;
        let host = parsed.host_str().ok_or_else(|| "invalid_remote_id".to_owned())?;
        let port = parsed.port().unwrap_or(LINK_PORT);
        let mut stream = TcpStream::connect((host, port)).map_err(|error| error.to_string())?;
        let mut file = fs::File::open(&source).map_err(|error| error.to_string())?;
        let size = file.metadata().map_err(|error| error.to_string())?.len();
        let header = format!("PUT /v1/link/file?path={encoded} HTTP/1.1\r\nHost: {host}\r\nAuthorization: Bearer {password}\r\nContent-Length: {size}\r\nConnection: close\r\n\r\n");
        stream.write_all(header.as_bytes()).map_err(|error| error.to_string())?;
        std::io::copy(&mut file, &mut stream).map_err(|error| error.to_string())?;
        let mut response = String::new();
        stream.read_to_string(&mut response).map_err(|error| error.to_string())?;
        if !response.starts_with("HTTP/1.1 200") { return Err("remote_upload_failed".to_owned()); }
        Ok(remote_path)
    }).await.map_err(|error| error.to_string())?
}

enum BValue {
    Int(i64),
    Bytes(Vec<u8>),
    List(Vec<BValue>),
    Dict(HashMap<Vec<u8>, BValue>),
}

fn parse_bencode(data: &[u8], position: &mut usize) -> Result<BValue, String> {
    let byte = *data
        .get(*position)
        .ok_or_else(|| "invalid_torrent".to_owned())?;
    match byte {
        b'i' => {
            *position += 1;
            let end = data[*position..]
                .iter()
                .position(|value| *value == b'e')
                .ok_or_else(|| "invalid_torrent".to_owned())?
                + *position;
            let value = std::str::from_utf8(&data[*position..end])
                .map_err(|_| "invalid_torrent")?
                .parse()
                .map_err(|_| "invalid_torrent")?;
            *position = end + 1;
            Ok(BValue::Int(value))
        }
        b'l' => {
            *position += 1;
            let mut values = Vec::new();
            while data.get(*position) != Some(&b'e') {
                values.push(parse_bencode(data, position)?);
            }
            *position += 1;
            Ok(BValue::List(values))
        }
        b'd' => {
            *position += 1;
            let mut values = HashMap::new();
            while data.get(*position) != Some(&b'e') {
                let BValue::Bytes(key) = parse_bencode(data, position)? else {
                    return Err("invalid_torrent".to_owned());
                };
                values.insert(key, parse_bencode(data, position)?);
            }
            *position += 1;
            Ok(BValue::Dict(values))
        }
        b'0'..=b'9' => {
            let colon = data[*position..]
                .iter()
                .position(|value| *value == b':')
                .ok_or_else(|| "invalid_torrent".to_owned())?
                + *position;
            let length: usize = std::str::from_utf8(&data[*position..colon])
                .map_err(|_| "invalid_torrent")?
                .parse()
                .map_err(|_| "invalid_torrent")?;
            let start = colon + 1;
            let end = start
                .checked_add(length)
                .filter(|end| *end <= data.len())
                .ok_or_else(|| "invalid_torrent".to_owned())?;
            *position = end;
            Ok(BValue::Bytes(data[start..end].to_vec()))
        }
        _ => Err("invalid_torrent".to_owned()),
    }
}

fn btext(value: Option<&BValue>) -> String {
    match value {
        Some(BValue::Bytes(bytes)) => String::from_utf8_lossy(bytes).into_owned(),
        _ => String::new(),
    }
}

fn inspect_torrent_file(path: &Path) -> Result<TorrentInspection, String> {
    let data = fs::read(path).map_err(|error| error.to_string())?;
    let mut position = 0;
    let BValue::Dict(root) = parse_bencode(&data, &mut position)? else {
        return Err("invalid_torrent".to_owned());
    };
    let Some(BValue::Dict(info)) = root.get(b"info".as_slice()) else {
        return Err("invalid_torrent".to_owned());
    };
    let name = btext(
        info.get(b"name.utf-8".as_slice())
            .or_else(|| info.get(b"name".as_slice())),
    );
    let mut files = Vec::new();
    if let Some(BValue::List(entries)) = info.get(b"files".as_slice()) {
        for (offset, entry) in entries.iter().enumerate() {
            let BValue::Dict(file) = entry else {
                continue;
            };
            let size = match file.get(b"length".as_slice()) {
                Some(BValue::Int(value)) if *value >= 0 => *value as u64,
                _ => 0,
            };
            let parts = match file
                .get(b"path.utf-8".as_slice())
                .or_else(|| file.get(b"path".as_slice()))
            {
                Some(BValue::List(parts)) => parts
                    .iter()
                    .map(|part| btext(Some(part)))
                    .filter(|part| !part.is_empty())
                    .collect::<Vec<_>>(),
                _ => Vec::new(),
            };
            files.push(TorrentFileInfo {
                index: offset + 1,
                path: parts.join("/"),
                size,
            });
        }
    } else if let Some(BValue::Int(length)) = info.get(b"length".as_slice()) {
        files.push(TorrentFileInfo {
            index: 1,
            path: name.clone(),
            size: (*length).max(0) as u64,
        });
    }
    if files.is_empty() {
        return Err("torrent_has_no_files".to_owned());
    }
    Ok(TorrentInspection {
        total_size: files.iter().map(|file| file.size).sum(),
        name,
        files,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolStatus {
    id: String,
    path: String,
    found: bool,
    version: Option<String>,
}

#[derive(Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MediaPreviewRequest {
    #[serde(default)]
    trace_id: Option<String>,
    url: String,
    #[serde(default)]
    audio_url: Option<String>,
    #[serde(default)]
    audio_cookie_header: Option<String>,
    #[serde(default)]
    media_kind: Option<String>,
    #[serde(default)]
    page_extractor: bool,
    user_agent: Option<String>,
    referer: Option<String>,
    cookie_header: Option<String>,
    content_type: Option<String>,
}

#[tauri::command]
fn get_app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn open_media_preview(
    app: &tauri::AppHandle,
    state: &AppState,
    request: MediaPreviewRequest,
) -> Result<(), String> {
    tiktok_preview::validate(&request)?;
    if prepared_preview::is_candidate(&request) {
        return prepared_preview::open(app, state, request);
    }
    if tiktok_preview::is_candidate(&request) {
        return tiktok_preview::open(app, request);
    }
    let parsed = url::Url::parse(&request.url).map_err(|_| "invalid_preview_url".to_owned())?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("invalid_preview_url".to_owned());
    }
    let player = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .media_player_path
        .clone()
        .unwrap_or_else(|| PathBuf::from(if cfg!(windows) { "vlc.exe" } else { "vlc" }));
    let effective_player = if cfg!(windows)
        && player
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("VLCPortable.exe"))
    {
        let bundled_vlc = player
            .parent()
            .map(|parent| parent.join("App").join("vlc").join("vlc.exe"));
        bundled_vlc
            .filter(|candidate| candidate.is_file())
            .unwrap_or_else(|| player.clone())
    } else {
        player.clone()
    };
    let mut command = Command::new(&effective_player);
    let player_name = effective_player
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if let Some(agent) = request.user_agent.filter(|value| !value.trim().is_empty()) {
        if player_name.contains("vlc") {
            command.arg(format!("--http-user-agent={agent}"));
        } else if player_name.contains("mpv") {
            command.arg(format!("--user-agent={agent}"));
        }
    }
    if player_name.contains("vlc") {
        command.arg("--http-reconnect");
    }
    if let Some(referer) = request
        .referer
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        if player_name.contains("vlc") {
            command.arg(format!("--http-referrer={referer}"));
        } else if player_name.contains("mpv") {
            command.arg(format!("--referrer={referer}"));
        }
    }
    command.arg(&request.url);
    diagnostic_log(
        state,
        "INFO",
        "media.preview_requested",
        &format!(
            "player={} configured_player={} url={} referer={}",
            effective_player.display(),
            player.display(),
            redact_url(&request.url),
            request
                .referer
                .as_deref()
                .and_then(host_from_url)
                .unwrap_or_else(|| "none".to_owned())
        ),
    );
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn configured_tool(path: &Option<PathBuf>, fallback: &str) -> PathBuf {
    path.clone()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| PathBuf::from(fallback))
}

fn http_origin(url: &str) -> Option<&str> {
    let scheme_end = url.find("://")? + 3;
    let path_start = url[scheme_end..]
        .find('/')
        .map(|index| scheme_end + index)
        .unwrap_or(url.len());
    Some(&url[..path_start])
}

fn version_line(executable: &Path, args: &[&str]) -> Option<String> {
    let mut command = Command::new(executable);
    command.args(args);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = if output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stderr)
    } else {
        String::from_utf8_lossy(&output.stdout)
    };
    text.lines()
        .next()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
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
struct AssociationStatus {
    id: String,
    enabled: bool,
    supported: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TransferLimits {
    max_active_downloads: usize,
    connections_per_download: usize,
    adaptive_efficiency: bool,
    global_bandwidth_limit: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UserAgentSetting {
    user_agent: String,
}

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
    #[serde(default)]
    trace_id: Option<String>,
    url: String,
    #[serde(default)]
    audio_url: Option<String>,
    file_name: Option<String>,
    page_url: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    thumbnail: Option<String>,
    #[serde(default)]
    media_kind: Option<String>,
    #[serde(default)]
    ambiguous_social_track: bool,
    #[serde(default)]
    expected_size: Option<u64>,
    duration: Option<f64>,
    cookie_header: Option<String>,
    user_agent: Option<String>,
    request_method: Option<String>,
    request_body: Option<String>,
    request_content_type: Option<String>,
    #[serde(default)]
    start_immediately: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BrowserDownloadComplete {
    url: String,
    file_name: String,
    total: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BridgeDownloadStatus {
    task_id: DownloadId,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtensionDiagnostic {
    event: String,
    level: Option<String>,
    trace_id: Option<String>,
    source: Option<String>,
    url: Option<String>,
    status: Option<u16>,
    bytes: Option<u64>,
    duration_ms: Option<u64>,
    detail: Option<String>,
    client_timestamp: Option<String>,
    delayed: Option<bool>,
}

struct BlobUpload {
    task_id: DownloadId,
    partial: PathBuf,
    destination: PathBuf,
    received: u64,
    total: Option<u64>,
    recording: bool,
    speed_sample_at: Instant,
    speed_sample_bytes: u64,
    smoothed_speed: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BlobBegin {
    file_name: String,
    total: u64,
    source: String,
    #[serde(default)]
    streaming: bool,
    #[serde(default)]
    recording: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BlobChunk {
    upload_id: uuid::Uuid,
    data: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BlobFinish {
    upload_id: uuid::Uuid,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadContext {
    #[serde(default)]
    trace_id: Option<String>,
    referer: Option<String>,
    known_duration: Option<f64>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    thumbnail: Option<String>,
    #[serde(default)]
    audio_url: Option<String>,
    cookie_header: Option<String>,
    user_agent: Option<String>,
    request_method: Option<String>,
    request_body: Option<String>,
    request_content_type: Option<String>,
}

#[derive(Clone)]
struct RequestIdentity {
    cookie_header: Option<String>,
    user_agent: Option<String>,
    request_method: String,
    request_body: Option<String>,
    request_content_type: Option<String>,
}

fn social_cookie_domain(url: &str) -> Option<&'static str> {
    let host = url::Url::parse(url).ok()?.host_str()?.to_ascii_lowercase();
    ["facebook.com", "instagram.com", "tiktok.com"]
        .into_iter()
        .find(|domain| host == *domain || host.ends_with(&format!(".{domain}")))
}

fn write_social_cookie_jar(path: &Path, url: &str, header: &str) -> Result<(), String> {
    let domain = social_cookie_domain(url).ok_or_else(|| "unsupported_cookie_domain".to_owned())?;
    let mut jar = String::from("# Netscape HTTP Cookie File\n");
    for item in header.split(';') {
        let Some((name, value)) = item.trim().split_once('=') else {
            continue;
        };
        if name.is_empty()
            || name
                .chars()
                .any(|character| matches!(character, '\t' | '\r' | '\n'))
            || value
                .chars()
                .any(|character| matches!(character, '\t' | '\r' | '\n'))
        {
            continue;
        }
        jar.push_str(&format!(".{domain}\tTRUE\t/\tTRUE\t0\t{name}\t{value}\n"));
    }
    fs::write(path, jar).map_err(|error| error.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DestinationChoice {
    path: String,
    is_default: bool,
    available: bool,
}

const BRIDGE_PORT: u16 = 17654;
const LINK_PORT: u16 = 17655;

fn local_link_ip() -> std::net::IpAddr {
    "127.0.0.1".parse().expect("valid loopback")
}

fn handle_link_connection(app: &tauri::AppHandle, mut stream: TcpStream) {
    let mut buffer = Vec::with_capacity(8192);
    let mut chunk = [0_u8; 4096];
    let header_end = loop {
        let Ok(count) = stream.read(&mut chunk) else {
            return;
        };
        if count == 0 || buffer.len() + count > 65_536 {
            return;
        }
        buffer.extend_from_slice(&chunk[..count]);
        if let Some(position) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
            break position;
        }
    };
    let headers = String::from_utf8_lossy(&buffer[..header_end + 4]).into_owned();
    if headers.starts_with("GET /mobile ") {
        let page = include_str!("../mobile.html");
        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Security-Policy: default-src 'self' 'unsafe-inline'\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{page}", page.len());
        let _ = stream.write_all(response.as_bytes());
        return;
    }
    if headers.starts_with("GET /v1/mobile/tasks ") {
        let state = app.state::<AppState>();
        let settings = match state.settings.lock() {
            Ok(value) => value,
            Err(_) => return,
        };
        if !bridge_authorized(&headers, &settings.link_password) {
            bridge_response(
                &mut stream,
                "401 Unauthorized",
                None,
                "{\"error\":\"unauthorized\"}",
            );
            return;
        }
        drop(settings);
        let queue = match state.queue.lock() {
            Ok(value) => value,
            Err(_) => return,
        };
        let body = serde_json::to_string(&*queue).unwrap_or_else(|_| "[]".to_owned());
        bridge_response(&mut stream, "200 OK", None, &body);
        return;
    }
    if headers.starts_with("PUT /v1/link/file?") {
        let state = app.state::<AppState>();
        let settings = match state.settings.lock() {
            Ok(value) => value,
            Err(_) => return,
        };
        if !bridge_authorized(&headers, &settings.link_password) {
            bridge_response(&mut stream, "401 Unauthorized", None, "");
            return;
        }
        drop(settings);
        let request_target = headers
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("");
        let path = url::Url::parse(&format!("http://localhost{request_target}"))
            .ok()
            .and_then(|url| {
                url.query_pairs()
                    .find(|(key, _)| key == "path")
                    .map(|(_, value)| value.into_owned())
            });
        let Some(path) = path.and_then(|value| safe_link_path(&value).ok()) else {
            bridge_response(&mut stream, "400 Bad Request", None, "");
            return;
        };
        let length = bridge_content_length(&headers);
        let Ok(mut file) = fs::File::create(path) else {
            bridge_response(&mut stream, "403 Forbidden", None, "");
            return;
        };
        let body_start = header_end + 4;
        let initial = &buffer[body_start..];
        if file.write_all(initial).is_err() {
            return;
        }
        let remaining = length.saturating_sub(initial.len());
        if std::io::copy(
            &mut std::io::Read::by_ref(&mut stream).take(remaining as u64),
            &mut file,
        )
        .is_err()
        {
            return;
        }
        bridge_response(&mut stream, "200 OK", None, "{\"ok\":true}");
        return;
    }
    let length = bridge_content_length(&headers);
    let body_start = header_end + 4;
    while buffer.len() < body_start + length {
        let Ok(count) = stream.read(&mut chunk) else {
            return;
        };
        if count == 0 {
            return;
        }
        buffer.extend_from_slice(&chunk[..count]);
    }
    let Some(header_end) = buffer.windows(4).position(|window| window == b"\r\n\r\n") else {
        return;
    };
    let headers = String::from_utf8_lossy(&buffer[..header_end + 4]);
    if headers.starts_with("POST /v1/mobile/add ") {
        let state = app.state::<AppState>();
        let settings = match state.settings.lock() {
            Ok(value) => value,
            Err(_) => return,
        };
        if !bridge_authorized(&headers, &settings.link_password) {
            bridge_response(
                &mut stream,
                "401 Unauthorized",
                None,
                "{\"error\":\"unauthorized\"}",
            );
            return;
        }
        drop(settings);
        let request = serde_json::from_slice::<MobileAddRequest>(&buffer[header_end + 4..]);
        match request {
            Ok(request) if classify_url(&request.url).is_some() => {
                let result = queue_from_bridge(
                    app,
                    BridgeDownload {
                        trace_id: None,
                        url: request.url,
                        audio_url: None,
                        file_name: None,
                        page_url: None,
                        title: None,
                        thumbnail: None,
                        media_kind: None,
                        ambiguous_social_track: false,
                        expected_size: None,
                        duration: None,
                        cookie_header: None,
                        user_agent: None,
                        request_method: None,
                        request_body: None,
                        request_content_type: None,
                        start_immediately: false,
                    },
                );
                if result.is_ok() {
                    bridge_response(&mut stream, "200 OK", None, "{\"ok\":true}");
                } else {
                    bridge_response(
                        &mut stream,
                        "400 Bad Request",
                        None,
                        "{\"error\":\"invalid_url\"}",
                    );
                }
            }
            _ => bridge_response(
                &mut stream,
                "400 Bad Request",
                None,
                "{\"error\":\"invalid_url\"}",
            ),
        }
        return;
    }
    if headers.starts_with("GET /v1/link/file?") {
        let state = app.state::<AppState>();
        let settings = match state.settings.lock() {
            Ok(value) => value,
            Err(_) => return,
        };
        if !bridge_authorized(&headers, &settings.link_password) {
            bridge_response(&mut stream, "401 Unauthorized", None, "");
            return;
        }
        let request_target = headers
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("");
        let path = url::Url::parse(&format!("http://localhost{request_target}"))
            .ok()
            .and_then(|url| {
                url.query_pairs()
                    .find(|(key, _)| key == "path")
                    .map(|(_, value)| value.into_owned())
            });
        let Some(path) = path.and_then(|value| safe_link_path(&value).ok()) else {
            bridge_response(&mut stream, "400 Bad Request", None, "");
            return;
        };
        let Ok(mut file) = fs::File::open(&path) else {
            bridge_response(&mut stream, "404 Not Found", None, "");
            return;
        };
        let Ok(metadata) = file.metadata() else {
            return;
        };
        let header = format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", metadata.len());
        if stream.write_all(header.as_bytes()).is_ok() {
            let _ = std::io::copy(&mut file, &mut stream);
        }
        return;
    }
    if !headers.starts_with("POST /v1/link/list ") {
        bridge_response(
            &mut stream,
            "404 Not Found",
            None,
            "{\"error\":\"not_found\"}",
        );
        return;
    }
    let request = serde_json::from_slice::<LinkListRequest>(&buffer[header_end + 4..]);
    let Ok(request) = request else {
        bridge_response(
            &mut stream,
            "400 Bad Request",
            None,
            "{\"error\":\"invalid_request\"}",
        );
        return;
    };
    let state = app.state::<AppState>();
    let settings = match state.settings.lock() {
        Ok(value) => value,
        Err(_) => return,
    };
    if request.password != settings.link_password {
        bridge_response(
            &mut stream,
            "401 Unauthorized",
            None,
            "{\"error\":\"unauthorized\"}",
        );
        return;
    }
    match list_link_directory(&request.path)
        .and_then(|entries| serde_json::to_string(&entries).map_err(|error| error.to_string()))
    {
        Ok(body) => bridge_response(&mut stream, "200 OK", None, &body),
        Err(_) => bridge_response(
            &mut stream,
            "400 Bad Request",
            None,
            "{\"error\":\"invalid_path\"}",
        ),
    }
}

fn run_link_server(app: tauri::AppHandle, listener: TcpListener) {
    for stream in listener.incoming().flatten() {
        let app = app.clone();
        let _ = std::thread::Builder::new()
            .name("apocalipse-link-client".into())
            .spawn(move || handle_link_connection(&app, stream));
    }
}

#[tauri::command]
fn inspect_url(url: String) -> Result<PlanResponse, String> {
    let capabilities = Capabilities {
        aria2: true,
        yt_dlp: true,
        n_m3u8dl_re: true,
        torrent: false,
    };
    let plan = plan_download(&url, capabilities).ok_or_else(|| "unsupported_url".to_owned())?;
    Ok(PlanResponse {
        primary: format!("{:?}", plan.primary),
        fallbacks: plan
            .fallbacks
            .iter()
            .map(|engine| format!("{engine:?}"))
            .collect(),
        reason: plan.reason.to_owned(),
    })
}

#[tauri::command]
async fn inspect_media_formats(
    state: State<'_, AppState>,
    url: String,
    cookie_header: Option<String>,
    user_agent: Option<String>,
    referer: Option<String>,
) -> Result<MediaInspection, String> {
    log_network_route(&state, "media_inspection", "YtDlp").await;
    let (executable, quickjs, credential) = {
        let settings = state.settings.lock().map_err(|error| error.to_string())?;
        (
            configured_tool(&settings.yt_dlp_path, "yt-dlp"),
            configured_tool(
                &settings.qjs_path,
                if cfg!(windows) { "qjs.exe" } else { "qjs" },
            ),
            website_credential_for_url(&settings, &url).cloned(),
        )
    };
    let mut command = tokio::process::Command::new(executable);
    command
        .args([
            "--dump-single-json",
            "--no-playlist",
            "--skip-download",
            "--no-warnings",
        ])
        .arg("--js-runtimes")
        .arg(format!("quickjs:{}", quickjs.display()))
        .arg(&url);
    let cookie_jar = cookie_header
        .as_deref()
        .filter(|value| !value.is_empty())
        .and_then(|cookie| {
            let path = state
                .queue_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(format!("inspect-cookies-{}.txt", uuid::Uuid::new_v4()));
            write_social_cookie_jar(&path, &url, cookie)
                .ok()
                .map(|_| path)
        });
    if let Some(path) = cookie_jar.as_ref() {
        command.arg("--cookies").arg(path);
    } else if let Some(cookie) = cookie_header.as_deref().filter(|value| !value.is_empty()) {
        command.arg("--add-headers").arg(format!("Cookie:{cookie}"));
    } else if url.contains("youtube.com/") || url.contains("youtu.be/") {
        // Manual URLs do not carry an extension identity. Reuse the browser
        // session so inspection and the actual download see the same YouTube
        // authentication/challenge state.
        command.args(["--cookies-from-browser", "chrome"]);
    }
    if let Some(value) = user_agent.as_deref().filter(|value| !value.is_empty()) {
        command.arg("--user-agent").arg(value);
    }
    if let Some(value) = referer.as_deref().filter(|value| !value.is_empty()) {
        command.arg("--referer").arg(value);
    }
    diagnostic_log(
        &state,
        "INFO",
        "yt_dlp.inspection_identity",
        &format!(
            "url={} cookies={} user_agent={} referer={}",
            redact_url(&url),
            cookie_header.as_deref().map_or(0, |value| value
                .split(';')
                .filter(|item| item.contains('='))
                .count()),
            user_agent.is_some(),
            referer
                .as_deref()
                .map(redact_url)
                .unwrap_or_else(|| "none".to_owned()),
        ),
    );
    if let Some(credential) = credential {
        command
            .arg("--username")
            .arg(credential.username)
            .arg("--password")
            .arg(credential.password);
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.as_std_mut().creation_flags(0x08000000);
    }
    let output = command
        .output()
        .await
        .map_err(|error| format!("yt_dlp_unavailable: {error}"))?;
    if let Some(path) = cookie_jar {
        let _ = fs::remove_file(path);
    }
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())?;
    let title = value
        .get("title")
        .and_then(|item| item.as_str())
        .unwrap_or("media")
        .to_owned();
    let thumbnail = value
        .get("thumbnail")
        .and_then(|item| item.as_str())
        .map(str::to_owned);
    let duration = value.get("duration").and_then(|item| item.as_f64());
    let mut formats = value
        .get("formats")
        .and_then(|item| item.as_array())
        .into_iter()
        .flatten()
        .filter_map(|format| {
            let id = format.get("format_id")?.as_str()?;
            let vcodec = format
                .get("vcodec")
                .and_then(|item| item.as_str())
                .unwrap_or("none");
            let acodec = format
                .get("acodec")
                .and_then(|item| item.as_str())
                .unwrap_or("none");
            if vcodec == "none" {
                return None;
            }
            let height = format
                .get("height")
                .and_then(|item| item.as_u64())
                .map(|value| format!("{value}p"))
                .unwrap_or_else(|| "video".into());
            let fps = format
                .get("fps")
                .and_then(|item| item.as_f64())
                .map(|value| format!(" · {} fps", value.round()))
                .unwrap_or_default();
            let extension = format
                .get("ext")
                .and_then(|item| item.as_str())
                .unwrap_or("");
            let size = format
                .get("filesize")
                .or_else(|| format.get("filesize_approx"))
                .and_then(|item| item.as_u64())
                .map(|value| format!(" · {:.1} MB", value as f64 / 1_048_576.0))
                .unwrap_or_default();
            let selection = if acodec == "none" {
                format!("{id}+bestaudio/best")
            } else {
                id.to_owned()
            };
            Some(MediaFormat {
                selection,
                label: format!("{height}{fps} · {extension}{size}"),
            })
        })
        .collect::<Vec<_>>();
    formats.reverse();
    formats.truncate(120);
    let safe_title = title
        .chars()
        .map(|character| {
            if "<>:\"/\\|?*".contains(character) {
                '_'
            } else {
                character
            }
        })
        .collect::<String>();
    Ok(MediaInspection {
        title,
        thumbnail,
        duration,
        suggested_file_name: format!("{safe_title}.mp4"),
        formats,
    })
}

fn load_queue(path: &Path) -> Vec<DownloadTask> {
    fs::read(path)
        .ok()
        .and_then(|data| serde_json::from_slice(&data).ok())
        .unwrap_or_default()
}

fn copy_directory_if_missing(source: &Path, destination: &Path) {
    if !source.is_dir() || destination.exists() {
        return;
    }
    if fs::create_dir_all(destination).is_err() {
        return;
    }
    let Ok(entries) = fs::read_dir(source) else {
        return;
    };
    for entry in entries.flatten() {
        let target = destination.join(entry.file_name());
        if entry.path().is_dir() {
            copy_directory_if_missing(&entry.path(), &target);
        } else if !target.exists() {
            let _ = fs::copy(entry.path(), target);
        }
    }
}

fn portable_data_directory<R: tauri::Runtime>(
    app: &tauri::App<R>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let executable = std::env::current_exe()?;
    let directory = executable
        .parent()
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "executable_has_no_parent")
        })?
        .join("data");
    fs::create_dir_all(&directory)?;
    let legacy = app.path().app_data_dir()?;
    for name in ["queue.json", "settings.json"] {
        let source = legacy.join(name);
        let destination = directory.join(name);
        if source.is_file() && !destination.exists() {
            let _ = fs::copy(source, destination);
        }
    }
    copy_directory_if_missing(&legacy.join("logs"), &directory.join("logs"));
    Ok(directory)
}

fn save_queue(state: &AppState, queue: &[DownloadTask]) -> Result<(), String> {
    if let Some(parent) = state.queue_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let data = serde_json::to_vec_pretty(queue).map_err(|error| error.to_string())?;
    let result = fs::write(&state.queue_path, data).map_err(|error| error.to_string());
    state.diagnostics.record("queue.persisted", if result.is_ok() { "INFO" } else { "ERROR" }, None, None,
        serde_json::json!({"ok":result.is_ok(),"taskCount":queue.len(),"taskRefs":queue.iter().take(24).map(|t|t.id.to_string()).collect::<Vec<_>>(),"truncated":queue.len()>24}));
    result
}

fn load_settings(path: &Path) -> UserSettings {
    let settings: UserSettings = fs::read(path)
        .ok()
        .and_then(|data| serde_json::from_slice(&data).ok())
        .unwrap_or_default();
    settings
}

fn save_settings(state: &AppState, settings: &UserSettings) -> Result<(), String> {
    if let Some(parent) = state.settings_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let data = serde_json::to_vec_pretty(settings).map_err(|error| error.to_string())?;
    fs::write(&state.settings_path, data).map_err(|error| error.to_string())
}

fn redact_url(url: &str) -> String {
    let (base, query) = match url.split_once('?') {
        Some(parts) => parts,
        None => (url.split('#').next().unwrap_or(url), ""),
    };
    let base = if let Some(scheme_end) = base.find("://") {
        let authority_start = scheme_end + 3;
        let authority_end = base[authority_start..]
            .find('/')
            .map_or(base.len(), |index| authority_start + index);
        match base[authority_start..authority_end].find('@') {
            Some(at) => format!(
                "{}{}",
                &base[..authority_start],
                &base[authority_start + at + 1..]
            ),
            None => base.to_owned(),
        }
    } else {
        base.to_owned()
    };
    let parameters = query
        .split('#')
        .next()
        .unwrap_or_default()
        .split('&')
        .filter(|item| !item.is_empty())
        .map(|item| {
            format!(
                "{}=<redacted>",
                item.split_once('=').map_or(item, |(name, _)| name)
            )
        })
        .collect::<Vec<_>>();
    if parameters.is_empty() {
        base
    } else {
        format!("{base}?{}", parameters.join("&"))
    }
}

fn sanitize_log_detail(detail: &str) -> String {
    let lowered = detail.to_ascii_lowercase();
    if [
        "cookie:",
        "cookie=",
        "authorization:",
        "authorization=",
        "password:",
        "password=",
        "passwd:",
        "passwd=",
    ]
    .iter()
    .any(|marker| lowered.contains(marker))
    {
        return "<redacted>".to_owned();
    }
    detail
        .split_whitespace()
        .map(|part| {
            if let Some(index) = part.find("http://").or_else(|| part.find("https://")) {
                let (prefix, url) = part.split_at(index);
                return format!("{prefix}{}", redact_url(url));
            }
            part.to_owned()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn diagnostic_log(state: &AppState, level: &str, event: &str, detail: &str) {
    state.diagnostics.observe_legacy(level, event, detail);
    let _write_guard = match state.log_write_lock.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    if let Some(parent) = state.log_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if fs::metadata(&state.log_path).is_ok_and(|metadata| metadata.len() > 2 * 1024 * 1024) {
        let rotated = state.log_path.with_extension("log.1");
        let _ = fs::remove_file(&rotated);
        let _ = fs::rename(&state.log_path, rotated);
    }
    let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&state.log_path)
    {
        let record = serde_json::json!({
            "timestamp": timestamp,
            "level": level,
            "event": event,
            "source": "desktop",
            "detail": sanitize_log_detail(detail),
        });
        let _ = writeln!(file, "{record}");
    }
}

fn remember_download_directory(state: &AppState, directory: &Path) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings
        .recent_download_directories
        .retain(|path| path != directory);
    settings
        .recent_download_directories
        .insert(0, directory.to_path_buf());
    settings.recent_download_directories.truncate(20);
    save_settings(state, &settings)
}

fn configured_download_directory(
    app: &tauri::AppHandle,
    state: &AppState,
) -> Result<PathBuf, String> {
    if let Some(path) = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .download_directory
        .clone()
    {
        return Ok(path);
    }
    app.path().download_dir().map_err(|error| error.to_string())
}

fn update_task(
    app: &tauri::AppHandle,
    id: DownloadId,
    persist: bool,
    update: impl FnOnce(&mut DownloadTask),
) {
    let state = app.state::<AppState>();
    let mut queue = match state.queue.lock() {
        Ok(queue) => queue,
        Err(_) => return,
    };
    if let Some(task) = queue.iter_mut().find(|task| task.id == id) {
        let before = std::mem::discriminant(&task.state);
        update(task);
        if before != std::mem::discriminant(&task.state) {
            state.diagnostics.record("task.state_changed", "INFO", None, Some(&id.to_string()),
                serde_json::json!({"state":serde_json::to_value(&task.state).ok().map(|v| match v { serde_json::Value::String(name) => name, serde_json::Value::Object(fields) => fields.keys().next().cloned().unwrap_or_default(), _ => "unknown".into() }),"bytes":task.received,"persistRequested":persist}));
        }
        if persist {
            let _ = save_queue(&state, &queue);
        }
    }
}

async fn run_adaptive_social_download(
    app: tauri::AppHandle,
    id: DownloadId,
    task: DownloadTask,
    mut cancellation: oneshot::Receiver<()>,
) {
    let Some(audio_url) = task.companion_audio_url.clone() else {
        return;
    };
    update_task(&app, id, true, |item| {
        item.state = DownloadState::Downloading;
        item.progress_percent = Some(0.0);
        item.resume_supported = Some(false);
    });
    let state = app.state::<AppState>();
    let ffmpeg = state
        .settings
        .lock()
        .map(|settings| {
            configured_tool(
                &settings.ffmpeg_path,
                if cfg!(windows) {
                    "ffmpeg.exe"
                } else {
                    "ffmpeg"
                },
            )
        })
        .unwrap_or_else(|_| {
            PathBuf::from(if cfg!(windows) {
                "ffmpeg.exe"
            } else {
                "ffmpeg"
            })
        });
    let identity = state
        .request_identities
        .lock()
        .ok()
        .and_then(|items| items.get(&id).cloned());
    let user_agent = identity
        .as_ref()
        .and_then(|value| value.user_agent.as_deref())
        .unwrap_or(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/152.0.0.0 Safari/537.36",
        );
    let mut headers = String::new();
    if let Some(referer) = task.referer.as_deref() {
        headers.push_str(&format!("Referer: {referer}\r\n"));
    }
    if let Some(cookie) = identity
        .as_ref()
        .and_then(|value| value.cookie_header.as_deref())
    {
        headers.push_str(&format!("Cookie: {cookie}\r\n"));
    }
    let temporary = task
        .destination
        .with_file_name(format!(".{id}.apocalipse-muxing.mp4"));
    let _ = fs::remove_file(&temporary);
    diagnostic_log(
        &state,
        "INFO",
        "adaptive_media.mux_start",
        &format!(
            "task={id} video={} audio={} destination={}",
            redact_url(&task.source),
            redact_url(&audio_url),
            task.destination.display()
        ),
    );
    let mut command = tokio::process::Command::new(ffmpeg);
    command.args(["-y", "-loglevel", "warning", "-user_agent", user_agent]);
    if !headers.is_empty() {
        command.args(["-headers", &headers]);
    }
    command.arg("-i").arg(&task.source);
    command.args(["-user_agent", user_agent]);
    if !headers.is_empty() {
        command.args(["-headers", &headers]);
    }
    command
        .arg("-i")
        .arg(&audio_url)
        .args([
            "-map",
            "0:v:0",
            "-map",
            "1:a:0",
            "-c:v",
            "copy",
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-movflags",
            "+faststart",
        ])
        .arg(&temporary)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.as_std_mut().creation_flags(0x08000000);
    }
    let result = match command.spawn() {
        Ok(mut child) => {
            let stderr = child
                .stderr
                .take()
                .map(|stream| tauri::async_runtime::spawn(read_process_tail(stream, None)));
            let status = tokio::select! {
                biased;
                _ = &mut cancellation => { let _ = child.kill().await; None }
                value = child.wait() => value.ok(),
            };
            let error_text = match stderr {
                Some(output) => {
                    String::from_utf8_lossy(&output.await.unwrap_or_default()).into_owned()
                }
                None => String::new(),
            };
            match status {
                Some(status) if status.success() && temporary.is_file() => {
                    fs::rename(&temporary, &task.destination).map_err(|error| error.to_string())
                }
                Some(status) => Err(external_error_detail(&error_text, status.code())),
                None => Err("cancelled".to_owned()),
            }
        }
        Err(error) => Err(format!("ffmpeg_unavailable: {error}")),
    };
    let _ = fs::remove_file(&temporary);
    match result {
        Ok(()) => {
            let size = fs::metadata(&task.destination)
                .map(|value| value.len())
                .unwrap_or_default();
            diagnostic_log(
                &state,
                "INFO",
                "adaptive_media.mux_completed",
                &format!("task={id} bytes={size}"),
            );
            update_task(&app, id, true, |item| {
                item.received = size;
                item.total = Some(size);
                item.progress_percent = Some(100.0);
                item.state = DownloadState::Completed;
                item.completed_at = Some(epoch_seconds());
            });
        }
        Err(message) if message == "cancelled" => {
            update_task(&app, id, true, |item| item.state = DownloadState::Paused);
        }
        Err(message) => {
            diagnostic_log(
                &state,
                "ERROR",
                "adaptive_media.mux_failed",
                &format!("task={id} error={message}"),
            );
            update_task(&app, id, true, |item| {
                item.state = DownloadState::Failed { message }
            });
        }
    }
    if let Ok(mut workers) = state.workers.lock() {
        workers.remove(&id);
    }
    start_next_queued(&app);
}

async fn run_download(
    app: tauri::AppHandle,
    id: DownloadId,
    request: DownloadRequest,
    mirrors: Vec<String>,
    mut cancellation: oneshot::Receiver<()>,
) {
    diagnostic_log(
        &app.state::<AppState>(),
        "INFO",
        "http.start",
        &format!("task={id} url={}", redact_url(&request.url)),
    );
    log_network_route(&app.state::<AppState>(), &id.to_string(), "NativeHttp").await;
    update_task(&app, id, true, |task| {
        task.state = DownloadState::Inspecting
    });
    let network = app
        .state::<AppState>()
        .settings
        .lock()
        .ok()
        .map(|settings| {
            let proxy = settings.proxy_enabled.then(|| {
                (
                    settings.proxy_url.clone(),
                    settings.proxy_username.clone(),
                    settings.proxy_password.clone(),
                )
            });
            let dns = if settings.dns_enabled {
                settings.dns_servers.clone()
            } else {
                Vec::new()
            };
            (proxy, dns)
        });
    let engine_result = match network {
        Some((proxy, dns)) => {
            let (url, username, password) = proxy.unwrap_or_default();
            DownloadEngine::with_network(
                url.as_deref(),
                username.as_deref(),
                password.as_deref(),
                &dns,
            )
        }
        None => DownloadEngine::new(),
    };
    let engine = match engine_result {
        Ok(engine) => engine,
        Err(error) => {
            diagnostic_log(
                &app.state::<AppState>(),
                "ERROR",
                "http.engine",
                &format!("task={id} error={error}"),
            );
            update_task(&app, id, true, |task| {
                task.state = DownloadState::Failed {
                    message: error.to_string(),
                }
            });
            return;
        }
    };
    let (events, mut receiver) = mpsc::channel(64);
    let mut download = Box::pin(download_with_mirrors(engine, request, mirrors, events));
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
                    Ok(()) => {
                        diagnostic_log(&app.state::<AppState>(), "INFO", "http.completed", &format!("task={id}"));
                        update_task(&app, id, true, |task| {
                            task.state = DownloadState::Completed;
                            task.completed_at = Some(epoch_seconds());
                        });
                    },
                    Err(error) => {
                        diagnostic_log(&app.state::<AppState>(), "ERROR", "http.failed", &format!("task={id} error={error}"));
                        update_task(&app, id, true, |task| task.state = DownloadState::Failed { message: error.to_string() });
                    },
                }
                break;
            }
            event = receiver.recv() => match event {
                Some(DownloadEvent::Started { resumed_at, total, connections, resume_supported }) => {
                    diagnostic_log(&app.state::<AppState>(), "INFO", "http.mode", &format!("task={id} connections={connections} segmented={}", connections > 1));
                    update_task(&app, id, true, |task| {
                        task.state = DownloadState::Downloading;
                        task.received = resumed_at;
                        task.total = total;
                        task.resume_supported = Some(resume_supported);
                    });
                },
                Some(DownloadEvent::Progress { received, total }) => update_task(&app, id, false, |task| {
                    task.received = received;
                    task.total = total;
                }),
                Some(DownloadEvent::Completed { bytes }) => update_task(&app, id, true, |task| {
                    task.received = bytes;
                    task.total = Some(bytes);
                    task.state = DownloadState::Completed;
                    task.completed_at = Some(epoch_seconds());
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

async fn download_with_mirrors(
    engine: DownloadEngine,
    request: DownloadRequest,
    mirrors: Vec<String>,
    events: mpsc::Sender<DownloadEvent>,
) -> anyhow::Result<()> {
    let mut sources = vec![request.url.clone()];
    sources.extend(mirrors);
    let mut last_error = None;
    for source in sources {
        let mut attempt = request.clone();
        attempt.url = source;
        match engine.download(attempt, events.clone()).await {
            Ok(()) => return Ok(()),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("no_download_source")))
}

fn finalize_media_page_download(
    app: &tauri::AppHandle,
    id: DownloadId,
    requested_destination: &Path,
    work_directory: &Path,
) -> Result<(), String> {
    let mut candidates = fs::read_dir(work_directory)
        .map_err(|error| error.to_string())?
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let metadata = entry.metadata().ok()?;
            let name = path.file_name()?.to_string_lossy().to_ascii_lowercase();
            (metadata.is_file()
                && metadata.len() > 0
                && !name.contains(".part")
                && !name.ends_with(".ytdl"))
            .then_some((path, metadata.len()))
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(_, size)| std::cmp::Reverse(*size));
    let (source, size) = candidates
        .into_iter()
        .next()
        .ok_or_else(|| "yt_dlp_final_file_missing".to_owned())?;
    let source_extension = source.extension().and_then(|value| value.to_str());
    let destination = if requested_destination.extension().is_none() {
        source_extension
            .map(|extension| requested_destination.with_extension(extension))
            .unwrap_or_else(|| requested_destination.to_path_buf())
    } else {
        requested_destination.to_path_buf()
    };
    if destination.exists() {
        return Err("yt_dlp_destination_already_exists".to_owned());
    }
    fs::copy(&source, &destination).map_err(|error| error.to_string())?;
    let _ = fs::remove_dir_all(work_directory);
    update_task(app, id, true, |item| {
        item.destination = destination.clone();
        item.received = size;
        item.total = Some(size);
    });
    diagnostic_log(
        &app.state::<AppState>(),
        "INFO",
        "yt_dlp.output_committed",
        &format!(
            "task={id} bytes={size} extension={} destination={}",
            source_extension.unwrap_or("none"),
            destination.display()
        ),
    );
    Ok(())
}

fn epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |value| value.as_secs())
}

async fn log_network_route(state: &AppState, operation: &str, engine: &str) {
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = tokio::process::Command::new("powershell.exe");
        command.args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            r#"$ErrorActionPreference='SilentlyContinue'; $routes=@(Get-NetRoute -AddressFamily IPv4 -DestinationPrefix '0.0.0.0/0' | Sort-Object RouteMetric,InterfaceMetric | ForEach-Object { $r=$_; $a=Get-NetAdapter -InterfaceIndex $r.InterfaceIndex; $dns=(Get-DnsClientServerAddress -InterfaceIndex $r.InterfaceIndex -AddressFamily IPv4).ServerAddresses; [pscustomobject]@{ifIndex=$r.InterfaceIndex; interface=$r.InterfaceAlias; description=$a.InterfaceDescription; status=$a.Status; metric=($r.RouteMetric+$r.InterfaceMetric); nextHop=$r.NextHop; dns=@($dns); vpnCandidate=([bool](($r.InterfaceAlias+' '+$a.InterfaceDescription) -match '(?i)vpn|avira|phantom|wireguard|wintun|openvpn|tap|tun|tailscale|zerotier'))} }); [pscustomobject]@{vpnDetected=([bool]($routes | Where-Object { $_.vpnCandidate })); routes=$routes} | ConvertTo-Json -Compress -Depth 5"#,
        ]);
        use std::os::windows::process::CommandExt;
        command.as_std_mut().creation_flags(0x08000000);
        command
    };
    #[cfg(target_os = "linux")]
    let mut command = {
        let mut command = tokio::process::Command::new("sh");
        command.args([
            "-c",
            "ip -j route show default 2>/dev/null || ip route show default 2>/dev/null",
        ]);
        command
    };
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = tokio::process::Command::new("sh");
        command.args([
            "-c",
            "route -n get default 2>/dev/null; scutil --proxy 2>/dev/null",
        ]);
        command
    };
    match tokio::time::timeout(Duration::from_secs(5), command.output()).await {
        Ok(Ok(output)) if output.status.success() => {
            let snapshot = String::from_utf8_lossy(&output.stdout)
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .take(12_000)
                .collect::<String>();
            diagnostic_log(
                state,
                "INFO",
                "network.route_snapshot",
                &format!("operation={operation} engine={engine} snapshot={snapshot}"),
            );
        }
        Ok(Ok(output)) => diagnostic_log(
            state,
            "WARN",
            "network.route_snapshot_failed",
            &format!(
                "operation={operation} engine={engine} status={}",
                output.status
            ),
        ),
        Ok(Err(error)) => diagnostic_log(
            state,
            "WARN",
            "network.route_snapshot_failed",
            &format!("operation={operation} engine={engine} error={error}"),
        ),
        Err(_) => diagnostic_log(
            state,
            "WARN",
            "network.route_snapshot_failed",
            &format!("operation={operation} engine={engine} error=timeout"),
        ),
    }
}

async fn run_external_download(
    app: tauri::AppHandle,
    id: DownloadId,
    task: DownloadTask,
    kind: DownloadKind,
    mut cancellation: oneshot::Receiver<()>,
) {
    diagnostic_log(
        &app.state::<AppState>(),
        "INFO",
        "external.start",
        &format!("task={id} engine={kind:?} url={}", redact_url(&task.source)),
    );
    log_network_route(
        &app.state::<AppState>(),
        &id.to_string(),
        &format!("{kind:?}"),
    )
    .await;
    update_task(&app, id, true, |item| {
        item.state = DownloadState::Downloading;
        item.progress_percent = Some(0.0);
        item.resume_supported = Some(match kind {
            DownloadKind::Torrent
            | DownloadKind::Magnet
            | DownloadKind::Ftp
            | DownloadKind::AcceleratedHttp => true,
            DownloadKind::MediaPage => true,
            DownloadKind::Hls => !task
                .format_selection
                .as_deref()
                .is_some_and(|value| value.starts_with("audio:")),
            _ => false,
        });
    });
    let directory = task.destination.parent().unwrap_or_else(|| Path::new("."));
    let file_name = task
        .destination
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("download");
    let media_work_directory = (kind == DownloadKind::MediaPage).then(|| {
        app.state::<AppState>()
            .queue_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("media-work")
            .join(id.to_string())
    });
    if let Some(work_directory) = media_work_directory.as_ref() {
        if let Err(error) = fs::create_dir_all(work_directory) {
            update_task(&app, id, true, |item| {
                item.state = DownloadState::Failed {
                    message: error.to_string(),
                }
            });
            return;
        }
    }
    let tools = app
        .state::<AppState>()
        .settings
        .lock()
        .map(|settings| {
            (
                configured_tool(&settings.ffmpeg_path, "ffmpeg"),
                configured_tool(&settings.yt_dlp_path, "yt-dlp"),
                configured_tool(
                    &settings.n_m3u8dl_re_path,
                    if cfg!(windows) {
                        "N_m3u8DL-RE.exe"
                    } else {
                        "N_m3u8DL-RE"
                    },
                ),
                configured_tool(
                    &settings.aria2_path,
                    if cfg!(windows) {
                        "aria2c.exe"
                    } else {
                        "aria2c"
                    },
                ),
                settings.connections_per_download.clamp(1, 32),
                settings
                    .proxy_enabled
                    .then(|| settings.proxy_url.clone())
                    .flatten(),
                settings.proxy_username.clone(),
                settings.proxy_password.clone(),
                if settings.dns_enabled {
                    settings.dns_servers.clone()
                } else {
                    Vec::new()
                },
            )
        })
        .unwrap_or_else(|_| {
            (
                "ffmpeg".into(),
                "yt-dlp".into(),
                "N_m3u8DL-RE".into(),
                "aria2c".into(),
                8,
                None,
                None,
                None,
                Vec::new(),
            )
        });
    let task_connections = task.connections_override.unwrap_or(tools.4).clamp(1, 32);
    diagnostic_log(
        &app.state::<AppState>(),
        "INFO",
        "external.configuration",
        &format!(
            "task={id} engine={kind:?} threads={} override={} proxy={} dns_servers={} selected_files={}",
            task_connections,
            task.connections_override.is_some(),
            tools.5.is_some(),
            tools.8.len(),
            task.torrent_selection.len(),
        ),
    );
    let identity = app
        .state::<AppState>()
        .request_identities
        .lock()
        .ok()
        .and_then(|identities| identities.get(&task.id).cloned());
    let configured_user_agent = app
        .state::<AppState>()
        .settings
        .lock()
        .ok()
        .and_then(|settings| settings.user_agent.clone());
    let global_bandwidth_limit = app
        .state::<AppState>()
        .settings
        .lock()
        .map(|settings| settings.global_bandwidth_limit)
        .unwrap_or_default();
    let bandwidth_limit = match (
        global_bandwidth_limit,
        task.bandwidth_limit.unwrap_or_default(),
    ) {
        (0, task_limit) => task_limit,
        (global_limit, 0) => global_limit,
        (global_limit, task_limit) => global_limit.min(task_limit),
    };
    let website_credential = app
        .state::<AppState>()
        .settings
        .lock()
        .ok()
        .and_then(|settings| {
            website_credential_for_download(&settings, &task.source, task.referer.as_deref())
                .cloned()
        });
    let user_agent = configured_user_agent.as_deref()
        .or_else(|| identity.as_ref().and_then(|value| value.user_agent.as_deref()))
        .unwrap_or("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/152.0.0.0 Safari/537.36");
    let proxy_url = tools
        .5
        .as_deref()
        .map(|url| external_proxy_url(url, tools.6.as_deref(), tools.7.as_deref()));
    let mut command = match kind {
        DownloadKind::MediaPage => {
            let mut command = tokio::process::Command::new(&tools.1);
            if let Some(proxy_url) = proxy_url.as_deref() {
                command.arg("--proxy").arg(proxy_url);
            }
            let selection = task
                .format_selection
                .as_deref()
                .unwrap_or("bestvideo+bestaudio/best");
            command.args(["--no-playlist", "--newline", "--verbose"]);
            if bandwidth_limit > 0 {
                command.arg("--limit-rate").arg(bandwidth_limit.to_string());
            }
            command
                .arg("--concurrent-fragments")
                .arg(task_connections.to_string());
            let quickjs_name = if cfg!(windows) { "qjs.exe" } else { "qjs" };
            let configured_quickjs = app
                .state::<AppState>()
                .settings
                .lock()
                .ok()
                .map(|settings| configured_tool(&settings.qjs_path, quickjs_name));
            let quickjs = configured_quickjs
                .filter(|path| path.is_file())
                .or_else(|| {
                    tools
                        .1
                        .parent()
                        .map(|directory| directory.join(quickjs_name))
                        .filter(|path| path.is_file())
                });
            if let Some(quickjs) = quickjs {
                command
                    .arg("--js-runtimes")
                    .arg(format!("quickjs:{}", quickjs.display()));
            } else {
                command.args(["--js-runtimes", "quickjs"]);
            }
            let browser_session_site = task.source.contains("instagram.com/")
                || task.source.contains("tiktok.com/")
                || task.source.contains("facebook.com/");
            if let Some(cookie) = identity
                .as_ref()
                .and_then(|value| value.cookie_header.as_deref())
                .filter(|value| !value.is_empty())
            {
                let cookie_jar = media_work_directory
                    .as_deref()
                    .map(|directory| directory.join("browser-cookies.txt"));
                if let Some(path) = cookie_jar
                    .as_ref()
                    .filter(|path| write_social_cookie_jar(path, &task.source, cookie).is_ok())
                {
                    command.arg("--cookies").arg(path);
                } else {
                    command.arg("--add-headers").arg(format!("Cookie:{cookie}"));
                }
            } else if task.source.contains("youtube.com/") || task.source.contains("youtu.be/") {
                command.args(["--cookies-from-browser", "chrome"]);
            }
            if task.source.contains("youtube.com/")
                || task.source.contains("youtu.be/")
                || browser_session_site
            {
                command.args([
                    "--retries",
                    "10",
                    "--fragment-retries",
                    "10",
                    "--retry-sleep",
                    "fragment:exp=1:8",
                ]);
            }
            command.args(["--user-agent", user_agent]);
            if let Some(credential) = website_credential.as_ref() {
                command
                    .arg("--username")
                    .arg(&credential.username)
                    .arg("--password")
                    .arg(&credential.password);
            }
            if let Some(referer) = task.referer.as_deref() {
                command.args(["--referer", referer]);
            }
            if let Some(audio_format) = selection.strip_prefix("audio:") {
                command.args(["-f", "bestaudio/best", "-x", "--audio-format", audio_format]);
            } else {
                command.args(["-f", selection, "--merge-output-format", "mp4"]);
            }
            command
                .arg("-P")
                .arg(media_work_directory.as_deref().unwrap_or(directory))
                .arg("-o")
                .arg("apocalipse-media.%(ext)s")
                .arg(&task.source);
            command
        }
        DownloadKind::Hls => {
            if let Some(audio_format) = task
                .format_selection
                .as_deref()
                .and_then(|value| value.strip_prefix("audio:"))
            {
                let mut command = tokio::process::Command::new(&tools.0);
                if let Some(proxy_url) = proxy_url.as_deref() {
                    command.arg("-http_proxy").arg(proxy_url);
                }
                command.arg("-y");
                if let Some(credential) = website_credential.as_ref() {
                    let basic =
                        BASE64.encode(format!("{}:{}", credential.username, credential.password));
                    command.args(["-headers", &format!("Authorization: Basic {basic}\r\n")]);
                }
                command.arg("-i").arg(&task.source).arg("-vn");
                match audio_format {
                    "mp3" => {
                        command.args(["-c:a", "libmp3lame", "-q:a", "2"]);
                    }
                    "wav" => {
                        command.args(["-c:a", "pcm_s16le"]);
                    }
                    "flac" => {
                        command.args(["-c:a", "flac"]);
                    }
                    "opus" => {
                        command.args(["-c:a", "libopus", "-b:a", "192k"]);
                    }
                    _ => {
                        command.args(["-c:a", "aac", "-b:a", "256k"]);
                    }
                }
                command.arg(&task.destination);
                command
            } else {
                let mut command = tokio::process::Command::new(&tools.2);
                if let Some(proxy_url) = proxy_url.as_deref() {
                    command.arg("--custom-proxy").arg(proxy_url);
                }
                let stem = task
                    .destination
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or("download");
                command
                    .arg(&task.source)
                    .arg("--save-dir")
                    .arg(directory)
                    .args([
                        "--save-name",
                        stem,
                        "--auto-select",
                        "--concurrent-download",
                        "--download-retry-count",
                        "10",
                        "--http-request-timeout",
                        "30",
                    ])
                    .arg("--thread-count")
                    .arg(task_connections.to_string())
                    .arg("--ffmpeg-binary-path")
                    .arg(&tools.0);
                if bandwidth_limit > 0 {
                    command.arg("--max-speed").arg(bandwidth_limit.to_string());
                }
                if let Some(referer) = task.referer.as_deref() {
                    command.arg("-H").arg(format!("Referer: {referer}"));
                    if let Some(origin) = http_origin(referer) {
                        command.arg("-H").arg(format!("Origin: {origin}"));
                    }
                }
                command.arg("-H").arg(format!("User-Agent: {user_agent}"));
                if let Some(credential) = website_credential.as_ref() {
                    let basic =
                        BASE64.encode(format!("{}:{}", credential.username, credential.password));
                    command
                        .arg("-H")
                        .arg(format!("Authorization: Basic {basic}"));
                }
                if let Some(cookie) = identity
                    .as_ref()
                    .and_then(|value| value.cookie_header.as_deref())
                {
                    command.arg("-H").arg(format!("Cookie: {cookie}"));
                }
                if task.source.contains("hdsex.org")
                    || task
                        .referer
                        .as_deref()
                        .is_some_and(|url| url.contains("hdsex.org"))
                {
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
        DownloadKind::Torrent
        | DownloadKind::Magnet
        | DownloadKind::Ftp
        | DownloadKind::AcceleratedHttp => {
            let mut command = tokio::process::Command::new(&tools.3);
            if !tools.8.is_empty() {
                command.arg(format!(
                    "--async-dns-server={}",
                    tools
                        .8
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                ));
            }
            if let Some(proxy_url) = tools.5.as_deref() {
                command.arg(format!("--all-proxy={proxy_url}"));
                if let Some(username) = tools.6.as_deref() {
                    command.arg(format!("--all-proxy-user={username}"));
                }
                if let Some(password) = tools.7.as_deref() {
                    command.arg(format!("--all-proxy-passwd={password}"));
                }
            }
            if kind == DownloadKind::Ftp {
                if let Some(credential) = website_credential.as_ref() {
                    command
                        .arg(format!("--ftp-user={}", credential.username))
                        .arg(format!("--ftp-passwd={}", credential.password));
                }
            }
            if kind == DownloadKind::AcceleratedHttp {
                command.args([
                    "--split=16",
                    "--max-connection-per-server=16",
                    "--min-split-size=1M",
                    "--optimize-concurrent-downloads=true",
                    "--stream-piece-selector=geom",
                ]);
                command.arg(format!("--out={file_name}"));
                command.arg(format!("--user-agent={user_agent}"));
                if let Some(referer) = task.referer.as_deref() {
                    command.arg(format!("--referer={referer}"));
                }
                if let Some(cookie) = identity
                    .as_ref()
                    .and_then(|value| value.cookie_header.as_deref())
                {
                    command.arg(format!("--header=Cookie: {cookie}"));
                }
            }
            command.arg(format!("--dir={}", directory.display())).args([
                "--summary-interval=1",
                "--console-log-level=notice",
                "--show-console-readout=true",
                "--download-result=hide",
                "--continue=true",
                "--enable-dht=true",
                "--enable-peer-exchange=true",
                "--bt-enable-lpd=true",
                "--bt-max-peers=100",
                "--bt-prioritize-piece=head=64M,tail=64M",
                "--file-allocation=trunc",
                "--seed-time=0",
            ]);
            if bandwidth_limit > 0 {
                command.arg(format!("--max-download-limit={bandwidth_limit}"));
            }
            if !task.torrent_selection.is_empty() {
                command.arg(format!(
                    "--select-file={}",
                    task.torrent_selection
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                ));
            }
            command.arg(&task.source);
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
    let mut result = match command.spawn() {
        Ok(mut child) => {
            let mut stdout = child.stdout.take();
            let mut stderr = child.stderr.take();
            let output_app = app.clone();
            let error_app = app.clone();
            let output = tokio::spawn(async move {
                match stdout.take() {
                    Some(stream) => read_process_tail(stream, Some((output_app, id, kind))).await,
                    None => Vec::new(),
                }
            });
            let errors = tokio::spawn(async move {
                match stderr.take() {
                    Some(stream) => read_process_tail(stream, Some((error_app, id, kind))).await,
                    None => Vec::new(),
                }
            });
            let status = tokio::select! {
                biased;
                _ = &mut cancellation => { let _ = child.kill().await; return; }
                status = child.wait() => status,
            };
            let mut text = String::from_utf8_lossy(&output.await.unwrap_or_default()).into_owned();
            text.push_str(&String::from_utf8_lossy(&errors.await.unwrap_or_default()));
            status
                .map_err(|error| error.to_string())
                .and_then(|status| {
                    if status.success() {
                        return Ok(());
                    }
                    if kind == DownloadKind::MediaPage {
                        if let Some(path) = write_yt_dlp_diagnostic(&app, id, &text, status.code())
                        {
                            diagnostic_log(
                                &app.state::<AppState>(),
                                "INFO",
                                "yt_dlp.report",
                                &format!("task={id} file={}", path.display()),
                            );
                        }
                    }
                    Err(external_error_detail(&text, status.code()))
                })
        }
        Err(error) => Err(format!("external_engine_unavailable: {error}")),
    };
    if result.is_ok() {
        if let Some(work_directory) = media_work_directory.as_deref() {
            result = finalize_media_page_download(&app, id, &task.destination, work_directory);
        }
    } else if let Some(work_directory) = media_work_directory.as_deref() {
        let _ = fs::remove_dir_all(work_directory);
    }
    match result {
        Ok(()) => {
            diagnostic_log(
                &app.state::<AppState>(),
                "INFO",
                "external.completed",
                &format!("task={id} engine={kind:?}"),
            );
            update_task(&app, id, true, |item| {
                item.progress_percent = Some(100.0);
                item.state = DownloadState::Completed;
                item.completed_at = Some(epoch_seconds());
            });
        }
        Err(message) => {
            diagnostic_log(
                &app.state::<AppState>(),
                "ERROR",
                "external.failed",
                &format!("task={id} engine={kind:?} error={message}"),
            );
            update_task(&app, id, true, |item| {
                item.progress_percent = None;
                item.state = DownloadState::Failed { message }
            });
        }
    }
    if let Ok(mut workers) = app.state::<AppState>().workers.lock() {
        workers.remove(&id);
    }
    start_next_queued(&app);
}

fn write_yt_dlp_diagnostic(
    app: &tauri::AppHandle,
    id: DownloadId,
    output: &str,
    exit_code: Option<i32>,
) -> Option<PathBuf> {
    let state = app.state::<AppState>();
    let directory = state.queue_path.parent()?.join("logs");
    fs::create_dir_all(&directory).ok()?;
    let path = directory.join(format!("yt-dlp-{id}.log"));
    let proxy_password = state
        .settings
        .lock()
        .ok()
        .and_then(|settings| settings.proxy_password.clone());
    let sanitized = output
        .lines()
        .map(|line| {
            if line.to_ascii_lowercase().contains("cookie:") {
                "[linha com cookie ocultada]".to_owned()
            } else {
                proxy_password
                    .as_deref()
                    .filter(|value| !value.is_empty())
                    .map_or_else(
                        || sanitize_log_detail(line),
                        |password| sanitize_log_detail(&line.replace(password, "<redacted>")),
                    )
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let contents = format!(
        "Apocalipse Download Manager - diagnóstico do yt-dlp\nTarefa: {id}\nCódigo de saída: {}\n\n{sanitized}\n",
        exit_code.map_or_else(|| "indisponível".to_owned(), |code| code.to_string()),
    );
    fs::write(&path, contents).ok()?;
    Some(path)
}

#[tauri::command]
fn read_general_log(state: State<'_, AppState>) -> Result<String, String> {
    diagnostic_log(&state, "INFO", "log.viewed", "viewed_inside_application");
    let _read_guard = state
        .log_write_lock
        .lock()
        .map_err(|error| error.to_string())?;
    match fs::read_to_string(&state.log_path) {
        Ok(contents) => {
            let start = contents.len().saturating_sub(512 * 1024);
            let start = contents
                .char_indices()
                .map(|(index, _)| index)
                .find(|index| *index >= start)
                .unwrap_or(0);
            Ok(contents[start..].to_owned())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error.to_string()),
    }
}

#[tauri::command]
fn clear_general_log(state: State<'_, AppState>) -> Result<(), String> {
    let rotated = state.log_path.with_extension("log.1");
    {
        let _write_guard = state
            .log_write_lock
            .lock()
            .map_err(|error| error.to_string())?;
        for path in [&state.log_path, &rotated] {
            match fs::remove_file(path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.to_string()),
            }
        }
    }
    diagnostic_log(&state, "INFO", "log.cleared", "cleared_by_user");
    Ok(())
}

fn write_diagnostic_zip(path: &Path, entries: Vec<(String, Vec<u8>)>) -> Result<(), String> {
    let mut output = Vec::new();
    let mut central = Vec::new();
    for (name, data) in &entries {
        let name = name.as_bytes();
        let offset = u32::try_from(output.len()).map_err(|_| "diagnostic_too_large")?;
        let size = u32::try_from(data.len()).map_err(|_| "diagnostic_too_large")?;
        let crc = crc32fast::hash(data);
        output.extend_from_slice(&0x04034b50_u32.to_le_bytes());
        output.extend_from_slice(&20_u16.to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes());
        output.extend_from_slice(&crc.to_le_bytes());
        output.extend_from_slice(&size.to_le_bytes());
        output.extend_from_slice(&size.to_le_bytes());
        output.extend_from_slice(&(name.len() as u16).to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes());
        output.extend_from_slice(name);
        output.extend_from_slice(data);
        central.push((name.to_vec(), crc, size, offset));
    }
    let central_offset = u32::try_from(output.len()).map_err(|_| "diagnostic_too_large")?;
    for (name, crc, size, offset) in &central {
        output.extend_from_slice(&0x02014b50_u32.to_le_bytes());
        output.extend_from_slice(&20_u16.to_le_bytes());
        output.extend_from_slice(&20_u16.to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes());
        output.extend_from_slice(&crc.to_le_bytes());
        output.extend_from_slice(&size.to_le_bytes());
        output.extend_from_slice(&size.to_le_bytes());
        output.extend_from_slice(&(name.len() as u16).to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes());
        output.extend_from_slice(&0_u32.to_le_bytes());
        output.extend_from_slice(&offset.to_le_bytes());
        output.extend_from_slice(name);
    }
    let central_size =
        u32::try_from(output.len()).map_err(|_| "diagnostic_too_large")? - central_offset;
    let count = u16::try_from(central.len()).map_err(|_| "diagnostic_too_large")?;
    output.extend_from_slice(&0x06054b50_u32.to_le_bytes());
    output.extend_from_slice(&0_u16.to_le_bytes());
    output.extend_from_slice(&0_u16.to_le_bytes());
    output.extend_from_slice(&count.to_le_bytes());
    output.extend_from_slice(&count.to_le_bytes());
    output.extend_from_slice(&central_size.to_le_bytes());
    output.extend_from_slice(&central_offset.to_le_bytes());
    output.extend_from_slice(&0_u16.to_le_bytes());
    fs::write(path, output).map_err(|error| error.to_string())
}

#[tauri::command]
fn export_diagnostic_bundle(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let now = chrono::Local::now();
    let file_name = format!(
        "Apocalipse-Diagnostico-{}.zip",
        now.format("%Y-%m-%d_%H-%M-%S")
    );
    let Some(path) = rfd::FileDialog::new()
        .set_file_name(&file_name)
        .add_filter("Apocalipse diagnostic", &["zip"])
        .save_file()
    else {
        return Ok(None);
    };

    diagnostic_log(
        &state,
        "INFO",
        "log.export_started",
        "diagnostic bundle requested",
    );
    let settings = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    let queue = state
        .queue
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    let settings_snapshot = serde_json::json!({
        "maxActiveDownloads": settings.max_active_downloads,
        "connectionsPerDownload": settings.connections_per_download,
        "adaptiveEfficiency": settings.adaptive_efficiency,
        "globalBandwidthLimit": settings.global_bandwidth_limit,
        "captureClipboard": settings.capture_clipboard,
        "proxyEnabled": settings.proxy_enabled,
        "proxyConfigured": settings.proxy_url.is_some(),
        "customDnsEnabled": settings.dns_enabled,
        "dnsServers": settings.dns_servers,
        "associations": settings.associations,
        "tools": {
            "ffmpeg": settings.ffmpeg_path.as_ref().is_some_and(|path| path.is_file()),
            "ytDlp": settings.yt_dlp_path.as_ref().is_some_and(|path| path.is_file()),
            "qjs": settings.qjs_path.as_ref().is_some_and(|path| path.is_file()),
            "nM3u8DlRe": settings.n_m3u8dl_re_path.as_ref().is_some_and(|path| path.is_file()),
            "aria2": settings.aria2_path.as_ref().is_some_and(|path| path.is_file()),
            "mediaPlayer": settings.media_player_path.as_ref().is_some_and(|path| path.is_file()),
        },
        "pairingTokenPresent": !settings.bridge_token.is_empty(),
    });
    let queue_snapshot = queue
        .iter()
        .map(|task| {
            serde_json::json!({
                "id": task.id,
                "source": redact_url(&task.source),
                "file": task.destination.file_name().map(|name| name.to_string_lossy()),
                "state": task.state,
                "received": task.received,
                "total": task.total,
                "downloadSpeed": task.download_speed,
                "uploadSpeed": task.upload_speed,
                "connectionsOverride": task.connections_override,
                "createdAt": task.created_at,
                "completedAt": task.completed_at,
            })
        })
        .collect::<Vec<_>>();
    let bridge_connected = state
        .bridge_last_seen
        .lock()
        .ok()
        .and_then(|seen| *seen)
        .is_some_and(|seen| seen.elapsed() < Duration::from_secs(90));
    let manifest = serde_json::json!({
        "format": "apocalipse-diagnostic-bundle",
        "formatVersion": 3,
        "createdAt": now.to_rfc3339(),
        "applicationVersion": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "bridgePort": BRIDGE_PORT,
        "bridgeConnected": bridge_connected,
        "privacy": "Secrets, credentials, cookies, authorization headers and URL parameter values are excluded or redacted."
    });

    let mut entries = vec![
        (
            "manifest.json".to_owned(),
            serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
        ),
        (
            "config/settings-safe.json".to_owned(),
            serde_json::to_vec_pretty(&settings_snapshot).map_err(|e| e.to_string())?,
        ),
        (
            "state/queue-safe.json".to_owned(),
            serde_json::to_vec_pretty(&queue_snapshot).map_err(|e| e.to_string())?,
        ),
    ];
    let mut all_events = String::new();
    for (source, name) in [
        (state.log_path.clone(), "logs/events.jsonl"),
        (
            state.log_path.with_extension("log.1"),
            "logs/events-previous.jsonl",
        ),
    ] {
        if let Ok(contents) = fs::read(source) {
            all_events.push_str(&String::from_utf8_lossy(&contents));
            entries.push((name.to_owned(), contents));
        }
    }
    let mut buckets = HashMap::<&str, String>::new();
    let mut levels = HashMap::<String, usize>::new();
    let mut event_counts = HashMap::<String, usize>::new();
    for line in all_events.lines().filter(|line| !line.trim().is_empty()) {
        let parsed = serde_json::from_str::<serde_json::Value>(line).ok();
        let event = parsed
            .as_ref()
            .and_then(|item| item.get("event"))
            .and_then(|item| item.as_str())
            .unwrap_or("legacy");
        let level = parsed
            .as_ref()
            .and_then(|item| item.get("level"))
            .and_then(|item| item.as_str())
            .unwrap_or("INFO");
        *levels.entry(level.to_owned()).or_default() += 1;
        *event_counts.entry(event.to_owned()).or_default() += 1;
        let bucket = if event.starts_with("extension.") {
            "extension-shortcuts-overlays"
        } else if event.starts_with("ui.") {
            "interface"
        } else if event.starts_with("bridge.") {
            "bridge"
        } else if event.starts_with("http.") {
            "http"
        } else if event.starts_with("blob.") || event.contains("recording") {
            "recordings"
        } else if event.starts_with("external.") || event.starts_with("yt_dlp.") {
            "media-torrent-hls"
        } else {
            "application"
        };
        let target = buckets.entry(bucket).or_default();
        target.push_str(line);
        target.push('\n');
    }
    for (bucket, contents) in buckets {
        entries.push((
            format!("logs/by-component/{bucket}.jsonl"),
            contents.into_bytes(),
        ));
    }
    let summary = serde_json::json!({
        "totalEvents": levels.values().sum::<usize>(),
        "levels": levels,
        "events": event_counts,
        "hint": "Start with ERROR/WARN events, then correlate matching task, trace and timestamp in logs/events.jsonl."
    });
    entries.push((
        "summary.json".to_owned(),
        serde_json::to_vec_pretty(&summary).map_err(|e| e.to_string())?,
    ));
    let guide = b"Apocalipse diagnostic bundle v3\nStart with RELATORIO_PARA_IA.txt and health/collectors.json. Legacy v2 files are preserved. A missing event is not proof of no activity. Review legacy logs before sharing.\n";
    entries.push(("README.txt".to_owned(), guide.to_vec()));
    entries.extend(state.diagnostics.export());
    write_diagnostic_zip(&path, entries)?;
    diagnostic_log(
        &state,
        "INFO",
        "log.export_completed",
        &format!("file={}", path.display()),
    );
    Ok(Some(path.to_string_lossy().into_owned()))
}

#[tauri::command]
fn diagnostics_status(state: State<'_, AppState>) -> serde_json::Value {
    let mut status = state.diagnostics.status();
    if let Some(object) = status.as_object_mut() {
        object.remove("salt");
    }
    status
}

#[tauri::command]
fn diagnostics_control(
    state: State<'_, AppState>,
    action: String,
) -> Result<serde_json::Value, String> {
    let mut status = state.diagnostics.control(&action, None)?;
    if let Some(object) = status.as_object_mut() {
        object.remove("salt");
    }
    Ok(status)
}

#[tauri::command]
fn record_diagnostics_ui(state: State<'_, AppState>, event: String, detail: serde_json::Value) {
    if event.len() <= 80
        && event
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.".contains(&b))
    {
        state
            .diagnostics
            .record(&format!("ui.{event}"), "INFO", None, None, detail);
    }
}

#[tauri::command]
fn copy_diagnostics_report(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    app.clipboard()
        .write_text(state.diagnostics.report())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn record_ui_diagnostic(
    state: State<'_, AppState>,
    level: String,
    event: String,
    detail: String,
) -> Result<(), String> {
    diagnostic_log(&state, &level, &format!("ui.{event}"), &detail);
    Ok(())
}

#[tauri::command]
fn set_application_language(state: State<'_, AppState>, language: String) -> Result<(), String> {
    if !matches!(language.as_str(), "en" | "pt-BR" | "zh-CN") {
        return Err("unsupported_language".to_owned());
    }
    let (show, quit) = tray_labels(&language);
    state
        .tray_show
        .set_text(show)
        .map_err(|error| error.to_string())?;
    state
        .tray_quit
        .set_text(quit)
        .map_err(|error| error.to_string())?;
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.language = language.clone();
    save_settings(&state, &settings)?;
    drop(settings);
    diagnostic_log(
        &state,
        "INFO",
        "application.language_changed",
        &format!("language={language}"),
    );
    Ok(())
}

#[tauri::command]
fn set_application_theme(state: State<'_, AppState>, theme: String) -> Result<(), String> {
    const THEMES: &[&str] = &[
        "void",
        "inferno",
        "toxic",
        "synthwave",
        "royal",
        "crimson",
        "arctic",
        "obsidian",
        "monochrome",
        "midnight",
        "forest",
        "graphite",
        "deepsea",
        "eclipse",
        "hazard",
        "cyberstorm",
        "ultraviolet",
        "emeraldgold",
        "scarletice",
        "coppernavy",
        "solarizednight",
        "pearlblue",
        "whiteaurora",
        "goldenivory",
        "crystalrose",
        "polarmint",
    ];
    if !THEMES.contains(&theme.as_str()) {
        return Err("unsupported_theme".to_owned());
    }
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.theme = theme.clone();
    save_settings(&state, &settings)?;
    drop(settings);
    diagnostic_log(
        &state,
        "INFO",
        "application.theme_changed",
        &format!("theme={theme}"),
    );
    Ok(())
}

#[tauri::command]
fn get_log_editor(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .log_editor_path
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default())
}

#[tauri::command]
fn set_log_editor(state: State<'_, AppState>, path: String) -> Result<String, String> {
    let path = optional_path(path);
    if path.as_ref().is_some_and(|value| !value.is_file()) {
        return Err("log_editor_not_found".to_owned());
    }
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.log_editor_path = path;
    save_settings(&state, &settings)?;
    Ok(settings
        .log_editor_path
        .as_ref()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default())
}

#[tauri::command]
fn open_log_external(state: State<'_, AppState>) -> Result<(), String> {
    diagnostic_log(
        &state,
        "INFO",
        "log.external",
        "opened_with_configured_editor",
    );
    let editor = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .log_editor_path
        .clone()
        .ok_or_else(|| "log_editor_not_configured".to_owned())?;
    if !editor.is_file() {
        return Err("log_editor_not_found".to_owned());
    }
    Command::new(editor)
        .arg(&state.log_path)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

async fn read_process_tail(
    mut stream: impl tokio::io::AsyncRead + Unpin,
    progress: Option<(tauri::AppHandle, DownloadId, DownloadKind)>,
) -> Vec<u8> {
    let mut tail = Vec::new();
    let mut chunk = [0_u8; 4096];
    loop {
        match stream.read(&mut chunk).await {
            Ok(0) | Err(_) => break,
            Ok(count) => {
                let text = String::from_utf8_lossy(&chunk[..count]);
                if let Some((app, id, kind)) = progress.as_ref() {
                    if matches!(
                        *kind,
                        DownloadKind::Torrent
                            | DownloadKind::Magnet
                            | DownloadKind::Ftp
                            | DownloadKind::AcceleratedHttp
                    ) {
                        if let Some((
                            received,
                            total,
                            percent,
                            download_speed,
                            upload_speed,
                            seeders,
                            leechers,
                            eta,
                        )) = parse_aria2_progress(&text)
                        {
                            update_task(app, *id, false, |task| {
                                task.received = received;
                                task.total = Some(total);
                                task.progress_percent = Some(percent);
                                task.download_speed = Some(download_speed);
                                task.upload_speed = Some(upload_speed);
                                task.torrent_seeders = Some(seeders);
                                task.torrent_leechers = Some(leechers);
                                task.torrent_eta = eta;
                            });
                        }
                    } else if let Some(percent) = parse_external_progress(&text) {
                        update_task(app, *id, false, |task| {
                            let percent = if *kind == DownloadKind::MediaPage {
                                percent.min(90.0)
                            } else {
                                percent
                            };
                            task.progress_percent =
                                Some(task.progress_percent.unwrap_or(0.0).max(percent));
                        });
                    }
                }
                tail.extend_from_slice(&chunk[..count]);
                if tail.len() > 65_536 {
                    tail.drain(..tail.len() - 65_536);
                }
            }
        }
    }
    tail
}

fn parse_aria2_size(value: &str) -> Option<u64> {
    let value =
        value.trim_start_matches(|character: char| !character.is_ascii_digit() && character != '.');
    let split = value
        .find(|character: char| !character.is_ascii_digit() && character != '.')
        .unwrap_or(value.len());
    let number = value[..split].parse::<f64>().ok()?;
    let unit = value[split..].to_ascii_lowercase();
    let multiplier = match unit.as_str() {
        "" | "b" => 1.0,
        "k" | "kb" | "kib" => 1024.0,
        "m" | "mb" | "mib" => 1024.0 * 1024.0,
        "g" | "gb" | "gib" => 1024.0 * 1024.0 * 1024.0,
        "t" | "tb" | "tib" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    Some((number * multiplier) as u64)
}

#[allow(clippy::type_complexity)]
fn parse_aria2_progress(text: &str) -> Option<(u64, u64, f64, u64, u64, u64, u64, Option<String>)> {
    text.lines().rev().find_map(|line| {
        let ratio = line.split_whitespace().find_map(|token| {
            let slash = token.find('/')?;
            let open = token[slash + 1..]
                .find('(')
                .map(|index| slash + 1 + index)?;
            let close = token[open + 1..].find('%').map(|index| open + 1 + index)?;
            let received = parse_aria2_size(&token[..slash])?;
            let total = parse_aria2_size(&token[slash + 1..open])?;
            let percent = token[open + 1..close].parse::<f64>().ok()?;
            if total > 0 && received <= total && (0.0..=100.0).contains(&percent) {
                Some((received, total, percent))
            } else {
                None
            }
        })?;
        let field = |prefix: &str| {
            line.split_whitespace()
                .find_map(|token| token.strip_prefix(prefix).and_then(parse_aria2_size))
                .unwrap_or(0)
        };
        let count = |prefix: &str| {
            line.split_whitespace()
                .find_map(|token| {
                    token
                        .strip_prefix(prefix)?
                        .trim_end_matches(']')
                        .parse::<u64>()
                        .ok()
                })
                .unwrap_or(0)
        };
        let connections = count("CN:");
        let seeders = count("SD:");
        let eta = line.split_whitespace().find_map(|token| {
            token
                .strip_prefix("ETA:")
                .map(|value| value.trim_end_matches(']').to_owned())
        });
        Some((
            ratio.0,
            ratio.1,
            ratio.2,
            field("DL:"),
            field("UL:"),
            seeders,
            connections.saturating_sub(seeders),
            eta,
        ))
    })
}

fn parse_external_progress(text: &str) -> Option<f64> {
    text.match_indices('%')
        .filter_map(|(end, _)| {
            let prefix = &text[..end];
            let start = prefix
                .char_indices()
                .rev()
                .take_while(|(_, character)| character.is_ascii_digit() || *character == '.')
                .last()
                .map(|(index, _)| index)?;
            prefix[start..].parse::<f64>().ok()
        })
        .rfind(|value| (0.0..=100.0).contains(value))
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
    let magnet_name = (classify_url(source) == Some(DownloadKind::Magnet))
        .then(|| {
            url::Url::parse(source)
                .ok()?
                .query_pairs()
                .find(|(key, _)| key == "dn")
                .map(|(_, value)| value.into_owned())
        })
        .flatten();
    magnet_name
        .as_deref()
        .unwrap_or(source)
        .split(['/', '\\'])
        .next_back()
        .and_then(|part| part.split(['?', '#']).next())
        .filter(|part| !part.is_empty())
        .unwrap_or("download")
        .chars()
        .map(|character| {
            if "<>:\"/\\|?*".contains(character) {
                '_'
            } else {
                character
            }
        })
        .collect()
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
    if name.is_empty()
        || name == "."
        || name == ".."
        || name
            .chars()
            .any(|character| "<>:\"/\\|?*".contains(character))
    {
        return Err("invalid_file_name".to_owned());
    }
    // Keep enough headroom for yt-dlp's temporary format suffixes and for the
    // Windows legacy MAX_PATH limit. Preserve the extension while shortening
    // unusually long titles from social networks.
    const MAX_FILE_NAME_UTF16: usize = 120;
    if name.encode_utf16().count() <= MAX_FILE_NAME_UTF16 {
        return Ok(name.to_owned());
    }
    let path = Path::new(name);
    let extension = path.extension().and_then(|value| value.to_str());
    let extension_chars = extension.map_or(0, |value| value.encode_utf16().count() + 1);
    let stem_limit = MAX_FILE_NAME_UTF16.saturating_sub(extension_chars).max(1);
    let mut used = 0;
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("download")
        .chars()
        .take_while(|character| {
            let width = character.len_utf16();
            if used + width > stem_limit {
                false
            } else {
                used += width;
                true
            }
        })
        .collect::<String>()
        .trim_end_matches([' ', '.'])
        .to_owned();
    Ok(extension
        .map(|extension| format!("{stem}.{extension}"))
        .unwrap_or(stem))
}

fn append_source_extension(file_name: String, source: &str, kind: DownloadKind) -> String {
    let has_extension = Path::new(&file_name)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| {
            (1..=10).contains(&value.len())
                && value
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric())
        });
    if has_extension {
        return file_name;
    }
    if kind == DownloadKind::MediaPage {
        return format!("{file_name}.mp4");
    }
    if kind != DownloadKind::Http {
        return file_name;
    }
    let source_name = suggested_name(source);
    let extension = Path::new(&source_name)
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| {
            (1..=10).contains(&value.len())
                && value
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric())
        });
    let query_extension = url::Url::parse(source).ok().and_then(|url| {
        url.query_pairs().find_map(|(key, value)| {
            matches!(
                key.to_ascii_lowercase().as_str(),
                "mime_type" | "mime" | "content_type"
            )
            .then(|| media_extension(&value))
            .flatten()
        })
    });
    extension
        .or(query_extension)
        .map(|extension| format!("{file_name}.{extension}"))
        .unwrap_or(file_name)
}

fn media_extension(mime: &str) -> Option<&'static str> {
    match mime
        .split(';')
        .next()?
        .trim()
        .to_ascii_lowercase()
        .replace('_', "/")
        .as_str()
    {
        "video/mp4" => Some("mp4"),
        "video/webm" | "audio/webm" => Some("webm"),
        "video/x-matroska" => Some("mkv"),
        "video/quicktime" => Some("mov"),
        "audio/mp4" | "audio/x-m4a" => Some("m4a"),
        "audio/mpeg" => Some("mp3"),
        "audio/aac" => Some("aac"),
        "audio/ogg" => Some("ogg"),
        "audio/opus" => Some("opus"),
        _ => None,
    }
}

// All queue producers must allocate and insert under the SAME queue lock.
// A queued/paused/failed task owns its path even before an engine creates .part.
fn destination_key(path: &Path) -> String {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    let value = normalized.to_string_lossy().into_owned();
    if cfg!(any(windows, target_os = "macos")) {
        value.to_lowercase()
    } else {
        value
    }
}

fn destination_exists(path: &Path) -> bool {
    // Do not reuse dangling symlinks or paths we cannot inspect.
    !matches!(fs::symlink_metadata(path), Err(error) if error.kind() == std::io::ErrorKind::NotFound)
}

fn unique_destination_with_queue(
    directory: &Path,
    file_name: &str,
    queue: &[DownloadTask],
) -> Result<PathBuf, String> {
    let reserved = queue
        .iter()
        .flat_map(|task| {
            [
                destination_key(&task.destination),
                destination_key(&partial_path(&task.destination)),
            ]
        })
        .collect::<HashSet<_>>();
    let path = Path::new(file_name);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("download");
    let extension = path.extension().and_then(|value| value.to_str());
    for index in 0..10_000 {
        let candidate_name = if index == 0 {
            file_name.to_owned()
        } else {
            match extension {
                Some(extension) => format!("{stem} ({index}).{extension}"),
                None => format!("{stem} ({index})"),
            }
        };
        let candidate = directory.join(candidate_name);
        let partial = partial_path(&candidate);
        if !reserved.contains(&destination_key(&candidate))
            && !reserved.contains(&destination_key(&partial))
            && !destination_exists(&candidate)
            && !destination_exists(&partial)
            && !destination_exists(&apocalipse_core::chunk_directory(&candidate))
        {
            return Ok(candidate);
        }
    }
    // Never fall back to an unchecked name or silently discard its extension.
    Err("download_destination_names_exhausted".to_owned())
}

fn unique_destination(directory: &Path, file_name: &str) -> Result<PathBuf, String> {
    unique_destination_with_queue(directory, file_name, &[])
}

fn reserve_queued_task(
    queue: &mut Vec<DownloadTask>,
    task: &mut DownloadTask,
    reject_active_duplicate: bool,
) -> Result<(), String> {
    if reject_active_duplicate {
        if let Some(existing) = queue.iter().find(|existing| {
            existing.source == task.source
                && !matches!(
                    existing.state,
                    DownloadState::Completed | DownloadState::Failed { .. }
                )
        }) {
            return Err(format!("duplicate_active_download:{}", existing.id));
        }
    }
    let directory = task.destination.parent().unwrap_or_else(|| Path::new("."));
    let file_name = task
        .destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "invalid_file_name".to_owned())?;
    task.destination = unique_destination_with_queue(directory, file_name, queue)?;
    queue.push(task.clone());
    Ok(())
}

#[tauri::command]
fn suggest_download_name(url: String) -> String {
    suggested_download_name(&url)
}

#[tauri::command]
fn list_downloads(state: State<'_, AppState>) -> Result<Vec<DownloadTask>, String> {
    state
        .queue
        .lock()
        .map(|queue| queue.clone())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn default_download_directory(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    configured_download_directory(&app, &state).map(|path| path.to_string_lossy().into_owned())
}

#[tauri::command]
fn set_default_download_directory(
    state: State<'_, AppState>,
    path: String,
) -> Result<String, String> {
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
fn list_download_directories(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<DestinationChoice>, String> {
    let default = configured_download_directory(&app, &state)?;
    let recent = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .recent_download_directories
        .clone();
    let mut paths = vec![default.clone()];
    for path in recent {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    Ok(paths
        .into_iter()
        .map(|path| DestinationChoice {
            is_default: path == default,
            available: path.is_dir(),
            path: path.to_string_lossy().into_owned(),
        })
        .collect())
}

#[tauri::command]
fn remove_download_directory(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let target = PathBuf::from(path);
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings
        .recent_download_directories
        .retain(|item| item != &target);
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
    dialog
        .pick_folder()
        .map(|path| path.to_string_lossy().into_owned())
}

#[tauri::command]
fn pick_executable(initial_path: Option<String>) -> Option<String> {
    let mut dialog = rfd::FileDialog::new();
    if let Some(path) = initial_path.filter(|value| !value.trim().is_empty()) {
        if let Some(parent) = Path::new(&path).parent() {
            dialog = dialog.set_directory(parent);
        }
    }
    dialog
        .pick_file()
        .map(|path| path.to_string_lossy().into_owned())
}

#[tauri::command]
fn pick_url_list() -> Result<Vec<String>, String> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("URL lists", &["txt", "csv"])
        .pick_file()
    else {
        return Ok(Vec::new());
    };
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let mut urls = Vec::new();
    for token in content
        .split(|character: char| character.is_whitespace() || character == ',' || character == ';')
    {
        let value = token.trim_matches(['\"', '\'', '[', ']']);
        if classify_url(value).is_some() && !urls.iter().any(|url| url == value) {
            urls.push(value.to_owned());
        }
        if urls.len() >= 500 {
            break;
        }
    }
    Ok(urls)
}

#[tauri::command]
fn get_tool_statuses(state: State<'_, AppState>) -> Result<Vec<ToolStatus>, String> {
    let settings = state.settings.lock().map_err(|error| error.to_string())?;
    let definitions = [
        (
            "ffmpeg",
            configured_tool(
                &settings.ffmpeg_path,
                if cfg!(windows) {
                    "ffmpeg.exe"
                } else {
                    "ffmpeg"
                },
            ),
            ["-version"].as_slice(),
        ),
        (
            "yt-dlp",
            configured_tool(
                &settings.yt_dlp_path,
                if cfg!(windows) {
                    "yt-dlp.exe"
                } else {
                    "yt-dlp"
                },
            ),
            ["--version"].as_slice(),
        ),
        (
            "qjs",
            configured_tool(
                &settings.qjs_path,
                if cfg!(windows) { "qjs.exe" } else { "qjs" },
            ),
            ["--version"].as_slice(),
        ),
        (
            "n-m3u8dl-re",
            configured_tool(
                &settings.n_m3u8dl_re_path,
                if cfg!(windows) {
                    "N_m3u8DL-RE.exe"
                } else {
                    "N_m3u8DL-RE"
                },
            ),
            ["--version"].as_slice(),
        ),
        (
            "aria2",
            configured_tool(
                &settings.aria2_path,
                if cfg!(windows) {
                    "aria2c.exe"
                } else {
                    "aria2c"
                },
            ),
            ["--version"].as_slice(),
        ),
    ];
    Ok(definitions
        .into_iter()
        .map(|(id, executable, args)| {
            let version = version_line(&executable, args);
            ToolStatus {
                id: id.to_owned(),
                path: executable.to_string_lossy().into_owned(),
                found: version.is_some(),
                version,
            }
        })
        .collect())
}

fn optional_path(value: String) -> Option<PathBuf> {
    let value = value.trim();
    (!value.is_empty()).then(|| PathBuf::from(value))
}

fn external_proxy_url(url: &str, username: Option<&str>, password: Option<&str>) -> String {
    let Ok(mut parsed) = url::Url::parse(url) else {
        return url.to_owned();
    };
    if let Some(username) = username.filter(|value| !value.is_empty()) {
        let _ = parsed.set_username(username);
        let _ = parsed.set_password(password);
    }
    parsed.to_string()
}

#[tauri::command]
fn set_tool_paths(
    state: State<'_, AppState>,
    ffmpeg: String,
    yt_dlp: String,
    qjs: String,
    n_m3u8dl_re: String,
    aria2: String,
) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.ffmpeg_path = optional_path(ffmpeg);
    settings.yt_dlp_path = optional_path(yt_dlp);
    settings.qjs_path = optional_path(qjs);
    settings.n_m3u8dl_re_path = optional_path(n_m3u8dl_re);
    settings.aria2_path = optional_path(aria2);
    save_settings(&state, &settings)
}

#[tauri::command]
fn get_media_player(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .media_player_path
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default())
}

#[tauri::command]
fn set_media_player(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.media_player_path = optional_path(path);
    save_settings(&state, &settings)
}

fn find_video_file(root: &Path, depth: usize) -> Option<PathBuf> {
    if depth > 6 {
        return None;
    }
    let mut best: Option<(u64, PathBuf)> = None;
    for entry in fs::read_dir(root).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(candidate) = find_video_file(&path, depth + 1) {
                if let Ok(metadata) = candidate.metadata() {
                    let size = metadata.len();
                    if best.as_ref().is_none_or(|(current, _)| size > *current) {
                        best = Some((size, candidate));
                    }
                }
            }
        } else if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|extension| {
                matches!(
                    extension.to_ascii_lowercase().as_str(),
                    "mp4" | "mkv" | "webm" | "avi" | "mov" | "m4v" | "ts"
                )
            })
        {
            if let Ok(metadata) = path.metadata() {
                let size = metadata.len();
                if best.as_ref().is_none_or(|(current, _)| size > *current) {
                    best = Some((size, path));
                }
            }
        }
    }
    best.map(|(_, path)| path)
}

fn find_named_file(root: &Path, expected_name: &str, depth: usize) -> Option<PathBuf> {
    if depth > 8 {
        return None;
    }
    for entry in fs::read_dir(root).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_named_file(&path, expected_name, depth + 1) {
                return Some(found);
            }
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case(expected_name))
        {
            return Some(path);
        }
    }
    None
}

fn active_torrent_video(directory: &Path) -> Option<PathBuf> {
    let mut best: Option<(u64, PathBuf)> = None;
    for entry in fs::read_dir(directory).ok()?.flatten() {
        let path = entry.path();
        let candidate = if path.is_dir() {
            find_video_file(&path, 0)
        } else if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|extension| {
                matches!(
                    extension.to_ascii_lowercase().as_str(),
                    "mp4" | "mkv" | "webm" | "avi" | "mov" | "m4v" | "ts"
                )
            })
        {
            Some(path)
        } else {
            None
        };
        let Some(candidate) = candidate else {
            continue;
        };
        let Ok(metadata) = candidate.metadata() else {
            continue;
        };
        let size = metadata.len();
        let control = PathBuf::from(format!("{}.aria2", candidate.display()));
        let priority = if control.exists() { u64::MAX / 2 } else { 0 };
        let score = priority.saturating_add(size);
        if best.as_ref().is_none_or(|(current, _)| score > *current) {
            best = Some((score, candidate));
        }
    }
    best.map(|(_, path)| path)
}

#[tauri::command]
fn preview_torrent(state: State<'_, AppState>, id: DownloadId) -> Result<(), String> {
    let (root, player) = {
        let queue = state.queue.lock().map_err(|error| error.to_string())?;
        let task = queue
            .iter()
            .find(|task| task.id == id)
            .ok_or_else(|| "download_not_found".to_owned())?;
        if !matches!(
            classify_url(&task.source),
            Some(DownloadKind::Torrent | DownloadKind::Magnet)
        ) {
            return Err("not_a_torrent".to_owned());
        }
        let settings = state.settings.lock().map_err(|error| error.to_string())?;
        (task.destination.clone(), settings.media_player_path.clone())
    };
    let video = if root.is_file() {
        root
    } else if root.is_dir() {
        find_video_file(&root, 0).ok_or_else(|| "torrent_video_not_available".to_owned())?
    } else {
        active_torrent_video(root.parent().unwrap_or(Path::new(".")))
            .ok_or_else(|| "torrent_video_not_available".to_owned())?
    };
    let player =
        player.unwrap_or_else(|| PathBuf::from(if cfg!(windows) { "vlc.exe" } else { "vlc" }));
    diagnostic_log(
        &state,
        "INFO",
        "torrent.preview_requested",
        &format!(
            "task={id} file={} player={}",
            video.display(),
            player.display()
        ),
    );
    let mut command = Command::new(&player);
    command.arg(&video);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    match command.spawn() {
        Ok(_) => {
            diagnostic_log(
                &state,
                "INFO",
                "torrent.preview_opened",
                &format!("task={id} file={}", video.display()),
            );
            Ok(())
        }
        Err(error) => {
            diagnostic_log(
                &state,
                "ERROR",
                "torrent.preview_failed",
                &format!("task={id} player={} error={error}", player.display()),
            );
            Err(error.to_string())
        }
    }
}

#[tauri::command]
async fn update_tool(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let executable = {
        let settings = state.settings.lock().map_err(|error| error.to_string())?;
        match id.as_str() {
            "yt-dlp" => configured_tool(
                &settings.yt_dlp_path,
                if cfg!(windows) {
                    "yt-dlp.exe"
                } else {
                    "yt-dlp"
                },
            ),
            "qjs" => configured_tool(
                &settings.qjs_path,
                if cfg!(windows) { "qjs.exe" } else { "qjs" },
            ),
            "aria2" => configured_tool(
                &settings.aria2_path,
                if cfg!(windows) {
                    "aria2c.exe"
                } else {
                    "aria2c"
                },
            ),
            "n-m3u8dl-re" => configured_tool(
                &settings.n_m3u8dl_re_path,
                if cfg!(windows) {
                    "N_m3u8DL-RE.exe"
                } else {
                    "N_m3u8DL-RE"
                },
            ),
            "ffmpeg" => configured_tool(
                &settings.ffmpeg_path,
                if cfg!(windows) {
                    "ffmpeg.exe"
                } else {
                    "ffmpeg"
                },
            ),
            _ => return Err("unknown_tool".to_owned()),
        }
    };
    if executable.is_dir() {
        return Err("tool_target_must_be_a_file".to_owned());
    }

    if id == "yt-dlp" {
        let before = version_line(&executable, &["--version"])
            .ok_or_else(|| "yt_dlp_not_found".to_owned())?;
        let mut command = Command::new(&executable);
        command.arg("-U");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let output = command.output().map_err(|error| error.to_string())?;
        if !output.status.success() {
            let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            return Err(if message.is_empty() {
                "yt_dlp_update_failed".to_owned()
            } else {
                message
            });
        }
        let after = version_line(&executable, &["--version"]).unwrap_or_else(|| before.clone());
        return Ok(if before == after {
            format!("yt-dlp already current ({after})")
        } else {
            format!("yt-dlp updated: {before} → {after}")
        });
    }

    #[cfg(not(target_os = "windows"))]
    return Err("automatic_binary_update_not_available_for_this_platform".to_owned());

    #[cfg(target_os = "windows")]
    {
        let (repository, executable_name, asset_markers, version_args): (
            &str,
            &str,
            &[&str],
            &[&str],
        ) = match id.as_str() {
            "qjs" => (
                "quickjs-ng/quickjs",
                "qjs.exe",
                &["qjs-windows-x86_64.exe"],
                &["--version"],
            ),
            "aria2" => (
                "aria2/aria2",
                "aria2c.exe",
                &["win", "64bit", ".zip"],
                &["--version"],
            ),
            "n-m3u8dl-re" => (
                "nilaoda/N_m3u8DL-RE",
                "N_m3u8DL-RE.exe",
                &["win-x64", ".zip"],
                &["--version"],
            ),
            "ffmpeg" => (
                "BtbN/FFmpeg-Builds",
                "ffmpeg.exe",
                &["win64", "gpl", ".zip"],
                &["-version"],
            ),
            _ => return Err("unknown_tool".to_owned()),
        };
        let before =
            version_line(&executable, version_args).unwrap_or_else(|| "unknown".to_owned());
        let api = format!("https://api.github.com/repos/{repository}/releases/latest");
        let client = reqwest::Client::builder()
            .user_agent("Apocalipse-Download-Manager")
            .build()
            .map_err(|error| error.to_string())?;
        let release: serde_json::Value = client
            .get(api)
            .send()
            .await
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map_err(|error| error.to_string())?
            .json()
            .await
            .map_err(|error| error.to_string())?;
        let tag = release
            .get("tag_name")
            .and_then(|value| value.as_str())
            .unwrap_or("latest");
        let assets = release
            .get("assets")
            .and_then(|value| value.as_array())
            .ok_or_else(|| "release_has_no_assets".to_owned())?;
        let asset = assets
            .iter()
            .find(|asset| {
                let name = asset
                    .get("name")
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                match id.as_str() {
                    "qjs" => name == "qjs-windows-x86_64.exe",
                    "ffmpeg" => name.ends_with("win64-gpl.zip") && !name.contains("shared"),
                    _ => asset_markers
                        .iter()
                        .all(|marker| name.contains(&marker.to_ascii_lowercase())),
                }
            })
            .ok_or_else(|| format!("compatible_release_asset_not_found:{repository}:{tag}"))?;
        let asset_name = asset
            .get("name")
            .and_then(|value| value.as_str())
            .unwrap_or("release.zip");
        let asset_url = asset
            .get("browser_download_url")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "release_asset_url_missing".to_owned())?;
        let bytes = client
            .get(asset_url)
            .send()
            .await
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map_err(|error| error.to_string())?
            .bytes()
            .await
            .map_err(|error| error.to_string())?;
        let sha256 = format!("{:x}", Sha256::digest(&bytes));
        let temporary =
            std::env::temp_dir().join(format!("apocalipse-tool-update-{}", uuid::Uuid::new_v4()));
        let (replacement, ffprobe_replacement) = if id == "qjs" {
            (bytes.to_vec(), Vec::new())
        } else {
            let archive_path = temporary.join("release.zip");
            let extracted = temporary.join("extracted");
            fs::create_dir_all(&extracted).map_err(|error| error.to_string())?;
            fs::write(&archive_path, &bytes).map_err(|error| error.to_string())?;
            let mut extractor = Command::new("tar.exe");
            extractor
                .arg("-xf")
                .arg(&archive_path)
                .arg("-C")
                .arg(&extracted);
            use std::os::windows::process::CommandExt;
            extractor.creation_flags(0x08000000);
            let extraction = extractor.output().map_err(|error| error.to_string())?;
            if !extraction.status.success() {
                let _ = fs::remove_dir_all(&temporary);
                return Err(format!(
                    "release_extraction_failed:{}",
                    String::from_utf8_lossy(&extraction.stderr).trim()
                ));
            }
            let replacement_path = find_named_file(&extracted, executable_name, 0)
                .ok_or_else(|| format!("replacement_executable_missing:{asset_name}"))?;
            let replacement = fs::read(replacement_path).map_err(|error| error.to_string())?;
            let ffprobe_replacement = if id == "ffmpeg" {
                fs::read(
                    find_named_file(&extracted, "ffprobe.exe", 0)
                        .ok_or_else(|| "ffprobe_missing_from_release".to_owned())?,
                )
                .map_err(|error| error.to_string())?
            } else {
                Vec::new()
            };
            (replacement, ffprobe_replacement)
        };
        let _ = fs::remove_dir_all(&temporary);
        if replacement.len() < 32_768 || (id == "ffmpeg" && ffprobe_replacement.len() < 32_768) {
            return Err(format!("replacement_executable_invalid:{asset_name}"));
        }
        let parent = executable
            .parent()
            .ok_or_else(|| "tool_target_has_no_directory".to_owned())?;
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let staged = parent.join(format!(".apocalipse-new-{executable_name}"));
        let backup = parent.join(format!(".{executable_name}.apocalipse-backup"));
        let ffprobe = parent.join("ffprobe.exe");
        let ffprobe_staged = parent.join(".apocalipse-new-ffprobe.exe");
        let ffprobe_backup = parent.join(".ffprobe.exe.apocalipse-backup");
        fs::write(&staged, &replacement).map_err(|error| error.to_string())?;
        if id == "ffmpeg" {
            fs::write(&ffprobe_staged, &ffprobe_replacement).map_err(|error| error.to_string())?;
        }
        let candidate_version = match version_line(&staged, version_args) {
            Some(version) => version,
            None => {
                let _ = fs::remove_file(&staged);
                let _ = fs::remove_file(&ffprobe_staged);
                return Err("downloaded_tool_validation_failed".to_owned());
            }
        };
        if id == "ffmpeg" && version_line(&ffprobe_staged, &["-version"]).is_none() {
            let _ = fs::remove_file(&staged);
            let _ = fs::remove_file(&ffprobe_staged);
            return Err("downloaded_ffprobe_validation_failed".to_owned());
        }
        if candidate_version == before {
            let _ = fs::remove_file(&staged);
            let _ = fs::remove_file(&ffprobe_staged);
            diagnostic_log(
                &state,
                "INFO",
                "tool.already_current",
                &format!("tool={id} version={before} asset={asset_name}"),
            );
            return Ok(format!("{id} already current ({before})"));
        }
        if backup.exists() {
            fs::remove_file(&backup).map_err(|error| error.to_string())?;
        }
        if ffprobe_backup.exists() {
            fs::remove_file(&ffprobe_backup).map_err(|error| error.to_string())?;
        }
        if executable.exists() {
            fs::rename(&executable, &backup).map_err(|error| error.to_string())?;
        }
        if id == "ffmpeg" && ffprobe.exists() {
            fs::rename(&ffprobe, &ffprobe_backup).map_err(|error| error.to_string())?;
        }
        if let Err(error) = fs::rename(&staged, &executable) {
            if backup.exists() {
                let _ = fs::rename(&backup, &executable);
            }
            if ffprobe_backup.exists() {
                let _ = fs::rename(&ffprobe_backup, &ffprobe);
            }
            return Err(error.to_string());
        }
        if id == "ffmpeg" {
            if let Err(error) = fs::rename(&ffprobe_staged, &ffprobe) {
                let _ = fs::remove_file(&executable);
                if backup.exists() {
                    let _ = fs::rename(&backup, &executable);
                }
                if ffprobe_backup.exists() {
                    let _ = fs::rename(&ffprobe_backup, &ffprobe);
                }
                return Err(error.to_string());
            }
        }
        let after = version_line(&executable, version_args);
        let ffprobe_valid = id != "ffmpeg" || version_line(&ffprobe, &["-version"]).is_some();
        if after.is_none() || !ffprobe_valid {
            let _ = fs::remove_file(&executable);
            if id == "ffmpeg" {
                let _ = fs::remove_file(&ffprobe);
            }
            if backup.exists() {
                let _ = fs::rename(&backup, &executable);
            }
            if ffprobe_backup.exists() {
                let _ = fs::rename(&ffprobe_backup, &ffprobe);
            }
            return Err("updated_tool_validation_failed_original_restored".to_owned());
        }
        if backup.exists() {
            fs::remove_file(&backup).map_err(|error| error.to_string())?;
        }
        if ffprobe_backup.exists() {
            fs::remove_file(&ffprobe_backup).map_err(|error| error.to_string())?;
        }
        let after = after.unwrap_or_else(|| tag.to_owned());
        diagnostic_log(&state, "INFO", "tool.updated", &format!("tool={id} repository={repository} tag={tag} asset={asset_name} sha256={sha256} before={before} after={after} target={}", executable.display()));
        Ok(format!("{id} updated: {before} → {after}"))
    }
}

#[tauri::command]
async fn inspect_torrent_metadata(
    state: State<'_, AppState>,
    source: String,
) -> Result<TorrentInspection, String> {
    match classify_url(&source) {
        Some(DownloadKind::Torrent) => inspect_torrent_file(Path::new(&source)),
        Some(DownloadKind::Magnet) => {
            let (aria2, root) = {
                let settings = state.settings.lock().map_err(|error| error.to_string())?;
                let root = state
                    .queue_path
                    .parent()
                    .unwrap_or(Path::new("."))
                    .join("torrent-metadata")
                    .join(uuid::Uuid::new_v4().to_string());
                (
                    configured_tool(
                        &settings.aria2_path,
                        if cfg!(windows) {
                            "aria2c.exe"
                        } else {
                            "aria2c"
                        },
                    ),
                    root,
                )
            };
            fs::create_dir_all(&root).map_err(|error| error.to_string())?;
            let mut command = tokio::process::Command::new(aria2);
            command
                .args([
                    "--bt-metadata-only=true",
                    "--bt-save-metadata=true",
                    "--seed-time=0",
                    "--summary-interval=0",
                ])
                .arg(format!("--dir={}", root.display()))
                .arg(&source);
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                command.as_std_mut().creation_flags(0x08000000);
            }
            let output = tokio::time::timeout(Duration::from_secs(120), command.output())
                .await
                .map_err(|_| "torrent_metadata_timeout".to_owned())?
                .map_err(|error| error.to_string())?;
            if !output.status.success() {
                let _ = fs::remove_dir_all(&root);
                return Err(external_error_detail(
                    &String::from_utf8_lossy(&output.stderr),
                    output.status.code(),
                ));
            }
            let torrent = fs::read_dir(&root)
                .map_err(|error| error.to_string())?
                .flatten()
                .map(|entry| entry.path())
                .find(|path| {
                    path.extension()
                        .is_some_and(|extension| extension.eq_ignore_ascii_case("torrent"))
                })
                .ok_or_else(|| "torrent_metadata_missing".to_owned())?;
            let result = inspect_torrent_file(&torrent);
            let _ = fs::remove_dir_all(&root);
            result
        }
        _ => Err("not_a_torrent".to_owned()),
    }
}

#[allow(clippy::too_many_arguments)]
fn enqueue_download_impl(
    app: tauri::AppHandle,
    state: &AppState,
    url: String,
    destination_directory: Option<String>,
    file_name: Option<String>,
    format_selection: Option<String>,
    torrent_selection: Option<Vec<usize>>,
    mirrors: Option<Vec<String>>,
    priority: Option<i8>,
    bandwidth_limit: Option<u64>,
    connections_override: Option<usize>,
    context: Option<DownloadContext>,
) -> Result<DownloadTask, String> {
    let diagnostic_trace = context.as_ref().and_then(|c| c.trace_id.clone());
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
        None => configured_download_directory(&app, state)?,
    };
    let proposed = file_name.unwrap_or_else(|| suggested_name(&url));
    let file_name = validate_file_name(&append_source_extension(proposed, &url, kind))?;
    remember_download_directory(state, &download_dir)?;
    let mut task = DownloadTask::new(&url, download_dir.join(&file_name));
    let mut request_identity = None;
    task.format_selection = format_selection.filter(|value| !value.trim().is_empty());
    task.torrent_selection = torrent_selection
        .unwrap_or_default()
        .into_iter()
        .filter(|index| *index > 0)
        .collect();
    task.mirrors = mirrors
        .unwrap_or_default()
        .into_iter()
        .filter(|mirror| mirror.starts_with("https://") || mirror.starts_with("http://"))
        .take(10)
        .collect();
    task.priority = priority.unwrap_or_default().clamp(-10, 10);
    task.bandwidth_limit = bandwidth_limit.filter(|limit| *limit > 0);
    task.connections_override = connections_override.map(|value| value.clamp(1, 32));
    if let Some(context) = context {
        task.referer = context
            .referer
            .filter(|url| url.starts_with("https://") || url.starts_with("http://"));
        task.known_duration = context
            .known_duration
            .filter(|duration| duration.is_finite() && *duration > 0.0);
        task.display_title = context.title.and_then(|value| {
            let value = value
                .chars()
                .filter(|character| !character.is_control())
                .take(512)
                .collect::<String>();
            (!value.trim().is_empty()).then(|| value.trim().to_owned())
        });
        task.thumbnail = context.thumbnail.filter(|value| {
            value.len() <= 8192
                && (value.starts_with("https://")
                    || value.starts_with("http://")
                    || value.starts_with("data:image/"))
        });
        task.companion_audio_url = context.audio_url.filter(|value| {
            value != &task.source && (value.starts_with("https://") || value.starts_with("http://"))
        });
        let cookie_header = context.cookie_header.filter(|value| {
            value.len() <= 16_384 && !value.contains('\r') && !value.contains('\n')
        });
        let user_agent = context
            .user_agent
            .filter(|value| value.len() <= 1024 && !value.contains('\r') && !value.contains('\n'));
        let request_method = context
            .request_method
            .as_deref()
            .unwrap_or("GET")
            .to_ascii_uppercase();
        let request_method = if request_method == "POST" {
            "POST"
        } else {
            "GET"
        }
        .to_owned();
        let request_body = context.request_body.filter(|value| value.len() <= 65_536);
        let request_content_type = context
            .request_content_type
            .filter(|value| value.len() <= 256 && !value.contains('\r') && !value.contains('\n'));
        if cookie_header.is_some() || user_agent.is_some() || request_method == "POST" {
            request_identity = Some(RequestIdentity {
                cookie_header,
                user_agent,
                request_method,
                request_body,
                request_content_type,
            });
        }
    }
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    let mut identities = state
        .request_identities
        .lock()
        .map_err(|error| error.to_string())?;
    reserve_queued_task(&mut queue, &mut task, true)?;
    state
        .diagnostics
        .bind_task(&task.id.to_string(), diagnostic_trace.as_deref());
    if let Err(error) = save_queue(state, &queue) {
        queue.pop();
        return Err(error);
    }
    if let Some(identity) = request_identity {
        identities.insert(task.id, identity);
    }
    drop(identities);
    drop(queue);
    diagnostic_log(
        state,
        "INFO",
        "task.enqueued",
        &format!(
            "task={} engine={kind:?} threads_override={} url={} file={}",
            task.id,
            task.connections_override
                .map_or_else(|| "global".to_owned(), |value| value.to_string()),
            redact_url(&task.source),
            task.destination.display()
        ),
    );
    start_download(&app, state, task.clone(), kind)?;
    Ok(task)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn enqueue_download(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    url: String,
    destination_directory: Option<String>,
    file_name: Option<String>,
    format_selection: Option<String>,
    torrent_selection: Option<Vec<usize>>,
    mirrors: Option<Vec<String>>,
    priority: Option<i8>,
    bandwidth_limit: Option<u64>,
    connections_override: Option<usize>,
    context: Option<DownloadContext>,
) -> Result<DownloadTask, String> {
    enqueue_download_impl(
        app,
        &state,
        url,
        destination_directory,
        file_name,
        format_selection,
        torrent_selection,
        mirrors,
        priority,
        bandwidth_limit,
        connections_override,
        context,
    )
}

fn queued_task_for_start(queue: &[DownloadTask], id: DownloadId) -> Option<DownloadTask> {
    queue
        .iter()
        .find(|item| item.id == id && item.state == DownloadState::Queued)
        .cloned()
}

fn start_download(
    app: &tauri::AppHandle,
    state: &AppState,
    task: DownloadTask,
    kind: DownloadKind,
) -> Result<(), String> {
    // A scheduler snapshot may outlive removal. Register only a task still queued.
    let queue = state.queue.lock().map_err(|error| error.to_string())?;
    let Some(task) = queued_task_for_start(&queue, task.id) else {
        return Ok(());
    };
    let mut workers = state.workers.lock().map_err(|error| error.to_string())?;
    if workers.contains_key(&task.id) {
        return Err("download_already_running".to_owned());
    }
    let limits = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    if workers.len() >= limits.max_active_downloads.clamp(1, 20) {
        return Ok(());
    }
    let (cancel, cancelled) = oneshot::channel();
    workers.insert(task.id, cancel);
    drop(workers);
    drop(queue);
    diagnostic_log(
        state,
        "INFO",
        "task.dispatched",
        &format!("task={} engine={kind:?}", task.id),
    );
    if kind == DownloadKind::Http && task.companion_audio_url.is_some() {
        tauri::async_runtime::spawn(run_adaptive_social_download(
            app.clone(),
            task.id,
            task,
            cancelled,
        ));
        return Ok(());
    }
    if kind == DownloadKind::Http
        && task.source.starts_with("https://")
        && task.source.contains(".freefilehub.com:")
    {
        diagnostic_log(
            state,
            "INFO",
            "http.accelerated",
            &format!("task={} engine=aria2", task.id),
        );
        tauri::async_runtime::spawn(run_external_download(
            app.clone(),
            task.id,
            task,
            DownloadKind::AcceleratedHttp,
            cancelled,
        ));
        return Ok(());
    }
    if kind == DownloadKind::Http {
        let identity = state
            .request_identities
            .lock()
            .ok()
            .and_then(|items| items.get(&task.id).cloned());
        let configured_connections =
            if limits.adaptive_efficiency && limits.max_active_downloads <= 3 && task.priority >= 0
            {
                limits.connections_per_download.max(16)
            } else {
                limits.connections_per_download
            };
        let connections = task
            .connections_override
            .unwrap_or_else(|| configured_connections.clamp(1, 32));
        let mut headers = Vec::new();
        if let Some(referer) = task.referer.as_ref() {
            headers.push(("Referer".to_owned(), referer.clone()));
        }
        if let Some(user_agent) = identity.as_ref().and_then(|item| item.user_agent.as_ref()) {
            headers.push(("User-Agent".to_owned(), user_agent.clone()));
        }
        if let Some(cookie) = identity
            .as_ref()
            .and_then(|item| item.cookie_header.as_ref())
        {
            headers.push(("Cookie".to_owned(), cookie.clone()));
        }
        if let Some(content_type) = identity
            .as_ref()
            .and_then(|item| item.request_content_type.as_ref())
        {
            headers.push(("Content-Type".to_owned(), content_type.clone()));
        }
        if let Some(credential) =
            website_credential_for_download(&limits, &task.source, task.referer.as_deref())
        {
            let basic = BASE64.encode(format!("{}:{}", credential.username, credential.password));
            headers.push(("Authorization".to_owned(), format!("Basic {basic}")));
        }
        let mirrors = task.mirrors.clone();
        let request = DownloadRequest {
            url: task.source,
            destination: task.destination,
            overwrite: false,
            connections,
            method: identity
                .as_ref()
                .map(|item| item.request_method.clone())
                .unwrap_or_else(|| "GET".to_owned()),
            body: identity.and_then(|item| item.request_body.map(String::into_bytes)),
            headers,
            limiters: {
                let mut limiters = vec![state.global_bandwidth_limiter.clone()];
                let task_limiter =
                    state
                        .download_bandwidth_limiters
                        .lock()
                        .ok()
                        .and_then(|mut items| {
                            let limit = task.bandwidth_limit.unwrap_or_default();
                            if limit == 0 {
                                items.remove(&task.id);
                                None
                            } else {
                                let limiter = items
                                    .entry(task.id)
                                    .or_insert_with(|| Arc::new(BandwidthLimiter::new(limit)))
                                    .clone();
                                limiter.set_limit(limit);
                                Some(limiter)
                            }
                        });
                if let Some(limiter) = task_limiter {
                    limiters.push(limiter);
                }
                limiters
            },
        };
        tauri::async_runtime::spawn(run_download(
            app.clone(),
            task.id,
            request,
            mirrors,
            cancelled,
        ));
    } else {
        tauri::async_runtime::spawn(run_external_download(
            app.clone(),
            task.id,
            task,
            kind,
            cancelled,
        ));
    }
    Ok(())
}

fn start_next_queued(app: &tauri::AppHandle) {
    let state = app.state::<AppState>();
    let maximum = state
        .settings
        .lock()
        .map(|settings| settings.max_active_downloads.clamp(1, 20))
        .unwrap_or(0);
    let available = state
        .workers
        .lock()
        .map(|workers| maximum.saturating_sub(workers.len()))
        .unwrap_or(0);
    if available == 0 {
        return;
    }
    let queued = state
        .queue
        .lock()
        .map(|queue| {
            let mut queued = queue
                .iter()
                .filter(|task| task.state == DownloadState::Queued)
                .cloned()
                .collect::<Vec<_>>();
            queued.sort_by_key(|task| {
                (
                    std::cmp::Reverse(task.priority),
                    task.total.unwrap_or(u64::MAX),
                    task.created_at,
                )
            });
            queued.truncate(available);
            queued
        })
        .unwrap_or_default();
    for task in queued {
        if let Some(kind) = classify_url(&task.source) {
            let _ = start_download(app, &state, task, kind);
        }
    }
}

#[tauri::command]
fn pause_download(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: DownloadId,
) -> Result<(), String> {
    let cancel = state
        .workers
        .lock()
        .map_err(|error| error.to_string())?
        .remove(&id)
        .ok_or_else(|| "download_not_running".to_owned())?;
    let _ = cancel.send(());
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    let task = queue
        .iter_mut()
        .find(|task| task.id == id)
        .ok_or_else(|| "download_not_found".to_owned())?;
    task.state = DownloadState::Paused;
    task.download_speed = Some(0);
    task.upload_speed = Some(0);
    let partial_bytes = task.received;
    save_queue(&state, &queue)?;
    drop(queue);
    diagnostic_log(
        &state,
        "INFO",
        "task.pause_confirmed",
        &format!("task={id} partial_bytes={partial_bytes} worker_signal=sent"),
    );
    start_next_queued(&app);
    Ok(())
}

#[tauri::command]
fn resume_download(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: DownloadId,
) -> Result<(), String> {
    let task = {
        let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
        let task = queue
            .iter_mut()
            .find(|task| task.id == id)
            .ok_or_else(|| "download_not_found".to_owned())?;
        match task.state {
            DownloadState::Paused | DownloadState::Failed { .. } => {
                task.state = DownloadState::Queued;
                let task = task.clone();
                save_queue(&state, &queue)?;
                task
            }
            _ => return Err("download_not_resumable".to_owned()),
        }
    };
    let kind = classify_url(&task.source).ok_or_else(|| "unsupported_url".to_owned())?;
    diagnostic_log(
        &state,
        "INFO",
        "task.resumed",
        &format!("task={id} engine={kind:?}"),
    );
    start_download(&app, &state, task, kind)
}

#[tauri::command]
fn redownload_downloads(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    ids: Vec<DownloadId>,
) -> Result<Vec<DownloadTask>, String> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let originals = state
        .queue
        .lock()
        .map_err(|error| error.to_string())?
        .iter()
        .filter(|task| ids.contains(&task.id))
        .cloned()
        .collect::<Vec<_>>();
    let saved_identities = state
        .request_identities
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    let mut repeated = Vec::with_capacity(originals.len());
    let mut repeated_identities = Vec::new();
    for original in originals {
        let directory = original
            .destination
            .parent()
            .unwrap_or_else(|| Path::new("."));
        let file_name = original
            .destination
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("download");
        let mut task = DownloadTask::new(&original.source, directory.join(file_name));
        task.format_selection = original.format_selection.clone();
        task.referer = original.referer.clone();
        task.known_duration = original.known_duration;
        task.display_title = original.display_title.clone();
        task.thumbnail = original.thumbnail.clone();
        if let Some(identity) = saved_identities.get(&original.id) {
            repeated_identities.push((task.id, identity.clone()));
        }
        repeated.push(task);
    }
    {
        let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
        let mut identities = state
            .request_identities
            .lock()
            .map_err(|error| error.to_string())?;
        let previous_len = queue.len();
        for task in &mut repeated {
            if let Err(error) = reserve_queued_task(&mut queue, task, false) {
                queue.truncate(previous_len);
                return Err(error);
            }
        }
        if let Err(error) = save_queue(&state, &queue) {
            queue.truncate(previous_len);
            return Err(error);
        }
        identities.extend(repeated_identities);
    }
    for task in &repeated {
        let kind = classify_url(&task.source).ok_or_else(|| "unsupported_url".to_owned())?;
        start_download(&app, &state, task.clone(), kind)?;
    }
    diagnostic_log(
        &state,
        "INFO",
        "task.redownload",
        &format!("count={} source_tasks={}", repeated.len(), ids.len()),
    );
    Ok(repeated)
}

#[tauri::command]
fn get_clipboard_monitor(state: State<'_, AppState>) -> Result<ClipboardStatus, String> {
    let enabled = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .capture_clipboard;
    Ok(ClipboardStatus { enabled })
}

#[tauri::command]
fn set_clipboard_monitor(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<ClipboardStatus, String> {
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
        adaptive_efficiency: settings.adaptive_efficiency,
        global_bandwidth_limit: settings.global_bandwidth_limit,
    })
}

#[tauri::command]
fn set_transfer_limits(
    state: State<'_, AppState>,
    max_active_downloads: usize,
    connections_per_download: usize,
    adaptive_efficiency: bool,
    global_bandwidth_limit: u64,
) -> Result<TransferLimits, String> {
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.max_active_downloads = max_active_downloads.clamp(1, 20);
    settings.connections_per_download = connections_per_download.clamp(1, 32);
    settings.adaptive_efficiency = adaptive_efficiency;
    settings.global_bandwidth_limit = global_bandwidth_limit.min(10 * 1024 * 1024 * 1024);
    state
        .global_bandwidth_limiter
        .set_limit(settings.global_bandwidth_limit);
    save_settings(&state, &settings)?;
    Ok(TransferLimits {
        max_active_downloads: settings.max_active_downloads,
        connections_per_download: settings.connections_per_download,
        adaptive_efficiency: settings.adaptive_efficiency,
        global_bandwidth_limit: settings.global_bandwidth_limit,
    })
}

#[tauri::command]
fn set_download_bandwidth_limit(
    state: State<'_, AppState>,
    id: DownloadId,
    bandwidth_limit: u64,
) -> Result<u64, String> {
    let limit = bandwidth_limit.min(10 * 1024 * 1024 * 1024);
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    let task = queue
        .iter_mut()
        .find(|task| task.id == id)
        .ok_or_else(|| "download_not_found".to_owned())?;
    task.bandwidth_limit = (limit > 0).then_some(limit);
    save_queue(&state, &queue)?;
    drop(queue);
    let mut limiters = state
        .download_bandwidth_limiters
        .lock()
        .map_err(|error| error.to_string())?;
    if limit == 0 {
        if let Some(limiter) = limiters.remove(&id) {
            limiter.set_limit(0);
        }
    } else {
        limiters
            .entry(id)
            .or_insert_with(|| Arc::new(BandwidthLimiter::new(limit)))
            .set_limit(limit);
    }
    Ok(limit)
}

#[tauri::command]
fn get_user_agent(state: State<'_, AppState>) -> Result<UserAgentSetting, String> {
    let value = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .user_agent
        .clone()
        .unwrap_or_default();
    Ok(UserAgentSetting { user_agent: value })
}

#[tauri::command]
fn set_user_agent(
    state: State<'_, AppState>,
    user_agent: String,
) -> Result<UserAgentSetting, String> {
    let value = user_agent.trim();
    if value.len() > 1024 || value.contains('\r') || value.contains('\n') {
        return Err("invalid_user_agent".to_owned());
    }
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.user_agent = (!value.is_empty()).then(|| value.to_owned());
    save_settings(&state, &settings)?;
    Ok(UserAgentSetting {
        user_agent: settings.user_agent.clone().unwrap_or_default(),
    })
}

#[tauri::command]
fn get_proxy_setting(state: State<'_, AppState>) -> Result<ProxySetting, String> {
    let settings = state.settings.lock().map_err(|error| error.to_string())?;
    Ok(ProxySetting {
        enabled: settings.proxy_enabled,
        url: settings.proxy_url.clone().unwrap_or_default(),
        username: settings.proxy_username.clone().unwrap_or_default(),
        has_password: settings
            .proxy_password
            .as_ref()
            .is_some_and(|value| !value.is_empty()),
    })
}

fn normalize_credential_host(value: &str) -> Result<String, String> {
    let candidate = value.trim().trim_end_matches('/');
    let parsed = if candidate.contains("://") {
        url::Url::parse(candidate)
    } else {
        url::Url::parse(&format!("https://{candidate}"))
    }
    .map_err(|_| "invalid_credential_host".to_owned())?;
    if !matches!(parsed.scheme(), "http" | "https" | "ftp")
        || parsed.port().is_some()
        || parsed.path() != "/"
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err("invalid_credential_host".to_owned());
    }
    parsed
        .host_str()
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| "invalid_credential_host".to_owned())
}

fn website_credential_for_url<'a>(
    settings: &'a UserSettings,
    source: &str,
) -> Option<&'a WebsiteCredential> {
    let host = url::Url::parse(source)
        .ok()?
        .host_str()?
        .to_ascii_lowercase();
    settings.website_credentials.iter().find(|credential| {
        host == credential.host || host.ends_with(&format!(".{}", credential.host))
    })
}

fn website_credential_for_download<'a>(
    settings: &'a UserSettings,
    source: &str,
    referer: Option<&str>,
) -> Option<&'a WebsiteCredential> {
    website_credential_for_url(settings, source).or_else(|| {
        let source_host = url::Url::parse(source)
            .ok()?
            .host_str()?
            .to_ascii_lowercase();
        let referer = referer?;
        let referer_host = url::Url::parse(referer)
            .ok()?
            .host_str()?
            .to_ascii_lowercase();
        let is_rsload_download = (source_host == "fixti.net"
            || source_host.ends_with(".fixti.net"))
            && (referer_host == "rsload.net" || referer_host.ends_with(".rsload.net"));
        is_rsload_download
            .then(|| website_credential_for_url(settings, referer))
            .flatten()
    })
}

#[tauri::command]
fn list_website_credentials(
    state: State<'_, AppState>,
) -> Result<Vec<WebsiteCredentialSummary>, String> {
    let settings = state.settings.lock().map_err(|error| error.to_string())?;
    Ok(settings
        .website_credentials
        .iter()
        .map(|credential| WebsiteCredentialSummary {
            host: credential.host.clone(),
            username: credential.username.clone(),
        })
        .collect())
}

#[tauri::command]
fn save_website_credential(
    state: State<'_, AppState>,
    host: String,
    username: String,
    password: String,
) -> Result<Vec<WebsiteCredentialSummary>, String> {
    let host = normalize_credential_host(&host)?;
    let username = username.trim();
    if username.is_empty()
        || password.is_empty()
        || username.len() > 512
        || password.len() > 2048
        || username
            .chars()
            .any(|character| matches!(character, '\r' | '\n'))
        || password
            .chars()
            .any(|character| matches!(character, '\r' | '\n'))
    {
        return Err("invalid_website_credential".to_owned());
    }
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    if let Some(existing) = settings
        .website_credentials
        .iter_mut()
        .find(|credential| credential.host == host)
    {
        existing.username = username.to_owned();
        existing.password = password;
    } else {
        settings.website_credentials.push(WebsiteCredential {
            host,
            username: username.to_owned(),
            password,
        });
    }
    settings
        .website_credentials
        .sort_by(|left, right| left.host.cmp(&right.host));
    save_settings(&state, &settings)?;
    Ok(settings
        .website_credentials
        .iter()
        .map(|credential| WebsiteCredentialSummary {
            host: credential.host.clone(),
            username: credential.username.clone(),
        })
        .collect())
}

#[tauri::command]
fn remove_website_credential(
    state: State<'_, AppState>,
    host: String,
) -> Result<Vec<WebsiteCredentialSummary>, String> {
    let host = normalize_credential_host(&host)?;
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings
        .website_credentials
        .retain(|credential| credential.host != host);
    save_settings(&state, &settings)?;
    Ok(settings
        .website_credentials
        .iter()
        .map(|credential| WebsiteCredentialSummary {
            host: credential.host.clone(),
            username: credential.username.clone(),
        })
        .collect())
}

#[tauri::command]
fn set_proxy_setting(
    state: State<'_, AppState>,
    enabled: bool,
    url: String,
    username: String,
    password: String,
    clear_password: bool,
) -> Result<ProxySetting, String> {
    let url = url.trim();
    let username = username.trim();
    if url.len() > 2048
        || username.len() > 512
        || password.len() > 2048
        || [url, username, password.as_str()].iter().any(|value| {
            value
                .chars()
                .any(|character| matches!(character, '\r' | '\n'))
        })
    {
        return Err("invalid_proxy_setting".to_owned());
    }
    if enabled {
        let parsed = url::Url::parse(url).map_err(|_| "invalid_proxy_url".to_owned())?;
        if !matches!(
            parsed.scheme(),
            "http" | "https" | "socks4" | "socks5" | "socks5h"
        ) || parsed.host().is_none()
        {
            return Err("invalid_proxy_url".to_owned());
        }
    }
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.proxy_enabled = enabled;
    settings.proxy_url = (!url.is_empty()).then(|| url.to_owned());
    settings.proxy_username = (!username.is_empty()).then(|| username.to_owned());
    if clear_password {
        settings.proxy_password = None;
    } else if !password.is_empty() {
        settings.proxy_password = Some(password);
    }
    save_settings(&state, &settings)?;
    diagnostic_log(
        &state,
        "INFO",
        "proxy.updated",
        &format!(
            "enabled={enabled} authenticated={}",
            settings.proxy_username.is_some()
        ),
    );
    Ok(ProxySetting {
        enabled: settings.proxy_enabled,
        url: settings.proxy_url.clone().unwrap_or_default(),
        username: settings.proxy_username.clone().unwrap_or_default(),
        has_password: settings
            .proxy_password
            .as_ref()
            .is_some_and(|value| !value.is_empty()),
    })
}

#[tauri::command]
fn get_dns_setting(state: State<'_, AppState>) -> Result<DnsSetting, String> {
    let settings = state.settings.lock().map_err(|error| error.to_string())?;
    Ok(DnsSetting {
        enabled: settings.dns_enabled,
        servers: settings
            .dns_servers
            .iter()
            .map(ToString::to_string)
            .collect(),
    })
}

fn parse_dns_servers(servers: &[String]) -> Result<Vec<std::net::IpAddr>, String> {
    if servers.len() > 8 {
        return Err("too_many_dns_servers".to_owned());
    }
    let mut parsed = Vec::with_capacity(servers.len());
    for server in servers {
        let address = server
            .trim()
            .parse::<std::net::IpAddr>()
            .map_err(|_| "invalid_dns_server".to_owned())?;
        if !parsed.contains(&address) {
            parsed.push(address);
        }
    }
    Ok(parsed)
}

#[tauri::command]
fn set_dns_setting(
    state: State<'_, AppState>,
    enabled: bool,
    servers: Vec<String>,
) -> Result<DnsSetting, String> {
    let parsed = parse_dns_servers(&servers)?;
    if enabled && parsed.is_empty() {
        return Err("dns_server_required".to_owned());
    }
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.dns_enabled = enabled;
    settings.dns_servers = parsed;
    save_settings(&state, &settings)?;
    diagnostic_log(
        &state,
        "INFO",
        "dns.updated",
        &format!("enabled={enabled} servers={}", settings.dns_servers.len()),
    );
    Ok(DnsSetting {
        enabled: settings.dns_enabled,
        servers: settings
            .dns_servers
            .iter()
            .map(ToString::to_string)
            .collect(),
    })
}

#[tauri::command]
fn read_clipboard_link(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    if !state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .capture_clipboard
    {
        return Ok(None);
    }
    // An empty or temporarily unavailable text clipboard is the normal idle
    // state of the optional monitor, not an application error.
    let Ok(value) = app.clipboard().read_text() else {
        return Ok(None);
    };
    let value = value.trim();
    Ok(classify_url(value).map(|_| value.to_owned()))
}

#[tauri::command]
fn get_bridge_pairing(state: State<'_, AppState>) -> Result<BridgePairing, String> {
    let token = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .bridge_token
        .clone();
    let connected = state
        .bridge_last_seen
        .lock()
        .map_err(|error| error.to_string())?
        .is_some_and(|seen| seen.elapsed() < Duration::from_secs(75));
    Ok(BridgePairing {
        token,
        port: BRIDGE_PORT,
        connected,
    })
}

#[tauri::command]
fn regenerate_bridge_token(state: State<'_, AppState>) -> Result<BridgePairing, String> {
    let token = {
        let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
        settings.bridge_token = default_bridge_token();
        save_settings(&state, &settings)?;
        settings.bridge_token.clone()
    };
    if let Ok(mut seen) = state.bridge_last_seen.lock() {
        *seen = None;
    }
    Ok(BridgePairing {
        token,
        port: BRIDGE_PORT,
        connected: false,
    })
}

#[tauri::command]
fn copy_bridge_token(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let token = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .bridge_token
        .clone();
    app.clipboard()
        .write_text(token)
        .map_err(|error| error.to_string())
}

fn bridge_origin(headers: &str) -> Option<&str> {
    headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if name.eq_ignore_ascii_case("origin") {
            let value = value.trim();
            (value.starts_with("chrome-extension://")
                || value.starts_with("moz-extension://")
                || value == "null")
                .then_some(value)
        } else {
            None
        }
    })
}

fn bridge_authorized(headers: &str, expected: &str) -> bool {
    headers.lines().any(|line| {
        line.split_once(':').is_some_and(|(name, value)| {
            name.eq_ignore_ascii_case("authorization")
                && value.trim() == format!("Bearer {expected}")
        })
    })
}

fn bridge_content_length(headers: &str) -> usize {
    headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0)
}

fn read_bridge_request(stream: &mut TcpStream) -> Option<Vec<u8>> {
    const MAX_REQUEST_SIZE: usize = 262_144;
    let mut request = Vec::with_capacity(8_192);
    let mut chunk = [0_u8; 8_192];
    loop {
        let count = stream.read(&mut chunk).ok()?;
        if count == 0 {
            break;
        }
        request.extend_from_slice(&chunk[..count]);
        if request.len() > MAX_REQUEST_SIZE {
            return None;
        }
        if let Some(header_end) = request.windows(4).position(|value| value == b"\r\n\r\n") {
            let body_start = header_end + 4;
            let headers = String::from_utf8_lossy(&request[..header_end]);
            if request.len() >= body_start + bridge_content_length(&headers) {
                break;
            }
        }
    }
    (!request.is_empty()).then_some(request)
}

fn bridge_response(stream: &mut TcpStream, status: &str, origin: Option<&str>, body: &str) {
    let cors = origin
        .map(|value| format!("Access-Control-Allow-Origin: {value}\r\nVary: Origin\r\n"))
        .unwrap_or_else(|| "Access-Control-Allow-Origin: *\r\n".to_owned());
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\n{cors}Access-Control-Allow-Headers: Authorization, Content-Type\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}

fn show_main_window(app: &tauri::AppHandle) {
    let main_app = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(window) = main_app.get_webview_window("main") {
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_always_on_top(true);
            let _ = window.set_focus();
            let _ = window.set_always_on_top(false);
        }
    });
}

#[tauri::command]
fn activate_main_window(app: tauri::AppHandle) {
    show_main_window(&app);
}

#[tauri::command]
fn open_paypal_donation() -> Result<(), String> {
    const URL: &str = "https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL";
    #[cfg(target_os = "windows")]
    let result = Command::new("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", URL])
        .spawn();
    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(URL).spawn();
    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open").arg(URL).spawn();
    result.map(|_| ()).map_err(|error| error.to_string())
}

fn queue_from_bridge(
    app: &tauri::AppHandle,
    mut request: BridgeDownload,
) -> Result<Option<DownloadId>, String> {
    let state = app.state::<AppState>();
    state.diagnostics.record("handoff.desktop_received", "INFO", request.trace_id.as_deref(), None,
        serde_json::json!({"url":request.url,"mediaKind":request.media_kind,"immediate":request.start_immediately,"paired":request.audio_url.is_some(),"ambiguous":request.ambiguous_social_track}));
    if request.ambiguous_social_track && request.audio_url.is_none() {
        state.diagnostics.record(
            "handoff.desktop_rejected",
            "WARN",
            request.trace_id.as_deref(),
            None,
            serde_json::json!({"reason":"incomplete_social_media_track"}),
        );
        diagnostic_log(
            &state,
            "WARN",
            "bridge.incomplete_social_track_rejected",
            &format!(
                "candidate={} reason=missing_permalink_or_companion_audio",
                redact_url(&request.url)
            ),
        );
        return Err("incomplete_social_media_track".to_owned());
    }
    if let Some(page_url) = contextual_media_page(
        &request.url,
        request.page_url.as_deref(),
        request.media_kind.as_deref(),
        request.audio_url.is_some(),
        request.request_method.as_deref(),
    ) {
        let original = std::mem::replace(&mut request.url, page_url.to_owned());
        request.request_method = None;
        request.request_body = None;
        request.request_content_type = None;
        diagnostic_log(
            &state,
            "INFO",
            "bridge.media_route_auto_selected",
            &format!(
                "from={} to={} engine=YtDlp reason=isolated_track_with_specific_page_context",
                redact_url(&original),
                redact_url(&request.url),
            ),
        );
    }
    let partial_video_candidate = request
        .media_kind
        .as_deref()
        .is_some_and(|kind| kind.eq_ignore_ascii_case("video"))
        && url::Url::parse(&request.url).ok().is_some_and(|url| {
            let mut has_byte_start = false;
            let mut has_byte_end = false;
            for (name, _) in url.query_pairs() {
                if name.eq_ignore_ascii_case("bytestart") {
                    has_byte_start = true;
                } else if name.eq_ignore_ascii_case("byteend") {
                    has_byte_end = true;
                }
            }
            has_byte_start && has_byte_end
        });
    if partial_video_candidate {
        if let Ok(mut expanded) = url::Url::parse(&request.url) {
            let original = request.url.clone();
            let query = expanded
                .query_pairs()
                .filter(|(name, _)| {
                    !name.eq_ignore_ascii_case("bytestart") && !name.eq_ignore_ascii_case("byteend")
                })
                .map(|(name, value)| (name.into_owned(), value.into_owned()))
                .collect::<Vec<_>>();
            expanded.set_query(None);
            expanded.query_pairs_mut().extend_pairs(query);
            request.url = expanded.into();
            diagnostic_log(
                &state,
                "INFO",
                "bridge.partial_media_candidate_expanded",
                &format!(
                    "candidate={} expanded={}",
                    redact_url(&original),
                    redact_url(&request.url)
                ),
            );
        }
    }
    if let Some(audio_url) = request.audio_url.as_mut() {
        if let Ok(mut expanded) = url::Url::parse(audio_url) {
            let query = expanded
                .query_pairs()
                .filter(|(name, _)| {
                    !name.eq_ignore_ascii_case("bytestart") && !name.eq_ignore_ascii_case("byteend")
                })
                .map(|(name, value)| (name.into_owned(), value.into_owned()))
                .collect::<Vec<_>>();
            expanded.set_query(None);
            expanded.query_pairs_mut().extend_pairs(query);
            *audio_url = expanded.into();
        }
    }
    let image_mislabeled_as_video = request
        .media_kind
        .as_deref()
        .is_some_and(|kind| kind.eq_ignore_ascii_case("video"))
        && url::Url::parse(&request.url)
            .ok()
            .and_then(|url| {
                Path::new(url.path())
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .map(str::to_ascii_lowercase)
            })
            .is_some_and(|extension| {
                matches!(
                    extension.as_str(),
                    "avif" | "bmp" | "gif" | "ico" | "jpg" | "jpeg" | "png" | "svg" | "webp"
                )
            });
    if image_mislabeled_as_video {
        if let Some(page_url) = request
            .page_url
            .clone()
            .filter(|url| matches!(classify_url(url), Some(DownloadKind::MediaPage)))
        {
            diagnostic_log(
                &state,
                "WARN",
                "bridge.video_image_candidate_rejected",
                &format!(
                    "candidate={} fallback={}",
                    redact_url(&request.url),
                    redact_url(&page_url)
                ),
            );
            request.url = page_url;
        }
    }
    classify_url(&request.url).ok_or_else(|| "unsupported_url".to_owned())?;
    let cookie_names = request
        .cookie_header
        .as_deref()
        .map(|value| {
            value
                .split(';')
                .filter_map(|item| item.trim().split_once('=').map(|(name, _)| name.trim()))
                .filter(|name| {
                    !name.is_empty()
                        && name.len() <= 80
                        && name.chars().all(|character| {
                            character.is_ascii_alphanumeric() || matches!(character, '_' | '-')
                        })
                })
                .take(24)
                .collect::<Vec<_>>()
                .join(",")
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "none".to_owned());
    diagnostic_log(
        &state,
        "INFO",
        "bridge.download",
        &format!(
            "url={} method={} media_kind={} companion_audio={} thumbnail={} expected_size={} user_agent={} cookie_count={} cookie_names={} cookie_bytes={}",
            redact_url(&request.url),
            request.request_method.as_deref().unwrap_or("GET"),
            request.media_kind.as_deref().unwrap_or("none"),
            request.audio_url.is_some(),
            request.thumbnail.is_some(),
            request.expected_size.map_or_else(|| "unknown".to_owned(), |value| value.to_string()),
            request.user_agent.as_deref().map(|value| value.split_whitespace().collect::<Vec<_>>().join(" ")).unwrap_or_else(|| "none".to_owned()),
            request.cookie_header.as_deref().map_or(0, |value| value.split(';').filter(|item| item.contains('=')).count()),
            cookie_names,
            request.cookie_header.as_deref().map_or(0, str::len),
        ),
    );
    if request.start_immediately {
        let context = DownloadContext {
            trace_id: request.trace_id.clone(),
            referer: request.page_url,
            known_duration: request.duration,
            title: request.title,
            thumbnail: request.thumbnail,
            audio_url: request.audio_url,
            cookie_header: request.cookie_header,
            user_agent: request.user_agent,
            request_method: request.request_method,
            request_body: request.request_body,
            request_content_type: request.request_content_type,
        };
        let task = enqueue_download_impl(
            app.clone(),
            &state,
            request.url,
            None,
            request.file_name,
            None,
            None,
            None,
            Some(10),
            None,
            None,
            Some(context),
        )?;
        show_main_window(app);
        return Ok(Some(task.id));
    }
    state.diagnostics.record(
        "handoff.save_dialog_pending",
        "INFO",
        request.trace_id.as_deref(),
        None,
        serde_json::json!({}),
    );
    state
        .bridge_pending
        .lock()
        .map_err(|error| error.to_string())?
        .push(request);
    show_main_window(app);
    let _ = app.emit("bridge-download-ready", ());
    let restored_app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(350));
        show_main_window(&restored_app);
        let _ = restored_app.emit("bridge-download-ready", ());
        std::thread::sleep(Duration::from_millis(850));
        show_main_window(&restored_app);
    });
    Ok(None)
}

fn register_browser_download(
    app: &tauri::AppHandle,
    request: BrowserDownloadComplete,
) -> Result<(), String> {
    if host_from_url(&request.url).is_none() {
        return Err("invalid_browser_download_url".to_owned());
    }
    let destination = PathBuf::from(request.file_name);
    if !destination.is_absolute() || !destination.is_file() {
        return Err("browser_download_not_found".to_owned());
    }
    let destination = destination
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let size = fs::metadata(&destination)
        .map_err(|error| error.to_string())?
        .len();
    let state = app.state::<AppState>();
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    if queue.iter().any(|task| {
        task.destination == destination
            && task.source == request.url
            && task.state == DownloadState::Completed
    }) {
        return Ok(());
    }
    let mut task = DownloadTask::new(&request.url, destination);
    task.state = DownloadState::Completed;
    task.received = size;
    task.total = Some(request.total.unwrap_or(size).max(size));
    task.progress_percent = Some(100.0);
    task.completed_at = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |value| value.as_secs()),
    );
    queue.push(task.clone());
    save_queue(&state, &queue)?;
    drop(queue);
    diagnostic_log(
        &state,
        "INFO",
        "browser_assisted.completed",
        &format!(
            "task={} bytes={} url={} file={}",
            task.id,
            size,
            redact_url(&task.source),
            task.destination.display()
        ),
    );
    show_main_window(app);
    Ok(())
}

fn association_id(source: &str) -> Option<&'static str> {
    let lower = source.to_ascii_lowercase();
    if lower.starts_with("magnet:") {
        Some("magnet")
    } else if lower.starts_with("sftp:") {
        Some("sftp")
    } else if lower.starts_with("ftp:") {
        Some("ftp")
    } else {
        let path = lower.split(['?', '#']).next().unwrap_or(&lower);
        if path.ends_with(".torrent") {
            Some("torrent")
        } else if path.ends_with(".m3u8") {
            Some("m3u8")
        } else {
            None
        }
    }
}

fn queue_associated_source(app: &tauri::AppHandle, source: String) -> Result<(), String> {
    let id = association_id(&source).ok_or_else(|| "unsupported_association".to_owned())?;
    let enabled = app
        .state::<AppState>()
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .associations
        .get(id)
        .copied()
        .unwrap_or(false);
    if !enabled {
        return Ok(());
    }
    queue_from_bridge(
        app,
        BridgeDownload {
            trace_id: None,
            url: source,
            audio_url: None,
            file_name: None,
            page_url: None,
            title: None,
            thumbnail: None,
            media_kind: None,
            ambiguous_social_track: false,
            expected_size: None,
            duration: None,
            cookie_header: None,
            user_agent: None,
            request_method: None,
            request_body: None,
            request_content_type: None,
            start_immediately: false,
        },
    )
    .map(|_| ())
}

#[tauri::command]
fn take_bridge_download(
    state: State<'_, AppState>,
    current_url: Option<String>,
) -> Result<Option<BridgeDownload>, String> {
    let mut pending = state
        .bridge_pending
        .lock()
        .map_err(|error| error.to_string())?;
    let index = current_url
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .and_then(|current| {
            let current = current.trim().trim_end_matches('#');
            pending.iter().position(|request| {
                request
                    .page_url
                    .as_deref()
                    .is_some_and(|page| page.trim().trim_end_matches('#') == current)
            })
        });
    let request = index
        .or_else(|| (!pending.is_empty()).then_some(0))
        .map(|index| pending.remove(index));
    drop(pending);
    if let Some(request) = request.as_ref() {
        diagnostic_log(
            &state,
            "INFO",
            "bridge.prompt",
            &format!("url={}", redact_url(&request.url)),
        );
    }
    Ok(request)
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    if value.len() > 131_072 || !value.len().is_multiple_of(2) {
        return Err("invalid_blob_chunk".to_owned());
    }
    (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16)
                .map_err(|_| "invalid_blob_chunk".to_owned())
        })
        .collect()
}

fn begin_blob_upload(app: &tauri::AppHandle, request: BlobBegin) -> Result<uuid::Uuid, String> {
    if request.total == 0 && !request.streaming {
        return Err("empty_blob".to_owned());
    }
    let state = app.state::<AppState>();
    let directory = configured_download_directory(app, &state)?;
    let file_name = validate_file_name(&request.file_name)?;
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    let mut task = DownloadTask::new(request.source, directory.join(&file_name));
    task.state = DownloadState::Downloading;
    task.total = (!request.streaming).then_some(request.total);
    let task_id = task.id;
    reserve_queued_task(&mut queue, &mut task, false)?;
    let destination = task.destination.clone();
    let partial = partial_path(&destination);
    if let Err(error) = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&partial)
    {
        queue.pop();
        return Err(error.to_string());
    }
    if let Err(error) = save_queue(&state, &queue) {
        queue.pop();
        let _ = fs::remove_file(&partial);
        return Err(error);
    }
    drop(queue);
    let upload_id = uuid::Uuid::new_v4();
    state
        .blob_uploads
        .lock()
        .map_err(|error| error.to_string())?
        .insert(
            upload_id,
            BlobUpload {
                task_id,
                partial,
                destination,
                received: 0,
                total: (!request.streaming).then_some(request.total),
                recording: request.recording,
                speed_sample_at: Instant::now(),
                speed_sample_bytes: 0,
                smoothed_speed: 0,
            },
        );
    diagnostic_log(
        &state,
        "INFO",
        "blob.start",
        &format!(
            "task={task_id} bytes={} streaming={}",
            request.total, request.streaming
        ),
    );
    Ok(upload_id)
}

fn append_blob_chunk(app: &tauri::AppHandle, request: BlobChunk) -> Result<(), String> {
    let data = decode_hex(&request.data)?;
    let state = app.state::<AppState>();
    let (task_id, received, total, download_speed) = {
        let mut uploads = state
            .blob_uploads
            .lock()
            .map_err(|error| error.to_string())?;
        let upload = uploads
            .get_mut(&request.upload_id)
            .ok_or_else(|| "blob_upload_not_found".to_owned())?;
        if upload
            .total
            .is_some_and(|total| upload.received + data.len() as u64 > total)
        {
            return Err("blob_too_large".to_owned());
        }
        OpenOptions::new()
            .append(true)
            .open(&upload.partial)
            .and_then(|mut file| file.write_all(&data))
            .map_err(|error| error.to_string())?;
        upload.received += data.len() as u64;
        let elapsed = upload.speed_sample_at.elapsed();
        if elapsed >= Duration::from_millis(200) {
            let bytes = upload.received.saturating_sub(upload.speed_sample_bytes);
            let instantaneous = (bytes as f64 / elapsed.as_secs_f64()) as u64;
            upload.smoothed_speed = if upload.smoothed_speed == 0 {
                instantaneous
            } else {
                (instantaneous as f64 * 0.65 + upload.smoothed_speed as f64 * 0.35) as u64
            };
            upload.speed_sample_at = Instant::now();
            upload.speed_sample_bytes = upload.received;
        }
        (
            upload.task_id,
            upload.received,
            upload.total,
            upload.smoothed_speed,
        )
    };
    update_task(app, task_id, false, |task| {
        task.received = received;
        task.progress_percent = total.map(|total| received as f64 * 100.0 / total as f64);
        task.download_speed = Some(download_speed);
    });
    let _ = app.emit(
        "blob-upload-progress",
        serde_json::json!({
            "taskId": task_id,
            "received": received,
            "total": total,
            "downloadSpeed": download_speed,
        }),
    );
    Ok(())
}

fn finish_blob_upload(app: &tauri::AppHandle, request: BlobFinish) -> Result<(), String> {
    let state = app.state::<AppState>();
    let upload = state
        .blob_uploads
        .lock()
        .map_err(|error| error.to_string())?
        .remove(&request.upload_id)
        .ok_or_else(|| "blob_upload_not_found".to_owned())?;
    if upload.total.is_some_and(|total| upload.received != total) || upload.received == 0 {
        return Err("incomplete_blob".to_owned());
    }
    fs::rename(&upload.partial, &upload.destination).map_err(|error| error.to_string())?;
    update_task(app, upload.task_id, true, |task| {
        task.received = upload.received;
        task.total = Some(upload.received);
        task.progress_percent = Some(100.0);
        task.download_speed = Some(0);
        task.state = DownloadState::Completed;
        task.completed_at = Some(epoch_seconds());
    });
    diagnostic_log(
        &state,
        "INFO",
        "blob.completed",
        &format!("task={} bytes={}", upload.task_id, upload.received),
    );
    state
        .recording_stops
        .lock()
        .map_err(|error| error.to_string())?
        .remove(&upload.task_id);
    show_main_window(app);
    if upload.recording {
        let _ = app.emit("recording-completed", upload.task_id);
    }
    Ok(())
}

#[tauri::command]
fn stop_recording(state: State<'_, AppState>, id: DownloadId) -> Result<(), String> {
    let active = state
        .blob_uploads
        .lock()
        .map_err(|error| error.to_string())?
        .values()
        .any(|upload| upload.task_id == id);
    if !active {
        return Err("recording_not_active".to_owned());
    }
    state
        .recording_stops
        .lock()
        .map_err(|error| error.to_string())?
        .insert(id);
    Ok(())
}

fn recording_stop_requested(app: &tauri::AppHandle, request: &BlobFinish) -> Result<bool, String> {
    let state = app.state::<AppState>();
    let task_id = state
        .blob_uploads
        .lock()
        .map_err(|error| error.to_string())?
        .get(&request.upload_id)
        .map(|upload| upload.task_id)
        .ok_or_else(|| "blob_upload_not_found".to_owned())?;
    let stop = state
        .recording_stops
        .lock()
        .map_err(|error| error.to_string())?
        .contains(&task_id);
    Ok(stop)
}

fn handle_bridge_connection(app: &tauri::AppHandle, mut stream: TcpStream) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
    let Some(buffer) = read_bridge_request(&mut stream) else {
        return;
    };
    let request = String::from_utf8_lossy(&buffer);
    let Some((headers, body)) = request.split_once("\r\n\r\n") else {
        return;
    };
    let origin = bridge_origin(headers);
    let has_origin = headers.lines().any(|line| {
        line.split_once(':')
            .is_some_and(|(name, _)| name.eq_ignore_ascii_case("origin"))
    });
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
    let token = match state.settings.lock() {
        Ok(settings) => settings.bridge_token.clone(),
        Err(_) => return,
    };
    if !bridge_authorized(headers, &token) {
        diagnostic_log(
            &state,
            "WARN",
            "bridge.authentication_failed",
            &format!(
                "request={} origin={}",
                first
                    .split_whitespace()
                    .take(2)
                    .collect::<Vec<_>>()
                    .join(" "),
                origin.unwrap_or("none")
            ),
        );
        bridge_response(&mut stream, "401 Unauthorized", origin, "{\"ok\":false}");
        return;
    }
    if let Ok(mut seen) = state.bridge_last_seen.lock() {
        *seen = Some(Instant::now());
    }
    diagnostic_log(
        &state,
        "DEBUG",
        "bridge.request",
        &format!(
            "request={} origin={}",
            first
                .split_whitespace()
                .take(2)
                .collect::<Vec<_>>()
                .join(" "),
            origin.unwrap_or("none")
        ),
    );
    if first.starts_with("GET /v1/health ") {
        diagnostic_log(
            &state,
            "DEBUG",
            "bridge.health",
            "extension heartbeat authenticated",
        );
        let (language, theme) = state
            .settings
            .lock()
            .ok()
            .map(|settings| (settings.language.clone(), settings.theme.clone()))
            .unwrap_or_else(|| (default_language(), default_theme()));
        bridge_response(
            &mut stream,
            "200 OK",
            origin,
            &serde_json::json!({ "ok": true, "language": language, "theme": theme }).to_string(),
        );
    } else if first.starts_with("GET /v1/activate ") {
        diagnostic_log(
            &state,
            "INFO",
            "application.second_instance",
            "existing instance activated",
        );
        show_main_window(app);
        bridge_response(&mut stream, "200 OK", origin, "{\"ok\":true}");
    } else if first.starts_with("GET /v1/diagnostics-v3/status ") {
        bridge_response(
            &mut stream,
            "200 OK",
            origin,
            &state.diagnostics.status().to_string(),
        );
    } else if first.starts_with("POST /v1/diagnostics-v3/control ") {
        let result = serde_json::from_str::<serde_json::Value>(body)
            .map_err(|_| "invalid_diagnostics_json".to_owned())
            .and_then(|value| {
                state.diagnostics.control(
                    value["action"].as_str().unwrap_or(""),
                    value["tabId"].as_i64(),
                )
            });
        match result {
            Ok(value) => bridge_response(&mut stream, "200 OK", origin, &value.to_string()),
            Err(error) => bridge_response(
                &mut stream,
                "400 Bad Request",
                origin,
                &serde_json::json!({"ok":false,"error":error}).to_string(),
            ),
        }
    } else if first.starts_with("POST /v1/diagnostics-v3/events ") {
        match serde_json::from_str::<serde_json::Value>(body) {
            Ok(value) => bridge_response(
                &mut stream,
                "202 Accepted",
                origin,
                &state.diagnostics.accept(&value).to_string(),
            ),
            Err(_) => bridge_response(&mut stream, "400 Bad Request", origin, "{\"ok\":false}"),
        }
    } else if first.starts_with("POST /v1/diagnostic ") {
        match serde_json::from_str::<ExtensionDiagnostic>(body) {
            Ok(item) => {
                let detail = format!(
                    "source={} trace={} url={} status={} bytes={} duration_ms={} delayed={} client_time={} {}",
                    item.source.as_deref().unwrap_or("browser-extension"),
                    item.trace_id.as_deref().unwrap_or("none"),
                    item.url.as_deref().map(redact_url).unwrap_or_else(|| "none".to_owned()),
                    item.status.map_or_else(|| "none".to_owned(), |v| v.to_string()),
                    item.bytes.map_or_else(|| "none".to_owned(), |v| v.to_string()),
                    item.duration_ms.map_or_else(|| "none".to_owned(), |v| v.to_string()),
                    item.delayed.unwrap_or(false),
                    item.client_timestamp.as_deref().unwrap_or("none"),
                    item.detail.as_deref().unwrap_or(""),
                );
                diagnostic_log(
                    &state,
                    if item.event.contains("failed") || item.event.contains("error") {
                        "ERROR"
                    } else if item.event.contains("unresolved") || item.event.contains("rejected") {
                        "WARN"
                    } else {
                        item.level.as_deref().unwrap_or("INFO")
                    },
                    &format!("extension.{}", item.event),
                    &detail,
                );
                bridge_response(&mut stream, "202 Accepted", origin, "{\"ok\":true}");
            }
            Err(error) => {
                diagnostic_log(
                    &state,
                    "WARN",
                    "bridge.diagnostic_invalid",
                    &error.to_string(),
                );
                bridge_response(&mut stream, "400 Bad Request", origin, "{\"ok\":false}");
            }
        }
    } else if first.starts_with("POST /v1/download ") {
        match serde_json::from_str::<BridgeDownload>(body)
            .map_err(|error| error.to_string())
            .and_then(|request| {
                let trace = request.trace_id.clone();
                let result = queue_from_bridge(app, request);
                state.diagnostics.record("handoff.desktop_reply", if result.is_ok() { "INFO" } else { "ERROR" }, trace.as_deref(), None,
                    serde_json::json!({"ok":result.is_ok(),"errorRef":result.as_ref().err(),"prompt":matches!(result,Ok(None))}));
                result
            })
        {
            Ok(Some(task_id)) => bridge_response(
                &mut stream,
                "202 Accepted",
                origin,
                &format!("{{\"ok\":true,\"taskId\":\"{task_id}\"}}"),
            ),
            Ok(None) => bridge_response(&mut stream, "202 Accepted", origin, "{\"ok\":true}"),
            Err(_) => bridge_response(&mut stream, "400 Bad Request", origin, "{\"ok\":false}"),
        }
    } else if first.starts_with("POST /v1/preview-media ") {
        match serde_json::from_str::<MediaPreviewRequest>(body)
            .map_err(|error| error.to_string())
            .and_then(|request| {
                let preparing = prepared_preview::is_candidate(&request);
                let trace = request.trace_id.clone();
                state.diagnostics.record("preview.desktop_received", "INFO", trace.as_deref(), None, serde_json::json!({"preparing":preparing}));
                let result = open_media_preview(app, &state, request).map(|()| preparing);
                state.diagnostics.record("preview.desktop_reply", if result.is_ok() { "INFO" } else { "ERROR" }, trace.as_deref(), None,
                    serde_json::json!({"ok":result.is_ok(),"preparing":preparing,"errorRef":result.as_ref().err()}));
                result
            }) {
            Ok(preparing) => bridge_response(
                &mut stream,
                "202 Accepted",
                origin,
                &serde_json::json!({"ok": true, "preparing": preparing}).to_string(),
            ),
            Err(error) => {
                diagnostic_log(&state, "ERROR", "media.preview_rejected", &error);
                bridge_response(
                    &mut stream,
                    "400 Bad Request",
                    origin,
                    &serde_json::json!({"ok": false, "error": error}).to_string(),
                );
            }
        }
    } else if first.starts_with("POST /v1/download-status ") {
        let status = serde_json::from_str::<BridgeDownloadStatus>(body)
            .ok()
            .and_then(|request| {
                state.queue.lock().ok().and_then(|queue| {
                    queue
                        .iter()
                        .find(|task| task.id == request.task_id)
                        .map(|task| match &task.state {
                            DownloadState::Completed => "completed",
                            DownloadState::Failed { .. } => "failed",
                            _ => "active",
                        })
                })
            });
        match status {
            Some(status) => bridge_response(
                &mut stream,
                "200 OK",
                origin,
                &format!("{{\"status\":\"{status}\"}}"),
            ),
            None => bridge_response(
                &mut stream,
                "404 Not Found",
                origin,
                "{\"status\":\"missing\"}",
            ),
        }
    } else if first.starts_with("POST /v1/browser-download-complete ") {
        match serde_json::from_str::<BrowserDownloadComplete>(body)
            .map_err(|error| error.to_string())
            .and_then(|request| register_browser_download(app, request))
        {
            Ok(()) => bridge_response(&mut stream, "202 Accepted", origin, "{\"ok\":true}"),
            Err(_) => bridge_response(&mut stream, "400 Bad Request", origin, "{\"ok\":false}"),
        }
    } else if first.starts_with("POST /v1/blob/begin ") {
        match serde_json::from_str::<BlobBegin>(body)
            .map_err(|error| error.to_string())
            .and_then(|request| begin_blob_upload(app, request))
        {
            Ok(upload_id) => bridge_response(
                &mut stream,
                "202 Accepted",
                origin,
                &format!("{{\"uploadId\":\"{upload_id}\"}}"),
            ),
            Err(_) => bridge_response(&mut stream, "400 Bad Request", origin, "{\"ok\":false}"),
        }
    } else if first.starts_with("POST /v1/blob/chunk ") {
        match serde_json::from_str::<BlobChunk>(body)
            .map_err(|error| error.to_string())
            .and_then(|request| append_blob_chunk(app, request))
        {
            Ok(()) => bridge_response(&mut stream, "202 Accepted", origin, "{\"ok\":true}"),
            Err(_) => bridge_response(&mut stream, "400 Bad Request", origin, "{\"ok\":false}"),
        }
    } else if first.starts_with("POST /v1/blob/status ") {
        match serde_json::from_str::<BlobFinish>(body)
            .map_err(|error| error.to_string())
            .and_then(|request| recording_stop_requested(app, &request))
        {
            Ok(stop) => bridge_response(
                &mut stream,
                "200 OK",
                origin,
                &format!("{{\"stop\":{stop}}}"),
            ),
            Err(_) => bridge_response(&mut stream, "400 Bad Request", origin, "{\"ok\":false}"),
        }
    } else if first.starts_with("POST /v1/blob/end ") {
        match serde_json::from_str::<BlobFinish>(body)
            .map_err(|error| error.to_string())
            .and_then(|request| finish_blob_upload(app, request))
        {
            Ok(()) => bridge_response(&mut stream, "200 OK", origin, "{\"ok\":true}"),
            Err(_) => bridge_response(&mut stream, "400 Bad Request", origin, "{\"ok\":false}"),
        }
    } else {
        bridge_response(&mut stream, "404 Not Found", origin, "{\"ok\":false}");
    }
}

fn run_extension_bridge(app: tauri::AppHandle, listener: TcpListener) {
    for stream in listener.incoming().flatten() {
        handle_bridge_connection(&app, stream);
    }
}

fn activate_running_instance(token: &str) -> bool {
    let Ok(mut stream) = TcpStream::connect(("127.0.0.1", BRIDGE_PORT)) else {
        return false;
    };
    let request = format!(
        "GET /v1/activate HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).is_ok()
}

fn forward_to_running_instance(source: &str, token: &str) -> bool {
    let Ok(mut stream) = TcpStream::connect(("127.0.0.1", BRIDGE_PORT)) else {
        return false;
    };
    let request = BridgeDownload {
        trace_id: None,
        url: source.to_owned(),
        audio_url: None,
        file_name: None,
        page_url: None,
        title: None,
        thumbnail: None,
        media_kind: None,
        ambiguous_social_track: false,
        expected_size: None,
        duration: None,
        cookie_header: None,
        user_agent: None,
        request_method: None,
        request_body: None,
        request_content_type: None,
        start_immediately: false,
    };
    let Ok(body) = serde_json::to_string(&request) else {
        return false;
    };
    let message = format!(
        "POST /v1/download HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    if stream.write_all(message.as_bytes()).is_err() {
        return false;
    }
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let mut response = String::new();
    stream.read_to_string(&mut response).is_ok() && response.starts_with("HTTP/1.1 202")
}

#[tauri::command]
fn reveal_download(state: State<'_, AppState>, id: DownloadId) -> Result<(), String> {
    let queue = state.queue.lock().map_err(|error| error.to_string())?;
    let task = queue
        .iter()
        .find(|task| task.id == id)
        .ok_or_else(|| "download_not_found".to_owned())?;
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    let partial = partial_path(&task.destination);
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    let target = if task.destination.exists() {
        Some(task.destination.clone())
    } else if partial.exists() {
        Some(partial)
    } else {
        None
    };
    let directory = task
        .destination
        .parent()
        .ok_or_else(|| "download_directory_not_found".to_owned())?;
    #[cfg(target_os = "windows")]
    let result = if let Some(target) = &target {
        Command::new("explorer.exe")
            .arg("/select,")
            .arg(target)
            .spawn()
    } else {
        Command::new("explorer.exe").arg(directory).spawn()
    };
    #[cfg(target_os = "macos")]
    let result = if let Some(target) = &target {
        Command::new("open").arg("-R").arg(target).spawn()
    } else {
        Command::new("open").arg(directory).spawn()
    };
    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open").arg(directory).spawn();
    result.map(|_| ()).map_err(|error| error.to_string())
}

#[tauri::command]
async fn verify_download_integrity(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: DownloadId,
    expected_sha256: Option<String>,
) -> Result<String, String> {
    let path = {
        let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
        let task = queue
            .iter_mut()
            .find(|task| task.id == id)
            .ok_or_else(|| "download_not_found".to_owned())?;
        if !task.destination.is_file() {
            return Err("download_file_not_found".to_owned());
        }
        task.state = DownloadState::Verifying;
        task.destination.clone()
    };
    let digest = tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        let mut file = fs::File::open(path).map_err(|error| error.to_string())?;
        let mut hasher = Sha256::new();
        let mut buffer = [0_u8; 1024 * 1024];
        loop {
            let count = file.read(&mut buffer).map_err(|error| error.to_string())?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    })
    .await
    .map_err(|error| error.to_string())??;
    let expected = expected_sha256
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let verified = expected.as_ref().is_none_or(|value| value == &digest);
    update_task(&app, id, true, |task| {
        task.state = DownloadState::Completed;
        task.sha256 = Some(digest.clone());
        task.integrity_verified = verified;
    });
    if verified {
        Ok(digest)
    } else {
        Err(format!("checksum_mismatch:{digest}"))
    }
}

#[tauri::command]
async fn export_recording(
    state: State<'_, AppState>,
    id: DownloadId,
    format: String,
    video_codec: String,
    audio_codec: String,
    output_directory: String,
) -> Result<String, String> {
    const FORMATS: [&str; 8] = ["mp4", "mkv", "webm", "mp3", "m4a", "opus", "flac", "wav"];
    if !FORMATS.contains(&format.as_str()) {
        return Err("unsupported_export_format".to_owned());
    }
    let source = {
        let queue = state.queue.lock().map_err(|error| error.to_string())?;
        let task = queue
            .iter()
            .find(|task| task.id == id)
            .ok_or_else(|| "download_not_found".to_owned())?;
        if task.state != DownloadState::Completed
            || !task
                .destination
                .to_string_lossy()
                .ends_with(".recording.webm")
        {
            return Err("recording_not_complete".to_owned());
        }
        task.destination.clone()
    };
    let ffmpeg = {
        let settings = state.settings.lock().map_err(|error| error.to_string())?;
        configured_tool(
            &settings.ffmpeg_path,
            if cfg!(windows) {
                "ffmpeg.exe"
            } else {
                "ffmpeg"
            },
        )
    };
    let stem = source
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("recording.recording.webm")
        .trim_end_matches(".recording.webm");
    let output_directory = PathBuf::from(output_directory);
    if !output_directory.is_dir() {
        return Err("export_directory_not_found".to_owned());
    }
    let output = unique_destination(&output_directory, &format!("{stem}.{format}"))?;
    let audio_only = matches!(format.as_str(), "mp3" | "m4a" | "opus" | "flac" | "wav");
    let mut command = tokio::process::Command::new(ffmpeg);
    command.args(["-y", "-i"]).arg(&source);
    if audio_only {
        command.arg("-vn");
    } else {
        let codec = match video_codec.as_str() {
            "copy" => "copy",
            "h264" => "libx264",
            "hevc" => "libx265",
            "vp9" => "libvpx-vp9",
            "av1" => "libaom-av1",
            _ => return Err("unsupported_video_codec".to_owned()),
        };
        command.args(["-c:v", codec]);
    }
    let codec = match audio_codec.as_str() {
        "copy" => "copy",
        "aac" => "aac",
        "opus" => "libopus",
        "mp3" => "libmp3lame",
        "flac" => "flac",
        _ => return Err("unsupported_audio_codec".to_owned()),
    };
    command.args(["-c:a", codec]).arg(&output);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.as_std_mut().creation_flags(0x08000000);
    }
    let result = command.output().await.map_err(|error| error.to_string())?;
    if !result.status.success() {
        return Err(String::from_utf8_lossy(&result.stderr).trim().to_owned());
    }
    let size = fs::metadata(&output)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    if output != source {
        fs::remove_file(&source)
            .map_err(|error| format!("recording_source_cleanup_failed: {error}"))?;
    }
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    let task = queue
        .iter_mut()
        .find(|task| task.id == id)
        .ok_or_else(|| "download_not_found".to_owned())?;
    task.destination = output.clone();
    task.received = size;
    task.total = Some(size);
    task.progress_percent = Some(100.0);
    task.completed_at = Some(epoch_seconds());
    save_queue(&state, &queue)?;
    Ok(output.to_string_lossy().into_owned())
}

#[cfg(target_os = "windows")]
fn autostart_enabled(_: &tauri::AppHandle) -> Result<bool, String> {
    Command::new("reg.exe")
        .args([
            "QUERY",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            "/v",
            "ApocalipseDownloadManager",
        ])
        .status()
        .map(|status| status.success())
        .map_err(|error| error.to_string())
}

#[cfg(target_os = "windows")]
fn configure_autostart(_: &tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let mut command = Command::new("reg.exe");
    command.args(if enabled {
        vec![
            "ADD".into(),
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run".into(),
            "/v".into(),
            "ApocalipseDownloadManager".into(),
            "/t".into(),
            "REG_SZ".into(),
            "/d".into(),
            format!("\"{}\" --hidden", executable.display()),
            "/f".into(),
        ]
    } else {
        vec![
            "DELETE".into(),
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run".into(),
            "/v".into(),
            "ApocalipseDownloadManager".into(),
            "/f".into(),
        ]
    });
    let status = command.status().map_err(|error| error.to_string())?;
    if enabled && !status.success() {
        return Err("autostart_update_failed".to_owned());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn autostart_entry(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .config_dir()
        .map(|path| path.join("autostart/apocalipse-download-manager.desktop"))
        .map_err(|error| error.to_string())
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
        if let Some(parent) = entry.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let escaped = executable.to_string_lossy().replace('"', "\\\"");
        fs::write(entry, format!("[Desktop Entry]\nType=Application\nName=Apocalipse Download Manager\nExec=\"{escaped}\" --hidden\nTerminal=false\nX-GNOME-Autostart-enabled=true\n" )).map_err(|error| error.to_string())?;
    } else if entry.exists() {
        fs::remove_file(entry).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn autostart_entry(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .home_dir()
        .map(|path| path.join("Library/LaunchAgents/com.linuxhell.apocalipse.plist"))
        .map_err(|error| error.to_string())
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
        if let Some(parent) = entry.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let escaped = executable
            .to_string_lossy()
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        let plist = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\"><dict><key>Label</key><string>com.linuxhell.apocalipse</string><key>ProgramArguments</key><array><string>{escaped}</string><string>--hidden</string></array><key>RunAtLoad</key><true/></dict></plist>\n");
        fs::write(entry, plist).map_err(|error| error.to_string())?;
    } else if entry.exists() {
        fs::remove_file(entry).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_autostart(app: tauri::AppHandle) -> Result<AutostartStatus, String> {
    Ok(AutostartStatus {
        enabled: autostart_enabled(&app)?,
    })
}

#[tauri::command]
fn set_autostart(app: tauri::AppHandle, enabled: bool) -> Result<AutostartStatus, String> {
    configure_autostart(&app, enabled)?;
    get_autostart(app)
}

const ASSOCIATION_IDS: [&str; 5] = ["m3u8", "torrent", "magnet", "ftp", "sftp"];

#[cfg(target_os = "windows")]
fn configure_association(id: &str, enabled: bool, _: &HashMap<String, bool>) -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let root = if matches!(id, "m3u8" | "torrent") {
        format!(r"HKCU\Software\Classes\.{id}")
    } else {
        format!(r"HKCU\Software\Classes\{id}")
    };
    if !enabled {
        let _ = Command::new("reg.exe")
            .args(["DELETE", &root, "/f"])
            .status();
        if matches!(id, "m3u8" | "torrent") {
            let prog_id = format!(r"HKCU\Software\Classes\Apocalipse.{id}");
            let _ = Command::new("reg.exe")
                .args(["DELETE", &prog_id, "/f"])
                .status();
        }
        return Ok(());
    }
    let prog_id = format!("Apocalipse.{id}");
    let class_root = if matches!(id, "m3u8" | "torrent") {
        let status = Command::new("reg.exe")
            .args(["ADD", &root, "/ve", "/t", "REG_SZ", "/d", &prog_id, "/f"])
            .status()
            .map_err(|error| error.to_string())?;
        if !status.success() {
            return Err("association_update_failed".to_owned());
        }
        format!(r"HKCU\Software\Classes\{prog_id}")
    } else {
        let status = Command::new("reg.exe")
            .args([
                "ADD",
                &root,
                "/v",
                "URL Protocol",
                "/t",
                "REG_SZ",
                "/d",
                "",
                "/f",
            ])
            .status()
            .map_err(|error| error.to_string())?;
        if !status.success() {
            return Err("association_update_failed".to_owned());
        }
        root
    };
    let command_key = format!(r"{class_root}\shell\open\command");
    let open_command = format!("\"{}\" --open \"%1\"", executable.display());
    let status = Command::new("reg.exe")
        .args([
            "ADD",
            &command_key,
            "/ve",
            "/t",
            "REG_SZ",
            "/d",
            &open_command,
            "/f",
        ])
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("association_update_failed".to_owned())
    }
}

#[cfg(target_os = "linux")]
fn configure_association(
    _: &str,
    _: bool,
    associations: &HashMap<String, bool>,
) -> Result<(), String> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "home_directory_unavailable".to_owned())?;
    let applications = home.join(".local/share/applications");
    fs::create_dir_all(&applications).map_err(|error| error.to_string())?;
    let entry = applications.join("apocalipse-download-manager.desktop");
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let definitions = [
        ("m3u8", "application/vnd.apple.mpegurl"),
        ("torrent", "application/x-bittorrent"),
        ("magnet", "x-scheme-handler/magnet"),
        ("ftp", "x-scheme-handler/ftp"),
        ("sftp", "x-scheme-handler/sftp"),
    ];
    let enabled = definitions
        .iter()
        .filter(|(id, _)| associations.get(*id).copied().unwrap_or(false))
        .collect::<Vec<_>>();
    if enabled.is_empty() {
        if entry.exists() {
            fs::remove_file(&entry).map_err(|error| error.to_string())?;
        }
    } else {
        let mime_types = enabled
            .iter()
            .map(|(_, mime)| *mime)
            .collect::<Vec<_>>()
            .join(";");
        let escaped = executable.to_string_lossy().replace('"', "\\\"");
        fs::write(&entry, format!("[Desktop Entry]\nType=Application\nName=Apocalipse Download Manager\nExec=\"{escaped}\" --open %U\nTerminal=false\nNoDisplay=true\nMimeType={mime_types};\n"))
            .map_err(|error| error.to_string())?;
        for (_, mime) in enabled {
            let _ = Command::new("xdg-mime")
                .args(["default", "apocalipse-download-manager.desktop", mime])
                .status();
        }
    }
    let _ = Command::new("update-desktop-database")
        .arg(&applications)
        .status();
    Ok(())
}

#[cfg(target_os = "macos")]
fn configure_association(_: &str, _: bool, _: &HashMap<String, bool>) -> Result<(), String> {
    // LaunchServices reads the handlers from the application bundle. Individual switches
    // control whether incoming items are accepted by Apocalipse.
    Ok(())
}

#[tauri::command]
fn get_associations(state: State<'_, AppState>) -> Result<Vec<AssociationStatus>, String> {
    let settings = state.settings.lock().map_err(|error| error.to_string())?;
    Ok(ASSOCIATION_IDS
        .iter()
        .map(|id| AssociationStatus {
            id: (*id).to_owned(),
            enabled: settings.associations.get(*id).copied().unwrap_or(false),
            supported: true,
        })
        .collect())
}

#[tauri::command]
fn set_association(state: State<'_, AppState>, id: String, enabled: bool) -> Result<(), String> {
    if !ASSOCIATION_IDS.contains(&id.as_str()) {
        return Err("unsupported_association".to_owned());
    }
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    let mut associations = settings.associations.clone();
    associations.insert(id.clone(), enabled);
    configure_association(&id, enabled, &associations)?;
    settings.associations = associations;
    save_settings(&state, &settings)
}

#[tauri::command]
async fn remove_downloads(
    state: State<'_, AppState>,
    ids: Vec<DownloadId>,
    delete_files: bool,
) -> Result<usize, String> {
    let removal_trace = uuid::Uuid::new_v4().to_string();
    state.diagnostics.record("task.removal_requested", "INFO", Some(&removal_trace), None,
        serde_json::json!({"taskRefs":ids.iter().map(|id|id.to_string()).collect::<Vec<_>>(),"deleteFiles":delete_files}));
    if ids.is_empty() {
        return Ok(0);
    }
    // Stop queued entries before signalling workers, so completion of another
    // task cannot redispatch an item during the asynchronous removal window.
    {
        let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
        for task in queue.iter_mut().filter(|task| ids.contains(&task.id)) {
            if task.state == DownloadState::Queued {
                task.state = DownloadState::Paused;
            }
        }
        save_queue(&state, &queue)?;
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
    let removed = state
        .queue
        .lock()
        .map_err(|error| error.to_string())?
        .iter()
        .filter(|task| ids.contains(&task.id))
        .cloned()
        .collect::<Vec<_>>();
    if delete_files {
        for task in &removed {
            cleanup_chunk_artifacts(&task.destination)
                .await
                .map_err(|error| error.to_string())?;
            for path in download_paths(task) {
                let torrent_root = matches!(
                    classify_url(&task.source),
                    Some(DownloadKind::Torrent | DownloadKind::Magnet)
                ) && path == task.destination;
                remove_path_with_retry(&path, torrent_root).await?;
            }
        }
    }
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    queue.retain(|task| !ids.contains(&task.id));
    if let Ok(mut identities) = state.request_identities.lock() {
        identities.retain(|id, _| !ids.contains(id));
    }
    save_queue(&state, &queue)?;
    diagnostic_log(
        &state,
        "INFO",
        "task.removed",
        &format!("count={} delete_files={delete_files}", removed.len()),
    );
    for task in &removed {
        state.diagnostics.record(
            "task.removal_confirmed",
            "INFO",
            Some(&removal_trace),
            Some(&task.id.to_string()),
            serde_json::json!({"persisted":true}),
        );
    }
    Ok(removed.len())
}

fn download_paths(task: &DownloadTask) -> Vec<PathBuf> {
    let partial = partial_path(&task.destination);
    let mut paths = vec![task.destination.clone(), partial];
    let recording_source = PathBuf::from(&task.source);
    if task.source.ends_with(".recording.webm") && recording_source.is_absolute() {
        paths.push(recording_source);
    }
    let stem = task
        .destination
        .file_stem()
        .and_then(|value| value.to_str());
    if let (Some(parent), Some(stem)) = (task.destination.parent(), stem) {
        for extension in [
            "mp4", "mkv", "ts", "webm", "m4a", "mp3", "wav", "flac", "opus", "aac",
        ] {
            let candidate = parent.join(format!("{stem}.{extension}"));
            if !paths.contains(&candidate) {
                paths.push(candidate);
            }
        }
    }
    paths
}

async fn remove_path_with_retry(path: &Path, allow_directory: bool) -> Result<(), String> {
    for attempt in 0..5 {
        let result = match tokio::fs::metadata(path).await {
            Ok(metadata)
                if metadata.is_dir()
                    && allow_directory
                    && path.parent().is_some()
                    && path.file_name().is_some() =>
            {
                tokio::fs::remove_dir_all(path).await
            }
            Ok(metadata) if metadata.is_dir() => {
                return Err("refusing_to_remove_directory".to_owned())
            }
            _ => tokio::fs::remove_file(path).await,
        };
        match result {
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
            let app_data = portable_data_directory(app)?;
            let queue_path = app_data.join("queue.json");
            let settings_path = app_data.join("settings.json");
            let log_path = app_data.join("logs").join("apocalipse.log");
            let initial_settings = load_settings(&settings_path);
            let (show_label, quit_label) = tray_labels(&initial_settings.language);
            let show = MenuItem::with_id(app, "show", show_label, true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", quit_label, true, None::<&str>)?;
            let arguments = std::env::args().collect::<Vec<_>>();
            let associated_source = arguments
                .iter()
                .position(|argument| argument == "--open")
                .and_then(|index| arguments.get(index + 1))
                .filter(|value| classify_url(value).is_some())
                .cloned();
            let bridge_listener = match TcpListener::bind(("127.0.0.1", BRIDGE_PORT)) {
                Ok(listener) => Some(listener),
                Err(_) => {
                    if let Some(source) = associated_source.as_deref() {
                        let _ = forward_to_running_instance(source, &initial_settings.bridge_token);
                    } else {
                        let _ = activate_running_instance(&initial_settings.bridge_token);
                    }
                    app.handle().exit(0);
                    return Ok(());
                }
            };
            let global_bandwidth_limiter = Arc::new(BandwidthLimiter::new(
                initial_settings.global_bandwidth_limit,
            ));
            app.manage(AppState {
                queue: Mutex::new(load_queue(&queue_path)),
                queue_path,
                workers: Mutex::new(HashMap::new()),
                settings: Mutex::new(initial_settings),
                settings_path,
                bridge_last_seen: Mutex::new(None),
                bridge_pending: Mutex::new(Vec::new()),
                blob_uploads: Mutex::new(HashMap::new()),
                recording_stops: Mutex::new(HashSet::new()),
                request_identities: Mutex::new(HashMap::new()),
                log_path,
                log_write_lock: Mutex::new(()),
                diagnostics: diagnostics_v3::Diagnostics::new(&app_data.join("logs")),
                global_bandwidth_limiter,
                download_bandwidth_limiters: Mutex::new(HashMap::new()),
                tray_show: show.clone(),
                tray_quit: quit.clone(),
            });
            diagnostic_log(
                &app.state::<AppState>(),
                "INFO",
                "application.started",
                env!("CARGO_PKG_VERSION"),
            );
            if let Some(listener) = bridge_listener {
                let bridge_app = app.handle().clone();
                std::thread::Builder::new()
                    .name("apocalipse-extension-bridge".into())
                    .spawn(move || run_extension_bridge(bridge_app, listener))?;
            }
            // File/control access stays local by default. Exposing it to the LAN requires a
            // separately designed authenticated transport instead of an implicit wildcard bind.
            if let Ok(listener) = TcpListener::bind(("127.0.0.1", LINK_PORT)) {
                let link_app = app.handle().clone();
                std::thread::Builder::new()
                    .name("apocalipse-link-server".into())
                    .spawn(move || run_link_server(link_app, listener))?;
            }
            if let Some(source) = associated_source {
                queue_associated_source(app.handle(), source).map_err(std::io::Error::other)?;
            }
            let menu = Menu::with_items(app, &[&show, &quit])?;
            // The detailed application artwork loses definition at the 16–24 px sizes used by
            // system trays. Keep a simplified, high-contrast asset specifically for this role.
            let icon = Image::new_owned(include_bytes!("../icons/tray.rgba").to_vec(), 32, 32);
            TrayIconBuilder::new()
                .icon(icon)
                .tooltip("Apocalipse Download Manager")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => show_main_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if matches!(
                        event,
                        TrayIconEvent::DoubleClick {
                            button: MouseButton::Left,
                            ..
                        }
                    ) {
                        show_main_window(tray.app_handle());
                    }
                })
                .build(app)?;
            if std::env::args().any(|argument| argument == "--hidden") {
                if let Some(window) = app.get_webview_window("main") {
                    window.hide()?;
                }
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
            inspect_torrent_metadata,
            get_link_identity,
            regenerate_link_password,
            list_local_link_files,
            list_remote_link_files,
            download_remote_link_file,
            upload_remote_link_file,
            list_downloads,
            enqueue_download,
            default_download_directory,
            set_default_download_directory,
            pick_directory,
            pick_executable,
            pick_url_list,
            activate_main_window,
            open_paypal_donation,
            get_tool_statuses,
            set_tool_paths,
            get_media_player,
            get_app_version,
            set_media_player,
            preview_torrent,
            update_tool,
            suggest_download_name,
            remove_downloads,
            read_general_log,
            clear_general_log,
            export_diagnostic_bundle,
            diagnostics_status,
            diagnostics_control,
            copy_diagnostics_report,
            record_diagnostics_ui,
            record_ui_diagnostic,
            set_application_language,
            set_application_theme,
            get_log_editor,
            set_log_editor,
            open_log_external,
            stop_recording,
            pause_download,
            resume_download,
            redownload_downloads,
            reveal_download,
            verify_download_integrity,
            export_recording,
            get_autostart,
            set_autostart,
            get_associations,
            set_association,
            get_clipboard_monitor,
            set_clipboard_monitor,
            read_clipboard_link,
            get_transfer_limits,
            set_transfer_limits,
            set_download_bandwidth_limit,
            get_user_agent,
            set_user_agent,
            get_proxy_setting,
            set_proxy_setting,
            list_website_credentials,
            save_website_credential,
            remove_website_credential,
            get_dns_setting,
            set_dns_setting,
            get_bridge_pairing,
            regenerate_bridge_token,
            copy_bridge_token,
            list_download_directories,
            remove_download_directory,
            clear_download_directories,
            take_bridge_download
        ])
        .build(tauri::generate_context!())
        .expect("failed to build Apocalipse Download Manager")
        .run(|_app, _event| {
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = _event {
                for url in urls {
                    let source = url
                        .to_file_path()
                        .map(|path| path.to_string_lossy().into_owned())
                        .unwrap_or_else(|_| url.to_string());
                    let _ = queue_associated_source(_app, source);
                }
            }
        });
}

#[cfg(test)]
mod tests {
    #[test]
    fn stale_scheduler_snapshot_cannot_dispatch_a_removed_or_stopped_task() {
        let task = DownloadTask::new("https://example.test/file.bin", PathBuf::from("file.bin"));
        let mut queue = vec![task.clone()];
        assert!(queued_task_for_start(&queue, task.id).is_some());
        queue[0].state = DownloadState::Paused;
        assert!(queued_task_for_start(&queue, task.id).is_none());
        queue.clear();
        queue.push(DownloadTask::new(
            "https://example.test/new.bin",
            PathBuf::from("new.bin"),
        ));
        assert!(queued_task_for_start(&queue, task.id).is_none());
    }

    use super::*;

    struct HandoffTestDirectory(PathBuf);

    impl HandoffTestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("adm-handoff-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for HandoffTestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn handoff_reserves_names_before_any_worker_creates_a_file() {
        let directory = HandoffTestDirectory::new();
        let mut queue = Vec::new();
        for index in 0..3 {
            let mut task = DownloadTask::new(
                format!("https://media.example/{index}"),
                directory.0.join("download.mp4"),
            );
            reserve_queued_task(&mut queue, &mut task, true).unwrap();
        }
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 0);
        assert_eq!(queue[0].destination, directory.0.join("download.mp4"));
        assert_eq!(queue[1].destination, directory.0.join("download (1).mp4"));
        assert_eq!(queue[2].destination, directory.0.join("download (2).mp4"));
    }

    #[test]
    fn handoff_concurrent_workers_keep_distinct_payloads_and_destinations() {
        let directory = HandoffTestDirectory::new();
        let queue = Mutex::new(Vec::new());
        let barrier = std::sync::Barrier::new(32);
        std::thread::scope(|scope| {
            for index in 0..32 {
                let queue = &queue;
                let barrier = &barrier;
                let destination = directory.0.join("download.mp4");
                scope.spawn(move || {
                    barrier.wait();
                    let source = format!("https://media.example/{index}/?mime_type=video_mp4");
                    let mut task = DownloadTask::new(source.clone(), destination);
                    {
                        let mut queue = queue.lock().unwrap();
                        reserve_queued_task(&mut queue, &mut task, true).unwrap();
                    }
                    // Every reservation exists before ANY producer writes a partial file.
                    barrier.wait();
                    fs::write(partial_path(&task.destination), source.as_bytes()).unwrap();
                    fs::rename(partial_path(&task.destination), &task.destination).unwrap();
                });
            }
        });
        let queue = queue.lock().unwrap();
        assert_eq!(queue.len(), 32);
        assert_eq!(
            queue
                .iter()
                .map(|task| &task.destination)
                .collect::<HashSet<_>>()
                .len(),
            32
        );
        for task in queue.iter() {
            assert_eq!(fs::read_to_string(&task.destination).unwrap(), task.source);
        }
    }

    #[test]
    fn handoff_duplicate_url_check_is_atomic() {
        let directory = HandoffTestDirectory::new();
        let queue = Mutex::new(Vec::new());
        let barrier = std::sync::Barrier::new(16);
        let accepted = std::sync::atomic::AtomicUsize::new(0);
        std::thread::scope(|scope| {
            for _ in 0..16 {
                let (queue, barrier, accepted) = (&queue, &barrier, &accepted);
                let destination = directory.0.join("video.mp4");
                scope.spawn(move || {
                    let mut task = DownloadTask::new("https://media.example/same", destination);
                    barrier.wait();
                    let result = reserve_queued_task(&mut queue.lock().unwrap(), &mut task, true);
                    match result {
                        Ok(()) => {
                            accepted.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        }
                        Err(error) => assert!(error.starts_with("duplicate_active_download:")),
                    }
                });
            }
        });
        assert_eq!(accepted.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(queue.lock().unwrap().len(), 1);
    }

    #[test]
    fn handoff_preserves_existing_files_partials_and_chunk_artifacts() {
        let directory = HandoffTestDirectory::new();
        fs::write(directory.0.join("video.mp4"), b"original").unwrap();
        fs::write(partial_path(&directory.0.join("video (1).mp4")), b"partial").unwrap();
        fs::create_dir_all(apocalipse_core::chunk_directory(
            &directory.0.join("video (2).mp4"),
        ))
        .unwrap();
        let mut queue = Vec::new();
        let mut task =
            DownloadTask::new("https://media.example/new", directory.0.join("video.mp4"));
        reserve_queued_task(&mut queue, &mut task, true).unwrap();
        assert_eq!(task.destination, directory.0.join("video (3).mp4"));
        assert_eq!(
            fs::read(directory.0.join("video.mp4")).unwrap(),
            b"original"
        );
        assert_eq!(
            fs::read(partial_path(&directory.0.join("video (1).mp4"))).unwrap(),
            b"partial"
        );
    }

    #[test]
    fn handoff_keeps_paused_failed_and_completed_queue_paths_reserved() {
        let directory = HandoffTestDirectory::new();
        for state in [
            DownloadState::Paused,
            DownloadState::Failed {
                message: "test".into(),
            },
            DownloadState::Completed,
        ] {
            let mut original =
                DownloadTask::new("https://media.example/old", directory.0.join("video.mp4"));
            original.state = state;
            let mut queue = vec![original];
            let mut task =
                DownloadTask::new("https://media.example/new", directory.0.join("video.mp4"));
            reserve_queued_task(&mut queue, &mut task, true).unwrap();
            assert_eq!(task.destination, directory.0.join("video (1).mp4"));
        }
    }

    #[test]
    fn handoff_redownload_can_repeat_a_source_without_reusing_its_destination() {
        let directory = HandoffTestDirectory::new();
        let mut queue = Vec::new();
        for _ in 0..3 {
            let mut task =
                DownloadTask::new("https://media.example/same", directory.0.join("download"));
            reserve_queued_task(&mut queue, &mut task, false).unwrap();
        }
        assert_eq!(
            queue
                .iter()
                .map(|task| &task.destination)
                .collect::<HashSet<_>>()
                .len(),
            3
        );
        assert_eq!(queue[2].destination, directory.0.join("download (2)"));
    }

    #[test]
    fn handoff_reserves_partial_namespace_and_normalized_paths() {
        let directory = HandoffTestDirectory::new();
        let original =
            DownloadTask::new("https://media.example/old", directory.0.join("video.mp4"));
        let mut queue = vec![original];
        let mut partial_named = DownloadTask::new(
            "https://media.example/new",
            directory.0.join("video.mp4.part"),
        );
        reserve_queued_task(&mut queue, &mut partial_named, true).unwrap();
        assert_ne!(
            partial_named.destination,
            partial_path(&queue[0].destination)
        );
        let mut normalized = DownloadTask::new(
            "https://media.example/other",
            directory.0.join("sub/../video.mp4"),
        );
        reserve_queued_task(&mut queue, &mut normalized, true).unwrap();
        assert_ne!(
            destination_key(&normalized.destination),
            destination_key(&queue[0].destination)
        );
    }

    #[cfg(any(windows, target_os = "macos"))]
    #[test]
    fn handoff_reserves_case_insensitive_file_names() {
        let directory = HandoffTestDirectory::new();
        let mut queue = vec![DownloadTask::new(
            "https://media.example/old",
            directory.0.join("Video.MP4"),
        )];
        let mut task =
            DownloadTask::new("https://media.example/new", directory.0.join("video.mp4"));
        reserve_queued_task(&mut queue, &mut task, true).unwrap();
        assert_eq!(task.destination, directory.0.join("video (1).mp4"));
    }

    #[test]
    fn handoff_exhausted_names_fail_instead_of_using_an_unchecked_fallback() {
        let directory = HandoffTestDirectory::new();
        let mut queue = vec![DownloadTask::new(
            "https://media.example/0",
            directory.0.join("video.mp4"),
        )];
        for index in 1..10_000 {
            queue.push(DownloadTask::new(
                format!("https://media.example/{index}"),
                directory.0.join(format!("video ({index}).mp4")),
            ));
        }
        let mut task =
            DownloadTask::new("https://media.example/new", directory.0.join("video.mp4"));
        assert_eq!(
            reserve_queued_task(&mut queue, &mut task, true).unwrap_err(),
            "download_destination_names_exhausted"
        );
        assert_eq!(queue.len(), 10_000);
    }

    #[test]
    fn handoff_signed_media_query_supplies_extension_without_mutating_url() {
        let source =
            "https://media.example/video/opaque/?a=1988&&mime_type=video_mp4&signature=a%2Bb%3D";
        assert_eq!(
            append_source_extension("download".into(), source, DownloadKind::Http),
            "download.mp4"
        );
        assert_eq!(
            append_source_extension("chosen.mkv".into(), source, DownloadKind::Http),
            "chosen.mkv"
        );
        assert_eq!(
            append_source_extension(
                "audio".into(),
                "https://media.example/?mime_type=audio%2Fmp4",
                DownloadKind::Http
            ),
            "audio.m4a"
        );
        assert_eq!(
            append_source_extension(
                "clip".into(),
                "https://media.example/?mime_type=video_webm",
                DownloadKind::Http
            ),
            "clip.webm"
        );
        assert_eq!(
            append_source_extension(
                "download".into(),
                "https://media.example/?mime_type=text_html",
                DownloadKind::Http
            ),
            "download"
        );
    }

    #[test]
    fn normalizes_site_credential_domains_and_matches_subdomains() {
        assert_eq!(
            normalize_credential_host("https://Example.COM/").as_deref(),
            Ok("example.com")
        );
        assert!(normalize_credential_host("https://example.com/login").is_err());
        let mut settings = UserSettings::default();
        settings.website_credentials.push(WebsiteCredential {
            host: "example.com".into(),
            username: "user".into(),
            password: "secret".into(),
        });
        assert_eq!(
            website_credential_for_url(&settings, "https://cdn.example.com/file")
                .map(|credential| credential.username.as_str()),
            Some("user")
        );
        assert!(website_credential_for_url(&settings, "https://notexample.com/file").is_none());
    }

    #[test]
    fn applies_rsload_credentials_only_to_its_known_download_host() {
        let mut settings = UserSettings::default();
        settings.website_credentials.push(WebsiteCredential {
            host: "rsload.net".into(),
            username: "rsload".into(),
            password: "rsload".into(),
        });
        assert!(website_credential_for_download(
            &settings,
            "https://s4.fixti.net/files/freeware/file.zip",
            Some("https://rsload.net/software/page.html")
        )
        .is_some());
        assert!(website_credential_for_download(
            &settings,
            "https://unrelated.example/file.zip",
            Some("https://rsload.net/software/page.html")
        )
        .is_none());
    }

    #[test]
    fn media_page_names_always_receive_mp4_extension() {
        assert_eq!(
            append_source_extension(
                "TikTok video".into(),
                "https://www.tiktok.com/@creator/video/123",
                DownloadKind::MediaPage
            ),
            "TikTok video.mp4"
        );
        assert_eq!(
            append_source_extension(
                "video.mkv".into(),
                "https://example.test/video",
                DownloadKind::MediaPage
            ),
            "video.mkv"
        );
    }

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
        let path =
            std::env::temp_dir().join(format!("apocalipse-missing-{}.mp4", uuid::Uuid::new_v4()));
        assert!(remove_path_with_retry(&path, false).await.is_ok());
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

    #[test]
    fn parses_real_aria2_transfer_progress() {
        assert_eq!(
            parse_aria2_progress("[#abc 5.0MiB/20MiB(25%) CN:4 SD:2 DL:1MiB ETA:15s]"),
            Some((
                5 * 1024 * 1024,
                20 * 1024 * 1024,
                25.0,
                1024 * 1024,
                0,
                2,
                2,
                Some("15s".to_owned())
            ))
        );
        assert_eq!(parse_aria2_progress("[#abc 0B/0B CN:1 DL:0B]"), None);
    }

    #[test]
    fn parses_bridge_content_length_case_insensitively() {
        assert_eq!(
            bridge_content_length("POST / HTTP/1.1\r\nContent-Length: 123"),
            123
        );
        assert_eq!(
            bridge_content_length("GET / HTTP/1.1\r\ncontent-length: 0"),
            0
        );
    }

    #[test]
    fn diagnostic_urls_hide_query_values_and_fragments() {
        assert_eq!(
            redact_url("https://example.com/file.zip?id=123&token=secret#part"),
            "https://example.com/file.zip?id=<redacted>&token=<redacted>",
        );
        assert_eq!(
            redact_url("https://example.com/file.zip#part"),
            "https://example.com/file.zip"
        );
        assert_eq!(
            redact_url("http://user:secret@proxy.example:8080/file"),
            "http://proxy.example:8080/file"
        );
        assert_eq!(
            redact_url("https://www.tiktok.com/@creator/video/123?q=test"),
            "https://www.tiktok.com/@creator/video/123?q=<redacted>"
        );
    }

    #[test]
    fn diagnostic_details_remove_credentials() {
        assert_eq!(sanitize_log_detail("Cookie: secret"), "<redacted>");
        assert_eq!(
            sanitize_log_detail("url=https://example.com/a?h=secret&e=123"),
            "url=https://example.com/a?h=<redacted>&e=<redacted>",
        );
    }

    #[test]
    fn external_proxy_credentials_are_url_encoded() {
        assert_eq!(
            external_proxy_url("socks5h://127.0.0.1:1080", Some("user name"), Some("p@ss")),
            "socks5h://user%20name:p%40ss@127.0.0.1:1080",
        );
    }

    #[test]
    fn parses_and_deduplicates_dns_servers() {
        let servers = vec![
            "1.1.1.1".to_owned(),
            " 1.1.1.1 ".to_owned(),
            "2606:4700:4700::1111".to_owned(),
        ];
        let parsed = parse_dns_servers(&servers).expect("valid DNS servers");
        assert_eq!(parsed.len(), 2);
        assert!(parse_dns_servers(&["not-an-address".to_owned()]).is_err());
    }
}
