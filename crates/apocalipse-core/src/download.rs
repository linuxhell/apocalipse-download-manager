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
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
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

const MIN_SEGMENT_CHUNK_SIZE: u64 = 4 * 1024 * 1024;
const MAX_SEGMENT_CHUNK_SIZE: u64 = 64 * 1024 * 1024;
const WORKER_START_INTERVAL_MS: u64 = 35;

fn adaptive_chunk_size(total: u64, connections: usize) -> u64 {
    let target_chunks = connections.clamp(1, 32) as u64 * 8;
    let raw = total.div_ceil(target_chunks.max(1));
    let mib = 1024 * 1024;
    raw.div_ceil(mib)
        .saturating_mul(mib)
        .clamp(MIN_SEGMENT_CHUNK_SIZE, MAX_SEGMENT_CHUNK_SIZE)
}

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

async fn request_range_from_sources(
    client: &Client,
    sources: &[String],
    headers: &[(String, String)],
    identity: &ResumeIdentity,
    start: u64,
    end: u64,
    total: u64,
    chunk_index: usize,
) -> Result<reqwest::Response> {
    let mut last_error = None;
    for offset in 0..sources.len() {
        let source = &sources[(chunk_index + offset) % sources.len()];
        let mut builder = apply_headers(client.get(source), headers)
            .header(header::RANGE, format!("bytes={start}-{end}"));
        if let Some(validator) = if_range_value(identity) {
            builder = builder.header(header::IF_RANGE, validator);
        }
        match builder.send().await {
            Ok(response)
                if response.status() == StatusCode::PARTIAL_CONTENT
                    && response
                        .headers()
                        .get(header::CONTENT_RANGE)
                        .and_then(|value| value.to_str().ok())
                        .is_some_and(|value| content_range_matches(value, start, end, total)) =>
            {
                return Ok(response);
            }
            Ok(response) => {
                last_error = Some(anyhow::anyhow!(
                    "source rejected bytes={start}-{end} with status {} or invalid Content-Range",
                    response.status()
                ));
            }
            Err(error) => last_error = Some(error.into()),
        }
    }
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("no verified source available")))
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
    host_profiles: Arc<std::sync::Mutex<HashMap<String, HostProfile>>>,
}

#[derive(Debug, Clone, Copy)]
struct HostProfile {
    stable_connections: usize,
}

#[derive(Clone)]
struct CustomDnsResolver {
    resolver: TokioResolver,
}

#[derive(Debug)]
struct SourceProbe {
    total: Option<u64>,
    etag: Option<String>,
    last_modified: Option<String>,
    digest: Option<String>,
    elapsed: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ResumeIdentity {
    source_hash: String,
    total: u64,
    etag: Option<String>,
    last_modified: Option<String>,
    chunk_size: u64,
}

fn source_hash(url: &str) -> String {
    format!("{:x}", Sha256::digest(url.as_bytes()))
}

fn if_range_value(identity: &ResumeIdentity) -> Option<&str> {
    identity
        .etag
        .as_deref()
        .filter(|value| !value.starts_with("W/"))
        .or(identity.last_modified.as_deref())
}

fn resume_manifest_path(destination: &Path) -> PathBuf {
    chunk_directory(destination).join("manifest.json")
}

fn same_download_identity(
    primary: &SourceProbe,
    candidate: &SourceProbe,
    advertised: bool,
) -> bool {
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
        Ok(Self {
            client,
            host_profiles: Arc::new(std::sync::Mutex::new(HashMap::new())),
        })
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
        let source = request.url.clone();
        self.download_with_sources(request, events, Arc::new(vec![source]))
            .await
    }

