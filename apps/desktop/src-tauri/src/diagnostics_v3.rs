use chrono::Local;
use serde_json::json;
use std::{
    collections::{HashMap, HashSet},
    fs,
    fs::OpenOptions,
    hash::{Hash, Hasher},
    io::Write,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex, OnceLock,
    },
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_LOG_BYTES: u64 = 8 * 1024 * 1024;
const ROTATIONS: usize = 4;
const SCHEMA_VERSION: u8 = 3;

static SEQUENCE: AtomicU64 = AtomicU64::new(1);
static SESSION: OnceLock<String> = OnceLock::new();
static STARTED: OnceLock<Mutex<HashMap<String, u128>>> = OnceLock::new();

#[derive(Clone, Debug, Default)]
pub struct HostSignal {
    #[allow(dead_code)]
    pub host: String,
    pub failures: usize,
    pub unique_traces: HashSet<String>,
    pub status_403: usize,
    pub status_404: usize,
    pub status_429: usize,
    pub unexpected_json: usize,
    pub unexpected_html: usize,
    pub browser_transport_failures: usize,
    pub browser_transport_successes: usize,
    pub direct_http_failures: usize,
    pub timeouts: usize,
}

impl HostSignal {
    fn new(host: String) -> Self {
        Self {
            host,
            ..Self::default()
        }
    }

    pub fn add_queue_failures(&mut self, count: usize) {
        self.failures = self.failures.max(count);
    }

    pub fn conservative_rule_is_safe(&self) -> bool {
        self.failures >= 2
            && self.status_403 == 0
            && self.status_404 == 0
            && self.status_429 == 0
            && self.unexpected_json == 0
            && self.unexpected_html == 0
            && self.browser_transport_failures == 0
            && self.timeouts == 0
    }

    pub fn confidence(&self) -> u8 {
        let repeat = self.failures.saturating_sub(1).min(4) as u8 * 8;
        (62 + repeat).min(94)
    }

    pub fn reason(&self) -> String {
        format!(
            "{} correlated failure(s) across {} trace(s); no authentication, one-shot-token, rate-limit or content-mismatch signal detected; retry with one conservative connection",
            self.failures,
            self.unique_traces.len().max(1)
        )
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |value| value.as_millis())
}

fn session_id() -> &'static str {
    SESSION.get_or_init(|| {
        format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |value| value.as_secs())
        )
    })
}

fn rotated_path(path: &Path, index: usize) -> PathBuf {
    path.with_extension(format!("log.{index}"))
}

fn rotate(path: &Path) {
    if !fs::metadata(path).is_ok_and(|metadata| metadata.len() > MAX_LOG_BYTES) {
        return;
    }
    let _ = fs::remove_file(rotated_path(path, ROTATIONS));
    for index in (1..ROTATIONS).rev() {
        let from = rotated_path(path, index);
        if from.exists() {
            let _ = fs::rename(from, rotated_path(path, index + 1));
        }
    }
    let _ = fs::rename(path, rotated_path(path, 1));
}

fn fingerprint(value: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:08x}", hasher.finish() as u32)
}

fn looks_like_uuid(segment: &str) -> bool {
    let bytes = segment.as_bytes();
    bytes.len() == 36
        && [8, 13, 18, 23].iter().all(|index| bytes[*index] == b'-')
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| [8, 13, 18, 23].contains(&index) || byte.is_ascii_hexdigit())
}

