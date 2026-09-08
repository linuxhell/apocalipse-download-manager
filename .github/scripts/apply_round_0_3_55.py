from pathlib import Path
import json

# 0.3.55: make pairing independent from the MV3 service worker.
# The 0.3.54 desktop listener is alive but receives no request at all, which
# means the popup -> runtime message -> worker path can fail before fetch().
# Pair and health-check directly from the extension popup, then persist the
# token so the worker can use it for all download operations.

p = Path('browser-extension/popup.js')
s = p.read_text(encoding='utf-8')

# Insert direct bridge helpers once.
anchor = '''const showBridgeError = (error) => {\n'''
helper = '''const POPUP_BRIDGE = "http://127.0.0.1:17654";\nconst directBridgeHealth = async (token) => {\n  const value = String(token || "").trim();\n  if (!value) throw new Error("not_paired");\n  const controller = new AbortController();\n  const timeout = setTimeout(() => controller.abort(), 4000);\n  try {\n    const response = await fetch(`${POPUP_BRIDGE}/v1/health`, {\n      method: "GET",\n      headers: { "Authorization": `Bearer ${value}` },\n      cache: "no-store",\n      signal: controller.signal,\n    });\n    if (!response.ok) throw new Error(`bridge_http_${response.status}`);\n    const payload = await response.json().catch(() => ({}));\n    if (payload?.ok === false) throw new Error("bridge_health_failed");\n    return true;\n  } finally { clearTimeout(timeout); }\n};\nconst workerBridgeStatus = (timeoutMs = 1800) => new Promise((resolve) => {\n  let settled = false;\n  const finish = (value) => { if (settled) return; settled = true; resolve(value); };\n  const timer = setTimeout(() => finish({ connected: false, error: "service_worker_timeout" }), timeoutMs);\n  try {\n    chrome.runtime.sendMessage({ type: "APOCALIPSE_BRIDGE_STATUS" }, (status) => {\n      clearTimeout(timer);\n      const error = chrome.runtime.lastError;\n      if (error) finish({ connected: false, error: error.message || "service_worker_unavailable" });\n      else finish(status || { connected: false, error: "service_worker_no_response" });\n    });\n  } catch (error) {\n    clearTimeout(timer);\n    finish({ connected: false, error: String(error) });\n  }\n});\nconst showWorkerWarning = () => {\n  const label = document.querySelector("#bridge-label");\n  label.removeAttribute("data-i18n");\n  if (locale === "pt_BR") label.textContent = "Desktop conectado; reinicie a extensão para ativar o motor de captura.";\n  else if (locale === "zh_CN") label.textContent = "桌面端已连接；请重新加载扩展以启用捕获引擎。";\n  else label.textContent = "Desktop connected; reload the extension to activate the capture engine.";\n};\n\nconst showBridgeError = (error) => {\n'''
if 'const POPUP_BRIDGE =' not in s:
    if anchor not in s:
        raise SystemExit('popup error helper anchor not found')
    s = s.replace(anchor, helper, 1)

old_initial = '''chrome.storage.local.get({ pairingToken: "" }, ({ pairingToken }) => {\n  document.querySelector("#pairing-token").value = pairingToken;\n  chrome.runtime.sendMessage({ type: "APOCALIPSE_BRIDGE_STATUS" }, (status) => {\n    const connected = Boolean(status?.connected);\n    setBridgeStatus(connected);\n    if (!connected && status?.error) showBridgeError(status.error);\n  });\n});'''
new_initial = '''chrome.storage.local.get({ pairingToken: "" }, async ({ pairingToken }) => {\n  document.querySelector("#pairing-token").value = pairingToken;\n  if (!pairingToken) { setBridgeStatus(false); return; }\n  try {\n    await directBridgeHealth(pairingToken);\n    setBridgeStatus(true);\n    const worker = await workerBridgeStatus();\n    if (!worker?.connected) showWorkerWarning();\n  } catch (error) {\n    setBridgeStatus(false);\n    showBridgeError(String(error));\n  }\n});'''
if old_initial not in s:
    raise SystemExit('popup initial status block not found')
