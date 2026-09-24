#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod aria2;
mod diagnostics;
mod prepared_preview;
mod thumbnail_cache;
mod tiktok_preview;

use apocalipse_core::{
    chunk_directory, classify_url, cleanup_chunk_artifacts, contextual_media_page, parse_metalink,
    partial_path, plan_download, BandwidthLimiter, Capabilities, DownloadEngine, DownloadEvent,
    DownloadId, DownloadKind, DownloadRequest, DownloadState, DownloadTask,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rustls::{
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime},
    ClientConfig, ClientConnection, DigitallySignedStruct, ServerConfig, ServerConnection,
    SignatureScheme, StreamOwned,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs,
    fs::OpenOptions,
    io::{Read, Write},
    net::{IpAddr, TcpListener, TcpStream, UdpSocket},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tokio::{
    io::AsyncReadExt,
    sync::{mpsc, oneshot},
};
use zeroize::Zeroize;

#[cfg(windows)]
use std::{
    ffi::{c_void, OsStr},
    os::windows::ffi::OsStrExt,
};

#[cfg(unix)]
use std::{
    ffi::c_void,
    os::raw::{c_char, c_int},
};

#[cfg(windows)]
#[link(name = "advapi32")]
extern "system" {
    fn LogonUserW(
        username: *const u16,
        domain: *const u16,
        password: *const u16,
        logon_type: u32,
        logon_provider: u32,
        token: *mut *mut c_void,
    ) -> i32;
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn CloseHandle(object: *mut c_void) -> i32;
}

#[cfg(windows)]
#[repr(C)]
struct WindowsShareInfo2 {
    netname: *mut u16,
    share_type: u32,
    remark: *mut u16,
    permissions: u32,
    max_uses: u32,
    current_uses: u32,
    path: *mut u16,
    password: *mut u16,
}

#[cfg(windows)]
#[link(name = "netapi32")]
extern "system" {
    fn NetShareEnum(
        server_name: *const u16,
        level: u32,
        buffer: *mut *mut u8,
        preferred_maximum_length: u32,
        entries_read: *mut u32,
        total_entries: *mut u32,
        resume_handle: *mut u32,
    ) -> u32;
    fn NetApiBufferFree(buffer: *mut c_void) -> u32;
}

#[cfg(unix)]
#[repr(C)]
struct PamMessage {
    msg_style: c_int,
    msg: *const c_char,
}

#[cfg(unix)]
#[repr(C)]
struct PamResponse {
    resp: *mut c_char,
    resp_retcode: c_int,
}

#[cfg(unix)]
#[repr(C)]
struct PamConversation {
    conv: Option<
        unsafe extern "C" fn(
            c_int,
            *const *const PamMessage,
            *mut *mut PamResponse,
            *mut c_void,
        ) -> c_int,
    >,
    appdata_ptr: *mut c_void,
}

#[cfg(unix)]
struct PamConversationData {
    username: Vec<u8>,
    password: Vec<u8>,
}

#[cfg(unix)]
#[link(name = "pam")]
extern "C" {
    fn pam_start(
        service_name: *const c_char,
        user: *const c_char,
        pam_conversation: *const PamConversation,
        pam_handle: *mut *mut c_void,
    ) -> c_int;
    fn pam_authenticate(pam_handle: *mut c_void, flags: c_int) -> c_int;
    fn pam_acct_mgmt(pam_handle: *mut c_void, flags: c_int) -> c_int;
    fn pam_end(pam_handle: *mut c_void, status: c_int) -> c_int;
}

#[cfg(unix)]
extern "C" {
    fn calloc(count: usize, size: usize) -> *mut c_void;
    fn free(pointer: *mut c_void);
    fn strdup(value: *const c_char) -> *mut c_char;
}

const VAULT_SERVICE: &str = "com.linuxhell.apocalipse";
const VAULT_BRIDGE_TOKEN: &str = "extension-bridge-token";
const VAULT_LINK_PASSWORD: &str = "apocalipse-link-password";
const VAULT_PROXY_PASSWORD: &str = "proxy-password";

fn vault_entry(account: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new(VAULT_SERVICE, account).map_err(|error| error.to_string())
}

fn vault_load(account: &str) -> Result<Option<String>, String> {
    match vault_entry(account)?.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

fn vault_store_verified(account: &str, secret: &str) -> Result<(), String> {
    let entry = vault_entry(account)?;
    entry
        .set_password(secret)
        .map_err(|error| error.to_string())?;
    let mut recovered = entry.get_password().map_err(|error| error.to_string())?;
    let matches = recovered == secret;
    recovered.zeroize();
    if !matches {
        return Err("credential_vault_verification_failed".to_owned());
    }
    Ok(())
}

fn vault_delete(account: &str) -> Result<(), String> {
    match vault_entry(account)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn website_vault_account(host: &str) -> String {
    format!("website:{host}")
}

fn host_rule_vault_account(pattern: &str) -> String {
    format!("host-rule:{pattern}")
}

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
    bridge_recent_prompts: Mutex<HashMap<String, Instant>>,
    browser_assisted_pending: Mutex<Vec<BrowserDownloadComplete>>,
    blob_uploads: Mutex<HashMap<uuid::Uuid, BlobUpload>>,
    recording_stops: Mutex<HashSet<DownloadId>>,
    request_identities: Mutex<HashMap<DownloadId, RequestIdentity>>,
    link_transfers: Mutex<HashMap<String, Arc<LinkTransferControl>>>,
    aria2_runtime: Mutex<Option<aria2::Runtime>>,
    aria2_tasks: Mutex<HashMap<DownloadId, String>>,
    log_path: PathBuf,
    log_write_lock: Mutex<()>,
    diagnostics: diagnostics::Diagnostics,
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
    } else if host.as_deref().is_some_and(|value| {
        value == "download.microsoft.com"
            || value.ends_with(".download.microsoft.com")
            || value == "software.download.prss.microsoft.com"
            || value.ends_with(".download.prss.microsoft.com")
    }) {
        // Microsoft's ISO CDN supports byte ranges. Start wide enough to avoid
        // the slow single-stream ramp while keeping the setting below the
        // application's global safety ceiling.
        Some(requested.unwrap_or(16).max(16).clamp(1, 32))
    } else {
        requested.map(|value| value.clamp(1, 32))
    }
}

fn requires_native_http_compatibility(url: &str) -> bool {
    host_from_url(url).as_deref().is_some_and(|value| {
        value == "oaiusercontent.com" || value.ends_with(".oaiusercontent.com")
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
    #[serde(default)]
    http_global_capacity: HttpCapacityEstimate,
    #[serde(default)]
    http_host_capacities: HashMap<String, HttpCapacityEstimate>,
    #[serde(default)]
    global_bandwidth_limit: u64,
    #[serde(default = "default_bridge_token", skip_serializing)]
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
    extractor_path: Option<PathBuf>,
    #[serde(default = "default_true")]
    aria2_rpc_enabled: bool,
    #[serde(default = "default_true")]
    aria2_rpc_auto_start: bool,
    #[serde(default)]
    aria2_rpc_port: Option<u16>,
    #[serde(default = "default_aria2_rpc_secret", skip_serializing)]
    aria2_rpc_secret: String,
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
    #[serde(default, skip_serializing)]
    proxy_password: Option<String>,
    #[serde(default, rename = "website_credentials", skip_serializing)]
    legacy_website_credentials: Vec<WebsiteCredential>,
    #[serde(default)]
    host_rules: Vec<HostRule>,
    #[serde(default)]
    dns_enabled: bool,
    #[serde(default)]
    dns_servers: Vec<std::net::IpAddr>,
    #[serde(default)]
    associations: HashMap<String, bool>,
    #[serde(default = "default_link_password", skip_serializing)]
    link_password: String,
    #[serde(default)]
    link_shares: Vec<LinkShare>,
    #[serde(default)]
    link_trusted_certificates: HashMap<String, String>,
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
    16
}
const fn default_true() -> bool {
    true
}
fn default_aria2_rpc_secret() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}
fn default_bridge_token() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}
fn default_link_password() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            download_directory: None,
            capture_clipboard: false,
            max_active_downloads: default_max_active(),
            connections_per_download: default_connections(),
            adaptive_efficiency: true,
            http_global_capacity: HttpCapacityEstimate::default(),
            http_host_capacities: HashMap::new(),
            global_bandwidth_limit: 0,
            bridge_token: default_bridge_token(),
            recent_download_directories: Vec::new(),
            ffmpeg_path: None,
            yt_dlp_path: None,
            qjs_path: None,
            n_m3u8dl_re_path: None,
            aria2_path: None,
            extractor_path: None,
            aria2_rpc_enabled: true,
            aria2_rpc_auto_start: true,
            aria2_rpc_port: None,
            aria2_rpc_secret: default_aria2_rpc_secret(),
            media_player_path: None,
            user_agent: None,
            log_editor_path: None,
            proxy_enabled: false,
            proxy_url: None,
            proxy_username: None,
            proxy_password: None,
            legacy_website_credentials: Vec::new(),
            host_rules: Vec::new(),
            dns_enabled: false,
            dns_servers: Vec::new(),
            associations: HashMap::new(),
            link_password: default_link_password(),
            link_shares: Vec::new(),
            link_trusted_certificates: HashMap::new(),
            language: default_language(),
            theme: default_theme(),
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct HttpCapacityEstimate {
    #[serde(default)]
    bytes_per_second: u64,
    #[serde(default)]
    low_runs: u32,
}

#[derive(Clone, Deserialize, Serialize)]
struct WebsiteCredential {
    host: String,
    username: String,
    #[serde(default, skip_serializing)]
    password: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct HostRule {
    pattern: String,
    #[serde(default)]
    username: Option<String>,
    #[serde(default, skip_serializing)]
    password: String,
    #[serde(default)]
    user_agent: Option<String>,
    #[serde(default)]
    connections: Option<usize>,
    #[serde(default)]
    bandwidth_limit: Option<u64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct HostRuleSummary {
    pattern: String,
    username: String,
    has_password: bool,
    user_agent: String,
    connections: Option<usize>,
    bandwidth_limit: Option<u64>,
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
    torrent_path: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkFileEntry {
    name: String,
    path: String,
    size: u64,
    directory: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkTransferProgress {
    transfer_id: String,
    direction: String,
    transferred: u64,
    total: u64,
    percent: f64,
    bytes_per_second: u64,
}

#[derive(Default)]
struct LinkTransferControl {
    paused: AtomicBool,
    cancelled: AtomicBool,
    progress: Mutex<Option<LinkTransferProgress>>,
}

struct LinkTransferReporter {
    app: tauri::AppHandle,
    transfer_id: String,
    direction: String,
    total: u64,
    transferred: u64,
    started: Instant,
    last_emit: Instant,
    last_speed_at: Instant,
    last_speed_bytes: u64,
    smoothed_speed: u64,
    last_diagnostic_at: Instant,
    control: Arc<LinkTransferControl>,
}

impl LinkTransferReporter {
    fn new(app: tauri::AppHandle, transfer_id: String, direction: &str, total: u64) -> Self {
        let control = Arc::new(LinkTransferControl {
            paused: AtomicBool::new(false),
            cancelled: AtomicBool::new(false),
            progress: Mutex::new(None),
        });
        if let Ok(mut transfers) = app.state::<AppState>().link_transfers.lock() {
            transfers.insert(transfer_id.clone(), control.clone());
        }
        let now = Instant::now();
        let mut reporter = Self {
            app,
            transfer_id,
            direction: direction.to_owned(),
            total,
            transferred: 0,
            started: now,
            last_emit: now.checked_sub(Duration::from_secs(1)).unwrap_or(now),
            last_speed_at: now,
            last_speed_bytes: 0,
            smoothed_speed: 0,
            last_diagnostic_at: now.checked_sub(Duration::from_secs(2)).unwrap_or(now),
            control,
        };
        reporter.emit(true);
        reporter
    }

    fn checkpoint(&self) -> Result<(), String> {
        while self.control.paused.load(Ordering::Acquire) {
            if self.control.cancelled.load(Ordering::Acquire) {
                return Err("cancelled".to_owned());
            }
            std::thread::sleep(Duration::from_millis(75));
        }
        if self.control.cancelled.load(Ordering::Acquire) {
            return Err("cancelled".to_owned());
        }
        Ok(())
    }

    fn set_total(&mut self, total: u64) {
        if self.total == 0 {
            self.total = total;
            self.emit(true);
        }
    }

    fn advance(&mut self, bytes: u64) {
        self.transferred = self.transferred.saturating_add(bytes);
        self.emit(self.total > 0 && self.transferred >= self.total);
    }

    fn finish(&mut self) {
        if self.total > 0 {
            self.transferred = self.total.max(self.transferred);
        }
        self.emit(true);
    }

    fn emit(&mut self, force: bool) {
        if !force && self.last_emit.elapsed() < Duration::from_millis(125) {
            return;
        }
        let now = Instant::now();
        self.last_emit = now;
        let sample_elapsed = now.duration_since(self.last_speed_at);
        if sample_elapsed >= Duration::from_millis(250) || force {
            let sample_ms = sample_elapsed.as_millis().max(1) as u64;
            let sample_bytes = self.transferred.saturating_sub(self.last_speed_bytes);
            let instantaneous = sample_bytes.saturating_mul(1000) / sample_ms;
            self.smoothed_speed = if self.smoothed_speed == 0 {
                instantaneous
            } else {
                (instantaneous as f64 * 0.65 + self.smoothed_speed as f64 * 0.35) as u64
            };
            self.last_speed_at = now;
            self.last_speed_bytes = self.transferred;
        }
        let percent = if self.total > 0 {
            (self.transferred as f64 * 100.0 / self.total as f64).clamp(0.0, 100.0)
        } else if force && self.transferred > 0 {
            100.0
        } else {
            0.0
        };
        let payload = LinkTransferProgress {
            transfer_id: self.transfer_id.clone(),
            direction: self.direction.clone(),
            transferred: self.transferred,
            total: self.total,
            percent,
            bytes_per_second: self.smoothed_speed,
        };
        if let Ok(mut progress) = self.control.progress.lock() {
            *progress = Some(payload.clone());
        }
        let _ = self.app.emit("link-transfer-progress", payload.clone());
        if force || self.last_diagnostic_at.elapsed() >= Duration::from_secs(1) {
            self.last_diagnostic_at = now;
            let state = self.app.state::<AppState>();
            diagnostic_log(
                &state,
                "INFO",
                "link.transfer_progress",
                &format!(
                    "transfer={} direction={} bytes={} total={} percent={:.2} speed_bps={}",
                    payload.transfer_id,
                    payload.direction,
                    payload.transferred,
                    payload.total,
                    payload.percent,
                    payload.bytes_per_second
                ),
            );
        }
    }
}

impl Drop for LinkTransferReporter {
    fn drop(&mut self) {
        if let Ok(mut transfers) = self.app.state::<AppState>().link_transfers.lock() {
            transfers.remove(&self.transfer_id);
        }
    }
}

#[tauri::command]
fn pause_link_transfer(
    state: State<'_, AppState>,
    transfer_id: String,
    paused: bool,
) -> Result<(), String> {
    let transfers = state
        .link_transfers
        .lock()
        .map_err(|error| error.to_string())?;
    let control = transfers
        .get(&transfer_id)
        .ok_or_else(|| "link_transfer_not_found".to_owned())?;
    control.paused.store(paused, Ordering::Release);
    Ok(())
}

#[tauri::command]
fn cancel_link_transfer(state: State<'_, AppState>, transfer_id: String) -> Result<(), String> {
    let transfers = state
        .link_transfers
        .lock()
        .map_err(|error| error.to_string())?;
    let control = transfers
        .get(&transfer_id)
        .ok_or_else(|| "link_transfer_not_found".to_owned())?;
    control.cancelled.store(true, Ordering::Release);
    control.paused.store(false, Ordering::Release);
    Ok(())
}

#[tauri::command]
fn get_link_transfer_progress(
    state: State<'_, AppState>,
    transfer_id: String,
) -> Result<Option<LinkTransferProgress>, String> {
    let transfers = state
        .link_transfers
        .lock()
        .map_err(|error| error.to_string())?;
    let Some(control) = transfers.get(&transfer_id) else {
        return Ok(None);
    };
    let progress = control
        .progress
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    Ok(progress)
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkShare {
    id: String,
    name: String,
    path: PathBuf,
    allow_write: bool,
    #[serde(default)]
    directory: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkListRequest {
    password: String,
    path: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkAuthRequest {
    username: String,
    password: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkAuthResponse {
    token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RemoteLinkAuthentication {
    token: String,
    fingerprint: String,
    first_trust: bool,
}

#[derive(Deserialize)]
struct MobileAddRequest {
    url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkIdentity {
    id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AboutMedia {
    background_data_url: Option<String>,
    photo_data_url: Option<String>,
    audio_data_url: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkCapabilities {
    allow_write: bool,
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

const ABOUT_CREATOR_JPEG: &[u8] = include_bytes!("../assets/about-creator.jpg");
const ABOUT_BACKGROUND_PNG: &[u8] = include_bytes!("../assets/about-background.png");
const ABOUT_THEME_MP4: &[u8] = include_bytes!("../assets/about-theme.mp4");

fn about_data_url(bytes: &[u8], mime: &str) -> String {
    format!("data:{mime};base64,{}", BASE64.encode(bytes))
}

fn about_media_snapshot() -> AboutMedia {
    AboutMedia {
        background_data_url: Some(about_data_url(ABOUT_BACKGROUND_PNG, "image/png")),
        photo_data_url: Some(about_data_url(ABOUT_CREATOR_JPEG, "image/jpeg")),
        audio_data_url: Some(about_data_url(ABOUT_THEME_MP4, "audio/mp4")),
    }
}

#[tauri::command]
fn get_about_media() -> AboutMedia {
    about_media_snapshot()
}

#[tauri::command]
fn get_about_background() -> String {
    about_data_url(ABOUT_BACKGROUND_PNG, "image/png")
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
            let file_type = entry.file_type().ok()?;
            if file_type.is_symlink() {
                return None;
            }
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
fn get_link_identity() -> LinkIdentity {
    let ip = local_link_ip();
    LinkIdentity {
        id: format!("{ip}:{LINK_PORT}"),
    }
}

#[tauri::command]
fn is_local_link_target(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    let (host, _, _) = link_remote_parts(&id)?;
    if matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1")
        || host
            .parse::<IpAddr>()
            .ok()
            .is_some_and(|address| address.is_loopback())
        || host.parse::<IpAddr>().ok() == Some(local_link_ip())
    {
        return Ok(true);
    }

    // A public address can hairpin through the router to this same machine.
    // Compare the peer certificate with this installation's own certificate
    // instead of trusting the address or skipping TLS authentication.
    let (_, peer_fingerprint, _, _) = connect_link_tls(&state, &id)?;
    let certificate_path = state
        .settings_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("link-tls-cert.der");
    let local_certificate =
        CertificateDer::from(fs::read(certificate_path).map_err(|error| error.to_string())?);
    Ok(peer_fingerprint == link_certificate_fingerprint(&local_certificate))
}

#[tauri::command]
async fn open_link_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("apocalipse-link") {
        window.show().map_err(|error| error.to_string())?;
        let _ = window.unminimize();
        let _ = window.maximize();
        window.set_focus().map_err(|error| error.to_string())?;
        return Ok(());
    }

    WebviewWindowBuilder::new(&app, "apocalipse-link", WebviewUrl::App("link.html".into()))
        .title("Apocalipse Link")
        .inner_size(1400.0, 900.0)
        .min_inner_size(960.0, 640.0)
        .resizable(true)
        .maximized(true)
        .decorations(true)
        .build()
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(windows)]
fn windows_wide(value: &str) -> Vec<u16> {
    OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(windows)]
fn windows_logon_candidates(value: &str) -> Result<Vec<(String, Option<String>)>, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("system_username_required".to_owned());
    }

    let mut candidates = Vec::new();
    if let Some((domain, user)) = value.split_once('\\') {
        if domain.is_empty() || user.is_empty() {
            return Err("invalid_system_username".to_owned());
        }
        if ["hotmail.com", "outlook.com", "live.com"]
            .iter()
            .any(|candidate| domain.eq_ignore_ascii_case(candidate))
            && !user.contains('@')
        {
            let email = format!("{user}@{domain}");
            candidates.push((email.clone(), Some("MicrosoftAccount".to_owned())));
            candidates.push((email, None));
        } else {
            candidates.push((user.to_owned(), Some(domain.to_owned())));
        }
    } else if value.contains('@') {
        // UPN is the documented form for a NULL domain. Windows 10/11 can also
        // require explicit MicrosoftAccount or AzureAD providers, so retry both.
        candidates.push((value.to_owned(), None));
        candidates.push((value.to_owned(), Some("MicrosoftAccount".to_owned())));
        candidates.push((value.to_owned(), Some("AzureAD".to_owned())));
    } else {
        candidates.push((value.to_owned(), Some(".".to_owned())));
        candidates.push((value.to_owned(), None));
    }

    candidates.dedup();
    Ok(candidates)
}

#[cfg(windows)]
fn verify_system_account(username: &str, password: &str) -> Result<(), String> {
    let candidates = windows_logon_candidates(username)?;
    let mut password_wide = windows_wide(password);
    let mut last_error = 0;

    for (account, domain) in candidates {
        let account_wide = windows_wide(&account);
        let domain_wide = domain.as_deref().map(windows_wide);
        let domain_ptr = domain_wide
            .as_ref()
            .map_or(std::ptr::null(), |value| value.as_ptr());

        // NETWORK avoids credential caching; INTERACTIVE is a compatibility
        // fallback for local/Microsoft accounts that reject network logon.
        for logon_type in [3_u32, 2_u32] {
            let mut token: *mut c_void = std::ptr::null_mut();
            let authenticated = unsafe {
                LogonUserW(
                    account_wide.as_ptr(),
                    domain_ptr,
                    password_wide.as_ptr(),
                    logon_type,
                    0,
                    &mut token,
                )
            };
            if authenticated != 0 {
                if !token.is_null() {
                    unsafe {
                        CloseHandle(token);
                    }
                }
                password_wide.zeroize();
                return Ok(());
            }
            last_error = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
        }
    }

    password_wide.zeroize();
    Err(format!("system_auth_failed:{last_error}"))
}

#[cfg(unix)]
unsafe fn free_pam_responses(responses: *mut PamResponse, count: usize) {
    if responses.is_null() {
        return;
    }
    for index in 0..count {
        let response = responses.add(index);
        if !(*response).resp.is_null() {
            free((*response).resp.cast());
        }
    }
    free(responses.cast());
}

#[cfg(unix)]
unsafe extern "C" fn pam_link_conversation(
    message_count: c_int,
    messages: *const *const PamMessage,
    responses: *mut *mut PamResponse,
    appdata_ptr: *mut c_void,
) -> c_int {
    const PAM_SUCCESS: c_int = 0;
    const PAM_PROMPT_ECHO_OFF: c_int = 1;
    const PAM_PROMPT_ECHO_ON: c_int = 2;
    const PAM_ERROR_MSG: c_int = 3;
    const PAM_TEXT_INFO: c_int = 4;
    const PAM_CONV_ERR: c_int = 19;

    if message_count <= 0 || messages.is_null() || responses.is_null() || appdata_ptr.is_null() {
        return PAM_CONV_ERR;
    }
    let data = &*(appdata_ptr as *const PamConversationData);
    let count = message_count as usize;
    let output = calloc(count, std::mem::size_of::<PamResponse>()) as *mut PamResponse;
    if output.is_null() {
        return PAM_CONV_ERR;
    }
    for index in 0..count {
        let message = *messages.add(index);
        if message.is_null() {
            free_pam_responses(output, index);
            return PAM_CONV_ERR;
        }
        let source = match (*message).msg_style {
            PAM_PROMPT_ECHO_OFF => data.password.as_ptr(),
            PAM_PROMPT_ECHO_ON => data.username.as_ptr(),
            PAM_ERROR_MSG | PAM_TEXT_INFO => std::ptr::null(),
            _ => {
                free_pam_responses(output, index);
                return PAM_CONV_ERR;
            }
        };
        if !source.is_null() {
            let copy = strdup(source.cast());
            if copy.is_null() {
                free_pam_responses(output, index);
                return PAM_CONV_ERR;
            }
            (*output.add(index)).resp = copy;
        }
    }
    *responses = output;
    PAM_SUCCESS
}

#[cfg(unix)]
fn pam_bytes(value: &str) -> Result<Vec<u8>, String> {
    if value.as_bytes().contains(&0) {
        return Err("invalid_system_credential".to_owned());
    }
    let mut bytes = value.as_bytes().to_vec();
    bytes.push(0);
    Ok(bytes)
}

#[cfg(unix)]
fn verify_system_account(username: &str, password: &str) -> Result<(), String> {
    const PAM_SUCCESS: c_int = 0;
    if username.trim().is_empty() {
        return Err("system_username_required".to_owned());
    }
    let mut data = PamConversationData {
        username: pam_bytes(username.trim())?,
        password: pam_bytes(password)?,
    };
    let conversation = PamConversation {
        conv: Some(pam_link_conversation),
        appdata_ptr: (&mut data as *mut PamConversationData).cast(),
    };
    let mut handle = std::ptr::null_mut();
    let service = b"login\0";
    let started = unsafe {
        pam_start(
            service.as_ptr().cast(),
            data.username.as_ptr().cast(),
            &conversation,
            &mut handle,
        )
    };
    let (result, final_status) = if started != PAM_SUCCESS {
        (Err(format!("system_auth_start_failed:{started}")), started)
    } else {
        let authenticated = unsafe { pam_authenticate(handle, 0) };
        if authenticated != PAM_SUCCESS {
            (
                Err(format!("system_auth_failed:{authenticated}")),
                authenticated,
            )
        } else {
            let account = unsafe { pam_acct_mgmt(handle, 0) };
            if account == PAM_SUCCESS {
                (Ok(()), PAM_SUCCESS)
            } else {
                (Err(format!("system_account_denied:{account}")), account)
            }
        }
    };
    if started == PAM_SUCCESS && !handle.is_null() {
        unsafe {
            pam_end(handle, final_status);
        }
    }
    data.password.zeroize();
    result
}

#[tauri::command]
fn authenticate_local_link_account(username: String, mut password: String) -> Result<(), String> {
    let result = verify_system_account(&username, &password);
    password.zeroize();
    result
}

fn stable_link_share_id(prefix: &str, name: &str, path: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prefix.as_bytes());
    hasher.update([0]);
    hasher.update(name.as_bytes());
    hasher.update([0]);
    hasher.update(path.to_string_lossy().as_bytes());
    let digest = hasher.finalize();
    let suffix = digest[..12]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("{prefix}-{suffix}")
}

#[cfg(windows)]
fn windows_wide_pointer_to_string(pointer: *const u16) -> String {
    if pointer.is_null() {
        return String::new();
    }
    let mut length = 0_usize;
    unsafe {
        while length < 32_768 && *pointer.add(length) != 0 {
            length += 1;
        }
        String::from_utf16_lossy(std::slice::from_raw_parts(pointer, length))
    }
}

#[cfg(windows)]
fn windows_shared_link_shares() -> Vec<LinkShare> {
    const NERR_SUCCESS: u32 = 0;
    const ERROR_MORE_DATA: u32 = 234;
    const MAX_PREFERRED_LENGTH: u32 = u32::MAX;
    const STYPE_DISKTREE: u32 = 0;
    const STYPE_MASK: u32 = 0x0000_00ff;
    const STYPE_SPECIAL: u32 = 0x8000_0000;

    let mut shares = Vec::new();
    let mut resume_handle = 0_u32;
    loop {
        let mut buffer = std::ptr::null_mut::<u8>();
        let mut entries_read = 0_u32;
        let mut total_entries = 0_u32;
        let status = unsafe {
            NetShareEnum(
                std::ptr::null(),
                2,
                &mut buffer,
                MAX_PREFERRED_LENGTH,
                &mut entries_read,
                &mut total_entries,
                &mut resume_handle,
            )
        };
        if status != NERR_SUCCESS && status != ERROR_MORE_DATA {
            if !buffer.is_null() {
                unsafe {
                    NetApiBufferFree(buffer.cast());
                }
            }
            break;
        }
        if !buffer.is_null() && entries_read > 0 {
            let entries = unsafe {
                std::slice::from_raw_parts(
                    buffer.cast::<WindowsShareInfo2>(),
                    entries_read as usize,
                )
            };
            for entry in entries {
                let is_disk = entry.share_type & STYPE_MASK == STYPE_DISKTREE;
                let is_special = entry.share_type & STYPE_SPECIAL != 0;
                if !is_disk || is_special {
                    continue;
                }
                let name = windows_wide_pointer_to_string(entry.netname);
                let path = PathBuf::from(windows_wide_pointer_to_string(entry.path));
                if name.is_empty()
                    || name.ends_with('$')
                    || path.as_os_str().is_empty()
                    || !path.is_absolute()
                {
                    continue;
                }
                shares.push(LinkShare {
                    id: stable_link_share_id("windows", &name, &path),
                    name,
                    path,
                    allow_write: false,
                    directory: true,
                });
            }
        }
        if !buffer.is_null() {
            unsafe {
                NetApiBufferFree(buffer.cast());
            }
        }
        if status != ERROR_MORE_DATA {
            break;
        }
    }
    shares
}

#[cfg(not(windows))]
fn windows_shared_link_shares() -> Vec<LinkShare> {
    Vec::new()
}

#[cfg(target_os = "linux")]
fn linux_smb_config_entries(contents: &str) -> Vec<(String, PathBuf)> {
    let mut result = Vec::new();
    let mut section = String::new();
    let mut section_path: Option<PathBuf> = None;

    let mut flush = |name: &mut String, path: &mut Option<PathBuf>| {
        let normalized = name.trim();
        let reserved = normalized.eq_ignore_ascii_case("global")
            || normalized.eq_ignore_ascii_case("homes")
            || normalized.eq_ignore_ascii_case("printers")
            || normalized.eq_ignore_ascii_case("print$");
        if !normalized.is_empty() && !reserved && !normalized.ends_with('$') {
            if let Some(candidate) = path.take() {
                if candidate.is_absolute() {
                    result.push((normalized.to_owned(), candidate));
                }
            }
        } else {
            *path = None;
        }
    };

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            flush(&mut section, &mut section_path);
            section = line[1..line.len() - 1].trim().to_owned();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case("path") {
            let value = value.trim().trim_matches('"');
            if !value.is_empty() {
                section_path = Some(PathBuf::from(value));
            }
        }
    }
    flush(&mut section, &mut section_path);
    result
}

#[cfg(target_os = "linux")]
fn linux_smb_usershare_entry(name: &str, contents: &str) -> Option<(String, PathBuf)> {
    if name.trim().is_empty() || name.ends_with('$') {
        return None;
    }
    let path = contents.lines().find_map(|raw_line| {
        let line = raw_line.trim();
        let (key, value) = line.split_once('=')?;
        key.trim()
            .eq_ignore_ascii_case("path")
            .then(|| PathBuf::from(value.trim().trim_matches('"')))
    })?;
    path.is_absolute().then(|| (name.to_owned(), path))
}

#[cfg(target_os = "linux")]
fn linux_shared_link_shares() -> Vec<LinkShare> {
    let mut entries = Vec::<(String, PathBuf)>::new();

    for config in ["/etc/samba/smb.conf", "/etc/samba/smb.conf.local"] {
        if let Ok(contents) = fs::read_to_string(config) {
            entries.extend(linux_smb_config_entries(&contents));
        }
    }

    for directory in ["/var/lib/samba/usershares", "/var/lib/samba/usershare"] {
        let Ok(items) = fs::read_dir(directory) else {
            continue;
        };
        for item in items.flatten() {
            if !item.path().is_file() {
                continue;
            }
            let name = item.file_name().to_string_lossy().into_owned();
            let Ok(contents) = fs::read_to_string(item.path()) else {
                continue;
            };
            if let Some(entry) = linux_smb_usershare_entry(&name, &contents) {
                entries.push(entry);
            }
        }
    }

    let mut shares = Vec::new();
    for (name, path) in entries {
        if shares.iter().any(|share: &LinkShare| share.path == path) {
            continue;
        }
        shares.push(LinkShare {
            id: stable_link_share_id("linux-smb", &name, &path),
            name,
            path,
            // OS-discovered shares are intentionally read-only inside Link.
            // The user can explicitly share the same folder in Apocalipse Link
            // to grant Link's own read/write permission.
            allow_write: false,
            directory: true,
        });
    }
    shares
}

#[cfg(not(target_os = "linux"))]
fn linux_shared_link_shares() -> Vec<LinkShare> {
    Vec::new()
}

fn effective_link_shares(settings: &UserSettings) -> Vec<LinkShare> {
    let mut shares = settings.link_shares.clone();
    for share in windows_shared_link_shares()
        .into_iter()
        .chain(linux_shared_link_shares())
    {
        if shares
            .iter()
            .any(|existing| existing.path == share.path || existing.id == share.id)
        {
            continue;
        }
        shares.push(share);
    }
    shares
}

fn link_share_entries(settings: &UserSettings) -> Vec<LinkFileEntry> {
    effective_link_shares(settings)
        .into_iter()
        .map(|share| {
            let metadata = fs::metadata(&share.path).ok();
            LinkFileEntry {
                name: share.name.clone(),
                path: format!("/shares/{}", share.id),
                size: metadata
                    .as_ref()
                    .filter(|value| value.is_file())
                    .map_or(0, |value| value.len()),
                directory: metadata
                    .as_ref()
                    .map_or(share.directory, |value| value.is_dir()),
            }
        })
        .collect()
}

fn resolve_link_share(
    settings: &UserSettings,
    virtual_path: &str,
) -> Result<(PathBuf, bool), String> {
    let mut parts = virtual_path.trim_matches('/').split('/');
    if parts.next() != Some("shares") {
        return Err("path_not_shared".to_owned());
    }
    let id = parts.next().ok_or_else(|| "path_not_shared".to_owned())?;
    let share = effective_link_shares(settings)
        .into_iter()
        .find(|share| share.id == id)
        .ok_or_else(|| "path_not_shared".to_owned())?;
    let mut path = share.path;
    for part in parts {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." || part.contains(['/', '\\']) {
            return Err("invalid_remote_path".to_owned());
        }
        path.push(part);
    }
    Ok((path, share.allow_write))
}

fn list_shared_link_directory(
    settings: &UserSettings,
    path: &str,
) -> Result<Vec<LinkFileEntry>, String> {
    if path.trim().is_empty() {
        return Ok(link_share_entries(settings));
    }
    let (directory, _) = resolve_link_share(settings, path)?;
    let mut entries = list_link_directory(&directory.to_string_lossy())?;
    for entry in &mut entries {
        let name = entry.name.clone();
        entry.path = remote_link_join(path, &name);
    }
    Ok(entries)
}

#[tauri::command]
fn list_link_shares(state: State<'_, AppState>) -> Result<Vec<LinkShare>, String> {
    Ok(state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .link_shares
        .clone())
}

#[tauri::command]
fn add_link_share(state: State<'_, AppState>) -> Result<Vec<LinkShare>, String> {
    let Some(path) = rfd::FileDialog::new().pick_folder() else {
        return Err("cancelled".to_owned());
    };
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("Shared folder")
        .to_owned();
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    if !settings.link_shares.iter().any(|share| share.path == path) {
        settings.link_shares.push(LinkShare {
            id: uuid::Uuid::new_v4().simple().to_string(),
            name,
            path,
            allow_write: false,
            directory: true,
        });
        save_settings(&state, &settings)?;
    }
    Ok(settings.link_shares.clone())
}

#[tauri::command]
fn add_link_file_share(state: State<'_, AppState>) -> Result<Vec<LinkShare>, String> {
    let Some(path) = rfd::FileDialog::new().pick_file() else {
        return Err("cancelled".to_owned());
    };
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("Shared file")
        .to_owned();
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    if !settings.link_shares.iter().any(|share| share.path == path) {
        settings.link_shares.push(LinkShare {
            id: uuid::Uuid::new_v4().simple().to_string(),
            name,
            path,
            allow_write: false,
            directory: false,
        });
        save_settings(&state, &settings)?;
    }
    Ok(settings.link_shares.clone())
}

#[tauri::command]
fn update_link_share(
    state: State<'_, AppState>,
    id: String,
    allow_write: bool,
) -> Result<Vec<LinkShare>, String> {
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    let share = settings
        .link_shares
        .iter_mut()
        .find(|share| share.id == id)
        .ok_or_else(|| "share_not_found".to_owned())?;
    share.allow_write = allow_write;
    save_settings(&state, &settings)?;
    Ok(settings.link_shares.clone())
}

#[tauri::command]
fn remove_link_share(state: State<'_, AppState>, id: String) -> Result<Vec<LinkShare>, String> {
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.link_shares.retain(|share| share.id != id);
    save_settings(&state, &settings)?;
    Ok(settings.link_shares.clone())
}

fn remove_link_path(path: &str) -> Result<(), String> {
    let path = safe_link_path(path)?;
    if path.file_name().is_none() {
        return Err("cannot_delete_link_root".to_owned());
    }
    let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
    if metadata.is_dir() {
        fs::remove_dir_all(path).map_err(|error| error.to_string())
    } else {
        fs::remove_file(path).map_err(|error| error.to_string())
    }
}

#[tauri::command]
fn delete_local_link_item(path: String) -> Result<(), String> {
    remove_link_path(&path)
}

#[derive(Debug)]
struct LinkServerCertVerifier;

impl ServerCertVerifier for LinkServerCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        // The exact certificate is pinned immediately after the handshake.
        // Signature validation below proves possession of its private key.
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        let provider = rustls::crypto::ring::default_provider();
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &provider.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        let provider = rustls::crypto::ring::default_provider();
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &provider.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

fn load_or_create_link_tls_config(directory: &Path) -> Result<Arc<ServerConfig>, String> {
    let cert_path = directory.join("link-tls-cert.der");
    let key_path = directory.join("link-tls-key.der");
    if !cert_path.is_file() || !key_path.is_file() {
        let rcgen::CertifiedKey { cert, key_pair } =
            rcgen::generate_simple_self_signed(vec!["apocalipse-link.local".to_owned()])
                .map_err(|error| error.to_string())?;
        fs::write(&cert_path, cert.der().as_ref()).map_err(|error| error.to_string())?;
        fs::write(&key_path, key_pair.serialize_der()).map_err(|error| error.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600));
        }
    }
    let certificate =
        CertificateDer::from(fs::read(&cert_path).map_err(|error| error.to_string())?);
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
        fs::read(&key_path).map_err(|error| error.to_string())?,
    ));
    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![certificate], key)
        .map(Arc::new)
        .map_err(|error| error.to_string())
}

fn link_remote_parts(id: &str) -> Result<(String, u16, String), String> {
    let value = id.trim();
    if value.is_empty() {
        return Err("invalid_remote_id".to_owned());
    }
    let normalized = if let Some(rest) = value.strip_prefix("http://") {
        format!("https://{rest}")
    } else if value.starts_with("https://") {
        value.to_owned()
    } else {
        format!("https://{value}")
    };
    let parsed = url::Url::parse(&normalized).map_err(|error| error.to_string())?;
    let host = parsed
        .host_str()
        .ok_or_else(|| "invalid_remote_id".to_owned())?
        .to_owned();
    let port = parsed.port().unwrap_or(LINK_PORT);
    let authority = if host.contains(':') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    };
    Ok((host, port, authority))
}

fn link_certificate_fingerprint(certificate: &CertificateDer<'_>) -> String {
    let digest = Sha256::digest(certificate.as_ref());
    digest
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(":")
}

type LinkTlsStream = StreamOwned<ClientConnection, TcpStream>;

fn connect_link_tls(
    state: &AppState,
    id: &str,
) -> Result<(LinkTlsStream, String, bool, String), String> {
    let (host, port, authority) = link_remote_parts(id)?;
    let mut socket =
        TcpStream::connect((host.as_str(), port)).map_err(|error| error.to_string())?;
    socket
        .set_read_timeout(Some(Duration::from_secs(45)))
        .map_err(|error| error.to_string())?;
    socket
        .set_write_timeout(Some(Duration::from_secs(45)))
        .map_err(|error| error.to_string())?;
    let config = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(LinkServerCertVerifier))
        .with_no_client_auth();
    let server_name = ServerName::try_from("apocalipse-link.local".to_owned())
        .map_err(|error| error.to_string())?;
    let mut connection =
        ClientConnection::new(Arc::new(config), server_name).map_err(|error| error.to_string())?;
    while connection.is_handshaking() {
        connection
            .complete_io(&mut socket)
            .map_err(|error| error.to_string())?;
    }
    let certificate = connection
        .peer_certificates()
        .and_then(|certificates| certificates.first())
        .ok_or_else(|| "link_tls_certificate_missing".to_owned())?;
    let fingerprint = link_certificate_fingerprint(certificate);
    let host_key = authority.to_ascii_lowercase();
    let first_trust = {
        let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
        match settings.link_trusted_certificates.get(&host_key) {
            Some(expected) if expected != &fingerprint => {
                return Err("link_tls_certificate_changed".to_owned())
            }
            Some(_) => false,
            None => {
                settings
                    .link_trusted_certificates
                    .insert(host_key, fingerprint.clone());
                save_settings(state, &settings)?;
                true
            }
        }
    };
    Ok((
        StreamOwned::new(connection, socket),
        fingerprint,
        first_trust,
        authority,
    ))
}

struct LinkHttpResponse {
    status: u16,
    body: Vec<u8>,
}

fn read_link_http_response<S: Read>(stream: &mut S) -> Result<LinkHttpResponse, String> {
    let mut response = Vec::with_capacity(8_192);
    let mut chunk = [0_u8; 8_192];
    let header_end = loop {
        let count = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if count == 0 || response.len() + count > 65_536 {
            return Err("invalid_link_http_response".to_owned());
        }
        response.extend_from_slice(&chunk[..count]);
        if let Some(position) = response.windows(4).position(|window| window == b"\r\n\r\n") {
            break position;
        }
    };
    let body_start = header_end + 4;
    let headers = String::from_utf8_lossy(&response[..body_start]);
    let status = headers
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| "invalid_link_http_status".to_owned())?;
    let content_length = bridge_content_length(&headers);
    if content_length > 16 * 1024 * 1024 {
        return Err("link_http_response_too_large".to_owned());
    }
    while response.len() < body_start + content_length {
        let remaining = body_start + content_length - response.len();
        let read_size = remaining.min(chunk.len());
        let count = stream
            .read(&mut chunk[..read_size])
            .map_err(|error| error.to_string())?;
        if count == 0 {
            return Err("incomplete_link_http_response".to_owned());
        }
        response.extend_from_slice(&chunk[..count]);
    }
    Ok(LinkHttpResponse {
        status,
        body: response[body_start..body_start + content_length].to_vec(),
    })
}

fn link_http_request(
    state: &AppState,
    id: &str,
    method: &str,
    target: &str,
    bearer: Option<&str>,
    body: &[u8],
    content_type: Option<&str>,
) -> Result<(LinkHttpResponse, String, bool), String> {
    let (mut stream, fingerprint, first_trust, authority) = connect_link_tls(state, id)?;
    let authorization = bearer
        .map(|value| format!("Authorization: Bearer {value}\r\n"))
        .unwrap_or_default();
    let content_type = content_type
        .map(|value| format!("Content-Type: {value}\r\n"))
        .unwrap_or_default();
    let headers = format!(
        "{method} {target} HTTP/1.1\r\nHost: {authority}\r\n{authorization}{content_type}Content-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(headers.as_bytes())
        .map_err(|error| error.to_string())?;
    if !body.is_empty() {
        stream.write_all(body).map_err(|error| error.to_string())?;
    }
    stream.flush().map_err(|error| error.to_string())?;
    Ok((
        read_link_http_response(&mut stream)?,
        fingerprint,
        first_trust,
    ))
}

fn ensure_link_http_success(response: &LinkHttpResponse) -> Result<(), String> {
    if (200..300).contains(&response.status) {
        Ok(())
    } else {
        Err(format!("remote_http_status:{}", response.status))
    }
}

fn read_link_download_head<S: Read>(stream: &mut S) -> Result<(u16, usize, Vec<u8>), String> {
    let mut buffer = Vec::with_capacity(8192);
    let mut chunk = [0_u8; 8192];
    let header_end = loop {
        let count = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if count == 0 || buffer.len() + count > 65_536 {
            return Err("invalid_link_http_response".to_owned());
        }
        buffer.extend_from_slice(&chunk[..count]);
        if let Some(position) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
            break position;
        }
    };
    let headers = String::from_utf8_lossy(&buffer[..header_end + 4]).into_owned();
    let status = headers
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| "invalid_link_http_status".to_owned())?;
    let length = bridge_content_length(&headers);
    Ok((status, length, buffer[header_end + 4..].to_vec()))
}

