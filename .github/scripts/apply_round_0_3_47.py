from pathlib import Path

# 0.3.47: fixes based on the 0.3.46 diagnostic, not heuristics.
# 1) ChatGPT Library: cancel Chrome's download BEFORE any awaited diagnostic/fetch.
# 2) Video overlays: live video src/blob and captured media resources beat the page URL.
# 3) Facebook sponsored: try the currently playing media before falling back to page/menu URL.

p = Path('browser-extension/background.js')
s = p.read_text(encoding='utf-8')

# Critical race: previous code logged first and only then cancelled Chrome. Cancel synchronously first.
old = '''    await diagnostic("chatgpt.library.intercepted", state, { detail: Number.isInteger(item?.id) ? `download_id=${item.id}` : "direct_click=1" });
    if (Number.isInteger(item?.id)) {
      await cancelChromeDownload(item.id);
      await eraseChromeDownload(item.id);
    }
'''
new = '''    if (Number.isInteger(item?.id)) {
      // Do this before *any* await. Chromium may show Save As as soon as the
      // download is created; diagnostic/bridge latency must never win this race.
      try { chrome.downloads.cancel(item.id); } catch {}
      try { chrome.downloads.erase({ id: item.id }); } catch {}
    }
    await diagnostic("chatgpt.library.intercepted", state, { detail: Number.isInteger(item?.id) ? `download_id=${item.id} chrome_cancel_first=1` : "direct_click=1" });
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'chrome_cancel_first=1' not in s:
    raise SystemExit('ChatGPT cancellation race block not found')

# Also cancel in onCreated itself before handing off to the async streamer.
old = '''chrome.downloads.onCreated.addListener((item) => {
  if (!chatgptLibraryUrl(item?.finalUrl || item?.url || "")) return;
  void streamChatGPTLibraryDownload(item);
});
'''
new = '''chrome.downloads.onCreated.addListener((item) => {
  if (!chatgptLibraryUrl(item?.finalUrl || item?.url || "")) return;
  // First instruction executed for a Library download: kill Chrome's native job.
  try { chrome.downloads.cancel(item.id); } catch {}
  void streamChatGPTLibraryDownload(item);
});
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'First instruction executed for a Library download' not in s:
    raise SystemExit('ChatGPT onCreated block not found')

p.write_text(s, encoding='utf-8')

p = Path('browser-extension/content.js')
s = p.read_text(encoding='utf-8')

# Pick actual media network resources seen by the browser. This is especially useful
# for MSE players where video.currentSrc is blob: but the bytes came from mp4/m3u8 URLs.
anchor = '''  const safeMediaFileName = (fallback = "video.mp4", blob = null) => {
'''
helper = '''  const recentNetworkMediaUrl = () => {
    try {
      const entries = performance.getEntriesByType("resource");
      for (let i = entries.length - 1; i >= 0; i -= 1) {
        const name = String(entries[i]?.name || "");
        if (!/^https?:/i.test(name)) continue;
        if (/\\.(?:mp4|webm|m3u8|mpd)(?:[?#]|$)/i.test(name)
            || /(?:video|manifest|playlist|master|DVIDS|dvidshub)/i.test(name)) return name;
      }
    } catch {}
    return null;
  };

'''
if 'const recentNetworkMediaUrl =' not in s:
    if anchor not in s: raise SystemExit('media filename anchor missing')
    s = s.replace(anchor, helper + anchor, 1)

# Inject a live-source fast path at the beginning of the download click handler,
# before the page URL can be selected as "currentUrl".
needle = '''        const currentUrl = resolved?.url || resolved || (isYouTubeVideo ? location.href : null);
        const liveBlobUrl = /^blob:/i.test(String(element.currentSrc || element.src || "")) ? (element.currentSrc || element.src) : null;
'''
replacement = '''        const liveSource = String(element.currentSrc || element.src || "");
        const liveBlobUrl = /^blob:/i.test(liveSource) ? liveSource : null;
        const liveHttpUrl = /^https?:/i.test(liveSource) ? liveSource : null;
        const networkMediaUrl = recentNetworkMediaUrl();

        // For a real <video>, the source feeding the player is more authoritative
        // than location.href. Try readable blob first; for MSE blobs, fall through
        // to the most recent underlying media request captured by Performance API.
        if (liveBlobUrl) {
          try {
            const fallbackName = isFacebookVideo ? facebookDownloadTitle(location.href) : null;
            await uploadBlobUrl(liveBlobUrl, fallbackName);
            button.textContent = "✓";
            button.title = "Enviado ao Apocalipse";
            setTimeout(() => { button.textContent = originalText; }, 1500);
            return;
          } catch (error) {
            console.debug("Apocalipse live blob is MSE/unreadable; trying network media", error);
          }
        }

        let currentUrl = liveHttpUrl || networkMediaUrl || resolved?.url || resolved || (isYouTubeVideo ? location.href : null);
'''
if needle in s:
    s = s.replace(needle, replacement, 1)
elif 'live blob is MSE/unreadable' not in s:
    raise SystemExit('currentUrl/liveBlob block not found')

# The previous 0.3.46 fallback block would retry the same blob after the fast path.
old = '''        if ((!currentUrl || (isFacebookVideo && !isFacebookMediaUrl(currentUrl))) && liveBlobUrl) {
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
'''
if old in s:
    s = s.replace(old, '', 1)

# On Facebook, a CDN/media URL is valid even though it is not facebook.com/reel/... .
# Do not reject a network media URL merely because it is not a Facebook page URL.
old = '''        if (!currentUrl || (isFacebookVideo && !isFacebookMediaUrl(currentUrl))) {
          button.textContent = "⚠";
'''
new = '''        const facebookPlayableUrl = isFacebookVideo && currentUrl && (
          isFacebookMediaUrl(currentUrl)
          || /\\.(?:mp4|webm|m3u8|mpd)(?:[?#]|$)/i.test(currentUrl)
          || /(?:fbcdn|fbsbx|video)/i.test(currentUrl)
        );
        if (!currentUrl || (isFacebookVideo && !facebookPlayableUrl)) {
          button.textContent = "⚠";
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'facebookPlayableUrl' not in s:
    raise SystemExit('Facebook warning predicate not found')

p.write_text(s, encoding='utf-8')

p = Path('browser-extension/manifest.json')
s = p.read_text(encoding='utf-8')
for old_version in ('0.3.43', '0.3.44', '0.3.45', '0.3.46'):
    s = s.replace(f'"version": "{old_version}"', '"version": "0.3.47"', 1)
p.write_text(s, encoding='utf-8')
