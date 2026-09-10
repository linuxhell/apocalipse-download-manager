//! Direct TikTok, Instagram and Facebook media use authenticated local streams.
use super::{diagnostic_log, AppState, MediaPreviewRequest};
use apocalipse_core::DownloadEngine;
use futures_util::StreamExt;
use reqwest::{header, Client, Method, Response, StatusCode};
use std::{
    net::TcpListener as StdListener,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tauri::{Emitter, Manager};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::oneshot,
    task::JoinSet,
};
use url::Url;

static SESSIONS: AtomicUsize = AtomicUsize::new(0);
type Reporter = Arc<dyn Fn(&'static str, String) + Send + Sync>;

pub(super) fn is_candidate(request: &MediaPreviewRequest) -> bool {
    let Ok(url) = Url::parse(&request.url) else {
        return false;
    };
    let host = url.host_str().unwrap_or("").to_ascii_lowercase();
    let known_host = [
        "tiktok.com",
        "tiktokcdn.com",
        "tiktokcdn-us.com",
        "tiktokcdn-eu.com",
        "tiktokv.com",
        "tiktokv.us",
        "byteoversea.com",
        "ibytedtos.com",
        "muscdn.com",
        "facebook.com",
        "fbcdn.net",
        "fbsbx.com",
        "instagram.com",
        "cdninstagram.com",
    ]
    .iter()
    .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")));
    let path = url.path().to_ascii_lowercase();
    // Leave pages, HLS/DASH manifests, other websites and torrent previews unchanged.
    if !known_host || path.ends_with(".m3u8") || path.ends_with(".mpd") {
        return false;
    }
    let content_type = request
        .content_type
        .as_deref()
        .unwrap_or("")
        .to_ascii_lowercase();
    if content_type.contains("mpegurl") || content_type.contains("dash+xml") {
        return false;
    }
    path.contains("/video/tos/")
        || path.contains("/aweme/v1/play/")
        || [".mp4", ".webm", ".m4a", ".mp3", ".mov"]
            .iter()
            .any(|ext| path.ends_with(ext))
        || content_type.starts_with("video/")
        || content_type.starts_with("audio/")
        || url.query_pairs().any(|(key, value)| {
            key.eq_ignore_ascii_case("mime_type")
                && (value.starts_with("video") || value.starts_with("audio"))
        })
}

fn checked_url(value: &str) -> Result<Url, String> {
    if value.len() > 32768 || value.chars().any(|c| c.is_control()) {
        return Err("invalid_preview_url".into());
    }
    let url = Url::parse(value).map_err(|_| "invalid_preview_url")?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("invalid_preview_url".into());
    }
    Ok(url)
}
pub(super) fn validate(request: &MediaPreviewRequest) -> Result<(), String> {
    checked_url(&request.url)?;
    if let Some(audio) = request.audio_url.as_deref() {
        checked_url(audio)?;
    }
    for (value, limit) in [
        (&request.user_agent, 1024),
        (&request.cookie_header, 16384),
        (&request.audio_cookie_header, 16384),
        (&request.referer, 8192),
    ] {
        if let Some(value) = value {
            if value.len() > limit
                || value.chars().any(|c| c.is_control())
                || header::HeaderValue::from_str(value).is_err()
            {
                return Err("invalid_preview_header".into());
            }
        }
    }
    if let Some(referer) = request.referer.as_deref().filter(|s| !s.is_empty()) {
        checked_url(referer)?;
    }
    Ok(())
}