fn copy_link_stream_with_progress<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    remaining: Option<u64>,
    reporter: &mut LinkTransferReporter,
) -> Result<u64, String> {
    let mut copied = 0_u64;
    let mut buffer = [0_u8; 128 * 1024];
    loop {
        reporter.checkpoint()?;
        let limit = remaining
            .map(|total| total.saturating_sub(copied))
            .unwrap_or(buffer.len() as u64);
        if limit == 0 {
            break;
        }
        let read_size = buffer.len().min(limit as usize);
        let count = reader
            .read(&mut buffer[..read_size])
            .map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        writer
            .write_all(&buffer[..count])
            .map_err(|error| error.to_string())?;
        copied = copied.saturating_add(count as u64);
        reporter.advance(count as u64);
    }
    if remaining.is_some_and(|expected| copied != expected) {
        return Err("incomplete_link_transfer".to_owned());
    }
    Ok(copied)
}

fn download_link_file_to(
    state: &AppState,
    id: &str,
    password: &str,
    remote_path: &str,
    destination: &Path,
    reporter: &mut LinkTransferReporter,
) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let encoded = url::form_urlencoded::byte_serialize(remote_path.as_bytes()).collect::<String>();
    let (mut stream, _, _, authority) = connect_link_tls(state, id)?;
    let request = format!(
        "GET /v1/link/file?path={encoded} HTTP/1.1\r\nHost: {authority}\r\nAuthorization: Bearer {password}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())?;
    let (status, length, initial) = read_link_download_head(&mut stream)?;
    if status != 200 {
        return Err(format!("remote_http_status:{status}"));
    }
    reporter.set_total(length as u64);
    let mut file = fs::File::create(destination).map_err(|error| error.to_string())?;
    let initial_len = initial.len().min(length);
    file.write_all(&initial[..initial_len])
        .map_err(|error| error.to_string())?;
    reporter.advance(initial_len as u64);
    let remaining = length.saturating_sub(initial_len) as u64;
    copy_link_stream_with_progress(&mut stream, &mut file, Some(remaining), reporter)?;
    file.flush().map_err(|error| error.to_string())?;
    Ok(())
}

fn send_link_directory(
    state: &AppState,
    id: &str,
    password: &str,
    remote_path: &str,
) -> Result<(), String> {
    let encoded = url::form_urlencoded::byte_serialize(remote_path.as_bytes()).collect::<String>();
    let (response, _, _) = link_http_request(
        state,
        id,
        "PUT",
        &format!("/v1/link/directory?path={encoded}"),
        Some(password),
        &[],
        None,
    )?;
    ensure_link_http_success(&response)
}

fn send_link_file(
    state: &AppState,
    id: &str,
    password: &str,
    source: &Path,
    remote_path: &str,
    reporter: &mut LinkTransferReporter,
) -> Result<(), String> {
    let encoded = url::form_urlencoded::byte_serialize(remote_path.as_bytes()).collect::<String>();
    let (mut stream, _, _, authority) = connect_link_tls(state, id)?;
    let mut file = fs::File::open(source).map_err(|error| error.to_string())?;
    let size = file.metadata().map_err(|error| error.to_string())?.len();
    let headers = format!(
        "PUT /v1/link/file?path={encoded} HTTP/1.1\r\nHost: {authority}\r\nAuthorization: Bearer {password}\r\nContent-Length: {size}\r\nConnection: close\r\n\r\n"
    );
    stream
        .write_all(headers.as_bytes())
        .map_err(|error| error.to_string())?;
    copy_link_stream_with_progress(&mut file, &mut stream, Some(size), reporter)?;
    stream.flush().map_err(|error| error.to_string())?;
    let response = read_link_http_response(&mut stream)?;
    ensure_link_http_success(&response)
}

fn link_path_total_size(path: &Path) -> Result<u64, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() {
        return Ok(0);
    }
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    let mut total = 0_u64;
    let mut pending = vec![path.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let file_type = entry.file_type().map_err(|error| error.to_string())?;
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                pending.push(entry.path());
            } else if file_type.is_file() {
                total = total
                    .saturating_add(entry.metadata().map_err(|error| error.to_string())?.len());
            }
        }
    }
    Ok(total)
}

fn copy_local_link_file(
    source: &Path,
    destination: &Path,
    reporter: &mut LinkTransferReporter,
) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut input = fs::File::open(source).map_err(|error| error.to_string())?;
    let mut output = fs::File::create(destination).map_err(|error| error.to_string())?;
    let size = input.metadata().map_err(|error| error.to_string())?.len();
    copy_link_stream_with_progress(&mut input, &mut output, Some(size), reporter)?;
    output.flush().map_err(|error| error.to_string())
}

#[tauri::command]
async fn delete_remote_link_item(
    app: tauri::AppHandle,
    id: String,
    password: String,
    path: String,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let encoded = url::form_urlencoded::byte_serialize(path.as_bytes()).collect::<String>();
        let (response, _, _) = link_http_request(
            &state,
            &id,
            "DELETE",
            &format!("/v1/link/item?path={encoded}"),
            Some(&password),
            &[],
            None,
        )?;
        ensure_link_http_success(&response)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn get_remote_link_capabilities(
    app: tauri::AppHandle,
    id: String,
    password: String,
    path: String,
) -> Result<LinkCapabilities, String> {
    tokio::task::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let encoded = url::form_urlencoded::byte_serialize(path.as_bytes()).collect::<String>();
        let (response, _, _) = link_http_request(
            &state,
            &id,
            "GET",
            &format!("/v1/link/capabilities?path={encoded}"),
            Some(&password),
            &[],
            None,
        )?;
        ensure_link_http_success(&response)?;
        serde_json::from_slice(&response.body).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
fn list_local_link_files(
    state: State<'_, AppState>,
    path: String,
) -> Result<Vec<LinkFileEntry>, String> {
    let settings = state.settings.lock().map_err(|error| error.to_string())?;
    list_shared_link_directory(&settings, &path)
}

#[tauri::command]
fn get_local_link_share_capabilities(
    state: State<'_, AppState>,
    path: String,
) -> Result<LinkCapabilities, String> {
    let settings = state.settings.lock().map_err(|error| error.to_string())?;
    let allow_write = resolve_link_share(&settings, &path)
        .map(|(_, write)| write)
        .unwrap_or(false);
    Ok(LinkCapabilities { allow_write })
}

#[tauri::command]
fn delete_local_shared_link_item(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let target = {
        let settings = state.settings.lock().map_err(|error| error.to_string())?;
        let (target, allow_write) = resolve_link_share(&settings, &path)?;
        if !allow_write {
            return Err("link_write_not_allowed".to_owned());
        }
        target
    };
    remove_link_path(&target.to_string_lossy())
}

fn copy_local_link_directory(
    source: &Path,
    destination: &Path,
    reporter: &mut LinkTransferReporter,
) -> Result<(), String> {
    if destination.starts_with(source) {
        return Err("destination_inside_source".to_owned());
    }
    fs::create_dir_all(destination).map_err(|error| error.to_string())?;
    let mut pending = vec![(source.to_path_buf(), destination.to_path_buf())];
    while let Some((current_source, current_destination)) = pending.pop() {
        reporter.checkpoint()?;
        fs::create_dir_all(&current_destination).map_err(|error| error.to_string())?;
        for entry in fs::read_dir(&current_source).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let file_type = entry.file_type().map_err(|error| error.to_string())?;
            if file_type.is_symlink() {
                continue;
            }
            let target = current_destination.join(entry.file_name());
            if file_type.is_dir() {
                pending.push((entry.path(), target));
            } else if file_type.is_file() {
                copy_local_link_file(&entry.path(), &target, reporter)?;
            }
        }
    }
    Ok(())
}

#[tauri::command]
async fn download_local_shared_link_item(
    app: tauri::AppHandle,
    path: String,
    directory: bool,
    file_name: Option<String>,
    transfer_id: Option<String>,
) -> Result<String, String> {
    let transfer_id = transfer_id.unwrap_or_else(|| uuid::Uuid::new_v4().simple().to_string());
    let source = {
        let state = app.state::<AppState>();
        let settings = state.settings.lock().map_err(|error| error.to_string())?;
        resolve_link_share(&settings, &path)?.0
    };
    let metadata = fs::symlink_metadata(&source).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() {
        return Err("symlink_not_allowed".to_owned());
    }
    if directory || metadata.is_dir() {
        let Some(parent) = rfd::FileDialog::new().pick_folder() else {
            return Err("cancelled".to_owned());
        };
        let name = file_name
            .as_deref()
            .filter(|value| !value.is_empty())
            .or_else(|| source.file_name().and_then(|value| value.to_str()))
            .unwrap_or("download");
        let destination = parent.join(name);
        let source_for_copy = source.clone();
        let destination_for_copy = destination.clone();
        let progress_app = app.clone();
        tokio::task::spawn_blocking(move || {
            let mut reporter = LinkTransferReporter::new(progress_app, transfer_id, "download", 0);
            reporter.checkpoint()?;
            let total = link_path_total_size(&source_for_copy)?;
            reporter.set_total(total);
            copy_local_link_directory(&source_for_copy, &destination_for_copy, &mut reporter)?;
            reporter.finish();
            Ok::<(), String>(())
        })
        .await
        .map_err(|error| error.to_string())??;
        return Ok(destination.to_string_lossy().into_owned());
    }
    let name = file_name
        .as_deref()
        .filter(|value| !value.is_empty())
        .or_else(|| source.file_name().and_then(|value| value.to_str()))
        .unwrap_or("download");
    let Some(destination) = rfd::FileDialog::new().set_file_name(name).save_file() else {
        return Err("cancelled".to_owned());
    };
    let source_for_copy = source.clone();
    let destination_for_copy = destination.clone();
    tokio::task::spawn_blocking(move || {
        let mut reporter = LinkTransferReporter::new(app, transfer_id, "download", 0);
        reporter.checkpoint()?;
        let total = link_path_total_size(&source_for_copy)?;
        reporter.set_total(total);
        copy_local_link_file(&source_for_copy, &destination_for_copy, &mut reporter)?;
        reporter.finish();
        Ok::<(), String>(())
    })
    .await
    .map_err(|error| error.to_string())??;
    Ok(destination.to_string_lossy().into_owned())
}

#[tauri::command]
async fn upload_local_shared_link_item(
    app: tauri::AppHandle,
    remote_directory: String,
    local_path: String,
    transfer_id: Option<String>,
) -> Result<String, String> {
    let transfer_id = transfer_id.unwrap_or_else(|| uuid::Uuid::new_v4().simple().to_string());
    let (source, destination_root) = {
        let state = app.state::<AppState>();
        let settings = state.settings.lock().map_err(|error| error.to_string())?;
        let source = resolve_link_share(&settings, &local_path)?.0;
        let (destination_root, allow_write) = resolve_link_share(&settings, &remote_directory)?;
        if !allow_write {
            return Err("link_write_not_allowed".to_owned());
        }
        (source, destination_root)
    };
    if !destination_root.is_dir() {
        return Err("select_remote_directory".to_owned());
    }
    let name = source
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "invalid_file_name".to_owned())?
        .to_owned();
    let destination = destination_root.join(&name);
    if source == destination {
        return Err("source_and_destination_are_same".to_owned());
    }
    let metadata = fs::symlink_metadata(&source).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() {
        return Err("symlink_not_allowed".to_owned());
    }
    let source_for_copy = source.clone();
    let destination_for_copy = destination.clone();
    tokio::task::spawn_blocking(move || {
        let mut reporter = LinkTransferReporter::new(app, transfer_id, "upload", 0);
        reporter.checkpoint()?;
        let total = link_path_total_size(&source_for_copy)?;
        reporter.set_total(total);
        if source_for_copy.is_dir() {
            copy_local_link_directory(&source_for_copy, &destination_for_copy, &mut reporter)?;
        } else {
            copy_local_link_file(&source_for_copy, &destination_for_copy, &mut reporter)?;
        }
        reporter.finish();
        Ok::<(), String>(())
    })
    .await
    .map_err(|error| error.to_string())??;
    Ok(remote_link_join(&remote_directory, &name))
}

