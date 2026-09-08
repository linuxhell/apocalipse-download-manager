from pathlib import Path
import json

# 0.3.63: the 0.3.62 diagnostic proves desktop health succeeds repeatedly while
# the MV3 worker never answers and workerBootstrapTrace remains empty. The only
# bootstrap boundary before capture registration is importScripts(service-worker.js).
# Remove that runtime boundary by building one self-contained service worker.

bg_path = Path('browser-extension/background.js')
sw_path = Path('browser-extension/service-worker.js')
bg = bg_path.read_text(encoding='utf-8')
sw = sw_path.read_text(encoding='utf-8')

start = bg.find('const APOCALIPSE_WORKER_TRACE_KEY = "workerBootstrapTrace";')
end_marker = 'apocalipseWorkerTrace("background.bootstrap.capture_code.begin");\n'
end = bg.find(end_marker)
if start != 0 or end < 0:
    raise SystemExit('0.3.56 importScripts bootstrap wrapper not found')
end += len(end_marker)

# service-worker.js already owns these helpers; background.js added duplicate
# fallback copies. Remove only the duplicate declarations from the capture layer
# before concatenating the two formerly separate classic scripts.
rest = bg[end:]
for block in [
'''const cancelBrowserDownload = (id) => new Promise((resolve) => {\n  chrome.downloads.cancel(id, () => { void chrome.runtime.lastError; resolve(); });\n});\n''',
'''const eraseBrowserDownload = (id) => new Promise((resolve) => {\n  chrome.downloads.erase({ id }, () => { void chrome.runtime.lastError; resolve(); });\n});\n''',
]:
    if block not in rest:
        raise SystemExit('expected duplicate download helper not found')
    rest = rest.replace(block, '', 1)

# Synchronous marker: unlike the old storage-only marker, this listener exists
# as part of worker registration itself. The popup probe can now prove startup
# without depending on importScripts or a delayed storage callback.
probe = r'''

const APOCALIPSE_WORKER_BUILD = "0.3.63-self-contained";
chrome.runtime.onMessage.addListener((message, sender, reply) => {
  if (message?.type !== "APOCALIPSE_WORKER_DIAGNOSTICS") return;
  reply({
    ok: true,
    version: chrome.runtime.getManifest().version,
    build: APOCALIPSE_WORKER_BUILD,
    trace: [{ phase: "background.self_contained.ready", at: new Date().toISOString() }],
    error: null,
  });
});
'''

# Remove the obsolete diagnostic listener appended by 0.3.56 so there is one
# authoritative response to APOCALIPSE_WORKER_DIAGNOSTICS.
old_tail = '''\n\napocalipseWorkerTrace("background.bootstrap.complete");\nchrome.runtime.onMessage.addListener((message, sender, reply) => {\n  if (message?.type !== "APOCALIPSE_WORKER_DIAGNOSTICS") return;\n  chrome.storage.local.get({ [APOCALIPSE_WORKER_TRACE_KEY]: [] }, (stored) => {\n    const error = chrome.runtime.lastError;\n    reply({\n      ok: !error,\n      version: chrome.runtime.getManifest().version,\n      trace: stored?.[APOCALIPSE_WORKER_TRACE_KEY] || [],\n      error: error?.message || null,\n    });\n  });\n  return true;\n});\n'''
if old_tail not in rest:
    raise SystemExit('obsolete 0.3.56 diagnostics listener not found')
rest = rest.replace(old_tail, '\n', 1)

combined = sw.rstrip() + probe + '\n\n' + rest.lstrip()
if 'importScripts(' in combined:
    raise SystemExit('runtime importScripts remains in generated worker')
if combined.count('APOCALIPSE_WORKER_DIAGNOSTICS') != 1:
    raise SystemExit('worker diagnostics probe is not unique')
if 'background.self_contained.ready' not in combined:
    raise SystemExit('self-contained ready marker missing')
bg_path.write_text(combined, encoding='utf-8')

manifest_path = Path('browser-extension/manifest.json')
manifest = json.loads(manifest_path.read_text(encoding='utf-8'))
manifest['version'] = '0.3.63'
manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print('Applied 0.3.63 self-contained MV3 worker fix')
