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
use serde::{de::DeserializeOwned, Deserialize, Serialize};
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
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufWriter},
    sync::{mpsc, Mutex},
};

use crate::validation::{validate_payload, PayloadExpectation};

const MIN_SEGMENT_CHUNK_SIZE: u64 = 4 * 1024 * 1024;
const MAX_SEGMENT_CHUNK_SIZE: u64 = 32 * 1024 * 1024;
const TARGET_CHUNKS_PER_WORKER: u64 = 8;
const WORKER_START_INTERVAL_MS: u64 = 35;
const JOURNAL_VERSION: u8 = 1;

#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub destination: PathBuf,
    pub overwrite: bool,
    pub connections: usize,
    pub method: String,
    pub body: Option<Vec<u8>>,
    pub headers: Vec<(String, String)>,
    /// Optional trusted SHA-256. When present, the .part file is never promoted
    /// to the final destination unless the digest matches exactly.
    pub expected_sha256: Option<String>,
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
    /// Structured, privacy-bounded engine telemetry. The desktop stores this
    /// only in an active diagnostics session so performance logging never
    /// becomes part of the hot-path legacy log by default.
    Diagnostic {
        event: &'static str,
        detail: serde_json::Value,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ResumeIdentity {
    total: u64,
    etag: Option<String>,
    last_modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SegmentJournal {
    version: u8,
    generation: u64,
    identity: ResumeIdentity,
    chunk_size: u64,
    completed: Vec<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SingleResumeJournal {
    version: u8,
    generation: u64,
    identity: ResumeIdentity,
}

#[derive(Debug)]
struct SourceProbe {
    total: Option<u64>,
    etag: Option<String>,
    last_modified: Option<String>,
    digest: Option<String>,
    elapsed: Duration,
}

fn same_download_identity(
    primary: &SourceProbe,
    candidate: &SourceProbe,
    _advertised: bool,
) -> bool {
    if primary.total.is_some() && candidate.total.is_some() && primary.total != candidate.total {
        return false;
    }
    if let (Some(left), Some(right)) = (primary.digest.as_deref(), candidate.digest.as_deref()) {
        return left.eq_ignore_ascii_case(right);
    }
    if let (Some(left), Some(right)) = (primary.etag.as_deref(), candidate.etag.as_deref()) {
        if !left.trim_start().starts_with("W/") && !right.trim_start().starts_with("W/") {
            return left == right;
        }
    }
    // Size and Last-Modified are useful hints for sequential failover, but
    // they are not cryptographic/content identities and must never authorize
    // striping bytes from different origins into the same output file.
    false
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
        let source = request.url.clone();
        self.download_from_sources(request, vec![source], events)
            .await
    }

    /// Download from one or more already-verified sources. When byte ranges are
    /// available, workers distribute chunks across mirrors concurrently.
    pub async fn download_from_sources(
        &self,
        mut request: DownloadRequest,
        mut sources: Vec<String>,
        events: mpsc::Sender<DownloadEvent>,
    ) -> Result<()> {
        if request.destination.exists() && !request.overwrite {
            bail!("destination already exists");
        }
        if let Some(parent) = request.destination.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut seen = HashSet::new();
        sources.retain(|source| source.starts_with("http://") || source.starts_with("https://"));
        sources.retain(|source| seen.insert(source.clone()));
        if sources.is_empty() {
            sources.push(request.url.clone());
        }
        if !sources.iter().any(|source| source == &request.url) {
            sources.insert(0, request.url.clone());
        }

        let can_segment = request.method.eq_ignore_ascii_case("GET") && request.body.is_none();
        let requested = request.connections.clamp(1, 32);
        if can_segment && requested > 1 {
            let probe_url = sources[0].clone();
            let mut probe_headers = headers_for_source(&request.url, &probe_url, &request.headers);
            let mut head = apply_headers(self.client.head(&probe_url), &probe_headers)
                .send()
                .await
                .ok();
            let mut probe = apply_headers(self.client.get(&probe_url), &probe_headers)
                .header(header::RANGE, "bytes=0-0")
                .send()
                .await;
            if probe
                .as_ref()
                .is_ok_and(|response| is_optional_referer_rejection(response.status()))
                && remove_header(&mut probe_headers, "referer")
            {
                head = apply_headers(self.client.head(&probe_url), &probe_headers)
                    .send()
                    .await
                    .ok();
                probe = apply_headers(self.client.get(&probe_url), &probe_headers)
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
                        let identity = resume_identity_from_headers(probe.headers(), total);
                        let useful_connections = adaptive_connection_count(total, requested);
                        if useful_connections > 1 {
                            let planned_chunk_size = adaptive_chunk_size(total, useful_connections);
                            let _ = events.try_send(DownloadEvent::Diagnostic {
                                event: "http.engine_plan",
                                detail: serde_json::json!({
                                    "totalBytes": total,
                                    "requestedConnections": requested,
                                    "activeConnections": useful_connections,
                                    "chunkBytes": planned_chunk_size,
                                    "chunkCount": total.div_ceil(planned_chunk_size),
                                    "sourceCount": sources.len(),
                                    "crossOriginSources": sources.iter().filter(|source| !same_origin(&request.url, source)).count(),
                                    "resumeValidator": if identity.etag.as_deref().is_some_and(|value| !value.trim_start().starts_with("W/")) {
                                        "etag"
                                    } else if identity.last_modified.is_some() {
                                        "last_modified"
                                    } else {
                                        "none"
                                    }
                                }),
                            });
                            let mut attempt_connections = useful_connections;
                            loop {
                                let segmented = self
                                    .download_segmented(
                                        request.clone(),
                                        events.clone(),
                                        total,
                                        attempt_connections,
                                        sources.clone(),
                                        identity.clone(),
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
                                        cleanup_chunk_artifacts(&request.destination).await?;
                                        if attempt_connections > 2 {
                                            attempt_connections = (attempt_connections / 2).max(2);
                                            continue;
                                        }
                                        break;
                                    }
                                    Err(error) => return Err(error),
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut last_error = None;
        for source in sources {
            let mut attempt = request.clone();
            attempt.headers = headers_for_source(&request.url, &source, &request.headers);
            attempt.url = source;
            match self.download_single(attempt, events.clone()).await {
                Ok(()) => return Ok(()),
                Err(error) => last_error = Some(error),
            }
        }
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("no_download_source")))
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
        let source_headers = headers_for_source(&request.url, url, &request.headers);
        let response = apply_headers(self.client.get(url), &source_headers)
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
        let existing = fs::metadata(&partial)
            .await
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        let resumable_request =
            request.method.eq_ignore_ascii_case("GET") && request.body.is_none() && existing > 0;
        let saved_resume = if resumable_request {
            load_single_resume_journal(&request.destination).await
        } else {
            None
        };
        let resume_from = saved_resume
            .as_ref()
            .filter(|journal| journal.version == JOURNAL_VERSION)
            .and_then(|journal| resume_validator(&journal.identity).map(|_| existing))
            .unwrap_or(0);

        let method = reqwest::Method::from_bytes(request.method.as_bytes())
            .context("invalid HTTP method")?;
        let send = |headers: &[(String, String)], resume: Option<(&ResumeIdentity, u64)>| {
            let mut builder =
                apply_headers(self.client.request(method.clone(), &request.url), headers);
            if let Some(body) = &request.body {
                builder = builder.body(body.clone());
            }
            if let Some((identity, offset)) = resume {
                builder = builder.header(header::RANGE, format!("bytes={offset}-"));
                if let Some(validator) = resume_validator(identity) {
                    builder = builder.header(header::IF_RANGE, validator);
                }
            }
            builder
        };

        let mut effective_headers = request.headers.clone();
        let resume_pair = saved_resume
            .as_ref()
            .filter(|_| resume_from > 0)
            .map(|journal| (&journal.identity, resume_from));
        let mut response = send(&effective_headers, resume_pair).send().await?;
        if is_optional_referer_rejection(response.status())
            && remove_header(&mut effective_headers, "referer")
        {
            response = send(&effective_headers, resume_pair).send().await?;
        }

        let mut resumed = false;
        if let Some(saved) = saved_resume.as_ref().filter(|_| resume_from > 0) {
            let candidate_total = response
                .headers()
                .get(header::CONTENT_RANGE)
                .and_then(|value| value.to_str().ok())
                .and_then(content_range_parts)
                .map(|(_, _, total)| total);
            let current_identity = candidate_total
                .map(|total| resume_identity_from_headers(response.headers(), total));
            resumed = response.status() == StatusCode::PARTIAL_CONTENT
                && response
                    .headers()
                    .get(header::CONTENT_RANGE)
                    .and_then(|value| value.to_str().ok())
                    .and_then(content_range_parts)
                    .is_some_and(|(start, _, _)| start == resume_from)
                && current_identity
                    .as_ref()
                    .is_some_and(|current| resume_identity_matches(&saved.identity, current));
            if !resumed {
                response = send(&effective_headers, None).send().await?;
            }
        }

        let _ = events.try_send(DownloadEvent::Diagnostic {
            event: "http.resume_decision",
            detail: serde_json::json!({
                "mode": "single",
                "candidateBytes": resume_from,
                "accepted": resumed,
                "validatorPresent": saved_resume
                    .as_ref()
                    .is_some_and(|journal| resume_validator(&journal.identity).is_some()),
                "reason": if resume_from == 0 {
                    "no_validated_checkpoint"
                } else if resumed {
                    "validator_and_content_range_match"
                } else {
                    "remote_identity_changed_or_range_rejected"
                }
            }),
        });
        let response = response.error_for_status()?;
        let start = if resumed { resume_from } else { 0 };
        let total = if resumed {
            response
                .headers()
                .get(header::CONTENT_RANGE)
                .and_then(|value| value.to_str().ok())
                .and_then(content_range_parts)
                .map(|(_, _, total)| total)
        } else {
            response.content_length()
        };
        let identity = total.map(|total| resume_identity_from_headers(response.headers(), total));
        let resume_supported = response
            .headers()
            .get(header::ACCEPT_RANGES)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.eq_ignore_ascii_case("bytes"))
            && identity.as_ref().and_then(resume_validator).is_some();

        if let Some(identity) = identity {
            let mut journal = SingleResumeJournal {
                version: JOURNAL_VERSION,
                generation: saved_resume
                    .as_ref()
                    .map_or(0, |journal| journal.generation),
                identity,
            };
            persist_single_resume_journal(&request.destination, &mut journal).await?;
        } else {
            clear_single_resume_journal(&request.destination).await;
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
        clear_single_resume_journal(&request.destination).await;
        finish_download(&request, &partial, received, total, &events).await
    }

    async fn download_segmented(
        &self,
        request: DownloadRequest,
        events: mpsc::Sender<DownloadEvent>,
        total: u64,
        connections: usize,
        sources: Vec<String>,
        identity: ResumeIdentity,
    ) -> Result<()> {
        let chunk_size = adaptive_chunk_size(total, connections);
        let chunk_count = total.div_ceil(chunk_size) as usize;
        let worker_count = connections.min(chunk_count).max(1);
        let next_chunk = Arc::new(AtomicUsize::new(0));
        let partial = partial_path(&request.destination);

        fs::create_dir_all(chunk_directory(&request.destination)).await?;
        let saved = load_segment_journal(&request.destination).await;
        let can_resume = saved.as_ref().is_some_and(|journal| {
            journal.version == JOURNAL_VERSION
                && journal.chunk_size == chunk_size
                && journal.completed.len() == chunk_count
                && resume_identity_matches(&journal.identity, &identity)
        }) && fs::metadata(&partial)
            .await
            .is_ok_and(|metadata| metadata.len() == total);

        let mut journal = if can_resume {
            saved.expect("checked above")
        } else {
            clear_segment_journal(&request.destination).await;
            let file = fs::File::create(&partial).await?;
            file.set_len(total).await?;
            SegmentJournal {
                version: JOURNAL_VERSION,
                generation: 0,
                identity: identity.clone(),
                chunk_size,
                completed: vec![false; chunk_count],
            }
        };
        if !can_resume {
            persist_segment_journal(&request.destination, &mut journal).await?;
        }

        let resumed = journal
            .completed
            .iter()
            .enumerate()
            .filter(|(_, complete)| **complete)
            .map(|(index, _)| chunk_bounds(index, chunk_size, total).2)
            .sum::<u64>();
        let _ = events.try_send(DownloadEvent::Diagnostic {
            event: "http.resume_decision",
            detail: serde_json::json!({
                "mode": "segmented",
                "accepted": can_resume,
                "resumedBytes": resumed,
                "completedChunks": journal.completed.iter().filter(|complete| **complete).count(),
                "chunkCount": chunk_count,
                "validatorPresent": resume_validator(&identity).is_some(),
                "reason": if can_resume {
                    "journal_and_remote_identity_match"
                } else {
                    "new_or_unvalidated_segment_session"
                }
            }),
        });
        let progress = Arc::new(AtomicU64::new(resumed));
        let journal = Arc::new(Mutex::new(journal));
        let sources = Arc::new(sources);

        let _ = events
            .send(DownloadEvent::Started {
                resumed_at: resumed,
                total: Some(total),
                connections: worker_count,
                resume_supported: resume_validator(&identity).is_some(),
            })
            .await;

        let mut jobs = FuturesUnordered::new();
        for worker_index in 0..worker_count {
            let client = self.client.clone();
            let primary_url = request.url.clone();
            let headers = request.headers.clone();
            let destination = request.destination.clone();
            let partial = partial.clone();
            let sender = events.clone();
            let shared = progress.clone();
            let cursor = next_chunk.clone();
            let limiters = request.limiters.clone();
            let journal = journal.clone();
            let sources = sources.clone();
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
                    if journal.lock().await.completed[index] {
                        continue;
                    }

                    let (start, end, expected) = chunk_bounds(index, chunk_size, total);
                    let mut last_error = None;
                    let mut completed = false;
                    for source_offset in 0..sources.len() {
                        let source_index = (index + worker_index + source_offset) % sources.len();
                        let source = &sources[source_index];
                        let source_headers = headers_for_source(&primary_url, source, &headers);
                        let attempt_started = Instant::now();
                        let response = match apply_headers(client.get(source), &source_headers)
                            .header(header::RANGE, format!("bytes={start}-{end}"))
                            .send()
                            .await
                        {
                            Ok(response) => response,
                            Err(error) => {
                                last_error = Some(anyhow::anyhow!(error));
                                continue;
                            }
                        };
                        if response.status() != StatusCode::PARTIAL_CONTENT {
                            last_error = Some(anyhow::anyhow!(
                                "server stopped supporting byte ranges: status {} for bytes={start}-{end}",
                                response.status()
                            ));
                            continue;
                        }
                        if !response
                            .headers()
                            .get(header::CONTENT_RANGE)
                            .and_then(|value| value.to_str().ok())
                            .and_then(content_range_parts)
                            .is_some_and(|(actual_start, actual_end, actual_total)| {
                                actual_start == start && actual_end == end && actual_total == total
                            })
                        {
                            last_error = Some(anyhow::anyhow!(
                                "invalid content-range for bytes={start}-{end}"
                            ));
                            continue;
                        }

                        let protocol = format!("{:?}", response.version());
                        let mut file = fs::OpenOptions::new()
                            .write(true)
                            .open(&partial)
                            .await?;
                        file.seek(std::io::SeekFrom::Start(start)).await?;
                        let mut stream = response.bytes_stream();
                        let mut downloaded = 0_u64;
                        let mut attempt_error = None;
                        while let Some(chunk) = stream.next().await {
                            match chunk {
                                Ok(chunk) => {
                                    apply_bandwidth_limits(&limiters, chunk.len()).await;
                                    if downloaded + chunk.len() as u64 > expected {
                                        attempt_error = Some(anyhow::anyhow!(
                                            "segment exceeded expected length"
                                        ));
                                        break;
                                    }
                                    if let Err(error) = file.write_all(&chunk).await {
                                        attempt_error = Some(error.into());
                                        break;
                                    }
                                    downloaded += chunk.len() as u64;
                                    let received =
                                        shared.fetch_add(chunk.len() as u64, Ordering::Relaxed)
                                            + chunk.len() as u64;
                                    let _ = sender.try_send(DownloadEvent::Progress {
                                        received,
                                        total: Some(total),
                                    });
                                }
                                Err(error) => {
                                    attempt_error =
                                        Some(anyhow::anyhow!("segmented network stream failed: {error}"));
                                    break;
                                }
                            }
                        }
                        if downloaded != expected && attempt_error.is_none() {
                            attempt_error = Some(anyhow::anyhow!(
                                "incomplete segment: received {downloaded} of {expected} bytes"
                            ));
                        }
                        if let Some(error) = attempt_error {
                            if downloaded > 0 {
                                shared.fetch_sub(downloaded, Ordering::Relaxed);
                            }
                            last_error = Some(error);
                            continue;
                        }
                        file.flush().await?;

                        {
                            let mut state = journal.lock().await;
                            state.completed[index] = true;
                            persist_segment_journal(&destination, &mut state).await?;
                        }
                        let elapsed_ms = attempt_started.elapsed().as_millis() as u64;
                        let _ = sender.try_send(DownloadEvent::Diagnostic {
                            event: "http.segment_completed",
                            detail: serde_json::json!({
                                "chunkIndex": index,
                                "chunkCount": chunk_count,
                                "bytes": expected,
                                "sourceIndex": source_index,
                                "sourceCount": sources.len(),
                                "attempts": source_offset + 1,
                                "elapsedMs": elapsed_ms,
                                "bytesPerSecond": if elapsed_ms > 0 {
                                    expected.saturating_mul(1000) / elapsed_ms
                                } else {
                                    expected
                                },
                                "transport": protocol
                            }),
                        });
                        completed = true;
                        break;
                    }

                    if !completed {
                        return Err(last_error.unwrap_or_else(|| {
                            anyhow::anyhow!("all verified mirrors failed for segment {index}")
                        }));
                    }
                }
                Result::<()>::Ok(())
            });
        }

        while let Some(result) = jobs.next().await {
            result?;
        }

        {
            let state = journal.lock().await;
            if state.completed.iter().any(|complete| !complete) {
                bail!("segmented journal incomplete after workers finished");
            }
        }
        let output = fs::OpenOptions::new().write(true).open(&partial).await?;
        output.sync_all().await?;
        clear_segment_journal(&request.destination).await;
        cleanup_chunk_artifacts(&request.destination).await?;
        finish_download(&request, &partial, total, Some(total), &events).await
    }
}

fn adaptive_connection_count(total: u64, requested: usize) -> usize {
    let size_cap = match total {
        0..=16_777_216 => 2,
        16_777_217..=67_108_864 => 4,
        67_108_865..=268_435_456 => 8,
        268_435_457..=1_073_741_824 => 16,
        _ => 32,
    };
    requested.min(size_cap).max(1)
}

fn adaptive_chunk_size(total: u64, connections: usize) -> u64 {
    let workers = connections.max(1) as u64;
    let target_chunks = workers.saturating_mul(TARGET_CHUNKS_PER_WORKER).max(1);
    let raw = total.div_ceil(target_chunks);
    let mib = 1024 * 1024;
    raw.div_ceil(mib)
        .saturating_mul(mib)
        .clamp(MIN_SEGMENT_CHUNK_SIZE, MAX_SEGMENT_CHUNK_SIZE)
}

fn chunk_bounds(index: usize, chunk_size: u64, total: u64) -> (u64, u64, u64) {
    let start = index as u64 * chunk_size;
    let end = (start + chunk_size).min(total) - 1;
    (start, end, end - start + 1)
}

fn content_range_parts(value: &str) -> Option<(u64, u64, u64)> {
    let value = value.trim().strip_prefix("bytes ")?;
    let (range, total) = value.split_once('/')?;
    let (start, end) = range.split_once('-')?;
    Some((start.parse().ok()?, end.parse().ok()?, total.parse().ok()?))
}

fn resume_identity_from_headers(
    headers: &reqwest::header::HeaderMap,
    total: u64,
) -> ResumeIdentity {
    ResumeIdentity {
        total,
        etag: headers
            .get(header::ETAG)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned),
        last_modified: headers
            .get(header::LAST_MODIFIED)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned),
    }
}

