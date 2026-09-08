from pathlib import Path
import json

# 0.3.54: Chrome local/loopback bridge preflight compatibility + observability.
# The desktop listener was confirmed alive in 0.3.53, but requests did not reach
# authentication. Add the Private Network Access response header and log the
# stages before auth so Chrome preflight failures are no longer invisible.

p = Path('apps/desktop/src-tauri/src/main.rs')
s = p.read_text(encoding='utf-8')

# Allow Chrome private/local-network preflights to opt in explicitly.
old = 'Access-Control-Allow-Headers: Authorization, Content-Type\\r\\\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\\r\\\nContent-Length: {}'
new = 'Access-Control-Allow-Headers: Authorization, Content-Type\\r\\\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\\r\\\nAccess-Control-Allow-Private-Network: true\\r\\\nContent-Length: {}'
if old not in s:
    raise SystemExit('bridge response header block not found')
s = s.replace(old, new, 1)

# Add logging immediately after parsing the HTTP request and before origin/auth.
needle = '''    let origin = bridge_origin(headers);\n    let has_origin = headers.lines().any(|line| {\n'''
replacement = '''    let origin = bridge_origin(headers);\n    let state = app.state::<AppState>();\n    let first = headers.lines().next().unwrap_or_default();\n    let has_pna_preflight = headers.lines().any(|line| {\n        line.split_once(':').is_some_and(|(name, value)|\n            name.eq_ignore_ascii_case("access-control-request-private-network")\n                && value.trim().eq_ignore_ascii_case("true"))\n    });\n    diagnostic_log(\n        &state,\n        "INFO",\n        "bridge.request.received",\n        &format!("request={} origin={} pna={}", first, origin.unwrap_or("none"), has_pna_preflight),\n    );\n    let has_origin = headers.lines().any(|line| {\n'''
if needle not in s:
    raise SystemExit('bridge origin block not found')
s = s.replace(needle, replacement, 1)

# The function previously declared first/state later. Remove those duplicate declarations.
s = s.replace('''    let first = headers.lines().next().unwrap_or_default();\n    if first.starts_with("OPTIONS ") {\n        bridge_response(&mut stream, "204 No Content", origin, "");\n        return;\n    }\n    let state = app.state::<AppState>();\n''', '''    if first.starts_with("OPTIONS ") {\n        diagnostic_log(&state, "INFO", "bridge.preflight.accepted", if has_pna_preflight { "private_network=true" } else { "private_network=false" });\n        bridge_response(&mut stream, "204 No Content", origin, "");\n        return;\n    }\n''', 1)

# Log origin rejection, auth failure and successful health explicitly.
s = s.replace('''    if has_origin && origin.is_none() {\n        bridge_response(&mut stream, "403 Forbidden", None, "{\\"ok\\":false}");\n        return;\n    }\n''', '''    if has_origin && origin.is_none() {\n        diagnostic_log(&state, "WARN", "bridge.origin.rejected", "unsupported_origin");\n        bridge_response(&mut stream, "403 Forbidden", None, "{\\"ok\\":false}");\n        return;\n    }\n''', 1)
s = s.replace('''    if !bridge_authorized(headers, &token) {\n        bridge_response(&mut stream, "401 Unauthorized", origin, "{\\"ok\\":false}");\n        return;\n    }\n''', '''    if !bridge_authorized(headers, &token) {\n        diagnostic_log(&state, "WARN", "bridge.auth.failed", "invalid_or_missing_token");\n        bridge_response(&mut stream, "401 Unauthorized", origin, "{\\"ok\\":false}");\n        return;\n    }\n''', 1)
s = s.replace('''    if first.starts_with("GET /v1/health ") {\n        bridge_response(&mut stream, "200 OK", origin, "{\\"ok\\":true}");\n''', '''    if first.starts_with("GET /v1/health ") {\n        diagnostic_log(&state, "INFO", "bridge.health.ok", "extension_health_check");\n        bridge_response(&mut stream, "200 OK", origin, "{\\"ok\\":true}");\n''', 1)

p.write_text(s, encoding='utf-8')

# Make popup display the exact bridge error returned by the service worker.
p = Path('browser-extension/popup.js')
popup = p.read_text(encoding='utf-8')
popup = popup.replace('''  chrome.runtime.sendMessage({ type: "APOCALIPSE_BRIDGE_STATUS" }, (status) => setBridgeStatus(Boolean(status?.connected)));''', '''  chrome.runtime.sendMessage({ type: "APOCALIPSE_BRIDGE_STATUS" }, (status) => {\n    const connected = Boolean(status?.connected);\n    setBridgeStatus(connected);\n    if (!connected && status?.error) showBridgeError(status.error);\n  });''', 1)
popup = popup.replace('''    setBridgeStatus(Boolean(status?.connected));\n  });\n}, 5000);''', '''    const connected = Boolean(status?.connected);\n    setBridgeStatus(connected);\n    if (!connected && status?.error) showBridgeError(status.error);\n  });\n}, 5000);''', 1)
p.write_text(popup, encoding='utf-8')

# Bump packaged extension version.
p = Path('browser-extension/manifest.json')
manifest = json.loads(p.read_text(encoding='utf-8'))
manifest['version'] = '0.3.54'
p.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

# Assertions: this round is specifically about visibility and preflight support.
assert 'Access-Control-Allow-Private-Network: true' in s
assert 'bridge.preflight.accepted' in s
assert 'bridge.request.received' in s
assert 'bridge.health.ok' in s
print('Applied 0.3.54 local bridge preflight + deep bridge diagnostics')
