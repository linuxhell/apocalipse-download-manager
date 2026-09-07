from pathlib import Path

p = Path('browser-extension/content.js')
s = p.read_text(encoding='utf-8')
anchor = '  const copyTextNow = (value) => {'
direct = '''  // Intercept ChatGPT Library links before Chrome creates its own download dialog.
  // Use composedPath + nearby link discovery because ChatGPT may wrap the visible
  // download control in buttons/spans instead of making the clicked node the anchor.
  const chatgptLibraryLinkForEvent = (event) => {
    const candidates = [];
    for (const node of event.composedPath?.() || []) {
      if (node?.href) candidates.push(node);
      const closest = node?.closest?.('a[href*="/backend-api/estuary/content"]');
      if (closest) candidates.push(closest);
      const nested = node?.querySelector?.('a[href*="/backend-api/estuary/content"]');
      if (nested) candidates.push(nested);
    }
    const target = event.target;
    for (let parent = target; parent && parent !== document.documentElement; parent = parent.parentElement) {
      const nested = parent.querySelector?.('a[href*="/backend-api/estuary/content"]');
      if (nested) candidates.push(nested);
      if (parent.matches?.('a[href*="/backend-api/estuary/content"]')) candidates.push(parent);
      if (candidates.length) break;
    }
    for (const candidate of candidates) {
      try {
        const parsed = new URL(candidate.href, location.href);
        if (parsed.hostname.toLowerCase() === "chatgpt.com" && parsed.pathname === "/backend-api/estuary/content") {
          return { url: parsed.href, fileName: candidate.getAttribute?.("download") || "" };
        }
      } catch {}
    }
    return null;
  };
  const interceptChatgptLibrary = (event) => {
    if (event.defaultPrevented || (typeof event.button === "number" && event.button !== 0)) return;
    const found = chatgptLibraryLinkForEvent(event);
    if (!found) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    chrome.runtime.sendMessage({
      type: "APOCALIPSE_CHATGPT_LIBRARY_DIRECT",
      url: found.url,
      pageUrl: location.href,
      fileName: found.fileName,
    }).catch(() => {});
  };
  document.addEventListener("pointerdown", interceptChatgptLibrary, true);
  document.addEventListener("click", interceptChatgptLibrary, true);

'''
if 'APOCALIPSE_CHATGPT_LIBRARY_DIRECT' not in s:
    if anchor not in s: raise SystemExit('content anchor missing')
    s = s.replace(anchor, direct + anchor, 1)

old_media = '''      if (!/(^|\\.)facebook\\.com$/i.test(parsed.hostname)) return false;
      return /(?:^|\\/)(?:reel|reels|watch|videos|posts|share)(?:\\/|$)/i.test(parsed.pathname)'''
new_media = '''      if (!/(^|\\.)facebook\\.com$/i.test(parsed.hostname)) return false;
      if (/\\/(?:watch\\/hashtag|hashtag)(?:\\/|$)/i.test(parsed.pathname)) return false;
      return /(?:^|\\/)(?:reel|reels|watch|videos|posts|share)(?:\\/|$)/i.test(parsed.pathname)'''
if old_media in s:
    s = s.replace(old_media, new_media, 1)
elif 'watch\\/hashtag' not in s:
    raise SystemExit('facebook media predicate missing')

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
    // Sponsored cards often expose landing/helper URLs. Copy link is authoritative.
    if (/(?:patrocinado|sponsored)/i.test(postText)) {
      const menuUrl = await facebookUrlFromMenu(element);
      if (menuUrl && menuUrl !== "clipboard-copied") return menuUrl;
    }
    const immediate = facebookUrlFor(element);
    if (immediate) return immediate;
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'Sponsored cards often expose landing/helper URLs' not in s:
    raise SystemExit('facebook reveal block missing')
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
  const directState = { traceId: crypto.randomUUID(), pageUrl: message.pageUrl || sender.tab?.url || null, url, startedAt: Date.now(), bytes: 0 };
  void diagnostic("chatgpt.library.direct_click", directState, { detail: "chrome_download_not_created=1" });
  void streamChatGPTLibraryDownload({ url, referrer: message.pageUrl || sender.tab?.url || "https://chatgpt.com/", filename: message.fileName || "" });
  reply({ ok: true, target: "apocalipse" });
});
'''
if 'invalid_chatgpt_library_url' not in s:
    if marker not in s: raise SystemExit('background marker missing')
    s = s.replace(marker, handler, 1)
elif 'chatgpt.library.direct_click' not in s:
    old_handler = '''chrome.runtime.onMessage.addListener((message, sender, reply) => {
  if (message?.type !== "APOCALIPSE_CHATGPT_LIBRARY_DIRECT") return;
  const url = chatgptLibraryUrl(message.url || "");
  if (!url) { reply({ ok: false, error: "invalid_chatgpt_library_url" }); return; }
  void streamChatGPTLibraryDownload({ url, referrer: message.pageUrl || sender.tab?.url || "https://chatgpt.com/", filename: message.fileName || "" });
  reply({ ok: true, target: "apocalipse" });
});
'''
    if old_handler in s:
        s = s.replace(old_handler, handler[len(marker)+1:], 1)

arm_old = '''    await debuggerAttach(debuggee);
    await diagnostic("rapidgator.cdp.debugger_attached", state);
    await debuggerCommand(debuggee, "Fetch.enable", {'''
arm_new = '''    await debuggerAttach(debuggee);
    await diagnostic("rapidgator.cdp.debugger_attached", state);
    await debuggerCommand(debuggee, "Page.setDownloadBehavior", { behavior: "deny" }).catch(() => {});
    await diagnostic("rapidgator.cdp.chrome_download_denied", state, { detail: "scope=tab" });
    await debuggerCommand(debuggee, "Fetch.enable", {'''
if 'rapidgator.cdp.chrome_download_denied' not in s:
    if arm_old not in s: raise SystemExit('rapidgator arm marker missing')
    s = s.replace(arm_old, arm_new, 1)

disarm_old = '''  rapidgatorArmedTabs.delete(tabId);
  await diagnostic("rapidgator.cdp.disarmed", state, { detail: `reason=${reason}` });
  await debuggerDetach({ tabId });'''
disarm_new = '''  rapidgatorArmedTabs.delete(tabId);
  await diagnostic("rapidgator.cdp.disarmed", state, { detail: `reason=${reason}` });
  await debuggerCommand({ tabId }, "Page.setDownloadBehavior", { behavior: "default" }).catch(() => {});
  await debuggerDetach({ tabId });'''
if 'behavior: "default"' not in s:
    if disarm_old not in s: raise SystemExit('rapidgator disarm marker missing')
    s = s.replace(disarm_old, disarm_new, 1)
p.write_text(s, encoding='utf-8')

p = Path('browser-extension/manifest.json')
s = p.read_text(encoding='utf-8')
for old_version in ('0.3.40', '0.3.41', '0.3.42', '0.3.43'):
    s = s.replace(f'"version": "{old_version}"', '"version": "0.3.43"', 1)
p.write_text(s, encoding='utf-8')