#[tauri::command]
async fn authenticate_remote_link_account(
    app: tauri::AppHandle,
    id: String,
    username: String,
    password: String,
) -> Result<RemoteLinkAuthentication, String> {
    tokio::task::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let mut request = LinkAuthRequest { username, password };
        let body = serde_json::to_vec(&request).map_err(|error| error.to_string())?;
        request.password.zeroize();
        let (response, fingerprint, first_trust) = link_http_request(
            &state,
            &id,
            "POST",
            "/v1/link/auth",
            None,
            &body,
            Some("application/json"),
        )?;
        if response.status == 401 {
            return Err("remote_system_auth_failed".to_owned());
        }
        ensure_link_http_success(&response)?;
        let authenticated: LinkAuthResponse =
            serde_json::from_slice(&response.body).map_err(|error| error.to_string())?;
        Ok(RemoteLinkAuthentication {
            token: authenticated.token,
            fingerprint,
            first_trust,
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

fn list_remote_link_directory_sync(
    state: &AppState,
    id: &str,
    password: &str,
    path: String,
) -> Result<Vec<LinkFileEntry>, String> {
    let body = serde_json::to_vec(&LinkListRequest {
        password: password.to_owned(),
        path,
    })
    .map_err(|error| error.to_string())?;
    let (response, _, _) = link_http_request(
        state,
        id,
        "POST",
        "/v1/link/list",
        None,
        &body,
        Some("application/json"),
    )?;
    ensure_link_http_success(&response)?;
    serde_json::from_slice(&response.body).map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_remote_link_files(
    app: tauri::AppHandle,
    id: String,
    password: String,
    path: String,
) -> Result<Vec<LinkFileEntry>, String> {
    tokio::task::spawn_blocking(move || {
        let state = app.state::<AppState>();
        list_remote_link_directory_sync(&state, &id, &password, path)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn download_remote_link_file(
    app: tauri::AppHandle,
    id: String,
    password: String,
    path: String,
    directory: bool,
    file_name: Option<String>,
    transfer_id: Option<String>,
) -> Result<String, String> {
    let transfer_id = transfer_id.unwrap_or_else(|| uuid::Uuid::new_v4().simple().to_string());
    let fallback_name = Path::new(&path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("download");
    let file_name = file_name
        .as_deref()
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback_name)
        .to_owned();
    if directory {
        let Some(destination_parent) = rfd::FileDialog::new().pick_folder() else {
            return Err("cancelled".to_owned());
        };
        let destination = destination_parent.join(&file_name);
        return tokio::task::spawn_blocking(move || {
            let state = app.state::<AppState>();
            fs::create_dir_all(&destination).map_err(|error| error.to_string())?;
            let mut reporter = LinkTransferReporter::new(app.clone(), transfer_id, "download", 0);
            let mut pending = vec![(path, destination.clone())];
            let mut files = Vec::<(String, PathBuf, u64)>::new();
            let mut total = 0_u64;
            while let Some((remote_directory, local_directory)) = pending.pop() {
                reporter.checkpoint()?;
                fs::create_dir_all(&local_directory).map_err(|error| error.to_string())?;
                for entry in
                    list_remote_link_directory_sync(&state, &id, &password, remote_directory)?
                {
                    let local_path = local_directory.join(&entry.name);
                    if entry.directory {
                        pending.push((entry.path, local_path));
                    } else {
                        total = total.saturating_add(entry.size);
                        files.push((entry.path, local_path, entry.size));
                    }
                }
            }
            reporter.set_total(total);
            for (remote_path, local_path, _size) in files {
                reporter.checkpoint()?;
                download_link_file_to(
                    &state,
                    &id,
                    &password,
                    &remote_path,
                    &local_path,
                    &mut reporter,
                )?;
            }
            reporter.finish();
            Ok(destination.to_string_lossy().into_owned())
        })
        .await
        .map_err(|error| error.to_string())?;
    }
    let Some(destination) = rfd::FileDialog::new().set_file_name(&file_name).save_file() else {
        return Err("cancelled".to_owned());
    };
    tokio::task::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let mut reporter = LinkTransferReporter::new(app.clone(), transfer_id, "download", 0);
        reporter.checkpoint()?;
        download_link_file_to(&state, &id, &password, &path, &destination, &mut reporter)?;
        reporter.finish();
        Ok(destination.to_string_lossy().into_owned())
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Fills a queued/paused/failed task's destination directly from a file
/// already found on a paired Apocalipse Link peer, instead of downloading
/// it over the internet - useful when the exact file already sits on
/// another of the user's own machines. This is a plain one-shot copy (no
/// partial-resume bookkeeping): it always writes the destination from
/// scratch, matching how a fresh Link download already behaves.
#[tauri::command]
async fn use_remote_link_file_for_download(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    task_id: DownloadId,
    id: String,
    password: String,
    path: String,
    transfer_id: Option<String>,
) -> Result<(), String> {
    let destination = {
        let queue = state.queue.lock().map_err(|error| error.to_string())?;
        let task = queue
            .iter()
            .find(|task| task.id == task_id)
            .ok_or_else(|| "download_not_found".to_owned())?;
        if !matches!(
            task.state,
            DownloadState::Queued | DownloadState::Paused | DownloadState::Failed { .. }
        ) {
            return Err("download_not_resumable".to_owned());
        }
        if matches!(
            classify_url(&task.source),
            Some(DownloadKind::Torrent | DownloadKind::Magnet)
        ) {
            return Err("torrent_relocate_unsupported".to_owned());
        }
        task.destination.clone()
    };
    let transfer_id = transfer_id.unwrap_or_else(|| uuid::Uuid::new_v4().simple().to_string());
    let app_for_task = app.clone();
    tokio::task::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let mut reporter = LinkTransferReporter::new(app.clone(), transfer_id, "download", 0);
        reporter.checkpoint()?;
        download_link_file_to(&state, &id, &password, &path, &destination, &mut reporter)?;
        reporter.finish();
        Ok::<_, String>(())
    })
    .await
    .map_err(|error| error.to_string())??;
    let state = app_for_task.state::<AppState>();
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    let task = queue
        .iter_mut()
        .find(|task| task.id == task_id)
        .ok_or_else(|| "download_not_found".to_owned())?;
    let received = fs::metadata(&task.destination)
        .map(|metadata| metadata.len())
        .unwrap_or(task.total.unwrap_or(0));
    task.state = DownloadState::Completed;
    task.received = received;
    task.total = Some(received);
    task.progress_percent = Some(100.0);
    task.download_speed = Some(0);
    task.upload_speed = Some(0);
    task.completed_at = Some(epoch_seconds());
    save_queue(&state, &queue)?;
    drop(queue);
    diagnostic_log(
        &state,
        "INFO",
        "task.filled_from_link",
        &format!("task={task_id} bytes={received}"),
    );
    Ok(())
}

fn remote_link_join(parent: &str, child: &str) -> String {
    let separator = if parent.contains('\\') && !parent.contains('/') {
        '\\'
    } else {
        '/'
    };
    format!(
        "{}{}{}",
        parent.trim_end_matches(['/', '\\']),
        separator,
        child
    )
}

#[tauri::command]
async fn upload_remote_link_file(
    app: tauri::AppHandle,
    id: String,
    password: String,
    remote_directory: String,
    local_path: String,
    transfer_id: Option<String>,
) -> Result<String, String> {
    let transfer_id = transfer_id.unwrap_or_else(|| uuid::Uuid::new_v4().simple().to_string());
    tokio::task::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let source = {
            let settings = state.settings.lock().map_err(|error| error.to_string())?;
            resolve_link_share(&settings, &local_path)?.0
        };
        if !source.is_file() && !source.is_dir() {
            return Err("selected_local_item_not_found".to_owned());
        }
        if remote_directory.trim().is_empty() {
            return Err("select_remote_directory".to_owned());
        }
        let name = source
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| "invalid_file_name".to_owned())?;
        let remote_path = remote_link_join(&remote_directory, name);
        let mut reporter = LinkTransferReporter::new(app.clone(), transfer_id, "upload", 0);
        reporter.checkpoint()?;
        let total = link_path_total_size(&source)?;
        reporter.set_total(total);
        if source.is_file() {
            send_link_file(&state, &id, &password, &source, &remote_path, &mut reporter)?;
            reporter.finish();
            return Ok(remote_path);
        }
        send_link_directory(&state, &id, &password, &remote_path)?;
        let mut pending = vec![(source, remote_path.clone())];
        while let Some((local_directory, target_directory)) = pending.pop() {
            reporter.checkpoint()?;
            for entry in fs::read_dir(local_directory).map_err(|error| error.to_string())? {
                let entry = entry.map_err(|error| error.to_string())?;
                let file_type = entry.file_type().map_err(|error| error.to_string())?;
                if file_type.is_symlink() {
                    continue;
                }
                let entry_name = entry.file_name().to_string_lossy().into_owned();
                let target_path = remote_link_join(&target_directory, &entry_name);
                if file_type.is_dir() {
                    send_link_directory(&state, &id, &password, &target_path)?;
                    pending.push((entry.path(), target_path));
                } else if file_type.is_file() {
                    send_link_file(
                        &state,
                        &id,
                        &password,
                        &entry.path(),
                        &target_path,
                        &mut reporter,
                    )?;
                }
            }
        }
        reporter.finish();
        Ok(remote_path)
    })
    .await
    .map_err(|error| error.to_string())?
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
    inspect_torrent_bytes(&data)
}

fn inspect_torrent_bytes(data: &[u8]) -> Result<TorrentInspection, String> {
    let mut position = 0;
    let BValue::Dict(root) = parse_bencode(data, &mut position)? else {
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
        torrent_path: None,
    })
}

fn torrent_store_directory(state: &AppState) -> Result<PathBuf, String> {
    let runtime_root = state.queue_path.parent().unwrap_or_else(|| Path::new("."));
    let directory = runtime_root.join("torrents");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(directory)
}

fn persist_torrent_bytes(state: &AppState, bytes: &[u8]) -> Result<PathBuf, String> {
    // Validate before writing anything into the persistent torrent store.
    inspect_torrent_bytes(bytes)?;
    let directory = torrent_store_directory(state)?;
    let digest = format!("{:x}", Sha256::digest(bytes));
    let path = directory.join(format!("{digest}.torrent"));
    if !path.is_file() {
        fs::write(&path, bytes).map_err(|error| error.to_string())?;
    }
    Ok(path)
}

fn is_managed_torrent_metadata_path(state: &AppState, path: &Path) -> bool {
    let Ok(directory) = torrent_store_directory(state) else {
        return false;
    };
    let valid_extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("torrent"));
    valid_extension && path.parent() == Some(directory.as_path())
}

fn validated_torrent_metadata_path(state: &AppState, value: Option<String>) -> Option<PathBuf> {
    let path = PathBuf::from(value?);
    (path.is_file() && is_managed_torrent_metadata_path(state, &path)).then_some(path)
}

async fn materialize_torrent_metadata_file(
    state: &AppState,
    endpoint: Option<&aria2::Endpoint>,
    source: &str,
) -> Result<PathBuf, String> {
    let local = PathBuf::from(source);
    if local.is_file() {
        let bytes = fs::read(&local).map_err(|error| error.to_string())?;
        return persist_torrent_bytes(state, &bytes);
    }

    if (source.starts_with("http://") || source.starts_with("https://"))
        && source
            .split(['?', '#'])
            .next()
            .is_some_and(|value| value.to_ascii_lowercase().ends_with(".torrent"))
    {
        let bytes = fetch_torrent_file_bytes(state, source).await?;
        return persist_torrent_bytes(state, &bytes);
    }

    if !source.starts_with("magnet:") {
        return Err("not_a_torrent".to_owned());
    }

    let endpoint = endpoint.ok_or_else(|| "aria2_endpoint_required".to_owned())?;
    let directory = torrent_store_directory(state)?;
    diagnostic_log(
        state,
        "INFO",
        "aria2.metadata_save_started",
        &format!("directory={}", directory.display()),
    );
    let path = endpoint
        .save_magnet_metadata(
            source,
            &directory,
            |elapsed, status, connections, seeders, total, completed, info_hash| {
                diagnostic_log(
                    state,
                    "INFO",
                    "aria2.metadata_save_progress",
                    &format!(
                        "elapsed={elapsed}s status={status} connections={connections} seeders={seeders} total={total} completed={completed} info_hash={}",
                        info_hash.unwrap_or("none")
                    ),
                );
            },
        )
        .await?;
    diagnostic_log(
        state,
        "INFO",
        "aria2.metadata_saved",
        &format!("path={} bytes={}", path.display(), fs::metadata(&path).map(|m| m.len()).unwrap_or(0)),
    );
    Ok(path)
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
struct Aria2RpcSettingsStatus {
    enabled: bool,
    auto_start: bool,
    configured_port: Option<u16>,
    connected: bool,
    active_port: Option<u16>,
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
    release_url: String,
    release_name: String,
    release_notes: String,
    published_at: String,
}

fn version_numbers(value: &str) -> Vec<u64> {
    value
        .trim()
        .trim_start_matches(['v', 'V'])
        .split('.')
        .map(|part| {
            part.split(|character: char| !character.is_ascii_digit())
                .next()
                .unwrap_or("0")
        })
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect()
}

async fn resolve_thumbnail_internal(state: &AppState, url: &str) -> Result<Option<String>, String> {
    let (proxy, dns) = {
        let settings = state.settings.lock().map_err(|error| error.to_string())?;
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
    };

    let (proxy_url, proxy_username, proxy_password) = proxy.unwrap_or_default();
    let network = thumbnail_cache::ThumbnailNetwork {
        proxy_url,
        proxy_username,
        proxy_password,
        dns_servers: dns,
    };

    let cache_root = state
        .queue_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("cache");
    match thumbnail_cache::resolve(&cache_root, &network, url).await {
        Ok(Some(result)) => {
            state.diagnostics.record(
                if result.cache_hit {
                    "thumbnail.cache_hit"
                } else {
                    "thumbnail.cached"
                },
                "INFO",
                None,
                None,
                serde_json::json!({
                    "bytes": result.bytes,
                    "contentHashPrefix": result.content_hash.chars().take(12).collect::<String>(),
                    "cacheVersion": 3
                }),
            );
            Ok(Some(result.data_url))
        }
        Ok(None) => Ok(None),
        Err(error) => {
            state.diagnostics.record(
                "thumbnail.cache_failed",
                "WARN",
                None,
                None,
                serde_json::json!({
                    "error": error,
                    "urlStored": false
                }),
            );
            Ok(None)
        }
    }
}

#[tauri::command]
async fn resolve_thumbnail(
    state: State<'_, AppState>,
    url: String,
) -> Result<Option<String>, String> {
    resolve_thumbnail_internal(&state, &url).await
}

fn prefetch_thumbnail(app: tauri::AppHandle, url: String) {
    if !matches!(
        url.split(':')
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "http" | "https"
    ) {
        return;
    }
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let _ = resolve_thumbnail_internal(&state, &url).await;
    });
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
    let release_url = payload["html_url"]
        .as_str()
        .and_then(valid_apocalipse_release_url)
        .unwrap_or("https://github.com/linuxhell/apocalipse-download-manager/releases")
        .to_owned();
    let release_name = payload["name"]
        .as_str()
        .unwrap_or_default()
        .trim()
        .to_owned();
    let release_notes = payload["body"]
        .as_str()
        .unwrap_or_default()
        .trim()
        .chars()
        .take(6000)
        .collect();
    let published_at = payload["published_at"]
        .as_str()
        .unwrap_or_default()
        .trim()
        .to_owned();
    Ok(AppUpdateStatus {
        update_available: version_numbers(&latest) > version_numbers(&current),
        current_version: current,
        latest_version: latest,
        release_url,
        release_name,
        release_notes,
        published_at,
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
    #[cfg(target_os = "linux")]
    if effective_player
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("AppImage"))
    {
        command.env("APPIMAGE_EXTRACT_AND_RUN", "1");
    }
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

fn configured_aria2(settings: &UserSettings) -> PathBuf {
    configured_tool(
        &settings.aria2_path,
        if cfg!(windows) {
            "aria2c.exe"
        } else {
            "aria2c"
        },
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExtractorKind {
    SevenZip,
    Rar,
    Unrar,
    Unar,
    Bsdtar,
    Tar,
}

fn extractor_kind(path: &Path) -> Option<ExtractorKind> {
    let name = path.file_stem()?.to_string_lossy().to_ascii_lowercase();
    if matches!(name.as_str(), "7z" | "7zz" | "7zr" | "7za") {
        Some(ExtractorKind::SevenZip)
    } else if matches!(name.as_str(), "winrar" | "rar") {
        Some(ExtractorKind::Rar)
    } else if name == "unrar" {
        Some(ExtractorKind::Unrar)
    } else if name == "unar" {
        Some(ExtractorKind::Unar)
    } else if name == "bsdtar" {
        Some(ExtractorKind::Bsdtar)
    } else if name == "tar" {
        Some(ExtractorKind::Tar)
    } else {
        None
    }
}

fn extractor_version(executable: &Path, kind: ExtractorKind) -> Option<String> {
    let stem = executable
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if stem == "winrar" {
        // WinRAR.exe without an archive is a GUI application and must never be
        // launched merely to populate the Tools status line.
        return executable.is_file().then(|| "WinRAR".to_owned());
    }
    let args: &[&str] = match kind {
        ExtractorKind::SevenZip => &[],
        ExtractorKind::Rar | ExtractorKind::Unrar => &["-?"],
        ExtractorKind::Unar => &["-v"],
        ExtractorKind::Bsdtar | ExtractorKind::Tar => &["--version"],
    };
    version_line(executable, args).or_else(|| executable.is_file().then(|| format!("{kind:?}")))
}

fn archive_name_without_extensions(path: &Path) -> String {
    let mut name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("archive")
        .to_owned();
    for suffix in [
        ".tar.gz", ".tar.bz2", ".tar.xz", ".tar.zst", ".tgz", ".tbz2", ".txz", ".zip", ".7z",
        ".rar", ".tar", ".gz", ".bz2", ".xz", ".zst", ".cab", ".arj", ".lha", ".lzh",
    ] {
        if name.to_ascii_lowercase().ends_with(suffix) {
            name.truncate(name.len() - suffix.len());
            break;
        }
    }
    if name.trim().is_empty() {
        "archive".to_owned()
    } else {
        name
    }
}

fn is_archive_file_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    [
        ".zip", ".7z", ".rar", ".tar", ".tar.gz", ".tgz", ".tar.bz2", ".tbz2", ".tar.xz", ".txz",
        ".tar.zst", ".gz", ".bz2", ".xz", ".zst", ".cab", ".arj", ".lha", ".lzh",
    ]
    .iter()
    .any(|suffix| lower.ends_with(suffix))
}

fn extraction_args(kind: ExtractorKind, archive: &Path, destination: &Path) -> Vec<String> {
    let archive = archive.to_string_lossy().into_owned();
    let destination = destination.to_string_lossy().into_owned();
    match kind {
        ExtractorKind::SevenZip => {
            vec!["x".into(), archive, format!("-o{destination}"), "-y".into()]
        }
        ExtractorKind::Rar | ExtractorKind::Unrar => vec![
            "x".into(),
            "-o+".into(),
            "-y".into(),
            archive,
            format!("{destination}{}", std::path::MAIN_SEPARATOR),
        ],
        ExtractorKind::Unar => vec!["-f".into(), "-o".into(), destination, archive],
        ExtractorKind::Bsdtar | ExtractorKind::Tar => {
            vec!["-xf".into(), archive, "-C".into(), destination]
        }
    }
}

fn archive_member_is_safe(name: &str) -> bool {
    let normalized = name.replace('\\', "/");
    if normalized.trim().is_empty() || normalized.starts_with('/') {
        return false;
    }
    let path = Path::new(&normalized);
    path.components().all(|component| {
        matches!(
            component,
            std::path::Component::Normal(_) | std::path::Component::CurDir
        )
    })
}

fn collect_lsar_names(value: &serde_json::Value, output: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(name) = map.get("XADFileName").and_then(|value| value.as_str()) {
                output.push(name.to_owned());
            }
            for value in map.values() {
                collect_lsar_names(value, output);
            }
        }
        serde_json::Value::Array(items) => {
            for value in items {
                collect_lsar_names(value, output);
            }
        }
        _ => {}
    }
}

fn list_archive_members(
    executable: &Path,
    kind: ExtractorKind,
    archive: &Path,
) -> Result<Vec<String>, String> {
    let mut command = match kind {
        ExtractorKind::Unar => {
            let sibling = executable
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(if cfg!(windows) { "lsar.exe" } else { "lsar" });
            if !sibling.is_file() {
                return Err("archive_listing_tool_missing:lsar".to_owned());
            }
            let mut command = Command::new(sibling);
            command.arg("-j").arg(archive);
            command
        }
        _ => {
            let mut command = Command::new(executable);
            match kind {
                ExtractorKind::SevenZip => {
                    command.args(["l", "-slt"]).arg(archive);
                }
                ExtractorKind::Rar | ExtractorKind::Unrar => {
                    command.args(["lb"]).arg(archive);
                }
                ExtractorKind::Bsdtar | ExtractorKind::Tar => {
                    command.args(["-tf"]).arg(archive);
                }
                ExtractorKind::Unar => unreachable!(),
            }
            command
        }
    };
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let output = command.output().map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err("archive_listing_failed".to_owned());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut names = match kind {
        ExtractorKind::SevenZip => {
            let mut members = Vec::new();
            let mut inside_members = false;
            for line in text.lines() {
                if line.trim() == "----------" {
                    inside_members = true;
                    continue;
                }
                if !inside_members {
                    continue;
                }
                if let Some(name) = line
                    .strip_prefix("Path = ")
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                {
                    members.push(name.to_owned());
                }
            }
            members
        }
        ExtractorKind::Unar => {
            let value: serde_json::Value =
                serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())?;
            let mut names = Vec::new();
            collect_lsar_names(&value, &mut names);
            names
        }
        _ => text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect(),
    };
    names.sort();
    names.dedup();
    if names.is_empty() {
        return Err("archive_has_no_members".to_owned());
    }
    if names.iter().any(|name| !archive_member_is_safe(name)) {
        return Err("archive_contains_unsafe_path".to_owned());
    }
    Ok(names)
}

fn move_tree(source: &Path, destination: &Path) -> Result<(), String> {
    if source.is_dir() {
        fs::create_dir_all(destination).map_err(|error| error.to_string())?;
        for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            move_tree(&entry.path(), &destination.join(entry.file_name()))?;
        }
        fs::remove_dir(source).map_err(|error| error.to_string())?;
    } else {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        if destination.exists() {
            if destination.is_dir() {
                return Err("archive_destination_conflict".to_owned());
            }
            fs::remove_file(destination).map_err(|error| error.to_string())?;
        }
        fs::rename(source, destination)
            .or_else(|_| {
                fs::copy(source, destination)
                    .map(|_| ())
                    .and_then(|_| fs::remove_file(source))
            })
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn extract_archive_safely(settings: &UserSettings, archive: &Path) -> Result<PathBuf, String> {
    if !archive.is_file()
        || !archive
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(is_archive_file_name)
    {
        return Err("not_archive_file".to_owned());
    }
    let executable = settings
        .extractor_path
        .clone()
        .ok_or_else(|| "archive_extractor_not_configured".to_owned())?;
    let kind =
        extractor_kind(&executable).ok_or_else(|| "unsupported_archive_extractor".to_owned())?;
    if !executable.is_file() {
        return Err("archive_extractor_not_found".to_owned());
    }
    let _members = list_archive_members(&executable, kind, archive)?;
    let parent = archive.parent().unwrap_or_else(|| Path::new("."));
    let staging = parent.join(format!(
        ".apocalipse-extract-{}",
        uuid::Uuid::new_v4().simple()
    ));
    fs::create_dir_all(&staging).map_err(|error| error.to_string())?;
    let mut command = Command::new(&executable);
    command.args(extraction_args(kind, archive, &staging));
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let output = command.output().map_err(|error| error.to_string())?;
    if !output.status.success() {
        let _ = fs::remove_dir_all(&staging);
        let detail = String::from_utf8_lossy(if output.stderr.is_empty() {
            &output.stdout
        } else {
            &output.stderr
        })
        .trim()
        .to_owned();
        return Err(if detail.is_empty() {
            "archive_extraction_failed".to_owned()
        } else {
            format!("archive_extraction_failed:{detail}")
        });
    }
    let entries = fs::read_dir(&staging)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    if entries.is_empty() {
        let _ = fs::remove_dir_all(&staging);
        return Err("archive_extraction_empty".to_owned());
    }
    let destination = if entries.len() == 1 && entries[0].is_dir() {
        let root_name = entries[0]
            .file_name()
            .ok_or_else(|| "archive_root_name_missing".to_owned())?;
        parent.join(root_name)
    } else {
        parent.join(archive_name_without_extensions(archive))
    };
    if entries.len() == 1 && entries[0].is_dir() {
        move_tree(&entries[0], &destination)?;
    } else {
        fs::create_dir_all(&destination).map_err(|error| error.to_string())?;
        for entry in entries {
            let entry_name = entry
                .file_name()
                .ok_or_else(|| "archive_entry_name_missing".to_owned())?;
            let target = destination.join(entry_name);
            move_tree(&entry, &target)?;
        }
    }
    let _ = fs::remove_dir_all(&staging);
    Ok(destination)
}

fn maybe_auto_extract_completed(app: &tauri::AppHandle, id: DownloadId) {
    let state = app.state::<AppState>();
    let task = state
        .queue
        .lock()
        .ok()
        .and_then(|queue| queue.iter().find(|task| task.id == id).cloned());
    let Some(task) =
        task.filter(|task| task.auto_extract && task.state == DownloadState::Completed)
    else {
        return;
    };
    let settings = match state.settings.lock() {
        Ok(settings) => settings.clone(),
        Err(_) => return,
    };
    let destination = task.destination.clone();
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let result = extract_archive_safely(&settings, &destination);
        let state = app.state::<AppState>();
        match &result {
            Ok(path) => diagnostic_log(
                &state,
                "INFO",
                "archive.auto_extract_completed",
                &format!("task={id} output={}", path.display()),
            ),
            Err(error) => diagnostic_log(
                &state,
                "ERROR",
                "archive.auto_extract_failed",
                &format!("task={id} error={error}"),
            ),
        }
    });
}

async fn aria2_endpoint(state: &AppState, force_start: bool) -> Result<aria2::Endpoint, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    if !settings.aria2_rpc_enabled {
        return Err("aria2_rpc_disabled".to_owned());
    }
    let executable = configured_aria2(&settings);
    let runtime_root = state
        .queue_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("aria2-rpc");
    let mut spawned = None;
    let endpoint = {
        let mut runtime = state
            .aria2_runtime
            .lock()
            .map_err(|error| error.to_string())?;
        let reuse = runtime.as_mut().is_some_and(aria2::Runtime::is_running);
        if !reuse {
            if !force_start && !settings.aria2_rpc_auto_start {
                return Err("aria2_rpc_not_running".to_owned());
            }
            if let Some(current) = runtime.as_mut() {
                current.terminate();
            }
            *runtime = Some(aria2::Runtime::spawn(
                &executable,
                &runtime_root,
                settings.aria2_rpc_port,
                &settings.aria2_rpc_secret,
            )?);
            if let Some(current) = runtime.as_ref() {
                spawned = Some((current.pid(), current.port()));
            }
        }
        runtime
            .as_ref()
            .map(aria2::Runtime::endpoint)
            .ok_or_else(|| "aria2_rpc_runtime_missing".to_owned())?
    };
    let runtime_spawned = spawned.is_some();
    if let Some((pid, port)) = spawned {
        diagnostic_log(
            state,
            "INFO",
            "aria2.runtime_spawned",
            &format!(
                "pid={pid} parent_pid={} port={port} binary=aria2c backend=classic listen_port=6881-6999 dht_listen_port=6881-6999 stop_with_parent=true",
                std::process::id()
            ),
        );
    }
    if runtime_spawned {
        endpoint.wait_ready().await?;
        endpoint
            .set_global_download_limit(settings.global_bandwidth_limit)
            .await?;
    }
    Ok(endpoint)
}

fn stop_aria2_runtime(state: &AppState) {
    if let Ok(mut runtime) = state.aria2_runtime.lock() {
        if let Some(runtime) = runtime.as_mut() {
            let pid = runtime.pid();
            let port = runtime.port();
            runtime.terminate();
            diagnostic_log(
                state,
                "INFO",
                "aria2.runtime_stopped",
                &format!("pid={pid} port={port}"),
            );
        }
        *runtime = None;
    }
}

fn running_aria2_endpoint(state: &AppState) -> Option<aria2::Endpoint> {
    let mut runtime = state.aria2_runtime.lock().ok()?;
    let runtime = runtime.as_mut()?;
    if !runtime.is_running() {
        return None;
    }
    Some(runtime.endpoint())
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
    #[cfg(target_os = "linux")]
    if executable
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("AppImage"))
    {
        command.env("APPIMAGE_EXTRACT_AND_RUN", "1");
    }
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

#[derive(Clone, Deserialize, Serialize)]
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
    source: String,
    received: u64,
    total: Option<u64>,
    recording: bool,
    prompt_for_destination: bool,
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
    #[serde(default)]
    prompt_for_destination: bool,
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
    is_live: bool,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    thumbnail: Option<String>,
    #[serde(default)]
    audio_url: Option<String>,
    #[serde(default)]
    expected_size: Option<u64>,
    cookie_header: Option<String>,
    user_agent: Option<String>,
    request_method: Option<String>,
    request_body: Option<String>,
    request_content_type: Option<String>,
    #[serde(default)]
    torrent_metadata_path: Option<String>,
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
    [
        ("facebook.com", "facebook.com"),
        ("instagram.com", "instagram.com"),
        ("tiktok.com", "tiktok.com"),
        ("twitch.tv", "twitch.tv"),
        ("twitch.com", "twitch.tv"),
        ("bilibili.com", "bilibili.com"),
        ("b23.tv", "bilibili.com"),
        ("bili.tv", "bilibili.com"),
    ]
    .into_iter()
    .find_map(|(source_domain, cookie_domain)| {
        (host == source_domain || host.ends_with(&format!(".{source_domain}")))
            .then_some(cookie_domain)
    })
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

/// Quotes a value the way yt-dlp's config-file parser (POSIX shlex) expects,
/// so it survives as a single argument even if it contains spaces or quotes.
fn shell_config_quote(value: &str) -> String {
    if !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || matches!(character, '-' | '_' | '.' | '/' | ':' | '@')
        })
    {
        return value.to_owned();
    }
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('\'');
    for character in value.chars() {
        if character == '\'' {
            quoted.push_str("'\\''");
        } else {
            quoted.push(character);
        }
    }
    quoted.push('\'');
    quoted
}

/// Writes yt-dlp `--username`/`--password` as a `--config-location` file
/// instead of passing the secret on argv, where it would be visible to other
/// local users via `ps`/`/proc/<pid>/cmdline` for the life of the process.
fn write_yt_dlp_credential_config(
    path: &Path,
    username: &str,
    password: &str,
) -> Result<(), String> {
    let contents = format!(
        "--username {}\n--password {}\n",
        shell_config_quote(username),
        shell_config_quote(password)
    );
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|error| error.to_string())?;
    file.write_all(contents.as_bytes())
        .map_err(|error| error.to_string())
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
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            socket.connect("1.1.1.1:80")?;
            socket.local_addr()
        })
        .ok()
        .map(|address| address.ip())
        .filter(|address| !address.is_unspecified() && !address.is_loopback())
        .unwrap_or_else(|| "127.0.0.1".parse().expect("valid loopback"))
}

fn active_network_ip() -> Option<std::net::IpAddr> {
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            socket.connect("1.1.1.1:80")?;
            socket.local_addr()
        })
        .ok()
        .map(|address| address.ip())
        .filter(|address| !address.is_unspecified() && !address.is_loopback())
}

fn reconnect_active_downloads_after_network_change(
    app: &tauri::AppHandle,
    previous: Option<std::net::IpAddr>,
    current: Option<std::net::IpAddr>,
) {
    let state = app.state::<AppState>();
    let active = state
        .workers
        .lock()
        .map(|mut workers| workers.drain().collect::<Vec<_>>())
        .unwrap_or_default();
    let mut ids = active.iter().map(|(id, _)| *id).collect::<HashSet<_>>();
    for (_, cancel) in active {
        let _ = cancel.send(());
    }
    if !ids.is_empty() {
        // aria2 restores unfinished session entries with the same GID.
        // Keep ADM's task->GID mapping so the restarted runtime can reconnect
        // to the restored transfer. If a GID was not persisted by the engine,
        // run_aria2_download detects it as stale and safely recreates the task.
        stop_aria2_runtime(&state);
    }
    if current.is_some() {
        if let Ok(queue) = state.queue.lock() {
            ids.extend(queue.iter().filter_map(|task| match &task.state {
                DownloadState::Failed { message } if message == "network_waiting_for_reconnect" => {
                    Some(task.id)
                }
                _ => None,
            }));
        }
    }
    if ids.is_empty() {
        return;
    }
    diagnostic_log(
        &state,
        "INFO",
        "network.interface_changed",
        &format!(
            "previous={} current={} tasks={}",
            previous.map_or_else(|| "offline".to_owned(), |ip| ip.to_string()),
            current.map_or_else(|| "offline".to_owned(), |ip| ip.to_string()),
            ids.len()
        ),
    );
    let resumed_app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(750));
        let state = resumed_app.state::<AppState>();
        if let Ok(mut queue) = state.queue.lock() {
            for task in queue.iter_mut().filter(|task| ids.contains(&task.id)) {
                if current.is_none() {
                    if task.state != DownloadState::Completed {
                        task.state = DownloadState::Failed {
                            message: "network_waiting_for_reconnect".to_owned(),
                        };
                        task.download_speed = Some(0);
                        task.upload_speed = Some(0);
                    }
                } else if matches!(
                    task.state,
                    DownloadState::Downloading
                        | DownloadState::Inspecting
                        | DownloadState::Paused
                        | DownloadState::Failed { .. }
                ) {
                    task.state = DownloadState::Queued;
                    task.download_speed = Some(0);
                    task.upload_speed = Some(0);
                }
            }
            let _ = save_queue(&state, &queue);
        }
        state.diagnostics.record(
            if current.is_some() {
                "network.reconnect_queued"
            } else {
                "network.waiting"
            },
            "INFO",
            None,
            None,
            serde_json::json!({"taskCount": ids.len()}),
        );
        if current.is_some() {
            start_next_queued(&resumed_app);
        }
    });
}

fn run_network_change_monitor(app: tauri::AppHandle) {
    let mut previous = active_network_ip();
    loop {
        std::thread::sleep(Duration::from_secs(2));
        let current = active_network_ip();
        if current != previous {
            reconnect_active_downloads_after_network_change(&app, previous, current);
            previous = current;
        }
    }
}

