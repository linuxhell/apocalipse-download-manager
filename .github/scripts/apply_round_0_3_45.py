from pathlib import Path

# Applied after 0.3.44 test patch.
# Goals:
# - Make the native Apocalipse destination dialog the foreground/owned dialog.
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
        // On Windows an unowned native FileDialog can end up behind Chrome when
        // a browser-triggered blob starts. Make the Apocalipse window its owner.
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

# --- Facebook: prefer canonical/copied URL and stable filename hints.
p = Path('browser-extension/content.js')
s = p.read_text(encoding='utf-8')

# Helper for a stable numeric/share identifier.
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

# For sponsored cards, Copy Link must be authoritative before any DOM/helper URL.
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
    // Sponsored cards often expose advertiser/landing URLs in their DOM. The
    // Facebook menu's Copy Link points at the actual video and wins every time.
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

# On Facebook buttons, use the resolved URL id as the title/filename hint and include
# the playing media URL as a fallback candidate for future backend recovery.
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
