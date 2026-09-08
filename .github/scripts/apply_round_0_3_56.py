from pathlib import Path
import json

# 0.3.56: evidence-only MV3 worker bootstrap instrumentation.
# The popup reaches the desktop bridge in 0.3.55, but worker status does not answer.
# Persist the exact bootstrap phase/error before changing capture behavior.

p = Path('browser-extension/background.js')
s = p.read_text(encoding='utf-8')
old = 'importScripts("service-worker.js");\n'
new = r'''const APOCALIPSE_WORKER_TRACE_KEY = "workerBootstrapTrace";
const apocalipseWorkerTrace = (phase, extra = {}) => {
  try {
    const entry = {
      at: new Date().toISOString(),
      phase,
      version: chrome.runtime.getManifest().version,
      ...extra,
    };
    chrome.storage.local.get({ [APOCALIPSE_WORKER_TRACE_KEY]: [] }, (stored) => {
      const error = chrome.runtime.lastError;
      if (error) return;
      const items = Array.isArray(stored[APOCALIPSE_WORKER_TRACE_KEY]) ? stored[APOCALIPSE_WORKER_TRACE_KEY] : [];
      items.push(entry);
      chrome.storage.local.set({ [APOCALIPSE_WORKER_TRACE_KEY]: items.slice(-80) });
    });
  } catch {}
};

apocalipseWorkerTrace("background.bootstrap.begin");
try {
  importScripts("service-worker.js");
  apocalipseWorkerTrace("background.importScripts.ok");
} catch (error) {
  apocalipseWorkerTrace("background.importScripts.error", {
    error: String(error),
    stack: String(error?.stack || "").slice(0, 4000),
  });
  throw error;
}
apocalipseWorkerTrace("background.bootstrap.capture_code.begin");
'''
if old not in s:
    raise SystemExit('background importScripts anchor not found')
s = s.replace(old, new, 1)

# Mark that background.js reached the end and expose a local-only probe. This probe
# deliberately does not touch the desktop bridge, so it isolates worker health.
s += r'''

apocalipseWorkerTrace("background.bootstrap.complete");
chrome.runtime.onMessage.addListener((message, sender, reply) => {
  if (message?.type !== "APOCALIPSE_WORKER_DIAGNOSTICS") return;
  chrome.storage.local.get({ [APOCALIPSE_WORKER_TRACE_KEY]: [] }, (stored) => {
    const error = chrome.runtime.lastError;
    reply({
      ok: !error,
      version: chrome.runtime.getManifest().version,
      trace: stored?.[APOCALIPSE_WORKER_TRACE_KEY] || [],
      error: error?.message || null,
    });
  });
  return true;
});
'''
p.write_text(s, encoding='utf-8')

# Popup: use the worker-only ping first. If it fails, include the persisted last
# bootstrap phase in the warning so the next screenshot/diagnostic identifies the
# exact boundary without another speculative patch.
p = Path('browser-extension/popup.js')
popup = p.read_text(encoding='utf-8')
anchor = '''const showWorkerWarning = () => {\n'''
helper = r'''const workerSelfTest = (timeoutMs = 1800) => new Promise((resolve) => {
  let settled = false;
  const finish = (value) => { if (settled) return; settled = true; resolve(value); };
  const timer = setTimeout(() => finish({ ok: false, error: "service_worker_timeout" }), timeoutMs);
  try {
    chrome.runtime.sendMessage({ type: "APOCALIPSE_WORKER_DIAGNOSTICS" }, (status) => {
      clearTimeout(timer);
      const error = chrome.runtime.lastError;
      if (error) finish({ ok: false, error: error.message || "service_worker_unavailable" });
      else finish(status || { ok: false, error: "service_worker_no_response" });
    });
  } catch (error) {
    clearTimeout(timer);
    finish({ ok: false, error: String(error) });
  }
});

const lastWorkerBootstrap = async () => {
  const stored = await chrome.storage.local.get({ workerBootstrapTrace: [] });
  const trace = Array.isArray(stored.workerBootstrapTrace) ? stored.workerBootstrapTrace : [];
  return trace.at(-1) || null;
};

const showWorkerWarning = async (worker = null) => {
'''
if anchor not in popup:
    raise SystemExit('showWorkerWarning anchor not found')
popup = popup.replace(anchor, helper, 1)
# Replace the old synchronous warning body through its closing marker before showBridgeError.
start = popup.index('const showWorkerWarning = async (worker = null) => {')
end = popup.index('\n\nconst showBridgeError', start)
warning = r'''const showWorkerWarning = async (worker = null) => {
  const label = document.querySelector("#bridge-label");
  const last = await lastWorkerBootstrap();
  const phase = last?.phase || "sem_rastro_de_bootstrap";
  const reason = worker?.error || last?.error || "worker_sem_resposta";
  label.removeAttribute("data-i18n");
  if (locale === "pt_BR") label.textContent = `Desktop conectado; motor de captura indisponível (${phase}: ${reason}).`;
  else if (locale === "zh_CN") label.textContent = `桌面端已连接；捕获引擎不可用 (${phase}: ${reason}).`;
  else label.textContent = `Desktop connected; capture engine unavailable (${phase}: ${reason}).`;
};'''
popup = popup[:start] + warning + popup[end:]

# 0.3.55 used bridge status as a proxy for worker health. Replace only those
# post-health checks with the local self-test; direct desktop health stays intact.
popup = popup.replace('const worker = await workerBridgeStatus();\n    if (!worker?.connected) showWorkerWarning();',
                      'const worker = await workerSelfTest();\n    if (!worker?.ok) await showWorkerWarning(worker);')
popup = popup.replace('const worker = await workerBridgeStatus();\n    if (!worker?.connected) {',
                      'const worker = await workerSelfTest();\n    if (!worker?.ok) {')
popup = popup.replace('      showWorkerWarning();\n', '      await showWorkerWarning(worker);\n')
p.write_text(popup, encoding='utf-8')

p = Path('browser-extension/manifest.json')
manifest = json.loads(p.read_text(encoding='utf-8'))
manifest['version'] = '0.3.56'
p.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

assert 'background.importScripts.error' in s
assert 'background.bootstrap.complete' in s
assert 'APOCALIPSE_WORKER_DIAGNOSTICS' in s
assert 'workerSelfTest' in popup
assert 'motor de captura indisponível' in popup
print('Applied 0.3.56 persistent MV3 worker bootstrap instrumentation')
