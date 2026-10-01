//! Optional transports negotiated through existing Link authorization.
use super::*;
use apocalipse_transport_lab::{dictionary, moqt, multipath};
use std::{
    net::ToSocketAddrs,
    sync::{atomic::AtomicUsize, OnceLock},
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkTransportOptions {
    enabled: bool,
    local_ips: Vec<IpAddr>,
}

#[tauri::command]
pub fn get_link_transport_options(
    state: State<'_, AppState>,
) -> Result<LinkTransportOptions, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(LinkTransportOptions {
        enabled: settings.link_quic_enabled,
        local_ips: settings.link_quic_local_ips.clone(),
    })
}

#[tauri::command]
pub fn set_link_transport_options(
    state: State<'_, AppState>,
    enabled: bool,
    local_ips: Vec<IpAddr>,
) -> Result<(), String> {
    if local_ips.len() > 3
        || local_ips
            .iter()
            .any(|ip| ip.is_unspecified() || ip.is_multicast())
    {
        return Err("invalid_source_ips".into());
    }
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.link_quic_enabled = enabled;
    settings.link_quic_local_ips = local_ips;
    save_settings(&state, &settings)
}

#[derive(Serialize, Deserialize)]
struct Ticket {
    port: u16,
    certificate: Vec<u8>,
    token: String,
}
static ACTIVE_SERVERS: AtomicUsize = AtomicUsize::new(0);
struct ServerSlot;
impl Drop for ServerSlot {
    fn drop(&mut self) {
        ACTIVE_SERVERS.fetch_sub(1, Ordering::AcqRel);
    }
}

pub(super) fn serve_ticket<S: Write>(app: &tauri::AppHandle, headers: &str, stream: &mut S) {
    let state = app.state::<AppState>();
    let path = {
        let Ok(settings) = state.settings.lock() else {
            return;
        };
        if !bridge_authorized(headers, &settings.link_password) {
            bridge_response(stream, "401 Unauthorized", None, "");
            return;
        }
        if !settings.link_quic_enabled {
            bridge_response(stream, "404 Not Found", None, "");
            return;
        }
        let target = headers
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("");
        let path = url::Url::parse(&format!("http://localhost{target}"))
            .ok()
            .and_then(|url| {
                url.query_pairs()
                    .find(|(key, _)| key == "path")
                    .map(|(_, value)| value.into_owned())
            });
        match path
            .and_then(|path| resolve_link_share(&settings, &path).ok())
            .map(|(path, _)| path)
        {
            Some(path) if path.is_file() => path,
            _ => {
                bridge_response(stream, "404 Not Found", None, "");
                return;
            }
        }
    };
    if ACTIVE_SERVERS
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
            (count < 8).then_some(count + 1)
        })
        .is_err()
    {
        bridge_response(stream, "503 Service Unavailable", None, "");
        return;
    }
    let slot = ServerSlot;
    let result =
        tauri::async_runtime::block_on(async { multipath::server("0.0.0.0:0".parse().unwrap()) });
    let (endpoint, certificate) = match result {
        Ok(value) => value,
        Err(_) => {
            bridge_response(stream, "503 Service Unavailable", None, "");
            return;
        }
    };
    let ticket = Ticket {
        port: endpoint.local_addr().unwrap().port(),
        certificate,
        token: uuid::Uuid::new_v4().simple().to_string()
            + &uuid::Uuid::new_v4().simple().to_string(),
    };
    let body = serde_json::to_string(&ticket).unwrap();
    tauri::async_runtime::spawn(async move {
        let _slot = slot;
        // One short-lived, selected-file endpoint; the TLS password is never sent over QUIC.
        let _ = multipath::serve_once(&endpoint, &path, &ticket.token).await;
        endpoint.close(0u8.into(), b"ticket finished");
    });
    bridge_response(stream, "200 OK", None, &body);
}

pub(super) fn download_link_file(
    state: &AppState,
    id: &str,
    password: &str,
    path: &str,
    destination: &Path,
    reporter: &mut LinkTransferReporter,
) -> Result<(), String> {
    let (enabled, ips) = {
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        (
            settings.link_quic_enabled,
            settings.link_quic_local_ips.clone(),
        )
    };
    // Preserve the existing overwrite decision for destinations already selected by the user.
    if !enabled || destination.exists() {
        return download_link_file_http_to(state, id, password, path, destination, reporter);
    }
    let encoded = url::form_urlencoded::byte_serialize(path.as_bytes()).collect::<String>();
    let (response, _, _) = link_http_request(
        state,
        id,
        "GET",
        &format!("/v1/link/quic?path={encoded}"),
        Some(password),
        &[],
        None,
    )?;
    if matches!(response.status, 404 | 503) {
        return download_link_file_http_to(state, id, password, path, destination, reporter);
    }
    ensure_link_http_success(&response)?;
    let ticket: Ticket = serde_json::from_slice(&response.body).map_err(|e| e.to_string())?;
    if ticket.certificate.len() > 16384
        || ticket.token.len() < 32
        || ticket.token.len() > 128
        || ticket.port == 0
    {
        return Err("invalid_quic_ticket".into());
    }
    let (host, _, _) = link_remote_parts(id)?;
    let remote = (host.as_str(), ticket.port)
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .find(|addr| addr.is_ipv4())
        .ok_or("quic_ipv4_address_required")?;
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let started_bytes = reporter.transferred;
    let started_total = reporter.total;
    let control = reporter.control.clone();
    let result = tokio::runtime::Handle::current().block_on(async {
        let endpoint = multipath::client("0.0.0.0:0".parse().unwrap(), ticket.certificate)?;
        let transfer = multipath::download_with_progress(&endpoint, remote, &ips, &ticket.token, destination, u64::MAX, |received, total| {
            reporter.checkpoint().map_err(anyhow::Error::msg)?;
            reporter.set_total(total);
            let delta = started_bytes.saturating_add(received).saturating_sub(reporter.transferred);
            reporter.advance(delta);
            Ok(())
        });
        tokio::pin!(transfer);
        loop {
            tokio::select! {
                result = &mut transfer => break result,
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    if control.cancelled.load(Ordering::Acquire) { break Err(anyhow::anyhow!("cancelled")); }
                }
            }
        }
    });
    match result {
        Ok(report) => {
            diagnostic_log(
                state,
                "INFO",
                "link.quic_completed",
                &format!(
                    "bytes={} additional_paths={} failed_paths={} sha256={}",
                    report.bytes, report.additional_paths, report.failed_paths, report.sha256
                ),
            );
            let _ = reporter.app.emit("link-transport-used", serde_json::json!({"transport":"QUIC", "additionalPaths":report.additional_paths,"failedPaths":report.failed_paths}));
            Ok(())
        }
        Err(error) => {
            reporter.checkpoint()?;
            // Temporary QUIC output is removed before retry; old files are never removed.
            reporter.transferred = started_bytes;
            reporter.total = started_total;
            reporter.emit(true);
            diagnostic_log(state, "WARN", "link.quic_fallback", &error.to_string());
            let _ = reporter.app.emit(
                "link-transport-used",
                serde_json::json!({"transport":"HTTPS"}),
            );
            download_link_file_http_to(state, id, password, path, destination, reporter)
        }
    }
}

