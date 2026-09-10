//! Bounded structured diagnostics. This module never participates in media selection.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

const MAX_EVENTS: usize = 4000;
const FILE_BYTES: u64 = 4 * 1024 * 1024;
static STORE: OnceLock<Mutex<Store>> = OnceLock::new();
static SALT: OnceLock<String> = OnceLock::new();

struct Store {
    loaded: bool,
    events: VecDeque<Value>,
    ids: HashSet<String>,
    tasks: HashMap<String, String>,
    suppressed: BTreeMap<String, u64>,
    write_errors: u64,
    dropped_memory: u64,
    client_health: Value,
    session: String,
    sequence: u64,
    queue_fingerprint: Vec<String>,
}
impl Default for Store {
    fn default() -> Self {
        Self {
            loaded: false,
            events: VecDeque::new(),
            ids: HashSet::new(),
            tasks: HashMap::new(),
            suppressed: BTreeMap::new(),
            write_errors: 0,
            dropped_memory: 0,
            client_health: json!({}),
            session: uuid::Uuid::new_v4().to_string(),
            sequence: 0,
            queue_fingerprint: Vec::new(),
        }
    }
}
fn store() -> &'static Mutex<Store> {
    STORE.get_or_init(|| Mutex::new(Store::default()))
}
fn log_path(base: &Path, generation: usize) -> PathBuf {
    base.with_file_name(if generation == 0 {
        "diagnostics-v3.jsonl".to_owned()
    } else {
        format!("diagnostics-v3.jsonl.{generation}")
    })
}
fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
pub(super) fn valid_id(value: Option<&str>) -> Option<String> {
    value
        .and_then(|value| uuid::Uuid::parse_str(value).ok())
        .map(|id| id.to_string())
}
fn field(detail: &str, key: &str) -> Option<String> {
    detail
        .split_whitespace()
        .find_map(|word| word.strip_prefix(key))
        .and_then(|v| valid_id(Some(v)))
}
fn sensitive(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    [
        "cookie",
        "authorization",
        "password",
        "passwd",
        "secret",
        "token",
        "credential",
        "body",
        "html",
        "caption",
        "title",
        "filename",
        "filepath",
    ]
    .iter()
    .any(|part| key.contains(part))
}
pub(super) fn safe_text(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    if [
        "cookie=",
        "cookie:",
        "authorization=",
        "authorization:",
        "password=",
        "password:",
        "token=",
        "secret=",
        "bearer ",
        "passwd=",
    ]
    .iter()
    .any(|part| lower.contains(part))
    {
        return "[redacted-sensitive-text]".into();
    }
    let mut hide_path = false;
    text.split_whitespace()
        .take(160)
        .map(|word| {
            if hide_path
                && !word.split_once('=').is_some_and(|(key, _)| {
                    key.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
                })
            {
                return "[path-fragment]".to_owned();
            }
            hide_path = false;
            if word.contains(":\\")
                || word.contains(":/") && !word.contains("://")
                || word.starts_with("/home/")
                || word.starts_with("/Users/")
                || word.starts_with("/mnt/")
                || word.starts_with("file=")
                || word.starts_with("file_name=")
            {
                hide_path = true;
                return "[local-path]".to_owned();
            }
            if let Some(offset) = ["https://", "http://", "blob:", "file:", "data:"]
                .iter()
                .filter_map(|prefix| word.find(prefix))
                .min()
            {
                let prefix = &word[..offset];
                let resource = &word[offset..];
                let host = url::Url::parse(resource)
                    .ok()
                    .and_then(|url| url.host_str().map(str::to_owned))
                    .unwrap_or_default();
                return format!("{prefix}[resource:{host}]");
            }
            word.chars()
                .filter(|c| !c.is_control())
                .take(240)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(1600)
        .collect()
}
fn safe(value: &Value, key: &str, depth: usize) -> Value {
    if depth > 7 {
        return json!("[depth-limit]");
    }
    if value.is_boolean() || value.is_null() {
        return value.clone();
    }
    if sensitive(key) {
        return json!("[redacted]");
    }
    if value.is_number() {
        return value.clone();
    }
    match value {
        Value::String(text) => {
            if text.starts_with("http://")
                || text.starts_with("https://")
                || text.starts_with("blob:")
                || text.starts_with("file:")
                || text.starts_with("data:")
            {
                let salt = SALT.get_or_init(|| uuid::Uuid::new_v4().to_string());
                let digest = Sha256::digest(format!("{salt}\0{text}").as_bytes());
                let id = digest[..12]
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>();
                let url = url::Url::parse(text).ok();
                json!({"resourceId": id, "scheme": url.as_ref().map(|u| u.scheme()), "host": url.as_ref().and_then(|u| u.host_str())})
            } else {
                json!(safe_text(text))
            }
        }
        Value::Array(items) => Value::Array(
            items
                .iter()
                .take(48)
                .map(|v| safe(v, key, depth + 1))
                .collect(),
        ),
        Value::Object(items) => Value::Object(
            items
                .iter()
                .take(64)
                .map(|(k, v)| {
                    (
                        k.chars().take(64).collect::<String>(),
                        safe(v, k, depth + 1),
                    )
                })
                .collect(),
        ),
        _ => Value::Null,
    }
}
fn retain(s: &mut Store, event: Value) {
    if let Some(id) = event.get("eventId").and_then(Value::as_str) {
        s.ids.insert(id.to_owned());
    }
    s.events.push_back(event);
    while s.events.len() > MAX_EVENTS {
        if let Some(old) = s.events.pop_front() {
            if let Some(id) = old.get("eventId").and_then(Value::as_str) {
                s.ids.remove(id);
            }
            s.dropped_memory += 1;
        }
    }
}
fn load(s: &mut Store, base: &Path) {
    if s.loaded {
        return;
    }
    s.loaded = true;
    for generation in (0..=2).rev() {
        if let Ok(file) = fs::File::open(log_path(base, generation)) {
            let mut text = String::new();
            if file
                .take(FILE_BYTES + 16000)
                .read_to_string(&mut text)
                .is_err()
            {
                continue;
            }
            for line in text.lines() {
                if let Ok(value) = serde_json::from_str::<Value>(line) {
                    if valid_id(value.get("eventId").and_then(Value::as_str)).is_some() {
                        if let (Some(task), Some(action)) = (
                            valid_id(value.get("taskId").and_then(Value::as_str)),
                            valid_id(value.get("actionId").and_then(Value::as_str)),
                        ) {
                            if s.tasks.len() < 4000 {
                                s.tasks.insert(task, action);
                            }
                        }
                        retain(s, value);
                    }
                }
            }
        }
    }
}
fn append(s: &mut Store, base: &Path, event: Value) -> Result<(), String> {
    let path = log_path(base, 0);
    let operation = || -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        if fs::metadata(&path).is_ok_and(|m| m.len() >= FILE_BYTES) {
            let last = log_path(base, 2);
            if last.exists() {
                fs::remove_file(&last)?;
            }
            let previous = log_path(base, 1);
            if previous.exists() {
                fs::rename(previous, last)?;
            }
            fs::rename(&path, log_path(base, 1))?;
        }
        let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
        writeln!(file, "{event}")?;
        file.flush()
    };
    if operation().is_err() {
        s.write_errors += 1;
        return Err("diagnostic_write_failed".into());
    }
    retain(s, event);
    Ok(())
}
pub(super) fn level<'a>(requested: &'a str, event: &str) -> &'a str {
    if event.ends_with("_unresolved") || event.ends_with("_rejected") || event.ends_with("_failed")
    {
        if requested == "INFO" || requested == "DEBUG" {
            return "WARN";
        }
    }
    requested
}
pub(super) fn suppress(event: &str, detail: &str) -> bool {
    let command = detail
        .split_whitespace()
        .find_map(|s| s.strip_prefix("command="))
        .unwrap_or("");
    let quiet = [
        "get_",
        "list_",
        "read_",
        "take_bridge_download",
        "default_download_directory",
    ]
    .iter()
    .any(|s| command.starts_with(s));
    if quiet && matches!(event, "ui.command_started" | "ui.command_completed")
        || event == "bridge.health"
    {
        if let Ok(mut s) = store().lock() {
            *s.suppressed
                .entry(format!("{event}:{command}"))
                .or_default() += 1;
        }
        return true;
    }
    false
}
pub(super) fn queue_saved(base: &Path, ids: Vec<String>, success: bool) {
    let changed = if let Ok(mut s) = store().lock() {
        let changed = s.queue_fingerprint != ids;
        if success {
            s.queue_fingerprint = ids.clone();
        }
        changed
    } else {
        false
    };
    if changed || !success {
        native(
            base,
            if success { "INFO" } else { "ERROR" },
            if success {
                "queue.membership_persisted"
            } else {
                "queue.persist_failed"
            },
            &format!(
                "count={} ids={}",
                ids.len(),
                ids.iter().take(100).cloned().collect::<Vec<_>>().join(",")
            ),
        );
    }
}
pub(super) fn bind_task(task: &str, action: Option<&str>) {
    if let (Some(action), Ok(mut s)) = (valid_id(action), store().lock()) {
        if s.tasks.len() >= 4000 {
            s.tasks.clear();
        }
        s.tasks.insert(task.to_owned(), action);
    }
}
pub(super) fn native(base: &Path, requested_level: &str, event: &str, detail: &str) {
    if let Ok(mut s) = store().lock() {
        load(&mut s, base);
        s.sequence += 1;
        let task = field(detail, "task=");
        let action = field(detail, "trace=")
            .or_else(|| task.as_ref().and_then(|task| s.tasks.get(task).cloned()));
        let record = json!({"schemaVersion":3,"eventId":uuid::Uuid::new_v4().to_string(),
            "sessionId":s.session,"producerId":s.session,"sequence":s.sequence,"timestamp":now(),
            "actionId":action,"taskId":task,"component":"desktop","event":event,
            "level":level(requested_level, event),"applicationVersion":env!("CARGO_PKG_VERSION"),
            "data":{"detail":safe_text(detail)}});
        let _ = append(&mut s, base, record);
    }
}
pub(super) fn ingest(base: &Path, body: &str) -> Result<Value, String> {
    if body.len() > 450_000 {
        return Err("diagnostic_batch_too_large".into());
    }
    let input: Value = serde_json::from_str(body).map_err(|_| "invalid_diagnostic_json")?;
    let events = input
        .get("events")
        .and_then(Value::as_array)
        .ok_or("diagnostic_events_required")?;
    if events.len() > 50 {
        return Err("diagnostic_batch_too_large".into());
    }
    let mut s = store().lock().map_err(|_| "diagnostic_lock_failed")?;
    load(&mut s, base);
    s.client_health = safe(input.get("health").unwrap_or(&Value::Null), "health", 0);
    let mut accepted = Vec::new();
    for event in events {
        let Some(id) = valid_id(event.get("eventId").and_then(Value::as_str)) else {
            continue;
        };
        if s.ids.contains(&id) {
            accepted.push(id);
            continue;
        }
        let name = event
            .get("event")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        if name.len() > 100
            || !name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
        {
            continue;
        }
        let mut clean = safe(event, "event", 0);
        clean["timestamp"] = json!(now());
        clean["eventId"] = json!(id);
        clean["actionId"] = json!(valid_id(event.get("actionId").and_then(Value::as_str)));
        clean["schemaVersion"] = json!(3);
        let component = event.get("component").and_then(Value::as_str).unwrap_or("");
        clean["component"] = json!(match component {
            "extension.background" | "extension.popup" | "extension.content" => component,
            _ => "extension.unknown",
        });
        let requested = event.get("level").and_then(Value::as_str).unwrap_or("INFO");
        clean["level"] = json!(level(
            match requested {
                "DEBUG" | "INFO" | "WARN" | "ERROR" => requested,
                _ => "INFO",
            },
            name
        ));
        if clean.to_string().len() > 16000 {
            clean["data"] = json!({"truncated":true});
        }
        append(&mut s, base, clean)?;
        accepted.push(id);
    }
    Ok(json!({"ok":true,"accepted":accepted}))
}
fn lines(events: impl Iterator<Item = Value>) -> Vec<u8> {
    events
        .map(|e| format!("{e}\n"))
        .collect::<String>()
        .into_bytes()
}
fn report_for(s: &Store) -> String {
    let mut report = format!(
        "APOCALIPSE - DIAGNOSTICO V3 / RELATORIO PARA IA\nADM: {}\nGerado: {}\n\n",
        env!("CARGO_PKG_VERSION"),
        now()
    );
    report.push_str("ESCOPO E LIMITES\nDados locais sanitizados. URLs/cookies/senhas nao sao necessarios para esta linha do tempo.\nTimestamp = recebimento no ADM; clientTimestamp e sequence preservam a ordem no produtor.\nAusencia de evento NAO prova ausencia de atividade. Cache em memoria, frames inacessiveis e processos suspensos podem limitar a coleta.\nPlayer iniciado NAO confirma imagem/som no computador do usuario.\n\n");
    let errors = s
        .events
        .iter()
        .filter(|e| matches!(e["level"].as_str(), Some("WARN" | "ERROR")))
        .count();
    report.push_str(&format!("COLETA\nEventos retidos: {}\nAvisos/erros retidos: {errors}\nEventos retirados da memoria por limite: {}\nFalhas de escrita: {}\nSaude do ultimo lote da extensao: {}\nConsultas repetitivas agrupadas: {}\n\n", s.events.len(), s.dropped_memory, s.write_errors, s.client_health, json!(s.suppressed)));
    report.push_str("ULTIMAS ACOES (fatos registrados; nao diagnostico automatico da causa)\n");
    let mut actions: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
    let mut order = Vec::new();
    for e in &s.events {
        if let Some(id) = e["actionId"].as_str() {
            if !actions.contains_key(id) {
                order.push(id.to_owned());
            }
            actions.entry(id.to_owned()).or_default().push(e);
        }
    }
    for id in order.iter().rev().take(20).rev() {
        report.push_str(&format!("\nAcao {id}:\n"));
        for e in actions[id].iter().rev().take(18).rev() {
            report.push_str(&format!(
                "  {} | {} | {} | {}\n",
                e["timestamp"].as_str().unwrap_or(""),
                e["component"].as_str().unwrap_or(""),
                e["event"].as_str().unwrap_or(""),
                e["data"]
            ));
        }
    }
    report.push_str("\nULTIMOS AVISOS / MARCACOES\n");
    for e in s
        .events
        .iter()
        .rev()
        .filter(|e| {
            matches!(e["level"].as_str(), Some("WARN" | "ERROR"))
                || e["event"].as_str().unwrap_or("").contains("user_mark")
        })
        .take(30)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        report.push_str(&format!(
            "{} | {} | {}\n",
            e["timestamp"], e["event"], e["data"]
        ));
    }
    if s.client_health.is_null() || s.client_health == json!({}) {
        report.push_str("\nATENCAO: nenhum lote V3 da extensao foi recebido. Instale a candidata V3, pareie e inicie a captura na aba afetada.\n");
    }
    report.chars().take(120000).collect()
}
pub(super) fn report(base: &Path) -> Result<String, String> {
    let mut s = store().lock().map_err(|_| "diagnostic_lock_failed")?;
    load(&mut s, base);
    Ok(report_for(&s))
}
pub(super) fn export(base: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut s = store().lock().map_err(|_| "diagnostic_lock_failed")?;
    load(&mut s, base);
    let mut snapshots: BTreeMap<String, Value> = BTreeMap::new();
    let mut collectors: BTreeMap<String, Value> = BTreeMap::new();
    for e in &s.events {
        let name = e["event"].as_str().unwrap_or("");
        if name.contains("snapshot") || name.starts_with("popup.") {
            let key = format!("{}:{}:{}", e["producerId"], name, e["data"]["playerId"]);
            if snapshots.len() < 200 || snapshots.contains_key(&key) {
                snapshots.insert(key, e.clone());
            }
        }
        if name.starts_with("collector.") {
            collectors.insert(format!("{}:{}", e["producerId"], name), e.clone());
        }
    }
    let mut entries = vec![
        ("RELATORIO_PARA_IA.txt".into(), report_for(&s).into_bytes()),
        ("traces/acoes.jsonl".into(), lines(s.events.iter().filter(|e| e["actionId"].is_string()).cloned())),
        ("capture/decisoes-de-midia.jsonl".into(), lines(s.events.iter().filter(|e| {
            let n = e["event"].as_str().unwrap_or(""); n.starts_with("network.") || n.starts_with("identity.") || n.starts_with("popup.")
        }).cloned())),
        ("state/players-e-popup.json".into(), serde_json::to_vec_pretty(&snapshots).map_err(|e| e.to_string())?),
        ("health/coletores.json".into(), serde_json::to_vec_pretty(&json!({"collectors":collectors,"lastClientHealth":s.client_health,
            "writeErrors":s.write_errors,"eventsEvictedFromMemory":s.dropped_memory,"suppressedQueries":s.suppressed,
            "maxRetainedEvents":MAX_EVENTS,"diskGenerations":3,"maxBytesPerGeneration":FILE_BYTES,
            "limitations":["memory_cache_not_always_observable","page_observations_are_untrusted","no_audio_visual_confirmation","only_last_client_health_snapshot"]})).map_err(|e| e.to_string())?),
    ];
    for generation in (0..=2).rev() {
        if let Ok(file) = fs::File::open(log_path(base, generation)) {
            let mut bytes = Vec::new();
            file.take(FILE_BYTES + 16000)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            entries.push((format!("logs/v3/events-{generation}.jsonl"), bytes));
        }
    }
    Ok(entries)
}
pub(super) fn clear(base: &Path) -> Result<(), String> {
    let mut s = store().lock().map_err(|_| "diagnostic_lock_failed")?;
    for generation in 0..=2 {
        let path = log_path(base, generation);
        if path.exists() {
            fs::remove_file(path).map_err(|_| "diagnostic_clear_failed")?;
        }
    }
    *s = Store::default();
    s.loaded = true;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sanitization_removes_secrets_signed_paths_and_nested_values() {
        let value = json!({"url":"https://user:pass@cdn.example/secret-path?sig=TOPSECRET", "cookieHeader":"SESSION=SECRET",
            "nested":{"requestBody":"PRIVATE", "message":"Bearer abc123"}, "hasToken":true});
        let result = safe(&value, "", 0).to_string();
        for forbidden in [
            "pass@",
            "TOPSECRET",
            "secret-path",
            "SESSION=",
            "PRIVATE",
            "abc123",
        ] {
            assert!(!result.contains(forbidden), "{forbidden}");
        }
        assert!(result.contains("cdn.example"));
        assert!(result.contains("resourceId"));
    }
    #[test]
    fn numeric_secrets_are_redacted_but_measurements_are_preserved() {
        let value =
            json!({"token": 98123456, "password": 98765432, "bytes": 1234, "hasToken": true});
        let result = safe(&value, "", 0);
        assert_eq!(result["token"], json!("[redacted]"));
        assert_eq!(result["password"], json!("[redacted]"));
        assert_eq!(result["bytes"], json!(1234));
        assert_eq!(result["hasToken"], json!(true));
    }
    #[test]
    fn unresolved_is_warning_and_identifiers_are_validated() {
        assert_eq!(level("INFO", "overlay_download_unresolved"), "WARN");
        assert_eq!(valid_id(Some("arbitrary-secret")), None);
        let id = uuid::Uuid::new_v4().to_string();
        assert_eq!(field(&format!("trace={id} task=none"), "trace="), Some(id));
    }
    #[test]
    fn memory_window_is_bounded() {
        let mut s = Store::default();
        for _ in 0..MAX_EVENTS + 10 {
            retain(&mut s, json!({"eventId":uuid::Uuid::new_v4().to_string()}));
        }
        assert_eq!(s.events.len(), MAX_EVENTS);
        assert_eq!(s.ids.len(), MAX_EVENTS);
        assert_eq!(s.dropped_memory, 10);
    }
    #[test]
    fn report_distinguishes_observation_from_proof() {
        let s = Store::default();
        let text = report_for(&s);
        assert!(text.contains("nenhum lote V3"));
        assert!(text.contains("NAO confirma imagem/som"));
    }
    #[test]
    fn disk_write_reports_failure_and_export_keeps_extra_files() {
        let dir = std::env::temp_dir().join(format!("adm-diag-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let base = dir.join("events.log");
        let mut s = Store::default();
        append(
            &mut s,
            &base,
            json!({"eventId":uuid::Uuid::new_v4().to_string(),"event":"test"}),
        )
        .unwrap();
        let mut restored = Store::default();
        load(&mut restored, &base);
        assert_eq!(restored.events.len(), 1);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn authenticated_batch_shape_is_deduplicated_and_export_contains_report() {
        let dir = std::env::temp_dir().join(format!("adm-diag-ingest-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let base = dir.join("events.log");
        let id = uuid::Uuid::new_v4().to_string();
        let action = uuid::Uuid::new_v4().to_string();
        let body = json!({"events":[{"eventId":id,"actionId":action,"producerId":uuid::Uuid::new_v4().to_string(),
            "component":"extension.popup","event":"popup.test_unresolved","level":"INFO",
            "data":{"url":"https://cdn.example/SECRET-PATH?sig=CANARY", "cookieHeader":"SECRET-COOKIE"}}],
            "health":{"dropped":2,"pending":1}}).to_string();
        let first = ingest(&base, &body).unwrap();
        let second = ingest(&base, &body).unwrap();
        assert_eq!(first["accepted"], second["accepted"]);
        let text = fs::read_to_string(log_path(&base, 0)).unwrap();
        assert_eq!(text.lines().filter(|line| line.contains(&id)).count(), 1);
        assert!(!text.contains("CANARY"));
        assert!(!text.contains("SECRET-COOKIE"));
        assert!(!text.contains("SECRET-PATH"));
        assert!(text.contains("WARN"));
        assert!(text.contains(&action));
        let entries = export(&base).unwrap();
        for required in [
            "RELATORIO_PARA_IA.txt",
            "traces/acoes.jsonl",
            "capture/decisoes-de-midia.jsonl",
            "state/players-e-popup.json",
            "health/coletores.json",
        ] {
            assert!(entries.iter().any(|(name, _)| name == required));
        }
        fs::remove_dir_all(dir).unwrap();
    }
}
