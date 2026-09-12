// Opt-in media interaction replay. It records structure and state, never page
// copy, keystrokes, form values, cookies, or arbitrary clipboard contents.
(() => {
  if (globalThis.ADM_DIAG_MEDIA_REPLAY) return;
  globalThis.ADM_DIAG_MEDIA_REPLAY = true;
  globalThis.ADM_DIAG?.register?.('diagnostics-media-replay.js');

  const supportedHost = host => /(^|\.)(?:facebook|tiktok|instagram)\.com$/i.test(host || '');
  const canonicalMediaUrl = value => {
    try {
      const url = new URL(String(value || '').trim());
      const host = url.hostname.toLowerCase();
      if (/(^|\.)tiktok\.com$/i.test(host) && /^\/@[A-Za-z0-9._-]+\/video\/\d+\/?$/i.test(url.pathname)) {
        return { platform: 'tiktok', canonicalClass: 'tiktok_video', url: `${url.origin}${url.pathname.replace(/\/$/, '')}` };
      }
      if (/(^|\.)facebook\.com$/i.test(host)) {
        const id = url.searchParams.get('v') || url.searchParams.get('story_fbid');
        if (/^\d{5,}$/.test(id || '')) return { platform: 'facebook', canonicalClass: 'facebook_watch_id', url: `https://www.facebook.com/watch/?v=${id}` };
        const route = url.pathname.match(/\/(reel|reels|videos|share\/v)\/[^/?#]+/i)?.[1]?.toLowerCase();
        if (route) {
          const canonicalClass = route === 'share/v' ? 'facebook_share_video'
            : route === 'videos' ? 'facebook_profile_video' : 'facebook_reel';
          return { platform: 'facebook', canonicalClass, url: `${url.origin}${url.pathname}` };
        }
      }
    } catch {}
    return null;
  };
  const editable = node => Boolean(node?.closest?.('input,textarea,select,[contenteditable="true"],[role="textbox"]'));
  const visibleVideo = (x, y, target) => {
    const direct = target?.closest?.('video');
    if (direct) return direct;
    const elements = document.elementsFromPoint?.(x, y) || [];
    const pointed = elements.find(node => node?.tagName === 'VIDEO' || node?.querySelector?.('video'));
    if (pointed?.tagName === 'VIDEO') return pointed;
    const nested = pointed?.querySelector?.('video');
    if (nested) return nested;
    const videos = [...document.querySelectorAll('video')].filter(video => {
      const rect = video.getBoundingClientRect?.();
      return rect && x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom;
    });
    return videos.length === 1 ? videos[0] : null;
  };
  const structuralRef = node => {
    const parts = [];
    for (let current = node, depth = 0; current && depth < 6; current = current.parentElement, depth += 1) {
      const tag = String(current.tagName || 'unknown').toLowerCase();
      const role = String(current.getAttribute?.('role') || '');
      const test = String(current.getAttribute?.('data-e2e') || current.getAttribute?.('data-testid') || '');
      parts.push(`${tag}:${role}:${test}`);
    }
    return parts.join('>');
  };
  const mediaCardClass = video => {
    if (!/(^|\.)facebook\.com$/i.test(location.hostname)) return 'not_facebook';
    const label = /(?:patrocinado|sponsored|publicidade|anúncio|anuncio|gesponsert|sponsorisé|赞助内容)/i;
    for (let node = video, depth = 0; node && depth < 18; node = node.parentElement, depth += 1) {
      if (node === document.body || node === document.documentElement) break;
      const videos = [...(node.querySelectorAll?.('video') || [])];
      if (videos.some(item => item !== video)) break;
      const explicit = node.matches?.('[data-ad-preview],[data-sponsored]')
        || [...(node.querySelectorAll?.('[data-ad-preview],[data-sponsored],[aria-label],[role="link"],a,span') || [])]
          .some(item => {
            const value = `${item.getAttribute?.('aria-label') || ''} ${item.textContent || ''}`.trim();
            return value.length <= 80 && label.test(value);
          });
      if (explicit) return 'facebook_sponsored';
      if (node.matches?.('[role="article"],article')) return 'facebook_ordinary';
    }
    return 'facebook_unknown';
  };
  const describe = (event, phase, traceId, video = null) => {
    const target = event.target;
    const control = target?.closest?.('button,a,[role="button"],[role="menuitem"],[tabindex]');
    const detail = {
      phase, eventType: event.type, mouseButton: event.button === 2 ? 'right' : event.button === 1 ? 'middle' : 'left',
      pointerType: event.pointerType || 'mouse', x: Math.round(event.clientX || 0), y: Math.round(event.clientY || 0),
      elementTag: String(target?.tagName || 'unknown').toLowerCase(), controlTag: String(control?.tagName || 'none').toLowerCase(),
      controlRole: String(control?.getAttribute?.('role') || 'none').toLowerCase(),
      elementRef: structuralRef(target), controlRef: structuralRef(control),
      editable: editable(target), mediaRelated: Boolean(video), overlayControl: Boolean(target?.closest?.('.apocalipse-media-actions,[data-apocalipse-player-id]')),
      mediaCardClass: video ? mediaCardClass(video) : 'none',
      pageUrl: location.href,
      ...(video && globalThis.ADM_DIAG?.player ? globalThis.ADM_DIAG.player(video) : {}),
    };
    void globalThis.ADM_DIAG?.emit?.('media_replay.interaction', detail, traceId);
  };

  let pending = null;
  const clearPending = expected => {
    if (!pending || (expected && pending !== expected)) return;
    clearInterval(pending.timer); clearTimeout(pending.expiry); pending = null;
  };
  const inspectClipboard = async state => {
    if (!state || pending !== state || Date.now() > state.until) return clearPending(state);
    let text = '';
    try { text = await navigator.clipboard.readText(); } catch {
      if (!state.readFailureLogged) {
        state.readFailureLogged = true;
        void globalThis.ADM_DIAG?.emit?.('media_replay.clipboard_probe', { result: 'unavailable', reason: 'clipboard_read_rejected' }, state.traceId, 'WARN');
      }
      return;
    }
    if (!text || text === state.before) return;
    const identity = canonicalMediaUrl(text);
    if (!identity) return; // Arbitrary clipboard content is never emitted.
    void globalThis.ADM_DIAG?.emit?.('media_replay.clipboard_probe', {
      result: 'resolved', reason: 'canonical_media_link_observed', platform: identity.platform,
      canonicalClass: identity.canonicalClass, url: identity.url, elapsedMs: Date.now() - state.startedAt,
      ...state.player,
    }, state.traceId);
    clearPending(state);
  };
  const beginClipboardWindow = async (event, video, traceId) => {
    clearPending();
    const state = { traceId, before: '', startedAt: Date.now(), until: Date.now() + 8000,
      player: globalThis.ADM_DIAG?.player?.(video) || {}, timer: null, expiry: null, readFailureLogged: false };
    // Publish the association before awaiting clipboard permission so the
    // following native contextmenu event keeps the same trace.
    pending = state;
    let before = '';
    try { before = await navigator.clipboard.readText(); } catch {}
    if (pending !== state) return;
    state.before = before;
    state.timer = setInterval(() => void inspectClipboard(state), 250);
    state.expiry = setTimeout(() => {
      if (pending === state) void globalThis.ADM_DIAG?.emit?.('media_replay.clipboard_probe', {
        result: 'unchanged', reason: 'canonical_media_link_not_observed', durationMs: 8000, ...state.player,
      }, traceId, 'WARN');
      clearPending(state);
    }, 8050);
    void globalThis.ADM_DIAG?.emit?.('media_replay.clipboard_window', {
      result: 'started', durationMs: 8000, platform: supportedHost(location.hostname) ? 'social' : 'other', ...state.player,
    }, traceId);
  };

  document.addEventListener('pointerdown', event => {
    if (!globalThis.ADM_DIAG?.active?.() || editable(event.target)) return;
    const video = visibleVideo(event.clientX, event.clientY, event.target);
    const relevant = Boolean(video || event.target?.closest?.('button,a,[role="button"],[role="menuitem"],[data-apocalipse-player-id]'));
    if (!relevant) return;
    const traceId = crypto.randomUUID();
    describe(event, 'pointer_down', traceId, video);
    if (event.button === 2 && video) void beginClipboardWindow(event, video, traceId);
  }, true);
  for (const type of ['click', 'dblclick', 'auxclick']) {
    document.addEventListener(type, event => {
      if (!globalThis.ADM_DIAG?.active?.() || editable(event.target)) return;
      const video = visibleVideo(event.clientX, event.clientY, event.target);
      const relevant = Boolean(video || event.target?.closest?.('button,a,[role="button"],[role="menuitem"],[data-apocalipse-player-id]'));
      if (relevant) describe(event, 'activation', crypto.randomUUID(), video);
    }, true);
  }
  document.addEventListener('contextmenu', event => {
    if (!globalThis.ADM_DIAG?.active?.() || editable(event.target)) return;
    const video = visibleVideo(event.clientX, event.clientY, event.target);
    if (video) describe(event, 'native_menu_opened', pending?.traceId || crypto.randomUUID(), video);
  }, true);
  addEventListener('focus', () => { if (pending) void inspectClipboard(pending); }, true);
  document.addEventListener('visibilitychange', () => { if (!document.hidden && pending) void inspectClipboard(pending); }, true);
})();
