from pathlib import Path
import json

# 0.3.65: adopt a single modifier transaction layer, inspired by the robust
# browser-download bypass gate used by Ghost Downloader, but extended with a
# force lease that survives async page/CDN handoffs. Bypass always wins.

# --- Isolated-world gesture signalling ---
p = Path('browser-extension/content.js')
s = p.read_text(encoding='utf-8')
old = '''  document.addEventListener("pointerdown", (event) => {\n    if (!modifierPressed(event, shortcutKeys.bypass)) return;\n    chrome.runtime.sendMessage({ type: "APOCALIPSE_BYPASS_NEXT", ttlMs: 15000 }).catch(() => {});\n  }, true);'''
new = '''  document.addEventListener("pointerdown", (event) => {\n    const bypass = modifierPressed(event, shortcutKeys.bypass);\n    const force = modifierPressed(event, shortcutKeys.force);\n    if (bypass) {\n      chrome.runtime.sendMessage({ type: "APOCALIPSE_BYPASS_NEXT", ttlMs: 4000 }).catch(() => {});\n    } else if (force) {\n      chrome.runtime.sendMessage({ type: "APOCALIPSE_FORCE_NEXT", ttlMs: 20000 }).catch(() => {});\n    }\n  }, true);'''
if old not in s: raise SystemExit('content pointerdown shortcut block not found')
s = s.replace(old, new, 1)
old = '''    if (bypass) {\n      chrome.runtime.sendMessage({ type: "APOCALIPSE_BYPASS_NEXT", ttlMs: 15000 }).catch(() => {});\n      return;\n    }\n    if ((event.ctrlKey || event.shiftKey || event.altKey) && !force) return;'''
new = '''    if (bypass) {\n      chrome.runtime.sendMessage({ type: "APOCALIPSE_BYPASS_NEXT", ttlMs: 4000 }).catch(() => {});\n      return;\n    }\n    if (force) {\n      // Force is a transaction, not an instruction to steal the visible href.\n      // Let the page run and observe the real downstream file request/download.\n      chrome.runtime.sendMessage({ type: "APOCALIPSE_FORCE_NEXT", ttlMs: 20000 }).catch(() => {});\n      return;\n    }\n    if (event.ctrlKey || event.shiftKey || event.altKey) return;'''
if old not in s: raise SystemExit('content click shortcut block not found')
s = s.replace(old, new, 1)
p.write_text(s, encoding='utf-8')

# --- MAIN-world force/bypass lease ---
p = Path('browser-extension/page-hook.js')
s = p.read_text(encoding='utf-8')
s = s.replace('''  let forceGestureUntil = 0;\n  let activeTraceId = "";''', '''  let forceGestureUntil = 0;\n  let bypassGestureUntil = 0;\n  let activeTraceId = "";''', 1)
s = s.replace('''  const bypassPressed = () => held.has(shortcuts.bypass || "Alt");\n  const forcePressed = () => held.has(shortcuts.force || "Shift");\n  const forceActive = () => forcePressed() || Date.now() < forceGestureUntil;''', '''  const bypassPressed = () => held.has(shortcuts.bypass || "Alt");\n  const bypassActive = () => bypassPressed() || Date.now() < bypassGestureUntil;\n  const forcePressed = () => held.has(shortcuts.force || "Shift");\n  const forceActive = () => !bypassActive() && (forcePressed() || Date.now() < forceGestureUntil);''', 1)
s = s.replace('mode: bypassPressed() ? "bypass" : (forceActive() ? "force" : "normal")', 'mode: bypassActive() ? "bypass" : (forceActive() ? "force" : "normal")')
s = s.replace('if (!forceActive() || bypassPressed() || !anchor?.href)', 'if (!forceActive() || bypassActive() || !anchor?.href)')
s = s.replace('if (!candidate || bypassPressed()) { if (candidate && bypassPressed())', 'if (!candidate || bypassActive()) { if (candidate && bypassActive())')
s = s.replace('const forcedAtCall = forceActive() && !bypassPressed();', 'const forcedAtCall = forceActive() && !bypassActive();')
s = s.replace('if (!candidate || bypassPressed()) return;', 'if (!candidate || bypassActive()) return;')
s = s.replace('if (forcePressed()) forceGestureUntil = Date.now() + 15000;', 'if (bypassPressed()) bypassGestureUntil = Date.now() + 4000;\n    else if (forcePressed()) forceGestureUntil = Date.now() + 20000;', 1)
s = s.replace('trace(forcePressed() ? "FORCE_ARMED" : (bypassPressed() ? "BYPASS_ARMED" : "AUTO_GESTURE")', 'trace(forcePressed() ? "FORCE_ARMED" : (bypassActive() ? "BYPASS_ARMED" : "AUTO_GESTURE")', 1)
old = '''    const candidate = classify(value);\n    if (emit(candidate, "window.fetch")) return Promise.resolve(new Response("", { status: 204, statusText: "Handled by Apocalipse" }));\n    const forcedAtCall = forceActive() && !bypassActive();'''
new = '''    const candidate = classify(value);\n    if (candidate?.kind === "chatgpt-library") {\n      trace(forceActive() ? "FORCE_PASSTHROUGH" : "AUTO_PASSTHROUGH", { primitive: "window.fetch", kind: candidate.kind, url: safeUrl(candidate.url) });\n    } else if (emit(candidate, "window.fetch")) {\n      return Promise.resolve(new Response("", { status: 204, statusText: "Handled by Apocalipse" }));\n    }\n    const forcedAtCall = forceActive() && !bypassActive();'''
if old not in s: raise SystemExit('page-hook fetch takeover block not found')
s = s.replace(old, new, 1)
p.write_text(s, encoding='utf-8')

