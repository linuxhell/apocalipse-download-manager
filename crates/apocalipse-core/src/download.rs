use anyhow::{bail, Context, Result};
use futures_util::{stream::FuturesUnordered, StreamExt};
use hickory_resolver::{
    config::{NameServerConfigGroup, ResolverConfig},
    name_server::TokioConnectionProvider,
    TokioResolver,
};
use reqwest::{
    dns::{Addrs, Name, Resolve, Resolving},
    header, Client, RequestBuilder, StatusCode,
};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    sync::mpsc,
};

use crate::validation::{validate_payload, PayloadExpectation};

const SEGMENT_CHUNK_SIZE: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub destination: PathBuf,
    pub overwrite: bool,
    pub connections: usize,
    pub method: String,
    pub body: Option<Vec<u8>>,
    pub headers: Vec<(String, String)>,
    pub limiters: Vec<Arc<BandwidthLimiter>>,
}

#[derive(Debug)]
pub struct BandwidthLimiter {
    bytes_per_second: AtomicU64,
    next_slot: tokio::sync::Mutex<Instant>,
}

impl BandwidthLimiter {
    pub fn new(bytes_per_second: u64) -> Self {
        Self {
            bytes_per_second: AtomicU64::new(bytes_per_second),
            next_slot: tokio::sync::Mutex::new(Instant::now()),
        }
    }

    pub fn set_limit(&self, bytes_per_second: u64) {
        self.bytes_per_second
            .store(bytes_per_second, Ordering::Relaxed);
    }

    async fn acquire(&self, bytes: usize) {
        let rate = self.bytes_per_second.load(Ordering::Relaxed);
        if rate == 0 || bytes == 0 {
            return;
        }
        let spacing = Duration::from_secs_f64(bytes as f64 / rate as f64);
        let mut next = self.next_slot.lock().await;
        let now = Instant::now();
        if *next > now {
            tokio::time::sleep(*next - now).await;
        }
        *next = std::cmp::max(*next, now) + spacing;
    }
}

async fn apply_bandwidth_limits(limiters: &[Arc<BandwidthLimiter>], bytes: usize) {
    for limiter in limiters {
        limiter.acquire(bytes).await;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadEvent {
    Started {
        resumed_at: u64,
        total: Option<u64>,
        connections: usize,
        resume_supported: bool,
    },
    Progress {
        received: u64,
        total: Option<u64>,
    },
    Completed {
        bytes: u64,
    },
}

#[derive(Clone)]
pub struct DownloadEngine {
    client: Client,
}

#[derive(Clone)]
struct CustomDnsResolver {
    resolver: TokioResolver,
}

#[derive(Debug)]
struct SourceProbe {
    total: Option<u64>,
    etag: Option<String>,
    digest: Option<String>,
    elapsed: Duration,
}

fn same_download_identity(primary: &SourceProbe, candidate: &SourceProbe, advertised: bool) -> bool {
    if primary.total.is_some() && candidate.total.is_some() && primary.total != candidate.total {
        return false;
    }
    if let (Some(left), Some(right)) = (primary.digest.as_deref(), candidate.digest.as_deref()) {
        return left.eq_ignore_ascii_case(right);
    }
    if let (Some(left), Some(right)) = (primary.etag.as_deref(), candidate.etag.as_deref()) {
        if !left.starts_with("W/") && !right.starts_with("W/") {
            return left == right;
        }
    }
    advertised && primary.total.is_some() && primary.total == candidate.total
}

impl Resolve for CustomDnsResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let resolver = self.resolver.clone();
        Box::pin(async move {
            let lookup = resolver.lookup_ip(name.as_str()).await?;
            let addrs: Addrs = Box::new(
                lookup
                    .into_iter()
                    .map(|address| SocketAddr::new(address, 0)),
            );
            Ok(addrs)
        })
    }
}

impl DownloadEngine {
    pub fn new() -> Result<Self> {
        Self::with_network(None, None, None, &[])
    }

