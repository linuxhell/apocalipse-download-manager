from pathlib import Path
import json

# 0.3.54: Chrome local/loopback bridge preflight compatibility + observability.
p = Path('apps/desktop/src-tauri/src/main.rs')
s = p.read_text(encoding='utf-8')

# Keep this insertion inside the existing Rust string literal. Using explicit
# \r\n escapes avoids source-line-continuation edge cases.
needle_header = '{cors}Access-Control-Allow-Headers:'
replacement_header = '{cors}Access-Control-Allow-Private-Network: true\\r\\nAccess-Control-Allow-Headers:'
if needle_header not in s:
    raise SystemExit('bridge CORS response string not found')
s = s.replace(needle_header, replacement_header, 1)

# Log every request before origin/auth so preflight failures are visible.
needle = '''    let origin = bridge_origin(headers);\n    let has_origin = headers.lines().any(|line| {\n'''
replacement = '''    let origin = bridge_origin(headers);\n    let state = app.state::<AppState>();\n    let first = headers.lines().next().unwrap_or_default();\n    let has_pna_preflight = headers.lines().any(|line| {\n        line.split_once(':').is_some_and(|(name, value)|\n            name.eq_ignore_ascii_case("access-control-request-private-network")\n                && value.trim().eq_ignore_ascii_case("true"))\n    });\n    diagnostic_log(\n        &state,\n        "INFO",\n        "bridge.request.received",\n        &format!("request={} origin={} pna={}", first, origin.unwrap_or("none"), has_pna_preflight),\n    );\n    let has_origin = headers.lines().any(|line| {\n'''
if needle not in s:
    raise SystemExit('bridge origin block not found')
s = s.replace(needle, replacement, 1)

old_late = '''    let first = headers.lines().next().unwrap_or_default();\n    if first.starts_with("OPTIONS ") {\n        bridge_response(&mut stream, "204 No Content", origin, "");\n        return;\n    }\n    let state = app.state::<AppState>();\n'''
new_late = '''    if first.starts_with("OPTIONS ") {\n        diagnostic_log(&state, "INFO", "bridge.preflight.accepted", if has_pna_preflight { "private_network=true" } else { "private_network=false" });\n        bridge_response(&mut stream, "204 No Content", origin, "");\n        return;\n    }\n'''
if old_late not in s:
    raise SystemExit('late first/state bridge block not found')
s = s.replace(old_late, new_late, 1)

old_origin = '''    if has_origin && origin.is_none() {\n        bridge_response(&mut stream, "403 Forbidden", None, "{\\"ok\\":false}");\n        return;\n    }\n'''
new_origin = '''    if has_origin && origin.is_none() {\n        diagnostic_log(&state, "WARN", "bridge.origin.rejected", "unsupported_origin");\n        bridge_response(&mut stream, "403 Forbidden", None, "{\\"ok\\":false}");\n        return;\n    }\n'''
if old_origin in s:
    s = s.replace(old_origin, new_origin, 1)
elif 'bridge.origin.rejected' not in s:
    raise SystemExit('origin reject block not found')

# 0.3.53 already adds bridge.auth.failed.
if 'bridge.auth.failed' not in s:
    old_auth = '''    if !bridge_authorized(headers, &token) {\n        bridge_response(&mut stream, "401 Unauthorized", origin, "{\\"ok\\":false}");\n        return;\n    }\n'''
    new_auth = '''    if !bridge_authorized(headers, &token) {\n        diagnostic_log(&state, "WARN", "bridge.auth.failed", "invalid_or_missing_token");\n        bridge_response(&mut stream, "401 Unauthorized", origin, "{\\"ok\\":false}");\n        return;\n    }\n'''
    if old_auth not in s:
        raise SystemExit('auth reject block not found')
    s = s.replace(old_auth, new_auth, 1)

old_health = '''    if first.starts_with("GET /v1/health ") {\n        bridge_response(&mut stream, "200 OK", origin, "{\\"ok\\":true}");\n'''
new_health = '''    if first.starts_with("GET /v1/health ") {\n        diagnostic_log(&state, "INFO", "bridge.health.ok", "extension_health_check");\n        bridge_response(&mut stream, "200 OK", origin, "{\\"ok\\":true}");\n'''
if old_health in s:
    s = s.replace(old_health, new_health, 1)
elif 'bridge.health.ok' not in s:
    raise SystemExit('health route block not found')

p.write_text(s, encoding='utf-8')

# Surface exact background bridge errors in the extension popup.
p = Path('browser-extension/popup.js')
popup = p.read_text(encoding='utf-8')
popup = popup.replace('''  chrome.runtime.sendMessage({ type: "APOCALIPSE_BRIDGE_STATUS" }, (status) => setBridgeStatus(Boolean(status?.connected)));''', '''  chrome.runtime.sendMessage({ type: "APOCALIPSE_BRIDGE_STATUS" }, (status) => {\n    const connected = Boolean(status?.connected);\n    setBridgeStatus(connected);\n    if (!connected && status?.error) showBridgeError(status.error);\n  });''', 1)
popup = popup.replace('''    setBridgeStatus(Boolean(status?.connected));\n  });\n}, 5000);''', '''    const connected = Boolean(status?.connected);\n    setBridgeStatus(connected);\n    if (!connected && status?.error) showBridgeError(status.error);\n  });\n}, 5000);''', 1)
p.write_text(popup, encoding='utf-8')

p = Path('browser-extension/manifest.json')
manifest = json.loads(p.read_text(encoding='utf-8'))
manifest['version'] = '0.3.54'
p.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

assert 'Access-Control-Allow-Private-Network: true' in s
assert 'bridge.preflight.accepted' in s
assert 'bridge.request.received' in s
assert 'bridge.auth.failed' in s
assert 'bridge.health.ok' in s
print('Applied 0.3.54 local bridge preflight + deep bridge diagnostics')