fn sanitize_url(raw: &str) -> String {
    let trimmed = raw.trim_matches(|character: char| {
        matches!(character, ')' | ']' | '}' | ',' | ';' | '\'' | '"')
    });
    let Ok(mut url) = url::Url::parse(trimmed) else {
        return "<invalid-url>".to_owned();
    };
    let _ = url.set_username("");
    let _ = url.set_password(None);
    url.set_fragment(None);
    if url.query().is_some() {
        let names = url
            .query_pairs()
            .map(|(name, _)| name.into_owned())
            .collect::<Vec<_>>();
        url.set_query(None);
        if !names.is_empty() {
            let query = names
                .into_iter()
                .map(|name| format!("{name}=<redacted>"))
                .collect::<Vec<_>>()
                .join("&");
            url.set_query(Some(&query));
        }
    }
    let segments = url
        .path_segments()
        .map(|items| {
            items
                .map(|segment| {
                    if looks_like_uuid(segment) {
                        format!("<id:{}>", fingerprint(segment))
                    } else if segment.len() > 96 {
                        format!("<segment:{}>", fingerprint(segment))
                    } else {
                        segment.to_owned()
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if !segments.is_empty() {
        url.set_path(&format!("/{}", segments.join("/")));
    }
    url.to_string()
}

fn sanitize_detail(detail: &str) -> String {
    let mut result = Vec::new();
    for field in detail.split_whitespace().take(96) {
        let lower = field.to_ascii_lowercase();
        if lower.starts_with("cookie=")
            || lower.starts_with("authorization=")
            || lower.starts_with("password=")
            || lower.starts_with("passwd=")
            || lower.starts_with("pairingtoken=")
            || lower.starts_with("pairing_token=")
        {
            let name = field.split_once('=').map_or("secret", |(name, _)| name);
            result.push(format!("{name}=<redacted>"));
            continue;
        }
        if let Some(index) = field.find("https://").or_else(|| field.find("http://")) {
            let (prefix, raw_url) = field.split_at(index);
            result.push(format!("{prefix}{}", sanitize_url(raw_url)));
            continue;
        }
        result.push(field.chars().take(512).collect());
    }
    result.join(" ")
}

fn field<'a>(detail: &'a str, name: &str) -> Option<&'a str> {
    detail
        .split_whitespace()
        .find_map(|item| item.strip_prefix(&format!("{name}=")))
}

fn find_url(detail: &str) -> Option<&str> {
    detail.split_whitespace().find_map(|item| {
        item.find("https://")
            .or_else(|| item.find("http://"))
            .map(|index| &item[index..])
    })
}

fn trace_for(detail: &str) -> Option<String> {
    field(detail, "trace")
        .or_else(|| field(detail, "task"))
        .or_else(|| field(detail, "transport"))
        .map(|value| {
            value
                .trim_matches(|character: char| {
                    !character.is_ascii_alphanumeric() && character != '-'
                })
                .to_owned()
        })
        .filter(|value| !value.is_empty())
}

fn host_for(detail: &str) -> Option<String> {
    field(detail, "host").map(str::to_owned).or_else(|| {
        find_url(detail).and_then(|raw| {
            url::Url::parse(
                raw.trim_matches(|character: char| {
                    matches!(character, ')' | ']' | '}' | ',' | ';')
                }),
            )
            .ok()
            .and_then(|url| url.host_str().map(|host| host.to_ascii_lowercase()))
        })
    })
}

fn status_for(detail: &str) -> Option<u16> {
    field(detail, "status")
        .and_then(|value| {
            value
                .trim_matches(|character: char| !character.is_ascii_digit())
                .parse()
                .ok()
        })
        .or_else(|| {
            [403_u16, 404, 408, 429, 500, 502, 503, 504]
                .into_iter()
                .find(|status| detail.contains(&status.to_string()))
        })
}

fn elapsed_for(event: &str, trace: Option<&str>, timestamp: u128) -> Option<u128> {
    let trace = trace?;
    let started = STARTED.get_or_init(|| Mutex::new(HashMap::new()));
    let Ok(mut started) = started.lock() else {
        return None;
    };
    let starts = event.ends_with(".start")
        || event.ends_with(".started")
        || event == "task.enqueued"
        || event == "rapidgator.transport.requested";
    let terminal = event.ends_with(".failed")
        || event.ends_with(".completed")
        || event.ends_with(".done")
        || event.ends_with(".ended");
    if starts {
        started.entry(trace.to_owned()).or_insert(timestamp);
        return Some(0);
    }
    if terminal {
        return started
            .remove(trace)
            .map(|start| timestamp.saturating_sub(start));
    }
    started
        .get(trace)
        .map(|start| timestamp.saturating_sub(*start))
}

pub fn write_event(path: &Path, lock: &Mutex<()>, level: &str, event: &str, detail: &str) {
    let _guard = match lock.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    rotate(path);
    let timestamp = now_ms();
    let local = Local::now();
    let sanitized = sanitize_detail(detail);
    let trace = trace_for(&sanitized);
    let host = host_for(&sanitized);
    let status = status_for(&sanitized);
    let extension_version = field(&sanitized, "extension_version").map(str::to_owned);
    let elapsed = elapsed_for(event, trace.as_deref(), timestamp);
    let record = json!({
        "schema": SCHEMA_VERSION,
        "timestamp": local.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        "date": local.format("%Y-%m-%d").to_string(),
        "time": local.format("%H:%M:%S%.3f").to_string(),
        "tsMs": timestamp.to_string(),
        "appVersion": env!("CARGO_PKG_VERSION"),
        "extensionVersion": extension_version,
        "seq": SEQUENCE.fetch_add(1, Ordering::Relaxed),
        "session": session_id(),
        "level": level.chars().take(12).collect::<String>(),
        "event": event.chars().take(96).collect::<String>(),
        "trace": trace,
        "host": host,
        "status": status,
        "elapsedMs": elapsed.map(|value| value.min(u64::MAX as u128) as u64),
        "detail": sanitized,
    });
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{record}");
    }
}

fn register_signal(
    signals: &mut HashMap<String, HostSignal>,
    trace_hosts: &mut HashMap<String, String>,
    event: &str,
    detail: &str,
    trace: Option<String>,
    explicit_host: Option<String>,
    status: Option<u16>,
) {
    let host = explicit_host.or_else(|| host_for(detail)).or_else(|| {
        trace
            .as_ref()
            .and_then(|trace| trace_hosts.get(trace).cloned())
    });
    let Some(host) = host else {
        return;
    };
    if let Some(trace) = trace.as_ref() {
        trace_hosts.insert(trace.clone(), host.clone());
    }
    let signal = signals
        .entry(host.clone())
        .or_insert_with(|| HostSignal::new(host));
    if let Some(trace) = trace {
        signal.unique_traces.insert(trace);
    }
    let lower_event = event.to_ascii_lowercase();
    let lower_detail = detail.to_ascii_lowercase();
    let failed = lower_event.ends_with(".failed")
        || lower_event.contains("failure")
        || lower_detail.contains("error=")
        || status.is_some_and(|status| status >= 400);
    if failed {
        signal.failures += 1;
    }
    match status {
        Some(403) => signal.status_403 += 1,
        Some(404) => signal.status_404 += 1,
        Some(429) => signal.status_429 += 1,
        _ => {}
    }
    if lower_detail.contains("application/json") || lower_event.contains("unexpected_json") {
        signal.unexpected_json += 1;
    }
    if lower_detail.contains("text/html") || lower_event.contains("unexpected_html") {
        signal.unexpected_html += 1;
    }
    if lower_event.contains("browser_transport") || lower_event.contains("rapidgator.transport") {
        if failed {
            signal.browser_transport_failures += 1;
        } else if lower_event.ends_with(".completed") {
            signal.browser_transport_successes += 1;
        }
    }
    if lower_event == "http.failed" {
        signal.direct_http_failures += 1;
    }
    if lower_event.contains("timeout")
        || lower_detail.contains("timeout")
        || lower_detail.contains("timed out")
    {
        signal.timeouts += 1;
    }
}

fn parse_json_line(
    line: &str,
    signals: &mut HashMap<String, HostSignal>,
    trace_hosts: &mut HashMap<String, String>,
) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
        return false;
    };
    let event = value
        .get("event")
        .and_then(|item| item.as_str())
        .unwrap_or_default();
    let detail = value
        .get("detail")
        .and_then(|item| item.as_str())
        .unwrap_or_default();
    let trace = value
        .get("trace")
        .and_then(|item| item.as_str())
        .map(str::to_owned);
    let host = value
        .get("host")
        .and_then(|item| item.as_str())
        .map(str::to_owned);
    let status = value
        .get("status")
        .and_then(|item| item.as_u64())
        .map(|value| value as u16);
    register_signal(signals, trace_hosts, event, detail, trace, host, status);
    true
}