    pub fn with_proxy(
        proxy_url: Option<&str>,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<Self> {
        Self::with_network(proxy_url, username, password, &[])
    }

    pub fn with_network(
        proxy_url: Option<&str>,
        username: Option<&str>,
        password: Option<&str>,
        dns_servers: &[IpAddr],
    ) -> Result<Self> {
        let client =
            Self::network_client_builder(proxy_url, username, password, dns_servers)?.build()?;
        Ok(Self { client })
    }

    /// Shared proxy/DNS settings with a caller-controlled redirect policy for preview.
    pub fn network_client_builder(
        proxy_url: Option<&str>,
        username: Option<&str>,
        password: Option<&str>,
        dns_servers: &[IpAddr],
    ) -> Result<reqwest::ClientBuilder> {
        let mut builder = Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .read_timeout(Duration::from_secs(60))
            .redirect(reqwest::redirect::Policy::limited(10))
            .pool_max_idle_per_host(32)
            .tcp_nodelay(true)
            .user_agent(concat!(
                "ApocalipseDownloadManager/",
                env!("CARGO_PKG_VERSION")
            ));
        if let Some(url) = proxy_url.map(str::trim).filter(|value| !value.is_empty()) {
            let mut proxy = reqwest::Proxy::all(url).context("invalid proxy URL")?;
            if let Some(user) = username.filter(|value| !value.is_empty()) {
                proxy = proxy.basic_auth(user, password.unwrap_or_default());
            }
            builder = builder.proxy(proxy);
        }
        if !dns_servers.is_empty() {
            let name_servers = NameServerConfigGroup::from_ips_clear(dns_servers, 53, true);
            let config = ResolverConfig::from_parts(None, Vec::new(), name_servers);
            let resolver =
                TokioResolver::builder_with_config(config, TokioConnectionProvider::default())
                    .build();
            builder = builder.dns_resolver(Arc::new(CustomDnsResolver { resolver }));
        }
        Ok(builder)
    }

    pub async fn download(
        &self,
        request: DownloadRequest,
        events: mpsc::Sender<DownloadEvent>,
    ) -> Result<()> {
        if request.destination.exists() && !request.overwrite {
            bail!("destination already exists");
        }
        if let Some(parent) = request.destination.parent() {
            fs::create_dir_all(parent).await?;
        }
        let can_segment = request.method.eq_ignore_ascii_case("GET") && request.body.is_none();
        let requested = request.connections.clamp(1, 32);
        let head = if can_segment && requested > 1 {
            apply_headers(self.client.head(&request.url), &request.headers)
                .send()
                .await
                .ok()
        } else {
            None
        };
        let total = head.as_ref().and_then(|response| response.content_length());
        if can_segment && requested > 1 {
            let probe = apply_headers(self.client.get(&request.url), &request.headers)
                .header(header::RANGE, "bytes=0-0")
                .send()
                .await;
            if let Ok(probe) = probe {
                if probe.status() == StatusCode::PARTIAL_CONTENT {
                    let range_total = probe
                        .headers()
                        .get(header::CONTENT_RANGE)
                        .and_then(|value| value.to_str().ok())
                        .and_then(content_range_total)
                        .or(total);
                    if let Some(total) = range_total {
                        let useful_connections = requested.min(total.div_ceil(4_194_304) as usize);
                        if useful_connections > 1 {
                            let mut attempt_connections = useful_connections;
                            loop {
                                let segmented = self
                                    .download_segmented(
                                        request.clone(),
                                        events.clone(),
                                        total,
                                        attempt_connections,
                                    )
                                    .await;
                                match segmented {
                                    Ok(()) => return Ok(()),
                                    Err(error)
                                        if error.chain().any(|cause| {
                                            cause
                                                .to_string()
                                                .contains("server stopped supporting byte ranges")
                                        }) =>
                                    {
                                        // File hosts may briefly reject part of a parallel burst.
                                        // Reduce concurrency progressively before giving up on
                                        // byte ranges and falling back to a single stream.
                                        cleanup_chunk_artifacts(&request.destination).await?;
                                        if attempt_connections > 2 {
                                            attempt_connections = (attempt_connections / 2).max(2);
                                            continue;
                                        }
                                        return self.download_single(request, events).await;
                                    }
                                    Err(error) => return Err(error),
                                }
                            }
                        }
                    }
                }
            }
        }
        self.download_single(request, events).await
    }

    /// Finds server-advertised duplicate resources and ranks every verified
    /// source by probe latency. Candidates are never inferred from host names.
    pub async fn verified_sources(
        &self,
        request: &DownloadRequest,
        supplied_mirrors: &[String],
    ) -> Vec<String> {
        let advertised = self.advertised_mirrors(request).await;
        let advertised_set = advertised.iter().cloned().collect::<HashSet<_>>();
        let mut candidates = vec![request.url.clone()];
        candidates.extend(advertised);
        candidates.extend(supplied_mirrors.iter().cloned());
        let mut seen = HashSet::new();
        candidates.retain(|url| seen.insert(url.clone()));

        let primary = self.probe_source(request, &request.url).await;
        let Some(primary_identity) = primary.as_ref() else {
            return candidates;
        };
        let mut probes = FuturesUnordered::new();
        for url in candidates {
            let engine = self.clone();
            let request = request.clone();
            let server_advertised = advertised_set.contains(&url);
            probes.push(async move {
                let probe = engine.probe_source(&request, &url).await?;
                Some((url, probe, server_advertised))
            });
        }
        let mut verified = Vec::new();
        while let Some(Some((url, probe, server_advertised))) = probes.next().await {
            if url == request.url || same_download_identity(primary_identity, &probe, server_advertised) {
                verified.push((url, probe.elapsed));
            }
        }
        verified.sort_by_key(|(_, elapsed)| *elapsed);
        if verified.is_empty() {
            vec![request.url.clone()]
        } else {
            verified.into_iter().map(|(url, _)| url).collect()
        }
    }

    async fn advertised_mirrors(&self, request: &DownloadRequest) -> Vec<String> {
        let response = apply_headers(self.client.head(&request.url), &request.headers)
            .send()
            .await;
        let Ok(response) = response else { return Vec::new() };
        response
            .headers()
            .get_all(header::LINK)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(|value| value.split(','))
            .filter(|part| {
                let lower = part.to_ascii_lowercase();
                lower.contains("rel=\"duplicate\"")
                    || lower.contains("rel=duplicate")
                    || lower.contains("rel=\"mirror\"")
                    || lower.contains("rel=mirror")
            })
            .filter_map(|part| part.split_once('<')?.1.split_once('>').map(|value| value.0.trim()))
            .filter_map(|value| reqwest::Url::parse(&request.url).ok()?.join(value).ok())
            .filter(|url| matches!(url.scheme(), "http" | "https") && url.username().is_empty() && url.password().is_none())
            .map(|url| url.to_string())
            .take(10)
            .collect()
    }

    async fn probe_source(&self, request: &DownloadRequest, url: &str) -> Option<SourceProbe> {
        let started = Instant::now();
        let response = apply_headers(self.client.get(url), &request.headers)
            .header(header::RANGE, "bytes=0-0")
            .send()
            .await
            .ok()?;
        if !(response.status().is_success() || response.status() == StatusCode::PARTIAL_CONTENT) {
            return None;
        }
        let headers = response.headers();
        let total = headers
            .get(header::CONTENT_RANGE)
            .and_then(|value| value.to_str().ok())
            .and_then(content_range_total)
            .or_else(|| response.content_length());
        Some(SourceProbe {
            total,
            etag: headers.get(header::ETAG).and_then(|value| value.to_str().ok()).map(str::to_owned),
            digest: headers.get("digest").or_else(|| headers.get("content-md5"))
                .and_then(|value| value.to_str().ok()).map(str::to_owned),
            elapsed: started.elapsed(),
        })
    }

    async fn download_single(
        &self,
        request: DownloadRequest,
        events: mpsc::Sender<DownloadEvent>,
    ) -> Result<()> {
        let partial = partial_path(&request.destination);
        let existing = fs::metadata(&partial)
            .await
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        let method = reqwest::Method::from_bytes(request.method.as_bytes())
            .context("invalid HTTP method")?;
        let mut builder = self.client.request(method, &request.url);
        for (name, value) in &request.headers {
            builder = builder.header(name.as_str(), value.as_str());
        }
        if let Some(body) = &request.body {
            builder = builder.body(body.clone());
        }
        if existing > 0 && request.method.eq_ignore_ascii_case("GET") && request.body.is_none() {
            builder = builder.header(header::RANGE, format!("bytes={existing}-"));
        }
        let response = builder.send().await?.error_for_status()?;
        let resumed = existing > 0 && response.status() == StatusCode::PARTIAL_CONTENT;
        let resume_supported = resumed
            || response
                .headers()
                .get(header::ACCEPT_RANGES)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.eq_ignore_ascii_case("bytes"));
        let start = if resumed { existing } else { 0 };
        let total = response.content_length().map(|size| size + start);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let expectation = payload_expectation(&request.destination);
        let _ = events
            .send(DownloadEvent::Started {
                resumed_at: start,
                total,
                connections: 1,
                resume_supported,
            })
            .await;
        let file = if resumed {
            fs::OpenOptions::new().append(true).open(&partial).await?
        } else {
            fs::File::create(&partial).await?
        };
        let mut file = BufWriter::with_capacity(1024 * 1024, file);
        let mut received = start;
        let mut stream = response.bytes_stream();
        let mut inspected = resumed;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("network stream failed")?;
            apply_bandwidth_limits(&request.limiters, chunk.len()).await;
            if !inspected {
                validate_payload(expectation, content_type.as_deref(), &chunk)?;
                inspected = true;
            }
            file.write_all(&chunk).await?;
            received += chunk.len() as u64;
            let _ = events.try_send(DownloadEvent::Progress { received, total });
        }
        file.flush().await?;
        finish_download(&request, &partial, received, total, &events).await
    }

    async fn download_segmented(
        &self,
        request: DownloadRequest,
        events: mpsc::Sender<DownloadEvent>,
        total: u64,
        connections: usize,
    ) -> Result<()> {
        let progress = Arc::new(AtomicU64::new(0));
        let chunk_count = total.div_ceil(SEGMENT_CHUNK_SIZE) as usize;
        let worker_count = connections.min(chunk_count);
        let next_chunk = Arc::new(AtomicUsize::new(0));

        fs::create_dir_all(chunk_directory(&request.destination)).await?;

        for index in 0..chunk_count {
            let start = index as u64 * SEGMENT_CHUNK_SIZE;
            let expected = (total - start).min(SEGMENT_CHUNK_SIZE);
            let chunk = chunk_path(&request.destination, index);
            let legacy = legacy_chunk_path(&request.destination, index);
            if fs::metadata(&chunk).await.is_err()
                && fs::metadata(&legacy).await.is_ok()
                && fs::rename(&legacy, &chunk).await.is_err()
            {
                fs::copy(&legacy, &chunk).await?;
                fs::remove_file(&legacy).await?;
            }
            let existing = fs::metadata(&chunk)
                .await
                .map(|metadata| metadata.len())
                .unwrap_or(0);
            if existing <= expected {
                progress.fetch_add(existing, Ordering::Relaxed);
            } else {
                fs::remove_file(chunk).await?;
            }
        }

        let mut jobs = FuturesUnordered::new();
        for worker_index in 0..worker_count {
            let client = self.client.clone();
            let url = request.url.clone();
            let headers = request.headers.clone();
            let destination = request.destination.clone();
            let sender = events.clone();
            let shared = progress.clone();
            let cursor = next_chunk.clone();
            let limiters = request.limiters.clone();
            jobs.push(async move {
                if worker_index > 0 {
                    tokio::time::sleep(Duration::from_millis(worker_index as u64 * 150)).await;
                }
                loop {
                    let index = cursor.fetch_add(1, Ordering::Relaxed);
                    if index >= chunk_count {
                        break;
                    }
                    let start = index as u64 * SEGMENT_CHUNK_SIZE;
                    let end = (start + SEGMENT_CHUNK_SIZE).min(total) - 1;
                    let expected = end - start + 1;
                    let segment = chunk_path(&destination, index);
                    let existing = fs::metadata(&segment)
                        .await
                        .map(|metadata| metadata.len())
                        .unwrap_or(0);
                    if existing == expected {
                        continue;
                    }
                    let response = apply_headers(client.get(&url), &headers)
                        .header(header::RANGE, format!("bytes={}-{}", start + existing, end))
                        .send()
                        .await?;
                    if response.status() != StatusCode::PARTIAL_CONTENT {
                        bail!(
                            "server stopped supporting byte ranges: status {} for bytes={}-{}",
                            response.status(),
                            start + existing,
                            end
                        );
                    }
                    let file = if existing > 0 {
                        fs::OpenOptions::new().append(true).open(&segment).await?
                    } else {
                        fs::File::create(&segment).await?
                    };
                    let mut file = BufWriter::with_capacity(1024 * 1024, file);
                    let mut downloaded = existing;
                    let mut stream = response.bytes_stream();
                    while let Some(chunk) = stream.next().await {
                        let chunk = chunk.context("segmented network stream failed")?;
                        apply_bandwidth_limits(&limiters, chunk.len()).await;
                        file.write_all(&chunk).await?;
                        downloaded += chunk.len() as u64;
                        let received = shared.fetch_add(chunk.len() as u64, Ordering::Relaxed)
                            + chunk.len() as u64;
                        let _ = sender.try_send(DownloadEvent::Progress {
                            received,
                            total: Some(total),
                        });
                    }
                    file.flush().await?;
                    if downloaded != expected {
                        bail!("incomplete segment: received {downloaded} of {expected} bytes");
                    }
                }
                Result::<()>::Ok(())
            });
        }
        let resumed = progress.load(Ordering::Relaxed);
        let _ = events
            .send(DownloadEvent::Started {
                resumed_at: resumed,
                total: Some(total),
                connections: worker_count,
                resume_supported: true,
            })
            .await;
        while let Some(result) = jobs.next().await {
            result?;
        }
        let partial = partial_path(&request.destination);
        let mut output = fs::File::create(&partial).await?;
        let mut buffer = vec![0_u8; 4 * 1024 * 1024];
        for index in 0..chunk_count {
            let segment = chunk_path(&request.destination, index);
            let mut input = fs::File::open(&segment).await?;
            loop {
                let count = input.read(&mut buffer).await?;
                if count == 0 {
                    break;
                }
                output.write_all(&buffer[..count]).await?;
            }
            fs::remove_file(segment).await?;
        }
        output.flush().await?;
        let _ = cleanup_chunk_artifacts(&request.destination).await;
        finish_download(&request, &partial, total, Some(total), &events).await
    }
}

