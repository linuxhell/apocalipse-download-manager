// One resolver for the popup scan and both overlay handlers. Never infer a video
// from the first entry in a feed, a caption, a nearby button, or stale page state.
(() => {
  globalThis.ADM_DIAG?.register("tiktok-identity.js");
  const reasons = new WeakMap();
  const explain = (video, reason, value, detail = {}) => {
    reasons.set(video, { reason, resolved: Boolean(value), ...detail });
    return value;
  };
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
  const videoVisible = (video) => {
    if (!video?.isConnected || video.hidden || video.getAttribute?.('aria-hidden') === 'true') return false;
    let rect; try { rect = video.getBoundingClientRect?.(); } catch { return false; }
    if (!rect || rect.width < 40 || rect.height < 20 || rect.bottom <= 0 || rect.right <= 0
      || rect.top >= (globalThis.innerHeight || 0) || rect.left >= (globalThis.innerWidth || 0)) return false;
    let style; try { style = globalThis.getComputedStyle?.(video); } catch {}
    return style?.display !== 'none' && style?.visibility !== 'hidden' && Number.parseFloat(style?.opacity ?? '1') > 0.01;
  };
  const scopesFor = (video) => {
    const scopes = [];
    for (let node = video, depth = 0; node && depth < 18; node = node.parentElement, depth++) {
      if (node === document.body || node === document.documentElement || /^(BODY|HTML)$/.test(node.tagName || '')) break;
      // TikTok keeps the next item as a hidden/preloaded <video>. Only another
      // actually visible player is a boundary; hidden preload players must not
      // prevent us from reaching the current card/framework identity.
      if ([...(node.querySelectorAll?.('video') || [])].some(other => other !== video && videoVisible(other))) break;
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
  const networkRecords = [];
  const records = (roots, followFiberParents = false) => {
    const found = [], queue = [...roots], seen = new WeakSet();
    for (let cursor = 0; cursor < queue.length && cursor < 14000; cursor++) {
      const value = queue[cursor];
      if (!value || typeof value !== 'object' || seen.has(value) || value.nodeType) continue;
      seen.add(value);
      const rawId = value.id || value.itemId || value.aweme_id || value.awemeId || '';
      const id = typeof rawId === 'number' && !Number.isSafeInteger(rawId) ? '' : String(rawId);
      const author = value.author?.uniqueId || value.author?.unique_id || value.authorInfo?.uniqueId;
      const url = /^\d+$/.test(id) && author ? validUrl(`https://www.tiktok.com/@${author}/video/${id}`) : null;
      if (url) {
        const media = value.video || value.videoInfo || null;
        found.push({ id, url, media,
          duration: Number(media?.duration || value.duration || 0),
          width: Number(media?.width || 0), height: Number(media?.height || 0) });
      }
      // Parent Fiber links are used only by the caller that requires one unique
      // identity. The normal fallback remains confined to the clicked card.
      for (const [key, child] of Object.entries(value)) {
        if (!['sibling', '_owner', 'alternate', 'stateNode'].includes(key)
          && (followFiberParents || key !== 'return') && child && typeof child === 'object') queue.push(child);
      }
    }
    return found;
  };
  const rememberNetworkPayload = payload => {
    for (const record of records([payload])) {
      const previous = networkRecords.findIndex(item => item.id === record.id);
      if (previous >= 0) networkRecords.splice(previous, 1);
      networkRecords.push({ ...record, seenAt: Date.now() });
    }
    while (networkRecords.length > 240) networkRecords.shift();
  };
  const inspectResponse = response => {
    try {
      const type = String(response?.headers?.get?.('content-type') || '');
      if (!/json/i.test(type)) return;
      void response.clone().json().then(rememberNetworkPayload).catch(() => {});
    } catch {}
  };
  // TikTok's current feed can render only Blob players and omit every permalink
  // and React item prop from the DOM. Preserve the identities from the site's
  // own feed response before that information is discarded by the renderer.
  if (typeof globalThis.fetch === 'function' && !globalThis.__apocalipseTikTokFetchIdentity) {
    globalThis.__apocalipseTikTokFetchIdentity = true;
    const nativeFetch = globalThis.fetch;
    globalThis.fetch = async function(...args) {
      const response = await nativeFetch.apply(this, args);
      inspectResponse(response);
      return response;
    };
  }
  if (typeof globalThis.XMLHttpRequest === 'function' && !globalThis.__apocalipseTikTokXhrIdentity) {
    globalThis.__apocalipseTikTokXhrIdentity = true;
    const nativeOpen = globalThis.XMLHttpRequest.prototype.open;
    globalThis.XMLHttpRequest.prototype.open = function(...args) {
      this.addEventListener?.('load', () => {
        try {
          const payload = this.responseType === 'json' ? this.response
            : JSON.parse(String(this.responseText || ''));
          rememberNetworkPayload(payload);
        } catch {}
      }, { once: true });
      return nativeOpen.apply(this, args);
    };
  }
  const networkCandidateFor = video => {
    const duration = Number(video?.duration || 0), width = Number(video?.videoWidth || 0), height = Number(video?.videoHeight || 0);
    const fresh = networkRecords.filter(record => Date.now() - record.seenAt < 10 * 60 * 1000);
    const sized = fresh.filter(record => record.width > 0 && record.height > 0
      && ((record.width === width && record.height === height) || (record.width === height && record.height === width)));
    const matched = (sized.length ? sized : fresh).filter(record => record.duration > 0 && duration > 0
      && Math.abs(record.duration - duration) < 1.25);
    const url = unique(matched.map(record => record.url));
    return { url, inspected: fresh.length, matched: matched.length };
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
        const siblingVideos = [...(node.querySelectorAll?.('video') || [])].filter(other => other.isConnected && videoVisible(other));
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
    const parentRecords = records(roots, true);
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
    const network = networkCandidateFor(video);
    if (network.url) return explain(video, "matched_feed_response", network.url,
      { networkRecordCount: network.inspected, networkMatchCount: network.matched });
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
    const parentIds = [...new Set(parentRecords.map(record => record.id).filter(Boolean))];
    const parentScoped = unique(parentRecords.map(record => record.url));
    if (parentScoped) return explain(video, "unique_parent_framework_record", parentScoped);
    const framed = frameAncestorUrl();
    if (framed) return explain(video, "parent_frame_candidate", framed);
    // Dedicated pages with one player remain supported. Never use the address
    // bar for a multi-player feed whose URL can lag behind scrolling.
    const videos = [...document.querySelectorAll('video')];
    const page = validUrl(location.href);
    if (!allowPage || !page || videos.length !== 1 || videos[0] !== video) return explain(video, "no_bound_permalink", null, {
      scopeCount: scopes.length, scopedRecordCount: scopedRecords.length,
      parentRecordCount: parentRecords.length, parentDistinctIds: parentIds.length,
      explicitIdCount: explicitIds.size, anchorCount: anchors.filter(Boolean).length,
      sourceIdentityPresent: Boolean(source),
      networkRecordCount: network.inspected, networkMatchCount: network.matched,
    });
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