    pub async fn download_verified_sources(
        &self,
        mut request: DownloadRequest,
        sources: Vec<String>,
        events: mpsc::Sender<DownloadEvent>,
    ) -> Result<()> {
        let sources = if sources.is_empty() {
            vec![request.url.clone()]
        } else {
            sources
        };
        request.url = sources[0].clone();
        let pooled = self
            .download_with_sources(request.clone(), events.clone(), Arc::new(sources.clone()))
            .await;
        if pooled.is_ok() || sources.len() == 1 {
            return pooled;
        }
        cleanup_chunk_artifacts(&request.destination).await?;
        let mut last_error = pooled.err();
        for source in sources {
            let mut attempt = request.clone();
            attempt.url = source.clone();
            match self
                .download_with_sources(attempt, events.clone(), Arc::new(vec![source]))
                .await
            {
                Ok(()) => return Ok(()),
                Err(error) => last_error = Some(error),
            }
        }
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("no verified source available")))
    }

    async fn download_with_sources(
        &self,
        request: DownloadRequest,
        events: mpsc::Sender<DownloadEvent>,
        sources: Arc<Vec<String>>,
    ) -> Result<()> {
        let mut request = request;
        if request.destination.exists() && !request.overwrite {
            bail!("destination already exists");
        }
        if let Some(parent) = request.destination.parent() {
            fs::create_dir_all(parent).await?;
        }
        let can_segment = request.method.eq_ignore_ascii_case("GET") && request.body.is_none();
        let requested = self.profiled_connections(&request.url, request.connections.clamp(1, 32));
        let mut head = if can_segment && requested > 1 {
            apply_headers(self.client.head(&request.url), &request.headers)
                .send()
                .await
                .ok()
        } else {
            None
        };
        if can_segment && requested > 1 {
            let mut probe = apply_headers(self.client.get(&request.url), &request.headers)
                .header(header::RANGE, "bytes=0-0")
                .send()
                .await;
            if probe
                .as_ref()
                .is_ok_and(|response| is_optional_referer_rejection(response.status()))
                && remove_header(&mut request.headers, "referer")
            {
                // Some cross-origin media CDNs reject the embedding page as a
                // Referer even though the exact URL is public and succeeds in
                // the browser. Retry once without that optional header.
                head = apply_headers(self.client.head(&request.url), &request.headers)
                    .send()
                    .await
                    .ok();
                probe = apply_headers(self.client.get(&request.url), &request.headers)
                    .header(header::RANGE, "bytes=0-0")
                    .send()
                    .await;
            }
            let total = head.as_ref().and_then(|response| response.content_length());
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
                            let identity = ResumeIdentity {
                                source_hash: source_hash(&request.url),
                                total,
                                etag: probe
                                    .headers()
                                    .get(header::ETAG)
                                    .and_then(|value| value.to_str().ok())
                                    .map(str::to_owned),
                                last_modified: probe
                                    .headers()
                                    .get(header::LAST_MODIFIED)
                                    .and_then(|value| value.to_str().ok())
                                    .map(str::to_owned),
                                chunk_size: adaptive_chunk_size(total, useful_connections),
                            };
                            let mut attempt_connections = useful_connections;
                            loop {
                                let segmented = self
                                    .download_segmented(
                                        request.clone(),
                                        events.clone(),
                                        total,
                                        attempt_connections,
                                        identity.clone(),
                                        sources.clone(),
                                    )
                                    .await;
                                match segmented {
                                    Ok(()) => {
                                        self.record_stable_connections(
                                            &request.url,
                                            attempt_connections,
                                        );
                                        return Ok(());
                                    }
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
                                        self.record_unstable_connections(
                                            &request.url,
                                            attempt_connections,
                                        );
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

    fn profiled_connections(&self, url: &str, requested: usize) -> usize {
        let Some(host) = reqwest::Url::parse(url)
            .ok()
            .and_then(|value| value.host_str().map(str::to_ascii_lowercase))
        else {
            return requested;
        };
        self.host_profiles
            .lock()
            .ok()
            .and_then(|profiles| profiles.get(&host).copied())
            .map_or(requested, |profile| {
                requested.min(profile.stable_connections)
            })
            .clamp(1, 32)
    }

    fn record_stable_connections(&self, url: &str, connections: usize) {
        let Some(host) = reqwest::Url::parse(url)
            .ok()
            .and_then(|value| value.host_str().map(str::to_ascii_lowercase))
        else {
            return;
        };
        if let Ok(mut profiles) = self.host_profiles.lock() {
            profiles.insert(
                host,
                HostProfile {
                    stable_connections: connections.clamp(1, 32),
                },
            );
        }
    }

    fn record_unstable_connections(&self, url: &str, connections: usize) {
        self.record_stable_connections(url, (connections / 2).max(1));
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
            if url == request.url
                || same_download_identity(primary_identity, &probe, server_advertised)
            {
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
        let Ok(response) = response else {
            return Vec::new();
        };
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
            .filter_map(|part| {
                part.split_once('<')?
                    .1
                    .split_once('>')
                    .map(|value| value.0.trim())
            })
            .filter_map(|value| reqwest::Url::parse(&request.url).ok()?.join(value).ok())
            .filter(|url| {
                matches!(url.scheme(), "http" | "https")
                    && url.username().is_empty()
                    && url.password().is_none()
            })
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
            etag: headers
                .get(header::ETAG)
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned),
            last_modified: headers
                .get(header::LAST_MODIFIED)
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned),
            digest: headers
                .get("digest")
                .or_else(|| headers.get("content-md5"))
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned),
            elapsed: started.elapsed(),
        })
    }

    async fn download_single(
        &self,
        request: DownloadRequest,
        events: mpsc::Sender<DownloadEvent>,
    ) -> Result<()> {
        let partial = partial_path(&request.destination);
        let mut existing = fs::metadata(&partial)
            .await
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        let mut stored_identity = load_resume_manifest(&request.destination).await;
        if existing > 0
            && (!request.method.eq_ignore_ascii_case("GET")
                || request.body.is_some()
                || stored_identity.as_ref().is_none_or(|identity| {
                    identity.source_hash != source_hash(&request.url) || identity.chunk_size != 0
                }))
        {
            fs::remove_file(&partial).await?;
            cleanup_chunk_artifacts(&request.destination).await?;
            existing = 0;
            stored_identity = None;
        }
        let method = reqwest::Method::from_bytes(request.method.as_bytes())
            .context("invalid HTTP method")?;
        let send = |headers: &[(String, String)]| {
            let mut builder =
                apply_headers(self.client.request(method.clone(), &request.url), headers);
            if let Some(body) = &request.body {
                builder = builder.body(body.clone());
            }
            if existing > 0 && request.method.eq_ignore_ascii_case("GET") && request.body.is_none()
            {
                builder = builder.header(header::RANGE, format!("bytes={existing}-"));
                if let Some(validator) = stored_identity.as_ref().and_then(if_range_value) {
                    builder = builder.header(header::IF_RANGE, validator);
                }
            }
            builder
        };
        let mut response = send(&request.headers).send().await?;
        let mut effective_headers = request.headers.clone();
        if is_optional_referer_rejection(response.status())
            && remove_header(&mut effective_headers, "referer")
        {
            response = send(&effective_headers).send().await?;
        }
        let response = response.error_for_status()?;
        let resumed = existing > 0 && response.status() == StatusCode::PARTIAL_CONTENT;
        let resume_supported = resumed
            || response
                .headers()
                .get(header::ACCEPT_RANGES)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.eq_ignore_ascii_case("bytes"));
        let start = if resumed { existing } else { 0 };
        let total = response.content_length().map(|size| size + start);
        if resumed
            && !total.is_some_and(|total| {
                response
                    .headers()
                    .get(header::CONTENT_RANGE)
                    .and_then(|value| value.to_str().ok())
                    .is_some_and(|value| {
                        content_range_matches(value, existing, total.saturating_sub(1), total)
                    })
            })
        {
            bail!("server returned an invalid content range while resuming");
        }
        if let Some(total) = total {
            let identity = ResumeIdentity {
                source_hash: source_hash(&request.url),
                total,
                etag: response
                    .headers()
                    .get(header::ETAG)
                    .and_then(|value| value.to_str().ok())
                    .map(str::to_owned),
                last_modified: response
                    .headers()
                    .get(header::LAST_MODIFIED)
                    .and_then(|value| value.to_str().ok())
                    .map(str::to_owned),
                chunk_size: 0,
            };
            prepare_resume_manifest(&request.destination, &identity).await?;
        }
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
        file.get_ref().sync_all().await?;
        let result = finish_download(&request, &partial, received, total, &events).await;
        if result.is_ok() {
            cleanup_chunk_artifacts(&request.destination).await?;
        }
        result
    }

    async fn download_segmented(
        &self,
        request: DownloadRequest,
        events: mpsc::Sender<DownloadEvent>,
        total: u64,
        connections: usize,
        identity: ResumeIdentity,
        sources: Arc<Vec<String>>,
    ) -> Result<()> {
        let progress = Arc::new(AtomicU64::new(0));
        let chunk_size = identity.chunk_size;
        let chunk_count = total.div_ceil(chunk_size) as usize;
        let worker_count = connections.min(chunk_count);
        let next_chunk = Arc::new(AtomicUsize::new(0));

        prepare_resume_manifest(&request.destination, &identity).await?;

        for index in 0..chunk_count {
            let start = index as u64 * chunk_size;
            let expected = (total - start).min(chunk_size);
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
            if existing == expected {
                if verify_or_record_chunk(&chunk).await? {
                    progress.fetch_add(existing, Ordering::Relaxed);
                } else {
                    fs::remove_file(&chunk).await?;
                    let _ = fs::remove_file(chunk_hash_path(&chunk)).await;
                }
            } else if existing < expected {
                progress.fetch_add(existing, Ordering::Relaxed);
            } else {
                fs::remove_file(chunk).await?;
            }
        }

        let mut jobs = FuturesUnordered::new();
        for worker_index in 0..worker_count {
            let client = self.client.clone();
            let sources = sources.clone();
            let headers = request.headers.clone();
            let destination = request.destination.clone();
            let sender = events.clone();
            let shared = progress.clone();
            let cursor = next_chunk.clone();
            let limiters = request.limiters.clone();
            let identity = identity.clone();
            jobs.push(async move {
                if worker_index > 0 {
                    tokio::time::sleep(Duration::from_millis(
                        worker_index as u64 * WORKER_START_INTERVAL_MS,
                    ))
                    .await;
                }
                loop {
                    let index = cursor.fetch_add(1, Ordering::Relaxed);
                    if index >= chunk_count {
                        break;
                    }
                    let start = index as u64 * chunk_size;
                    let end = (start + chunk_size).min(total) - 1;
                    let expected = end - start + 1;
                    let segment = chunk_path(&destination, index);
                    let existing = fs::metadata(&segment)
                        .await
                        .map(|metadata| metadata.len())
                        .unwrap_or(0);
                    if existing == expected {
                        continue;
                    }
                    let requested_start = start + existing;
                    let response = request_range_from_sources(
                        &client,
                        &sources,
                        &headers,
                        &identity,
                        requested_start,
                        end,
                        total,
                        index,
                    )
                    .await?;
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
                    file.get_ref().sync_data().await?;
                    if downloaded != expected {
                        bail!("incomplete segment: received {downloaded} of {expected} bytes");
                    }
                    record_chunk_hash(&segment).await?;
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
        output.sync_all().await?;
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

fn is_optional_referer_rejection(status: StatusCode) -> bool {
    matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN)
}

fn remove_header(headers: &mut Vec<(String, String)>, target: &str) -> bool {
    let previous = headers.len();
    headers.retain(|(name, _)| !name.eq_ignore_ascii_case(target));
    headers.len() != previous
}

async fn prepare_resume_manifest(destination: &Path, identity: &ResumeIdentity) -> Result<()> {
    let manifest_path = resume_manifest_path(destination);
    let existing = fs::read(&manifest_path)
        .await
        .ok()
        .and_then(|data| serde_json::from_slice::<ResumeIdentity>(&data).ok());
    if existing.as_ref().is_some_and(|stored| stored != identity)
        || (manifest_path.exists() && existing.is_none())
    {
        cleanup_chunk_artifacts(destination).await?;
    }
    fs::create_dir_all(chunk_directory(destination)).await?;
    if existing.as_ref() == Some(identity) {
        return Ok(());
    }
    let temporary = manifest_path.with_extension("json.tmp");
    let data = serde_json::to_vec(identity)?;
    let mut file = fs::File::create(&temporary).await?;
    file.write_all(&data).await?;
    file.sync_all().await?;
    fs::rename(temporary, manifest_path).await?;
    Ok(())
}

async fn load_resume_manifest(destination: &Path) -> Option<ResumeIdentity> {
    let data = fs::read(resume_manifest_path(destination)).await.ok()?;
    serde_json::from_slice(&data).ok()
}

fn content_range_matches(value: &str, start: u64, end: u64, total: u64) -> bool {
    let Some(value) = value.trim().strip_prefix("bytes ") else {
        return false;
    };
    let Some((range, reported_total)) = value.split_once('/') else {
        return false;
    };
    let Some((reported_start, reported_end)) = range.split_once('-') else {
        return false;
    };
    reported_start.parse::<u64>().ok() == Some(start)
        && reported_end.parse::<u64>().ok() == Some(end)
        && reported_total.parse::<u64>().ok() == Some(total)
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

fn chunk_hash_path(chunk: &Path) -> PathBuf {
    PathBuf::from(format!("{}.sha256", chunk.display()))
}

async fn sha256_file(path: &Path) -> Result<String> {
    let mut input = fs::File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let count = input.read(&mut buffer).await?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

async fn record_chunk_hash(chunk: &Path) -> Result<()> {
    let digest = sha256_file(chunk).await?;
    let target = chunk_hash_path(chunk);
    let temporary = target.with_extension("sha256.tmp");
    let mut output = fs::File::create(&temporary).await?;
    output.write_all(digest.as_bytes()).await?;
    output.sync_all().await?;
    fs::rename(temporary, target).await?;
    Ok(())
}

async fn verify_or_record_chunk(chunk: &Path) -> Result<bool> {
    let digest = sha256_file(chunk).await?;
    match fs::read_to_string(chunk_hash_path(chunk)).await {
        Ok(expected) => Ok(expected.trim().eq_ignore_ascii_case(&digest)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            record_chunk_hash(chunk).await?;
            Ok(true)
        }
        Err(error) => Err(error.into()),
    }
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
        assert!(content_range_matches(
            "bytes 4096-8191/16384",
            4096,
            8191,
            16384
        ));
        assert!(!content_range_matches(
            "bytes 0-4095/16384",
            4096,
            8191,
            16384
        ));
    }

    #[test]
    fn strong_etag_is_preferred_for_safe_resume() {
        let identity = ResumeIdentity {
            source_hash: "hash".into(),
            total: 10,
            etag: Some("\"version-2\"".into()),
            last_modified: Some("Wed, 01 Jan 2025 00:00:00 GMT".into()),
            chunk_size: 0,
        };
        assert_eq!(if_range_value(&identity), Some("\"version-2\""));
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
    fn adaptive_chunks_leave_multiple_jobs_per_worker() {
        assert_eq!(adaptive_chunk_size(8 * 1024 * 1024, 8), 4 * 1024 * 1024);
        assert_eq!(
            adaptive_chunk_size(64 * 1024 * 1024 * 1024, 8),
            64 * 1024 * 1024
        );
        let size = adaptive_chunk_size(1024 * 1024 * 1024, 8);
        assert!(1024 * 1024 * 1024_u64.div_ceil(size) >= 8 * 8);
    }

    #[test]
    fn mirror_identity_rejects_different_files() {
        let primary = SourceProbe {
            total: Some(10_000),
            etag: Some("\"file-a\"".into()),
            last_modified: None,
            digest: None,
            elapsed: Duration::from_millis(20),
        };
        let different_size = SourceProbe {
            total: Some(9_999),
            etag: Some("\"file-a\"".into()),
            last_modified: None,
            digest: None,
            elapsed: Duration::from_millis(10),
        };
        let different_etag = SourceProbe {
            total: Some(10_000),
            etag: Some("\"file-b\"".into()),
            last_modified: None,
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
            last_modified: None,
            digest: None,
            elapsed: Duration::from_millis(20),
        };
        let duplicate = SourceProbe {
            total: Some(10_000),
            etag: None,
            last_modified: None,
            digest: None,
            elapsed: Duration::from_millis(10),
        };
        assert!(same_download_identity(&primary, &duplicate, true));
        assert!(!same_download_identity(&primary, &duplicate, false));
    }

    #[test]
    fn referer_fallback_removes_only_referer_case_insensitively() {
        let mut headers = vec![
            ("User-Agent".into(), "browser".into()),
            ("ReFeReR".into(), "https://embed.example/".into()),
            ("Cookie".into(), "session=kept".into()),
        ];
        assert!(remove_header(&mut headers, "referer"));
        assert_eq!(headers.len(), 2);
        assert!(headers.iter().any(|(name, _)| name == "User-Agent"));
        assert!(headers.iter().any(|(name, _)| name == "Cookie"));
        assert!(!remove_header(&mut headers, "referer"));
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
