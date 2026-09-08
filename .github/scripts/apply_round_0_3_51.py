from pathlib import Path
import json

# 0.3.51: focused UI correction from 0.3.50 screenshots.
# - Metrics belong to download/task pages only, never Themes/Language/Logs.
# - Settings must not depend on controls moved out to Themes/Logs.

p = Path('apps/desktop/ui/app.js')
s = p.read_text(encoding='utf-8')

# Make utility-page metric hiding deterministic even if the older navigation
# replacement did not match exactly in a generated test build.
old = '    document.querySelector(".metrics").hidden = ["link", "matrix"].includes(activePage);'
new = '    document.querySelector(".metrics").hidden = ["link", "matrix", "themes", "language"].includes(activePage);'
if old in s:
    s = s.replace(old, new, 1)
old2 = '    document.querySelector(".panel").hidden = ["link", "matrix"].includes(activePage);'
new2 = '    document.querySelector(".panel").hidden = ["link", "matrix", "themes", "language"].includes(activePage);'
if old2 in s:
    s = s.replace(old2, new2, 1)

# Settings no longer owns appearance or diagnostics. Do not request/populate
# the removed log-editor control; a null element here made the Settings click
# fall into its catch block before settingsDialog.showModal().
s = s.replace('autostart, directory, clipboard, limits, pairing, userAgent, logEditor, proxy, dns, associations, websiteCredentials',
              'autostart, directory, clipboard, limits, pairing, userAgent, proxy, dns, associations, websiteCredentials', 1)
s = s.replace('      invoke("get_log_editor"),\n', '', 1)
s = s.replace('    document.querySelector("#log-editor").value = logEditor;\n', '', 1)
s = s.replace('    updateLogEditorControls();\n    settingsDialog.showModal();', '    settingsDialog.showModal();', 1)

# Defensive compatibility: if any old code still calls this helper after the
# diagnostics controls were moved, it must be harmless.
s = s.replace('function updateLogEditorControls() {\n  const configured = Boolean(document.querySelector("#log-editor").value.trim());\n  document.querySelector("#remove-log-editor").disabled = !configured;\n  document.querySelector("#open-log-external").disabled = !configured;\n}',
'''function updateLogEditorControls() {
  const editor = document.querySelector("#log-editor");
  const remove = document.querySelector("#remove-log-editor");
  const open = document.querySelector("#open-log-external");
  if (!editor || !remove || !open) return;
  const configured = Boolean(editor.value.trim());
  remove.disabled = !configured;
  open.disabled = !configured;
}''', 1)

p.write_text(s, encoding='utf-8')

# CSS guard: utility pages explicitly suppress the global metrics strip.
p = Path('apps/desktop/ui/styles.css')
css = p.read_text(encoding='utf-8')
if '.metrics[hidden]' not in css:
    css += '\n/* 0.3.51 utility pages: hidden global task metrics must never occupy space. */\n.metrics[hidden] { display: none !important; }\n'
p.write_text(css, encoding='utf-8')

# Bump extension/package test version.
p = Path('browser-extension/manifest.json')
manifest = json.loads(p.read_text(encoding='utf-8'))
manifest['version'] = '0.3.51'
p.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

# Build-time assertions for the exact regressions fixed here.
assert 'invoke("get_log_editor"),' not in s, 'Settings still requests removed log editor control'
assert 'document.querySelector("#log-editor").value = logEditor;' not in s, 'Settings still populates removed log editor control'
assert '["link", "matrix", "themes", "language"].includes(activePage)' in s, 'Utility pages do not hide global panels'
print('Applied 0.3.51 Settings + utility-page layout correction')
