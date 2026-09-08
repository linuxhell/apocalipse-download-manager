from pathlib import Path
import re
import json

# 0.3.59: complete the per-download thread override patch by adding a neutral
# connections_override value to DownloadContext literals created outside the UI.
p = Path('apps/desktop/src-tauri/src/main.rs')
s = p.read_text(encoding='utf-8')

# Any DownloadContext initializer that does not explicitly set the new field must
# receive None so legacy bridge/import/clipboard paths continue to compile and
# preserve their previous behavior.
def patch_context(match):
    body = match.group(0)
    if 'connections_override:' in body:
        return body
    if 'request_content_type:' in body:
        close = body.rfind('}')
        return body[:close] + '    connections_override: None,\n' + body[close:]
    return body

s = re.sub(r'DownloadContext \{\n.*?\n\s*\}', patch_context, s, flags=re.S)
p.write_text(s, encoding='utf-8')

# Keep the extension/package version aligned with the compiled test build.
p = Path('browser-extension/manifest.json')
manifest = json.loads(p.read_text(encoding='utf-8'))
manifest['version'] = '0.3.59'
p.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

main = Path('apps/desktop/src-tauri/src/main.rs').read_text(encoding='utf-8')
for block in re.findall(r'DownloadContext \{\n.*?\n\s*\}', main, flags=re.S):
    if 'request_content_type:' in block:
        assert 'connections_override:' in block

print('Applied 0.3.59 DownloadContext compatibility fix')
