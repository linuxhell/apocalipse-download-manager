from pathlib import Path

cargo = Path('apps/desktop/src-tauri/Cargo.toml')
s = cargo.read_text()
if 'chrono = ' not in s:
    s += '\nchrono = "0.4"\nzip = { version = "2", default-features = false, features = ["deflate"] }\n'
cargo.write_text(s)

p = Path('apps/desktop/src-tauri/src/diagnostics_v3.rs')
s = p.read_text()
if 'use chrono::Local;' not in s:
    s = s.replace('use serde_json::json;\n', 'use chrono::Local;\nuse serde_json::json;\n', 1)
old = '''    let timestamp = now_ms();
    let sanitized = sanitize_detail(detail);
    let trace = trace_for(&sanitized);
    let host = host_for(&sanitized);
    let status = status_for(&sanitized);
    let elapsed = elapsed_for(event, trace.as_deref(), timestamp);
    let record = json!({
        "schema": SCHEMA_VERSION,
        "tsMs": timestamp.to_string(),
        "seq": SEQUENCE.fetch_add(1, Ordering::Relaxed),
        "session": session_id(),
        "level": level.chars().take(12).collect::<String>(),
        "event": event.chars().take(96).collect::<String>(),
        "trace": trace,
        "host": host,
        "status": status,
        "elapsedMs": elapsed.map(|value| value.min(u64::MAX as u128) as u64),
        "detail": sanitized,
    });'''
new = '''    let timestamp = now_ms();
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
    });'''
if old not in s:
    raise SystemExit('diagnostics write_event marker not found')
s = s.replace(old, new, 1)
p.write_text(s)

p = Path('browser-extension/background.js')
s = p.read_text()
marker = '    extra.detail || "",\n'
if 'extension_version=' not in s:
    s = s.replace(marker, marker + '    `extension_version=${chrome.runtime.getManifest().version}`,\n', 1)
p.write_text(s)

p = Path('apps/desktop/src-tauri/src/main.rs')
s = p.read_text()
start = s.find('#[tauri::command]\nfn export_diagnostics(')
end = s.find('#[tauri::command]\nfn get_log_editor(', start)
if start < 0 or end < 0:
    raise SystemExit('diagnostic export markers not found')
replacement = r'''#[tauri::command]
fn export_diagnostics(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    use zip::write::SimpleFileOptions;

    let directory = configured_download_directory(&app, &state)?;
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let local = chrono::Local::now();
    let base = format!("Apocalipse-Diagnostico-{}", local.format("%Y-%m-%d_%H-%M-%S"));
    let mut destination = directory.join(format!("{base}.zip"));
    let mut suffix = 2_u32;
    while destination.exists() {
        destination = directory.join(format!("{base}-{suffix}.zip"));
        suffix += 1;
    }
    let file = OpenOptions::new().create_new(true).write(true).open(&destination).map_err(|error| error.to_string())?;
    let mut archive = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    archive.start_file("events.jsonl", options).map_err(|error| error.to_string())?;
    for index in (1..=4).rev() {
        let path = state.log_path.with_extension(format!("log.{index}"));
        if let Ok(contents) = fs::read_to_string(path) {
            archive.write_all(contents.as_bytes()).map_err(|error| error.to_string())?;
            if !contents.ends_with('\n') {
                archive.write_all(b"\n").map_err(|error| error.to_string())?;
            }
        }
    }
    if let Ok(contents) = fs::read_to_string(&state.log_path) {
        archive.write_all(contents.as_bytes()).map_err(|error| error.to_string())?;
    }

    archive.start_file("summary.txt", options).map_err(|error| error.to_string())?;
    writeln!(archive, "Apocalipse Download Manager diagnostic export").map_err(|error| error.to_string())?;
    writeln!(archive, "Generated: {}", local.to_rfc3339()).map_err(|error| error.to_string())?;
    writeln!(archive, "App version: {}", env!("CARGO_PKG_VERSION")).map_err(|error| error.to_string())?;
    writeln!(archive, "Log schema: 3").map_err(|error| error.to_string())?;
    writeln!(archive, "Clear internal logs never deletes this exported ZIP.").map_err(|error| error.to_string())?;

    archive.start_file("system.json", options).map_err(|error| error.to_string())?;
    let system = serde_json::json!({
        "appVersion": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "generatedAt": local.to_rfc3339(),
        "diagnosticSchema": 3
    });
    archive.write_all(serde_json::to_string_pretty(&system).map_err(|error| error.to_string())?.as_bytes()).map_err(|error| error.to_string())?;

    archive.start_file("matrix-rules.json", options).map_err(|error| error.to_string())?;
    let rules = state.site_rules.lock().map_err(|error| error.to_string())?.clone();
    archive.write_all(serde_json::to_string_pretty(&rules).map_err(|error| error.to_string())?.as_bytes()).map_err(|error| error.to_string())?;

    archive.finish().map_err(|error| error.to_string())?;
    diagnostic_log(&state, "INFO", "diagnostics.exported", &format!("file={}", destination.file_name().and_then(|name| name.to_str()).unwrap_or("diagnostic.zip")));
    Ok(destination.display().to_string())
}

#[tauri::command]
fn clear_general_log(state: State<'_, AppState>) -> Result<(), String> {
    {
        let _write_guard = state.log_write_lock.lock().map_err(|error| error.to_string())?;
        let mut paths = vec![state.log_path.clone()];
        paths.extend((1..=4).map(|index| state.log_path.with_extension(format!("log.{index}"))));
        for path in paths {
            match fs::remove_file(path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.to_string()),
            }
        }
    }
    diagnostic_log(&state, "INFO", "log.cleared", "cleared_by_user scope=internal_logs_only");
    Ok(())
}

'''
s = s[:start] + replacement + s[end:]
p.write_text(s)
