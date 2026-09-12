// TikTok share dialogs can refuse a synthetic Copy click after user activation
// expires. Resolve the exact item from the freshly opened dialog's own framework
// data before using clipboard as a last resort, and always dismiss our dialog.
(() => {
  const C = globalThis.ADM_SOCIAL_HOME_FEED_V3_CORE;
  const T = globalThis.ADM_SOCIAL_HOME_FEED_V3_TT;
  if (!C || !T || T.menuV4) return;

  const canonical = value => {
    try {
      const url = new URL(String(value || '').replaceAll('\\/', '/'), location.href);
      const match = url.pathname.match(/^\/@([A-Za-z0-9._-]+)\/video\/(\d+)\/?$/);
      return /(^|\.)tiktok\.com$/i.test(url.hostname) && match
        ? `https://www.tiktok.com/@${match[1]}/video/${match[2]}` : null;
    } catch { return null; }
  };
  const shortLink = value => {
    for (const raw of String(value || '').replaceAll('\\/', '/').match(/https?:\/\/[^\s<>"']+/ig) || []) {
      try {
        const url = new URL(raw), host = url.hostname.toLowerCase();
        if (host === 'vm.tiktok.com' || host === 'vt.tiktok.com'
          || ((host === 'tiktok.com' || host.endsWith('.tiktok.com')) && /^\/t\//i.test(url.pathname))) return url.href;
      } catch {}
    }
    return null;
  };
  const identityFromNodes = nodes => {
    const roots = [];
    for (const start of nodes) {
      for (let node = start, depth = 0; node && depth < 10; node = node.parentElement, depth += 1) {
        for (const key of Object.getOwnPropertyNames(node)) {
          if (/^__(?:reactProps|reactFiber|vue)/i.test(key)) roots.push(node[key]);
        }
      }
    }
    const queue = [...roots], seen = new WeakSet();
    let candidate = null;
    for (let cursor = 0; cursor < queue.length && cursor < 8000; cursor += 1) {
      const value = queue[cursor];
      if (typeof value === 'string') {
        const direct = canonical(value) || value.match(/https?:\/\/(?:www\.)?tiktok\.com\/@[A-Za-z0-9._-]+\/video\/\d+/i)?.[0];
        if (direct && canonical(direct)) return { url: canonical(direct), candidate, inspected: cursor + 1 };
        candidate ||= shortLink(value);
        continue;
      }
      if (!value || typeof value !== 'object' || seen.has(value) || value.nodeType) continue;
      seen.add(value);
      const rawId = value.id || value.itemId || value.aweme_id || value.awemeId || '';
      const id = typeof rawId === 'number' && !Number.isSafeInteger(rawId) ? '' : String(rawId);
      const author = value.author?.uniqueId || value.author?.unique_id || value.authorInfo?.uniqueId;
      if (/^\d+$/.test(id) && author && (value.video || value.videoInfo)) {
        const url = canonical(`https://www.tiktok.com/@${author}/video/${id}`);
        if (url) return { url, candidate, inspected: cursor + 1 };
      }
      for (const [key, child] of Object.entries(value)) {
        if (!['return', 'sibling', '_owner', 'alternate', 'stateNode'].includes(key)) queue.push(child);
      }
    }
    return { url: null, candidate, inspected: Math.min(queue.length, 8000) };
  };
  const dismiss = () => {
    const close = [...document.querySelectorAll('[data-e2e*="close"],[data-testid*="close"],[aria-label],[title],button')]
      .find(node => C.vis(node) && /^(?:close|fechar|关闭|關閉|cerrar)$/i.test(String(node.getAttribute?.('aria-label') || node.title || node.textContent || '').trim()));
    if (close) C.clickTarget(close)?.click?.();
    else if (typeof KeyboardEvent === 'function') document.dispatchEvent?.(new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', bubbles: true }));
  };

  T.menu = async video => {
    const videoRect = C.rect(video);
    if (!videoRect) return { url: null, reason: 'tiktok_player_rect_missing', source: 'menu_copy_link' };
    const controls = C.controls(video,
      'button,[role="button"],[data-e2e],[data-testid],[aria-label],[title],[tabindex="0"]',
      '[data-e2e*="share"],[data-testid*="share"]');
    const shareRe = /(?:share|compartilhar|compartilhe|分享|共享|공유|シェア)/i;
    const band = controls.items.filter(entry => {
      if (!C.vis(entry.target)) return false;
      const point = C.center(C.rect(entry.target));
      return point.y >= videoRect.top - 180 && point.y <= videoRect.bottom + 180
        && point.x >= videoRect.left - 260 && point.x <= videoRect.right + 420;
    });
    const labeled = band.filter(entry => shareRe.test(entry.text));
    const data = band.filter(entry => /share/i.test(`${entry.raw?.getAttribute?.('data-e2e') || ''} ${entry.raw?.getAttribute?.('data-testid') || ''}`));
    const share = (labeled.length ? labeled : data)[0]?.target;
    const base = { source: 'menu_copy_link', scope: controls.s, buttonCandidates: controls.items.length,
      labeledButtons: labeled.length, dataE2eCandidates: data.length,
      ariaShareCandidates: band.filter(entry => shareRe.test(entry.target.getAttribute?.('aria-label') || '')).length,
      geometryMatchedControls: band.length, extendedScopeNodes: controls.s.nodes.length };
    if (!share) return { ...base, url: null, reason: 'tiktok_share_button_not_found' };

    const selector = '[data-e2e],[data-testid],[aria-label],[title],button,[role="button"],[role="menuitem"],a';
    const before = new Set([...document.querySelectorAll(selector)].filter(C.vis));
    share.click();
    let fresh = [], copy = null, found = { url: null, candidate: null, inspected: 0 };
    const copyRe = /(?:copy.?link|copiar link|\bcopy\b|copiar|复制|複製|복사|コピー)/i;
    for (let attempt = 0; attempt < 24 && !copy && !found.url; attempt += 1) {
      await C.wait(100);
      fresh = [...document.querySelectorAll(selector)].filter(node => C.vis(node) && !before.has(node));
      found = identityFromNodes(fresh);
      copy = fresh.map(C.clickTarget).find(node => copyRe.test(C.ev(node)));
    }
    if (found.url) {
      dismiss();
      return { ...base, url: found.url, reason: 'tiktok_share_dialog_framework_identity',
        freshItemsSeen: fresh.length, frameworkValuesInspected: found.inspected };
    }
    if (!copy) {
      dismiss();
      return { ...base, url: null, reason: 'tiktok_copy_link_item_not_found', freshItemsSeen: fresh.length,
        frameworkValuesInspected: found.inspected };
    }
    copy.click();
    setTimeout(dismiss, 600);
    return { ...base, url: null, reason: 'tiktok_copy_link_clicked', freshItemsSeen: fresh.length,
      frameworkValuesInspected: found.inspected, clipboardRequested: true,
      clipboardCandidate: found.candidate || null };
  };
  T.menuV4 = true;
})();
