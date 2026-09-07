from pathlib import Path

# 0.3.46 test patch
# - Treat blob: video sources as first-class media candidates.
# - Upload readable blob URLs directly through the existing Apocalipse blob bridge.
# - Use blob fallback for Facebook when URL extraction/yt-dlp handoff is unreliable.
# - Harden Chrome native Save As suppression for ChatGPT Library and Rapidgator via CDP.

# --- content.js: blob media path + ChatGPT library early suppression hint.
p = Path('browser-extension/content.js')
s = p.read_text(encoding='utf-8')

anchor = '''  const uploadBlob = async (blob, fileName) => {
'''
helper = '''  const safeMediaFileName = (fallback = "video.mp4", blob = null) => {
    const title = (document.title || "video").replace(/[<>:\\"/\\\\|?*]+/g, "_").trim().slice(0, 100) || "video";
    const type = String(blob?.type || "").toLowerCase();
    const ext = type.includes("webm") ? ".webm" : type.includes("ogg") ? ".ogv" : ".mp4";
    if (/\\.[A-Za-z0-9]{2,5}$/.test(fallback)) return fallback;
    return `${title}${ext}`;
  };
  const uploadBlobUrl = async (url, fileName = null) => {
    if (!/^blob:/i.test(String(url || ""))) throw new Error("not_blob_url");
    const response = await fetch(url);
    if (!response.ok) throw new Error(`blob_http_${response.status}`);
    const blob = await response.blob();
    if (!blob.size) throw new Error("empty_blob_url");
    const name = safeMediaFileName(fileName || "video", blob);
    await uploadBlob(blob, name);
    return { ok: true, bytes: blob.size, fileName: name };
  };

'''
if 'const uploadBlobUrl =' not in s:
    if anchor not in s: raise SystemExit('uploadBlob anchor missing')
    s = s.replace(anchor, helper + anchor, 1)

# ChatGPT Library: hint background on pointerdown for download controls even when href is created later by React.
anchor = '''  const copyTextNow = (value) => {
'''
chat = '''  const looksLikeChatgptLibraryDownloadControl = (event) => {
    if (!/(^|\\.)chatgpt\\.com$/i.test(location.hostname)) return false;
    const path = location.pathname.toLowerCase();
    if (!path.includes("library")) return false;
    for (const node of event.composedPath?.() || []) {
      const label = `${node?.getAttribute?.("aria-label") || ""} ${node?.title || ""} ${node?.textContent || ""}`.trim();
      if (/(?:download|baixar|下载)/i.test(label)) return true;
      const href = node?.href || node?.closest?.('a[href*="/backend-api/estuary/content"]')?.href;
      if (href && /\\/backend-api\\/estuary\\/content/i.test(href)) return true;
    }
    return false;
  };
  document.addEventListener("pointerdown", (event) => {
    if (!looksLikeChatgptLibraryDownloadControl(event)) return;
    chrome.runtime.sendMessage({ type: "APOCALIPSE_CHATGPT_LIBRARY_ARM_DENY" }).catch(() => {});
  }, true);

'''
if 'APOCALIPSE_CHATGPT_LIBRARY_ARM_DENY' not in s:
    if anchor not in s: raise SystemExit('copyTextNow anchor missing')
    s = s.replace(anchor, chat + anchor, 1)

# Prefer blob direct upload before showing warning for generic video overlays.
old = '''        const currentUrl = resolved?.url || resolved || (isYouTubeVideo ? location.href : null);
        if (!currentUrl || (isFacebookVideo && !isFacebookMediaUrl(currentUrl))) {
          button.textContent = "⚠";
          button.title = "Abra o vídeo ou use os três pontos e Copiar link";
          setTimeout(() => { button.textContent = originalText; }, 2500);
          return;
        }
'''
new = '''        const currentUrl = resolved?.url || resolved || (isYouTubeVideo ? location.href : null);
        const liveBlobUrl = /^blob:/i.test(String(element.currentSrc || element.src || "")) ? (element.currentSrc || element.src) : null;
        if ((!currentUrl || (isFacebookVideo && !isFacebookMediaUrl(currentUrl))) && liveBlobUrl) {
          try {
            const fallbackName = isFacebookVideo ? facebookDownloadTitle(location.href) : null;
            await uploadBlobUrl(liveBlobUrl, fallbackName);
            button.textContent = "✓";
            button.title = "Enviado ao Apocalipse";
            setTimeout(() => { button.textContent = originalText; }, 1500);
            return;
          } catch (error) {
            console.debug("Apocalipse blob fallback failed", error);
          }
        }
        if (!currentUrl || (isFacebookVideo && !isFacebookMediaUrl(currentUrl))) {
          button.textContent = "⚠";
          button.title = "Abra o vídeo ou use os três pontos e Copiar link";
          setTimeout(() => { button.textContent = originalText; }, 2500);
          return;
        }
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'liveBlobUrl' not in s:
    raise SystemExit('overlay warning block missing')

# If a non-Facebook site exposes blob: directly, use it instead of handing blob URL to normal downloader.
old = '''        const copiedToClipboard = isFacebookVideo ? copyTextNow(currentUrl) : false;
        const directFacebookMedia = isFacebookVideo ? absolute(element.currentSrc || element.src) : null;
