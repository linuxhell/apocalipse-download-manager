from pathlib import Path
import json

# 0.3.52: bridge connectivity regression after removing site-rules backend.
# The extension health heartbeat still chained /v1/site-rules after /v1/health.
# That 404 made bridgeRequest() set bridgeConnected=false immediately after a
# successful health check, so the popup always showed disconnected.

p = Path('browser-extension/service-worker.js')
s = p.read_text(encoding='utf-8')

old = '''    bridgeRequest("/v1/health")
      .then(() => bridgeRequest("/v1/site-rules"))
      .then((rules) => { if (Array.isArray(rules)) siteRules = rules; })
      .then(() => flushAssistedDownloads())
      .then(() => flushDirectDownloads())
      .catch(() => {});'''
new = '''    bridgeRequest("/v1/health")
      .then(() => flushAssistedDownloads())
      .then(() => flushDirectDownloads())
      .catch(() => {});'''
if old not in s:
    raise SystemExit('expected stale site-rules heartbeat chain not found')
s = s.replace(old, new, 1)

# The removed remote rules endpoint must never participate in connection state.
assert '/v1/site-rules' not in s, 'stale /v1/site-rules request remains in extension'

p.write_text(s, encoding='utf-8')

p = Path('browser-extension/manifest.json')
manifest = json.loads(p.read_text(encoding='utf-8'))
manifest['version'] = '0.3.52'
p.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

print('Applied 0.3.52 bridge heartbeat correction')
