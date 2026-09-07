use std::{fs, path::Path};

fn function_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let open = source[start..].find('{')? + start;
    let mut depth = 0_i32;
    let mut index = open;
    let mut string = false;
    let mut character = false;
    let mut escape = false;
    let mut line_comment = false;
    let mut block_comment = 0_u32;
    while index < bytes.len() {
        let current = bytes[index];
        let next = bytes.get(index + 1).copied();
        if line_comment {
            if current == b'\n' {
                line_comment = false;
            }
            index += 1;
            continue;
        }
        if block_comment > 0 {
            if current == b'/' && next == Some(b'*') {
                block_comment += 1;
                index += 2;
                continue;
            }
            if current == b'*' && next == Some(b'/') {
                block_comment -= 1;
                index += 2;
                continue;
            }
            index += 1;
            continue;
        }
        if string {
            if escape {
                escape = false;
            } else if current == b'\\' {
                escape = true;
            } else if current == b'"' {
                string = false;
            }
            index += 1;
            continue;
        }
        if character {
            if escape {
                escape = false;
            } else if current == b'\\' {
                escape = true;
            } else if current == b'\'' {
                character = false;
            }
            index += 1;
            continue;
        }
        if current == b'/' && next == Some(b'/') {
            line_comment = true;
            index += 2;
            continue;
        }
        if current == b'/' && next == Some(b'*') {
            block_comment = 1;
            index += 2;
            continue;
        }
        if current == b'"' {
            string = true;
            index += 1;
            continue;
        }
        if current == b'\'' {
            character = true;
            index += 1;
            continue;
        }
        match current {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index + 1);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn replace_function(source: &mut String, signature: &str, replacement: &str) {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("unable to find function signature: {signature}"));
    let end = function_end(source, start)
        .unwrap_or_else(|| panic!("unable to find function end: {signature}"));
    source.replace_range(start..end, replacement);
}

