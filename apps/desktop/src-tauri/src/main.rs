Warning: truncated output (original token count: 69853)
Total output lines: 7952

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
    clipboard_suppressed_until: Mutex<Option<Instant>>,
    clipboard_suppressed_value: Mutex<Option<String>>,
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

fn site_connection_override(url: &str, requested: Option<usize>) -> Option<usize> {
    let host = host_from_url(url);
    if host
        .as_deref()
        .is_some_and(|value| value == "pixeldrain.com" || value.ends_with(".pixeldrain.com"))
    {
        // Pixeldrain may reject or destabilize segmented requests. Keep the
        // transfer on its single original stream regardless of the global or
        // per-task connection preference.
        Some(1)
    } else {
        requested.map(|value| value.clamp(1, 32))
    }
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

#[derive(Serialize)]
struct AppUpdateStatus {
    current_version: String,
    latest_version: String,
    update_available: bool,
}

fn version_numbers(value: &str) -> Vec<u64> {
    value
        .trim()
        .trim_start_matches(['v', 'V'])
        .split('.')
        .map(|part| part.split(|character: char| !character.is_ascii_digit()).next().unwrap_or("0"))
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect()
}

#[tauri::command]
async fn check_app_update() -> Result<AppUpdateStatus, String> {
    let current = env!("CARGO_PKG_VERSION").to_owned();
    let response = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .map_err(|error| error.to_string())?
        .get("https://api.github.com/repos/linuxhell/apocalipse-download-manager/releases/latest")
        .header(reqwest::header::USER_AGENT, "Apocalipse-Download-Manager")
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    let payload: serde_json::Value = response.json().await.map_err(|error| error.to_string())?;
    let latest = payload["tag_name"]
        .as_str()
        .ok_or_else(|| "release_without_tag".to_owned())?
        .trim_start_matches(['v', 'V'])
        .to_owned();
    Ok(AppUpdateStatus {
        update_available: version_numbers(&latest) > version_numbers(&current),
        current_version: current,
        latest_version: latest,
    })
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
    …39853 tokens truncated…             "202 Accepted",
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
    } else if first.starts_with("POST /v1/clipboard-suppress ") {
        let trace = serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|value| value["traceId"].as_str().map(str::to_owned))
            .filter(|value| uuid::Uuid::parse_str(value).is_ok());
        if let Ok(mut until) = state.clipboard_suppressed_until.lock() {
            *until = Some(Instant::now() + Duration::from_secs(4));
        }
        state.diagnostics.record(
            "clipboard.preview_identity_suppressed",
            "INFO",
            trace.as_deref(),
            None,
            serde_json::json!({"durationMs":4000,"saveDialogBlocked":true}),
        );
        diagnostic_log(
            &state,
            "DEBUG",
            "clipboard.preview_identity_suppressed",
            "duration_ms=4000",
        );
        bridge_response(&mut stream, "202 Accepted", origin, "{\"ok\":true}");
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
                let hls_workspace = matches!(classify_url(&task.source), Some(DownloadKind::Hls))
                    && hls_workspace_path(task).as_ref() == Some(&path);
                remove_path_with_retry(&path, torrent_root || hls_workspace).await?;
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
    if matches!(classify_url(&task.source), Some(DownloadKind::Hls)) {
        if let Some(workspace) = hls_workspace_path(task) {
            if !paths.contains(&workspace) {
                paths.push(workspace);
            }
        }
    }
    paths
}

fn hls_workspace_path(task: &DownloadTask) -> Option<PathBuf> {
    let parent = task.destination.parent()?;
    let stem = task.destination.file_stem()?;
    if stem.is_empty() {
        return None;
    }
    Some(parent.join(stem))
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
                clipboard_suppressed_until: Mutex::new(None),
                clipboard_suppressed_value: Mutex::new(None),
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
            check_app_update,
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
    fn pixeldrain_always_uses_one_connection_without_matching_spoofed_hosts() {
        assert_eq!(
            site_connection_override("https://pixeldrain.com/u/example", Some(8)),
            Some(1)
        );
        assert_eq!(
            site_connection_override("https://cdn.pixeldrain.com/api/file/example", None),
            Some(1)
        );
        assert_eq!(
            site_connection_override("https://pixeldrain.com.evil.test/file", Some(8)),
            Some(8)
        );
        assert_eq!(
            site_connection_override("https://example.test/file", None),
            None
        );
    }

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
        assert!(paths.contains(&PathBuf::from("C:/Downloads/157651625")));
        assert_eq!(
            hls_workspace_path(&task),
            Some(PathBuf::from("C:/Downloads/157651625"))
        );
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
