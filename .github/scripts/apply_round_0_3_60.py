from pathlib import Path
import json

# 0.3.60: removed Logs/Matrix controls must not leave null .onclick assignments.
# Those runtime exceptions stopped app.js before the sidebar/page handlers were
# fully registered, making every session look dead even though the Rust build passed.

p = Path('apps/desktop/ui/app.js')
s = p.read_text(encoding='utf-8')

# Remove the legacy diagnostic viewer/editor handlers. The Logs page now exposes
# only Export diagnostics and Clear internal logs, as requested.
start = s.find('async function refreshDiagnosticLog() {')
end = s.find('document.querySelectorAll("[data-tool-pick]")', start)
if start >= 0 and end > start:
    keep = '''document.querySelector("#export-diagnostics").onclick = async (event) => {\n  const button = event.currentTarget; button.disabled = true;\n  try { const path = await invoke("export_diagnostics"); window.alert(`${t("diagnosticsExported")}: ${path}`); } catch (error) { console.error(error); window.alert(String(error)); } finally { button.disabled = false; }\n};\ndocument.querySelector("#clear-internal-logs").onclick = async (event) => {\n  const button = event.currentTarget; button.disabled = true;\n  try { await invoke("clear_general_log"); } catch (error) { console.error(error); } finally { button.disabled = false; }\n};\n'''
    s = s[:start] + keep + s[end:]
else:
    raise SystemExit('legacy diagnostics/site-rules handler block not found')

# Defensive guard for the two surviving Logs buttons in case an older layout is
# ever packaged accidentally. A missing optional control must never abort app init.
s = s.replace('document.querySelector("#export-diagnostics").onclick = async (event) => {',
'''const exportDiagnosticsButton = document.querySelector("#export-diagnostics");
if (exportDiagnosticsButton) exportDiagnosticsButton.onclick = async (event) => {''', 1)
s = s.replace('document.querySelector("#clear-internal-logs").onclick = async (event) => {',
'''const clearInternalLogsButton = document.querySelector("#clear-internal-logs");
if (clearInternalLogsButton) clearInternalLogsButton.onclick = async (event) => {''', 1)

p.write_text(s, encoding='utf-8')

# Bump package/extension test version.
p = Path('browser-extension/manifest.json')
manifest = json.loads(p.read_text(encoding='utf-8'))
manifest['version'] = '0.3.60'
p.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

# Build-time regression checks: obsolete controls must not be referenced by direct
# event wiring anymore, because they are intentionally absent from the UI.
app = Path('apps/desktop/ui/app.js').read_text(encoding='utf-8')
for stale in [
    '#manage-site-rules', '#save-site-rules', '#reset-site-rules',
    '#pick-log-editor', '#remove-log-editor', '#open-log-external',
    '#open-log', '#clear-log', '#refresh-log',
]:
    assert f'document.querySelector("{stale}").onclick' not in app, stale
assert 'exportDiagnosticsButton' in app
assert 'clearInternalLogsButton' in app
print('Applied 0.3.60 UI navigation runtime crash fix')