fn apply_headers(mut builder: RequestBuilder, headers: &[(String, String)]) -> RequestBuilder {
    for (name, value) in headers {
        if ["range", "content-length", "connection", "host"]
            .iter()
            .any(|blocked| name.eq_ignore_ascii_case(blocked))
        {
            continue;
        }
        builder = builder.header(name.as_str(), value.as_str());
    }
    builder
}

fn content_range_total(value: &str) -> Option<u64> {
    value.rsplit_once('/')?.1.trim().parse().ok()
}

fn payload_expectation(destination: &Path) -> PayloadExpectation {
    match destination
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("zip") => PayloadExpectation::Zip,
        Some(_) => PayloadExpectation::Binary,
        None => PayloadExpectation::Any,
    }
}

async fn finish_download(
    request: &DownloadRequest,
    partial: &Path,
    received: u64,
    total: Option<u64>,
    events: &mpsc::Sender<DownloadEvent>,
) -> Result<()> {
    if let Some(expected) = total {
        if received != expected {
            bail!("incomplete download: received {received} of {expected} bytes");
        }
    }
    if request.overwrite && request.destination.exists() {
        fs::remove_file(&request.destination).await?;
    }
    fs::rename(partial, &request.destination).await?;
    let _ = events
        .send(DownloadEvent::Completed { bytes: received })
        .await;
    Ok(())
}