fn meta_host(host: &str) -> bool {
    [
        "fbcdn.net",
        "fbsbx.com",
        "cdninstagram.com",
        "instagram.com",
        "facebook.com",
    ]
    .iter()
    .any(|domain| {
        host.eq_ignore_ascii_case(domain)
            || host.to_ascii_lowercase().ends_with(&format!(".{domain}"))
    })
}
fn full_media_url(value: &str) -> String {
    let Ok(url) = Url::parse(value) else {
        return value.to_owned();
    };
    if !url.host_str().is_some_and(meta_host) {
        return value.to_owned();
    }
    let (without_fragment, fragment) = value
        .split_once('#')
        .map_or((value, None), |(a, b)| (a, Some(b)));
    let Some((base, query)) = without_fragment.split_once('?') else {
        return value.to_owned();
    };
    let is_bound = |pair: &str, key: &str| {
        pair.split_once('=').is_some_and(|(name, number)| {
            name.eq_ignore_ascii_case(key)
                && !number.is_empty()
                && number.bytes().all(|c| c.is_ascii_digit())
        })
    };
    if !query.split('&').any(|p| is_bound(p, "bytestart"))
        || !query.split('&').any(|p| is_bound(p, "byteend"))
    {
        return value.to_owned();
    }
    let remaining = query
        .split('&')
        .filter(|p| !is_bound(p, "bytestart") && !is_bound(p, "byteend"))
        .collect::<Vec<_>>()
        .join("&");
    let mut result = base.to_owned();
    if !remaining.is_empty() {
        result.push('?');
        result.push_str(&remaining);
    }
    if let Some(fragment) = fragment {
        result.push('#');
        result.push_str(fragment);
    }
    result
}
pub(super) fn effective_player(player: &Path) -> PathBuf {
    if player
        .file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("VLCPortable.exe"))
    {
        if let Some(root) = player.parent() {
            for folder in ["vlc", "vlc64"] {
                let candidate = root.join("App").join(folder).join("vlc.exe");
                if candidate.is_file() {
                    return candidate;
                }
            }
        }
    }
    player.to_owned()
}
pub(super) fn player_command(player: &Path, target: &str) -> Command {
    let mut command = Command::new(player);
    #[cfg(windows)]
    if player
        .file_stem()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.to_ascii_lowercase().contains("vlc"))
    {
        command.args([
            "--no-one-instance",
            "--no-one-instance-when-started-from-file",
            "--no-playlist-enqueue",
        ]);
    }
    command
        .arg(target)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if player.is_absolute() {
        if let Some(parent) = player.parent().filter(|path| path.is_dir()) {
            command.current_dir(parent);
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command
}

// Only these directly launched binaries have a process lifetime we own.
// PotPlayer, portable launchers and other players may hand off to an existing
// window and exit successfully while that window still reads the local URL.
pub(super) fn owns_player_process(player: &Path) -> bool {
    matches!(
        player
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str(),
        "vlc" | "vlc.exe" | "mpv" | "mpv.exe"
    )
}
async fn retain_handoff_session(mut keepalive: oneshot::Sender<()>) {
    // The relay still enforces idle/lifetime limits. Release this sender as soon
    // as the receiver closes; do not retain an unbounded background process.
    keepalive.closed().await;
}

pub(super) fn open(app: &tauri::AppHandle, mut request: MediaPreviewRequest) -> Result<(), String> {
    validate(&request)?;
    let settings = app
        .state::<AppState>()
        .settings
        .lock()
        .map_err(|_| "preview_settings_unavailable")?
        .clone();
    if request.user_agent.as_deref().is_none_or(|s| s.is_empty()) {
        request.user_agent = settings.user_agent.clone();
    }
    if request.referer.as_deref().is_none_or(|s| s.is_empty())
        && !checked_url(&request.url)?.host_str().is_some_and(meta_host)
    {
        request.referer = Some("https://www.tiktok.com/".into());
    }
    validate(&request)?;
    let configured = settings
        .media_player_path
        .clone()
        .unwrap_or_else(|| PathBuf::from(if cfg!(windows) { "vlc.exe" } else { "vlc" }));
    let player = effective_player(&configured);
    let (proxy, user, password) = if settings.proxy_enabled {
        (
            settings.proxy_url.as_deref(),
            settings.proxy_username.as_deref(),
            settings.proxy_password.as_deref(),
        )
    } else {
        (None, None, None)
    };
    let dns = if settings.dns_enabled {
        settings.dns_servers.as_slice()
    } else {
        &[]
    };
    let builder = DownloadEngine::network_client_builder(proxy, user, password, dns)
        .map_err(|_| "preview_network_configuration_failed")?;
    let trace = request
        .trace_id
        .clone()
        .filter(|id| uuid::Uuid::parse_str(id).is_ok())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let relay = Relay::bind(request, builder)?;
    let report_app = app.clone();
    let first_error = std::sync::atomic::AtomicBool::new(false);
    let report: Reporter = Arc::new(move |event, detail| {
        let error = event == "media.preview_error";
        diagnostic_log(
            &report_app.state::<AppState>(),
            if error { "ERROR" } else { "INFO" },
            event,
            &format!("trace={trace} {detail}"),
        );
        if error && !first_error.swap(true, Ordering::SeqCst) {
            super::show_main_window(&report_app);
            let _ = report_app.emit("media-preview-error", detail);
        }
    });
    // Verify the selected stream before opening VLC. Starting a process alone
    // does not establish that the remote media is accessible or complete.
    if let Err(error) = tauri::async_runtime::block_on(relay.probe()) {
        report("media.preview_error", error.clone());
        return Err(error);
    }
    let mut command = player_command(&player, &relay.target);
    let (stop, stopped) = oneshot::channel();
    tauri::async_runtime::spawn(relay.run(
        stopped,
        report.clone(),
        Duration::from_secs(120),
        Duration::from_secs(7200),
    ));
    let mut child = command.spawn().map_err(|error| {
        let message = format!("preview_player_start_failed:{:?}", error.kind());
        report("media.preview_error", message.clone());
        message
    })?;
    let pid = child.id();
    // This is the bridge worker, not the UI thread. Report immediate startup failures.
    std::thread::sleep(Duration::from_millis(200));
    if let Ok(Some(status)) = child.try_wait() {
        if !status.success() {
            let message = format!("preview_player_exited:code={}", status.code().unwrap_or(-1));
            report("media.preview_error", message.clone());
            return Err(message);
        }
    }
    report(
        "media.preview_player_started",
        format!(
            "pid={pid} transport=social_local_stream player={}",
            player.display()
        ),
    );
    let owns_process = owns_player_process(&player);
    tauri::async_runtime::spawn_blocking(move || {
        let outcome = child.wait();
        let handed_off = !owns_process && outcome.as_ref().is_ok_and(|status| status.success());
        match outcome {
            Ok(status) if status.success() => {
                report("media.preview_player_exited", format!("pid={pid} code=0"))
            }
            Ok(status) => report(
                "media.preview_error",
                format!("preview_player_exited:code={}", status.code().unwrap_or(-1)),
            ),
            Err(_) => report("media.preview_error", "preview_player_wait_failed".into()),
        }
        if handed_off {
            report(
                "media.preview_handoff_retained",
                "waiting_for_local_transport_idle".into(),
            );
            tauri::async_runtime::spawn(retain_handoff_session(stop));
        } else {
            let _ = stop.send(());
        }
    });
    Ok(())
}

struct Slot;
impl Slot {
    fn acquire() -> Result<Self, String> {
        SESSIONS
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
                (n < 8).then_some(n + 1)
            })
            .map(|_| Self)
            .map_err(|_| "too_many_media_previews".into())
    }
}
impl Drop for Slot {
    fn drop(&mut self) {
        SESSIONS.fetch_sub(1, Ordering::SeqCst);
    }
}
struct Relay {
    listener: StdListener,
    target: String,
    route: String,
    host: String,
    request: MediaPreviewRequest,
    client: Client,
    _slot: Slot,
}
impl Relay {
    fn bind(
        mut request: MediaPreviewRequest,
        builder: reqwest::ClientBuilder,
    ) -> Result<Self, String> {
        validate(&request)?;
        request.url = full_media_url(&request.url);
        let slot = Slot::acquire()?;
        // Redirects must be followed explicitly so captured cookies cannot escape their scope.
        let client = builder
            .redirect(reqwest::redirect::Policy::none())
            .referer(false)
            .connect_timeout(Duration::from_secs(15))
            .read_timeout(Duration::from_secs(60))
            .build()
            .map_err(|_| "preview_network_configuration_failed")?;
        let listener =
            StdListener::bind(("127.0.0.1", 0)).map_err(|_| "preview_listener_failed")?;
        listener
            .set_nonblocking(true)
            .map_err(|_| "preview_listener_failed")?;
        let host = listener
            .local_addr()
            .map_err(|_| "preview_listener_failed")?
            .to_string();
        let route = format!("/{}/media", uuid::Uuid::new_v4().simple());
        let target = format!("http://{host}{route}");
        Ok(Self {
            listener,
            target,
            route,
            host,
            request,
            client,
            _slot: slot,
        })
    }
    async fn probe(&self) -> Result<(), String> {
        let local = LocalRequest {
            method: Method::GET,
            range: Some("bytes=0-1023".into()),
        };
        tokio::time::timeout(Duration::from_secs(12), async {
            let response = upstream(&self.client, &self.request, &local).await?;
            validate_response(&response)?;
            if !matches!(
                response.status(),
                StatusCode::OK | StatusCode::PARTIAL_CONTENT
            ) {
                return Err(format!(
                    "preview_upstream_http_{}",
                    response.status().as_u16()
                ));
            }
            if response.status() == StatusCode::PARTIAL_CONTENT
                && !response
                    .headers()
                    .get(header::CONTENT_RANGE)
                    .and_then(|value| value.to_str().ok())
                    .is_some_and(|value| value.starts_with("bytes 0-"))
            {
                return Err("preview_incomplete_media".into());
            }
            let mut chunks = response.bytes_stream();
            loop {
                match chunks.next().await {
                    Some(Ok(chunk)) if !chunk.is_empty() => return Ok(()),
                    Some(Ok(_)) => continue,
                    Some(Err(_)) => return Err("preview_upstream_read_failed".into()),
                    None => return Err("preview_empty_media".into()),
                }
            }
        })
        .await
        .map_err(|_| "preview_upstream_timeout".to_owned())?
    }
    async fn run(
        self,
        mut stop: oneshot::Receiver<()>,
        report: Reporter,
        idle: Duration,
        lifetime: Duration,
    ) {
        let Ok(listener) = TcpListener::from_std(self.listener) else {
            report("media.preview_error", "preview_listener_failed".into());
            return;
        };
        let request = Arc::new(self.request);
        let expires = tokio::time::sleep(lifetime);
        tokio::pin!(expires);
        let mut connections = JoinSet::new();
        loop {
            tokio::select! {
                _ = &mut stop => break,
                _ = &mut expires => break,
                _ = tokio::time::sleep(idle), if connections.is_empty() => break,
                _ = connections.join_next(), if !connections.is_empty() => {},
                accepted = listener.accept() => {
                    let Ok((stream, peer)) = accepted else { break; };
                    if !peer.ip().is_loopback() || connections.len() >= 8 { drop(stream); continue; }
                    let (request, client, route, host, report) = (request.clone(), self.client.clone(), self.route.clone(), self.host.clone(), report.clone());
                    connections.spawn(async move {
                        if let Err(error) = serve(stream, &route, &host, &request, &client, &report).await { report("media.preview_error", error); }
                    });
                }
            }
        }
        connections.abort_all();
        while connections.join_next().await.is_some() {}
        report("media.preview_closed", "local_transport_closed".into());
    }
}
struct LocalRequest {
    method: Method,
    range: Option<String>,
}
fn valid_range(value: &str) -> bool {
    let Some((start, end)) = value.strip_prefix("bytes=").and_then(|s| s.split_once('-')) else {
        return false;
    };
    let number = |s: &str| {
        s.is_empty()
            || (s.len() <= 20 && s.bytes().all(|c| c.is_ascii_digit()) && s.parse::<u64>().is_ok())
    };
    !(start.is_empty() && end.is_empty()) && number(start) && number(end)
}
fn parse_request(text: &str, route: &str, host: &str) -> Result<LocalRequest, ()> {
    let mut lines = text.split("\r\n");
    let parts = lines.next().ok_or(())?.split(' ').collect::<Vec<_>>();
    if parts.len() != 3 || parts[1] != route || !matches!(parts[2], "HTTP/1.1" | "HTTP/1.0") {
        return Err(());
    }
    let method = match parts[0] {
        "GET" => Method::GET,
        "HEAD" => Method::HEAD,
        _ => return Err(()),
    };
    let (mut saw_host, mut range) = (false, None);
    for line in lines.take_while(|s| !s.is_empty()) {
        let (key, value) = line.split_once(':').ok_or(())?;
        if !key.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-') {
            return Err(());
        }
        let value = value.trim();
        match key.to_ascii_lowercase().as_str() {
            "host" => {
                if saw_host || value != host {
                    return Err(());
                }
                saw_host = true;
            }
            "origin" | "transfer-encoding" => return Err(()),
            "content-length" if value != "0" => return Err(()),
            "range" => {
                if range.is_some() || !valid_range(value) {
                    return Err(());
                }
                range = Some(value.to_owned());
            }
            _ => {}
        }
    }
    if !saw_host {
        return Err(());
    }
    Ok(LocalRequest { method, range })
}
async fn reject(stream: &mut TcpStream, status: &str) {
    let message = format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n");
    let _ =
        tokio::time::timeout(Duration::from_secs(5), stream.write_all(message.as_bytes())).await;
}
async fn upstream(
    client: &Client,
    request: &MediaPreviewRequest,
    local: &LocalRequest,
) -> Result<Response, String> {
    let mut url = request.url.clone();
    let mut method = local.method.clone();
    let mut send_cookies = true;
    for hop in 0..=5 {
        let mut builder = client
            .request(method.clone(), &url)
            .header(header::ACCEPT_ENCODING, "identity");
        if let Some(agent) = request.user_agent.as_deref().filter(|s| !s.is_empty()) {
            builder = builder.header(header::USER_AGENT, agent);
        }
        if let Some(referer) = request.referer.as_deref().filter(|s| !s.is_empty()) {
            builder = builder.header(header::REFERER, referer);
        }
        if send_cookies {
            if let Some(cookies) = request.cookie_header.as_deref().filter(|s| !s.is_empty()) {
                let mut value =
                    header::HeaderValue::from_str(cookies).map_err(|_| "invalid_preview_header")?;
                value.set_sensitive(true);
                builder = builder.header(header::COOKIE, value);
            }
        }
        if let Some(range) = local.range.as_deref() {
            builder = builder.header(header::RANGE, range);
        }
        let response = builder.send().await.map_err(|error| {
            if error.is_timeout() {
                "preview_upstream_timeout"
            } else {
                "preview_upstream_connection_failed"
            }
        })?;
        if method == Method::HEAD && matches!(response.status().as_u16(), 405 | 501) {
            method = Method::GET;
            continue;
        }
        if response.status().is_redirection() {
            if hop == 5 {
                return Err("preview_too_many_redirects".into());
            }
            let current = checked_url(&url)?;
            let next = response
                .headers()
                .get(header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| current.join(v).ok())
                .ok_or("invalid_preview_redirect")?;
            checked_url(next.as_str())?;
            if current.scheme() == "https" && next.scheme() != "https" {
                return Err("insecure_preview_redirect".into());
            }
            // Cookies are selected for the original URL, including its path, not for redirects.
            send_cookies = false;
            url = next.to_string();
            continue;
        }
        return Ok(response);
    }
    Err("preview_too_many_redirects".into())
}
fn validate_response(response: &Response) -> Result<(), String> {
    let status = response.status();
    if !matches!(
        status,
        StatusCode::OK | StatusCode::PARTIAL_CONTENT | StatusCode::RANGE_NOT_SATISFIABLE
    ) {
        return Err(format!("preview_upstream_http_{}", status.as_u16()));
    }
    if response
        .headers()
        .get(header::CONTENT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| !v.eq_ignore_ascii_case("identity"))
    {
        return Err("preview_encoded_media".into());
    }
    let mime = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();
    if status != StatusCode::RANGE_NOT_SATISFIABLE
        && (mime.starts_with("text/")
            || mime.starts_with("image/")
            || mime.contains("mpegurl")
            || mime.contains("dash+xml")
            || mime.contains("json")
            || mime.contains("xml"))
    {
        return Err("preview_not_direct_media".into());
    }
    if status != StatusCode::RANGE_NOT_SATISFIABLE && response.content_length() == Some(0) {
        return Err("preview_empty_media".into());
    }
    Ok(())
}