fn parse_legacy_line(
    line: &str,
    signals: &mut HashMap<String, HostSignal>,
    trace_hosts: &mut HashMap<String, String>,
) {
    let mut fields = line.split_whitespace();
    let _timestamp = fields.next();
    let _level = fields.next();
    let event = fields.next().unwrap_or_default();
    let detail = fields.collect::<Vec<_>>().join(" ");
    let trace = trace_for(&detail);
    let host = host_for(&detail);
    let status = status_for(&detail);
    register_signal(signals, trace_hosts, event, &detail, trace, host, status);
}

pub fn analyze(path: &Path) -> HashMap<String, HostSignal> {
    let mut signals = HashMap::new();
    let mut trace_hosts = HashMap::new();
    let mut paths = (1..=ROTATIONS)
        .rev()
        .map(|index| rotated_path(path, index))
        .collect::<Vec<_>>();
    paths.push(path.to_path_buf());
    for log_path in paths {
        let Ok(contents) = fs::read_to_string(log_path) else {
            continue;
        };
        for line in contents.lines().filter(|line| !line.trim().is_empty()) {
            if !parse_json_line(line, &mut signals, &mut trace_hosts) {
                parse_legacy_line(line, &mut signals, &mut trace_hosts);
            }
        }
    }
    signals
}
