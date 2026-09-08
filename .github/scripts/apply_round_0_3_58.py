from pathlib import Path
import json
import re

# 0.3.58: per-download connection/thread override in the Add Download dialog.
# Default is 8; user can choose 1..32 after analysis. The override is task-local
# and must take precedence over adaptive/global connection settings.

# --- UI markup ---
p = Path('apps/desktop/ui/index.html')
s = p.read_text(encoding='utf-8')
anchor = '<div id="analysis" class="analysis" hidden></div>\n'
control = '''<div id="analysis" class="analysis" hidden></div>\n        <section id="download-connections-control" class="download-connections-control" hidden>\n          <div><label for="download-connections" data-i18n="downloadThreads">Threads / connections</label><output id="download-connections-value">8</output></div>\n          <input id="download-connections" type="range" min="1" max="32" step="1" value="8" />\n          <small data-i18n="downloadThreadsHint">Per-download override. Some sites only work with 1 connection.</small>\n        </section>\n'''
if 'id="download-connections"' not in s:
    if anchor not in s:
        raise SystemExit('analysis markup anchor not found')
    s = s.replace(anchor, control, 1)
p.write_text(s, encoding='utf-8')

# --- UI behavior / translations ---
p = Path('apps/desktop/ui/app.js')
js = p.read_text(encoding='utf-8')

# Add compact translations next to existing bandwidth strings. Use direct object
# insertion so every shipped language has a readable label.
for lang, label, hint in [
    ('en', 'Threads / connections', 'Per-download override. Some sites only work with 1 connection.'),
    ('pt-BR', 'Threads / conexões', 'Ajuste somente deste download. Alguns sites funcionam apenas com 1 conexão.'),
    ('zh-CN', '线程 / 连接数', '仅对此下载生效。有些网站只能使用 1 个连接。'),
]:
    # Find the language catalog block and inject once after sourceUrl.
    block_start = js.find(f'  {json.dumps(lang)}: {{') if lang != 'en' else js.find('  en: {')
    if block_start < 0:
        raise SystemExit(f'catalog {lang} not found')
    block_end = js.find('\n  },', block_start)
    chunk = js[block_start:block_end]
    if 'downloadThreads:' not in chunk:
        source_match = re.search(r'(\n\s*sourceUrl:\s*[^\n]+\n)', chunk)
        if not source_match:
            raise SystemExit(f'sourceUrl anchor missing in {lang}')
        insertion = source_match.group(1) + f'    downloadThreads: {json.dumps(label, ensure_ascii=False)},\n    downloadThreadsHint: {json.dumps(hint, ensure_ascii=False)},\n'
        chunk = chunk[:source_match.start()] + insertion + chunk[source_match.end():]
        js = js[:block_start] + chunk + js[block_end:]

# Helpers keep the slider deterministic: every new task starts at 8 and it is
# shown only after a successful Analyze step.
helper_anchor = 'function resetMediaInspection() {'
helper = '''function resetDownloadConnections() {\n  const slider = document.querySelector("#download-connections");\n  const value = document.querySelector("#download-connections-value");\n  const control = document.querySelector("#download-connections-control");\n  if (slider) slider.value = "8";\n  if (value) value.value = "8";\n  if (control) control.hidden = true;\n}\nfunction showDownloadConnections() {\n  const control = document.querySelector("#download-connections-control");\n  if (control) control.hidden = false;\n}\n\n'''
if 'function resetDownloadConnections()' not in js:
    if helper_anchor not in js:
        raise SystemExit('resetMediaInspection anchor not found')
    js = js.replace(helper_anchor, helper + helper_anchor, 1)

# Update output while dragging.
listener_anchor = 'document.querySelector("#url").oninput = () => {'
listener = '''const downloadConnectionsSlider = document.querySelector("#download-connections");\nif (downloadConnectionsSlider) downloadConnectionsSlider.oninput = (event) => {\n  document.querySelector("#download-connections-value").value = event.target.value;\n};\n'''
if 'downloadConnectionsSlider.oninput' not in js:
    js = js.replace(listener_anchor, listener + listener_anchor, 1)

# Reset for new task and whenever URL changes.
js = js.replace('      resetMediaInspection();\n      invoke("default_download_directory")', '      resetMediaInspection();\n      resetDownloadConnections();\n      invoke("default_download_directory")', 1)
js = js.replace('  resetMediaInspection();\n};\n\nfunction secondsLabel', '  resetMediaInspection();\n  resetDownloadConnections();\n};\n\nfunction secondsLabel', 1)

# Successful analysis reveals the per-download slider immediately above Add.
analyze_marker = '    document.querySelector("#analyze").hidden = true;\n    document.querySelector("#enqueue").hidden = false;'
if analyze_marker not in js:
    raise SystemExit('analyze success marker not found')
js = js.replace(analyze_marker, '    showDownloadConnections();\n' + analyze_marker, 1)

# Pass the selected override with this task only.
context_marker = '        context: {\n          referer: pendingReferer,'
if context_marker not in js:
    raise SystemExit('enqueue context marker not found')
js = js.replace(context_marker, '        context: {\n          connectionsOverride: Number(document.querySelector("#download-connections").value) || 8,\n          referer: pendingReferer,', 1)

