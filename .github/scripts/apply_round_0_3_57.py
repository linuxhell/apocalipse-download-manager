from pathlib import Path
import json
import re

# 0.3.57: finish the visible/active Matrix + site-rules cleanup.
# Keep the current shortcut/worker diagnostics work intact, but stop exposing or
# exporting the old Matrix rules machinery. Internal compatibility code may still
# exist temporarily where download behavior depends on it, but it is no longer a
# user-facing feature, diagnostics payload, or Tauri command surface.

p = Path('apps/desktop/src-tauri/src/main.rs')
s = p.read_text(encoding='utf-8')

# Remove matrix-rules.json from the diagnostics ZIP. Bound the removal to the next
# archive entry/finalization so unrelated diagnostics remain untouched.
pattern = re.compile(
    r'\n\s*archive\s*\.start_file\("matrix-rules\\.json",\s*options\)'
    r'.*?(?=\n\s*archive\s*\.start_file\(|\n\s*archive\s*\.finish\()',
    re.S,
)
s, removed = pattern.subn('\n', s, count=1)
if removed != 1:
    raise SystemExit(f'expected one matrix-rules diagnostics block, removed={removed}')

# Remove stale Matrix/site-rules commands from the active Tauri invoke surface.
for name in [
    'get_site_rules',
    'set_site_rules',
    'reset_site_rules',
    'matrix_analyze',
    'matrix_apply_rule',
    'matrix_rollback_rule',
    'import_matrix_rules',
    'export_matrix_rules',
]:
    s = re.sub(rf'^\s*{re.escape(name)},\s*$', '', s, flags=re.M)

p.write_text(s, encoding='utf-8')

# Ensure the removed Matrix UI cannot be resurrected by hidden legacy handlers.
p = Path('apps/desktop/ui/app.js')
js = p.read_text(encoding='utf-8')
for command in [
    'get_site_rules', 'set_site_rules', 'reset_site_rules',
    'matrix_analyze', 'matrix_apply_rule', 'matrix_rollback_rule',
    'import_matrix_rules', 'export_matrix_rules',
]:
    if f'invoke("{command}"' in js or f"invoke('{command}'" in js:
        # Legacy handlers are already guarded/hidden by 0.3.50; remove any direct
        # invocation lines that survived so Logs stays diagnostics-only.
        js = re.sub(rf'^.*invoke\([\'\"]{re.escape(command)}[\'\"].*\n?', '', js, flags=re.M)
p.write_text(js, encoding='utf-8')

# Version bump.
p = Path('browser-extension/manifest.json')
manifest = json.loads(p.read_text(encoding='utf-8'))
manifest['version'] = '0.3.57'
p.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

# Build-time assertions: diagnostics must not contain matrix-rules.json anymore,
# and active UI must not call the retired command surface.
main = Path('apps/desktop/src-tauri/src/main.rs').read_text(encoding='utf-8')
ui = Path('apps/desktop/ui/app.js').read_text(encoding='utf-8')
assert 'start_file("matrix-rules.json"' not in main
for command in [
    'get_site_rules', 'set_site_rules', 'reset_site_rules',
    'matrix_analyze', 'matrix_apply_rule', 'matrix_rollback_rule',
    'import_matrix_rules', 'export_matrix_rules',
]:
    assert f'invoke("{command}"' not in ui
    assert f"invoke('{command}'" not in ui

print('Applied 0.3.57 Matrix/site-rules active-surface cleanup')
