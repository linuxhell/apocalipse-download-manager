from pathlib import Path
import json
import re

# 0.3.57: finish the visible/active Matrix + site-rules cleanup.
p = Path('apps/desktop/src-tauri/src/main.rs')
s = p.read_text(encoding='utf-8')

# Remove matrix-rules.json from the diagnostics ZIP. The old expression escaped
# the dot twice and therefore searched for a literal backslash on Windows CI.
pattern = re.compile(
    r'\n\s*archive\s*\.start_file\("matrix-rules\.json",\s*options\)'
    r'.*?(?=\n\s*archive\s*\.start_file\(|\n\s*archive\s*\.finish\()',
    re.S,
)
s, removed = pattern.subn('\n', s, count=1)
if removed != 1:
    raise SystemExit(f'expected one matrix-rules diagnostics block, removed={removed}')

for name in [
    'get_site_rules','set_site_rules','reset_site_rules','matrix_analyze',
    'matrix_apply_rule','matrix_rollback_rule','import_matrix_rules','export_matrix_rules',
]:
    s = re.sub(rf'^\s*{re.escape(name)},\s*$', '', s, flags=re.M)
p.write_text(s, encoding='utf-8')

# UI commands are removed only as direct calls; the page itself is already replaced
# by Logs in the later UI patch.
p = Path('apps/desktop/ui/app.js')
js = p.read_text(encoding='utf-8')
for command in [
    'get_site_rules','set_site_rules','reset_site_rules','matrix_analyze',
    'matrix_apply_rule','matrix_rollback_rule','import_matrix_rules','export_matrix_rules',
]:
    js = re.sub(rf'^.*invoke\([\'\"]{re.escape(command)}[\'\"].*\n?', '', js, flags=re.M)
p.write_text(js, encoding='utf-8')

p = Path('browser-extension/manifest.json')
manifest = json.loads(p.read_text(encoding='utf-8'))
manifest['version'] = '0.3.57'
p.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

main = Path('apps/desktop/src-tauri/src/main.rs').read_text(encoding='utf-8')
ui = Path('apps/desktop/ui/app.js').read_text(encoding='utf-8')
assert 'start_file("matrix-rules.json"' not in main
for command in [
    'get_site_rules','set_site_rules','reset_site_rules','matrix_analyze',
    'matrix_apply_rule','matrix_rollback_rule','import_matrix_rules','export_matrix_rules',
]:
    assert f'invoke("{command}"' not in ui
    assert f"invoke('{command}'" not in ui
print('Applied 0.3.57 Matrix/site-rules active-surface cleanup')