# Reset after enqueue.
js = js.replace('    document.querySelector("#download-bandwidth-limit").value = "0";\n    resetMediaInspection();', '    document.querySelector("#download-bandwidth-limit").value = "0";\n    resetDownloadConnections();\n    resetMediaInspection();', 1)
p.write_text(js, encoding='utf-8')

# --- Styling ---
p = Path('apps/desktop/ui/styles.css')
css = p.read_text(encoding='utf-8')
if '.download-connections-control' not in css:
    css += '''\n.download-connections-control { margin-top: 12px; padding: 12px 14px; border: 1px solid var(--line, rgba(255,255,255,.12)); border-radius: 10px; }\n.download-connections-control > div { display: flex; align-items: center; justify-content: space-between; gap: 12px; }\n.download-connections-control output { min-width: 2.2em; text-align: center; font-weight: 700; }\n.download-connections-control input[type="range"] { width: 100%; margin: 9px 0 4px; }\n.download-connections-control small { display: block; opacity: .72; }\n'''
p.write_text(css, encoding='utf-8')

# --- Desktop backend: keep the override per task in RequestIdentity ---
p = Path('apps/desktop/src-tauri/src/main.rs')
r = p.read_text(encoding='utf-8')

# DownloadContext receives the optional value from the UI.
ctx_pat = r'(struct DownloadContext \{.*?request_content_type: Option<String>,)(\n\})'
if 'connections_override: Option<usize>' not in re.search(r'struct DownloadContext \{.*?\n\}', r, re.S).group(0):
    r, n = re.subn(ctx_pat, r'\1\n    #[serde(default)]\n    connections_override: Option<usize>,\2', r, count=1, flags=re.S)
    if n != 1: raise SystemExit('DownloadContext patch failed')

# RequestIdentity carries it to the worker for this specific task.
ri_match = re.search(r'struct RequestIdentity \{.*?\n\}', r, re.S)
if not ri_match: raise SystemExit('RequestIdentity not found')
if 'connections_override' not in ri_match.group(0):
    patched = ri_match.group(0).replace('    request_content_type: Option<String>,', '    request_content_type: Option<String>,\n    connections_override: Option<usize>,')
    r = r[:ri_match.start()] + patched + r[ri_match.end():]

# During enqueue, sanitize override and make sure an identity entry is stored even
# when there are no cookies/custom headers.
needle = '''        let request_content_type = context\n            .request_content_type\n            .filter(|value| value.len() <= 256 && !value.contains('\\r') && !value.contains('\\n'));'''
if needle in r and 'let connections_override = context.connections_override' not in r:
    r = r.replace(needle, needle + '\n        let connections_override = context.connections_override.map(|value| value.clamp(1, 32));', 1)

r = r.replace('if cookie_header.is_some() || user_agent.is_some() || request_method == "POST" {', 'if cookie_header.is_some() || user_agent.is_some() || request_method == "POST" || connections_override.is_some() {', 1)

# Add field to the primary identity initializer containing request_content_type.
primary = '''                    RequestIdentity {\n                        cookie_header,\n                        user_agent,\n                        request_method,\n                        request_body,\n                        request_content_type,\n                    },'''
if primary in r:
    r = r.replace(primary, '''                    RequestIdentity {\n                        cookie_header,\n                        user_agent,\n                        request_method,\n                        request_body,\n                        request_content_type,\n                        connections_override,\n                    },''', 1)
else:
    raise SystemExit('primary RequestIdentity initializer not found')

# Any remaining RequestIdentity literals need a neutral value.
def add_none_to_identity(match):
    body = match.group(0)
    if 'connections_override' in body:
        return body
    if 'request_content_type:' in body:
        return body[:-1] + '    connections_override: None,\n}'
    return body
r = re.sub(r'RequestIdentity \{\n.*?\n\s*\}', add_none_to_identity, r, flags=re.S)

# User choice wins over adaptive/global settings. This is the key behavior for
# sites that reject segmented/multi-connection downloads.
old_cfg = '''        let configured_connections =\n            if limits.adaptive_efficiency && limits.max_active_downloads <= 3 && task.priority >= 0\n            {\n                limits.connections_per_download.max(16)\n            } else {\n                limits.connections_per_download\n            };'''
new_cfg = '''        let configured_connections = identity\n            .as_ref()\n            .and_then(|item| item.connections_override)\n            .unwrap_or_else(|| {\n                if limits.adaptive_efficiency && limits.max_active_downloads <= 3 && task.priority >= 0 {\n                    limits.connections_per_download.max(16)\n                } else {\n                    limits.connections_per_download\n                }\n            });'''
if old_cfg not in r:
    raise SystemExit('configured_connections block not found')
r = r.replace(old_cfg, new_cfg, 1)

p.write_text(r, encoding='utf-8')

# Version bump.
p = Path('browser-extension/manifest.json')
manifest = json.loads(p.read_text(encoding='utf-8'))
manifest['version'] = '0.3.58'
p.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

assert 'id="download-connections"' in Path('apps/desktop/ui/index.html').read_text(encoding='utf-8')
assert 'connectionsOverride:' in Path('apps/desktop/ui/app.js').read_text(encoding='utf-8')
assert 'connections_override: Option<usize>' in Path('apps/desktop/src-tauri/src/main.rs').read_text(encoding='utf-8')
print('Applied 0.3.58 per-download thread/connection slider')
