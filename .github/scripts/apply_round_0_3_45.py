from pathlib import Path

# Applied after 0.3.44 test patch.
# Goals:
# - Make the native Apocalipse destination dialog the foreground/owned dialog.
# - Suppress Chrome's own Save As dialog for ChatGPT Library and Rapidgator final downloads.
# - Make Facebook sponsored/reel controls prefer the authoritative Copy Link URL.
# - Give Facebook downloads a stable reel/share id instead of generic download.mp4 when possible.

# --- Desktop native destination dialog: parent it to the main Apocalipse window.
p = Path('apps/desktop/src-tauri/src/main.rs')
s = p.read_text(encoding='utf-8')
old = '''        let Some(path) = rfd::FileDialog::new()
            .set_directory(&directory)
            .set_file_name(&file_name)
            .save_file()
'''
new = '''        let main_window = app.get_webview_window("main");
        if let Some(window) = main_window.as_ref() {
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
        }
        // Browser-triggered blob transfers can race Chrome's own Save As window.
        // Own the dialog with the Apocalipse main window so it stays in front.
        let mut dialog = rfd::FileDialog::new()
            .set_title("Apocalipse Download Manager - Save as")
            .set_directory(&directory)
            .set_file_name(&file_name);
        if let Some(window) = main_window.as_ref() {
            dialog = dialog.set_parent(window);
        }
        let Some(path) = dialog.save_file()
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'dialog = dialog.set_parent(window);' not in s:
    raise SystemExit('native blob destination dialog anchor missing')
p.write_text(s, encoding='utf-8')

# --- Service worker: kill browser-native downloads before Chrome can show Save As.
p = Path('browser-extension/service-worker.js')
s = p.read_text(encoding='utf-8')
anchor = '''function isChatGPTLibraryDownload(value) {
  try {
    const url = new URL(value);
    return url.hostname.toLowerCase() === "chatgpt.com" && url.pathname === "/backend-api/estuary/content";
  } catch {
    return false;
  }
}
'''
helper = anchor + '''
function isRapidgatorFinalDownload(value) {
  try {
    const url = new URL(value);
    return /^s\\d+\\.rapidgator\\.net$/i.test(url.hostname)
      && /^\\/download\\/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\\/?$/i.test(url.pathname);
  } catch {
    return false;
  }
}
'''
if 'function isRapidgatorFinalDownload' not in s:
    if anchor not in s: raise SystemExit('download predicate anchor missing')
    s = s.replace(anchor, helper, 1)

old = '''  // background.js owns ChatGPT Library downloads and streams them through the
  // authenticated blob bridge. Never let the generic interception path suggest
  // or recreate the original Chrome download, otherwise a second Save As flow
  // can appear after Apocalipse has already accepted the transfer.
  if (isChatGPTLibraryDownload(url)) return true;
'''
new = '''  // These are already owned by the dedicated authenticated streaming/CDP paths.
  // Cancel right here, inside filename determination, before Chrome can show its
  // own Save As window over the Apocalipse destination dialog.
  if (isChatGPTLibraryDownload(url) || (bridgeConnected && isRapidgatorFinalDownload(url))) {
    await cancelBrowserDownload(item.id).catch(() => {});
    await eraseBrowserDownload(item.id);
    return true;
  }
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'isRapidgatorFinalDownload(url)' not in s or 'await cancelBrowserDownload(item.id)' not in s:
    raise SystemExit('special browser cancellation block missing')
p.write_text(s, encoding='utf-8')

# --- Facebook: prefer canonical/copied URL and stable filename hints.
p = Path('browser-extension/content.js')
s = p.read_text(encoding='utf-8')

anchor = '''  const waitForFacebookUrl = async (element, attempts = 20) => {
'''
helper = '''  const facebookMediaId = (url) => {
    try {
      const parsed = new URL(url, location.href);
      const numeric = parsed.pathname.match(/\\/(?:reel|reels|videos|posts)\\/(\\d+)/i)?.[1]
        || parsed.searchParams.get("v")
        || parsed.searchParams.get("fbid")
        || parsed.searchParams.get("story_fbid");
      if (numeric) return numeric;
      const shared = parsed.pathname.match(/\\/share\\/[rv]\\/([^/?#]+)/i)?.[1];
      if (shared) return shared.replace(/[^A-Za-z0-9_-]+/g, "");
    } catch {}
    return null;
  };
  const facebookDownloadTitle = (url) => {
    const id = facebookMediaId(url);
    return id ? `${id}.mp4` : "facebook-video.mp4";
  };

'''
if 'const facebookMediaId =' not in s:
    if anchor not in s: raise SystemExit('facebook helper anchor missing')
    s = s.replace(anchor, helper + anchor, 1)

old = '''  const revealFacebookUrl = async (element) => {
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
new = '''  const revealFacebookUrl = async (element) => {
    if (!/(^|\\.)facebook\\.com$/i.test(location.hostname)) return null;
    const postText = element.closest?.('[role="article"],article')?.textContent || "";
    const sponsored = /(?:patrocinado|sponsored)/i.test(postText);
    if (sponsored) {
      const menuUrl = await facebookUrlFromMenu(element);
      if (menuUrl && menuUrl !== "clipboard-copied") return menuUrl;
    }
    const immediate = facebookUrlFor(element);
    if (immediate) return immediate;
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'const sponsored =' not in s:
    raise SystemExit('facebook reveal canonical block missing')

old = '''        const copiedToClipboard = isFacebookVideo ? copyTextNow(currentUrl) : false;
        chrome.runtime.sendMessage({ type: "APOCALIPSE_DOWNLOAD", item: { url: currentUrl, duration: resolved?.duration || null, requestUrls: resolved?.requestUrls || [], userAgent: navigator.userAgent, kind: element.tagName.toLowerCase(), title: document.title, thumbnail: thumbnailFor(element, "video") } }, (result) => {
'''
new = '''        const copiedToClipboard = isFacebookVideo ? copyTextNow(currentUrl) : false;
        const directFacebookMedia = isFacebookVideo ? absolute(element.currentSrc || element.src) : null;
        const requestUrls = [...new Set([
          ...(resolved?.requestUrls || []),
          ...(directFacebookMedia && /^https?:/i.test(directFacebookMedia) ? [directFacebookMedia] : []),
        ])];
        chrome.runtime.sendMessage({ type: "APOCALIPSE_DOWNLOAD", item: { url: currentUrl, duration: resolved?.duration || null, requestUrls, userAgent: navigator.userAgent, kind: element.tagName.toLowerCase(), title: isFacebookVideo ? facebookDownloadTitle(currentUrl) : document.title, thumbnail: thumbnailFor(element, "video") } }, (result) => {
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'facebookDownloadTitle(currentUrl)' not in s:
    raise SystemExit('facebook download handoff anchor missing')
p.write_text(s, encoding='utf-8')

# --- Version marker.
p = Path('browser-extension/manifest.json')
s = p.read_text(encoding='utf-8')
for old_version in ('0.3.43', '0.3.44'):
    s = s.replace(f'"version": "{old_version}"', '"version": "0.3.45"', 1)
p.write_text(s, encoding='utf-8')