fn resume_validator(identity: &ResumeIdentity) -> Option<&str> {
    identity
        .etag
        .as_deref()
        .filter(|etag| !etag.trim_start().starts_with("W/"))
        .or(identity.last_modified.as_deref())
}

fn resume_identity_matches(saved: &ResumeIdentity, current: &ResumeIdentity) -> bool {
    if saved.total != current.total {
        return false;
    }
    match (
        saved
            .etag
            .as_deref()
            .filter(|etag| !etag.trim_start().starts_with("W/")),
        current
            .etag
            .as_deref()
            .filter(|etag| !etag.trim_start().starts_with("W/")),
    ) {
        (Some(left), Some(right)) => return left == right,
        (Some(_), None) | (None, Some(_)) => return false,
        _ => {}
    }
    match (
        saved.last_modified.as_deref(),
        current.last_modified.as_deref(),
    ) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}

fn journal_slot_paths(destination: &Path, prefix: &str) -> [PathBuf; 2] {
    let directory = chunk_directory(destination);
    [
        directory.join(format!("{prefix}-a.json")),
        directory.join(format!("{prefix}-b.json")),
    ]
}

async fn read_json<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let bytes = fs::read(path).await.ok()?;
    serde_json::from_slice(&bytes).ok()
}

async fn write_json_slot<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    let bytes = serde_json::to_vec(value)?;
    let mut file = fs::File::create(path).await?;
    file.write_all(&bytes).await?;
    file.sync_all().await?;
    Ok(())
}

