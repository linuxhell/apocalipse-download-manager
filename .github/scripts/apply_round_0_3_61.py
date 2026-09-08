from pathlib import Path
import json

# 0.3.61: remove the complete obsolete Matrix block atomically.
# 0.3.60 started replacement inside refreshDiagnosticLog but its end marker sat
# inside a legacy handler, leaving a dangling catch/finally and invalid app.js.
p = Path('apps/desktop/ui/app.js')
s = p.read_text(encoding='utf-8')

start = s.find('async function refreshMatrix() {')
end = s.find('document.querySelector("#download-list").addEventListener("pointerdown"', start)
if start < 0 or end <= start:
    raise SystemExit(f'complete legacy Matrix block not found: start={start} end={end}')

# Preserve only the two requested Logs actions. Everything from refreshMatrix
# through the old log-editor helper is legacy and is removed as one syntactic unit.
logs = '''const exportDiagnosticsButton = document.querySelector("#export-diagnostics");
if (exportDiagnosticsButton) exportDiagnosticsButton.onclick = async (event) => {
  const button = event.currentTarget;
  button.disabled = true;
  try {
    const path = await invoke("export_diagnostics");
    window.alert(`${t("diagnosticsExported")}: ${path}`);
  } catch (error) {
    console.error(error);
    window.alert(String(error));
  } finally {
    button.disabled = false;
  }
};
const clearInternalLogsButton = document.querySelector("#clear-internal-logs");
if (clearInternalLogsButton) clearInternalLogsButton.onclick = async (event) => {
  const button = event.currentTarget;
  button.disabled = true;
  try {
    await invoke("clear_general_log");
  } catch (error) {
    console.error(error);
  } finally {
    button.disabled = false;
  }
};
'''
s = s[:start] + logs + s[end:]
p.write_text(s, encoding='utf-8')

p = Path('browser-extension/manifest.json')
manifest = json.loads(p.read_text(encoding='utf-8'))
manifest['version'] = '0.3.61'
p.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

app = Path('apps/desktop/ui/app.js').read_text(encoding='utf-8')
for stale in [
    'refreshMatrix', 'matrix_analyze', 'matrix_apply_rule', 'matrix_rollback_rule',
    'import_matrix_rules', 'export_matrix_rules', '#matrix-scan', '#matrix-import',
    '#matrix-export', '#log-editor', '#remove-log-editor', '#open-log-external',
]:
    assert stale not in app, stale
assert 'exportDiagnosticsButton' in app
assert 'clearInternalLogsButton' in app
print('Applied 0.3.61 atomic Matrix/log-editor UI cleanup')
