//! Opt-in diagnostic sessions. Never use these observations to choose a download.
//! Detail strings are allowlisted codes or session-salted references, not user content.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_FILE: u64 = 4 * 1024 * 1024;
const MAX_BATCH: usize = 40;
const MAX_EVENTS: usize = 20_000;
const SESSION_MS: u64 = 10 * 60 * 1000;
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}
fn uuid(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok()
}
fn code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.:-".contains(&b))
}
fn reference(salt: &str, value: &str) -> String {
    let hash = Sha256::digest(format!("{salt}\0{value}").as_bytes());
    format!(
        "s-{}",
        hash[..12]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}
fn safe_detail(value: &Value, salt: &str, key: &str, depth: usize) -> Value {
    if depth > 6 {
        return json!("[depth-limit]");
    }
    let lowered = key.to_ascii_lowercase();
    if [
        "cookie",
        "authorization",
        "password",
        "passwd",
        "secret",
        "token",
        "header",
        "body",
        "title",
        "text",
        "html",
        "filename",
        "stack",
        "message",
    ]
    .iter()
    .any(|k| lowered.contains(k))
    {
        return json!("[redacted]");
    }
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => value.clone(),
        Value::String(s) => {
            if lowered.ends_with("url")
                || lowered.ends_with("src")
                || ["poster", "sourceref", "resourceref", "pageref"].contains(&lowered.as_str())
            {
                let refer = reference(salt, s).replacen("s-", "u-", 1);
                if s.is_empty() {
                    return json!({"scheme":"empty"});
                }
                if let Ok(url) = url::Url::parse(s) {
                    return json!({"ref":refer,"scheme":url.scheme(),"host":if ["http","https"].contains(&url.scheme()) {url.host_str().unwrap_or("")} else {""},
                        "queryPresent":url.query().is_some(),"fragmentPresent":url.fragment().is_some()});
                }
                return json!({"ref":refer,"scheme":"invalid"});
            }
            let is_ref = s.len() == 26
                && (s.starts_with("s-") || s.starts_with("u-"))
                && s[2..].bytes().all(|b| b.is_ascii_hexdigit());
            let is_id = (key.ends_with("Id") || key == "id" || key == "taskRefs") && uuid(s);
            let enum_key = [
                "reason",
                "kind",
                "mediaKind",
                "method",
                "component",
                "world",
                "readyState",
                "sourceScheme",
                "eventType",
                "stage",
                "status",
                "state",
                "handler",
                "result",
                "engine",
                "mode",
                "command",
                "script",
                "errorName",
                "decision",
                "transport",
                "scheme",
                "version",
            ]
            .contains(&key);
            let host = key == "host"
                && s.len() <= 200
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b".-".contains(&b));
            let mime = key == "contentType"
                && s.len() <= 96
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"/.-+".contains(&b));
            if is_ref || is_id || (enum_key && code(s)) || host || mime {
                json!(s)
            } else {
                json!(reference(salt, s))
            }
        }
        Value::Array(items) => Value::Array(
            items
                .iter()
                .take(24)
                .map(|v| safe_detail(v, salt, key, depth + 1))
                .collect(),
        ),
        Value::Object(items) => Value::Object(
            items
                .iter()
                .take(48)
                .filter(|(k, _)| code(k))
                .map(|(k, v)| (k.clone(), safe_detail(v, salt, k, depth + 1)))
                .collect(),
        ),
    }
}
#[derive(Default)]
struct Store {
    config: Value,
    received: u64,
    rejected: u64,
    duplicates: u64,
    rotated: u64,
    write_errors: u64,
    server_sequence: u64,
    ids: HashSet<String>,
    task_traces: HashMap<String, String>,
    client_health: Value,
}
pub(super) struct Diagnostics {
    inner: Mutex<Store>,
    path: PathBuf,
    meta: PathBuf,
}
impl Diagnostics {
    pub(super) fn new(directory: &Path) -> Self {
        let path = directory.join("diagnostics-v3.jsonl");
        let meta = directory.join("diagnostics-v3-session.json");
        let config = fs::read(&meta)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
            .unwrap_or(Value::Null);
        let mut store = Store {
            config,
            ..Store::default()
        };
        for file in [path.with_extension("jsonl.1"), path.clone()] {
            if let Ok(text) = fs::read_to_string(file) {
                for line in text.lines().rev().take(MAX_EVENTS) {
                    if let Ok(record) = serde_json::from_str::<Value>(line) {
                        if record["sessionId"] == store.config["sessionId"] {
                            if let Some(id) = record["id"].as_str() {
                                store.ids.insert(id.to_owned());
                            }
                            store.server_sequence = store
                                .server_sequence
                                .max(record["serverSequence"].as_u64().unwrap_or(0));
                            if let (Some(task), Some(trace)) =
                                (record["taskId"].as_str(), record["traceId"].as_str())
                            {
                                if uuid(task) && uuid(trace) {
                                    store.task_traces.insert(task.into(), trace.into());
                                }
                            }
                        }
                    }
                }
            }
        }
        Self {
            inner: Mutex::new(store),
            path,
            meta,
        }
    }
    fn active(store: &Store) -> bool {
        store.config["active"] == true && store.config["expiresAt"].as_u64().unwrap_or(0) > now()
    }
    fn config(store: &Store) -> Value {
        if !store.config.is_object() {
            return json!({"active":false,"formatVersion":3});
        }
        let mut config = store.config.clone();
        config["active"] = json!(Self::active(store));
        config
    }
    pub(super) fn status(&self) -> Value {
        self.inner
            .lock()
            .map(|store| Self::config(&store))
            .unwrap_or(json!({"active":false,"error":"diagnostics_lock_failed"}))
    }
    fn persist(&self, store: &mut Store) {
        if let Some(parent) = self.meta.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let result = (|| -> std::io::Result<()> {
            let tmp = self.meta.with_extension("tmp");
            let mut options = OpenOptions::new();
            options.write(true).create(true).truncate(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&tmp)?;
            file.write_all(store.config.to_string().as_bytes())?;
            file.sync_all()?;
            // Windows rename does not replace a file. The metadata holds only this short-lived session.
            if self.meta.exists() {
                fs::remove_file(&self.meta)?;
            }
            fs::rename(tmp, &self.meta)
        })();
        if result.is_err() {
            store.write_errors += 1;
        }
    }
    fn append(&self, store: &mut Store, mut event: Value) -> bool {
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if fs::metadata(&self.path).is_ok_and(|m| m.len() >= MAX_FILE) {
            let previous = self.path.with_extension("jsonl.1");
            let _ = fs::remove_file(&previous);
            if fs::rename(&self.path, &previous).is_err() {
                store.write_errors += 1;
                return false;
            }
            store.rotated += 1;
        }
        store.server_sequence += 1;
        event["serverSequence"] = json!(store.server_sequence);
        event["receivedAt"] = json!(now());
        let mut options = OpenOptions::new();
        options.create(true).append(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let result = options
            .open(&self.path)
            .and_then(|mut file| writeln!(file, "{event}"));
        if result.is_err() {
            store.write_errors += 1;
            false
        } else {
            store.received += 1;
            true
        }
    }
    fn native_locked(
        &self,
        store: &mut Store,
        event: &str,
        level: &str,
        trace: Option<&str>,
        task: Option<&str>,
        detail: Value,
    ) {
        if !Self::active(store) {
            return;
        }
        let salt = store.config["salt"].as_str().unwrap_or("");
        let trace = trace
            .filter(|v| uuid(v))
            .map(str::to_owned)
            .or_else(|| task.and_then(|t| store.task_traces.get(t).cloned()));
        let record = json!({"schemaVersion":3,"id":uuid::Uuid::new_v4().to_string(),"sessionId":store.config["sessionId"],
            "traceId":trace,"taskId":task.filter(|v| uuid(v)),"event":event,"level":level,"component":"desktop",
            "version":env!("CARGO_PKG_VERSION"),"build":option_env!("ADM_BUILD_SHA").unwrap_or("unknown"),
            "clientTimestamp":now(),"detail":safe_detail(&detail,salt,"",0)});
        self.append(store, record);
    }
    pub(super) fn record(
        &self,
        event: &str,
        level: &str,
        trace: Option<&str>,
        task: Option<&str>,
        detail: Value,
    ) {
        if let Ok(mut store) = self.inner.lock() {
            self.native_locked(&mut store, event, level, trace, task, detail);
        }
    }
    pub(super) fn bind_task(&self, task: &str, trace: Option<&str>) {
        if let Some(trace) = trace.filter(|v| uuid(v)) {
            if let Ok(mut store) = self.inner.lock() {
                if Self::active(&store) {
                    if store.task_traces.len() >= 2000 {
                        store.task_traces.clear();
                    }
                    store.task_traces.insert(task.into(), trace.into());
                    self.native_locked(
                        &mut store,
                        "task.trace_bound",
                        "INFO",
                        Some(trace),
                        Some(task),
                        json!({}),
                    );
                }
            }
        }
    }
    pub(super) fn control(&self, action: &str, tab_id: Option<i64>) -> Result<Value, String> {
        let mut store = self.inner.lock().map_err(|_| "diagnostics_lock_failed")?;
        match action {
            "start" => {
                if Self::active(&store) {
                    self.native_locked(
                        &mut store,
                        "session.replaced",
                        "INFO",
                        None,
                        None,
                        json!({}),
                    );
                }
                *store = Store {
                    config: json!({"formatVersion":3,"sessionId":uuid::Uuid::new_v4().to_string(),
                    "salt":format!("{}{}",uuid::Uuid::new_v4(),uuid::Uuid::new_v4()),"active":true,"tabId":tab_id.filter(|v| *v >= 0),
                    "startedAt":now(),"expiresAt":now()+SESSION_MS,"markers":0}),
                    ..Store::default()
                };
                self.native_locked(
                    &mut store,
                    "session.started",
                    "INFO",
                    None,
                    None,
                    json!({"durationMs":SESSION_MS,"tabId":tab_id}),
                );
            }
            "stop" => {
                self.native_locked(&mut store, "session.stopped", "INFO", None, None, json!({}));
                if store.config.is_object() {
                    store.config["active"] = json!(false);
                    store.config["endedAt"] = json!(now());
                }
            }
            "mark" => {
                if !Self::active(&store) {
                    return Err("diagnostics_session_not_active".into());
                }
                let marker = store.config["markers"].as_u64().unwrap_or(0) + 1;
                store.config["markers"] = json!(marker);
                self.native_locked(
                    &mut store,
                    "session.problem_marked",
                    "WARN",
                    None,
                    None,
                    json!({"marker":marker}),
                );
            }
            "clear" => {
                for path in [
                    &self.path,
                    self.path.with_extension("jsonl.1").as_path(),
                    &self.meta,
                ] {
                    if path.exists() {
                        fs::remove_file(path).map_err(|_| "diagnostics_clear_failed")?;
                    }
                }
                *store = Store::default();
            }
            _ => return Err("invalid_diagnostics_action".into()),
        }
        if action != "clear" {
            self.persist(&mut store);
        }
        Ok(Self::config(&store))
    }
    pub(super) fn accept(&self, batch: &Value) -> Value {
        let Ok(mut store) = self.inner.lock() else {
            return json!({"ok":false});
        };
        let session_matches = batch["sessionId"].as_str().is_some_and(|id| uuid(id))
            && batch["sessionId"] == store.config["sessionId"];
        let grace_end = store.config["endedAt"]
            .as_u64()
            .unwrap_or(u64::MAX)
            .min(store.config["expiresAt"].as_u64().unwrap_or(0))
            .saturating_add(60_000);
        if !session_matches || now() > grace_end {
            return json!({"ok":false,"error":"diagnostics_session_expired"});
        }
        let salt = store.config["salt"].as_str().unwrap_or("").to_owned();
        let mut ack = Vec::new();
        let mut accepted = 0;
        if let Some(events) = batch["events"].as_array() {
            for input in events.iter().take(MAX_BATCH) {
                let Some(id) = input["id"].as_str().filter(|v| uuid(v)) else {
                    store.rejected += 1;
                    continue;
                };
                if store.ids.contains(id) {
                    store.duplicates += 1;
                    ack.push(id.to_owned());
                    continue;
                }
                let name = input["event"].as_str().unwrap_or("");
                if !code(name) || input["sessionId"] != store.config["sessionId"] {
                    store.rejected += 1;
                    ack.push(id.to_owned());
                    continue;
                }
                if input["tabId"].as_i64() != store.config["tabId"].as_i64() {
                    store.rejected += 1;
                    ack.push(id.to_owned());
                    continue;
                }
                let detail = safe_detail(&input["detail"], &salt, "", 0);
                let detail = if detail.to_string().len() > 8000 {
                    json!({"truncated":true})
                } else {
                    detail
                };
                let level = input["level"]
                    .as_str()
                    .filter(|v| ["DEBUG", "INFO", "WARN", "ERROR"].contains(v))
                    .unwrap_or("INFO");
                let event = json!({"schemaVersion":3,"id":id,"sessionId":store.config["sessionId"],
                    "contextId":input["contextId"].as_str().filter(|v| uuid(v)),"sequence":input["sequence"].as_u64(),
                    "traceId":input["traceId"].as_str().filter(|v| uuid(v)),"event":name,"level":level,
                    "component":input["component"].as_str().filter(|v| ["worker","popup","content"].contains(v)).unwrap_or("extension"),
                    "version":input["version"].as_str().filter(|v| code(v)),"clientTimestamp":input["clientTimestamp"].as_u64(),
                    "monoMs":input["monoMs"].as_f64(),"tabId":input["tabId"].as_i64(),"frameId":input["frameId"].as_i64(),"detail":detail});
                if self.append(&mut store, event) {
                    if store.ids.len() >= MAX_EVENTS {
                        store.ids.clear();
                    }
                    store.ids.insert(id.into());
                    ack.push(id.to_owned());
                    accepted += 1;
                }
            }
        }
        store.client_health = safe_detail(&batch["health"], &salt, "", 0);
        json!({"ok":true,"accepted":accepted,"ackIds":ack,"writeErrors":store.write_errors})
    }
    pub(super) fn observe_legacy(&self, level: &str, event: &str, detail: &str) {
        if event.starts_with("ui.command_")
            || event.starts_with("bridge.")
            || event.starts_with("extension.")
            || level == "DEBUG"
        {
            return;
        }
        let field = |name: &str| {
            detail
                .split_whitespace()
                .find_map(|v| v.strip_prefix(name))
                .filter(|v| uuid(v))
        };
        let mut data = json!({"detailRef":detail});
        for name in ["engine", "mode", "state", "reason"] {
            if let Some(value) = detail
                .split_whitespace()
                .find_map(|v| v.strip_prefix(&format!("{name}=")))
            {
                data[name] = json!(value);
            }
        }
        self.record(event, level, field("trace="), field("task="), data);
    }
    fn records_locked(&self, session_id: &Value) -> (Vec<Value>, u64) {
        let mut records = Vec::new();
        let mut parse_errors = 0;
        for path in [self.path.with_extension("jsonl.1"), self.path.clone()] {
            if let Ok(text) = fs::read_to_string(path) {
                for line in text.lines() {
                    match serde_json::from_str::<Value>(line) {
                        Ok(event) if event["sessionId"] == *session_id => records.push(event),
                        Ok(_) => {}
                        Err(_) => parse_errors += 1,
                    }
                }
            }
        }
        records.sort_by_key(|v| v["serverSequence"].as_u64().unwrap_or(0));
        (records, parse_errors)
    }
    pub(super) fn export(&self) -> Vec<(String, Vec<u8>)> {
        let Ok(store) = self.inner.lock() else {
            return vec![(
                "health/collectors.json".into(),
                br#"{"error":"diagnostics_lock_failed"}"#.to_vec(),
            )];
        };
        let (records, parse_errors) = self.records_locked(&store.config["sessionId"]);
        let health = json!({"formatVersion":3,"sessionId":store.config["sessionId"],"active":Self::active(&store),
            "startedAt":store.config["startedAt"],"expiresAt":store.config["expiresAt"],"endedAt":store.config["endedAt"],
            "exportedAt":now(),"eventsInSnapshot":records.len(),"receivedThisProcess":store.received,"rejected":store.rejected,
            "duplicates":store.duplicates,"rotationsThisProcess":store.rotated,"writeErrors":store.write_errors,"parseErrors":parse_errors,
            "clientHealth":store.client_health,"maxBytesPerFile":MAX_FILE,"filesRetained":2,
            "limitations":["webRequest_does_not_observe_memory_cache","cross_world_probe_is_not_trusted_page_evidence",
            "closed_page_may_lose_unflushed_events","player_started_does_not_prove_picture_or_sound","only_selected_tab_is_observed",
            "no_page_text_cookies_credentials_or_media_bytes_collected","timestamps_from_different_contexts_are_not_causal_order"]});
        let mut actions = HashMap::<String, Vec<&Value>>::new();
        let mut players = HashMap::<String, Value>::new();
        let mut decisions = Vec::new();
        let mut contexts = HashSet::new();
        let mut warnings = Vec::new();
        for record in &records {
            if let Some(trace) = record["traceId"].as_str() {
                actions.entry(trace.into()).or_default().push(record);
            }
            if let Some(context) = record["contextId"].as_str() {
                contexts.insert(context.to_owned());
            }
            let name = record["event"].as_str().unwrap_or("");
            if name.starts_with("capture.")
                || name.starts_with("identity.")
                || name.contains("candidate")
            {
                decisions.push(record.clone());
            }
            if name == "players.snapshot" || name == "popup.render" || name == "popup.row_state" {
                let key = format!("{}:{}", record["contextId"], name);
                players.insert(key, record.clone());
            }
            if ["ERROR", "WARN"].contains(&record["level"].as_str().unwrap_or("")) {
                warnings.push(record);
            }
        }
        let jsonl = |items: &[Value]| {
            items
                .iter()
                .map(|v| format!("{v}\n"))
                .collect::<String>()
                .into_bytes()
        };
        let mut action_rows = actions.iter().map(|(trace,items)| json!({"traceId":trace,"events":items,
            "firstSequence":items.first().map(|v| &v["serverSequence"]),
            "lastStage":items.last().map(|v| &v["event"]),"interpretation":"last_observed_stage_not_proven_root_cause"})).collect::<Vec<_>>();
        action_rows.sort_by_key(|v| v["firstSequence"].as_u64().unwrap_or(0));
        let mut report = format!("ADM - DIAGNOSTICO V3 / DIAGNOSTICS V3\n\nAplicativo / Application: {}\nBuild: {}\nSessao / Session: {}\nEventos / Events: {} | Contextos / Contexts: {} | WARN/ERROR: {}\n\nFATOS REGISTRADOS / RECORDED FACTS\n",
            env!("CARGO_PKG_VERSION"),option_env!("ADM_BUILD_SHA").unwrap_or("unknown"),store.config["sessionId"],records.len(),contexts.len(),warnings.len());
        if records.is_empty() {
            report.push_str("Nenhuma sessao detalhada registrada. Inicie o diagnostico na extensao, reproduza o problema e exporte novamente. / No detailed session recorded. Start diagnostics in the extension, reproduce, then export.\n");
        }
        for record in warnings.iter().rev().take(30).rev() {
            report.push_str(&format!(
                "#{} {} {} trace={} detail={}\n",
                record["serverSequence"],
                record["level"],
                record["event"],
                record["traceId"],
                record["detail"]
            ));
        }
        report.push_str("\nLINHA DO TEMPO POR TENTATIVA / ACTION TIMELINES (last 30)\n");
        for action in action_rows.iter().rev().take(30).rev() {
            report.push_str(&format!("\nTrace {}\n", action["traceId"]));
            if let Some(events) = action["events"].as_array() {
                for record in events.iter().take(60) {
                    report.push_str(&format!(
                        "  #{} {} {} task={}\n",
                        record["serverSequence"],
                        record["component"],
                        record["event"],
                        record["taskId"]
                    ));
                }
            }
        }
        report.push_str(&format!(
            "\nSAUDE DOS COLETORES / COLLECTOR HEALTH\n{}\n",
            serde_json::to_string_pretty(&health).unwrap_or_default()
        ));
        report.push_str("\nLIMITES / LIMITATIONS\nAusencia de evento nao prova ausencia de atividade. O ultimo evento nao e uma causa confirmada.\nNo event does not prove no activity. The last observed event is not a confirmed root cause.\nCache em memoria, fechamento de popup/aba e interrupcoes podem deixar lacunas. / Memory cache, closed pages and interruptions may leave gaps.\nSomente a aba escolhida e observada. Abrir o processo do player nao prova imagem ou som. / Only the selected tab is observed. Player process startup does not prove picture or sound.\nNao publicar os logs antigos ou dados pessoais sem revisar. / Review legacy logs and personal data before sharing.\n");
        if report.len() > 128 * 1024 {
            let mut limit = 128 * 1024;
            while !report.is_char_boundary(limit) {
                limit -= 1;
            }
            report.truncate(limit);
            report.push_str(
                "\n[REPORT TRUNCATED - full bounded events are in logs/diagnostics-v3.jsonl]\n",
            );
        }
        vec![
            ("RELATORIO_PARA_IA.txt".into(), report.into_bytes()),
            ("logs/diagnostics-v3.jsonl".into(), jsonl(&records)),
            ("traces/actions.jsonl".into(), jsonl(&action_rows)),
            ("capture/media-decisions.jsonl".into(), jsonl(&decisions)),
            (
                "state/players-popup.json".into(),
                serde_json::to_vec_pretty(&players).unwrap_or_default(),
            ),
            (
                "health/collectors.json".into(),
                serde_json::to_vec_pretty(&health).unwrap_or_default(),
            ),
        ]
    }
    pub(super) fn report(&self) -> String {
        self.export()
            .into_iter()
            .find(|(name, _)| name == "RELATORIO_PARA_IA.txt")
            .map(|(_, bytes)| String::from_utf8_lossy(&bytes).into_owned())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (Diagnostics, PathBuf) {
        let path = std::env::temp_dir().join(format!("adm-diag-test-{}", uuid::Uuid::new_v4()));
        (Diagnostics::new(&path), path)
    }
    #[test]
    fn redaction_does_not_store_user_text_or_credentials() {
        let value = json!({"cookie":"secret-cookie","token":"secret-token","url":"https://u:p@video.example/private?sig=SECRET#frag",
            "title":"private-title","errorRef":"Error private local path", "reason":"host_not_in_capture_filter","contentType":"video/mp4"});
        let safe = safe_detail(&value, "salt", "", 0).to_string();
        for secret in [
            "secret-cookie",
            "secret-token",
            "SECRET",
            "private-title",
            "u:p",
            "private local path",
        ] {
            assert!(!safe.contains(secret));
        }
        assert!(safe.contains("host_not_in_capture_filter"));
        assert!(safe.contains("video/mp4"));
    }
    #[test]
    fn session_is_opt_in_and_expires() {
        let (diag, path) = setup();
        diag.record("test.event", "INFO", None, None, json!({}));
        assert!(!path.join("diagnostics-v3.jsonl").exists());
        let config = diag.control("start", Some(7)).unwrap();
        assert_eq!(config["tabId"], 7);
        {
            let mut store = diag.inner.lock().unwrap();
            store.config["expiresAt"] = json!(now() - 1);
        }
        assert_eq!(diag.status()["active"], false);
        let _ = fs::remove_dir_all(path);
    }
    #[test]
    fn retries_are_idempotent_and_export_retains_trace_and_marker() {
        let (diag, path) = setup();
        let config = diag.control("start", Some(7)).unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let trace = uuid::Uuid::new_v4().to_string();
        let batch = json!({"sessionId":config["sessionId"],"events":[{"id":id,"sessionId":config["sessionId"],"traceId":trace,
            "event":"popup.click","tabId":7,"component":"popup","level":"INFO","detail":{"cookie":"PRIVATE"}}]});
        assert_eq!(diag.accept(&batch)["accepted"], 1);
        assert_eq!(diag.accept(&batch)["accepted"], 0);
        diag.control("mark", None).unwrap();
        diag.control("stop", None).unwrap();
        let report = diag.report();
        assert!(report.contains(&trace));
        assert!(report.contains("session.problem_marked"));
        assert!(!report.contains("PRIVATE"));
        let restored = Diagnostics::new(&path);
        assert_eq!(restored.accept(&batch)["accepted"], 0);
        let _ = fs::remove_dir_all(path);
    }
    #[test]
    fn task_binding_and_queue_changes_have_same_trace() {
        let (diag, path) = setup();
        diag.control("start", None).unwrap();
        let task = uuid::Uuid::new_v4().to_string();
        let trace = uuid::Uuid::new_v4().to_string();
        diag.bind_task(&task, Some(&trace));
        diag.observe_legacy(
            "ERROR",
            "http.failed",
            &format!("task={task} error=private"),
        );
        assert!(diag.report().contains("http.failed"));
        let records: Vec<_> = diag.export();
        let (_, raw) = records
            .iter()
            .find(|(n, _)| n == "logs/diagnostics-v3.jsonl")
            .unwrap();
        let found = String::from_utf8_lossy(raw)
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .find(|v| v["event"] == "http.failed")
            .unwrap();
        assert_eq!(found["traceId"], trace);
        let _ = fs::remove_dir_all(path);
    }
    #[test]
    fn foreign_session_and_wrong_tab_do_not_enter_exports() {
        let (diag, path) = setup();
        let cfg = diag.control("start", Some(7)).unwrap();
        let batch = json!({"sessionId":cfg["sessionId"],"events":[{"id":uuid::Uuid::new_v4().to_string(),"sessionId":cfg["sessionId"],"tabId":8,"event":"wrong.tab"}]});
        assert_eq!(diag.accept(&batch)["accepted"], 0);
        let foreign = json!({"sessionId":uuid::Uuid::new_v4().to_string(),"events":[]});
        assert_eq!(diag.accept(&foreign)["ok"], false);
        let _ = fs::remove_dir_all(path);
    }
    #[test]
    fn url_refs_match_javascript_salted_sha256_and_no_paths_are_exported() {
        let value = safe_detail(
            &json!("https://video.example/PRIVATE?sig=SECRET"),
            "salt",
            "url",
            0,
        );
        assert_eq!(value["host"], "video.example");
        assert_eq!(
            value["ref"],
            reference("salt", "https://video.example/PRIVATE?sig=SECRET").replacen("s-", "u-", 1)
        );
        assert!(!value.to_string().contains("PRIVATE"));
        assert!(!value.to_string().contains("SECRET"));
    }
    #[test]
    fn clear_removes_detailed_session_and_preserves_other_files() {
        let (diag, path) = setup();
        diag.control("start", None).unwrap();
        fs::write(path.join("apocalipse.log"), "keep legacy").unwrap();
        diag.control("clear", None).unwrap();
        assert_eq!(diag.status()["active"], false);
        assert!(!path.join("diagnostics-v3.jsonl").exists());
        assert!(path.join("apocalipse.log").exists());
        let _ = fs::remove_dir_all(path);
    }
}