async fn load_segment_journal(destination: &Path) -> Option<SegmentJournal> {
    let paths = journal_slot_paths(destination, "segments");
    let left = read_json::<SegmentJournal>(&paths[0]).await;
    let right = read_json::<SegmentJournal>(&paths[1]).await;
    [left, right]
        .into_iter()
        .flatten()
        .max_by_key(|journal| journal.generation)
}

async fn persist_segment_journal(destination: &Path, journal: &mut SegmentJournal) -> Result<()> {
    journal.generation = journal.generation.saturating_add(1);
    let paths = journal_slot_paths(destination, "segments");
    let slot = (journal.generation as usize) & 1;
    write_json_slot(&paths[slot], journal).await
}

async fn clear_segment_journal(destination: &Path) {
    for path in journal_slot_paths(destination, "segments") {
        let _ = fs::remove_file(path).await;
    }
}

async fn load_single_resume_journal(destination: &Path) -> Option<SingleResumeJournal> {
    let paths = journal_slot_paths(destination, "resume");
    let left = read_json::<SingleResumeJournal>(&paths[0]).await;
    let right = read_json::<SingleResumeJournal>(&paths[1]).await;
    [left, right]
        .into_iter()
        .flatten()
        .max_by_key(|journal| journal.generation)
}

async fn persist_single_resume_journal(
    destination: &Path,
    journal: &mut SingleResumeJournal,
) -> Result<()> {
    journal.generation = journal.generation.saturating_add(1);
    let paths = journal_slot_paths(destination, "resume");
    let slot = (journal.generation as usize) & 1;
    write_json_slot(&paths[slot], journal).await
}

