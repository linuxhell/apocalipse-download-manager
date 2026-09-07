from pathlib import Path

p = Path('browser-extension/content.js')
s = p.read_text(encoding='utf-8')
anchor = '  const copyTextNow = (value) => {'
direct = '''  // Intercept ChatGPT Library links before Chrome creates its own download dialog.
  document.addEventListener("click", (event) => {
    if (event.defaultPrevented || event.button !== 0) return;
    const anchor = event.target.closest?.('a[href*="/backend-api/estuary/content"]');
    if (!anchor) return;
    let url = null;
    try {
      const parsed = new URL(anchor.href, location.href);
      if (parsed.hostname.toLowerCase() === "chatgpt.com" && parsed.pathname === "/backend-api/estuary/content") url = parsed.href;
    } catch {}
    if (!url) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    chrome.runtime.sendMessage({ type: "APOCALIPSE_CHATGPT_LIBRARY_DIRECT", url, pageUrl: location.href, fileName: anchor.getAttribute("download") || "" }).catch(() => {});
  }, true);

'''
if 'APOCALIPSE_CHATGPT_LIBRARY_DIRECT' not in s:
    if anchor not in s: raise SystemExit('content anchor missing')
    s = s.replace(anchor, direct + anchor, 1)
old = '''  const revealFacebookUrl = async (element) => {
    const immediate = facebookUrlFor(element);
    if (immediate) return immediate;
    if (!/(^|\\.)facebook\\.com$/i.test(location.hostname)) return null;
    const postText = element.closest?.('[role="article"],article')?.textContent || "";
    if (/(?:patrocinado|sponsored)/i.test(postText)) {
      const menuUrl = await facebookUrlFromMenu(element);
      if (menuUrl) return menuUrl;
    }
'''
new = '''  const revealFacebookUrl = async (element) => {
    if (!/(^|\\.)facebook\\.com$/i.test(location.hostname)) return null;
    const postText = element.closest?.('[role="article"],article')?.textContent || "";
    // Sponsored/non-friend cards can expose a landing URL; Copy link has the canonical video URL.
    if (/(?:patrocinado|sponsored)/i.test(postText)) {
      const menuUrl = await facebookUrlFromMenu(element);
      if (menuUrl) return menuUrl;
    }
    const immediate = facebookUrlFor(element);
    if (immediate) return immediate;
'''
if old in s: s = s.replace(old, new, 1)
elif 'Sponsored/non-friend cards' not in s: raise SystemExit('facebook block missing')
p.write_text(s, encoding='utf-8')

p = Path('browser-extension/background.js')
s = p.read_text(encoding='utf-8')
s = s.replace('  if (!url || !item?.id || chatgptLibraryTransfers.has(item.id)) return;\n  chatgptLibraryTransfers.add(item.id);', '  if (!url) return;\n  const transferKey = Number.isInteger(item?.id) ? `download:${item.id}` : `direct:${url}`;\n  if (chatgptLibraryTransfers.has(transferKey)) return;\n  chatgptLibraryTransfers.add(transferKey);', 1)
s = s.replace('    await diagnostic("chatgpt.library.intercepted", state, { detail: `download_id=${item.id}` });\n    await cancelChromeDownload(item.id);\n    await eraseChromeDownload(item.id);', '    await diagnostic("chatgpt.library.intercepted", state, { detail: Number.isInteger(item?.id) ? `download_id=${item.id}` : "direct_click=1" });\n    if (Number.isInteger(item?.id)) {\n      await cancelChromeDownload(item.id);\n      await eraseChromeDownload(item.id);\n    }', 1)
s = s.replace('    chatgptLibraryTransfers.delete(item.id);', '    chatgptLibraryTransfers.delete(transferKey);', 1)
marker = '''chrome.downloads.onCreated.addListener((item) => {
  if (!chatgptLibraryUrl(item?.finalUrl || item?.url || "")) return;
  void streamChatGPTLibraryDownload(item);
});
'''
handler = marker + '''
chrome.runtime.onMessage.addListener((message, sender, reply) => {
  if (message?.type !== "APOCALIPSE_CHATGPT_LIBRARY_DIRECT") return;
  const url = chatgptLibraryUrl(message.url || "");
  if (!url) { reply({ ok: false, error: "invalid_chatgpt_library_url" }); return; }
  void streamChatGPTLibraryDownload({ url, referrer: message.pageUrl || sender.tab?.url || "https://chatgpt.com/", filename: message.fileName || "" });
  reply({ ok: true, target: "apocalipse" });
});
'''
if 'invalid_chatgpt_library_url' not in s:
    if marker not in s: raise SystemExit('background marker missing')
    s = s.replace(marker, handler, 1)

# Rapidgator: while this tab is armed, deny Chrome's own download UI. Fetch still
# pauses the final response and streams it into Apocalipse, which owns destination selection.
arm_old = '''    await debuggerAttach(debuggee);\n    await diagnostic("rapidgator.cdp.debugger_attached", state);\n    await debuggerCommand(debuggee, "Fetch.enable", {'''
arm_new = '''    await debuggerAttach(debuggee);\n    await diagnostic("rapidgator.cdp.debugger_attached", state);\n    await debuggerCommand(debuggee, "Page.setDownloadBehavior", { behavior: "deny" }).catch(() => {});\n    await diagnostic("rapidgator.cdp.chrome_download_denied", state, { detail: "scope=tab" });\n    await debuggerCommand(debuggee, "Fetch.enable", {'''
if 'rapidgator.cdp.chrome_download_denied' not in s:
    if arm_old not in s: raise SystemExit('rapidgator arm marker missing')
    s = s.replace(arm_old, arm_new, 1)

disarm_old = '''  rapidgatorArmedTabs.delete(tabId);\n  await diagnostic("rapidgator.cdp.disarmed", state, { detail: `reason=${reason}` });\n  await debuggerDetach({ tabId });'''
disarm_new = '''  rapidgatorArmedTabs.delete(tabId);\n  await diagnostic("rapidgator.cdp.disarmed", state, { detail: `reason=${reason}` });\n  await debuggerCommand({ tabId }, "Page.setDownloadBehavior", { behavior: "default" }).catch(() => {});\n  await debuggerDetach({ tabId });'''
if 'behavior: "default"' not in s:
    if disarm_old not in s: raise SystemExit('rapidgator disarm marker missing')
    s = s.replace(disarm_old, disarm_new, 1)
p.write_text(s, encoding='utf-8')

p = Path('browser-extension/manifest.json')
s = p.read_text(encoding='utf-8')
for old_version in ('0.3.40', '0.3.41'):
    s = s.replace(f'"version": "{old_version}"', '"version": "0.3.42"', 1)
p.write_text(s, encoding='utf-8')