async fn serve(
    mut stream: TcpStream,
    route: &str,
    host: &str,
    request: &MediaPreviewRequest,
    client: &Client,
    report: &Reporter,
) -> Result<(), String> {
    let read = async {
        let mut buffer = Vec::new();
        let mut chunk = [0_u8; 1024];
        loop {
            let read = stream.read(&mut chunk).await.map_err(|_| ())?;
            if read == 0 {
                return Err(());
            }
            buffer.extend_from_slice(&chunk[..read]);
            if buffer.len() > 8192 {
                return Err(());
            }
            if let Some(end) = buffer.windows(4).position(|s| s == b"\r\n\r\n") {
                return parse_request(
                    std::str::from_utf8(&buffer[..end + 4]).map_err(|_| ())?,
                    route,
                    host,
                );
            }
        }
    };
    let Ok(Ok(local)) = tokio::time::timeout(Duration::from_secs(5), read).await else {
        reject(&mut stream, "403 Forbidden").await;
        return Ok(());
    };
    let response = match upstream(client, request, &local).await {
        Ok(response) => response,
        Err(error) => {
            reject(&mut stream, "502 Bad Gateway").await;
            return Err(error);
        }
    };
    let status = response.status();
    if let Err(error) = validate_response(&response) {
        reject(&mut stream, "502 Bad Gateway").await;
        return Err(error);
    }
    let mime = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();
    let mut headers = format!("HTTP/1.1 {} {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\n", status.as_u16(), status.canonical_reason().unwrap_or("OK"));
    for key in [
        header::CONTENT_TYPE,
        header::CONTENT_LENGTH,
        header::CONTENT_RANGE,
        header::ACCEPT_RANGES,
    ] {
        if status == StatusCode::RANGE_NOT_SATISFIABLE && key != header::CONTENT_RANGE {
            continue;
        }
        if let Some(value) = response.headers().get(&key).and_then(|v| v.to_str().ok()) {
            if !value.chars().any(|c| c.is_control()) {
                headers.push_str(&format!("{key}: {value}\r\n"));
            }
        }
    }
    if mime.is_empty() {
        headers.push_str("Content-Type: application/octet-stream\r\n");
    }
    if status == StatusCode::RANGE_NOT_SATISFIABLE {
        headers.push_str("Content-Length: 0\r\n");
    }
    headers.push_str("\r\n");
    if !matches!(
        tokio::time::timeout(
            Duration::from_secs(20),
            stream.write_all(headers.as_bytes())
        )
        .await,
        Ok(Ok(()))
    ) {
        return Ok(());
    }
    if local.method == Method::HEAD || status == StatusCode::RANGE_NOT_SATISFIABLE {
        return Ok(());
    }
    let mut chunks = response.bytes_stream();
    let mut bytes = 0_u64;
    while let Some(chunk) = chunks.next().await {
        let chunk = chunk.map_err(|_| "preview_upstream_read_failed")?;
        // The player normally closes a previous range when seeking.
        if !matches!(
            tokio::time::timeout(Duration::from_secs(20), stream.write_all(&chunk)).await,
            Ok(Ok(()))
        ) {
            return Ok(());
        }
        if bytes == 0 && !chunk.is_empty() {
            report(
                "media.preview_streaming",
                format!("http_status={}", status.as_u16()),
            );
        }
        bytes += chunk.len() as u64;
    }
    report("media.preview_transfer_completed", format!("bytes={bytes}"));
    Ok(())
}

