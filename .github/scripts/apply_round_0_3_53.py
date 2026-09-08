from pathlib import Path
import json

# 0.3.53: make bridge connectivity failures observable.
# The desktop previously continued normally when port 17654 could not be bound,
# and unauthorized extension requests returned 401 without entering diagnostics.

p = Path('apps/desktop/src-tauri/src/main.rs')
s = p.read_text(encoding='utf-8')

old = '''            let bridge_listener = match TcpListener::bind(("127.0.0.1", BRIDGE_PORT)) {
                Ok(listener) => Some(listener),
                Err(_) => {
                    if associated_source.as_deref().is_some_and(|source| {
                        forward_to_running_instance(source, &initial_settings.bridge_token)
                    }) {
                        app.handle().exit(0);
                        return Ok(());
                    }
                    None
                }
            };'''
new = '''            let (bridge_listener, bridge_bind_error) = match TcpListener::bind(("127.0.0.1", BRIDGE_PORT)) {
                Ok(listener) => (Some(listener), None),
                Err(error) => {
                    if associated_source.as_deref().is_some_and(|source| {
                        forward_to_running_instance(source, &initial_settings.bridge_token)
                    }) {
                        app.handle().exit(0);
                        return Ok(());
                    }
                    (None, Some(error.to_string()))
                }
            };'''
if old not in s:
    raise SystemExit('bridge listener setup block not found')
s = s.replace(old, new, 1)

old = '''            diagnostic_log(
                &app.state::<AppState>(),
                "INFO",
                "application.started",
                env!("CARGO_PKG_VERSION"),
            );
            if let Some(listener) = bridge_listener {'''
new = '''            diagnostic_log(
                &app.state::<AppState>(),
                "INFO",
                "application.started",
                env!("CARGO_PKG_VERSION"),
            );
            if let Some(error) = bridge_bind_error.as_deref() {
                diagnostic_log(
                    &app.state::<AppState>(),
                    "ERROR",
                    "bridge.listener.failed",
                    &format!("address=127.0.0.1:{BRIDGE_PORT} error={error}"),
                );
            } else {
                diagnostic_log(
                    &app.state::<AppState>(),
                    "INFO",
                    "bridge.listener.started",
                    &format!("address=127.0.0.1:{BRIDGE_PORT}"),
                );
            }
            if let Some(listener) = bridge_listener {'''
if old not in s:
    raise SystemExit('application.started block not found')
s = s.replace(old, new, 1)

old = '''    if !bridge_authorized(headers, &token) {
        bridge_response(&mut stream, "401 Unauthorized", origin, "{\\\"ok\\\":false}");
        return;
    }'''
new = '''    if !bridge_authorized(headers, &token) {
        let request_line = first.split_whitespace().take(2).collect::<Vec<_>>().join(" ");
        diagnostic_log(
            &state,
            "WARN",
            "bridge.auth.failed",
            &format!("request={} origin={}", request_line, origin.unwrap_or("none")),
        );
        bridge_response(&mut stream, "401 Unauthorized", origin, "{\\\"ok\\\":false}");
        return;
    }'''
if old not in s:
    raise SystemExit('bridge authorization block not found')
s = s.replace(old, new, 1)

# Do not log every heartbeat forever, but record that an authenticated extension
# has actually reached the bridge at least through the existing bridge_last_seen state.
old = '''    if first.starts_with("GET /v1/health ") {
        bridge_response(&mut stream, "200 OK", origin, "{\\\"ok\\\":true}");'''
new = '''    if first.starts_with("GET /v1/health ") {
        bridge_response(&mut stream, "200 OK", origin, "{\\\"ok\\\":true}");'''
if old not in s:
    raise SystemExit('health route not found')

p.write_text(s, encoding='utf-8')

p = Path('browser-extension/manifest.json')
manifest = json.loads(p.read_text(encoding='utf-8'))
manifest['version'] = '0.3.53'
p.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

assert 'bridge.listener.failed' in s
assert 'bridge.listener.started' in s
assert 'bridge.auth.failed' in s
print('Applied 0.3.53 bridge observability correction')