fn patch_main(manifest: &Path) {
    let path = manifest.join("src/main.rs");
    let mut source = fs::read_to_string(&path).expect("read main.rs");

    if !source.contains("mod diagnostics_v3;") {
        let marker = "#![cfg_attr(not(debug_assertions), windows_subsystem = \"windows\")]\n";
        source = source.replacen(marker, &format!("{marker}\nmod diagnostics_v3;\n"), 1);
    }

    if !source.contains("struct BridgeDiagnosticEvent") {
        let marker = "#[derive(Deserialize)]\n#[serde(rename_all = \"camelCase\")]\nstruct BlobBegin {";
        let diagnostic = r#"#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BridgeDiagnosticEvent {
    event: String,
    level: Option<String>,
    trace_id: Option<String>,
    source: Option<String>,
    url: Option<String>,
    status: Option<u16>,
    bytes: Option<u64>,
    duration_ms: Option<u64>,
    detail: Option<String>,
}

"#;
        source = source.replacen(marker, &format!("{diagnostic}{marker}"), 1);
    }

    if !source.contains("POST /v1/diagnostic ") {
        let marker = "    } else if first.starts_with(\"POST /v1/blob/begin \") {";
        let route = r#"    } else if first.starts_with("POST /v1/diagnostic ") {
        match serde_json::from_str::<BridgeDiagnosticEvent>(body) {
            Ok(request)
                if !request.event.trim().is_empty()
                    && request.event.len() <= 96
                    && request
                        .event
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')) =>
            {
                let level = request
                    .level
                    .as_deref()
                    .map(str::to_ascii_uppercase)
                    .filter(|level| matches!(level.as_str(), "DEBUG" | "INFO" | "WARN" | "ERROR"))
                    .unwrap_or_else(|| "INFO".to_owned());
                let trace = request
                    .trace_id
                    .as_deref()
                    .filter(|value| value.len() <= 128 && !value.contains(['\r', '\n']))
                    .unwrap_or("none");
                let source = request
                    .source
                    .as_deref()
                    .filter(|value| value.len() <= 64 && !value.contains(['\r', '\n']))
                    .unwrap_or("browser");
                let url = request
                    .url
                    .as_deref()
                    .filter(|value| value.len() <= 4096 && !value.contains(['\r', '\n']))
                    .unwrap_or("none");
                let detail = request
                    .detail
                    .as_deref()
                    .filter(|value| value.len() <= 8192 && !value.contains(['\r', '\n']))
                    .unwrap_or("");
                diagnostic_log(
                    &app.state::<AppState>(),
                    &level,
                    &request.event,
                    &format!(
                        "trace={trace} source={source} url={url} status={} bytes={} duration_ms={} {detail}",
                        request.status.map_or_else(|| "none".to_owned(), |value| value.to_string()),
                        request.bytes.map_or_else(|| "none".to_owned(), |value| value.to_string()),
                        request.duration_ms.map_or_else(|| "none".to_owned(), |value| value.to_string()),
                    ),
                );
                bridge_response(&mut stream, "202 Accepted", origin, "{\"ok\":true}");
            }
            _ => bridge_response(&mut stream, "400 Bad Request", origin, "{\"ok\":false}"),
        }
"#;
        source = source.replacen(marker, &format!("{route}{marker}"), 1);
    }

    replace_function(
        &mut source,
        "fn diagnostic_log(",
        r#"fn diagnostic_log(state: &AppState, level: &str, event: &str, detail: &str) {
    diagnostics_v3::write_event(&state.log_path, &state.log_write_lock, level, event, detail);
}"#,
    );

    replace_function(
        &mut source,
        "fn matrix_analyze(",
        r#"fn matrix_analyze(state: State<'_, AppState>) -> Result<MatrixStatus, String> {
    let queue = state.queue.lock().map_err(|error| error.to_string())?;
    let rules = state.site_rules.lock().map_err(|error| error.to_string())?;
    let mut signals = diagnostics_v3::analyze(&state.log_path);
    let mut queue_failures = HashMap::<String, HashSet<String>>::new();
    for task in queue
        .iter()
        .filter(|task| matches!(&task.state, DownloadState::Failed { .. }))
    {
        if let Ok(url) = url::Url::parse(&task.source) {
            if let Some(host) = url.host_str() {
                queue_failures
                    .entry(host.to_ascii_lowercase())
                    .or_default()
                    .insert(task.id.to_string());
            }
        }
    }
    for (host, task_ids) in queue_failures {
        signals
            .entry(host.clone())
            .or_insert_with(|| diagnostics_v3::HostSignal { host, ..Default::default() })
            .add_queue_failures(task_ids.len());
    }

    let mut proposals = signals
        .into_iter()
        .filter(|(host, signal)| {
            matching_site_rule(&format!("https://{host}/"), &rules).is_none()
                && signal.conservative_rule_is_safe()
        })
        .map(|(host, signal)| MatrixRuleProposal {
            failures: signal.failures,
            confidence: signal.confidence(),
            reason: signal.reason(),
            host,
        })
        .collect::<Vec<_>>();
    proposals.sort_by(|left, right| {
        right
            .confidence
            .cmp(&left.confidence)
            .then_with(|| right.failures.cmp(&left.failures))
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
        .collect::<Vec<_>>();
    drop(queue);
    drop(rules);
    diagnostic_log(
        &state,
        "INFO",
        "matrix.v3.analyzed",
        &format!("proposals={} active_rules={}", proposals.len(), applied_rules.len()),
    );
    Ok(MatrixStatus {
        version: "Matrix Ultimate v3 AI".to_owned(),
        active_rules: applied_rules.len(),
        proposals,
        applied_rules,
    })
}"#,
    );

    fs::write(path, source).expect("write patched main.rs");
}

fn patch_ui(manifest: &Path) {
    let ui = manifest.parent().expect("src-tauri parent").join("ui");
    for name in ["app.js", "index.html"] {
        let path = ui.join(name);
        let Ok(mut source) = fs::read_to_string(&path) else { continue; };
        source = source.replace("Matrix Ultimate v2 AI", "Matrix Ultimate v3 AI");
        source = source.replace(
            "Matrix continuously monitors local failures and proposes safe rules. No rule executes website code.",
            "Matrix v3 correlates structured traces, HTTP failures, browser handoffs and content mismatches before proposing a safe rule. No rule executes website code.",
        );
        source = source.replace(
            "A Matrix monitora continuamente as falhas locais e propõe regras seguras. Nenhuma regra executa código de sites.",
            "A Matrix v3 correlaciona rastros estruturados, falhas HTTP, transferências do navegador e respostas incompatíveis antes de propor uma regra segura. Nenhuma regra executa código de sites.",
        );
        fs::write(path, source).expect("write patched UI");
    }
}

fn main() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let manifest = Path::new(&manifest);
    patch_main(manifest);
    patch_ui(manifest);
    println!("cargo:rerun-if-changed=src/diagnostics_v3.rs");
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=../ui/app.js");
    tauri_build::build()
}