# --- Central worker gate ---
p = Path('browser-extension/background.js')
s = p.read_text(encoding='utf-8')
anchor = 'const CAPTURE_TTL_MS = 20000;\n'
insert = '''const CAPTURE_TTL_MS = 20000;\n\n// Central modifier transaction state. Bypass always wins. Force survives the\n// initiating click long enough for async pages/CDNs to create the real download.\nlet forceUntil = 0;\nlet forceTabId = null;\nlet bypassTabId = null;\n\nfunction armBypass(tabId, ttlMs = 4000) {\n  bypassUntil = Math.max(bypassUntil || 0, Date.now() + Math.max(500, Math.min(Number(ttlMs) || 4000, 30000)));\n  bypassTabId = Number.isInteger(tabId) ? tabId : null;\n  forceUntil = 0;\n  forceTabId = null;\n}\nfunction armForce(tabId, ttlMs = 20000) {\n  if (bypassHeld || Date.now() < (bypassUntil || 0)) return;\n  forceUntil = Math.max(forceUntil, Date.now() + Math.max(1000, Math.min(Number(ttlMs) || 20000, 30000)));\n  forceTabId = Number.isInteger(tabId) ? tabId : null;\n}\nfunction sameLeaseTab(leaseTabId, tabId) {\n  return leaseTabId == null || tabId == null || leaseTabId === tabId;\n}\nfunction bypassIsActive(tabId = null) {\n  try {\n    const now = Date.now();\n    return Boolean(bypassHeld)\n      || (now < Number(bypassUntil || 0) && sameLeaseTab(bypassTabId, tabId))\n      || now < Number(bypassNextUntil || 0);\n  } catch { return false; }\n}\nfunction forceIsActive(tabId = null) {\n  try {\n    if (bypassIsActive(tabId)) return false;\n    return Boolean(forceHeld) || (Date.now() < forceUntil && sameLeaseTab(forceTabId, tabId));\n  } catch { return false; }\n}\n'''
if anchor not in s: raise SystemExit('capture TTL anchor not found')
s = s.replace(anchor, insert, 1)

# Remove the older one-shot bypass gate; it consumed the lease in whichever
# interception layer happened to check first, allowing a later route to steal it.
start = s.find('function bypassIsActive() {', s.find('async function streamCapturedUrl'))
end = s.find('\n\n\n// Fallback only.', start)
if start < 0 or end < 0: raise SystemExit('old bypass gate not found')
s = s[:start] + s[end + 2:]