pub fn partial_path(destination: &Path) -> PathBuf {
    destination.with_extension(format!(
        "{}part",
        destination
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| format!("{value}."))
            .unwrap_or_default()
    ))
}

pub fn segment_path(destination: &Path, index: usize) -> PathBuf {
    PathBuf::from(format!("{}.part.{index:02}", destination.display()))
}

pub fn chunk_directory(destination: &Path) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(destination.to_string_lossy().as_bytes());
    let identifier = hasher
        .finalize()
        .iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    destination
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(".apocalipse-parts")
        .join(identifier)
}

fn chunk_path(destination: &Path, index: usize) -> PathBuf {
    chunk_directory(destination).join(format!("{index:06}.part"))
}

fn legacy_chunk_path(destination: &Path, index: usize) -> PathBuf {
    PathBuf::from(format!("{}.part.chunk.{index:06}", destination.display()))
}

pub async fn cleanup_chunk_artifacts(destination: &Path) -> Result<()> {
    let directory = chunk_directory(destination);
    match fs::remove_dir_all(&directory).await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    if let Some(root) = directory.parent() {
        let _ = fs::remove_dir(root).await;
    }

    let Some(parent) = destination.parent() else {
        return Ok(());
    };
    let Some(file_name) = destination.file_name().and_then(|value| value.to_str()) else {
        return Ok(());
    };
    let prefix = format!("{file_name}.part.chunk.");
    let mut entries = match fs::read_dir(parent).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let Some(suffix) = name.strip_prefix(&prefix) else {
            continue;
        };
        if suffix.len() == 6 && suffix.bytes().all(|byte| byte.is_ascii_digit()) {
            match fs::remove_file(entry.path()).await {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
    }
    for index in 0..32 {
        match fs::remove_file(segment_path(destination, index)).await {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_names_are_deterministic() {
        assert_eq!(
            partial_path(Path::new("video.mp4")),
            PathBuf::from("video.mp4.part")
        );
        assert_eq!(
            segment_path(Path::new("video.mp4"), 7),
            PathBuf::from("video.mp4.part.07")
        );
    }

    #[test]
    fn reads_total_size_from_content_range() {
        assert_eq!(
            content_range_total("bytes 0-0/5368709120"),
            Some(5_368_709_120)
        );
        assert_eq!(content_range_total("bytes */4096"), Some(4096));
        assert_eq!(content_range_total("bytes 0-0/*"), None);
    }

    #[test]
    fn adaptive_chunk_names_do_not_collide_with_legacy_segments() {
        assert_eq!(
            chunk_path(Path::new("image.iso"), 42),
            chunk_directory(Path::new("image.iso")).join("000042.part")
        );
        assert_ne!(
            chunk_path(Path::new("image.iso"), 0),
            segment_path(Path::new("image.iso"), 0)
        );
    }

    #[test]
    fn different_destinations_have_different_chunk_directories() {
        assert_ne!(
            chunk_directory(Path::new("first.iso")),
            chunk_directory(Path::new("second.iso"))
        );
    }

    #[test]
    fn mirror_identity_rejects_different_files() {
        let primary = SourceProbe {
            total: Some(10_000),
            etag: Some("\"file-a\"".into()),
            digest: None,
            elapsed: Duration::from_millis(20),
        };
        let different_size = SourceProbe {
            total: Some(9_999),
            etag: Some("\"file-a\"".into()),
            digest: None,
            elapsed: Duration::from_millis(10),
        };
        let different_etag = SourceProbe {
            total: Some(10_000),
            etag: Some("\"file-b\"".into()),
            digest: None,
            elapsed: Duration::from_millis(10),
        };
        assert!(!same_download_identity(&primary, &different_size, true));
        assert!(!same_download_identity(&primary, &different_etag, false));
    }

    #[test]
    fn server_advertised_duplicate_requires_the_same_size_without_a_hash() {
        let primary = SourceProbe {
            total: Some(10_000),
            etag: None,
            digest: None,
            elapsed: Duration::from_millis(20),
        };
        let duplicate = SourceProbe {
            total: Some(10_000),
            etag: None,
            digest: None,
            elapsed: Duration::from_millis(10),
        };
        assert!(same_download_identity(&primary, &duplicate, true));
        assert!(!same_download_identity(&primary, &duplicate, false));
    }

    #[tokio::test]
    async fn cleanup_only_removes_chunks_for_the_exact_destination() {
        let root =
            std::env::temp_dir().join(format!("apocalipse-chunk-cleanup-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let destination = root.join("image.iso");
        let chunk = legacy_chunk_path(&destination, 1);
        let unrelated = root.join("image.iso.part.chunk.backup");
        std::fs::write(&chunk, b"chunk").unwrap();
        std::fs::write(&unrelated, b"keep").unwrap();

        cleanup_chunk_artifacts(&destination).await.unwrap();

        assert!(!chunk.exists());
        assert!(unrelated.exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
