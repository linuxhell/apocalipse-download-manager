// One resolver for the popup scan and both overlay handlers. Never infer a video
// from the first entry in a feed, a caption, a nearby button, or stale page state.
(() => {
  globalThis.ADM_DIAG?.register("tiktok-identity.js");
  const reasons = new WeakMap();
  const explain = (video, reason, value) => { reasons.set(video, { reason, resolved: Boolean(value) }); return value; };
  const buttons = new WeakMap(), pageBindings = new WeakMap(), resolvedBindings = new WeakMap();
  const boundVideoProperty = Symbol.for('apocalipse.tiktok.boundVideo');
  const validUrl = (value) => {
    try {
      const url = new URL(String(value || '').replaceAll('\\/', '/'), location.href);
      const match = url.pathname.match(/^\/@([A-Za-z0-9._-]+)\/video\/(\d+)\/?$/);
      return /(^|\.)tiktok\.com$/i.test(url.hostname) && match
        ? `https://www.tiktok.com/@${match[1]}/video/${match[2]}` : null;
    } catch { return null; }
  };
  const mediaKey = (value) => {
    try {
      const url = new URL(value);
      if (!/^https?:$/.test(url.protocol)) return null;
      // The play endpoint identifies its asset in video_id, not in the path.
      const id = url.searchParams.get('video_id');
      if (id) return `video_id:${id}`;
      if (!/\/video\/tos\//i.test(url.pathname)) return url.href;
      return `${url.hostname}${url.pathname}`;
    } catch { return null; }
  };
  const scopesFor = (video) => {
    const scopes = [];
    for (let node = video, depth = 0; node && depth < 18; node = node.parentElement, depth++) {
      if (node === document.body || node === document.documentElement || /^(BODY|HTML)$/.test(node.tagName || '')) break;
      if ([...(node.querySelectorAll?.('video') || [])].some(other => other !== video)) break;
      scopes.push(node);
      if (node !== video && node.matches?.('article,[data-e2e="recommend-list-item-container"],[data-e2e="feed-video"]')) break;
    }
    return scopes;
  };
  const unique = (values) => {
    const urls = [...new Set(values.filter(Boolean))];
    const ids = new Set(urls.map(url => url.match(/\/video\/(\d+)/)?.[1]));
    return ids.size === 1 ? urls[0] : null;
  };
  const containsSource = (root, source) => {
    if (!source) return false;
    const queue = [root], seen = new WeakSet();
    for (let cursor = 0; cursor < queue.length && cursor < 2000; cursor++) {
      const value = queue[cursor];
      if (typeof value === 'string') {
        if (mediaKey(value) === source) return true;
      } else if (value && typeof value === 'object' && !seen.has(value)) {
        seen.add(value);
        queue.push(...Object.values(value));
      }
    }
    return false;
  };
  const records = (roots) => {
    const found = [], queue = [...roots], seen = new WeakSet();
    for (let cursor = 0; cursor < queue.length && cursor < 14000; cursor++) {
      const value = queue[cursor];
      if (!value || typeof value !== 'object' || seen.has(value) || value.nodeType) continue;
      seen.add(value);
      const rawId = value.id || value.itemId || value.aweme_id || value.awemeId || '';
      const id = typeof rawId === 'number' && !Number.isSafeInteger(rawId) ? '' : String(rawId);
      const author = value.author?.uniqueId || value.author?.unique_id || value.authorInfo?.uniqueId;
      const url = /^\d+$/.test(id) && author ? validUrl(`https://www.tiktok.com/@${author}/video/${id}`) : null;
      if (url) found.push({ id, url, media: value.video || value.videoInfo || null });
      // Fiber return/sibling links escape the clicked card. Do not traverse them.
      for (const [key, child] of Object.entries(value)) {
        if (!['return', 'sibling', '_owner', 'alternate', 'stateNode'].includes(key) && child && typeof child === 'object') queue.push(child);
      }
    }
    return found;
  };
  const frameAncestorUrl = () => {
    if (typeof window === 'undefined') return null;
    let currentWindow = window;
    for (let frameDepth = 0; frameDepth < 4; frameDepth += 1) {
      let frame = null, parentDocument = null, parentWindow = null;
      try {
        if (currentWindow === currentWindow.top) break;
        frame = currentWindow.frameElement;
        parentWindow = currentWindow.parent;
        parentDocument = parentWindow.document;
      } catch { break; }
      if (!frame || !parentDocument) break;
      const anchors = [], explicitIds = new Set(), roots = [];
      let node = frame;
      for (let depth = 0; node && depth < 14; depth += 1, node = node.parentElement) {
        if (node === parentDocument.body || node === parentDocument.documentElement) break;
        const siblingFrames = [...(node.querySelectorAll?.('iframe') || [])]
          .filter(other => other !== frame && other.isConnected);
        const siblingVideos = [...(node.querySelectorAll?.('video') || [])].filter(other => other.isConnected);
        if (depth > 0 && (siblingFrames.length || siblingVideos.length)) break;
        for (const name of ['data-video-id', 'data-item-id', 'data-aweme-id']) {
          const id = node.getAttribute?.(name);
          if (id && /^\d+$/.test(id)) explicitIds.add(id);
        }
        for (const anchor of node.querySelectorAll?.('a[href*="/video/"]') || []) anchors.push(validUrl(anchor.href));
        for (const key of Object.getOwnPropertyNames(node)) {
          if (/^__(?:reactProps|reactFiber|vue)/i.test(key)) roots.push(node[key]);
        }
        const direct = unique(anchors);
        if (direct) return direct;
        if (node.matches?.('article,[data-e2e="recommend-list-item-container"],[data-e2e="feed-video"],[data-e2e="browse-video"]')) break;
      }
      const scopedRecords = records(roots);
      if (explicitIds.size === 1) {
        const id = [...explicitIds][0];
        const exact = unique([...anchors, ...scopedRecords.map(record => record.url)]
          .filter(url => url?.endsWith(`/video/${id}`)));
        if (exact) return exact;
      } else if (explicitIds.size === 0) {
        const scoped = unique([...anchors, ...scopedRecords.map(record => record.url)]);
        if (scoped) return scoped;
      }
      currentWindow = parentWindow;
    }
    return null;
  };
  const resolveCandidate = (video, allowPage = true) => {
    if (!video || !/(^|\.)tiktok\.com$/i.test(location.hostname)) return null;
    reasons.delete(video);
    const scopes = scopesFor(video), anchors = [], explicitIds = new Set(), roots = [];
    for (const node of scopes) {
      for (const name of ['data-video-id', 'data-item-id', 'data-aweme-id']) {
        const id = node.getAttribute?.(name);
        if (id && /^\d+$/.test(id)) explicitIds.add(id);
      }
      for (const anchor of node.querySelectorAll?.('a[href*="/video/"]') || []) anchors.push(validUrl(anchor.href));
      for (const key of Object.getOwnPropertyNames(node)) {
        if (/^__(?:reactProps|reactFiber|vue)/i.test(key)) roots.push(node[key]);
      }
    }
    const scopedRecords = records(roots);
    const source = mediaKey(video.currentSrc || video.src || '');
    let matched = scopedRecords.filter(record => containsSource(record.media, source));
    if (!matched.length && source) {
      const globalRoots = [];
      for (const script of document.querySelectorAll('script[type="application/json"],script[id*="DATA"],script[id*="STATE"]')) {
        if ((script.textContent || '').length > 15000000) continue;
        try { globalRoots.push(JSON.parse(script.textContent)); } catch {}
      }
      matched = records(globalRoots).filter(record => containsSource(record.media, source));
    }
    if (matched.length) return explain(video, "matched_media_source", unique(matched.map(record => record.url)));
    if (explicitIds.size === 1) {
      const id = [...explicitIds][0];
      return unique([...anchors, ...scopedRecords.map(record => record.url)].filter(url => url?.endsWith(`/video/${id}`)));
    }
    if (explicitIds.size > 1) return explain(video, "conflicting_explicit_ids", null);
    // A unique permalink in this card is authoritative; multiple links are not.
    if (anchors.some(Boolean)) return explain(video, "scoped_card_links", unique(anchors));
    // Props are only usable without a source match when scoped to an actual card.
    if (scopes.some(node => node.matches?.('article,[data-e2e="recommend-list-item-container"],[data-e2e="feed-video"]'))) {
      const scoped = unique(scopedRecords.map(record => record.url));
      if (scopedRecords.length) return explain(video, "scoped_framework_records", scoped);
    }
    const framed = frameAncestorUrl();
    if (framed) return explain(video, "parent_frame_candidate", framed);
    // Dedicated pages with one player remain supported. Never use the address
    // bar for a multi-player feed whose URL can lag behind scrolling.
    const videos = [...document.querySelectorAll('video')];
    const page = validUrl(location.href);
    if (!allowPage || !page || videos.length !== 1 || videos[0] !== video) return explain(video, "no_bound_permalink", null);
    const sourceIdentity = mediaKey(video.currentSrc || video.src || '') || String(video.currentSrc || video.src || '');
    const previous = pageBindings.get(video);
    if (previous?.page === page && previous.source !== sourceIdentity) return null;
    pageBindings.set(video, { page, source: sourceIdentity });
    return page;
  };
  const resolveLocal = (video, allowPage = true) => {
    const url = resolveCandidate(video, allowPage);
    void globalThis.ADM_DIAG?.emit("identity.local_decision", { ...(reasons.get(video) || { reason: "explicit_id_or_page" }),
      resolved: Boolean(url), ...(video && globalThis.ADM_DIAG ? globalThis.ADM_DIAG.player(video) : {}) });
    if (!url) return null;
    const source = mediaKey(video.currentSrc || video.src || '') || String(video.currentSrc || video.src || '');
    const previous = resolvedBindings.get(video);
    // Virtualized feeds update the player before its DOM metadata. Do not use
    // the previous card's unchanged link for a different media source.
    if (previous?.url === url && previous.source !== source && previous.page === location.href) return null;
    resolvedBindings.set(video, { url, source, page: location.href });
    return url;
  };
  globalThis.ApocalipseTikTokIdentity = {
    resolve(video) {
      // The MAIN-world reader can see framework props that ISOLATED scripts
      // cannot. Exchange only strings on the selected element, never credentials.
      if (video?.dispatchEvent && globalThis.Event) {
        const source = String(video.currentSrc || video.src || '');
        try {
          video.removeAttribute('data-apocalipse-current-permalink');
          video.removeAttribute('data-apocalipse-identity-diagnostic');
          video.dispatchEvent(new Event('apocalipse-tiktok-identity-request', { bubbles: true }));
          const evidence = video.getAttribute('data-apocalipse-identity-diagnostic');
          try { void globalThis.ADM_DIAG?.emit('identity.main_probe', { mainReady: Boolean(evidence),
            ...(evidence && evidence.length < 500 ? JSON.parse(evidence) : {}) }); } catch {}
          video.removeAttribute('data-apocalipse-identity-diagnostic');
          const candidate = validUrl(video.getAttribute('data-apocalipse-current-permalink'));
          if (candidate && source === String(video.currentSrc || video.src || '')) return candidate;
        } catch {} finally { video.removeAttribute('data-apocalipse-current-permalink'); video.removeAttribute('data-apocalipse-identity-diagnostic'); }
      }
      return resolveLocal(video);
    },
    resolveLocal, scopesFor, validUrl, frameAncestorUrl,
    diagnosticState(video) { return reasons.get(video) || { reason: "not_evaluated" }; },
    bind(button, video) {
      buttons.set(button, video);
      // Keep the exact player on the DOM button too. TikTok can re-run an
      // isolated content-script realm while preserving injected DOM nodes;
      // a new WeakMap must still be able to recover the original binding.
      try { button[boundVideoProperty] = video; } catch {}
    },
    videoFor(button) {
      const video = buttons.get(button) || button?.[boundVideoProperty];
      if (!video?.isConnected) return null;
      if (!buttons.has(button)) buttons.set(button, video);
      return video;
    },
  };
})();