async fn clear_single_resume_journal(destination: &Path) {
    for path in journal_slot_paths(destination, "resume") {
        let _ = fs::remove_file(path).await;
    }
}

fn same_origin(left: &str, right: &str) -> bool {
    let Ok(left) = reqwest::Url::parse(left) else {
        return false;
    };
    let Ok(right) = reqwest::Url::parse(right) else {
        return false;
    };
    left.scheme() == right.scheme()
        && left.host_str() == right.host_str()
        && left.port_or_known_default() == right.port_or_known_default()
}

fn headers_for_source(
    primary_url: &str,
    source_url: &str,
    headers: &[(String, String)],
) -> Vec<(String, String)> {
    if same_origin(primary_url, source_url) {
        return headers.to_vec();
    }
    headers
        .iter()
        .filter(|(name, _)| {
            !["authorization", "proxy-authorization", "cookie"]
                .iter()
                .any(|blocked| name.eq_ignore_ascii_case(blocked))
        })
        .cloned()
        .collect()
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

    if let Some(expected) = request
        .expected_sha256
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let expected = expected.to_ascii_lowercase();
        if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            bail!("invalid expected SHA-256");
        }
        let actual = sha256_file(partial).await?;
        let _ = events.try_send(DownloadEvent::Diagnostic {
            event: "http.integrity_check",
            detail: serde_json::json!({
                "algorithm": "sha256",
                "matched": actual == expected
            }),
        });
        if actual != expected {
            bail!("sha256 mismatch");
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

async fn sha256_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
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
    fn advertised_mirror_needs_digest_or_strong_etag_before_striping() {
        let primary = SourceProbe {
            total: Some(10_000),
            etag: None,
            last_modified: Some("Mon, 01 Jan 2024 00:00:00 GMT".into()),
            digest: None,
            elapsed: Duration::from_millis(20),
        };
        let same_size = SourceProbe {
            total: Some(10_000),
            etag: None,
            last_modified: Some("Mon, 01 Jan 2024 00:00:00 GMT".into()),
            digest: None,
            elapsed: Duration::from_millis(10),
        };
        assert!(!same_download_identity(&primary, &same_size, true));

        let primary = SourceProbe {
            etag: Some("\"file-v1\"".into()),
            ..primary
        };
        let candidate = SourceProbe {
            etag: Some("\"file-v1\"".into()),
            ..same_size
        };
        assert!(same_download_identity(&primary, &candidate, true));
    }

    #[test]
    fn mirror_identity_rejects_different_files() {
        let primary = SourceProbe {
            total: Some(10_000),
            etag: Some("\"file-a\"".into()),
            last_modified: Some("Mon, 01 Jan 2024 00:00:00 GMT".into()),
            digest: None,
            elapsed: Duration::from_millis(20),
        };
        let different_size = SourceProbe {
            total: Some(9_999),
            etag: Some("\"file-a\"".into()),
            last_modified: Some("Mon, 01 Jan 2024 00:00:00 GMT".into()),
            digest: None,
            elapsed: Duration::from_millis(10),
        };
        let different_etag = SourceProbe {
            total: Some(10_000),
            etag: Some("\"file-b\"".into()),
            last_modified: Some("Mon, 01 Jan 2024 00:00:00 GMT".into()),
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
    async fn expected_sha256_blocks_promotion_of_corrupted_partial() {
        let root = std::env::temp_dir().join(format!("adm-integrity-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).await.unwrap();
        let destination = root.join("payload.bin");
        let partial = partial_path(&destination);
        fs::write(&partial, b"correct payload").await.unwrap();
        let (tx, _rx) = mpsc::channel(4);
        let request = DownloadRequest {
            url: "https://example.test/payload.bin".into(),
            destination: destination.clone(),
            overwrite: false,
            connections: 1,
            method: "GET".into(),
            body: None,
            headers: Vec::new(),
            expected_sha256: Some("0".repeat(64)),
            limiters: Vec::new(),
        };
        assert!(finish_download(&request, &partial, 15, Some(15), &tx).await.is_err());
        assert!(!destination.exists());
        assert!(partial.exists());
        let _ = fs::remove_dir_all(root).await;
    }

    #[test]
    fn adaptive_scheduler_uses_smaller_chunks_to_reduce_tail_latency() {
        assert_eq!(adaptive_chunk_size(64 * 1024 * 1024, 8), 4 * 1024 * 1024);
        assert!(adaptive_chunk_size(8 * 1024 * 1024 * 1024, 8) <= MAX_SEGMENT_CHUNK_SIZE);
        assert!(adaptive_chunk_size(8 * 1024 * 1024 * 1024, 8) >= MIN_SEGMENT_CHUNK_SIZE);
    }

    #[test]
    fn resume_requires_a_stable_remote_validator() {
        let stable = ResumeIdentity {
            total: 1024,
            etag: Some("\"abc\"".into()),
            last_modified: None,
        };
        let changed = ResumeIdentity {
            total: 1024,
            etag: Some("\"def\"".into()),
            last_modified: None,
        };
        let unvalidated = ResumeIdentity {
            total: 1024,
            etag: None,
            last_modified: None,
        };
        assert!(resume_identity_matches(&stable, &stable));
        assert!(!resume_identity_matches(&stable, &changed));
        assert!(!resume_identity_matches(&unvalidated, &unvalidated));
    }

    #[test]
    fn mirror_headers_never_leak_credentials_cross_origin() {
        let headers = vec![
            ("Cookie".into(), "session=secret".into()),
            ("Authorization".into(), "Bearer secret".into()),
            ("User-Agent".into(), "browser".into()),
            ("Referer".into(), "https://example.test/page".into()),
        ];
        let filtered = headers_for_source(
            "https://example.test/file",
            "https://mirror.test/file",
            &headers,
        );
        assert!(!filtered
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("cookie")));
        assert!(!filtered
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("authorization")));
        assert!(filtered
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("user-agent")));
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
