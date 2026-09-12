// Resolve the exact currently visible Facebook/TikTok player only when the user
// asks to Preview/Download an unresolved Blob/MediaStream row. Hidden preload
// players are ignored; ambiguous visible players remain unresolved.
(() => {
  if (globalThis.ADM_SOCIAL_PLAYER_RESOLVER) return;
  globalThis.ADM_DIAG?.register?.('social-player-resolution.js');

  const rectFor = element => {
    try { return element?.getBoundingClientRect?.() || null; } catch { return null; }
  };

  const cssVisible = element => {
    if (!element || element.isConnected === false || element.hidden || element.getAttribute?.('aria-hidden') === 'true') return false;
    const rect = rectFor(element);
    if (!rect || rect.width < 40 || rect.height < 20) return false;
    const vh = Number(globalThis.innerHeight) || document.documentElement?.clientHeight || 0;
    const vw = Number(globalThis.innerWidth) || document.documentElement?.clientWidth || 0;
    if (!(vh > 0) || !(vw > 0) || rect.bottom <= 0 || rect.right <= 0 || rect.top >= vh || rect.left >= vw) return false;
    let style = null;
    try { style = globalThis.getComputedStyle?.(element) || null; } catch {}
    return style?.display !== 'none' && style?.visibility !== 'hidden' && style?.visibility !== 'collapse'
      && Number.parseFloat(style?.opacity ?? '1') > 0.01;
  };

  const rectDistance = (a, b) => {
    if (!a || !b) return Infinity;
    return Math.abs(a.left - b.left) + Math.abs(a.top - b.top)
      + Math.abs(a.width - b.width) + Math.abs(a.height - b.height);
  };

  const findExactVisibleVideo = request => {
    if (!request?.rect) return null;
    const candidates = [...document.querySelectorAll('video')]
      .filter(cssVisible)
      .map(video => ({ video, score: rectDistance(rectFor(video), request.rect) }))
      .filter(item => item.score <= 12)
      .sort((a, b) => a.score - b.score);
    if (!candidates[0]) return null;
    if (candidates[1] && candidates[1].score - candidates[0].score < 3) return null;
    const video = candidates[0].video;
    if (Number.isFinite(request.duration) && request.duration > 0
      && Number.isFinite(video.duration) && video.duration > 0
      && Math.abs(video.duration - request.duration) > 0.75) return null;
    return video;
  };

  const competingVisibleVideo = (other, current) => other !== current && cssVisible(other);

  const scopedNodes = video => {
    const result = [];
    for (let node = video, depth = 0; node && depth < 20; node = node.parentElement, depth += 1) {
      if (node === document.body || node === document.documentElement || /^(BODY|HTML)$/.test(node.tagName || '')) break;
      if ([...(node.querySelectorAll?.('video') || [])].some(other => competingVisibleVideo(other, video))) break;
      result.push(node);
      if (node !== video && node.matches?.('article,[role="article"],[data-e2e="recommend-list-item-container"],[data-e2e="feed-video"],[data-e2e="browse-video"]')) break;
    }
    return result;
  };

  const canonicalFacebook = value => {
    try {
      const url = new URL(value, location.href);
      if (!/(^|\.)facebook\.com$/i.test(url.hostname)) return null;
      if (/\/(?:photo|photos)(?:\.php|\/|$)/i.test(url.pathname)) return null;
      return /(?:^|\/)(?:reel|reels|watch|videos|posts|share)(?:\/|$)/i.test(url.pathname)
        || /\/(?:permalink|story)\.php$/i.test(url.pathname)
        || url.searchParams.has('v') || url.searchParams.has('story_fbid') || url.searchParams.has('fbid')
        ? url.href : null;
    } catch { return null; }
  };

  const facebookFromDom = video => {
    const page = canonicalFacebook(location.href);
    if (page && [...document.querySelectorAll('video')].filter(cssVisible).length === 1) return page;
    const selector = [
      'a[href*="/reel/"]', 'a[href*="/reels/"]', 'a[href*="/videos/"]', 'a[href*="/posts/"]',
      'a[href*="/watch/"]', 'a[href*="/watch?"]', 'a[href*="/permalink.php"]', 'a[href*="/story.php"]',
      'a[href*="/share/r/"]', 'a[href*="/share/v/"]',
    ].join(',');
    for (const node of scopedNodes(video)) {
      const urls = [...(node.querySelectorAll?.(selector) || [])].map(anchor => canonicalFacebook(anchor.href)).filter(Boolean);
      const unique = [...new Set(urls)];
      if (unique.length === 1) return unique[0];
      if (unique.length > 1) return null;
      const path = String(node.innerHTML || '').replaceAll('\\/', '/')
        .match(/\/(?:reel|reels|videos|posts|share\/[rv])\/[A-Za-z0-9._-]+/i)?.[0];
      const markupUrl = path && canonicalFacebook(path);
      if (markupUrl) return markupUrl;
    }
    return null;
  };

  const facebookFromMenu = async video => {
    const videoRect = rectFor(video);
    const post = video.closest?.('[role="article"],article') || video.parentElement;
    if (!videoRect || !post) return null;
    let container = post;
    for (let depth = 0; container?.parentElement && depth < 6; depth += 1) {
      const rect = rectFor(container);
      if (rect && rect.top <= videoRect.top - 20 && rect.right >= videoRect.right - 20) break;
      container = container.parentElement;
    }
    const buttons = [...(container?.querySelectorAll?.('button,[role="button"]') || [])];
    const labeled = buttons.filter(button => {
      const label = `${button.getAttribute?.('aria-label') || ''} ${button.title || ''} ${button.textContent || ''}`.trim();
      return /(?:ações|acoes|opções|opcoes|actions|options|more|menu|更多|更多选项)/i.test(label) || /^\s*(?:\.\.\.|…|⋯)\s*$/.test(label);
    });
    const candidates = labeled.length ? labeled : buttons;
    const menuButton = candidates.filter(button => {
      const rect = rectFor(button);
      return rect && rect.width > 0 && rect.height > 0 && rect.top < videoRect.top + 100;
    }).sort((left, right) => {
      const a = rectFor(left), b = rectFor(right);
      const score = rect => rect ? Math.abs(rect.right - videoRect.right) + Math.abs(rect.bottom - videoRect.top) : Infinity;
      return score(a) - score(b);
    })[0];
    if (!menuButton) return null;
    menuButton.click();
    let copyItem = null;
    for (let attempt = 0; attempt < 20 && !copyItem; attempt += 1) {
      await new Promise(resolve => setTimeout(resolve, 100));
      copyItem = [...document.querySelectorAll('[role="menuitem"],[role="menuitemradio"]')]
        .find(item => /(?:copiar link|copy link|复制链接|複製連結)/i.test(item.textContent || ''));
    }
    if (!copyItem) { try { menuButton.click(); } catch {} return null; }
    let before = '';
    try { before = await navigator.clipboard.readText(); } catch {}
    copyItem.click();
    for (let attempt = 0; attempt < 20; attempt += 1) {
      await new Promise(resolve => setTimeout(resolve, 100));
      try {
        const copied = await navigator.clipboard.readText();
        const canonical = copied && copied !== before ? canonicalFacebook(copied) : null;
        if (canonical) return canonical;
      } catch {}
    }
    return null;
  };

  const canonicalTikTok = value => {
    try {
      const url = new URL(String(value || '').replaceAll('\\/', '/'), location.href);
      const match = url.pathname.match(/^\/@([A-Za-z0-9._-]+)\/video\/(\d+)\/?$/);
      return /(^|\.)tiktok\.com$/i.test(url.hostname) && match
        ? `https://www.tiktok.com/@${match[1]}/video/${match[2]}` : null;
    } catch { return null; }
  };

  const uniqueTikTok = values => {
    const urls = [...new Set(values.map(canonicalTikTok).filter(Boolean))];
    const ids = new Set(urls.map(url => url.match(/\/video\/(\d+)/)?.[1]).filter(Boolean));
    return ids.size === 1 ? urls[0] : null;
  };

  const frameworkTikTokRecords = roots => {
    const found = [], queue = [...roots], seen = new WeakSet();
    for (let cursor = 0; cursor < queue.length && cursor < 12000; cursor += 1) {
      const value = queue[cursor];
      if (!value || typeof value !== 'object' || seen.has(value) || value.nodeType) continue;
      seen.add(value);
      const rawId = value.id || value.itemId || value.aweme_id || value.awemeId || '';
      const id = typeof rawId === 'number' && !Number.isSafeInteger(rawId) ? '' : String(rawId);
      const author = value.author?.uniqueId || value.author?.unique_id || value.authorInfo?.uniqueId;
      const url = /^\d+$/.test(id) && author ? canonicalTikTok(`https://www.tiktok.com/@${author}/video/${id}`) : null;
      if (url) found.push(url);
      for (const [key, child] of Object.entries(value)) {
        if (!['return', 'sibling', '_owner', 'alternate', 'stateNode'].includes(key) && child && typeof child === 'object') queue.push(child);
      }
    }
    return found;
  };

  const tiktokFromVisibleCard = video => {
    try {
      const existing = globalThis.ApocalipseTikTokIdentity?.resolve?.(video);
      if (existing) return canonicalTikTok(existing);
    } catch {}
    const anchors = [], explicitIds = new Set(), roots = [];
    for (const node of scopedNodes(video)) {
      for (const name of ['data-video-id', 'data-item-id', 'data-aweme-id']) {
        const id = node.getAttribute?.(name);
        if (id && /^\d+$/.test(id)) explicitIds.add(id);
      }
      for (const anchor of node.querySelectorAll?.('a[href*="/video/"]') || []) anchors.push(anchor.href);
      for (const key of Object.getOwnPropertyNames(node)) {
        if (/^__(?:reactProps|reactFiber|vue)/i.test(key)) roots.push(node[key]);
      }
    }
    const records = frameworkTikTokRecords(roots);
    if (explicitIds.size === 1) {
      const id = [...explicitIds][0];
      return uniqueTikTok([...anchors, ...records].filter(value => canonicalTikTok(value)?.endsWith(`/video/${id}`)));
    }
    if (explicitIds.size > 1) return null;
    return uniqueTikTok([...anchors, ...records]);
  };

  const stableDuring = async (video, operation) => {
    const beforePage = location.href;
    const beforeSource = String(video.currentSrc || video.src || '');
    const beforeRect = rectFor(video);
    let changed = false;
    const mark = () => { changed = true; };
    for (const event of ['emptied', 'loadstart', 'loadedmetadata']) video.addEventListener?.(event, mark, true);
    try {
      const value = await operation();
      const afterRect = rectFor(video);
      if (changed || !value || !video.isConnected || location.href !== beforePage
        || String(video.currentSrc || video.src || '') !== beforeSource
        || rectDistance(beforeRect, afterRect) > 12 || !cssVisible(video)) return null;
      return value;
    } finally {
      for (const event of ['emptied', 'loadstart', 'loadedmetadata']) video.removeEventListener?.(event, mark, true);
    }
  };

  const resolveRequest = async request => {
    if (request?.pageUrl && request.pageUrl !== location.href) return null;
    const video = findExactVisibleVideo(request);
    if (!video) return null;
    const host = location.hostname.toLowerCase();
    const url = await stableDuring(video, async () => {
      if (/(^|\.)facebook\.com$/.test(host)) return facebookFromDom(video) || await facebookFromMenu(video);
      if (/(^|\.)tiktok\.com$/.test(host)) {
        for (let attempt = 0; attempt < 10; attempt += 1) {
          const found = tiktokFromVisibleCard(video);
          if (found) return found;
          await new Promise(resolve => setTimeout(resolve, 80));
        }
      }
      return null;
    });
    if (!url) return null;
    const title = video.closest?.('article,[role="article"],[data-e2e="recommend-list-item-container"],[data-e2e="feed-video"]')?.innerText
      || document.title || 'Video';
    return {
      url,
      extractorUrl: url,
      kind: 'video',
      pageExtractor: true,
      visualOnly: false,
      recommended: true,
      ambiguousSocialTrack: false,
      title: String(title).trim().replace(/\s+/g, ' ').slice(0, 240),
      thumbnail: request.thumbnail || '',
      duration: Number.isFinite(video.duration) && video.duration > 0 ? video.duration : request.duration || null,
      size: null,
      ext: 'mp4',
    };
  };

  chrome.runtime.onMessage.addListener((message, _sender, reply) => {
    if (message?.type !== 'APOCALIPSE_RESOLVE_VISIBLE_SOCIAL_MEDIA') return;
    const traceId = globalThis.ADM_DIAG?.begin?.('player.resolve_requested', { source: 'social_player_resolution' }) || null;
    resolveRequest(message.request || {}).then(item => {
      void globalThis.ADM_DIAG?.emit?.('player.resolve_result', { resolved: Boolean(item), url: item?.url || '' }, traceId, item ? 'INFO' : 'WARN');
      reply(item ? { ok: true, item } : { ok: false, error: 'media_identity_unresolved' });
    }).catch(error => {
      void globalThis.ADM_DIAG?.emit?.('player.resolve_result', { resolved: false, errorRef: String(error) }, traceId, 'ERROR');
      reply({ ok: false, error: 'media_identity_unresolved' });
    });
    return true;
  });

  globalThis.ADM_SOCIAL_PLAYER_RESOLVER = {
    cssVisible, findExactVisibleVideo, scopedNodes, facebookFromDom, tiktokFromVisibleCard, resolveRequest,
  };
})();