#[tauri::command]
pub async fn download_with_dictionary(
    dictionary_url: String,
    artifact_url: String,
    sha256: String,
) -> Result<String, String> {
    let dictionary_url = url::Url::parse(&dictionary_url).map_err(|e| e.to_string())?;
    let artifact_url = url::Url::parse(&artifact_url).map_err(|e| e.to_string())?;
    let name = artifact_url
        .path_segments()
        .and_then(|segments| segments.last())
        .filter(|name| !name.is_empty())
        .unwrap_or("download.bin");
    let Some(destination) = rfd::FileDialog::new().set_file_name(name).save_file() else {
        return Err("cancelled".into());
    };
    let data = dictionary::fetch(&dictionary_url, &artifact_url, &sha256, 256 * 1024 * 1024)
        .await
        .map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        let mut temporary =
            tempfile::NamedTempFile::new_in(destination.parent().ok_or("invalid_destination")?)
                .map_err(|e| e.to_string())?;
        temporary.write_all(&data).map_err(|e| e.to_string())?;
        temporary.as_file().sync_all().map_err(|e| e.to_string())?;
        temporary
            .persist_noclobber(&destination)
            .map_err(|e| e.to_string())?;
        Ok(destination.to_string_lossy().into_owned())
    })
    .await
    .map_err(|e| e.to_string())?
}

type Captures = Mutex<HashMap<String, tokio::sync::oneshot::Sender<()>>>;
static CAPTURES: OnceLock<Captures> = OnceLock::new();
struct CaptureSlot(String);
impl Drop for CaptureSlot {
    fn drop(&mut self) {
        if let Ok(mut jobs) = CAPTURES.get_or_init(Default::default).lock() {
            jobs.remove(&self.0);
        }
    }
}
#[tauri::command]
pub fn stop_transport_capture(job_id: String) -> Result<(), String> {
    let sender = CAPTURES
        .get_or_init(Default::default)
        .lock()
        .map_err(|e| e.to_string())?
        .remove(&job_id)
        .ok_or("capture_not_found")?;
    let _ = sender.send(());
    Ok(())
}
#[tauri::command]
pub async fn capture_moq_objects(
    relay: String,
    broadcast: String,
    track: String,
    seconds: u64,
    job_id: String,
) -> Result<String, String> {
    let relay = url::Url::parse(&relay).map_err(|e| e.to_string())?;
    if job_id.len() > 128 || seconds == 0 || seconds > 3600 {
        return Err("invalid_capture_parameters".into());
    }
    let Some(destination) = rfd::FileDialog::new()
        .set_file_name("capture.admmoq")
        .save_file()
    else {
        return Err("cancelled".into());
    };
    if destination.exists() {
        return Err("destination_already_exists".into());
    }
    let (sender, stop) = tokio::sync::oneshot::channel();
    {
        let mut jobs = CAPTURES
            .get_or_init(Default::default)
            .lock()
            .map_err(|e| e.to_string())?;
        if jobs.len() >= 4 || jobs.contains_key(&job_id) {
            return Err("capture_busy".into());
        }
        jobs.insert(job_id.clone(), sender);
    }
    let _slot = CaptureSlot(job_id);
    let temporary_directory =
        tempfile::tempdir_in(destination.parent().ok_or("invalid_destination")?)
            .map_err(|e| e.to_string())?;
    let temporary_capture = temporary_directory.path().join("objects");
    moqt::capture_with_stop(
        relay,
        &broadcast,
        &track,
        &temporary_capture,
        Duration::from_secs(seconds),
        256 * 1024 * 1024,
        async {
            let _ = stop.await;
        },
    )
    .await
    .map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        let mut file =
            tempfile::NamedTempFile::new_in(destination.parent().ok_or("invalid_destination")?)
                .map_err(|e| e.to_string())?;
        let mut input = fs::File::open(&temporary_capture).map_err(|e| e.to_string())?;
        std::io::copy(&mut input, &mut file).map_err(|e| e.to_string())?;
        file.as_file().sync_all().map_err(|e| e.to_string())?;
        file.persist_noclobber(&destination)
            .map_err(|e| e.to_string())?;
        drop(temporary_directory);
        Ok(destination.to_string_lossy().into_owned())
    })
    .await
    .map_err(|e| e.to_string())?
}