# 0.3.60 removed the old dedicated stream/CDP implementations, but this stale
# early return still cancelled Library/Rapidgator before the generic handoff.
old = '''async function takeBrowserDownload(item, eraseFromHistory = false) {\n  let url = item.finalUrl || item.url;\n  if (!item.id) return false;\n  // These are already owned by the dedicated authenticated streaming/CDP paths.\n  // Cancel right here, inside filename determination, before Chrome can show its\n  // own Save As window over the Apocalipse destination dialog.\n  if (isChatGPTLibraryDownload(url) || (bridgeConnected && isRapidgatorFinalDownload(url))) {\n    await cancelBrowserDownload(item.id).catch(() => {});\n    await eraseBrowserDownload(item.id);\n    return true;\n  }'''
new = '''async function takeBrowserDownload(item, eraseFromHistory = false) {\n  let url = item.finalUrl || item.url;\n  if (!item.id) return false;\n  const modifierTabId = Number.isInteger(item.tabId) ? item.tabId : null;\n  if (bypassIsActive(modifierTabId)) return false;'''
if old not in s: raise SystemExit('stale Library/Rapidgator early-cancel block not found')
s = s.replace(old, new, 1)
s = s.replace('''if (Date.now() < bypassNextUntil) {\n      bypassNextUntil = 0;\n      return false;\n    }\n    if (!bridgeConnected || bypassHeld || Date.now() < bypassUntil) return false;''', '''if (bypassIsActive(modifierTabId)) return false;\n    if (!bridgeConnected) return false;''', 1)
s = s.replace('''  if (Date.now() < bypassNextUntil) {\n    bypassNextUntil = 0;\n    return false;\n  }\n  if (!bridgeConnected || bypassHeld || Date.now() < bypassUntil) return false;''', '''  if (bypassIsActive(modifierTabId)) return false;\n  if (!bridgeConnected) return false;''', 1)
s = s.replace('const forced = (() => { try { return forceHeld || Date.now() < forceUntil; } catch { return false; } })();', 'const forced = forceIsActive(Number.isInteger(item?.tabId) ? item.tabId : null);', 1)
s = s.replace('if (!recognized || bypassIsActive()) return;', 'if (!recognized || bypassIsActive(Number.isInteger(item?.tabId) ? item.tabId : null)) return;', 1)

needle = '''chrome.runtime.onMessage.addListener((message, sender, reply) => {\n  if (message?.type === "APOCALIPSE_CAPTURE_TRACE") {'''
repl = '''chrome.runtime.onMessage.addListener((message, sender, reply) => {\n  const shortcutTabId = Number.isInteger(sender.tab?.id) ? sender.tab.id : null;\n  if (message?.type === "APOCALIPSE_SHORTCUT_STATE") {\n    bypassHeld = Boolean(message.bypassPressed);\n    forceHeld = Boolean(message.forcePressed) && !bypassHeld;\n    if (bypassHeld) armBypass(shortcutTabId, 4000);\n    else if (forceHeld) armForce(shortcutTabId, 20000);\n    reply({ ok: true, mode: bypassHeld ? "bypass" : (forceHeld ? "force" : "normal") });\n    return;\n  }\n  if (message?.type === "APOCALIPSE_BYPASS_NEXT") {\n    armBypass(shortcutTabId, message.ttlMs);\n    reply({ ok: true, mode: "bypass" });\n    return;\n  }\n  if (message?.type === "APOCALIPSE_FORCE_NEXT") {\n    armForce(shortcutTabId, message.ttlMs);\n    reply({ ok: true, mode: "force" });\n    return;\n  }\n  if (message?.type === "APOCALIPSE_CAPTURE_TRACE") {'''
if needle not in s: raise SystemExit('capture message listener anchor not found')
s = s.replace(needle, repl, 1)
s = s.replace('''  if (message?.type !== "APOCALIPSE_PRE_DOWNLOAD_URL") return;\n  if (bypassIsActive()) {''', '''  if (message?.type !== "APOCALIPSE_PRE_DOWNLOAD_URL") return;\n  if (bypassIsActive(shortcutTabId)) {''', 1)
s = s.replace('force: Boolean(message.force),', 'force: Boolean(message.force) || forceIsActive(shortcutTabId),', 1)
p.write_text(s, encoding='utf-8')

manifest_path = Path('browser-extension/manifest.json')
manifest = json.loads(manifest_path.read_text(encoding='utf-8'))
manifest['version'] = '0.3.65'
manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

# Build-time invariants.
assert 'APOCALIPSE_FORCE_NEXT' in Path('browser-extension/content.js').read_text(encoding='utf-8')
bg = Path('browser-extension/background.js').read_text(encoding='utf-8')
assert 'function forceIsActive' in bg
assert 'stale Library/Rapidgator early-cancel block' not in bg
assert 'isChatGPTLibraryDownload(url) || (bridgeConnected && isRapidgatorFinalDownload(url))' not in bg
print('Applied 0.3.65 centralized Alt bypass + Shift force transaction')
