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
    is_live: bool,
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
    path: Str