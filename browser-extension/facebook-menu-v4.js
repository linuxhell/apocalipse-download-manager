// Facebook friend/collection posts may omit Copy Link from the three-dot menu.
// Retry through the post's Share control without ever turning Preview into Download.
(() => {
  const C = globalThis.ADM_SOCIAL_HOME_FEED_V3_CORE;
  const F = globalThis.ADM_SOCIAL_HOME_FEED_V3_FB;
  if (!C || !F || F.menuV4) return;
  const original = F.menu;
  const copyRe = F.copyRe;
  const dismiss = () => {
    if (typeof KeyboardEvent !== 'function') return;
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', bubbles: true }));
    document.dispatchEvent(new KeyboardEvent('keyup', { key: 'Escape', code: 'Escape', bubbles: true }));
  };
  const copyFromVisibleMenu = async before => {
    const selector = '[role="menuitem"],[role="option"],[aria-label],[title],[tabindex="0"],a,button';
    for (let attempt = 0; attempt < 24; attempt += 1) {
      await C.wait(100);
      const now = [...document.querySelectorAll(selector)].filter(C.vis);
      const copy = now.find(node => !before.has(node) && copyRe.test(C.ev(node)))
        || now.find(node => copyRe.test(C.ev(node)));
      if (!copy) continue;
      for (const key of ['href', 'data-url', 'data-clipboard-text']) {
        const info = F.info(copy.getAttribute?.(key) || copy[key]);
        if (info.url) return { ...info, clipboardRequested: false };
      }
      copy.click();
      return { url: null, reason: 'facebook_share_copy_link_clicked', clipboardRequested: true };
    }
    return null;
  };
  F.menu = async video => {
    const first = await original(video);
    if (first.url || first.clipboardRequested || first.reason !== 'facebook_copy_link_item_not_found') return first;
    dismiss();
    await C.wait(80);
    const rect = C.rect(video);
    const controls = C.controls(video, 'button,[role="button"],[aria-label],[title],[tabindex="0"]', 'button,[role="button"]');
    const shareRe = /(?:share|compartilhar|compartilhe|partilhar|分享|공유|シェア)/i;
    const candidates = controls.items.filter(entry => {
      if (!C.vis(entry.target) || !shareRe.test(entry.text)) return false;
      const point = C.center(C.rect(entry.target));
      return point.x >= rect.left - 180 && point.x <= rect.right + 180
        && point.y >= rect.bottom - 140 && point.y <= rect.bottom + 300;
    });
    if (candidates.length !== 1) return { ...first, reason: candidates.length ? 'facebook_share_button_ambiguous' : 'facebook_share_button_not_found', shareCandidates: candidates.length };
    const selector = '[role="menuitem"],[role="option"],[aria-label],[title],[tabindex="0"],a,button';
    const before = new Set([...document.querySelectorAll(selector)].filter(C.vis));
    candidates[0].target.click();
    const result = await copyFromVisibleMenu(before);
    return result ? { ...first, ...result, source: 'share_copy_link', shareCandidates: 1 }
      : { ...first, reason: 'facebook_share_copy_link_item_not_found', shareCandidates: 1 };
  };
  F.menuV4 = true;
})();