'''
new = '''        if (/^blob:/i.test(String(currentUrl || ""))) {
          try {
            await uploadBlobUrl(currentUrl);
            button.textContent = "✓";
            button.title = "Enviado ao Apocalipse";
            setTimeout(() => { button.textContent = originalText; }, 1500);
            return;
          } catch (error) {
            console.debug("Apocalipse direct blob failed", error);
          }
        }
        const copiedToClipboard = isFacebookVideo ? copyTextNow(currentUrl) : false;
        const directFacebookMedia = isFacebookVideo ? absolute(element.currentSrc || element.src) : null;
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'Apocalipse direct blob failed' not in s:
    raise SystemExit('facebook handoff anchor missing')

# Ensure blob URL counts as downloadable media for overlay creation.
old = '''      const canDownload = Boolean(url && /^https?:/.test(url));
'''
new = '''      const liveMediaUrl = element.currentSrc || element.src || "";
      const canDownload = Boolean((url && /^https?:/.test(url)) || /^blob:/i.test(liveMediaUrl));
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'const liveMediaUrl =' not in s:
    raise SystemExit('canDownload anchor missing')

p.write_text(s, encoding='utf-8')

# --- background.js: stronger CDP native-download suppression + ChatGPT downloadWillBegin recovery.
p = Path('browser-extension/background.js')
s = p.read_text(encoding='utf-8')

anchor = '''const rapidgatorArmedTabs = new Map();
const chatgptLibraryTransfers = new Set();
'''
replacement = '''const rapidgatorArmedTabs = new Map();
const chatgptLibraryTransfers = new Set();
const chatgptLibraryDeniedTabs = new Map();
'''
if 'chatgptLibraryDeniedTabs' not in s:
    if anchor not in s: raise SystemExit('background maps anchor missing')
    s = s.replace(anchor, replacement, 1)

helper_anchor = '''async function rapidgatorBridgeHealth() {
'''
helper = '''async function denyNativeDownloads(debuggee) {
  // Browser domain is stronger on recent Chromium; Page is kept as fallback.
  await debuggerCommand(debuggee, "Browser.setDownloadBehavior", { behavior: "deny", eventsEnabled: true }).catch(() => {});
  await debuggerCommand(debuggee, "Page.setDownloadBehavior", { behavior: "deny" }).catch(() => {});
}

async function restoreNativeDownloads(debuggee) {
  await debuggerCommand(debuggee, "Browser.setDownloadBehavior", { behavior: "default", eventsEnabled: false }).catch(() => {});
  await debuggerCommand(debuggee, "Page.setDownloadBehavior", { behavior: "default" }).catch(() => {});
}

async function armChatgptLibraryDeny(tabId, pageUrl) {
  if (!tabId) throw new Error("chatgpt_tab_missing");
  const existing = chatgptLibraryDeniedTabs.get(tabId);
  if (existing) {
    clearTimeout(existing.timer);
    existing.timer = setTimeout(() => void disarmChatgptLibraryDeny(tabId), 8000);
    return { armed: true, reused: true };
  }
  const debuggee = { tabId };
  await debuggerAttach(debuggee);
  await debuggerCommand(debuggee, "Page.enable").catch(() => {});
  await denyNativeDownloads(debuggee);
  const state = { traceId: crypto.randomUUID(), pageUrl: pageUrl || null, startedAt: Date.now(), bytes: 0 };
  const timer = setTimeout(() => void disarmChatgptLibraryDeny(tabId), 8000);
  chatgptLibraryDeniedTabs.set(tabId, { state, timer });
  await diagnostic("chatgpt.library.chrome_download_denied", state, { detail: `tab=${tabId}` });
  return { armed: true, reused: false };
}

async function disarmChatgptLibraryDeny(tabId) {
  const entry = chatgptLibraryDeniedTabs.get(tabId);
  if (!entry) return;
  chatgptLibraryDeniedTabs.delete(tabId);
  clearTimeout(entry.timer);
  const debuggee = { tabId };
  await restoreNativeDownloads(debuggee);
  await debuggerDetach(debuggee);
}

'''
if 'async function denyNativeDownloads' not in s:
    if helper_anchor not in s: raise SystemExit('health anchor missing')
    s = s.replace(helper_anchor, helper + helper_anchor, 1)

# Rapidgator: replace weaker Page-only deny/default with helper.
s = s.replace('''    await debuggerCommand(debuggee, "Page.setDownloadBehavior", { behavior: "deny" }).catch(() => {});
    await diagnostic("rapidgator.cdp.chrome_download_denied", state, { detail: "scope=tab" });
''', '''    await denyNativeDownloads(debuggee);
    await diagnostic("rapidgator.cdp.chrome_download_denied", state, { detail: "scope=browser+tab" });
''', 1)
s = s.replace('''  await debuggerCommand({ tabId }, "Page.setDownloadBehavior", { behavior: "default" }).catch(() => {});
  await debuggerDetach({ tabId });''', '''  await restoreNativeDownloads({ tabId });
  await debuggerDetach({ tabId });''', 1)

# Listen for denied ChatGPT download initiation. CDP gives us URL+suggested filename before Chrome Save As.
listener_anchor = '''chrome.debugger.onEvent.addListener((source, method, params) => {
  if (!source.tabId || method !== "Fetch.requestPaused") return;
'''
listener = '''chrome.debugger.onEvent.addListener((source, method, params) => {
  if (source.tabId && method === "Page.downloadWillBegin" && chatgptLibraryDeniedTabs.has(source.tabId)) {
    const entry = chatgptLibraryDeniedTabs.get(source.tabId);
    const url = chatgptLibraryUrl(params?.url || "");
    if (url) {
      void diagnostic("chatgpt.library.download_will_begin", entry.state, { detail: `suggested=${params?.suggestedFilename || ""}` });
      void streamChatGPTLibraryDownload({
        url,
        referrer: entry.state.pageUrl || "https://chatgpt.com/",
        filename: params?.suggestedFilename || "",
      }).finally(() => void disarmChatgptLibraryDeny(source.tabId));
      return;
    }
  }
  if (!source.tabId || method !== "Fetch.requestPaused") return;
'''
if 'chatgpt.library.download_will_begin' not in s:
    if listener_anchor not in s: raise SystemExit('debugger event anchor missing')
    s = s.replace(listener_anchor, listener, 1)

# Message from content script on pointerdown.
msg_anchor = '''chrome.runtime.onMessage.addListener((message, sender, reply) => {
  if (message?.type !== "APOCALIPSE_RAPIDGATOR_ARM") return;
'''
msg = '''chrome.runtime.onMessage.addListener((message, sender, reply) => {
  if (message?.type === "APOCALIPSE_CHATGPT_LIBRARY_ARM_DENY") {
    armChatgptLibraryDeny(sender.tab?.id, sender.tab?.url || null)
      .then(reply)
      .catch((error) => reply({ armed: false, error: String(error) }));
    return true;
  }
  if (message?.type !== "APOCALIPSE_RAPIDGATOR_ARM") return;
'''
if 'APOCALIPSE_CHATGPT_LIBRARY_ARM_DENY' not in s:
    if msg_anchor not in s: raise SystemExit('runtime arm listener missing')
    s = s.replace(msg_anchor, msg, 1)

# Cleanup ChatGPT debugger on tab close.
old = '''chrome.tabs.onRemoved.addListener((tabId) => {
  if (rapidgatorArmedTabs.has(tabId)) void disarmRapidgator(tabId, "tab_closed");
});
'''
new = '''chrome.tabs.onRemoved.addListener((tabId) => {
  if (rapidgatorArmedTabs.has(tabId)) void disarmRapidgator(tabId, "tab_closed");
  if (chatgptLibraryDeniedTabs.has(tabId)) void disarmChatgptLibraryDeny(tabId);
});
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'chatgptLibraryDeniedTabs.has(tabId)' not in s:
    raise SystemExit('tab cleanup missing')

p.write_text(s, encoding='utf-8')

# --- version
p = Path('browser-extension/manifest.json')
s = p.read_text(encoding='utf-8')
for old_version in ('0.3.43', '0.3.44', '0.3.45'):
    s = s.replace(f'"version": "{old_version}"', '"version": "0.3.46"', 1)
p.write_text(s, encoding='utf-8')
