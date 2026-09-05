#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use apocalipse_core::{
    classify_url, cleanup_chunk_artifacts, partial_path, plan_download, Capabilities,
    DownloadEngine, DownloadEvent, DownloadId, DownloadKind, DownloadRequest, DownloadState,
    DownloadTask,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs,
    fs::OpenOptions,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream, UdpSocket},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::Mutex,
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
    site_rules: Mutex<Vec<SiteRule>>,
    site_rules_path: PathBuf,
    ed2k_search: Mutex<Option<Ed2kSearchSession>>,
}

struct Ed2kSearchSession {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl Drop for Ed2kSearchSession {
    fn drop(&mut self) {
        let _ = self.input.write_all(b"quit\n");
        let _ = self.input.flush();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum SiteRuleAction {
    Standard,
    SingleConnection,
    UupdumpPost,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SiteRule {
    id: String,
    name: String,
    hosts: Vec<String>,
    action: SiteRuleAction,
    enabled: bool,
    connections: usize,
}

fn default_site_rules() -> Vec<SiteRule> {
    vec![
        SiteRule {
            id: "uupdump".to_owned(),
            name: "UUP dump".to_owned(),
            hosts: vec!["uupdump.net".to_owned(), "*.uupdump.net".to_owned()],
            action: SiteRuleAction::UupdumpPost,
            enabled: true,
            connections: 1,
        },
        SiteRule {
            id: "rapidgator".to_owned(),
            name: "Rapidgator".to_owned(),
            hosts: vec!["rapidgator.net".to_owned(), "*.rapidgator.net".to_owned()],
            action: SiteRuleAction::SingleConnection,
            enabled: true,
            connections: 1,
        },
        SiteRule {
            id: "pixeldrain".to_owned(),
            name: "Pixeldrain".to_owned(),
            hosts: vec!["pixeldrain.com".to_owned(), "*.pixeldrain.com".to_owned()],
            action: SiteRuleAction::SingleConnection,
            enabled: true,
            connections: 1,
        },
    ]
}

fn valid_site_rule(rule: &SiteRule) -> bool {
    !rule.id.trim().is_empty()
        && rule.id.len() <= 64
        && !rule.name.trim().is_empty()
        && rule.name.len() <= 120
        && !rule.hosts.is_empty()
        && rule.hosts.len() <= 32
        && rule.hosts.iter().all(|host| {
            let host = host.trim().trim_start_matches("*.");
            !host.is_empty()
                && host.len() <= 253
                && host.chars().all(|character| {
                    character.is_ascii_alphanumeric() || matches!(character, '.' | '-')
                })
        })
        && (1..=32).contains(&rule.connections)
}

fn load_site_rules(path: &Path) -> Vec<SiteRule> {
    let loaded = fs::read(path)
        .ok()
        .and_then(|data| serde_json::from_slice::<Vec<SiteRule>>(&data).ok())
        .filter(|rules| {
            !rules.is_empty() && rules.len() <= 100 && rules.iter().all(valid_site_rule)
        });
    let Some(mut rules) = loaded else {
        return default_site_rules();
    };
    for rule in default_site_rules() {
        if rules.len() < 100 && !rules.iter().any(|existing| existing.id == rule.id) {
            rules.push(rule);
        }
    }
    rules
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

fn matching_site_rule(url: &str, rules: &[SiteRule]) -> Option<SiteRule> {
    let host = host_from_url(url)?;
    rules
        .iter()
        .find(|rule| {
            rule.enabled
                && rule
                    .hosts
                    .iter()
                    .any(|pattern| host_matches_pattern(&host, pattern))
        })
        .cloned()
}

fn host_matches_pattern(host: &str, pattern: &str) -> bool {
    let pattern = pattern.trim().to_ascii_lowercase();
    pattern
        .strip_prefix("*.")
        .map_or(host == pattern, |suffix| {
            host == suffix || host.ends_with(&format!(".{suffix}"))
        })
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
    ed2k_path: Option<PathBuf>,
    #[serde(default = "default_ed2k_host")]
    ed2k_host: String,
    #[serde(default = "default_ed2k_port")]
    ed2k_port: u16,
    #[serde(default)]
    ed2k_password: String,
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
    dns_enabled: bool,
    #[serde(default)]
    dns_servers: Vec<std::net::IpAddr>,
    #[serde(default)]
    associations: HashMap<String, bool>,
    #[serde(default = "default_link_password")]
    link_password: String,
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
fn default_ed2k_host() -> String {
    "127.0.0.1".to_owned()
}
const fn default_ed2k_port() -> u16 {
    4712
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            download_directory: None,
            capture_clipboard: false,
            max_active_downloads: default_max_active(),
            connections_per_download: default_connections(),
            adaptive_efficiency: true,
            bridge_token: default_bridge_token(),
            recent_download_directories: Vec::new(),
            ffmpeg_path: None,
            yt_dlp_path: None,
            n_m3u8dl_re_path: None,
            aria2_path: None,
            ed2k_path: None,
            ed2k_host: default_ed2k_host(),
            ed2k_port: default_ed2k_port(),
            ed2k_password: String::new(),
            media_player_path: None,
            user_agent: None,
            log_editor_path: None,
            proxy_enabled: false,
            proxy_url: None,
            proxy_username: None,
            proxy_password: None,
            dns_enabled: false,
            dns_servers: Vec::new(),
            associations: HashMap::new(),
            link_password: default_link_password(),
        }
    }
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Ed2kEngineStatus {
    helper_found: bool,
    controller_found: bool,
    daemon_found: bool,
    version: Option<String>,
    connected: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Ed2kConnectionSetting {
    host: String,
    port: u16,
    password_configured: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Ed2kNetworkStatus {
    ed2k_connected: bool,
    kad_connected: bool,
    high_id: bool,
    firewalled: bool,
    download_speed: String,
    upload_speed: String,
    sources: u64,
    raw: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Ed2kSearchResult {
    number: u64,
    name: String,
    size_mib: f64,
    sources: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Ed2kSearchResponse {
    search_id: Option<u64>,
    results: Vec<Ed2kSearchResult>,
    raw: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Ed2kTransfer {
    hash: String,
    name: String,
    percent: f64,
    active_sources: u64,
    total_sources: u64,
    status: String,
    priority: String,
    speed: String,
}

fn amule_component(helper: &Path, names: &[&str]) -> Option<PathBuf> {
    let directory = helper
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    names
        .iter()
        .map(|name| directory.join(name))
        .find(|path| path.is_file())
}

#[tauri::command]
fn get_ed2k_engine_status(state: State<'_, AppState>) -> Result<Ed2kEngineStatus, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    let helper = configured_ed2k_tool(&settings);
    let version = version_line(&helper, &["--version"]);
    let connected = amule_command(&settings, "status").is_ok();
    Ok(Ed2kEngineStatus {
        helper_found: version.is_some(),
        controller_found: amule_component(
            &helper,
            &[if cfg!(windows) {
                "amulecmd.exe"
            } else {
                "amulecmd"
            }],
        )
        .is_some(),
        daemon_found: amule_component(
            &helper,
            &[
                if cfg!(windows) {
                    "amuled.exe"
                } else {
                    "amuled"
                },
                if cfg!(windows) { "amule.exe" } else { "amule" },
            ],
        )
        .is_some(),
        version,
        connected,
    })
}

fn amule_controller(settings: &UserSettings) -> PathBuf {
    let helper = configured_ed2k_tool(settings);
    amule_component(
        &helper,
        &[if cfg!(windows) {
            "amulecmd.exe"
        } else {
            "amulecmd"
        }],
    )
    .unwrap_or_else(|| {
        PathBuf::from(if cfg!(windows) {
            "amulecmd.exe"
        } else {
            "amulecmd"
        })
    })
}

fn configure_amule_command(command: &mut Command, settings: &UserSettings) -> Result<(), String> {
    if settings.ed2k_password.is_empty() {
        return Err("ed2k_password_required".to_owned());
    }
    command.args([
        "-h",
        &settings.ed2k_host,
        "-p",
        &settings.ed2k_port.to_string(),
        "-P",
        &settings.ed2k_password,
        "-l",
        "en",
    ]);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    Ok(())
}

fn amule_command(settings: &UserSettings, instruction: &str) -> Result<String, String> {
    let mut command = Command::new(amule_controller(settings));
    configure_amule_command(&mut command, settings)?;
    let output = command
        .args(["--command", instruction])
        .output()
        .map_err(|error| format!("amulecmd_not_found: {error}"))?;
    let text = if output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stderr).into_owned()
    } else {
        String::from_utf8_lossy(&output.stdout).into_owned()
    };
    if output.status.success()
        && !text.contains("Connection Failed")
        && !text.contains("Request failed")
    {
        Ok(text)
    } else {
        Err(text.trim().to_owned())
    }
}

fn parse_after_label<'a>(text: &'a str, label: &str) -> &'a str {
    text.lines()
        .find_map(|line| line.split_once(label).map(|(_, value)| value.trim()))
        .unwrap_or("")
}

#[tauri::command]
fn get_ed2k_connection(state: State<'_, AppState>) -> Result<Ed2kConnectionSetting, String> {
    let settings = state.settings.lock().map_err(|error| error.to_string())?;
    Ok(Ed2kConnectionSetting {
        host: settings.ed2k_host.clone(),
        port: settings.ed2k_port,
        password_configured: !settings.ed2k_password.is_empty(),
    })
}

#[tauri::command]
fn set_ed2k_connection(
    state: State<'_, AppState>,
    host: String,
    port: u16,
    password: String,
) -> Result<(), String> {
    if host.trim().is_empty() || port == 0 {
        return Err("invalid_ed2k_connection".to_owned());
    }
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.ed2k_host = host.trim().to_owned();
    settings.ed2k_port = port;
    if !password.is_empty() {
        settings.ed2k_password = password;
    }
    *state
        .ed2k_search
        .lock()
        .map_err(|error| error.to_string())? = None;
    save_settings(&state, &settings)
}

#[tauri::command]
fn start_ed2k_engine(state: State<'_, AppState>) -> Result<(), String> {
    let settings = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    let helper = configured_ed2k_tool(&settings);
    let daemon = amule_component(
        &helper,
        &[
            if cfg!(windows) {
                "amuled.exe"
            } else {
                "amuled"
            },
            if cfg!(windows) { "amule.exe" } else { "amule" },
        ],
    )
    .ok_or_else(|| "amule_engine_not_found".to_owned())?;
    let mut command = Command::new(daemon);
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

#[tauri::command]
fn connect_ed2k_networks(state: State<'_, AppState>) -> Result<(), String> {
    let settings = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    amule_command(&settings, "connect")?;
    Ok(())
}

#[tauri::command]
fn ed2k_network_status(state: State<'_, AppState>) -> Result<Ed2kNetworkStatus, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    let raw = amule_command(&settings, "status")?;
    let ed2k_line = parse_after_label(&raw, "eD2k:");
    let kad_line = parse_after_label(&raw, "Kad:");
    Ok(Ed2kNetworkStatus {
        ed2k_connected: ed2k_line.starts_with("Connected"),
        kad_connected: kad_line.starts_with("Connected"),
        high_id: ed2k_line.contains("HighID"),
        firewalled: kad_line.contains("firewalled") || ed2k_line.contains("LowID"),
        download_speed: parse_after_label(&raw, "Download:").to_owned(),
        upload_speed: parse_after_label(&raw, "Upload:").to_owned(),
        sources: parse_after_label(&raw, "Total sources:")
            .parse()
            .unwrap_or(0),
        raw,
    })
}

fn start_search_session(settings: &UserSettings) -> Result<Ed2kSearchSession, String> {
    let mut command = Command::new(amule_controller(settings));
    configure_amule_command(&mut command, settings)?;
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = command
        .spawn()
        .map_err(|error| format!("amulecmd_not_found: {error}"))?;
    let input = child
        .stdin
        .take()
        .ok_or_else(|| "amulecmd_stdin".to_owned())?;
    let output = BufReader::new(
        child
            .stdout
            .take()
            .ok_or_else(|| "amulecmd_stdout".to_owned())?,
    );
    let mut session = Ed2kSearchSession {
        child,
        input,
        output,
    };
    read_amule_prompt(&mut session)?;
    Ok(session)
}

fn read_amule_prompt(session: &mut Ed2kSearchSession) -> Result<String, String> {
    let mut bytes = Vec::new();
    loop {
        let available = session
            .output
            .fill_buf()
            .map_err(|error| error.to_string())?;
        if available.is_empty() {
            return Err("amulecmd_closed".to_owned());
        }
        let take = available.len();
        bytes.extend_from_slice(&available[..take]);
        session.output.consume(take);
        if bytes.ends_with(b"aMulecmd$ ") {
            break;
        }
        if bytes.len() > 4 * 1024 * 1024 {
            return Err("amulecmd_output_too_large".to_owned());
        }
    }
    Ok(String::from_utf8_lossy(&bytes)
        .trim_end_matches("aMulecmd$ ")
        .to_owned())
}

fn interactive_amule(session: &mut Ed2kSearchSession, instruction: &str) -> Result<String, String> {
    if instruction
        .chars()
        .any(|value| matches!(value, '\r' | '\n'))
    {
        return Err("invalid_ed2k_command".to_owned());
    }
    writeln!(session.input, "{instruction}")
        .and_then(|_| session.input.flush())
        .map_err(|error| error.to_string())?;
    read_amule_prompt(session)
}

fn parse_search_results(raw: &str) -> Vec<Ed2kSearchResult> {
    raw.lines()
        .filter_map(|line| {
            let (number, rest) = line.trim_start().split_once('.')?;
            let number = number.parse().ok()?;
            let mut tail = rest.trim().rsplitn(3, char::is_whitespace);
            let sources = tail.next()?.parse().ok()?;
            let size_mib = tail.next()?.parse().ok()?;
            let name = tail.next()?.trim().to_owned();
            (!name.is_empty()).then_some(Ed2kSearchResult {
                number,
                name,
                size_mib,
                sources,
            })
        })
        .collect()
}

#[tauri::command]
fn ed2k_search(
    state: State<'_, AppState>,
    query: String,
    search_type: String,
    file_type: String,
) -> Result<Ed2kSearchResponse, String> {
    let query = query.trim();
    if query.len() < 2 || query.chars().any(|value| matches!(value, '\r' | '\n')) {
        return Err("invalid_ed2k_search".to_owned());
    }
    let network = match search_type.as_str() {
        "kad" => "kad",
        "local" => "local",
        _ => "global",
    };
    let filter = match file_type.as_str() {
        "Audio" | "Video" | "Image" | "Doc" | "Pro" | "Arc" | "Iso" => {
            format!(" --type {file_type}")
        }
        _ => String::new(),
    };
    let settings = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    let mut guard = state
        .ed2k_search
        .lock()
        .map_err(|error| error.to_string())?;
    if guard
        .as_mut()
        .is_some_and(|session| session.child.try_wait().ok().flatten().is_some())
    {
        *guard = None;
    }
    if guard.is_none() {
        *guard = Some(start_search_session(&settings)?);
    }
    let raw = interactive_amule(
        guard.as_mut().unwrap(),
        &format!("search {network}{filter} {query}"),
    )?;
    let search_id = raw
        .split("Search started (id ")
        .nth(1)
        .and_then(|value| value.split(')').next())
        .and_then(|value| value.parse().ok());
    Ok(Ed2kSearchResponse {
        search_id,
        results: Vec::new(),
        raw,
    })
}

#[tauri::command]
fn ed2k_search_results(
    state: State<'_, AppState>,
    search_id: Option<u64>,
) -> Result<Ed2kSearchResponse, String> {
    let mut guard = state
        .ed2k_search
        .lock()
        .map_err(|error| error.to_string())?;
    let session = guard
        .as_mut()
        .ok_or_else(|| "ed2k_search_not_started".to_owned())?;
    let raw = interactive_amule(
        session,
        &search_id.map_or_else(|| "results".to_owned(), |id| format!("results {id}")),
    )?;
    Ok(Ed2kSearchResponse {
        search_id,
        results: parse_search_results(&raw),
        raw,
    })
}

#[tauri::command]
fn ed2k_download_result(state: State<'_, AppState>, number: u64) -> Result<(), String> {
    let mut guard = state
        .ed2k_search
        .lock()
        .map_err(|error| error.to_string())?;
    let session = guard
        .as_mut()
        .ok_or_else(|| "ed2k_search_not_started".to_owned())?;
    interactive_amule(session, &format!("download {number}"))?;
    Ok(())
}

#[tauri::command]
fn list_ed2k_transfers(state: State<'_, AppState>) -> Result<Vec<Ed2kTransfer>, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    let raw = amule_command(&settings, "show dl")?;
    let mut transfers = Vec::new();
    let mut lines = raw.lines().peekable();
    while let Some(line) = lines.next() {
        let clean = line.trim_start_matches(" > ");
        let Some((hash, name)) = clean.split_once('\t') else {
            continue;
        };
        if hash.len() != 32 {
            continue;
        }
        let detail = lines.next().unwrap_or("").trim_start_matches(" > ").trim();
        let fields = detail.split('\t').map(str::trim).collect::<Vec<_>>();
        let percent = fields
            .first()
            .and_then(|value| {
                value
                    .trim_matches(|character| matches!(character, '[' | ']' | '%'))
                    .parse()
                    .ok()
            })
            .unwrap_or(0.0);
        let (active_sources, total_sources) = fields
            .get(1)
            .and_then(|value| value.split_once('/'))
            .map(|(a, b)| (a.trim().parse().unwrap_or(0), b.trim().parse().unwrap_or(0)))
            .unwrap_or((0, 0));
        transfers.push(Ed2kTransfer {
            hash: hash.to_owned(),
            name: name.to_owned(),
            percent,
            active_sources,
            total_sources,
            status: fields.get(4).unwrap_or(&"").to_string(),
            priority: fields.get(6).unwrap_or(&"").to_string(),
            speed: fields.last().unwrap_or(&"").to_string(),
        });
    }
    Ok(transfers)
}

#[tauri::command]
fn control_ed2k_transfer(
    state: State<'_, AppState>,
    action: String,
    hash: String,
) -> Result<(), String> {
    if hash.len() != 32 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("invalid_ed2k_hash".to_owned());
    }
    let instruction = match action.as_str() {
        "pause" => "pause",
        "resume" => "resume",
        "cancel" => "cancel",
        "low" => "priority low",
        "normal" => "priority normal",
        "high" => "priority high",
        "auto" => "priority auto",
        _ => return Err("invalid_ed2k_action".to_owned()),
    };
    let settings = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    amule_command(&settings, &format!("{instruction} {hash}"))?;
    Ok(())
}

fn configured_tool(path: &Option<PathBuf>, fallback: &str) -> PathBuf {
    path.clone()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| PathBuf::from(fallback))
}

fn bundled_ed2k_helper() -> Option<PathBuf> {
    let executable_directory = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))?;
    let file_name = if cfg!(windows) { "ed2k.exe" } else { "ed2k" };
    ["Data", "data"]
        .into_iter()
        .map(|directory| {
            executable_directory
                .join(directory)
                .join("ed2k")
                .join(file_name)
        })
        .find(|path| path.is_file())
}

fn configured_ed2k_tool(settings: &UserSettings) -> PathBuf {
    settings
        .ed2k_path
        .clone()
        .filter(|value| !value.as_os_str().is_empty())
        .or_else(bundled_ed2k_helper)
        .unwrap_or_else(|| PathBuf::from(if cfg!(windows) { "ed2k.exe" } else { "ed2k" }))
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
    url: String,
    file_name: Option<String>,
    page_url: Option<String>,
    duration: Option<f64>,
    cookie_header: Option<String>,
    user_agent: Option<String>,
    request_method: Option<String>,
    request_body: Option<String>,
    request_content_type: Option<String>,
}

struct BlobUpload {
    task_id: DownloadId,
    partial: PathBuf,
    destination: PathBuf,
    received: u64,
    total: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BlobBegin {
    file_name: String,
    total: u64,
    source: String,
    #[serde(default)]
    streaming: bool,
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
    referer: Option<String>,
    known_duration: Option<f64>,
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
    UdpSocket::bind(("0.0.0.0", 0))
        .ok()
        .and_then(|socket| {
            socket.connect(("8.8.8.8", 80)).ok()?;
            socket.local_addr().ok().map(|address| address.ip())
        })
        .filter(|ip| !ip.is_loopback())
        .unwrap_or_else(|| "127.0.0.1".parse().expect("valid loopback"))
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
                        url: request.url,
                        file_name: None,
                        page_url: None,
                        duration: None,
                        cookie_header: None,
                        user_agent: None,
                        request_method: None,
                        request_body: None,
                        request_content_type: None,
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
        amule: false,
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
) -> Result<MediaInspection, String> {
    let executable = configured_tool(
        &state
            .settings
            .lock()
            .map_err(|error| error.to_string())?
            .yt_dlp_path,
        "yt-dlp",
    );
    let mut command = tokio::process::Command::new(executable);
    command
        .args([
            "--dump-single-json",
            "--no-playlist",
            "--skip-download",
            "--no-warnings",
        ])
        .arg(&url);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.as_std_mut().creation_flags(0x08000000);
    }
    let output = command
        .output()
        .await
        .map_err(|error| format!("yt_dlp_unavailable: {error}"))?;
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
    for name in ["queue.json", "settings.json", "site-rules.json"] {
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
    fs::write(&state.queue_path, data).map_err(|error| error.to_string())
}

fn load_settings(path: &Path) -> UserSettings {
    fs::read(path)
        .ok()
        .and_then(|data| serde_json::from_slice(&data).ok())
        .unwrap_or_default()
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
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |value| value.as_secs());
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&state.log_path)
    {
        let _ = writeln!(
            file,
            "[{timestamp}] {level} {event} {}",
            sanitize_log_detail(detail)
        );
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
    mirrors: Vec<String>,
    mut cancellation: oneshot::Receiver<()>,
) {
    diagnostic_log(
        &app.state::<AppState>(),
        "INFO",
        "http.start",
        &format!("task={id} url={}", redact_url(&request.url)),
    );
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
                Some(DownloadEvent::Started { resumed_at, total, connections }) => {
                    diagnostic_log(&app.state::<AppState>(), "INFO", "http.mode", &format!("task={id} connections={connections} segmented={}", connections > 1));
                    update_task(&app, id, true, |task| {
                        task.state = DownloadState::Downloading;
                        task.received = resumed_at;
                        task.total = total;
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

fn epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |value| value.as_secs())
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
    update_task(&app, id, true, |item| {
        item.state = DownloadState::Downloading;
        item.progress_percent = Some(0.0);
    });
    let directory = task.destination.parent().unwrap_or_else(|| Path::new("."));
    let file_name = task
        .destination
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("download");
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
                configured_ed2k_tool(&settings),
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
                "ed2k".into(),
                8,
                None,
                None,
                None,
                Vec::new(),
            )
        });
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
    let user_agent = configured_user_agent.as_deref()
        .or_else(|| identity.as_ref().and_then(|value| value.user_agent.as_deref()))
        .unwrap_or("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/152.0.0.0 Safari/537.36");
    let proxy_url = tools
        .6
        .as_deref()
        .map(|url| external_proxy_url(url, tools.7.as_deref(), tools.8.as_deref()));
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
            command
                .arg("--concurrent-fragments")
                .arg(tools.5.to_string());
            let quickjs_name = if cfg!(windows) { "qjs.exe" } else { "qjs" };
            let adjacent_quickjs = tools
                .1
                .parent()
                .map(|directory| directory.join(quickjs_name))
                .filter(|path| path.is_file());
            if let Some(quickjs) = adjacent_quickjs {
                command
                    .arg("--js-runtimes")
                    .arg(format!("quickjs:{}", quickjs.display()));
            } else {
                command.args(["--js-runtimes", "quickjs"]);
            }
            if let Some(cookie) = identity
                .as_ref()
                .and_then(|value| value.cookie_header.as_deref())
                .filter(|value| !value.is_empty())
            {
                command.arg("--add-headers").arg(format!("Cookie:{cookie}"));
            } else if task.source.contains("youtube.com/") || task.source.contains("youtu.be/") {
                command.args(["--cookies-from-browser", "chrome"]);
            }
            if task.source.contains("youtube.com/") || task.source.contains("youtu.be/") {
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
                .arg(directory)
                .arg("-o")
                .arg(file_name)
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
                command.args(["-y", "-i"]).arg(&task.source).arg("-vn");
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
                    .arg(tools.5.to_string())
                    .arg("--ffmpeg-binary-path")
                    .arg(&tools.0);
                if let Some(referer) = task.referer.as_deref() {
                    command.arg("-H").arg(format!("Referer: {referer}"));
                    if let Some(origin) = http_origin(referer) {
                        command.arg("-H").arg(format!("Origin: {origin}"));
                    }
                }
                command.arg("-H").arg(format!("User-Agent: {user_agent}"));
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
        DownloadKind::Torrent | DownloadKind::Magnet | DownloadKind::Ftp => {
            let mut command = tokio::process::Command::new(&tools.3);
            if !tools.9.is_empty() {
                command.arg(format!(
                    "--async-dns-server={}",
                    tools
                        .9
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                ));
            }
            if let Some(proxy_url) = tools.6.as_deref() {
                command.arg(format!("--all-proxy={proxy_url}"));
                if let Some(username) = tools.7.as_deref() {
                    command.arg(format!("--all-proxy-user={username}"));
                }
                if let Some(password) = tools.8.as_deref() {
                    command.arg(format!("--all-proxy-passwd={password}"));
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
                "--bt-prioritize-piece=head=32M,tail=32M",
                "--file-allocation=trunc",
                "--seed-time=0",
            ]);
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
        DownloadKind::Ed2k => {
            let settings = app
                .state::<AppState>()
                .settings
                .lock()
                .map(|value| value.clone())
                .unwrap_or_default();
            if settings.ed2k_password.is_empty() {
                update_task(&app, id, true, |item| {
                    item.state = DownloadState::Failed {
                        message: "ed2k_password_required".to_owned(),
                    }
                });
                return;
            }
            let mut command = tokio::process::Command::new(amule_controller(&settings));
            command.args([
                "-h",
                &settings.ed2k_host,
                "-p",
                &settings.ed2k_port.to_string(),
                "-P",
                &settings.ed2k_password,
                "-l",
                "en",
                "--command",
                &format!("add {}", task.source),
            ]);
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

fn save_site_rules(state: &AppState, rules: &[SiteRule]) -> Result<(), String> {
    if let Some(parent) = state.site_rules_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    if state.site_rules_path.exists() {
        fs::copy(
            &state.site_rules_path,
            state.site_rules_path.with_extension("backup.json"),
        )
        .map_err(|error| error.to_string())?;
    }
    let data = serde_json::to_vec_pretty(rules).map_err(|error| error.to_string())?;
    fs::write(&state.site_rules_path, data).map_err(|error| error.to_string())
}

#[tauri::command]
fn get_site_rules(state: State<'_, AppState>) -> Result<String, String> {
    let rules = state.site_rules.lock().map_err(|error| error.to_string())?;
    serde_json::to_string_pretty(&*rules).map_err(|error| error.to_string())
}

#[tauri::command]
fn set_site_rules(state: State<'_, AppState>, json: String) -> Result<String, String> {
    let rules = serde_json::from_str::<Vec<SiteRule>>(&json)
        .map_err(|error| format!("invalid_site_rules_json: {error}"))?;
    if rules.is_empty() || rules.len() > 100 || !rules.iter().all(valid_site_rule) {
        return Err("invalid_site_rules".to_owned());
    }
    let mut ids = rules
        .iter()
        .map(|rule| rule.id.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();
    ids.sort();
    if ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("duplicate_site_rule_id".to_owned());
    }
    save_site_rules(&state, &rules)?;
    *state.site_rules.lock().map_err(|error| error.to_string())? = rules;
    diagnostic_log(
        &state,
        "INFO",
        "site_rules.updated",
        &format!("count={}", ids.len()),
    );
    get_site_rules(state)
}

#[tauri::command]
fn reset_site_rules(state: State<'_, AppState>) -> Result<String, String> {
    let rules = default_site_rules();
    save_site_rules(&state, &rules)?;
    *state.site_rules.lock().map_err(|error| error.to_string())? = rules;
    diagnostic_log(&state, "INFO", "site_rules.reset", "defaults_restored");
    get_site_rules(state)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MatrixRuleProposal {
    host: String,
    failures: usize,
    confidence: u8,
    reason: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MatrixAppliedRule {
    id: String,
    name: String,
    host: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MatrixStatus {
    version: String,
    active_rules: usize,
    proposals: Vec<MatrixRuleProposal>,
    applied_rules: Vec<MatrixAppliedRule>,
}

#[tauri::command]
fn matrix_analyze(state: State<'_, AppState>) -> Result<MatrixStatus, String> {
    let queue = state.queue.lock().map_err(|error| error.to_string())?;
    let rules = state.site_rules.lock().map_err(|error| error.to_string())?;
    let mut failures = HashMap::<String, HashSet<String>>::new();
    for task in queue
        .iter()
        .filter(|task| matches!(&task.state, DownloadState::Failed { .. }))
    {
        if let Ok(url) = url::Url::parse(&task.source) {
            if let Some(host) = url.host_str() {
                failures
                    .entry(host.to_ascii_lowercase())
                    .or_default()
                    .insert(task.id.to_string());
            }
        }
    }

    let mut log = fs::read_to_string(state.log_path.with_extension("log.1")).unwrap_or_default();
    log.push_str(&fs::read_to_string(&state.log_path).unwrap_or_default());
    let mut task_hosts = HashMap::<String, String>::new();
    for line in log.lines() {
        if !line.contains("task.enqueued") {
            continue;
        }
        let task = line
            .split_whitespace()
            .find_map(|field| field.strip_prefix("task="));
        let source = line
            .split_whitespace()
            .find_map(|field| field.strip_prefix("url="));
        let identified = task
            .zip(source)
            .and_then(|(task, source)| host_from_url(source).map(|host| (task, host)));
        if let Some((task, host)) = identified {
            task_hosts.insert(task.to_owned(), host);
        }
    }
    for line in log.lines().filter(|line| line.contains("http.failed")) {
        let task = line
            .split_whitespace()
            .find_map(|field| field.strip_prefix("task="));
        let identified = task.and_then(|task| task_hosts.get(task).map(|host| (task, host)));
        if let Some((task, host)) = identified {
            failures
                .entry(host.clone())
                .or_default()
                .insert(task.to_owned());
        }
    }

    let mut proposals = failures
        .into_iter()
        .filter(|(host, _)| matching_site_rule(&format!("https://{host}/"), &rules).is_none())
        .map(|(host, task_ids)| {
            let count = task_ids.len();
            MatrixRuleProposal {
                confidence: (55 + count.saturating_sub(1).min(4) * 10) as u8,
                reason: format!(
                    "{count} failed download(s); retry with one conservative connection"
                ),
                host,
                failures: count,
            }
        })
        .collect::<Vec<_>>();
    proposals.sort_by(|left, right| {
        right
            .failures
            .cmp(&left.failures)
            .then_with(|| left.host.cmp(&right.host))
    });
    let applied_rules = rules
        .iter()
        .filter(|rule| rule.enabled && rule.action != SiteRuleAction::Standard)
        .map(|rule| MatrixAppliedRule {
            id: rule.id.clone(),
            name: rule.name.clone(),
            host: rule.hosts.first().cloned().unwrap_or_default(),
        })
        .collect();
    Ok(MatrixStatus {
        version: "Matrix Ultimate v2 AI".to_owned(),
        active_rules: rules.iter().filter(|rule| rule.enabled).count(),
        proposals,
        applied_rules,
    })
}

#[tauri::command]
fn matrix_apply_rule(state: State<'_, AppState>, host: String) -> Result<String, String> {
    let host = host.trim().to_ascii_lowercase();
    if host.is_empty()
        || host.len() > 253
        || !host
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-'))
    {
        return Err("invalid_matrix_host".to_owned());
    }
    let mut rules = state
        .site_rules
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    if let Some(rule) = rules.iter_mut().find(|rule| {
        rule.hosts
            .iter()
            .any(|candidate| host_matches_pattern(&host, candidate))
    }) {
        rule.enabled = true;
        rule.action = SiteRuleAction::SingleConnection;
        rule.connections = 1;
    } else {
        let id_host = host.replace('.', "-");
        rules.push(SiteRule {
            id: format!("matrix-{id_host}"),
            name: format!("Matrix: {host}"),
            hosts: vec![host.clone(), format!("*.{host}")],
            action: SiteRuleAction::SingleConnection,
            enabled: true,
            connections: 1,
        });
    }
    if rules.len() > 100 || !rules.iter().all(valid_site_rule) {
        return Err("invalid_site_rules".to_owned());
    }
    save_site_rules(&state, &rules)?;
    *state.site_rules.lock().map_err(|error| error.to_string())? = rules;
    diagnostic_log(
        &state,
        "INFO",
        "matrix.rule_applied",
        &format!("host={host} action=single_connection"),
    );
    Ok(host)
}

#[tauri::command]
fn matrix_rollback_rule(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let mut rules = state
        .site_rules
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    let rule = rules
        .iter_mut()
        .find(|rule| rule.id == id && rule.action != SiteRuleAction::Standard)
        .ok_or_else(|| "matrix_rule_not_found".to_owned())?;
    rule.enabled = false;
    save_site_rules(&state, &rules)?;
    *state.site_rules.lock().map_err(|error| error.to_string())? = rules;
    diagnostic_log(
        &state,
        "INFO",
        "matrix.rule_rolled_back",
        &format!("id={id}"),
    );
    Ok(id)
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
                        DownloadKind::Torrent | DownloadKind::Magnet | DownloadKind::Ftp
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
    let ed2k_name = parse_ed2k_file_link(source).map(|(name, _)| name);
    let magnet_name = (classify_url(source) == Some(DownloadKind::Magnet))
        .then(|| {
            url::Url::parse(source)
                .ok()?
                .query_pairs()
                .find(|(key, _)| key == "dn")
                .map(|(_, value)| value.into_owned())
        })
        .flatten();
    ed2k_name
        .as_deref()
        .or(magnet_name.as_deref())
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

fn parse_ed2k_file_link(source: &str) -> Option<(String, u64)> {
    let fields = source.split('|').collect::<Vec<_>>();
    if fields.len() < 6 || fields[0] != "ed2k://" || !fields[1].eq_ignore_ascii_case("file") {
        return None;
    }
    let mut decoded = Vec::with_capacity(fields[2].len());
    let bytes = fields[2].as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(value) = u8::from_str_radix(&fields[2][index + 1..index + 3], 16) {
                decoded.push(value);
                index += 3;
                continue;
            }
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    let name = String::from_utf8_lossy(&decoded).replace(['/', '\\'], "_");
    let size = fields[3].parse::<u64>().ok()?;
    (!name.trim().is_empty() && size > 0).then_some((name, size))
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
    Ok(name.to_owned())
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
    extension
        .map(|extension| format!("{file_name}.{extension}"))
        .unwrap_or(file_name)
}

fn unique_destination(directory: &Path, file_name: &str) -> PathBuf {
    let original = directory.join(file_name);
    if !original.exists() && !partial_path(&original).exists() {
        return original;
    }
    let path = Path::new(file_name);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("download");
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
        (
            "ed2k",
            configured_ed2k_tool(&settings),
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

fn uupdump_urls(url: &str) -> Option<(String, String)> {
    let lower = url.to_ascii_lowercase();
    let prefixes = [
        "https://uupdump.net/",
        "https://www.uupdump.net/",
        "http://uupdump.net/",
        "http://www.uupdump.net/",
    ];
    let prefix = prefixes
        .into_iter()
        .find(|prefix| lower.starts_with(prefix))?;
    let remainder = &url[prefix.len()..];
    let (path, query) = remainder.split_once('?').unwrap_or((remainder, ""));
    if !matches!(
        path.to_ascii_lowercase().as_str(),
        "download.php" | "get.php"
    ) {
        return None;
    }
    let suffix = if query.is_empty() {
        String::new()
    } else {
        format!("?{query}")
    };
    Some((
        format!("https://uupdump.net/get.php{suffix}"),
        format!("https://uupdump.net/download.php{suffix}"),
    ))
}

#[tauri::command]
fn set_tool_paths(
    state: State<'_, AppState>,
    ffmpeg: String,
    yt_dlp: String,
    n_m3u8dl_re: String,
    aria2: String,
    ed2k: String,
) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.ffmpeg_path = optional_path(ffmpeg);
    settings.yt_dlp_path = optional_path(yt_dlp);
    settings.n_m3u8dl_re_path = optional_path(n_m3u8dl_re);
    settings.aria2_path = optional_path(aria2);
    settings.ed2k_path = optional_path(ed2k);
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
    let mut command = Command::new(
        player.unwrap_or_else(|| PathBuf::from(if cfg!(windows) { "vlc.exe" } else { "vlc" })),
    );
    command.arg(video);
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

#[tauri::command]
fn update_tool(state: State<'_, AppState>, id: String) -> Result<String, String> {
    if id == "ed2k" {
        let settings = state
            .settings
            .lock()
            .map_err(|error| error.to_string())?
            .clone();
        let executable = configured_ed2k_tool(&settings);
        let directory = executable
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let updater = ["amule-updater.exe", "amule-updater", "updater.exe"]
            .into_iter()
            .map(|name| directory.join(name))
            .find(|path| path.is_file());
        if let Some(updater) = updater {
            Command::new(updater)
                .spawn()
                .map_err(|error| error.to_string())?;
            return Ok("aMule updater started; the configured ed2k helper will be refreshed with its matching package".to_owned());
        }
        open_external_url("https://github.com/amule-org/amule/releases/latest")?;
        return Ok("Official aMule release opened. Replace the complete portable package so ed2k and its matching libraries stay compatible".to_owned());
    }
    if id != "yt-dlp" {
        return Err("manual_update_required: this engine has no safe in-place updater".to_owned());
    }
    let executable = {
        let settings = state.settings.lock().map_err(|error| error.to_string())?;
        configured_tool(
            &settings.yt_dlp_path,
            if cfg!(windows) {
                "yt-dlp.exe"
            } else {
                "yt-dlp"
            },
        )
    };
    if executable.is_dir() {
        return Err("tool_target_must_be_a_file".to_owned());
    }
    let before =
        version_line(&executable, &["--version"]).ok_or_else(|| "yt_dlp_not_found".to_owned())?;
    let mut command = Command::new(&executable);
    command.args(["--update-to", "stable"]);
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
    Ok(if before == after {
        format!("yt-dlp already current ({after})")
    } else {
        format!("yt-dlp updated: {before} → {after}")
    })
}

fn open_external_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let result = Command::new("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn();
    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(url).spawn();
    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open").arg(url).spawn();
    result.map(|_| ()).map_err(|error| error.to_string())
}

#[tauri::command]
fn open_amule(state: State<'_, AppState>) -> Result<(), String> {
    let settings = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    let helper = configured_ed2k_tool(&settings);
    let directory = helper
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let candidates = if cfg!(windows) {
        vec![directory.join("amule.exe"), directory.join("aMule.exe")]
    } else {
        vec![
            directory.join("amule"),
            directory.join("aMule"),
            PathBuf::from("amule"),
        ]
    };
    let executable = candidates
        .into_iter()
        .find(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from(if cfg!(windows) { "amule.exe" } else { "amule" }));
    Command::new(executable)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("amule_not_found: {error}"))
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
    context: Option<DownloadContext>,
) -> Result<DownloadTask, String> {
    if let Some(existing) = state
        .queue
        .lock()
        .map_err(|error| error.to_string())?
        .iter()
        .find(|task| {
            task.source == url
                && !matches!(
                    task.state,
                    DownloadState::Completed | DownloadState::Failed { .. }
                )
        })
    {
        return Err(format!("duplicate_active_download:{}", existing.id));
    }
    let site_rule = {
        let rules = state.site_rules.lock().map_err(|error| error.to_string())?;
        matching_site_rule(&url, &rules)
    };
    let uupdump = site_rule
        .as_ref()
        .filter(|rule| rule.action == SiteRuleAction::UupdumpPost)
        .and_then(|_| uupdump_urls(&url));
    let url = uupdump.as_ref().map(|item| item.0.clone()).unwrap_or(url);
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
    if let Some((_, size)) = parse_ed2k_file_link(&url) {
        task.total = Some(size);
    }
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
    if let Some(context) = context {
        task.referer = context
            .referer
            .filter(|url| url.starts_with("https://") || url.starts_with("http://"));
        task.known_duration = context
            .known_duration
            .filter(|duration| duration.is_finite() && *duration > 0.0);
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
            state
                .request_identities
                .lock()
                .map_err(|error| error.to_string())?
                .insert(
                    task.id,
                    RequestIdentity {
                        cookie_header,
                        user_agent,
                        request_method,
                        request_body,
                        request_content_type,
                    },
                );
        }
    }
    if let Some((_, page_url)) = uupdump {
        task.referer = Some(page_url);
        let mut identities = state
            .request_identities
            .lock()
            .map_err(|error| error.to_string())?;
        let identity = identities
            .entry(task.id)
            .or_insert_with(|| RequestIdentity {
                cookie_header: None,
                user_agent: None,
                request_method: "POST".to_owned(),
                request_body: None,
                request_content_type: None,
            });
        identity.request_method = "POST".to_owned();
        identity.request_body = Some("autodl=2&updates=1".to_owned());
        identity.request_content_type = Some("application/x-www-form-urlencoded".to_owned());
    }
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    queue.push(task.clone());
    save_queue(&state, &queue)?;
    drop(queue);
    diagnostic_log(
        &state,
        "INFO",
        "task.enqueued",
        &format!(
            "task={} engine={kind:?} rule={} url={} file={}",
            task.id,
            site_rule.as_ref().map_or("none", |rule| rule.id.as_str()),
            redact_url(&task.source),
            task.destination.display()
        ),
    );
    start_download(&app, &state, task.clone(), kind)?;
    Ok(task)
}

fn start_download(
    app: &tauri::AppHandle,
    state: &AppState,
    task: DownloadTask,
    kind: DownloadKind,
) -> Result<(), String> {
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
    diagnostic_log(
        state,
        "INFO",
        "task.dispatched",
        &format!("task={} engine={kind:?}", task.id),
    );
    if kind == DownloadKind::Http {
        let identity = state
            .request_identities
            .lock()
            .ok()
            .and_then(|items| items.get(&task.id).cloned());
        let rule = state
            .site_rules
            .lock()
            .ok()
            .and_then(|rules| matching_site_rule(&task.source, &rules));
        let configured_connections =
            if limits.adaptive_efficiency && limits.max_active_downloads <= 3 && task.priority >= 0
            {
                limits.connections_per_download.max(16)
            } else {
                limits.connections_per_download
            };
        let connections = rule
            .as_ref()
            .filter(|rule| {
                matches!(
                    rule.action,
                    SiteRuleAction::SingleConnection | SiteRuleAction::UupdumpPost
                )
            })
            .map_or_else(
                || configured_connections.clamp(1, 32),
                |rule| rule.connections,
            );
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
    save_queue(&state, &queue)?;
    drop(queue);
    diagnostic_log(&state, "INFO", "task.paused", &format!("task={id}"));
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
        let mut task =
            DownloadTask::new(&original.source, unique_destination(directory, file_name));
        task.format_selection = original.format_selection.clone();
        task.referer = original.referer.clone();
        task.known_duration = original.known_duration;
        if let Some(identity) = saved_identities.get(&original.id) {
            repeated_identities.push((task.id, identity.clone()));
        }
        repeated.push(task);
    }
    {
        let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
        queue.extend(repeated.iter().cloned());
        save_queue(&state, &queue)?;
    }
    if !repeated_identities.is_empty() {
        state
            .request_identities
            .lock()
            .map_err(|error| error.to_string())?
            .extend(repeated_identities);
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
    })
}

#[tauri::command]
fn set_transfer_limits(
    state: State<'_, AppState>,
    max_active_downloads: usize,
    connections_per_download: usize,
    adaptive_efficiency: bool,
) -> Result<TransferLimits, String> {
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.max_active_downloads = max_active_downloads.clamp(1, 20);
    settings.connections_per_download = connections_per_download.clamp(1, 32);
    settings.adaptive_efficiency = adaptive_efficiency;
    save_settings(&state, &settings)?;
    Ok(TransferLimits {
        max_active_downloads: settings.max_active_downloads,
        connections_per_download: settings.connections_per_download,
        adaptive_efficiency: settings.adaptive_efficiency,
    })
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
    let value = app
        .clipboard()
        .read_text()
        .map_err(|error| error.to_string())?;
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

fn queue_from_bridge(app: &tauri::AppHandle, request: BridgeDownload) -> Result<(), String> {
    let state = app.state::<AppState>();
    classify_url(&request.url).ok_or_else(|| "unsupported_url".to_owned())?;
    diagnostic_log(
        &state,
        "INFO",
        "bridge.download",
        &format!(
            "url={} method={}",
            redact_url(&request.url),
            request.request_method.as_deref().unwrap_or("GET")
        ),
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
    Ok(())
}

fn association_id(source: &str) -> Option<&'static str> {
    let lower = source.to_ascii_lowercase();
    if lower.starts_with("magnet:") {
        Some("magnet")
    } else if lower.starts_with("ed2k:") {
        Some("ed2k")
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
            url: source,
            file_name: None,
            page_url: None,
            duration: None,
            cookie_header: None,
            user_agent: None,
            request_method: None,
            request_body: None,
            request_content_type: None,
        },
    )
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
    let destination = unique_destination(&directory, &file_name);
    let partial = partial_path(&destination);
    fs::File::create(&partial).map_err(|error| error.to_string())?;
    let mut task = DownloadTask::new(request.source, destination.clone());
    task.state = DownloadState::Downloading;
    task.total = (!request.streaming).then_some(request.total);
    let task_id = task.id;
    {
        let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
        queue.push(task);
        save_queue(&state, &queue)?;
    }
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
    let (task_id, received, total) = {
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
        (upload.task_id, upload.received, upload.total)
    };
    update_task(app, task_id, false, |task| {
        task.received = received;
        task.progress_percent = total.map(|total| received as f64 * 100.0 / total as f64);
    });
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
    let _ = app.emit("recording-completed", upload.task_id);
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
        bridge_response(&mut stream, "401 Unauthorized", origin, "{\"ok\":false}");
        return;
    }
    if let Ok(mut seen) = state.bridge_last_seen.lock() {
        *seen = Some(Instant::now());
    }
    if first.starts_with("GET /v1/health ") {
        bridge_response(&mut stream, "200 OK", origin, "{\"ok\":true}");
    } else if first.starts_with("GET /v1/site-rules ") {
        let body = state
            .site_rules
            .lock()
            .ok()
            .and_then(|rules| serde_json::to_string(&*rules).ok())
            .unwrap_or_else(|| "[]".to_owned());
        bridge_response(&mut stream, "200 OK", origin, &body);
    } else if first.starts_with("POST /v1/download ") {
        match serde_json::from_str::<BridgeDownload>(body)
            .map_err(|error| error.to_string())
            .and_then(|request| queue_from_bridge(app, request))
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

fn forward_to_running_instance(source: &str, token: &str) -> bool {
    let Ok(mut stream) = TcpStream::connect(("127.0.0.1", BRIDGE_PORT)) else {
        return false;
    };
    let request = BridgeDownload {
        url: source.to_owned(),
        file_name: None,
        page_url: None,
        duration: None,
        cookie_header: None,
        user_agent: None,
        request_method: None,
        request_body: None,
        request_content_type: None,
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
            .arg(format!("/select,{}", target.display()))
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
    let output = unique_destination(&output_directory, &format!("{stem}.{format}"));
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

const ASSOCIATION_IDS: [&str; 6] = ["m3u8", "torrent", "magnet", "ftp", "sftp", "ed2k"];

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
        ("ed2k", "x-scheme-handler/ed2k"),
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
            let site_rules_path = app_data.join("site-rules.json");
            let initial_site_rules = load_site_rules(&site_rules_path);
            if !site_rules_path.exists() {
                let data = serde_json::to_vec_pretty(&initial_site_rules)?;
                fs::write(&site_rules_path, data)?;
            }
            let initial_settings = load_settings(&settings_path);
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
                    if associated_source.as_deref().is_some_and(|source| {
                        forward_to_running_instance(source, &initial_settings.bridge_token)
                    }) {
                        app.handle().exit(0);
                        return Ok(());
                    }
                    None
                }
            };
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
                site_rules: Mutex::new(initial_site_rules),
                site_rules_path,
                ed2k_search: Mutex::new(None),
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
            if let Ok(listener) = TcpListener::bind(("0.0.0.0", LINK_PORT)) {
                let link_app = app.handle().clone();
                std::thread::Builder::new()
                    .name("apocalipse-link-server".into())
                    .spawn(move || run_link_server(link_app, listener))?;
            }
            if let Some(source) = associated_source {
                queue_associated_source(app.handle(), source).map_err(std::io::Error::other)?;
            }
            let show = MenuItem::with_id(app, "show", "Show Apocalipse", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
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
            get_ed2k_engine_status,
            get_ed2k_connection,
            set_ed2k_connection,
            start_ed2k_engine,
            connect_ed2k_networks,
            ed2k_network_status,
            ed2k_search,
            ed2k_search_results,
            ed2k_download_result,
            list_ed2k_transfers,
            control_ed2k_transfer,
            set_tool_paths,
            get_media_player,
            set_media_player,
            open_amule,
            preview_torrent,
            update_tool,
            suggest_download_name,
            remove_downloads,
            read_general_log,
            clear_general_log,
            get_log_editor,
            set_log_editor,
            open_log_external,
            get_site_rules,
            set_site_rules,
            reset_site_rules,
            matrix_analyze,
            matrix_apply_rule,
            matrix_rollback_rule,
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
            get_user_agent,
            set_user_agent,
            get_proxy_setting,
            set_proxy_setting,
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
    use super::*;

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
    fn normalizes_uupdump_download_to_required_post_endpoint() {
        let (download, referer) =
            uupdump_urls("https://uupdump.net/download.php?id=abc&pack=pt-br&edition=professional")
                .expect("UUP dump URL");
        assert_eq!(
            download,
            "https://uupdump.net/get.php?id=abc&pack=pt-br&edition=professional"
        );
        assert_eq!(
            referer,
            "https://uupdump.net/download.php?id=abc&pack=pt-br&edition=professional"
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
    fn site_rules_match_exact_hosts_and_subdomains() {
        let rules = default_site_rules();
        assert_eq!(
            matching_site_rule("https://uupdump.net/get.php?id=1", &rules)
                .unwrap()
                .id,
            "uupdump"
        );
        assert_eq!(
            matching_site_rule("https://www.uupdump.net/download.php", &rules)
                .unwrap()
                .id,
            "uupdump"
        );
        assert!(matching_site_rule("https://example.com/uupdump.net/file", &rules).is_none());
        assert_eq!(
            matching_site_rule("https://s14.rapidgator.net/download/token", &rules)
                .unwrap()
                .id,
            "rapidgator"
        );
        assert_eq!(
            matching_site_rule("https://pixeldrain.com/api/file/FhcC8Fyd?download", &rules)
                .unwrap()
                .id,
            "pixeldrain"
        );
    }

    #[test]
    fn site_rule_validation_rejects_unsafe_hosts_and_connections() {
        let mut rule = default_site_rules().remove(0);
        rule.hosts = vec!["https://uupdump.net/path".to_owned()];
        assert!(!valid_site_rule(&rule));
        rule.hosts = vec!["uupdump.net".to_owned()];
        rule.connections = 0;
        assert!(!valid_site_rule(&rule));
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