// Reuse the authenticated, redirect-safe transport when a preview needs local
// video/audio muxing. Each input carries only its own captured cookie scope.
pub(super) async fn download_to(
    mut request: MediaPreviewRequest,
    builder: reqwest::ClientBuilder,
    destination: &Path,
    limit: u64,
) -> Result<(), String> {
    validate(&request)?;
    request.url = full_media_url(&request.url);
    let client = builder
        .redirect(reqwest::redirect::Policy::none())
        .referer(false)
        .connect_timeout(Duration::from_secs(15))
        .read_timeout(Duration::from_secs(60))
        .build()
        .map_err(|_| "preview_network_configuration_failed")?;
    let response = upstream(
        &client,
        &request,
        &LocalRequest {
            method: Method::GET,
            range: None,
        },
    )
    .await?;
    validate_response(&response)?;
    // A bound CDN response is not a complete track, even when HTTP succeeds.
    if response.status() != StatusCode::OK {
        return Err("preview_incomplete_media_response".into());
    }
    if response.content_length().is_some_and(|size| size > limit) {
        return Err("preview_size_limit".into());
    }
    let mut output = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .await
        .map_err(|_| "preview_file_create_failed")?;
    let mut bytes = 0_u64;
    let mut chunks = response.bytes_stream();
    while let Some(chunk) = chunks.next().await {
        let chunk = chunk.map_err(|_| "preview_upstream_read_failed")?;
        bytes = bytes.saturating_add(chunk.len() as u64);
        if bytes > limit {
            return Err("preview_size_limit".into());
        }
        output
            .write_all(&chunk)
            .await
            .map_err(|_| "preview_file_write_failed")?;
    }
    if bytes == 0 {
        return Err("preview_empty_media".into());
    }
    output
        .flush()
        .await
        .map_err(|_| "preview_file_write_failed")?;
    Ok(())
}

#[cfg(test)]
mod tests;