s = s.replace(old_initial, new_initial, 1)

old_interval = '''setInterval(() => {\n  chrome.runtime.sendMessage({ type: "APOCALIPSE_BRIDGE_STATUS" }, (status) => {\n    if (chrome.runtime.lastError) return;\n    const connected = Boolean(status?.connected);\n    setBridgeStatus(connected);\n    if (!connected && status?.error) showBridgeError(status.error);\n  });\n}, 5000);'''
new_interval = '''setInterval(async () => {\n  const { pairingToken = "" } = await chrome.storage.local.get({ pairingToken: "" });\n  if (!pairingToken) { setBridgeStatus(false); return; }\n  try {\n    await directBridgeHealth(pairingToken);\n    setBridgeStatus(true);\n  } catch (error) {\n    setBridgeStatus(false);\n    showBridgeError(String(error));\n  }\n}, 5000);'''
if old_interval not in s:
    raise SystemExit('popup status interval block not found')
s = s.replace(old_interval, new_interval, 1)

old_connect = '''document.querySelector("#connect").onclick = () => {\n  const token = document.querySelector("#pairing-token").value;\n  const button = document.querySelector("#connect");\n  button.disabled = true;\n  chrome.runtime.sendMessage({ type: "APOCALIPSE_PAIR", token }, (status) => {\n    button.disabled = false;\n    setBridgeStatus(Boolean(status?.connected));\n    if (!status?.connected) showBridgeError(status?.error || chrome.runtime.lastError?.message || "unavailable");\n  });\n};'''
new_connect = '''document.querySelector("#connect").onclick = async () => {\n  const token = document.querySelector("#pairing-token").value.trim();\n  const button = document.querySelector("#connect");\n  button.disabled = true;\n  try {\n    await directBridgeHealth(token);\n    await chrome.storage.local.set({ pairingToken: token });\n    setBridgeStatus(true);\n    // Wake/synchronize the worker, but never make the button depend on it.\n    const worker = await workerBridgeStatus();\n    if (!worker?.connected) {\n      try { chrome.runtime.sendMessage({ type: "APOCALIPSE_PAIR", token }, () => void chrome.runtime.lastError); } catch {}\n      showWorkerWarning();\n    }\n  } catch (error) {\n    setBridgeStatus(false);\n    showBridgeError(String(error));\n  } finally {\n    button.disabled = false;\n  }\n};'''
if old_connect not in s:
    raise SystemExit('popup connect handler not found')
s = s.replace(old_connect, new_connect, 1)

p.write_text(s, encoding='utf-8')

# Worker: expose a lightweight self-test message that does not touch the bridge.
p = Path('browser-extension/service-worker.js')
w = p.read_text(encoding='utf-8')
message_anchor = '''chrome.runtime.onMessage.addListener((message, sender, reply) => {\n'''
message_replacement = '''chrome.runtime.onMessage.addListener((message, sender, reply) => {\n  if (message?.type === "APOCALIPSE_WORKER_PING") {\n    reply({ ok: true, version: chrome.runtime.getManifest().version });\n    return;\n  }\n'''
if 'APOCALIPSE_WORKER_PING' not in w:
    if message_anchor not in w:
        raise SystemExit('worker message listener anchor not found')
    w = w.replace(message_anchor, message_replacement, 1)
p.write_text(w, encoding='utf-8')

p = Path('browser-extension/manifest.json')
manifest = json.loads(p.read_text(encoding='utf-8'))
manifest['version'] = '0.3.55'
p.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

assert 'directBridgeHealth' in s
assert 'document.querySelector("#connect").onclick = async' in s
assert 'APOCALIPSE_WORKER_PING' in w
print('Applied 0.3.55 direct popup pairing + worker-independent health check')