fn handle_link_connection<S: Read + Write>(app: &tauri::AppHandle, mut stream: S) {
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
    if headers.starts_with("GET /v1/link/capabilities?")
        || headers.starts_with("GET /v1/link/capabilities ")
    {
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
        let virtual_path = url::Url::parse(&format!("http://localhost{request_target}"))
            .ok()
            .and_then(|url| {
                url.query_pairs()
                    .find(|(key, _)| key == "path")
                    .map(|(_, value)| value.into_owned())
            })
            .unwrap_or_default();
        let allow_write = resolve_link_share(&settings, &virtual_path)
            .map(|(_, write)| write)
            .unwrap_or(false);
        let body = serde_json::to_string(&LinkCapabilities { allow_write })
            .unwrap_or_else(|_| "{\"allowWrite\":false}".to_owned());
        bridge_response(&mut stream, "200 OK", None, &body);
        return;
    }
    if headers.starts_with("DELETE /v1/link/item?") {
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
        let resolved = path
            .ok_or_else(|| "invalid_path".to_owned())
            .and_then(|value| {
                let is_root = value.trim_matches('/').split('/').count() <= 2;
                let (path, allow_write) = resolve_link_share(&settings, &value)?;
                if !allow_write || is_root {
                    return Err("link_write_not_allowed".to_owned());
                }
                Ok(path)
            });
        drop(settings);
        match resolved.and_then(|value| remove_link_path(&value.to_string_lossy())) {
            Ok(()) => bridge_response(&mut stream, "200 OK", None, "{\"ok\":true}"),
            Err(_) => bridge_response(
                &mut stream,
                "400 Bad Request",
                None,
                "{\"error\":\"delete_failed\"}",
            ),
        }
        return;
    }
    if headers.starts_with("PUT /v1/link/directory?") {
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
        let Some((path, true)) = path.and_then(|value| resolve_link_share(&settings, &value).ok())
        else {
            bridge_response(&mut stream, "400 Bad Request", None, "");
            return;
        };
        drop(settings);
        match fs::create_dir_all(path) {
            Ok(()) => bridge_response(&mut stream, "200 OK", None, "{\"ok\":true}"),
            Err(_) => bridge_response(&mut stream, "403 Forbidden", None, ""),
        }
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
        let Some((path, true)) = path.and_then(|value| resolve_link_share(&settings, &value).ok())
        else {
            bridge_response(&mut stream, "400 Bad Request", None, "");
            return;
        };
        drop(settings);
        if let Some(parent) = path.parent() {
            if fs::create_dir_all(parent).is_err() {
                bridge_response(&mut stream, "403 Forbidden", None, "");
                return;
            }
        }
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
    if headers.starts_with("POST /v1/link/auth ") {
        let request = serde_json::from_slice::<LinkAuthRequest>(&buffer[header_end + 4..]);
        let Ok(mut request) = request else {
            bridge_response(
                &mut stream,
                "400 Bad Request",
                None,
                "{\"error\":\"invalid_request\"}",
            );
            return;
        };
        let authentication = verify_system_account(&request.username, &request.password);
        request.password.zeroize();
        match authentication {
            Ok(()) => {
                let state = app.state::<AppState>();
                let token = match state.settings.lock() {
                    Ok(settings) => settings.link_password.clone(),
                    Err(_) => return,
                };
                let body = serde_json::to_string(&LinkAuthResponse { token })
                    .unwrap_or_else(|_| "{\"error\":\"serialization_failed\"}".to_owned());
                bridge_response(&mut stream, "200 OK", None, &body);
            }
            Err(_) => {
                std::thread::sleep(Duration::from_millis(750));
                bridge_response(
                    &mut stream,
                    "401 Unauthorized",
                    None,
                    "{\"error\":\"authentication_failed\"}",
                );
            }
        }
        return;
    }
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
        let Some((path, _)) = path.and_then(|value| resolve_link_share(&settings, &value).ok())
        else {
            bridge_response(&mut stream, "400 Bad Request", None, "");
            return;
        };
        drop(settings);
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
    match list_shared_link_directory(&settings, &request.path)
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

fn run_link_server(app: tauri::AppHandle, listener: TcpListener, tls_config: Arc<ServerConfig>) {
    for stream in listener.incoming().flatten() {
        let app = app.clone();
        let tls_config = tls_config.clone();
        let _ = std::thread::Builder::new()
            .name("apocalipse-link-client".into())
            .spawn(move || {
                let Ok(connection) = ServerConnection::new(tls_config) else {
                    return;
                };
                handle_link_connection(&app, StreamOwned::new(connection, stream));
            });
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileHostResolution {
    url: String,
    adapted: bool,
    adapter: Option<String>,
}

fn supported_file_host(host: &str) -> Option<&'static str> {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    if host == "gofile.io" || host.ends_with(".gofile.io") {
        Some("gofile")
    } else if host == "mediafire.com" || host.ends_with(".mediafire.com") {
        Some("mediafire")
    } else if host == "datanodes.to" || host.ends_with(".datanodes.to") {
        Some("datanodes")
    } else if host == "archive.org" || host.ends_with(".archive.org") {
        Some("archive")
    } else {
        None
    }
}

fn html_attribute_urls(html: &str, attribute: &str) -> Vec<String> {
    let mut urls = Vec::new();
    for quote in ['"', '\''] {
        let needle = format!("{attribute}={quote}");
        let mut rest = html;
        while let Some(start) = rest.find(&needle) {
            let value = &rest[start + needle.len()..];
            let Some(end) = value.find(quote) else {
                break;
            };
            let candidate = value[..end].replace("&amp;", "&").replace("&#38;", "&");
            if !candidate.trim().is_empty() {
                urls.push(candidate);
            }
            rest = &value[end + quote.len_utf8()..];
        }
    }
    urls
}

fn file_host_candidate_trusted(adapter: &str, host: &str) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    match adapter {
        "gofile" => host == "gofile.io" || host.ends_with(".gofile.io"),
        "mediafire" => host == "mediafire.com" || host.ends_with(".mediafire.com"),
        "datanodes" => host == "datanodes.to" || host.ends_with(".datanodes.to"),
        "archive" => host == "archive.org" || host.ends_with(".archive.org"),
        _ => false,
    }
}

fn file_host_candidate_score(adapter: &str, value: &url::Url) -> i32 {
    let path = value.path().to_ascii_lowercase();
    let host = value.host_str().unwrap_or_default().to_ascii_lowercase();
    let mut score = 0;
    if path.contains("/download") {
        score += 60;
    }
    if path.contains("/file/") || path.contains("/files/") {
        score += 20;
    }
    if [
        ".zip", ".7z", ".rar", ".tar", ".gz", ".xz", ".zst", ".iso", ".exe", ".msi", ".dmg",
        ".pkg", ".deb", ".rpm", ".mp4", ".mkv", ".pdf",
    ]
    .iter()
    .any(|extension| path.ends_with(extension))
    {
        score += 50;
    }
    match adapter {
        "mediafire" if host.contains("download") || host.contains("mediafire") => score += 35,
        "gofile" if host.contains("gofile") => score += 25,
        "datanodes" if host.contains("datanodes") => score += 25,
        "archive" if path.starts_with("/download/") => score += 70,
        _ => {}
    }
    score
}

async fn resolve_file_host_url_internal(url: &str) -> Result<FileHostResolution, String> {
    let parsed = url::Url::parse(url).map_err(|_| "invalid_url".to_owned())?;
    let host = parsed.host_str().ok_or_else(|| "invalid_url".to_owned())?;
    let Some(adapter) = supported_file_host(host) else {
        return Ok(FileHostResolution {
            url: url.to_owned(),
            adapted: false,
            adapter: None,
        });
    };
    if adapter == "archive" && parsed.path().starts_with("/download/") {
        return Ok(FileHostResolution {
            url: url.to_owned(),
            adapted: false,
            adapter: Some(adapter.to_owned()),
        });
    }
    let response = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 ApocalipseDownloadManager/0.4")
        .redirect(reqwest::redirect::Policy::limited(5))
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| error.to_string())?
        .get(parsed.clone())
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    let final_url = response.url().clone();
    let html = response.text().await.map_err(|error| error.to_string())?;
    let mut best: Option<(i32, String)> = None;
    for value in html_attribute_urls(&html, "href")
        .into_iter()
        .chain(html_attribute_urls(&html, "data-url"))
        .chain(html_attribute_urls(&html, "data-download"))
    {
        let Ok(candidate) = final_url.join(value.trim()) else {
            continue;
        };
        if !matches!(candidate.scheme(), "http" | "https") {
            continue;
        }
        let candidate_host = candidate.host_str().unwrap_or_default();
        if !file_host_candidate_trusted(adapter, candidate_host) {
            continue;
        }
        let score = file_host_candidate_score(adapter, &candidate);
        if score >= 50 && best.as_ref().is_none_or(|(current, _)| score > *current) {
            best = Some((score, candidate.to_string()));
        }
    }
    let resolved = best
        .map(|(_, value)| value)
        .unwrap_or_else(|| url.to_owned());
    Ok(FileHostResolution {
        adapted: resolved != url,
        url: resolved,
        adapter: Some(adapter.to_owned()),
    })
}

#[tauri::command]
async fn resolve_file_host_url(url: String) -> Result<FileHostResolution, String> {
    resolve_file_host_url_internal(&url).await
}

#[tauri::command]
fn inspect_url(url: String) -> Result<PlanResponse, String> {
    let capabilities = Capabilities {
        aria2: true,
        yt_dlp: true,
        n_m3u8dl_re: true,
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
            effective_credential_for_download(&settings, &url, None),
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
        .args([
            "--ignore-config",
            "--retries",
            "30",
            "--extractor-retries",
            "10",
            "--retry-sleep",
            "2",
            "--retry-sleep",
            "extractor:2",
            "--socket-timeout",
            "30",
        ])
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
    let credential_config = credential.as_ref().and_then(|credential| {
        let path = state
            .queue_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(format!("inspect-credentials-{}.txt", uuid::Uuid::new_v4()));
        write_yt_dlp_credential_config(&path, &credential.username, &credential.password)
            .ok()
            .map(|_| path)
    });
    if let Some(path) = credential_config.as_ref() {
        command.arg("--config-location").arg(path);
    }
    if let Some(mut credential) = credential {
        credential.password.zeroize();
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
    if let Some(path) = credential_config {
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
    let is_live = value
        .get("is_live")
        .and_then(|item| item.as_bool())
        .unwrap_or(false)
        || value
            .get("live_status")
            .and_then(|item| item.as_str())
            .is_some_and(|status| status == "is_live");
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
        is_live,
        suggested_file_name: format!("{safe_title}.mp4"),
        formats,
    })
}

fn load_queue(path: &Path) -> Vec<DownloadTask> {
    let mut queue: Vec<DownloadTask> = fs::read(path)
        .ok()
        .and_then(|data| serde_json::from_slice(&data).ok())
        .unwrap_or_default();
    let mut recovered = false;
    for task in &mut queue {
        if matches!(
            task.state,
            DownloadState::Downloading | DownloadState::Inspecting | DownloadState::Verifying
        ) {
            // Workers do not survive a process restart. Keep the partial data
            // and persisted aria2 GID, but present the task as paused
            // until the user resumes it (or the scheduler explicitly queues it).
            task.state = DownloadState::Paused;
            task.download_speed = Some(0);
            task.upload_speed = Some(0);
            recovered = true;
        }
    }
    if recovered {
        if let Ok(data) = serde_json::to_vec_pretty(&queue) {
            let _ = fs::write(path, data);
        }
    }
    queue
}

fn restored_aria2_task_map(queue: &[DownloadTask]) -> HashMap<DownloadId, String> {
    queue
        .iter()
        .filter(|task| task.state != DownloadState::Completed)
        .filter_map(|task| task.aria2_gid.clone().map(|gid| (task.id, gid)))
        .collect()
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

fn load_settings(path: &Path) -> Result<UserSettings, String> {
    let mut settings: UserSettings = fs::read(path)
        .ok()
        .and_then(|data| serde_json::from_slice(&data).ok())
        .unwrap_or_default();
    for share in &mut settings.link_shares {
        if let Ok(metadata) = fs::metadata(&share.path) {
            share.directory = metadata.is_dir();
        }
    }
    hydrate_and_migrate_secrets(&mut settings)?;
    write_settings(path, &settings)?;
    Ok(settings)
}

fn migrate_secret(account: &str, in_memory: &mut String) -> Result<(), String> {
    if let Some(secret) = vault_load(account)? {
        in_memory.zeroize();
        *in_memory = secret;
        return Ok(());
    }
    vault_store_verified(account, in_memory)
}

fn hydrate_and_migrate_secrets(settings: &mut UserSettings) -> Result<(), String> {
    migrate_secret(VAULT_BRIDGE_TOKEN, &mut settings.bridge_token)?;
    migrate_secret(VAULT_LINK_PASSWORD, &mut settings.link_password)?;
    if settings.link_password.len() < 24 {
        settings.link_password.zeroize();
        settings.link_password = default_link_password();
        vault_store_verified(VAULT_LINK_PASSWORD, &settings.link_password)?;
    }

    if let Some(mut legacy) = settings.proxy_password.take() {
        if vault_load(VAULT_PROXY_PASSWORD)?.is_none() {
            vault_store_verified(VAULT_PROXY_PASSWORD, &legacy)?;
        }
        legacy.zeroize();
    }
    settings.proxy_password = vault_load(VAULT_PROXY_PASSWORD)?;

    for credential in &mut settings.legacy_website_credentials {
        let account = website_vault_account(&credential.host);
        let secret = vault_load(&account)?.unwrap_or_else(|| credential.password.clone());
        if !secret.is_empty()
            && !settings
                .host_rules
                .iter()
                .any(|rule| rule.pattern == credential.host)
        {
            vault_store_verified(&host_rule_vault_account(&credential.host), &secret)?;
            settings.host_rules.push(HostRule {
                pattern: credential.host.clone(),
                username: Some(credential.username.clone()),
                password: secret,
                user_agent: None,
                connections: None,
                bandwidth_limit: None,
            });
        }
        let _ = vault_delete(&account);
        credential.password.zeroize();
    }
    settings.legacy_website_credentials.clear();
    for rule in &mut settings.host_rules {
        let account = host_rule_vault_account(&rule.pattern);
        if !rule.password.is_empty() && vault_load(&account)?.is_none() {
            vault_store_verified(&account, &rule.password)?;
        }
        rule.password.zeroize();
        rule.password = vault_load(&account)?.unwrap_or_default();
    }
    Ok(())
}

fn write_settings(path: &Path, settings: &UserSettings) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let data = serde_json::to_vec_pretty(settings).map_err(|error| error.to_string())?;
    fs::write(path, data).map_err(|error| error.to_string())
}

fn save_settings(state: &AppState, settings: &UserSettings) -> Result<(), String> {
    write_settings(&state.settings_path, settings)
}

fn update_capacity_estimate(
    estimate: &mut HttpCapacityEstimate,
    observed: u64,
    decay_after_low_runs: u32,
) {
    if observed == 0 {
        return;
    }
    if estimate.bytes_per_second == 0 || observed > estimate.bytes_per_second {
        estimate.bytes_per_second = observed;
        estimate.low_runs = 0;
        return;
    }
    if observed >= estimate.bytes_per_second.saturating_mul(85) / 100 {
        estimate.low_runs = 0;
        return;
    }
    estimate.low_runs = estimate.low_runs.saturating_add(1);
    if estimate.low_runs >= decay_after_low_runs.max(1) {
        estimate.bytes_per_second =
            (estimate.bytes_per_second.saturating_mul(95) / 100).max(observed);
        estimate.low_runs = 0;
    }
}

fn learn_http_capacity(state: &AppState, host: Option<&str>, observed: u64) {
    if observed < 1024 * 1024 {
        return;
    }
    let mut settings = match state.settings.lock() {
        Ok(settings) => settings,
        Err(_) => return,
    };
    let previous_global = settings.http_global_capacity.bytes_per_second;
    let previous_host = host
        .and_then(|host| settings.http_host_capacities.get(host))
        .map(|estimate| estimate.bytes_per_second)
        .unwrap_or(0);

    if let Some(host) = host {
        if settings.http_host_capacities.contains_key(host)
            || settings.http_host_capacities.len() < 256
        {
            let estimate = settings
                .http_host_capacities
                .entry(host.to_owned())
                .or_default();
            update_capacity_estimate(estimate, observed, 4);
        }
    }

    if observed > settings.http_global_capacity.bytes_per_second {
        settings.http_global_capacity.bytes_per_second = observed;
        settings.http_global_capacity.low_runs = 0;
    } else if previous_global > 0 && previous_host >= previous_global.saturating_mul(85) / 100 {
        // Only a host that historically came close to the user's global best
        // is allowed to vote that the actual access link became slower.
        update_capacity_estimate(&mut settings.http_global_capacity, observed, 8);
    }

    let global = settings.http_global_capacity.bytes_per_second;
    let host_capacity = host
        .and_then(|host| settings.http_host_capacities.get(host))
        .map(|estimate| estimate.bytes_per_second)
        .unwrap_or(0);
    if save_settings(state, &settings).is_ok() {
        state.diagnostics.record(
            "http.capacity_learned",
            "INFO",
            None,
            None,
            serde_json::json!({
                "observedSustainedBytesPerSecond": observed,
                "globalCapacityBytesPerSecond": global,
                "host": host,
                "hostCapacityBytesPerSecond": host_capacity
            }),
        );
    }
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
        "token:",
        "token=",
        "secret:",
        "secret=",
        "rpc-secret",
        "signature:",
        "signature=",
        "sig:",
        "sig=",
        "api-key",
        "apikey",
    ]
    .iter()
    .any(|marker| lowered.contains(marker))
    {
        return "<redacted-sensitive-line>".to_owned();
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

fn read_sanitized_log_tail(path: &Path, max_bytes: usize) -> Option<Vec<u8>> {
    let bytes = fs::read(path).ok()?;
    let start = bytes.len().saturating_sub(max_bytes);
    let text = String::from_utf8_lossy(&bytes[start..]);
    Some(
        text.lines()
            .map(sanitize_log_detail)
            .collect::<Vec<_>>()
            .join("\n")
            .into_bytes(),
    )
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
    if fs::metadata(&state.log_path).is_ok_and(|metadata| metadata.len() > 8 * 1024 * 1024) {
        let rotated = state.log_path.with_extension("log.1");
        let _ = fs::remove_file(&rotated);
        let _ = fs::rename(&state.log_path, rotated);
    }
    let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let local_timestamp = chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&state.log_path)
    {
        let record = serde_json::json!({
            "timestamp": timestamp,
            "localTimestamp": local_timestamp,
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
        task.state = DownloadState::Inspecting;
        task.download_speed = Some(0);
        task.upload_speed = Some(0);
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
    let request_host = host_from_url(&request.url);
    let mut previous_perf_rate: Option<u64> = None;
    let mut sustainable_peak = 0_u64;
    let mut completed_ok = false;
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
    let mut active_connections = 1_usize;
    let mut perf_last_at = Instant::now();
    let mut perf_last_bytes = 0_u64;
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
                        completed_ok = true;
                        diagnostic_log(&app.state::<AppState>(), "INFO", "http.completed", &format!("task={id}"));
                        update_task(&app, id, true, |task| {
                            task.state = DownloadState::Completed;
                            task.download_speed = Some(0);
                            task.upload_speed = Some(0);
                            task.completed_at = Some(epoch_seconds());
                        });
                        maybe_auto_extract_completed(&app, id);
                    },
                    Err(error) => {
                        diagnostic_log(&app.state::<AppState>(), "ERROR", "http.failed", &format!("task={id} error={error}"));
                        update_task(&app, id, true, |task| {
                            task.state = DownloadState::Failed { message: error.to_string() };
                            task.download_speed = Some(0);
                            task.upload_speed = Some(0);
                        });
                    },
                }
                break;
            }
            event = receiver.recv() => match event {
                Some(DownloadEvent::Started { resumed_at, total, connections, resume_supported }) => {
                    active_connections = connections;
                    perf_last_at = Instant::now();
                    perf_last_bytes = resumed_at;
                    diagnostic_log(&app.state::<AppState>(), "INFO", "http.mode", &format!("task={id} connections={connections} segmented={}", connections > 1));
                    app.state::<AppState>().diagnostics.record(
                        "http.transfer_started",
                        "INFO",
                        None,
                        Some(&id.to_string()),
                        serde_json::json!({
                            "resumedBytes": resumed_at,
                            "totalBytes": total,
                            "activeConnections": connections,
                            "resumeSupported": resume_supported,
                            "mode": if connections > 1 { "segmented" } else { "single" }
                        }),
                    );
                    update_task(&app, id, true, |task| {
                        task.state = DownloadState::Downloading;
                        task.received = resumed_at;
                        task.total = total;
                        task.download_speed = Some(0);
                        task.upload_speed = Some(0);
                        task.resume_supported = Some(resume_supported);
                    });
                },
                Some(DownloadEvent::Progress { received, total }) => {
                    update_task(&app, id, false, |task| {
                        task.received = received;
                        task.total = total;
                    });
                    let elapsed = perf_last_at.elapsed();
                    if elapsed >= Duration::from_millis(350) {
                        let elapsed_ms = elapsed.as_millis() as u64;
                        let interval_bytes = received.saturating_sub(perf_last_bytes);
                        let bytes_per_second = if elapsed_ms > 0 {
                            interval_bytes.saturating_mul(1000) / elapsed_ms
                        } else {
                            0
                        };
                        if let Some(previous) = previous_perf_rate {
                            sustainable_peak = sustainable_peak.max(previous.min(bytes_per_second));
                        }
                        previous_perf_rate = Some(bytes_per_second);
                        update_task(&app, id, false, |task| {
                            task.download_speed = Some(bytes_per_second);
                        });
                        app.state::<AppState>().diagnostics.record(
                            "http.performance_sample",
                            "INFO",
                            None,
                            Some(&id.to_string()),
                            serde_json::json!({
                                "receivedBytes": received,
                                "totalBytes": total,
                                "intervalBytes": interval_bytes,
                                "intervalMs": elapsed_ms,
                                "bytesPerSecond": bytes_per_second,
                                "displayBytesPerSecond": bytes_per_second,
                                "speedSource": "native-http",
                                "activeConnections": active_connections
                            }),
                        );
                        perf_last_at = Instant::now();
                        perf_last_bytes = received;
                    }
                },
                Some(DownloadEvent::Diagnostic { event, detail }) => {
                    if event == "http.connection_admission" {
                        if let Some(connections) = detail
                            .get("admittedConnections")
                            .and_then(serde_json::Value::as_u64)
                        {
                            active_connections = connections.max(1) as usize;
                        }
                    }
                    app.state::<AppState>().diagnostics.record(
                        event,
                        "INFO",
                        None,
                        Some(&id.to_string()),
                        detail,
                    );
                },
                Some(DownloadEvent::Completed { bytes }) => update_task(&app, id, true, |task| {
                    task.received = bytes;
                    task.total = Some(bytes);
                    task.download_speed = Some(0);
                    task.upload_speed = Some(0);
                    task.state = DownloadState::Completed;
                    task.completed_at = Some(epoch_seconds());
                }),
                None => break,
            }
        }
    }
    if completed_ok && sustainable_peak > 0 {
        learn_http_capacity(
            &app.state::<AppState>(),
            request_host.as_deref(),
            sustainable_peak,
        );
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
    let sources = engine.verified_sources(&request, &mirrors).await;
    engine.download_from_sources(request, sources, events).await
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

fn aria2_http_proxy_url(settings: &UserSettings) -> Option<String> {
    if !settings.proxy_enabled {
        return None;
    }
    let raw = settings.proxy_url.as_deref()?;
    let parsed = url::Url::parse(raw).ok()?;
    (parsed.scheme() == "http").then(|| raw.to_owned())
}

fn aria2_request_context(
    state: &AppState,
    task: &DownloadTask,
) -> (aria2::RequestContext, usize, u64) {
    let settings = state
        .settings
        .lock()
        .ok()
        .map(|value| value.clone())
        .unwrap_or_default();
    let identity = state
        .request_identities
        .lock()
        .ok()
        .and_then(|items| items.get(&task.id).cloned());
    let host_rule = host_rule_for_url(&settings, &task.source).cloned();
    let connections = task
        .connections_override
        .or_else(|| host_rule.as_ref().and_then(|rule| rule.connections))
        .unwrap_or(settings.connections_per_download)
        .clamp(1, 32);
    let mut headers = HashMap::new();
    if let Some(referer) = task.referer.as_deref() {
        headers.insert("Referer".to_owned(), referer.to_owned());
    }
    if let Some(user_agent) = host_rule
        .as_ref()
        .and_then(|rule| rule.user_agent.as_ref())
        .or(settings.user_agent.as_ref())
        .or_else(|| {
            identity
                .as_ref()
                .and_then(|value| value.user_agent.as_ref())
        })
    {
        headers.insert("User-Agent".to_owned(), user_agent.clone());
    }
    if let Some(cookie) = identity
        .as_ref()
        .and_then(|value| value.cookie_header.as_ref())
    {
        headers.insert("Cookie".to_owned(), cookie.clone());
    }
    if let Some(content_type) = identity
        .as_ref()
        .and_then(|value| value.request_content_type.as_ref())
    {
        headers.insert("Content-Type".to_owned(), content_type.clone());
    }
    if let Some(credential) =
        effective_credential_for_download(&settings, &task.source, task.referer.as_deref())
    {
        let basic = BASE64.encode(format!("{}:{}", credential.username, credential.password));
        headers.insert("Authorization".to_owned(), format!("Basic {basic}"));
    }
    let download_limit = task
        .bandwidth_limit
        .or_else(|| host_rule.as_ref().and_then(|rule| rule.bandwidth_limit))
        .unwrap_or_default();
    (
        aria2::RequestContext {
            method: identity
                .as_ref()
                .map(|value| value.request_method.clone())
                .unwrap_or_else(|| "GET".to_owned()),
            headers,
            body: identity
                .and_then(|value| value.request_body)
                .unwrap_or_default(),
            proxy_url: aria2_http_proxy_url(&settings),
            proxy_username: settings.proxy_username.clone(),
            proxy_password: settings.proxy_password.clone(),
            proxy_required: settings.proxy_enabled,
        },
        connections,
        download_limit,
    )
}

async fn run_aria2_download(
    app: tauri::AppHandle,
    id: DownloadId,
    task: DownloadTask,
    kind: DownloadKind,
    mut cancellation: oneshot::Receiver<()>,
) {
    let startup_started_at = Instant::now();
    let state = app.state::<AppState>();
    let is_bittorrent = matches!(kind, DownloadKind::Torrent | DownloadKind::Magnet);
    update_task(&app, id, false, |item| {
        item.state = DownloadState::Downloading;
        item.progress_percent = Some(0.0);
        item.download_speed = Some(0);
        item.upload_speed = Some(0);
        item.resume_supported = Some(true);
    });
    let route_app = app.clone();
    let route_operation = id.to_string();
    tauri::async_runtime::spawn(async move {
        let state = route_app.state::<AppState>();
        log_network_route(&state, &route_operation, "aria2").await;
    });
    let endpoint = match aria2_endpoint(&state, false).await {
        Ok(endpoint) => endpoint,
        Err(message) => {
            update_task(&app, id, true, |item| {
                item.state = DownloadState::Failed {
                    message: format!("aria2_unavailable:{message}"),
                }
            });
            if let Ok(mut workers) = state.workers.lock() {
                workers.remove(&id);
            }
            start_next_queued(&app);
            return;
        }
    };
    diagnostic_log(
        &state,
        "INFO",
        "aria2.endpoint_ready",
        &format!(
            "task={id} elapsed_ms={}",
            startup_started_at.elapsed().as_millis()
        ),
    );
    let (context, connections, download_limit) = aria2_request_context(&state, &task);
    let existing_task = state
        .aria2_tasks
        .lock()
        .ok()
        .and_then(|items| items.get(&id).cloned());
    let existing_task = match existing_task {
        Some(gid) => match endpoint.status(&gid).await {
            Ok(status) => Some((gid, status.status)),
            Err(error) => {
                diagnostic_log(
                    &state,
                    "WARN",
                    "aria2.restored_gid_stale",
                    &format!("task={id} gid={gid} error={error} recreating=true"),
                );
                if let Ok(mut items) = state.aria2_tasks.lock() {
                    items.remove(&id);
                }
                update_task(&app, id, true, |item| item.aria2_gid = None);
                None
            }
        },
        None => None,
    };
    let gid = match existing_task {
        Some((gid, status)) => {
            if status == "paused" {
                if let Err(error) = endpoint.resume(&gid).await {
                    update_task(&app, id, true, |item| {
                        item.state = DownloadState::Failed {
                            message: format!("aria2_resume_failed:{error}"),
                        }
                    });
                    if let Ok(mut workers) = state.workers.lock() {
                        workers.remove(&id);
                    }
                    start_next_queued(&app);
                    return;
                }
            }
            gid
        }
        None => {
            let is_http = matches!(kind, DownloadKind::Http | DownloadKind::AcceleratedHttp);
            let add_uri_started_at = Instant::now();
            let added = if is_bittorrent && context.proxy_required {
                Err("aria2_bittorrent_proxy_unsupported".to_owned())
            } else if is_bittorrent {
                // Every Torrent/Magnet task is started from a real .torrent
                // persisted under data/torrents. This keeps metadata discovery
                // separate from payload transfer and makes select-file deterministic.
                let torrent_path = match task
                    .torrent_metadata_path
                    .as_ref()
                    .filter(|path| path.is_file())
                    .cloned()
                {
                    Some(path) => Ok(path),
                    None => materialize_torrent_metadata_file(
                        &state,
                        Some(&endpoint),
                        &task.source,
                    )
                    .await,
                };
                match torrent_path {
                    Ok(path) => {
                        update_task(&app, id, true, |item| {
                            item.torrent_metadata_path = Some(path.clone())
                        });
                        match fs::read(&path) {
                            Ok(bytes) => {
                                endpoint
                                    .add_bittorrent(
                                        &bytes,
                                        &task.destination,
                                        &task.torrent_selection,
                                        download_limit,
                                    )
                                    .await
                            }
                            Err(error) => Err(error.to_string()),
                        }
                    }
                    Err(error) => Err(error),
                }
            } else if context.proxy_required && context.proxy_url.is_none() {
                Err("aria2_proxy_scheme_unsupported".to_owned())
            } else {
                endpoint
                    .add_download(
                        &task.source,
                        &task.destination,
                        connections,
                        &context,
                        is_http,
                        download_limit,
                    )
                    .await
            };
            match added {
                Ok(gid) => {
                    diagnostic_log(
                        &state,
                        "INFO",
                        "aria2.add_uri_returned",
                        &format!(
                            "task={id} gid={gid} add_ms={} total_startup_ms={}",
                            add_uri_started_at.elapsed().as_millis(),
                            startup_started_at.elapsed().as_millis()
                        ),
                    );
                    if let Ok(mut items) = state.aria2_tasks.lock() {
                        items.insert(id, gid.clone());
                    }
                    update_task(&app, id, true, |item| item.aria2_gid = Some(gid.clone()));
                    gid
                }
                Err(error) => {
                    update_task(&app, id, true, |item| {
                        item.state = DownloadState::Failed {
                            message: format!("aria2_create_failed:{error}"),
                        }
                    });
                    if let Ok(mut workers) = state.workers.lock() {
                        workers.remove(&id);
                    }
                    start_next_queued(&app);
                    return;
                }
            }
        }
    };
    diagnostic_log(&state,"INFO","aria2.task_started",&format!("task={id} gid={gid} engine={kind:?} connections={} listen_port=6881-6999 dht_listen_port=6881-6999",connections.min(16)));
    state.diagnostics.record(if is_bittorrent{"torrent.engine_selected"}else{"http.engine_selected"},"INFO",None,Some(&id.to_string()),serde_json::json!({"engine":"aria2-rpc","backend":"classic","connections":connections.min(16),"kind":format!("{kind:?}"),"maxConnectionPerServer":connections.min(16),"split":connections.min(16),"listenPort":"6881-6999","dhtListenPort":"6881-6999","fileAllocation":"none"}));

    let mut last_at = Instant::now();
    let transfer_started_at = startup_started_at;
    let mut last_downloaded = 0_u64;
    let mut first_payload_logged = false;
    let mut workers_2_logged = false;
    let mut workers_4_logged = false;
    let mut workers_8_logged = false;
    let mut workers_16_logged = false;
    // Poll faster only during startup so the UI can reflect real payload
    // almost immediately without keeping the RPC hot for the entire transfer.
    let mut startup_fast_polling = true;
    let mut interval = tokio::time::interval(Duration::from_millis(100));
    let mut terminal = false;
    loop {
        tokio::select! {
            biased;
            _ = &mut cancellation => {
                let _ = endpoint.pause(&gid).await;
                diagnostic_log(&state, "INFO", "aria2.paused", &format!("task={id} gid={gid}"));
                return;
            }
            _ = interval.tick() => {
                let status = match endpoint.status(&gid).await {
                    Ok(status) => status,
                    Err(error) => {
                        diagnostic_log(&state, "WARN", "aria2.status_failed", &format!("task={id} error={error}"));
                        continue;
                    }
                };
                let now = Instant::now();
                let startup_ms = transfer_started_at.elapsed().as_millis();
                if !first_payload_logged && status.downloaded > 0 {
                    first_payload_logged = true;
                    diagnostic_log(
                        &state,
                        "INFO",
                        "aria2.first_payload_byte",
                        &format!(
                            "task={id} gid={gid} elapsed_ms={startup_ms} bytes={} connections={}",
                            status.downloaded, status.connections
                        ),
                    );
                }
                for (threshold, logged, event) in [
                    (2_u64, &mut workers_2_logged, "aria2.workers_2"),
                    (4_u64, &mut workers_4_logged, "aria2.workers_4"),
                    (8_u64, &mut workers_8_logged, "aria2.workers_8"),
                    (16_u64, &mut workers_16_logged, "aria2.workers_16"),
                ] {
                    if !*logged && status.connections >= threshold {
                        *logged = true;
                        diagnostic_log(
                            &state,
                            "INFO",
                            event,
                            &format!(
                                "task={id} gid={gid} elapsed_ms={startup_ms} connections={}",
                                status.connections
                            ),
                        );
                    }
                }
                if startup_fast_polling
                    && (first_payload_logged
                        || transfer_started_at.elapsed() >= Duration::from_secs(2))
                {
                    startup_fast_polling = false;
                    interval = tokio::time::interval(Duration::from_millis(350));
                }
                let elapsed = now.duration_since(last_at).as_secs_f64().max(0.001);
                let raw_speed = if status.downloaded >= last_downloaded {
                    ((status.downloaded - last_downloaded) as f64 / elapsed) as u64
                } else {
                    status.speed
                };
                last_at = now;
                last_downloaded = status.downloaded;
                let percent = if status.total > 0 {
                    Some((status.downloaded as f64 * 100.0 / status.total as f64).clamp(0.0, 100.0))
                } else {
                    None
                };
                let eta_seconds = if is_bittorrent && status.speed > 0 && status.total > status.downloaded {
                    Some((status.total - status.downloaded) / status.speed)
                } else {
                    None
                };
                update_task(&app, id, false, |item| {
                    item.received = status.downloaded;
                    item.total = (status.total > 0).then_some(status.total);
                    item.progress_percent = percent;
                    item.download_speed = Some(status.speed);
                    item.upload_speed = Some(status.upload_speed);
                    if is_bittorrent {
                        item.torrent_leechers = None;
                        item.torrent_seeders = status.seeders;
                        item.torrent_eta = eta_seconds.map(|seconds| format!("{seconds}s"));
                        item.torrent_web_seeds = (status.web_seeds > 0).then_some(status.web_seeds);
                    }
                });
                state.diagnostics.record(
                    if is_bittorrent { "torrent.performance_sample" } else { "http.performance_sample" },
                    "INFO",
                    None,
                    Some(&id.to_string()),
                    serde_json::json!({
                        "engine": "aria2-rpc",
                        "bytesPerSecond": status.speed,
                        "reportedBytesPerSecond": status.speed,
                        "computedDeltaBytesPerSecond": raw_speed,
                        "speedSource": "aria2",
                        "receivedBytes": status.downloaded,
                        "totalBytes": status.total,
                        "progressPercent": percent,
                        "activeConnections": status.connections,
                        "livePeers": status.seeders,
                        "sampleWindowMs": (elapsed * 1000.0).round() as u64
                    }),
                );
                match status.status.as_str() {
                    "complete" => {
                        if is_bittorrent
                            && task
                                .expected_size
                                .is_some_and(|expected| status.total < expected)
                        {
                            diagnostic_log(
                                &state,
                                "ERROR",
                                "aria2.torrent_expected_size_mismatch",
                                &format!(
                                    "task={id} gid={gid} reported_total={} expected_min={}",
                                    status.total,
                                    task.expected_size.unwrap_or_default()
                                ),
                            );
                            update_task(&app, id, true, |item| {
                                item.state = DownloadState::Failed {
                                    message: "aria2_torrent_expected_size_mismatch".to_owned(),
                                };
                                item.aria2_gid = None;
                            });
                            if let Ok(mut items) = state.aria2_tasks.lock() {
                                items.remove(&id);
                            }
                            terminal = true;
                            break;
                        }
                        if is_bittorrent && status.selected_file_bytes > status.total {
                            diagnostic_log(
                                &state,
                                "ERROR",
                                "aria2.torrent_size_mismatch",
                                &format!(
                                    "task={id} gid={gid} reported_total={} selected_file_bytes={}",
                                    status.total, status.selected_file_bytes
                                ),
                            );
                            update_task(&app, id, true, |item| {
                                item.state = DownloadState::Failed {
                                    message: "aria2_torrent_size_mismatch".to_owned(),
                                };
                                item.aria2_gid = None;
                            });
                            if let Ok(mut items) = state.aria2_tasks.lock() {
                                items.remove(&id);
                            }
                            terminal = true;
                            break;
                        }
                        update_task(&app, id, true, |item| {
                            item.received = status.total.max(status.downloaded);
                            item.total = Some(status.total.max(status.downloaded));
                            item.progress_percent = Some(100.0);
                            item.download_speed = Some(0);
                            item.upload_speed = Some(0);
                            item.state = DownloadState::Completed;
                            item.completed_at = Some(epoch_seconds());
                            item.aria2_gid = None;
                        });
                        maybe_auto_extract_completed(&app, id);
                        let _ = endpoint.remove_result(&gid).await;
                        if let Ok(mut items) = state.aria2_tasks.lock() {
                            items.remove(&id);
                        }
                        terminal = true;
                        break;
                    }
                    "error" | "removed" => {
                        let engine_tag = if is_bittorrent { "aria2_torrent_error" } else { "aria2_task_failed" };
                        update_task(&app, id, true, |item| {
                            item.state = DownloadState::Failed {
                                message: status
                                    .error_message
                                    .as_deref()
                                    .map(|reason| format!("{engine_tag}:{reason}"))
                                    .unwrap_or_else(|| engine_tag.to_owned()),
                            };
                            item.download_speed = Some(0);
                            item.upload_speed = Some(0);
                            item.aria2_gid = None;
                        });
                        if let Ok(mut items) = state.aria2_tasks.lock() {
                            items.remove(&id);
                        }
                        terminal = true;
                        break;
                    }
                    "paused" => {
                        update_task(&app, id, true, |item| {
                            item.state = DownloadState::Paused;
                            item.download_speed = Some(0);
                            item.upload_speed = Some(0);
                        });
                        terminal = true;
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
    if terminal {
        if let Ok(mut workers) = state.workers.lock() {
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
        item.download_speed = Some(0);
        item.upload_speed = Some(0);
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
                configured_aria2(&settings),
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
                if cfg!(windows) {
                    "aria2c.exe".into()
                } else {
                    "aria2".into()
                },
                16,
                None,
                None,
                None,
                Vec::new(),
            )
        });
    let host_rule = app
        .state::<AppState>()
        .settings
        .lock()
        .ok()
        .and_then(|settings| host_rule_for_url(&settings, &task.source).cloned());
    let task_connections = task
        .connections_override
        .or_else(|| host_rule.as_ref().and_then(|rule| rule.connections))
        .unwrap_or(tools.4)
        .clamp(1, 32);
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
    let configured_user_agent = host_rule
        .as_ref()
        .and_then(|rule| rule.user_agent.clone())
        .or_else(|| {
            app.state::<AppState>()
                .settings
                .lock()
                .ok()
                .and_then(|settings| settings.user_agent.clone())
        });
    let global_bandwidth_limit = app
        .state::<AppState>()
        .settings
        .lock()
        .map(|settings| settings.global_bandwidth_limit)
        .unwrap_or_default();
    let bandwidth_limit = match (
        global_bandwidth_limit,
        task.bandwidth_limit
            .or_else(|| host_rule.as_ref().and_then(|rule| rule.bandwidth_limit))
            .unwrap_or_default(),
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
            effective_credential_for_download(&settings, &task.source, task.referer.as_deref())
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
            let media_source =
                canonical_facebook_video_url(&task.source).unwrap_or_else(|| task.source.clone());
            if let Some(proxy_url) = proxy_url.as_deref() {
                command.arg("--proxy").arg(proxy_url);
            }
            let selection = task
                .format_selection
                .as_deref()
                .unwrap_or("bestvideo+bestaudio/best");
            command.args([
                "--no-playlist",
                "--newline",
                "--verbose",
                "--progress-delta",
                "0.5",
                "--progress-template",
                "download:ADM_PROGRESS|%(progress.downloaded_bytes)s|%(progress.total_bytes)s|%(progress.total_bytes_estimate)s|%(progress.speed)s|%(progress._percent_str)s",
            ]);
            if bandwidth_limit > 0 {
                command.arg("--limit-rate").arg(bandwidth_limit.to_string());
            }
            let media_connections =
                if task.source.contains("youtube.com/") || task.source.contains("youtu.be/") {
                    task_connections.max(16)
                } else {
                    task_connections
                };
            command
                .arg("--concurrent-fragments")
                .arg(media_connections.to_string());
            if task.is_live {
                command.args(["--live-from-start", "--hls-use-mpegts"]);
            }
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
                || task.source.contains("facebook.com/")
                || task.source.contains("twitch.tv/")
                || task.source.contains("twitch.com/")
                || task.source.contains("bilibili.com/")
                || task.source.contains("b23.tv/")
                || task.source.contains("bili.tv/");
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
            } else if browser_session_site
                || task.source.contains("youtube.com/")
                || task.source.contains("youtu.be/")
            {
                command.args(["--cookies-from-browser", "chrome"]);
            }
            command.args([
                "--ignore-config",
                "--retries",
                "30",
                "--fragment-retries",
                "30",
                "--extractor-retries",
                "10",
                "--retry-sleep",
                "2",
                "--retry-sleep",
                "extractor:2",
                "--retry-sleep",
                "fragment:exp=1:8",
                "--socket-timeout",
                "30",
            ]);
            command.args(["--user-agent", user_agent]);
            if let Some(credential) = website_credential.as_ref() {
                let credential_config = media_work_directory
                    .as_deref()
                    .map(|directory| directory.join("yt-dlp-credentials.txt"))
                    .filter(|path| {
                        write_yt_dlp_credential_config(
                            path,
                            &credential.username,
                            &credential.password,
                        )
                        .is_ok()
                    });
                if let Some(path) = credential_config.as_ref() {
                    command.arg("--config-location").arg(path);
                } else {
                    command
                        .arg("--username")
                        .arg(&credential.username)
                        .arg("--password")
                        .arg(&credential.password);
                }
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
                .arg(&media_source);
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
                command.arg("-user_agent").arg(&user_agent);
                if let Some(referer) = task.referer.as_deref() {
                    command.arg("-referer").arg(referer);
                }
                let mut request_headers = String::new();
                if let Some(credential) = website_credential.as_ref() {
                    let basic =
                        BASE64.encode(format!("{}:{}", credential.username, credential.password));
                    request_headers.push_str(&format!("Authorization: Basic {basic}\r\n"));
                }
                if let Some(cookie) = identity
                    .as_ref()
                    .and_then(|value| value.cookie_header.as_deref())
                {
                    request_headers.push_str(&format!("Cookie: {cookie}\r\n"));
                }
                if let Some(referer) = task.referer.as_deref() {
                    if let Some(origin) = http_origin(referer) {
                        request_headers.push_str(&format!("Origin: {origin}\r\n"));
                    }
                }
                if !request_headers.is_empty() {
                    command.arg("-headers").arg(request_headers);
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
                // N_m3u8DL-RE creates its segment workspace relative to the
                // process directory unless an explicit temporary directory is
                // supplied. A GUI application launched on Windows can inherit
                // C:\Windows\System32, where regular users cannot write.
                command.current_dir(directory);
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
                    .arg("--tmp-dir")
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
        _ => return,
    };
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.as_std_mut().creation_flags(0x08000000);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.as_std_mut().process_group(0);
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
                _ = &mut cancellation => {
                    terminate_process_tree(&mut child).await;
                    return;
                }
                status = child.wait() => status,
            };
            let mut text = String::from_utf8_lossy(&output.await.unwrap_or_default()).into_owned();
            text.push_str(&String::from_utf8_lossy(&errors.await.unwrap_or_default()));
            status
                .map_err(|error| error.to_string())
                .and_then(|status| {
                    let engine = engine_log_name(kind, &task);
                    if let Some(path) = write_engine_diagnostic(
                        &app,
                        id,
                        engine,
                        &text,
                        status.code(),
                        status.success(),
                    ) {
                        diagnostic_log(
                            &app.state::<AppState>(),
                            "INFO",
                            "external.engine_report",
                            &format!(
                                "task={id} engine={engine} success={} exit_code={} file={}",
                                status.success(),
                                status.code().unwrap_or(-1),
                                path.display()
                            ),
                        );
                    }
                    if status.success() {
                        return Ok(());
                    }
                    let detail = external_error_detail(&text, status.code());
                    if kind == DownloadKind::MediaPage
                        && task.source.contains("facebook.com/")
                        && text.to_ascii_lowercase().contains("cannot parse data")
                    {
                        Err("facebook_direct_download_unavailable_use_recording".to_owned())
                    } else {
                        Err(detail)
                    }
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
                item.download_speed = Some(0);
                item.upload_speed = Some(0);
                item.state = DownloadState::Completed;
                item.completed_at = Some(epoch_seconds());
            });
            maybe_auto_extract_completed(&app, id);
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
                item.download_speed = Some(0);
                item.upload_speed = Some(0);
                item.state = DownloadState::Failed { message }
            });
        }
    }
    if let Ok(mut workers) = app.state::<AppState>().workers.lock() {
        workers.remove(&id);
    }
    start_next_queued(&app);
}

#[cfg(target_os = "windows")]
async fn terminate_process_tree(child: &mut tokio::process::Child) {
    if let Some(pid) = child.id() {
        let mut command = tokio::process::Command::new("taskkill.exe");
        command.args(["/PID", &pid.to_string(), "/T", "/F"]);
        use std::os::windows::process::CommandExt;
        command.as_std_mut().creation_flags(0x08000000);
        let _ = command.status().await;
    }
    let _ = child.kill().await;
    let _ = child.wait().await;
}

#[cfg(unix)]
async fn terminate_process_tree(child: &mut tokio::process::Child) {
    if let Some(pid) = child.id() {
        let _ = tokio::process::Command::new("kill")
            .args(["-TERM", &format!("-{pid}")])
            .status()
            .await;
    }
    let _ = child.kill().await;
    let _ = child.wait().await;
}

fn engine_log_name(kind: DownloadKind, task: &DownloadTask) -> &'static str {
    match kind {
        DownloadKind::MediaPage => "yt-dlp",
        DownloadKind::Hls
            if task
                .format_selection
                .as_deref()
                .is_some_and(|value| value.starts_with("audio:")) =>
        {
            "ffmpeg"
        }
        DownloadKind::Hls => "n-m3u8dl-re",
        _ => "external",
    }
}

