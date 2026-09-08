from pathlib import Path
import json

# 0.3.62: 0.3.60 already inserts guarded Logs handlers later in app.js.
# 0.3.61 inserted another pair while atomically removing Matrix, producing a
# duplicate const declaration. Keep exactly one complete pair.
p = Path('apps/desktop/ui/app.js')
s = p.read_text(encoding='utf-8')
marker = 'const exportDiagnosticsButton = document.querySelector("#export-diagnostics");'
starts = []
pos = 0
while True:
    i = s.find(marker, pos)
    if i < 0:
        break
    starts.append(i)
    pos = i + len(marker)
if len(starts) != 2:
    raise SystemExit(f'expected exactly 2 Logs handler blocks after 0.3.61, found {len(starts)}')

# Remove the first duplicate as one unit, ending immediately before download-list
# wiring. The second block from 0.3.60 remains as the single canonical handler.
end_marker = 'document.querySelector("#download-list").addEventListener("pointerdown"'
end = s.find(end_marker, starts[0])
if end < 0 or end > starts[1]:
    raise SystemExit('could not bound first duplicate Logs block safely')
s = s[:starts[0]] + s[end:]

if s.count(marker) != 1:
    raise SystemExit(f'Logs handler dedupe failed: {s.count(marker)} export handlers remain')
clear_marker = 'const clearInternalLogsButton = document.querySelector("#clear-internal-logs");'
if s.count(clear_marker) != 1:
    raise SystemExit(f'Logs handler dedupe failed: {s.count(clear_marker)} clear handlers remain')
p.write_text(s, encoding='utf-8')

p = Path('browser-extension/manifest.json')
manifest = json.loads(p.read_text(encoding='utf-8'))
manifest['version'] = '0.3.62'
p.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print('Applied 0.3.62 unique Logs handler fix')