fn write_engine_diagnostic(
    app: &tauri::AppHandle,
    id: DownloadId,
    engine: &str,
    output: &str,
    exit_code: Option<i32>,
    success: bool,
) -> Option<PathBuf> {
    let state = app.state::<AppState>();
    let directory = state.queue_path.parent()?.join("logs").join("engines");
    fs::create_dir_all(&directory).ok()?;
    let safe_engine = engine
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>();
    let path = directory.join(format!("{safe_engine}-{id}.log"));
    let proxy_password = state
        .settings
        .lock()
        .ok()
        .and_then(|settings| settings.proxy_password.clone());
    let sanitized = output
        .lines()
        .map(|line| {
            proxy_password
                .as_deref()
                .filter(|value| !value.is_empty())
                .map_or_else(
                    || sanitize_log_detail(line),
                    |password| sanitize_log_detail(&line.replace(password, "<redacted>")),
                )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let contents = format!(
        "Apocalipse Download Manager - engine diagnostic\nLocal time: {}\nEngine: {engine}\nTask: {id}\nSuccess: {success}\nExit code: {}\n\n{sanitized}\n",
        chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        exit_code.map_or_else(|| "unavailable".to_owned(), |code| code.to_string()),
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
        "dnsServerCount": settings.dns_servers.len(),
        "associations": settings.associations,
        "tools": {
            "ffmpeg": settings.ffmpeg_path.as_ref().is_some_and(|path| path.is_file()),
            "ytDlp": settings.yt_dlp_path.as_ref().is_some_and(|path| path.is_file()),
            "qjs": settings.qjs_path.as_ref().is_some_and(|path| path.is_file()),
            "nM3u8DlRe": settings.n_m3u8dl_re_path.as_ref().is_some_and(|path| path.is_file()),
            "aria2": settings.aria2_path.as_ref().is_some_and(|path| path.is_file()),
            "aria2RpcEnabled": settings.aria2_rpc_enabled,
            "aria2RpcAutoStart": settings.aria2_rpc_auto_start,
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
        "formatVersion": 4,
        "createdAt": now.to_rfc3339(),
        "applicationVersion": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "bridgePort": BRIDGE_PORT,
        "bridgeConnected": bridge_connected,
        "privacy": "Secrets, credentials, cookies, authorization headers, typed page content and URL parameter values are excluded or redacted.",
        "timelineOrdering": "local timestamp plus monotonic server sequence",
        "debugger": "Apocalipse Forensic Debugger V4"
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
        } else if event.starts_with("link.") || event.contains("link_transfer") {
            "link"
        } else if event.starts_with("aria2.") {
            "aria2"
        } else if event.starts_with("torrent.") {
            "torrent"
        } else if event.starts_with("http.") {
            "http"
        } else if event.starts_with("thumbnail.") {
            "thumbnails"
        } else if event.starts_with("tool.") {
            "tools"
        } else if event.starts_with("media.preview") || event.starts_with("preview.") {
            "preview"
        } else if event.starts_with("blob.") || event.contains("recording") {
            "recordings"
        } else if event.starts_with("external.") || event.starts_with("yt_dlp.") {
            "external-media-engines"
        } else {
            "application"
        };
        let target = buckets.entry(bucket).or_default();
        target.push_str(line);
        target.push('\n');
    }
    // thumbnails.jsonl is a documented, always-present debugger domain
    // (see debugger-index.json below); export it even when no
    // "thumbnail.*" events were captured so the exported bundle always
    // matches the documented layout instead of silently omitting the file.
    buckets.entry("thumbnails").or_default();
    for (bucket, contents) in buckets {
        entries.push((
            format!("logs/by-component/{bucket}.jsonl"),
            contents.into_bytes(),
        ));
    }
    let important_events = all_events
        .lines()
        .filter(|line| {
            serde_json::from_str::<serde_json::Value>(line)
                .ok()
                .and_then(|value| value["level"].as_str().map(str::to_owned))
                .is_some_and(|level| matches!(level.as_str(), "WARN" | "ERROR"))
        })
        .collect::<Vec<_>>()
        .join("\n");
    if !important_events.is_empty() {
        entries.push((
            "logs/warnings-errors.jsonl".to_owned(),
            format!("{important_events}\n").into_bytes(),
        ));
    }

    let runtime_root = state.queue_path.parent().unwrap_or_else(|| Path::new("."));
    let engines_dir = runtime_root.join("logs").join("engines");
    if let Ok(files) = fs::read_dir(&engines_dir) {
        let mut files = files.filter_map(Result::ok).collect::<Vec<_>>();
        files.sort_by_key(|entry| entry.metadata().and_then(|meta| meta.modified()).ok());
        for entry in files.into_iter().rev().take(40) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Some(bytes) = read_sanitized_log_tail(&path, 512 * 1024) {
                if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                    entries.push((format!("engines/{name}"), bytes));
                }
            }
        }
    }
    let aria2_log = runtime_root.join("aria2-rpc").join("aria2.log");
    if let Some(bytes) = read_sanitized_log_tail(&aria2_log, 1024 * 1024) {
        entries.push(("engines/aria2-runtime.log".to_owned(), bytes));
    }
    let debugger_index = serde_json::json!({
        "format": "Apocalipse Forensic Debugger V4",
        "startHere": [
            "RELATORIO_PARA_IA.txt",
            "summary.json",
            "timeline/events-local.jsonl",
            "correlation/index.json",
            "incidents/problem-windows.jsonl"
        ],
        "domains": {
            "interface": "logs/by-component/interface.jsonl",
            "browserExtension": "logs/by-component/extension-shortcuts-overlays.jsonl",
            "socialMedia": "social/player-debugger.jsonl",
            "link": "logs/by-component/link.jsonl",
            "aria2": ["logs/by-component/aria2.jsonl", "engines/aria2-runtime.log"],
            "torrent": "logs/by-component/torrent.jsonl",
            "http": "logs/by-component/http.jsonl",
            "externalMediaEngines": ["logs/by-component/external-media-engines.jsonl", "engines/"],
            "preview": "logs/by-component/preview.jsonl",
            "thumbnails": "logs/by-component/thumbnails.jsonl",
            "recordings": "logs/by-component/recordings.jsonl",
            "warningsAndErrors": "logs/warnings-errors.jsonl"
        },
        "ordering": "Use local time first; serverSequence breaks ties and is authoritative within the desktop collector.",
        "privacy": "No passwords, cookies, authorization headers, tokens, typed page text or raw page HTML are intentionally retained."
    });
    entries.push((
        "debugger-index.json".to_owned(),
        serde_json::to_vec_pretty(&debugger_index).map_err(|e| e.to_string())?,
    ));
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
    let guide = b"Apocalipse Forensic Debugger V4\nStart with debugger-index.json, RELATORIO_PARA_IA.txt, timeline/events-local.jsonl, correlation/index.json and incidents/problem-windows.jsonl. Engine logs are sanitized and stored under engines/. A missing event is not proof of no activity.\n";
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
        let trace = detail
            .get("traceId")
            .and_then(|value| value.as_str())
            .filter(|value| uuid::Uuid::parse_str(value).is_ok())
            .map(str::to_owned);
        let task = detail
            .get("taskId")
            .and_then(|value| value.as_str())
            .filter(|value| uuid::Uuid::parse_str(value).is_ok())
            .map(str::to_owned);
        let level = detail
            .get("level")
            .and_then(|value| value.as_str())
            .filter(|value| matches!(*value, "DEBUG" | "INFO" | "WARN" | "ERROR"))
            .unwrap_or("INFO")
            .to_owned();
        state.diagnostics.record(
            &format!("ui.{event}"),
            &level,
            trace.as_deref(),
            task.as_deref(),
            detail,
        );
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
fn set_application_theme(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    theme: String,
) -> Result<(), String> {
    const THEMES: &[&str] = &[
        "void", "nebula", "ember", "jade", "plasma", "glacier", "amber", "abyss", "rust", "venom",
        "wine", "linen", "sky", "blossom", "sage", "sand", "lilac", "mist", "citrus", "coral",
        "frost",
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
    // Every window (the main window, the Apocalipse Link popup, and any
    // future standalone window) applies the theme itself from its own
    // localStorage, which Tauri does not share across separate webview
    // windows on every platform. Broadcasting the change lets a window
    // that is already open follow it instead of staying on whatever
    // theme it happened to load with.
    let _ = app.emit("theme-changed", &theme);
    Ok(())
}

#[tauri::command]
fn get_application_theme(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .theme
        .clone())
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
    let mut progress_buffer = String::new();
    let mut chunk = [0_u8; 4096];
    loop {
        match stream.read(&mut chunk).await {
            Ok(0) | Err(_) => break,
            Ok(count) => {
                let text = String::from_utf8_lossy(&chunk[..count]);
                if progress_buffer.len() > 16_384 {
                    progress_buffer.clear();
                }
                progress_buffer.push_str(&text);
                if let Some((app, id, kind)) = progress.as_ref() {
                    if *kind == DownloadKind::MediaPage {
                        if let Some((received, total, percent, speed)) =
                            parse_yt_dlp_progress(&progress_buffer)
                        {
                            update_task(app, *id, false, |task| {
                                task.received = received;
                                task.total = total;
                                task.download_speed = Some(speed);
                                if let Some(percent) = percent {
                                    task.progress_percent = Some(
                                        task.progress_percent.unwrap_or(0.0).max(percent.min(90.0)),
                                    );
                                }
                            });
                        } else if let Some(percent) = parse_external_progress(&progress_buffer) {
                            update_task(app, *id, false, |task| {
                                task.progress_percent = Some(
                                    task.progress_percent.unwrap_or(0.0).max(percent.min(90.0)),
                                );
                            });
                        }
                    } else {
                        let percent = parse_external_progress(&progress_buffer);
                        let speed = (*kind == DownloadKind::Hls)
                            .then(|| parse_external_download_speed(&progress_buffer))
                            .flatten();
                        if percent.is_some() || speed.is_some() {
                            update_task(app, *id, false, |task| {
                                if let Some(percent) = percent {
                                    task.progress_percent =
                                        Some(task.progress_percent.unwrap_or(0.0).max(percent));
                                }
                                if let Some(speed) = speed {
                                    task.download_speed = Some(speed);
                                }
                            });
                        }
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

fn parse_external_download_speed(text: &str) -> Option<u64> {
    let mut latest = None;
    for token in text.split_whitespace() {
        let token = token
            .trim_matches(|character: char| !character.is_ascii_alphanumeric() && character != '.');
        let (number, multiplier) = if let Some(value) = token.strip_suffix("GBps") {
            (value, 1024_u64.pow(3))
        } else if let Some(value) = token.strip_suffix("MBps") {
            (value, 1024_u64.pow(2))
        } else if let Some(value) = token.strip_suffix("KBps") {
            (value, 1024_u64)
        } else if let Some(value) = token.strip_suffix("Bps") {
            (value, 1_u64)
        } else {
            continue;
        };
        let Some(value) = number
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite() && *value >= 0.0)
        else {
            continue;
        };
        latest = Some((value * multiplier as f64) as u64);
    }
    latest
}

fn parse_yt_dlp_number(value: &str) -> Option<u64> {
    let value = value.trim();
    if value.is_empty() || value.eq_ignore_ascii_case("NA") || value.eq_ignore_ascii_case("none") {
        return None;
    }
    value
        .parse::<f64>()
        .ok()
        .filter(|number| number.is_finite() && *number >= 0.0)
        .map(|number| number as u64)
}

fn parse_yt_dlp_progress(text: &str) -> Option<(u64, Option<u64>, Option<f64>, u64)> {
    text.lines().rev().find_map(|line| {
        let payload = line.trim().strip_prefix("ADM_PROGRESS|")?;
        let mut fields = payload.split('|');
        let received = parse_yt_dlp_number(fields.next()?)?;
        let exact_total = parse_yt_dlp_number(fields.next()?);
        let estimated_total = parse_yt_dlp_number(fields.next()?);
        let speed = parse_yt_dlp_number(fields.next()?).unwrap_or(0);
        let percent = fields
            .next()
            .map(str::trim)
            .map(|value| value.trim_end_matches('%').trim())
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|value| (0.0..=100.0).contains(value));
        Some((received, exact_total.or(estimated_total), percent, speed))
    })
}

fn canonical_facebook_video_url(source: &str) -> Option<String> {
    let parsed = url::Url::parse(source).ok()?;
    let host = parsed.host_str()?.to_ascii_lowercase();
    if host != "facebook.com" && !host.ends_with(".facebook.com") {
        return None;
    }
    let segments = parsed.path_segments()?.collect::<Vec<_>>();
    let videos = segments.iter().position(|segment| *segment == "videos")?;
    let video_id = segments[videos + 1..]
        .iter()
        .rev()
        .find(|segment| segment.len() >= 6 && segment.bytes().all(|byte| byte.is_ascii_digit()))?;
    Some(format!("https://www.facebook.com/watch/?v={video_id}"))
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
    let resolved_name = magnet_name;
    resolved_name
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

/// Windows treats these device names as reserved regardless of case or
/// extension (`CON.txt`, `com1.tar.gz`, ...); matching is against the
/// component before the first `.`, as Windows itself does.
fn windows_reserved_file_stem(name: &str) -> bool {
    let base = name.split('.').next().unwrap_or(name);
    matches!(
        base.to_ascii_uppercase().as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
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
    let name = if windows_reserved_file_stem(name) {
        format!("_{name}")
    } else {
        name.to_owned()
    };
    let name = name.as_str();
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
        let torrent_task = matches!(
            classify_url(&task.source),
            Some(DownloadKind::Torrent | DownloadKind::Magnet)
        );
        if let Some(existing) = queue.iter().find(|existing| {
            existing.source == task.source
                && (torrent_task
                    || !matches!(
                        existing.state,
                        DownloadState::Completed | DownloadState::Failed { .. }
                    ))
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

fn reorder_queue_subset(queue: &mut [DownloadTask], ids: &[DownloadId]) -> Result<(), String> {
    if ids.len() < 2 {
        return Ok(());
    }
    let requested = ids.iter().cloned().collect::<HashSet<_>>();
    if requested.len() != ids.len() {
        return Err("duplicate_download_id_in_order".to_owned());
    }
    let positions = queue
        .iter()
        .enumerate()
        .filter_map(|(index, task)| requested.contains(&task.id).then_some(index))
        .collect::<Vec<_>>();
    if positions.len() != ids.len() {
        return Err("download_not_found".to_owned());
    }
    let tasks = queue
        .iter()
        .filter(|task| requested.contains(&task.id))
        .map(|task| (task.id, task.clone()))
        .collect::<HashMap<_, _>>();
    for (position, id) in positions.into_iter().zip(ids.iter()) {
        queue[position] = tasks
            .get(id)
            .cloned()
            .ok_or_else(|| "download_not_found".to_owned())?;
    }
    Ok(())
}

#[tauri::command]
fn reorder_downloads(state: State<'_, AppState>, ids: Vec<DownloadId>) -> Result<(), String> {
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    reorder_queue_subset(&mut queue, &ids)?;
    save_queue(&state, &queue)?;
    state.diagnostics.record(
        "queue.reordered",
        "INFO",
        None,
        None,
        serde_json::json!({
            "taskCount": ids.len(),
            "taskRefs": ids.iter().take(24).map(ToString::to_string).collect::<Vec<_>>(),
            "truncated": ids.len() > 24
        }),
    );
    Ok(())
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
async fn get_tool_statuses(state: State<'_, AppState>) -> Result<Vec<ToolStatus>, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
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
    ];
    let mut statuses = definitions
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
        .collect::<Vec<_>>();

    let aria2_path = configured_aria2(&settings);
    let aria2_endpoint = {
        let mut runtime = state
            .aria2_runtime
            .lock()
            .map_err(|error| error.to_string())?;
        if runtime.as_mut().is_some_and(aria2::Runtime::is_running) {
            runtime.as_ref().map(aria2::Runtime::endpoint)
        } else {
            None
        }
    };
    let aria2_version = match aria2_endpoint {
        Some(endpoint) => endpoint.version().await.ok(),
        None => None,
    };
    statuses.push(ToolStatus {
        id: "aria2".to_owned(),
        path: aria2_path.to_string_lossy().into_owned(),
        found: aria2_path.is_file(),
        version: aria2_version,
    });

    let extractor = settings.extractor_path.clone().unwrap_or_default();
    let kind = extractor_kind(&extractor);
    let version = kind.and_then(|kind| extractor_version(&extractor, kind));
    statuses.push(ToolStatus {
        id: "extractor".to_owned(),
        path: extractor.to_string_lossy().into_owned(),
        found: kind.is_some() && extractor.is_file(),
        version: version.or_else(|| kind.map(|value| format!("{value:?}"))),
    });
    let player = settings.media_player_path.clone().unwrap_or_default();
    statuses.push(ToolStatus {
        id: "player".to_owned(),
        path: player.to_string_lossy().into_owned(),
        found: player.is_file(),
        // Never execute an arbitrary configured player just to display its status.
        // VLC on Windows can open an interactive help console for version/help probes.
        // Downloaded mpv builds are validated during installation instead.
        version: None,
    });
    Ok(statuses)
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
    extractor: String,
) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.ffmpeg_path = optional_path(ffmpeg);
    settings.yt_dlp_path = optional_path(yt_dlp);
    settings.qjs_path = optional_path(qjs);
    settings.n_m3u8dl_re_path = optional_path(n_m3u8dl_re);
    settings.aria2_path = optional_path(aria2);
    settings.extractor_path = optional_path(extractor);
    save_settings(&state, &settings)
}

#[tauri::command]
async fn get_aria2_rpc_settings(
    state: State<'_, AppState>,
) -> Result<Aria2RpcSettingsStatus, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    let (connected, active_port, endpoint) = {
        let mut runtime = state
            .aria2_runtime
            .lock()
            .map_err(|error| error.to_string())?;
        let connected = runtime.as_mut().is_some_and(aria2::Runtime::is_running);
        let active_port = if connected {
            runtime.as_ref().map(aria2::Runtime::port)
        } else {
            None
        };
        let endpoint = if connected {
            runtime.as_ref().map(aria2::Runtime::endpoint)
        } else {
            None
        };
        (connected, active_port, endpoint)
    };
    let version = match endpoint {
        Some(endpoint) => endpoint.version().await.ok(),
        None => None,
    };
    Ok(Aria2RpcSettingsStatus {
        enabled: settings.aria2_rpc_enabled,
        auto_start: settings.aria2_rpc_auto_start,
        configured_port: settings.aria2_rpc_port,
        connected,
        active_port,
        version,
    })
}

#[tauri::command]
fn set_aria2_rpc_settings(
    state: State<'_, AppState>,
    enabled: bool,
    auto_start: bool,
    port: Option<u16>,
) -> Result<(), String> {
    if port.is_some_and(|port| port < 1024) {
        return Err("aria2_rpc_port_invalid".to_owned());
    }
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    let changed = settings.aria2_rpc_enabled != enabled
        || settings.aria2_rpc_auto_start != auto_start
        || settings.aria2_rpc_port != port;
    settings.aria2_rpc_enabled = enabled;
    settings.aria2_rpc_auto_start = auto_start;
    settings.aria2_rpc_port = port;
    save_settings(&state, &settings)?;
    drop(settings);
    if changed {
        stop_aria2_runtime(&state);
    }
    Ok(())
}

#[tauri::command]
async fn test_aria2_rpc(state: State<'_, AppState>) -> Result<Aria2RpcSettingsStatus, String> {
    let endpoint = aria2_endpoint(&state, true).await?;
    endpoint.version().await?;
    get_aria2_rpc_settings(state).await
}

#[tauri::command]
fn regenerate_aria2_rpc_token(state: State<'_, AppState>) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    settings.aria2_rpc_secret = default_aria2_rpc_secret();
    save_settings(&state, &settings)?;
    drop(settings);
    stop_aria2_runtime(&state);
    Ok(())
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

fn is_previewable_video_path(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    let lower = name.to_ascii_lowercase();
    let payload_name = lower.strip_suffix(".part").unwrap_or(&lower);
    Path::new(payload_name)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| {
            matches!(
                extension,
                "mp4" | "mkv" | "webm" | "avi" | "mov" | "m4v" | "ts"
            )
        })
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
        } else if is_previewable_video_path(&path) {
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

fn extract_zip_archive(bytes: &[u8], destination: &Path) -> Result<(), String> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|error| error.to_string())?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
        let Some(relative) = entry.enclosed_name() else {
            return Err("release_archive_contains_unsafe_path".to_owned());
        };
        let target = destination.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&target).map_err(|error| error.to_string())?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let mut output = fs::File::create(&target).map_err(|error| error.to_string())?;
        std::io::copy(&mut entry, &mut output).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg(unix)]
fn extract_tar_archive(bytes: &[u8], asset_name: &str, destination: &Path) -> Result<(), String> {
    let archive_path = destination.join(asset_name);
    fs::write(&archive_path, bytes).map_err(|error| error.to_string())?;
    let listing = Command::new("tar")
        .arg("-tf")
        .arg(&archive_path)
        .output()
        .map_err(|error| error.to_string())?;
    if !listing.status.success() {
        return Err(String::from_utf8_lossy(&listing.stderr).trim().to_owned());
    }
    for entry in String::from_utf8_lossy(&listing.stdout).lines() {
        let path = Path::new(entry);
        if path.is_absolute()
            || path.components().any(|component| {
                matches!(
                    component,
                    std::path::Component::ParentDir | std::path::Component::Prefix(_)
                )
            })
        {
            return Err("release_archive_contains_unsafe_path".to_owned());
        }
    }
    let status = Command::new("tar")
        .arg("-xf")
        .arg(&archive_path)
        .arg("-C")
        .arg(destination)
        .status()
        .map_err(|error| error.to_string())?;
    let _ = fs::remove_file(&archive_path);
    status
        .success()
        .then_some(())
        .ok_or_else(|| "release_extraction_failed".to_owned())
}

fn extract_release_archive(
    bytes: &[u8],
    asset_name: &str,
    destination: &Path,
) -> Result<(), String> {
    let lower = asset_name.to_ascii_lowercase();
    if lower.ends_with(".zip") {
        return extract_zip_archive(bytes, destination);
    }
    #[cfg(unix)]
    if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") || lower.ends_with(".tar.xz") {
        return extract_tar_archive(bytes, asset_name, destination);
    }
    Err(format!("unsupported_release_archive:{asset_name}"))
}

fn release_platform_architecture() -> Result<(&'static str, &'static str), String> {
    let platform = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        return Err("tool_update_platform_unsupported".to_owned());
    };
    let architecture = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        return Err("tool_update_architecture_unsupported".to_owned());
    };
    Ok((platform, architecture))
}

/// aria2 publishes exact, versioned asset names (e.g.
/// `aria2-2.8.1-linux-x86_64`), unlike upstream aria2's static builds
/// which this used to match by substring. Matching the exact suffix (minus
/// the version, which changes every release) is both simpler and safer than
/// substring markers here.
fn aria2_asset_suffix() -> Result<&'static str, String> {
    if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        Ok("windows-x64.exe")
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        Ok("linux-x64")
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
        Ok("macos-x64")
    } else {
        Err("manual_update_required:aria2".to_owned())
    }
}

fn active_torrent_video(directory: &Path) -> Option<PathBuf> {
    let mut best: Option<(u64, PathBuf)> = None;
    for entry in fs::read_dir(directory).ok()?.flatten() {
        let path = entry.path();
        let candidate = if path.is_dir() {
            find_video_file(&path, 0)
        } else if is_previewable_video_path(&path) {
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
        if best.as_ref().is_none_or(|(current, _)| size > *current) {
            best = Some((size, candidate));
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
    #[cfg(target_os = "linux")]
    if player
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("AppImage"))
    {
        command.env("APPIMAGE_EXTRACT_AND_RUN", "1");
    }
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

fn portable_tools_directory() -> Result<PathBuf, String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let root = executable
        .parent()
        .ok_or_else(|| "executable_has_no_parent".to_owned())?
        .join("tools");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(root)
}

fn release_asset<F>(release: &serde_json::Value, predicate: F) -> Result<(String, String), String>
where
    F: Fn(&str) -> bool,
{
    let assets = release
        .get("assets")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "release_has_no_assets".to_owned())?;
    assets
        .iter()
        .find_map(|asset| {
            let name = asset.get("name")?.as_str()?;
            if !predicate(&name.to_ascii_lowercase()) {
                return None;
            }
            let url = asset.get("browser_download_url")?.as_str()?;
            Some((name.to_owned(), url.to_owned()))
        })
        .ok_or_else(|| "compatible_release_asset_not_found".to_owned())
}

async fn github_latest_release(
    client: &reqwest::Client,
    repository: &str,
) -> Result<serde_json::Value, String> {
    client
        .get(format!(
            "https://api.github.com/repos/{repository}/releases/latest"
        ))
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?
        .json()
        .await
        .map_err(|error| error.to_string())
}

async fn download_release_bytes(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, String> {
    Ok(client
        .get(url)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?
        .bytes()
        .await
        .map_err(|error| error.to_string())?
        .to_vec())
}

fn verify_release_sha256(
    checksum_text: &str,
    asset_name: &str,
    bytes: &[u8],
) -> Result<(), String> {
    let expected = checksum_text
        .split_whitespace()
        .next()
        .filter(|value| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or_else(|| format!("checksum_invalid:{asset_name}"))?
        .to_ascii_lowercase();
    let actual = format!("{:x}", Sha256::digest(bytes));
    if actual != expected {
        return Err(format!(
            "checksum_mismatch:{asset_name}:expected={expected}:actual={actual}"
        ));
    }
    Ok(())
}

fn install_validated_executable(
    bytes: &[u8],
    target: &Path,
    version_args: &[&str],
) -> Result<String, String> {
    if bytes.len() < 16_384 {
        return Err("downloaded_tool_payload_too_small".to_owned());
    }
    let parent = target
        .parent()
        .ok_or_else(|| "tool_target_has_no_directory".to_owned())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let name = target
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "tool_target_has_no_filename".to_owned())?;
    let staged = parent.join(format!(".apocalipse-download-{name}"));
    let backup = parent.join(format!(".apocalipse-download-backup-{name}"));
    fs::write(&staged, bytes).map_err(|error| error.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&staged, fs::Permissions::from_mode(0o755))
            .map_err(|error| error.to_string())?;
    }
    let version = version_line(&staged, version_args).ok_or_else(|| {
        let _ = fs::remove_file(&staged);
        "downloaded_tool_validation_failed".to_owned()
    })?;
    let _ = fs::remove_file(&backup);
    if target.exists() {
        fs::rename(target, &backup).map_err(|error| error.to_string())?;
    }
    if let Err(error) = fs::rename(&staged, target) {
        if backup.exists() {
            let _ = fs::rename(&backup, target);
        }
        return Err(error.to_string());
    }
    let _ = fs::remove_file(&backup);
    Ok(version)
}

fn copy_validated_executable(
    source: &Path,
    target: &Path,
    version_args: &[&str],
) -> Result<String, String> {
    let bytes = fs::read(source).map_err(|error| error.to_string())?;
    install_validated_executable(&bytes, target, version_args)
}

fn clean_directory(directory: &Path) -> Result<(), String> {
    if directory.exists() {
        fs::remove_dir_all(directory).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(directory).map_err(|error| error.to_string())
}

#[tauri::command]
async fn download_tool(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let (platform, architecture) = release_platform_architecture()?;
    let tools_root = portable_tools_directory()?;
    let tool_dir = tools_root.join(match id.as_str() {
        "n-m3u8dl-re" => "n-m3u8dl-re",
        value => value,
    });
    fs::create_dir_all(&tool_dir).map_err(|error| error.to_string())?;

    let client = reqwest::Client::builder()
        .user_agent("Apocalipse-Download-Manager")
        .build()
        .map_err(|error| error.to_string())?;

    let result = match id.as_str() {
        "yt-dlp" => {
            let release = github_latest_release(&client, "yt-dlp/yt-dlp").await?;
            let expected = match (platform, architecture) {
                ("windows", "x86_64") => "yt-dlp.exe",
                ("windows", "aarch64") => "yt-dlp_arm64.exe",
                ("linux", "x86_64") => "yt-dlp_linux",
                ("linux", "aarch64") => "yt-dlp_linux_aarch64",
                ("macos", _) => "yt-dlp_macos",
                _ => return Err("tool_download_platform_unsupported:yt-dlp".to_owned()),
            };
            let (_, url) = release_asset(&release, |name| name == expected.to_ascii_lowercase())?;
            let bytes = download_release_bytes(&client, &url).await?;
            let target = tool_dir.join(if cfg!(windows) {
                "yt-dlp.exe"
            } else {
                "yt-dlp"
            });
            install_validated_executable(&bytes, &target, &["--version"])?;
            target
        }
        "qjs" => {
            let release = github_latest_release(&client, "quickjs-ng/quickjs").await?;
            let expected = match (platform, architecture) {
                ("windows", "x86_64") => "qjs-windows-x86_64.exe",
                ("windows", "aarch64") => {
                    return Err("tool_download_platform_unsupported:qjs".to_owned())
                }
                ("linux", "x86_64") => "qjs-linux-x86_64",
                ("linux", "aarch64") => "qjs-linux-aarch64",
                ("macos", "x86_64") => "qjs-darwin-x86_64",
                ("macos", "aarch64") => "qjs-darwin-arm64",
                _ => return Err("tool_download_platform_unsupported:qjs".to_owned()),
            };
            let (_, url) = release_asset(&release, |name| name == expected)?;
            let bytes = download_release_bytes(&client, &url).await?;
            let target = tool_dir.join(if cfg!(windows) { "qjs.exe" } else { "qjs" });
            install_validated_executable(&bytes, &target, &["--version"])?;
            target
        }
        "aria2" => {
            let release =
                github_latest_release(&client, "FerroDownload/aria2-static-builds").await?;
            let suffix = aria2_asset_suffix()?;
            let (asset_name, url) = release_asset(&release, |name| {
                name.starts_with("aria2c-") && name.ends_with(suffix) && !name.ends_with(".sha256")
            })?;
            let checksum_name = format!("{asset_name}.sha256");
            let (_, checksum_url) = release_asset(&release, |name| name == checksum_name)?;
            let bytes = download_release_bytes(&client, &url).await?;
            let checksum_bytes = download_release_bytes(&client, &checksum_url).await?;
            let checksum_text =
                String::from_utf8(checksum_bytes).map_err(|error| error.to_string())?;
            verify_release_sha256(&checksum_text, &asset_name, &bytes)?;
            let target = tool_dir.join(if cfg!(windows) {
                "aria2c.exe"
            } else {
                "aria2c"
            });
            install_validated_executable(&bytes, &target, &["--version"])?;
            target
        }
        "n-m3u8dl-re" => {
            let release = github_latest_release(&client, "nilaoda/N_m3u8DL-RE").await?;
            let (platform_marker, arch_marker) = match (platform, architecture) {
                ("windows", "x86_64") => ("win-", "x64"),
                ("windows", "aarch64") => ("win-", "arm64"),
                ("linux", "x86_64") => ("linux-", "x64"),
                ("linux", "aarch64") => ("linux-", "arm64"),
                ("macos", "x86_64") => ("osx-", "x64"),
                ("macos", "aarch64") => ("osx-", "arm64"),
                _ => return Err("tool_download_platform_unsupported:n-m3u8dl-re".to_owned()),
            };
            let (asset_name, url) = release_asset(&release, |name| {
                name.contains(platform_marker)
                    && name.contains(arch_marker)
                    && (name.ends_with(".zip") || name.ends_with(".tar.gz"))
            })?;
            let bytes = download_release_bytes(&client, &url).await?;
            clean_directory(&tool_dir)?;
            extract_release_archive(&bytes, &asset_name, &tool_dir)?;
            let executable_name = if cfg!(windows) {
                "N_m3u8DL-RE.exe"
            } else {
                "N_m3u8DL-RE"
            };
            let target = find_named_file(&tool_dir, executable_name, 0)
                .ok_or_else(|| format!("replacement_executable_missing:{asset_name}"))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&target, fs::Permissions::from_mode(0o755))
                    .map_err(|error| error.to_string())?;
            }
            version_line(&target, &["--version"])
                .ok_or_else(|| "downloaded_tool_validation_failed".to_owned())?;
            target
        }
        "ffmpeg" => {
            let (repository, release) = if platform == "macos" {
                let repository = "eugeneware/ffmpeg-static";
                (
                    repository,
                    github_latest_release(&client, repository).await?,
                )
            } else {
                let repository = "BtbN/FFmpeg-Builds";
                (
                    repository,
                    github_latest_release(&client, repository).await?,
                )
            };
            if platform == "macos" {
                let suffix = if architecture == "aarch64" {
                    "arm64"
                } else {
                    "x64"
                };
                let (_, ffmpeg_url) =
                    release_asset(&release, |name| name == format!("ffmpeg-darwin-{suffix}"))?;
                let (_, ffprobe_url) =
                    release_asset(&release, |name| name == format!("ffprobe-darwin-{suffix}"))?;
                let ffmpeg_bytes = download_release_bytes(&client, &ffmpeg_url).await?;
                let ffprobe_bytes = download_release_bytes(&client, &ffprobe_url).await?;
                let ffmpeg_target = tool_dir.join("ffmpeg");
                let ffprobe_target = tool_dir.join("ffprobe");
                install_validated_executable(&ffmpeg_bytes, &ffmpeg_target, &["-version"])?;
                install_validated_executable(&ffprobe_bytes, &ffprobe_target, &["-version"])?;
                ffmpeg_target
            } else {
                let marker = match (platform, architecture) {
                    ("windows", "x86_64") => "win64",
                    ("windows", "aarch64") => "winarm64",
                    ("linux", "x86_64") => "linux64",
                    ("linux", "aarch64") => "linuxarm64",
                    _ => {
                        return Err(format!(
                            "tool_download_platform_unsupported:ffmpeg:{repository}"
                        ))
                    }
                };
                let (asset_name, url) = release_asset(&release, |name| {
                    if platform == "windows" {
                        name.ends_with(&format!("{marker}-gpl.zip")) && !name.contains("shared")
                    } else {
                        name.ends_with(&format!("{marker}-gpl.tar.xz")) && !name.contains("shared")
                    }
                })?;
                let bytes = download_release_bytes(&client, &url).await?;
                let temporary = std::env::temp_dir()
                    .join(format!("apocalipse-tool-download-{}", uuid::Uuid::new_v4()));
                let extracted = temporary.join("extracted");
                fs::create_dir_all(&extracted).map_err(|error| error.to_string())?;
                extract_release_archive(&bytes, &asset_name, &extracted)?;
                let ffmpeg_name = if cfg!(windows) {
                    "ffmpeg.exe"
                } else {
                    "ffmpeg"
                };
                let ffprobe_name = if cfg!(windows) {
                    "ffprobe.exe"
                } else {
                    "ffprobe"
                };
                let ffmpeg_source = find_named_file(&extracted, ffmpeg_name, 0)
                    .ok_or_else(|| "ffmpeg_missing_from_release".to_owned())?;
                let ffprobe_source = find_named_file(&extracted, ffprobe_name, 0)
                    .ok_or_else(|| "ffprobe_missing_from_release".to_owned())?;
                let ffmpeg_target = tool_dir.join(ffmpeg_name);
                let ffprobe_target = tool_dir.join(ffprobe_name);
                copy_validated_executable(&ffmpeg_source, &ffmpeg_target, &["-version"])?;
                copy_validated_executable(&ffprobe_source, &ffprobe_target, &["-version"])?;
                let _ = fs::remove_dir_all(&temporary);
                ffmpeg_target
            }
        }
        "extractor" => {
            let release = github_latest_release(&client, "ip7z/7zip").await?;
            if platform == "windows" {
                let marker = if architecture == "aarch64" {
                    "-arm64.exe"
                } else {
                    "-x64.exe"
                };
                let (asset_name, url) = release_asset(&release, |name| {
                    name.starts_with("7z") && name.ends_with(marker)
                })?;
                let bytes = download_release_bytes(&client, &url).await?;
                if bytes.len() < 500_000 {
                    return Err(format!("downloaded_tool_payload_too_small:{asset_name}"));
                }
                clean_directory(&tool_dir)?;
                let installer = std::env::temp_dir()
                    .join(format!("apocalipse-7zip-{}.exe", uuid::Uuid::new_v4()));
                fs::write(&installer, bytes).map_err(|error| error.to_string())?;
                let mut command = Command::new(&installer);
                command.arg("/S").arg(format!("/D={}", tool_dir.display()));
                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::process::CommandExt;
                    command.creation_flags(0x08000000);
                }
                let status = command.status().map_err(|error| error.to_string())?;
                let _ = fs::remove_file(&installer);
                if !status.success() {
                    return Err("seven_zip_portable_install_failed".to_owned());
                }
                let target = tool_dir.join("7z.exe");
                extractor_version(&target, ExtractorKind::SevenZip)
                    .ok_or_else(|| "downloaded_tool_validation_failed".to_owned())?;
                target
            } else {
                let marker = match (platform, architecture) {
                    ("linux", "x86_64") => "linux-x64.tar.xz",
                    ("linux", "aarch64") => "linux-arm64.tar.xz",
                    ("macos", _) => "-mac.tar.xz",
                    _ => return Err("tool_download_platform_unsupported:extractor".to_owned()),
                };
                let (asset_name, url) = release_asset(&release, |name| name.ends_with(marker))?;
                let bytes = download_release_bytes(&client, &url).await?;
                clean_directory(&tool_dir)?;
                extract_release_archive(&bytes, &asset_name, &tool_dir)?;
                let target = find_named_file(&tool_dir, "7zz", 0)
                    .ok_or_else(|| "seven_zip_executable_missing".to_owned())?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&target, fs::Permissions::from_mode(0o755))
                        .map_err(|error| error.to_string())?;
                }
                extractor_version(&target, ExtractorKind::SevenZip)
                    .ok_or_else(|| "downloaded_tool_validation_failed".to_owned())?;
                target
            }
        }
        "player" => {
            if platform == "linux" {
                let release = github_latest_release(&client, "pkgforge-dev/mpv-AppImage").await?;
                let marker = if architecture == "aarch64" {
                    "aarch64.appimage"
                } else {
                    "x86_64.appimage"
                };
                let (_, url) = release_asset(&release, |name| {
                    name.ends_with(marker) && !name.ends_with(".zsync")
                })?;
                let bytes = download_release_bytes(&client, &url).await?;
                let target = tool_dir.join("mpv.AppImage");
                install_validated_executable(&bytes, &target, &["--version"])?;
                target
            } else {
                let release = github_latest_release(&client, "mpv-player/mpv").await?;
                let (asset_name, url) =
                    release_asset(&release, |name| match (platform, architecture) {
                        ("windows", "x86_64") => {
                            name.contains("x86_64-pc-windows-msvc") && name.ends_with(".zip")
                        }
                        ("windows", "aarch64") => {
                            name.contains("aarch64-pc-windows-msvc") && name.ends_with(".zip")
                        }
                        ("macos", "x86_64") => {
                            name.contains("macos")
                                && name.contains("intel")
                                && name.ends_with(".zip")
                        }
                        ("macos", "aarch64") => {
                            name.contains("macos-14-arm") && name.ends_with(".zip")
                        }
                        _ => false,
                    })?;
                let bytes = download_release_bytes(&client, &url).await?;
                clean_directory(&tool_dir)?;
                extract_release_archive(&bytes, &asset_name, &tool_dir)?;
                let executable_name = if platform == "windows" {
                    "mpv.exe"
                } else {
                    "mpv"
                };
                let target = find_named_file(&tool_dir, executable_name, 0)
                    .ok_or_else(|| format!("replacement_executable_missing:{asset_name}"))?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&target, fs::Permissions::from_mode(0o755))
                        .map_err(|error| error.to_string())?;
                }
                version_line(&target, &["--version"])
                    .ok_or_else(|| "downloaded_tool_validation_failed".to_owned())?;
                target
            }
        }
        _ => return Err("unknown_tool".to_owned()),
    };

    diagnostic_log(
        &state,
        "INFO",
        "tool.downloaded",
        &format!("tool={id} target={}", result.display()),
    );
    Ok(result.to_string_lossy().into_owned())
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
            "aria2" => configured_aria2(&settings),
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

    {
        let (platform, architecture) = release_platform_architecture()?;
        let aria2_suffix = aria2_asset_suffix()?;
        let (repository, executable_name, asset_markers, version_args): (
            &str,
            &str,
            &[&str],
            &[&str],
        ) = match id.as_str() {
            "qjs" => (
                "quickjs-ng/quickjs",
                if cfg!(windows) { "qjs.exe" } else { "qjs" },
                &[],
                &["--version"],
            ),
            "aria2" => (
                "FerroDownload/aria2-static-builds",
                if cfg!(windows) {
                    "aria2c.exe"
                } else {
                    "aria2c"
                },
                std::slice::from_ref(&aria2_suffix),
                &["--version"],
            ),
            "n-m3u8dl-re" => (
                "nilaoda/N_m3u8DL-RE",
                if cfg!(windows) {
                    "N_m3u8DL-RE.exe"
                } else {
                    "N_m3u8DL-RE"
                },
                &[],
                &["--version"],
            ),
            "ffmpeg" => (
                if cfg!(target_os = "macos") {
                    "eugeneware/ffmpeg-static"
                } else {
                    "BtbN/FFmpeg-Builds"
                },
                if cfg!(windows) {
                    "ffmpeg.exe"
                } else {
                    "ffmpeg"
                },
                &[],
                &["-version"],
            ),
            _ => return Err("unknown_tool".to_owned()),
        };
        let before =
            version_line(&executable, version_args).unwrap_or_else(|| "not installed".to_owned());

        let client = reqwest::Client::builder()
            .user_agent("Apocalipse-Download-Manager")
            .build()
            .map_err(|error| error.to_string())?;
        // GitHub's /latest endpoint excludes drafts and prereleases.
        let release: serde_json::Value = client
            .get(format!(
                "https://api.github.com/repos/{repository}/releases/latest"
            ))
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
                    "qjs" => match (platform, architecture) {
                        ("windows", "x86_64") => name == "qjs-windows-x86_64.exe",
                        ("linux", "x86_64") => name == "qjs-linux-x86_64",
                        ("macos", "x86_64") => name == "qjs-darwin-x86_64",
                        _ => false,
                    },
                    "n-m3u8dl-re" => {
                        let platform_marker = match platform {
                            "windows" => "win-",
                            "linux" => "linux-",
                            "macos" => "osx-",
                            _ => return false,
                        };
                        let arch_marker = "x64";
                        name.contains(platform_marker)
                            && name.contains(arch_marker)
                            && (name.ends_with(".zip") || name.ends_with(".tar.gz"))
                    }
                    "ffmpeg" => match platform {
                        "windows" => name.ends_with("win64-gpl.zip") && !name.contains("shared"),
                        "linux" => {
                            let marker = "linux64";
                            name.ends_with(&format!("{marker}-gpl.tar.xz"))
                                && !name.contains("shared")
                        }
                        "macos" => {
                            let marker = "x64";
                            name == format!("ffmpeg-darwin-{marker}")
                        }
                        _ => false,
                    },
                    "aria2" => {
                        name.starts_with("aria2c-")
                            && name.ends_with(aria2_suffix)
                            && !name.ends_with(".sha256")
                    }

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
        if id == "aria2" {
            let checksum_name = format!("{asset_name}.sha256");
            let checksum_asset = assets
                .iter()
                .find(|candidate| {
                    candidate.get("name").and_then(|value| value.as_str())
                        == Some(checksum_name.as_str())
                })
                .ok_or_else(|| format!("checksum_asset_missing:{checksum_name}"))?;
            let checksum_url = checksum_asset
                .get("browser_download_url")
                .and_then(|value| value.as_str())
                .ok_or_else(|| "release_asset_url_missing".to_owned())?;
            let checksum_text = client
                .get(checksum_url)
                .send()
                .await
                .map_err(|error| error.to_string())?
                .error_for_status()
                .map_err(|error| error.to_string())?
                .text()
                .await
                .map_err(|error| error.to_string())?;
            verify_release_sha256(&checksum_text, asset_name, &bytes)?;
        }
        let temporary =
            std::env::temp_dir().join(format!("apocalipse-tool-update-{}", uuid::Uuid::new_v4()));
        let (replacement, ffprobe_replacement) =
            if id == "qjs" || id == "aria2" || (id == "ffmpeg" && platform == "macos") {
                let ffprobe_replacement = if id == "ffmpeg" {
                    let marker = "x64";
                    let expected = format!("ffprobe-darwin-{marker}");
                    let probe_asset = assets
                        .iter()
                        .find(|asset| {
                            asset.get("name").and_then(|value| value.as_str())
                                == Some(expected.as_str())
                        })
                        .ok_or_else(|| "ffprobe_missing_from_release".to_owned())?;
                    let probe_url = probe_asset
                        .get("browser_download_url")
                        .and_then(|value| value.as_str())
                        .ok_or_else(|| "release_asset_url_missing".to_owned())?;
                    client
                        .get(probe_url)
                        .send()
                        .await
                        .map_err(|error| error.to_string())?
                        .error_for_status()
                        .map_err(|error| error.to_string())?
                        .bytes()
                        .await
                        .map_err(|error| error.to_string())?
                        .to_vec()
                } else {
                    Vec::new()
                };
                (bytes.to_vec(), ffprobe_replacement)
            } else {
                let extracted = temporary.join("extracted");
                fs::create_dir_all(&extracted).map_err(|error| error.to_string())?;
                if let Err(error) = extract_release_archive(&bytes, asset_name, &extracted) {
                    let _ = fs::remove_dir_all(&temporary);
                    return Err(format!("release_extraction_failed:{error}"));
                }
                let replacement_path = find_named_file(&extracted, executable_name, 0)
                    .ok_or_else(|| format!("replacement_executable_missing:{asset_name}"))?;
                let replacement = fs::read(replacement_path).map_err(|error| error.to_string())?;
                let ffprobe_replacement = if id == "ffmpeg" {
                    fs::read(
                        find_named_file(
                            &extracted,
                            if cfg!(windows) {
                                "ffprobe.exe"
                            } else {
                                "ffprobe"
                            },
                            0,
                        )
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
        if id == "aria2" {
            stop_aria2_runtime(&state);
        }
        let parent = executable
            .parent()
            .ok_or_else(|| "tool_target_has_no_directory".to_owned())?;
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let staged = parent.join(format!(".apocalipse-new-{executable_name}"));
        let backup = parent.join(format!(".{executable_name}.apocalipse-backup"));
        let ffprobe_name = if cfg!(windows) {
            "ffprobe.exe"
        } else {
            "ffprobe"
        };
        let ffprobe = parent.join(ffprobe_name);
        let ffprobe_staged = parent.join(format!(".apocalipse-new-{ffprobe_name}"));
        let ffprobe_backup = parent.join(format!(".{ffprobe_name}.apocalipse-backup"));
        fs::write(&staged, &replacement).map_err(|error| error.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&staged, fs::Permissions::from_mode(0o755))
                .map_err(|error| error.to_string())?;
        }
        if id == "ffmpeg" {
            fs::write(&ffprobe_staged, &ffprobe_replacement).map_err(|error| error.to_string())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&ffprobe_staged, fs::Permissions::from_mode(0o755))
                    .map_err(|error| error.to_string())?;
            }
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
    if !matches!(
        classify_url(&source),
        Some(DownloadKind::Torrent | DownloadKind::Magnet)
    ) {
        return Err("not_a_torrent".to_owned());
    }

    let endpoint = if source.starts_with("magnet:") {
        if state
            .settings
            .lock()
            .map_err(|error| error.to_string())?
            .proxy_enabled
        {
            return Err("aria2_bittorrent_proxy_unsupported".to_owned());
        }
        Some(aria2_endpoint(&state, true).await?)
    } else {
        None
    };

    let path = materialize_torrent_metadata_file(&state, endpoint.as_ref(), &source)
        .await
        .map_err(|error| {
            diagnostic_log(
                &state,
                "WARN",
                "aria2.metadata_save_failed",
                &format!("error={error}"),
            );
            error
        })?;
    let mut inspection = inspect_torrent_file(&path)?;
    inspection.torrent_path = Some(path.to_string_lossy().into_owned());
    diagnostic_log(
        &state,
        "INFO",
        "aria2.metadata_parsed",
        &format!(
            "path={} name={} files={} total_size={}",
            path.display(),
            inspection.name,
            inspection.files.len(),
            inspection.total_size
        ),
    );
    Ok(inspection)
}

async fn read_response_limited(
    mut response: reqwest::Response,
    max_bytes: usize,
    too_large_error: &str,
) -> Result<Vec<u8>, String> {
    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        return Err(too_large_error.to_owned());
    }
    let mut bytes = Vec::with_capacity(
        response
            .content_length()
            .unwrap_or_default()
            .min(max_bytes as u64) as usize,
    );
    while let Some(chunk) = response.chunk().await.map_err(|error| error.to_string())? {
        if bytes.len().saturating_add(chunk.len()) > max_bytes {
            return Err(too_large_error.to_owned());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

async fn fetch_torrent_file_bytes(state: &AppState, source: &str) -> Result<Vec<u8>, String> {
    let (proxy_url, proxy_username, proxy_password, dns_servers) = state
        .settings
        .lock()
        .map(|settings| {
            (
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
        .map_err(|error| error.to_string())?;
    let client = DownloadEngine::network_client_builder(
        proxy_url.as_deref(),
        proxy_username.as_deref(),
        proxy_password.as_deref(),
        &dns_servers,
    )
    .map_err(|error| error.to_string())?
    .build()
    .map_err(|error| error.to_string())?;
    let response = client
        .get(source)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    read_response_limited(response, 32 * 1024 * 1024, "torrent_file_payload_too_large").await
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
    auto_extract: Option<bool>,
    expected_size: Option<u64>,
    expected_sha256: Option<String>,
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
        .take(32)
        .collect();
    task.sha256 = expected_sha256
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()));
    task.priority = priority.unwrap_or_default().clamp(-10, 10);
    task.bandwidth_limit = bandwidth_limit.filter(|limit| *limit > 0);
    let pixeldrain_single_connection = host_from_url(&url)
        .as_deref()
        .is_some_and(|value| value == "pixeldrain.com" || value.ends_with(".pixeldrain.com"));
    task.connections_override = site_connection_override(&url, connections_override);
    task.auto_extract = auto_extract.unwrap_or(false) && is_archive_file_name(&file_name);
    task.expected_size = expected_size.filter(|value| *value > 0);
    if let Some(context) = context {
        task.torrent_metadata_path =
            validated_torrent_metadata_path(state, context.torrent_metadata_path);
        if task.expected_size.is_none() {
            task.expected_size = context.expected_size.filter(|value| *value > 0);
        }
        task.referer = context
            .referer
            .filter(|url| url.starts_with("https://") || url.starts_with("http://"));
        task.known_duration = context
            .known_duration
            .filter(|duration| duration.is_finite() && *duration > 0.0);
        task.is_live = context.is_live;
        task.display_title = context.title.and_then(|value| {
            let value = value
                .chars()
                .filter(|character| !character.is_control())
                .take(512)
                .collect::<String>();
            (!value.trim().is_empty()).then(|| value.trim().to_owned())
        });
        task.thumbnail = context.thumbnail.filter(|value| {
            value.len() <= 600_000
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
    if pixeldrain_single_connection {
        diagnostic_log(
            state,
            "INFO",
            "site_rule.pixeldrain_single_connection",
            &format!("task={} host=pixeldrain.com connections=1", task.id),
        );
    }
    if let Some(thumbnail) = task.thumbnail.clone() {
        prefetch_thumbnail(app.clone(), thumbnail);
    }
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
    auto_extract: Option<bool>,
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
        auto_extract,
        None,
        None,
        context,
    )
}

fn queued_task_for_start(queue: &[DownloadTask], id: DownloadId) -> Option<DownloadTask> {
    queue
        .iter()
        .find(|item| item.id == id && item.state == DownloadState::Queued)
        .cloned()
}

async fn run_metalink_manifest(
    app: tauri::AppHandle,
    id: DownloadId,
    task: DownloadTask,
    mut cancellation: oneshot::Receiver<()>,
) {
    let state = app.state::<AppState>();
    diagnostic_log(
        &state,
        "INFO",
        "metalink.inspect_started",
        &format!("task={id} url={}", redact_url(&task.source)),
    );
    update_task(&app, id, true, |item| {
        item.state = DownloadState::Inspecting
    });

    let (proxy, dns, identity, credential) = {
        let settings = match state.settings.lock() {
            Ok(settings) => settings.clone(),
            Err(error) => {
                update_task(&app, id, true, |item| {
                    item.state = DownloadState::Failed {
                        message: error.to_string(),
                    }
                });
                return;
            }
        };
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
        let identity = state
            .request_identities
            .lock()
            .ok()
            .and_then(|items| items.get(&id).cloned());
        let credential =
            effective_credential_for_download(&settings, &task.source, task.referer.as_deref());
        (proxy, dns, identity, credential)
    };

    let builder = match proxy {
        Some((url, username, password)) => DownloadEngine::network_client_builder(
            url.as_deref(),
            username.as_deref(),
            password.as_deref(),
            &dns,
        ),
        None => DownloadEngine::network_client_builder(None, None, None, &dns),
    };
    let client = match builder.and_then(|builder| builder.build().map_err(Into::into)) {
        Ok(client) => client,
        Err(error) => {
            update_task(&app, id, true, |item| {
                item.state = DownloadState::Failed {
                    message: error.to_string(),
                }
            });
            if let Ok(mut workers) = state.workers.lock() {
                workers.remove(&id);
            }
            start_next_queued(&app);
            return;
        }
    };

    let mut request = client.get(&task.source);
    if let Some(referer) = task.referer.as_deref() {
        request = request.header("Referer", referer);
    }
    if let Some(user_agent) = identity
        .as_ref()
        .and_then(|item| item.user_agent.as_deref())
    {
        request = request.header("User-Agent", user_agent);
    }
    if let Some(cookie) = identity
        .as_ref()
        .and_then(|item| item.cookie_header.as_deref())
    {
        request = request.header("Cookie", cookie);
    }
    if let Some(credential) = credential {
        request = request.basic_auth(credential.username, Some(credential.password));
    }

    let response = tokio::select! {
        _ = &mut cancellation => {
            if let Ok(mut workers) = state.workers.lock() {
                workers.remove(&id);
            }
            start_next_queued(&app);
            return;
        }
        response = request.send() => response
    };
    let response = match response.and_then(|response| response.error_for_status()) {
        Ok(response) => response,
        Err(error) => {
            diagnostic_log(
                &state,
                "ERROR",
                "metalink.inspect_failed",
                &error.to_string(),
            );
            update_task(&app, id, true, |item| {
                item.state = DownloadState::Failed {
                    message: format!("metalink_fetch_failed: {error}"),
                }
            });
            if let Ok(mut workers) = state.workers.lock() {
                workers.remove(&id);
            }
            start_next_queued(&app);
            return;
        }
    };

    let bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(error) => {
            update_task(&app, id, true, |item| {
                item.state = DownloadState::Failed {
                    message: format!("metalink_read_failed: {error}"),
                }
            });
            if let Ok(mut workers) = state.workers.lock() {
                workers.remove(&id);
            }
            start_next_queued(&app);
            return;
        }
    };
    let files = match parse_metalink(&bytes, Some(&task.source)) {
        Ok(files) => files,
        Err(error) => {
            diagnostic_log(&state, "ERROR", "metalink.invalid", &error.to_string());
            update_task(&app, id, true, |item| {
                item.state = DownloadState::Failed {
                    message: format!("metalink_invalid: {error}"),
                }
            });
            if let Ok(mut workers) = state.workers.lock() {
                workers.remove(&id);
            }
            start_next_queued(&app);
            return;
        }
    };

    let destination_directory = task
        .destination
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_string_lossy()
        .into_owned();
    let mut prepared = Vec::new();
    for file in files.into_iter().take(256) {
        let Some(primary) = file.urls.first().cloned() else {
            continue;
        };
        let fallback_name = suggested_name(&primary);
        let name = file
            .name
            .filter(|name| validate_file_name(name).is_ok())
            .unwrap_or(fallback_name);
        prepared.push((
            primary,
            name,
            file.urls.into_iter().skip(1).take(31).collect::<Vec<_>>(),
            file.size,
            file.sha256,
        ));
    }
    if prepared.is_empty() {
        update_task(&app, id, true, |item| {
            item.state = DownloadState::Failed {
                message: "metalink_contains_no_downloads".to_owned(),
            }
        });
        if let Ok(mut workers) = state.workers.lock() {
            workers.remove(&id);
        }
        start_next_queued(&app);
        return;
    }

    {
        let mut queue = match state.queue.lock() {
            Ok(queue) => queue,
            Err(error) => {
                update_task(&app, id, true, |item| {
                    item.state = DownloadState::Failed {
                        message: error.to_string(),
                    }
                });
                return;
            }
        };
        queue.retain(|item| item.id != id);
        if let Err(error) = save_queue(&state, &queue) {
            diagnostic_log(&state, "ERROR", "metalink.queue_replace_failed", &error);
            return;
        }
    }
    if let Ok(mut identities) = state.request_identities.lock() {
        identities.remove(&id);
    }
    if let Ok(mut workers) = state.workers.lock() {
        workers.remove(&id);
    }

    let total_children = prepared.len();
    let mut accepted = 0_usize;
    for (primary, name, mirrors, expected_size, sha256) in prepared {
        match enqueue_download_impl(
            app.clone(),
            &state,
            primary,
            Some(destination_directory.clone()),
            Some(name),
            None,
            None,
            Some(mirrors),
            Some(task.priority),
            task.bandwidth_limit,
            task.connections_override,
            Some(task.auto_extract),
            expected_size,
            sha256,
            None,
        ) {
            Ok(_) => accepted += 1,
            Err(error) => diagnostic_log(
                &state,
                "ERROR",
                "metalink.child_rejected",
                &format!("error={error}"),
            ),
        }
    }
    diagnostic_log(
        &state,
        "INFO",
        "metalink.expanded",
        &format!("task={id} children={accepted}/{total_children}"),
    );
    start_next_queued(&app);
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
    if kind == DownloadKind::Metalink {
        tauri::async_runtime::spawn(run_metalink_manifest(app.clone(), task.id, task, cancelled));
        return Ok(());
    }
    if kind == DownloadKind::Http && task.companion_audio_url.is_some() {
        tauri::async_runtime::spawn(run_adaptive_social_download(
            app.clone(),
            task.id,
            task,
            cancelled,
        ));
        return Ok(());
    }
    if matches!(kind, DownloadKind::Torrent | DownloadKind::Magnet) {
        diagnostic_log(
            state,
            "INFO",
            "aria2.dispatched",
            &format!("task={} engine={kind:?}", task.id),
        );
        tauri::async_runtime::spawn(run_aria2_download(
            app.clone(),
            task.id,
            task,
            kind,
            cancelled,
        ));
        return Ok(());
    }
    let special_http_request = kind == DownloadKind::Http
        && state
            .request_identities
            .lock()
            .ok()
            .and_then(|items| items.get(&task.id).cloned())
            .is_some_and(|identity| {
                !identity.request_method.eq_ignore_ascii_case("GET")
                    || identity
                        .request_body
                        .as_deref()
                        .is_some_and(|body| !body.is_empty())
            });
    let aria2_http_network_compatible =
        !limits.dns_enabled && (!limits.proxy_enabled || aria2_http_proxy_url(&limits).is_some());
    let native_http_compatibility = kind == DownloadKind::Http
        && (requires_native_http_compatibility(&task.source)
            || special_http_request
            || !aria2_http_network_compatible);
    if native_http_compatibility {
        diagnostic_log(
            state,
            "INFO",
            "http.native_compatibility_route",
            &format!(
                "task={} reason=aria2_rpc_compatibility host={}",
                task.id,
                host_from_url(&task.source).unwrap_or_default()
            ),
        );
    }
    if !native_http_compatibility
        && matches!(
            kind,
            DownloadKind::Http | DownloadKind::AcceleratedHttp | DownloadKind::Ftp
        )
    {
        diagnostic_log(
            state,
            "INFO",
            "aria2.dispatched",
            &format!("task={} engine={kind:?}", task.id),
        );
        tauri::async_runtime::spawn(run_aria2_download(
            app.clone(),
            task.id,
            task,
            kind,
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
        let host_rule = host_rule_for_url(&limits, &task.source).cloned();
        let connections = task
            .connections_override
            .or_else(|| host_rule.as_ref().and_then(|rule| rule.connections))
            .unwrap_or_else(|| configured_connections.clamp(1, 32))
            .clamp(1, 32);
        let mut headers = Vec::new();
        if let Some(referer) = task.referer.as_ref() {
            headers.push(("Referer".to_owned(), referer.clone()));
        }
        if let Some(user_agent) = host_rule
            .as_ref()
            .and_then(|rule| rule.user_agent.as_ref())
            .or(limits.user_agent.as_ref())
            .or_else(|| identity.as_ref().and_then(|item| item.user_agent.as_ref()))
        {
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
            effective_credential_for_download(&limits, &task.source, task.referer.as_deref())
        {
            let basic = BASE64.encode(format!("{}:{}", credential.username, credential.password));
            headers.push(("Authorization".to_owned(), format!("Basic {basic}")));
        }
        let mirrors = task.mirrors.clone();
        let capacity_host = host_from_url(&task.source);
        let host_capacity_hint_bps = capacity_host
            .as_ref()
            .and_then(|host| limits.http_host_capacities.get(host))
            .map(|estimate| estimate.bytes_per_second)
            .filter(|value| *value > 0);
        let network_capacity_hint_bps = (limits.http_global_capacity.bytes_per_second > 0)
            .then_some(limits.http_global_capacity.bytes_per_second);
        let request = DownloadRequest {
            url: task.source,
            destination: task.destination,
            overwrite: false,
            connections,
            adaptive_connections: true,
            network_capacity_hint_bps,
            host_capacity_hint_bps,
            method: identity
                .as_ref()
                .map(|item| item.request_method.clone())
                .unwrap_or_else(|| "GET".to_owned()),
            body: identity.and_then(|item| item.request_body.map(String::into_bytes)),
            headers,
            expected_size: task.expected_size,
            expected_sha256: task.sha256.clone(),
            limiters: {
                let mut limiters = vec![state.global_bandwidth_limiter.clone()];
                let task_limiter =
                    state
                        .download_bandwidth_limiters
                        .lock()
                        .ok()
                        .and_then(|mut items| {
                            let limit = task
                                .bandwidth_limit
                                .or_else(|| {
                                    host_rule.as_ref().and_then(|rule| rule.bandwidth_limit)
                                })
                                .unwrap_or_default();
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

/// Repoints a paused or failed task at a partial file the user moved to a
/// different folder or drive (e.g. onto larger external storage), so the
/// existing partial bytes are reused instead of starting over. This does
/// not move any file itself: the caller is expected to have already moved
/// the destination file (and its matching `.part`/`.aria2` sidecar, if
/// any) into `new_directory` outside the app. Safety is unchanged from an
/// ordinary resume: the engine still re-validates the partial data against
/// the remote resource (ETag/Last-Modified/size) before continuing, so a
/// mismatched or unrelated file at the new path is rejected the same way a
/// stale partial in the original folder would be, never trusted blindly.
#[tauri::command]
fn relocate_download(
    state: State<'_, AppState>,
    id: DownloadId,
    new_directory: String,
) -> Result<DownloadTask, String> {
    let new_directory = PathBuf::from(new_directory);
    if !new_directory.is_absolute() {
        return Err("destination_must_be_absolute".to_owned());
    }
    if !new_directory.is_dir() {
        return Err("destination_directory_not_found".to_owned());
    }
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    let task = queue
        .iter_mut()
        .find(|task| task.id == id)
        .ok_or_else(|| "download_not_found".to_owned())?;
    if !matches!(
        task.state,
        DownloadState::Paused | DownloadState::Failed { .. }
    ) {
        return Err("download_not_resumable".to_owned());
    }
    if matches!(
        classify_url(&task.source),
        Some(DownloadKind::Torrent | DownloadKind::Magnet)
    ) {
        return Err("torrent_relocate_unsupported".to_owned());
    }
    let file_name = task
        .destination
        .file_name()
        .ok_or_else(|| "download_destination_invalid".to_owned())?;
    let new_destination = new_directory.join(file_name);
    if new_destination == task.destination {
        return Err("destination_unchanged".to_owned());
    }
    task.destination = new_destination;
    let task = task.clone();
    save_queue(&state, &queue)?;
    drop(queue);
    if let Ok(mut mappings) = state.aria2_tasks.lock() {
        mappings.remove(&id);
    }
    {
        let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
        if let Some(item) = queue.iter_mut().find(|item| item.id == id) {
            item.aria2_gid = None;
        }
        save_queue(&state, &queue)?;
    }
    diagnostic_log(
        &state,
        "INFO",
        "task.relocated",
        &format!("task={id} destination={}", task.destination.display()),
    );
    Ok(task)
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
        task.torrent_selection = original.torrent_selection.clone();
        task.referer = original.referer.clone();
        task.known_duration = original.known_duration;
        task.is_live = original.is_live;
        task.display_title = original.display_title.clone();
        task.thumbnail = original.thumbnail.clone();
        task.companion_audio_url = original.companion_audio_url.clone();
        task.mirrors = original.mirrors.clone();
        task.priority = original.priority;
        task.bandwidth_limit = original.bandwidth_limit;
        task.connections_override = original.connections_override;
        task.expected_size = original.expected_size;
        task.sha256 = original.sha256.clone();
        task.auto_extract = original.auto_extract;
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
async fn set_transfer_limits(
    state: State<'_, AppState>,
    max_active_downloads: usize,
    connections_per_download: usize,
    adaptive_efficiency: bool,
    global_bandwidth_limit: u64,
) -> Result<TransferLimits, String> {
    let result = {
        let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
        settings.max_active_downloads = max_active_downloads.clamp(1, 20);
        settings.connections_per_download = connections_per_download.clamp(1, 32);
        settings.adaptive_efficiency = adaptive_efficiency;
        settings.global_bandwidth_limit = global_bandwidth_limit.min(10 * 1024 * 1024 * 1024);
        state
            .global_bandwidth_limiter
            .set_limit(settings.global_bandwidth_limit);
        save_settings(&state, &settings)?;
        TransferLimits {
            max_active_downloads: settings.max_active_downloads,
            connections_per_download: settings.connections_per_download,
            adaptive_efficiency: settings.adaptive_efficiency,
            global_bandwidth_limit: settings.global_bandwidth_limit,
        }
    };
    if let Some(endpoint) = running_aria2_endpoint(&state) {
        endpoint
            .set_global_download_limit(result.global_bandwidth_limit)
            .await?;
    }
    Ok(result)
}

#[tauri::command]
async fn set_download_bandwidth_limit(
    state: State<'_, AppState>,
    id: DownloadId,
    bandwidth_limit: u64,
) -> Result<u64, String> {
    let limit = bandwidth_limit.min(10 * 1024 * 1024 * 1024);
    {
        let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
        let task = queue
            .iter_mut()
            .find(|task| task.id == id)
            .ok_or_else(|| "download_not_found".to_owned())?;
        task.bandwidth_limit = (limit > 0).then_some(limit);
        save_queue(&state, &queue)?;
    }
    {
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
    }
    let gid = state
        .aria2_tasks
        .lock()
        .ok()
        .and_then(|items| items.get(&id).cloned());
    if let (Some(gid), Some(endpoint)) = (gid, running_aria2_endpoint(&state)) {
        endpoint.set_download_limit(&gid, limit).await?;
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

fn normalize_host_rule_pattern(value: &str) -> Result<String, String> {
    let value = value.trim().to_ascii_lowercase();
    if let Some(suffix) = value.strip_prefix("*.") {
        if suffix.contains('*') {
            return Err("invalid_host_rule_pattern".to_owned());
        }
        let host = normalize_credential_host(suffix)
            .map_err(|_| "invalid_host_rule_pattern".to_owned())?;
        return Ok(format!("*.{host}"));
    }
    if value.contains('*') {
        return Err("invalid_host_rule_pattern".to_owned());
    }
    normalize_credential_host(&value).map_err(|_| "invalid_host_rule_pattern".to_owned())
}

fn host_rule_matches(pattern: &str, host: &str) -> bool {
    if let Some(suffix) = pattern.strip_prefix("*.") {
        host != suffix && host.ends_with(&format!(".{suffix}"))
    } else {
        host == pattern
    }
}

fn host_rule_for_url<'a>(settings: &'a UserSettings, source: &str) -> Option<&'a HostRule> {
    let host = url::Url::parse(source)
        .ok()?
        .host_str()?
        .trim_end_matches('.')
        .to_ascii_lowercase();
    settings
        .host_rules
        .iter()
        .filter(|rule| host_rule_matches(&rule.pattern, &host))
        .max_by_key(|rule| {
            (
                usize::from(!rule.pattern.starts_with("*.")),
                rule.pattern.trim_start_matches("*.").len(),
            )
        })
}

fn effective_credential_for_download(
    settings: &UserSettings,
    source: &str,
    referer: Option<&str>,
) -> Option<WebsiteCredential> {
    let credential_from_rule = |rule: &HostRule| {
        if let Some(username) = rule.username.as_deref().filter(|value| !value.is_empty()) {
            if !rule.password.is_empty() {
                return Some(WebsiteCredential {
                    host: rule.pattern.clone(),
                    username: username.to_owned(),
                    password: rule.password.clone(),
                });
            }
        }
        None
    };
    host_rule_for_url(settings, source)
        .and_then(credential_from_rule)
        .or_else(|| {
            let source_host = url::Url::parse(source)
                .ok()?
                .host_str()?
                .to_ascii_lowercase();
            let referer = referer?;
            let referer_host = url::Url::parse(referer)
                .ok()?
                .host_str()?
                .to_ascii_lowercase();
            ((source_host == "fixti.net" || source_host.ends_with(".fixti.net"))
                && (referer_host == "rsload.net" || referer_host.ends_with(".rsload.net")))
            .then(|| host_rule_for_url(settings, referer).and_then(credential_from_rule))
            .flatten()
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

fn host_rule_summaries(settings: &UserSettings) -> Vec<HostRuleSummary> {
    settings
        .host_rules
        .iter()
        .map(|rule| HostRuleSummary {
            pattern: rule.pattern.clone(),
            username: rule.username.clone().unwrap_or_default(),
            has_password: !rule.password.is_empty(),
            user_agent: rule.user_agent.clone().unwrap_or_default(),
            connections: rule.connections,
            bandwidth_limit: rule.bandwidth_limit,
        })
        .collect()
}

#[tauri::command]
fn list_host_rules(state: State<'_, AppState>) -> Result<Vec<HostRuleSummary>, String> {
    let settings = state.settings.lock().map_err(|error| error.to_string())?;
    Ok(host_rule_summaries(&settings))
}

#[tauri::command]
fn save_host_rule(
    state: State<'_, AppState>,
    pattern: String,
    username: String,
    password: String,
    user_agent: String,
    connections: Option<usize>,
    bandwidth_limit: Option<u64>,
    clear_password: bool,
) -> Result<Vec<HostRuleSummary>, String> {
    let pattern = normalize_host_rule_pattern(&pattern)?;
    let username = username.trim();
    let user_agent = user_agent.trim();
    if username.len() > 512
        || password.len() > 2048
        || user_agent.len() > 1024
        || [username, password.as_str(), user_agent]
            .iter()
            .any(|value| {
                value
                    .chars()
                    .any(|character| matches!(character, '\r' | '\n'))
            })
        || connections.is_some_and(|value| !(1..=32).contains(&value))
    {
        return Err("invalid_host_rule".to_owned());
    }

    let account = host_rule_vault_account(&pattern);
    let mut secret = vault_load(&account)?.unwrap_or_default();
    if clear_password {
        vault_delete(&account)?;
        secret.zeroize();
    }
    if !password.is_empty() {
        vault_store_verified(&account, &password)?;
        secret.zeroize();
        secret = password;
    }

    let existing_username = state.settings.lock().ok().and_then(|settings| {
        settings
            .host_rules
            .iter()
            .find(|rule| rule.pattern == pattern)
            .and_then(|rule| rule.username.clone())
    });
    let username = if username.is_empty() && !clear_password {
        existing_username
    } else {
        (!username.is_empty()).then(|| username.to_owned())
    };
    if username.is_some() != !secret.is_empty() {
        secret.zeroize();
        return Err("host_rule_credentials_must_have_username_and_password".to_owned());
    }
    let user_agent = (!user_agent.is_empty()).then(|| user_agent.to_owned());
    let bandwidth_limit = bandwidth_limit.filter(|value| *value > 0);
    if username.is_none()
        && user_agent.is_none()
        && connections.is_none()
        && bandwidth_limit.is_none()
    {
        secret.zeroize();
        return Err("empty_host_rule".to_owned());
    }

    let pattern_key = pattern.clone();
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    if let Some(existing) = settings
        .host_rules
        .iter_mut()
        .find(|rule| rule.pattern == pattern_key)
    {
        existing.password.zeroize();
        existing.username = username;
        existing.password = secret;
        existing.user_agent = user_agent;
        existing.connections = connections;
        existing.bandwidth_limit = bandwidth_limit;
    } else {
        settings.host_rules.push(HostRule {
            pattern,
            username,
            password: secret,
            user_agent,
            connections,
            bandwidth_limit,
        });
    }
    settings.host_rules.sort_by(|left, right| {
        let left_exact = !left.pattern.starts_with("*.");
        let right_exact = !right.pattern.starts_with("*.");
        right_exact
            .cmp(&left_exact)
            .then_with(|| right.pattern.len().cmp(&left.pattern.len()))
            .then_with(|| left.pattern.cmp(&right.pattern))
    });
    save_settings(&state, &settings)?;
    diagnostic_log(
        &state,
        "INFO",
        "host_rule.updated",
        &format!(
            "pattern={} password_stored={}",
            pattern_key,
            settings
                .host_rules
                .iter()
                .find(|rule| rule.pattern == pattern_key)
                .is_some_and(|rule| !rule.password.is_empty())
        ),
    );
    Ok(host_rule_summaries(&settings))
}

#[tauri::command]
fn remove_host_rule(
    state: State<'_, AppState>,
    pattern: String,
) -> Result<Vec<HostRuleSummary>, String> {
    let pattern = normalize_host_rule_pattern(&pattern)?;
    vault_delete(&host_rule_vault_account(&pattern))?;
    let mut settings = state.settings.lock().map_err(|error| error.to_string())?;
    for rule in settings
        .host_rules
        .iter_mut()
        .filter(|rule| rule.pattern == pattern)
    {
        rule.password.zeroize();
    }
    settings.host_rules.retain(|rule| rule.pattern != pattern);
    save_settings(&state, &settings)?;
    diagnostic_log(
        &state,
        "INFO",
        "host_rule.removed",
        &format!("pattern={pattern}"),
    );
    Ok(host_rule_summaries(&settings))
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
        vault_delete(VAULT_PROXY_PASSWORD)?;
        if let Some(secret) = settings.proxy_password.as_mut() {
            secret.zeroize();
        }
        settings.proxy_password = None;
    } else if !password.is_empty() {
        vault_store_verified(VAULT_PROXY_PASSWORD, &password)?;
        if let Some(secret) = settings.proxy_password.as_mut() {
            secret.zeroize();
        }
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
    let clipboard_is_suppressed = || -> Result<bool, String> {
        Ok(state
            .clipboard_suppressed_until
            .lock()
            .map_err(|error| error.to_string())?
            .is_some_and(|until| Instant::now() < until))
    };
    let suppressed_before_read = clipboard_is_suppressed()?;
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
    // The suppression request can arrive while the OS clipboard read is in
    // progress. Recheck after the read so Facebook's internal Copy Link probe
    // can never escape through an already-running clipboard poll.
    let suppressed = suppressed_before_read || clipboard_is_suppressed()?;
    let mut suppressed_value = state
        .clipboard_suppressed_value
        .lock()
        .map_err(|error| error.to_string())?;
    if suppressed {
        *suppressed_value = Some(value.to_owned());
        return Ok(None);
    }
    // Copy Link remains in the Windows clipboard after the timed guard ends.
    // Keep consuming that exact internal value until the user copies something
    // else, otherwise the next 750 ms UI poll opens a delayed save dialog.
    if suppressed_value.as_deref() == Some(value) {
        return Ok(None);
    }
    *suppressed_value = None;
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
        let token = default_bridge_token();
        vault_store_verified(VAULT_BRIDGE_TOKEN, &token)?;
        settings.bridge_token.zeroize();
        settings.bridge_token = token;
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
    // Authenticated media requests may include a bounded thumbnail. Keep a
    // defensive ceiling while allowing the extension's portable preview image.
    const MAX_REQUEST_SIZE: usize = 786_432;
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

fn bridge_response<S: Write>(stream: &mut S, status: &str, origin: Option<&str>, body: &str) {
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
            let _ = window.set_size(tauri::LogicalSize::new(1280.0, 850.0));
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

fn valid_apocalipse_release_url(value: &str) -> Option<&str> {
    let parsed = url::Url::parse(value).ok()?;
    let host = parsed
        .host_str()?
        .trim_end_matches('.')
        .to_ascii_lowercase();
    let path = parsed.path();
    (parsed.scheme() == "https"
        && host == "github.com"
        && (path == "/linuxhell/apocalipse-download-manager/releases"
            || path.starts_with("/linuxhell/apocalipse-download-manager/releases/")))
    .then_some(value)
}

#[tauri::command]
fn open_apocalipse_releases(url: Option<String>) -> Result<(), String> {
    const RELEASES: &str = "https://github.com/linuxhell/apocalipse-download-manager/releases";
    let target = url
        .as_deref()
        .and_then(valid_apocalipse_release_url)
        .unwrap_or(RELEASES);
    #[cfg(target_os = "windows")]
    let result = Command::new("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", target])
        .spawn();
    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(target).spawn();
    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open").arg(target).spawn();
    result.map(|_| ()).map_err(|error| error.to_string())
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

fn duplicate_bridge_prompt(state: &AppState, request: &BridgeDownload) -> bool {
    if request.start_immediately {
        return false;
    }
    let now = Instant::now();
    let key = format!("{:x}", Sha256::digest(request.url.as_bytes()));
    let Ok(mut recent) = state.bridge_recent_prompts.lock() else {
        return false;
    };
    recent.retain(|_, seen| now.duration_since(*seen) < Duration::from_secs(3));
    if recent
        .get(&key)
        .is_some_and(|seen| now.duration_since(*seen) < Duration::from_secs(2))
    {
        true
    } else {
        recent.insert(key, now);
        false
    }
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
    if duplicate_bridge_prompt(&state, &request) {
        state.diagnostics.record(
            "handoff.duplicate_prompt_suppressed",
            "INFO",
            request.trace_id.as_deref(),
            None,
            serde_json::json!({"windowMs":2000}),
        );
        diagnostic_log(
            &state,
            "INFO",
            "bridge.duplicate_prompt_suppressed",
            &format!("url={} window_ms=2000", redact_url(&request.url)),
        );
        return Ok(None);
    }
    if request.start_immediately {
        let context = DownloadContext {
            trace_id: request.trace_id.clone(),
            referer: request.page_url,
            known_duration: request.duration,
            is_live: false,
            title: request.title,
            thumbnail: request.thumbnail,
            audio_url: request.audio_url,
            expected_size: request.expected_size,
            cookie_header: request.cookie_header,
            user_agent: request.user_agent,
            request_method: request.request_method,
            request_body: request.request_body,
            request_content_type: request.request_content_type,
            torrent_metadata_path: None,
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
            None,
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
    let source = PathBuf::from(&request.file_name);
    if !source.is_absolute() || !source.is_file() {
        return Err("browser_download_not_found".to_owned());
    }
    let source = source.canonicalize().map_err(|error| error.to_string())?;
    let file_name = source
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "invalid_file_name".to_owned())?;
    let state = app.state::<AppState>();

    if is_archive_file_name(file_name) {
        let mut request = request;
        request.file_name = source.to_string_lossy().into_owned();
        state
            .browser_assisted_pending
            .lock()
            .map_err(|error| error.to_string())?
            .push(request);
        diagnostic_log(
            &state,
            "INFO",
            "browser_assisted.archive_prompt_pending",
            &format!("file={}", source.display()),
        );
        show_main_window(app);
        let _ = app.emit("browser-assisted-ready", ());
        return Ok(());
    }

    let size = fs::metadata(&source)
        .map_err(|error| error.to_string())?
        .len();
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    if queue.iter().any(|task| {
        task.destination == source
            && task.source == request.url
            && task.state == DownloadState::Completed
    }) {
        return Ok(());
    }
    let mut task = DownloadTask::new(&request.url, source);
    task.state = DownloadState::Completed;
    task.received = size;
    task.total = Some(request.total.unwrap_or(size).max(size));
    task.progress_percent = Some(100.0);
    task.completed_at = Some(epoch_seconds());
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

#[tauri::command]
fn take_browser_assisted_download(
    state: State<'_, AppState>,
) -> Result<Option<BrowserDownloadComplete>, String> {
    let mut pending = state
        .browser_assisted_pending
        .lock()
        .map_err(|error| error.to_string())?;
    Ok((!pending.is_empty()).then(|| pending.remove(0)))
}

#[tauri::command]
fn import_browser_assisted_download(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    local_path: String,
    url: String,
    destination_directory: String,
    file_name: String,
    auto_extract: Option<bool>,
) -> Result<DownloadTask, String> {
    if host_from_url(&url).is_none() {
        return Err("invalid_browser_download_url".to_owned());
    }
    let source = PathBuf::from(local_path.trim());
    if !source.is_absolute() || !source.is_file() {
        return Err("browser_download_not_found".to_owned());
    }
    let source = source.canonicalize().map_err(|error| error.to_string())?;
    let directory = PathBuf::from(destination_directory.trim());
    if !directory.is_absolute() {
        return Err("invalid_destination_directory".to_owned());
    }
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let file_name = validate_file_name(&file_name)?;
    remember_download_directory(&state, &directory)?;

    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    let requested = directory.join(&file_name);
    let destination = if destination_key(&requested) == destination_key(&source) {
        source.clone()
    } else {
        unique_destination_with_queue(&directory, &file_name, &queue)?
    };

    if destination_key(&destination) != destination_key(&source) {
        fs::rename(&source, &destination)
            .or_else(|_| {
                fs::copy(&source, &destination)
                    .map(|_| ())
                    .and_then(|_| fs::remove_file(&source))
            })
            .map_err(|error| error.to_string())?;
    }

    let size = fs::metadata(&destination)
        .map_err(|error| error.to_string())?
        .len();
    let mut task = DownloadTask::new(&url, destination);
    task.state = DownloadState::Completed;
    task.received = size;
    task.total = Some(size);
    task.progress_percent = Some(100.0);
    task.completed_at = Some(epoch_seconds());
    task.auto_extract = auto_extract.unwrap_or(false)
        && task
            .destination
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(is_archive_file_name);
    queue.push(task.clone());
    save_queue(&state, &queue)?;
    drop(queue);

    diagnostic_log(
        &state,
        "INFO",
        "browser_assisted.imported",
        &format!(
            "task={} auto_extract={} file={}",
            task.id,
            task.auto_extract,
            task.destination.display()
        ),
    );
    if task.auto_extract {
        maybe_auto_extract_completed(&app, task.id);
    }
    Ok(task)
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
    let prompt_for_destination = request.prompt_for_destination && !request.recording;
    let directory = if prompt_for_destination {
        state
            .queue_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("staging")
            .join("browser-captures")
    } else {
        configured_download_directory(app, &state)?
    };
    let file_name = validate_file_name(&request.file_name)?;
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    let mut task = DownloadTask::new(request.source.clone(), directory.join(&file_name));
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
                source: request.source,
                received: 0,
                total: (!request.streaming).then_some(request.total),
                recording: request.recording,
                prompt_for_destination,
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
            "task={task_id} bytes={} streaming={} prompt_for_destination={prompt_for_destination}",
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

    if upload.prompt_for_destination {
        {
            let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
            queue.retain(|task| task.id != upload.task_id);
            save_queue(&state, &queue)?;
        }
        state
            .browser_assisted_pending
            .lock()
            .map_err(|error| error.to_string())?
            .push(BrowserDownloadComplete {
                url: upload.source.clone(),
                file_name: upload.destination.to_string_lossy().into_owned(),
                total: Some(upload.received),
            });
        diagnostic_log(
            &state,
            "INFO",
            "blob.destination_prompt_pending",
            &format!(
                "task={} bytes={} file={}",
                upload.task_id,
                upload.received,
                upload.destination.display()
            ),
        );
        show_main_window(app);
        let _ = app.emit("browser-assisted-ready", ());
        return Ok(());
    }

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
    delete_torrent_metadata: Option<bool>,
) -> Result<usize, String> {
    let delete_torrent_metadata = delete_files && delete_torrent_metadata.unwrap_or(false);
    let removal_trace = uuid::Uuid::new_v4().to_string();
    state.diagnostics.record("task.removal_requested", "INFO", Some(&removal_trace), None,
        serde_json::json!({"taskRefs":ids.iter().map(|id|id.to_string()).collect::<Vec<_>>(),"deleteFiles":delete_files,"deleteTorrentMetadata":delete_torrent_metadata}));
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

    let aria2_targets = {
        let aria2_tasks = state
            .aria2_tasks
            .lock()
            .map_err(|error| error.to_string())?;
        removed
            .iter()
            .filter_map(|task| {
                aria2_tasks
                    .get(&task.id)
                    .cloned()
                    .map(|gid| (task.clone(), gid))
            })
            .collect::<Vec<_>>()
    };
    if !aria2_targets.is_empty() {
        match aria2_endpoint(&state, true).await {
            Ok(endpoint) => {
                for (task, gid) in &aria2_targets {
                    let _ = endpoint.remove(gid).await;
                    let _ = endpoint.remove_result(gid).await;
                    if let Ok(mut items) = state.aria2_tasks.lock() {
                        items.remove(&task.id);
                    }
                    state.diagnostics.record(
                        "aria2.task_removed",
                        "INFO",
                        Some(&removal_trace),
                        Some(&task.id.to_string()),
                        serde_json::json!({
                            "gid": gid,
                            "deleteFiles": delete_files
                        }),
                    );
                }
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }
            Err(error) => {
                diagnostic_log(
                    &state,
                    "WARN",
                    "aria2.remove_unavailable",
                    &format!("error={error}"),
                );
            }
        }
    }
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
            if delete_torrent_metadata && task.state == DownloadState::Completed {
                if let Some(path) = task.torrent_metadata_path.as_deref() {
                    if is_managed_torrent_metadata_path(&state, path) {
                        remove_path_with_retry(path, false).await?;
                        state.diagnostics.record(
                            "torrent.metadata_deleted",
                            "INFO",
                            Some(&removal_trace),
                            Some(&task.id.to_string()),
                            serde_json::json!({"path": path.to_string_lossy()}),
                        );
                    }
                }
            }
            if matches!(classify_url(&task.source), Some(DownloadKind::MediaPage)) {
                if let Some(parent) = state.queue_path.parent() {
                    let workspace = parent.join("media-work").join(task.id.to_string());
                    remove_path_with_retry(&workspace, true).await?;
                }
            }
            cleanup_chunk_artifacts(&task.destination)
                .await
                .map_err(|error| error.to_string())?;
            let partial_remaining = partial_path(&task.destination).exists();
            let chunk_artifacts_remaining = chunk_directory(&task.destination).exists();
            state.diagnostics.record(
                "task.removal_disk_cleanup",
                if partial_remaining || chunk_artifacts_remaining {
                    "ERROR"
                } else {
                    "INFO"
                },
                Some(&removal_trace),
                Some(&task.id.to_string()),
                serde_json::json!({
                    "partialRemaining": partial_remaining,
                    "chunkArtifactsRemaining": chunk_artifacts_remaining
                }),
            );
            if partial_remaining || chunk_artifacts_remaining {
                return Err("download_cleanup_incomplete".to_owned());
            }
        }
    }
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    queue.retain(|task| !ids.contains(&task.id));
    if let Ok(mut identities) = state.request_identities.lock() {
        identities.retain(|id, _| !ids.contains(id));
    }
    if let Ok(mut mappings) = state.aria2_tasks.lock() {
        mappings.retain(|id, _| !ids.contains(id));
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
    let aria2_control = PathBuf::from(format!("{}.aria2", task.destination.display()));
    if !paths.contains(&aria2_control) {
        paths.push(aria2_control);
    }

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
    let _ = rustls::crypto::ring::default_provider().install_default();
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let app_data = portable_data_directory(app)?;
            let link_tls_config =
                load_or_create_link_tls_config(&app_data).map_err(std::io::Error::other)?;
            let queue_path = app_data.join("queue.json");
            let settings_path = app_data.join("settings.json");
            let log_path = app_data.join("logs").join("apocalipse.log");
            let mut initial_settings =
                load_settings(&settings_path).map_err(std::io::Error::other)?;
            if initial_settings.aria2_path.is_none() {
                let aria2_dir = app_data.join("tools").join("aria2");
                fs::create_dir_all(&aria2_dir)?;
                initial_settings.aria2_path = Some(aria2_dir.join(if cfg!(windows) {
                    "aria2c.exe"
                } else {
                    "aria2c"
                }));
                write_settings(&settings_path, &initial_settings).map_err(std::io::Error::other)?;
            }
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
            let initial_queue = load_queue(&queue_path);
            let initial_aria2_tasks = restored_aria2_task_map(&initial_queue);
            app.manage(AppState {
                queue: Mutex::new(initial_queue),
                queue_path,
                workers: Mutex::new(HashMap::new()),
                settings: Mutex::new(initial_settings),
                settings_path,
                bridge_last_seen: Mutex::new(None),
                clipboard_suppressed_until: Mutex::new(None),
                clipboard_suppressed_value: Mutex::new(None),
                bridge_pending: Mutex::new(Vec::new()),
                bridge_recent_prompts: Mutex::new(HashMap::new()),
                browser_assisted_pending: Mutex::new(Vec::new()),
                blob_uploads: Mutex::new(HashMap::new()),
                recording_stops: Mutex::new(HashSet::new()),
                request_identities: Mutex::new(HashMap::new()),
                link_transfers: Mutex::new(HashMap::new()),
                aria2_runtime: Mutex::new(None),
                aria2_tasks: Mutex::new(initial_aria2_tasks),
                log_path,
                log_write_lock: Mutex::new(()),
                diagnostics: diagnostics::Diagnostics::new(&app_data.join("logs")),
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
            {
                let network_app = app.handle().clone();
                std::thread::Builder::new()
                    .name("apocalipse-network-monitor".into())
                    .spawn(move || run_network_change_monitor(network_app))?;
            }
            // Apocalipse Link uses TLS on loopback, LAN and Internet-facing binds.
            // Only authenticated sessions can enumerate the explicitly allowed shares.
            if let Ok(listener) = TcpListener::bind(("0.0.0.0", LINK_PORT)) {
                let link_app = app.handle().clone();
                let link_tls_config = link_tls_config.clone();
                std::thread::Builder::new()
                    .name("apocalipse-link-server".into())
                    .spawn(move || run_link_server(link_app, listener, link_tls_config))?;
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
                        TrayIconEvent::Click {
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
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            resolve_file_host_url,
            inspect_url,
            inspect_media_formats,
            inspect_torrent_metadata,
            get_link_identity,
            pause_link_transfer,
            cancel_link_transfer,
            get_link_transfer_progress,
            is_local_link_target,
            get_about_media,
            get_about_background,
            open_link_window,
            authenticate_local_link_account,
            authenticate_remote_link_account,
            list_link_shares,
            add_link_share,
            add_link_file_share,
            update_link_share,
            remove_link_share,
            list_local_link_files,
            get_local_link_share_capabilities,
            download_local_shared_link_item,
            upload_local_shared_link_item,
            delete_local_shared_link_item,
            list_remote_link_files,
            get_remote_link_capabilities,
            download_remote_link_file,
            use_remote_link_file_for_download,
            upload_remote_link_file,
            delete_local_link_item,
            delete_remote_link_item,
            list_downloads,
            reorder_downloads,
            enqueue_download,
            default_download_directory,
            set_default_download_directory,
            pick_directory,
            pick_executable,
            pick_url_list,
            activate_main_window,
            open_apocalipse_releases,
            open_paypal_donation,
            get_tool_statuses,
            set_tool_paths,
            get_aria2_rpc_settings,
            set_aria2_rpc_settings,
            test_aria2_rpc,
            regenerate_aria2_rpc_token,
            get_media_player,
            get_app_version,
            check_app_update,
            resolve_thumbnail,
            set_media_player,
            preview_torrent,
            download_tool,
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
            get_application_theme,
            get_log_editor,
            set_log_editor,
            open_log_external,
            stop_recording,
            pause_download,
            resume_download,
            relocate_download,
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
            list_host_rules,
            save_host_rule,
            remove_host_rule,
            get_dns_setting,
            set_dns_setting,
            get_bridge_pairing,
            regenerate_bridge_token,
            copy_bridge_token,
            list_download_directories,
            remove_download_directory,
            clear_download_directories,
            take_bridge_download,
            take_browser_assisted_download,
            import_browser_assisted_download
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
    fn release_links_are_restricted_to_the_official_apocalipse_repository() {
        assert!(valid_apocalipse_release_url(
            "https://github.com/linuxhell/apocalipse-download-manager/releases"
        )
        .is_some());
        assert!(valid_apocalipse_release_url(
            "https://github.com/linuxhell/apocalipse-download-manager/releases/tag/v1.2.3"
        )
        .is_some());
        assert!(valid_apocalipse_release_url(
            "https://github.com/linuxhell/apocalipse-download-manager.evil.test/releases"
        )
        .is_none());
        assert!(valid_apocalipse_release_url(
            "https://evil.test/linuxhell/apocalipse-download-manager/releases"
        )
        .is_none());
        assert!(valid_apocalipse_release_url(
            "http://github.com/linuxhell/apocalipse-download-manager/releases"
        )
        .is_none());
    }

    #[test]
    fn file_host_adapters_match_only_real_supported_domains() {
        assert_eq!(supported_file_host("www.mediafire.com"), Some("mediafire"));
        assert_eq!(supported_file_host("gofile.io"), Some("gofile"));
        assert_eq!(supported_file_host("cdn.datanodes.to"), Some("datanodes"));
        assert_eq!(supported_file_host("archive.org"), Some("archive"));
        assert_eq!(supported_file_host("mediafire.com.evil.test"), None);
        assert_eq!(supported_file_host("fakearchive.org"), None);
        assert!(file_host_candidate_trusted(
            "mediafire",
            "download123.mediafire.com"
        ));
        assert!(file_host_candidate_trusted(
            "archive",
            "ia801.example.archive.org"
        ));
        assert!(!file_host_candidate_trusted("mediafire", "evil.example"));
        assert!(!file_host_candidate_trusted(
            "gofile",
            "gofile.io.evil.test"
        ));
    }

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
    fn temporary_chatgpt_files_use_native_http_without_matching_spoofed_hosts() {
        assert!(requires_native_http_compatibility(
            "https://sdmntprbrazilsouth.oaiusercontent.com/files/example/raw"
        ));
        assert!(requires_native_http_compatibility(
            "https://oaiusercontent.com/files/example/raw"
        ));
        assert!(!requires_native_http_compatibility(
            "https://oaiusercontent.com.evil.test/files/example/raw"
        ));
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

    #[test]
    fn validate_file_name_escapes_windows_reserved_device_names() {
        assert_eq!(validate_file_name("CON").unwrap(), "_CON");
        assert_eq!(validate_file_name("con.txt").unwrap(), "_con.txt");
        assert_eq!(validate_file_name("Com1.tar.gz").unwrap(), "_Com1.tar.gz");
        assert_eq!(validate_file_name("lpt9").unwrap(), "_lpt9");
        assert_eq!(validate_file_name("nul").unwrap(), "_nul");
        assert_eq!(
            validate_file_name("normal-video.mp4").unwrap(),
            "normal-video.mp4"
        );
        assert_eq!(validate_file_name("Concert.mp4").unwrap(), "Concert.mp4");
        assert_eq!(validate_file_name("COM10.mp4").unwrap(), "COM10.mp4");
    }

    #[test]
    fn yt_dlp_credential_config_quotes_special_characters_and_round_trips() {
        assert_eq!(shell_config_quote("plainuser"), "plainuser");
        assert_eq!(shell_config_quote("a b"), "'a b'");
        assert_eq!(shell_config_quote("it's a p'wd"), "'it'\\''s a p'\\''wd'");
        assert_eq!(shell_config_quote(""), "''");

        let directory = std::env::temp_dir().join(format!(
            "apocalipse-credential-config-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).expect("create temp dir");
        let path = directory.join("config.txt");
        write_yt_dlp_credential_config(&path, "user name", "p@ss 'word'").expect("write config");
        let contents = fs::read_to_string(&path).expect("read config");
        assert_eq!(
            contents,
            "--username 'user name'\n--password 'p@ss '\\''word'\\'''\n"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&path).expect("metadata").permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn extractor_detection_and_safe_destination_arguments_are_cross_platform() {
        assert_eq!(
            extractor_kind(Path::new("7zz")),
            Some(ExtractorKind::SevenZip)
        );
        assert_eq!(
            extractor_kind(Path::new("WinRAR.exe")),
            Some(ExtractorKind::Rar)
        );
        assert_eq!(
            extractor_kind(Path::new("unrar")),
            Some(ExtractorKind::Unrar)
        );
        assert_eq!(extractor_kind(Path::new("unar")), Some(ExtractorKind::Unar));
        assert_eq!(
            extractor_kind(Path::new("bsdtar")),
            Some(ExtractorKind::Bsdtar)
        );
        assert_eq!(extractor_kind(Path::new("tar")), Some(ExtractorKind::Tar));
        assert!(is_archive_file_name("backup.tar.zst"));
        assert_eq!(
            archive_name_without_extensions(Path::new("backup.tar.gz")),
            "backup"
        );
        let args = extraction_args(
            ExtractorKind::SevenZip,
            Path::new("a.zip"),
            Path::new("out"),
        );
        assert_eq!(args[0], "x");
        assert!(args.iter().any(|arg| arg.starts_with("-o")));
    }

    #[test]
    fn manual_queue_reordering_preserves_unfiltered_tasks() {
        let first = DownloadTask::new("https://example.test/first", PathBuf::from("first.bin"));
        let middle = DownloadTask::new("https://example.test/middle", PathBuf::from("middle.bin"));
        let last = DownloadTask::new("https://example.test/last", PathBuf::from("last.bin"));
        let first_id = first.id;
        let middle_id = middle.id;
        let last_id = last.id;
        let mut queue = vec![first, middle, last];

        reorder_queue_subset(&mut queue, &[last_id, first_id]).unwrap();

        assert_eq!(queue[0].id, last_id);
        assert_eq!(queue[1].id, middle_id);
        assert_eq!(queue[2].id, first_id);
    }

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
    fn http_capacity_learning_rises_fast_and_falls_slowly() {
        let mut estimate = HttpCapacityEstimate::default();
        update_capacity_estimate(&mut estimate, 100_000_000, 4);
        assert_eq!(estimate.bytes_per_second, 100_000_000);

        update_capacity_estimate(&mut estimate, 117_000_000, 4);
        assert_eq!(estimate.bytes_per_second, 117_000_000);
        for _ in 0..3 {
            update_capacity_estimate(&mut estimate, 60_000_000, 4);
        }
        assert_eq!(estimate.bytes_per_second, 117_000_000);
        update_capacity_estimate(&mut estimate, 60_000_000, 4);
        assert_eq!(estimate.bytes_per_second, 111_150_000);
    }

    #[test]
    fn host_rules_prefer_exact_then_most_specific_wildcard_without_spoofing() {
        let mut settings = UserSettings::default();
        settings.host_rules = vec![
            HostRule {
                pattern: "*.example.com".into(),
                username: None,
                password: String::new(),
                user_agent: Some("wild".into()),
                connections: Some(6),
                bandwidth_limit: None,
            },
            HostRule {
                pattern: "*.cdn.example.com".into(),
                username: None,
                password: String::new(),
                user_agent: Some("specific".into()),
                connections: Some(10),
                bandwidth_limit: None,
            },
            HostRule {
                pattern: "media.cdn.example.com".into(),
                username: None,
                password: String::new(),
                user_agent: Some("exact".into()),
                connections: Some(12),
                bandwidth_limit: None,
            },
        ];

        assert_eq!(
            host_rule_for_url(&settings, "https://media.cdn.example.com/file")
                .and_then(|rule| rule.connections),
            Some(12)
        );
        assert_eq!(
            host_rule_for_url(&settings, "https://other.cdn.example.com/file")
                .and_then(|rule| rule.connections),
            Some(10)
        );
        assert_eq!(
            host_rule_for_url(&settings, "https://www.example.com/file")
                .and_then(|rule| rule.connections),
            Some(6)
        );
        assert!(host_rule_for_url(&settings, "https://example.com.evil.test/file").is_none());
        assert!(normalize_host_rule_pattern("foo.*.example.com").is_err());
        assert_eq!(
            normalize_host_rule_pattern("*.Example.COM").as_deref(),
            Ok("*.example.com")
        );
    }

    #[test]
    fn normalizes_site_domains_for_transfer_rules() {
        assert_eq!(
            normalize_credential_host("https://Example.COM/").as_deref(),
            Ok("example.com")
        );
        assert!(normalize_credential_host("https://example.com/login").is_err());
    }

    #[test]
    fn applies_rsload_credentials_only_to_its_known_download_host() {
        let mut settings = UserSettings::default();
        settings.host_rules.push(HostRule {
            pattern: "rsload.net".into(),
            username: Some("rsload".into()),
            password: "rsload".into(),
            user_agent: None,
            connections: None,
            bandwidth_limit: None,
        });
        assert!(effective_credential_for_download(
            &settings,
            "https://s4.fixti.net/files/freeware/file.zip",
            Some("https://rsload.net/software/page.html")
        )
        .is_some());
        assert!(effective_credential_for_download(
            &settings,
            "https://unrelated.example/file.zip",
            Some("https://rsload.net/software/page.html")
        )
        .is_none());
    }

    #[test]
    fn social_cookie_domains_cover_authenticated_media_sites_without_spoofing() {
        assert_eq!(
            social_cookie_domain("https://www.facebook.com/reel/123"),
            Some("facebook.com")
        );
        assert_eq!(
            social_cookie_domain("https://www.instagram.com/reel/example/"),
            Some("instagram.com")
        );
        assert_eq!(
            social_cookie_domain("https://www.tiktok.com/@creator/video/123"),
            Some("tiktok.com")
        );
        assert_eq!(
            social_cookie_domain("https://www.twitch.tv/videos/123"),
            Some("twitch.tv")
        );
        assert_eq!(
            social_cookie_domain("https://www.bilibili.com/video/BV1xx"),
            Some("bilibili.com")
        );
        assert_eq!(
            social_cookie_domain("https://b23.tv/example"),
            Some("bilibili.com")
        );
        assert_eq!(
            social_cookie_domain("https://facebook.com.evil.test/video"),
            None
        );
        assert_eq!(
            social_cookie_domain("https://bilibili.com.evil.test/video"),
            None
        );
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
    fn parses_native_n_m3u8dl_download_speed_without_confusing_bits_or_ffmpeg_rate() {
        assert_eq!(
            parse_external_download_speed("VID 23.50% 12.34MBps"),
            Some((12.34_f64 * 1024.0 * 1024.0) as u64)
        );
        assert_eq!(
            parse_external_download_speed("Audio 980.00KBps"),
            Some((980.0_f64 * 1024.0) as u64)
        );
        assert_eq!(parse_external_download_speed("limit 15Mbps"), None);
        assert_eq!(parse_external_download_speed("speed=1.25x"), None);
    }

    #[test]
    fn parses_structured_yt_dlp_speed_and_estimated_total() {
        assert_eq!(
            parse_yt_dlp_progress("ADM_PROGRESS|1048576|NA|4194304|524288| 25.0%"),
            Some((1048576, Some(4194304), Some(25.0), 524288))
        );
        assert_eq!(
            parse_yt_dlp_progress("ADM_PROGRESS|2097152|4194304|NA|NA| 50.0%"),
            Some((2097152, Some(4194304), Some(50.0), 0))
        );
    }

    #[test]
    fn canonicalizes_composite_facebook_video_links() {
        assert_eq!(
            canonical_facebook_video_url(
                "https://www.facebook.com/61592165240994/videos/pcb.1848807619833077/1054475184024216"
            )
            .as_deref(),
            Some("https://www.facebook.com/watch/?v=1054475184024216")
        );
        assert!(
            canonical_facebook_video_url("https://www.facebook.com/reel/1084652417273846")
                .is_none()
        );
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
    fn joins_remote_link_paths_using_the_remote_separator() {
        assert_eq!(remote_link_join(r"C:\Users", "Folder"), r"C:\Users\Folder");
        assert_eq!(
            remote_link_join("/home/user/", "Folder"),
            "/home/user/Folder"
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
        assert_eq!(
            sanitize_log_detail("Cookie: secret"),
            "<redacted-sensitive-line>"
        );
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
