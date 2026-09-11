// 0.3.127 test guard: keep one logical media row per social post/reel and
// suppress anonymous adaptive CDN tracks from the popup without touching the
// download engine. Real DOM audio/images remain visible in their own tabs.
(() => {
  if (globalThis.ADM_POPUP_MEDIA_GUARD) return;

  const socialHost = value => {
    try { return /(^|\.)(?:facebook|instagram|tiktok)\.com$/i.test(new URL(value).hostname); }
    catch { return false; }
  };

  const socialIdentity = value => {
    try {
      const url = new URL(value);
      const host = url.hostname.toLowerCase();
      if (/(^|\.)tiktok\.com$/.test(host)) {
        const id = url.pathname.match(/^\/@[^/]+\/video\/(\d+)/i)?.[1];
        return id ? `tiktok:${id}` : null;
      }
      if (/(^|\.)instagram\.com$/.test(host)) {
        const code = url.pathname.match(/^\/(?:reel|reels|p)\/([^/?#]+)/i)?.[1];
        return code ? `instagram:${code}` : null;
      }
      if (/(^|\.)facebook\.com$/.test(host)) {
        const id = url.pathname.match(/\/(?:reel|reels|videos|posts)\/([^/?#]+)/i)?.[1]
          || url.searchParams.get('v') || url.searchParams.get('story_fbid') || url.searchParams.get('fbid')
          || url.pathname.match(/\/share\/[rv]\/([^/?#]+)/i)?.[1];
        return id ? `facebook:${id}` : null;
      }
    } catch {}
    return null;
  };

  const itemIdentity = item => item?.kind === 'video'
    ? socialIdentity(item.extractorUrl || item.url || '')
    : null;

  const quality = item => (item?.pageExtractor ? 32 : 0) + (item?.playerBound ? 16 : 0)
    + (item?.thumbnail ? 8 : 0) + (item?.recommended ? 4 : 0) + (!item?.retained ? 2 : 0);

  const mergeBest = (a, b) => {
    const preferred = quality(b) >= quality(a) ? b : a;
    const fallback = preferred === b ? a : b;
    return {
      ...fallback,
      ...preferred,
      thumbnail: preferred.thumbnail || fallback.thumbnail || '',
      title: preferred.title || fallback.title,
      previewUrl: preferred.previewUrl || fallback.previewUrl,
      audioUrl: preferred.audioUrl || fallback.audioUrl,
      duration: preferred.duration || fallback.duration || null,
      size: preferred.size || fallback.size || null,
    };
  };

  function cleanScanned(items, pageUrl = '') {
    const source = Array.isArray(items) ? items : [];
    const logical = new Map();
    for (const item of source) {
      if (!item?.url) continue;
      const social = socialHost(pageUrl) || socialHost(item.url) || socialHost(item.extractorUrl || '') || socialHost(item.playerPageUrl || '');
      // An unresolved visual player with no proven thumbnail is not a media row.
      if (social && item.kind === 'video' && item.visualOnly && !item.thumbnail) continue;
      // A social extractor URL without a thumbnail is kept internal until the
      // next refresh proves its visual identity; showing it would be an alien.
      if (social && item.kind === 'video' && item.pageExtractor && !item.thumbnail && !item.visualOnly) continue;
      const identity = itemIdentity(item);
      const key = `${item.kind || 'unknown'}:${identity || item.url}`;
      const previous = logical.get(key);
      logical.set(key, previous ? mergeBest(previous, item) : item);
    }
    return [...logical.values()];
  }

  const observedUrls = items => {
    const result = new Set();
    const add = value => { if (/^https?:/i.test(value || '')) result.add(value); };
    for (const item of items || []) {
      add(item?.url); add(item?.previewUrl); add(item?.audioUrl);
      for (const url of item?.requestUrls || []) add(url);
    }
    return result;
  };

  const obviousStandaloneAudio = item => {
    try {
      const url = new URL(item?.url || '');
      return /\.(?:mp3|wav|flac|ogg|opus|aac)(?:$|[?#])/i.test(url.href);
    } catch { return false; }
  };

  function filterNetwork(items, state) {
    const list = Array.isArray(items) ? items : [];
    if (!state) return list;
    const scanned = state.scanned || [];
    const observed = observedUrls(scanned);
    const social = socialHost(state.pageUrl || '') || scanned.some(item => socialHost(item.url) || socialHost(item.extractorUrl || '') || socialHost(item.playerPageUrl || ''));
    const hasLogicalVideo = scanned.some(item => item.kind === 'video' && (itemIdentity(item) || item.pageExtractor));
    const hasMsePlayer = scanned.some(item => item.kind === 'video' && item.visualOnly);
    if (!social && !hasMsePlayer && !hasLogicalVideo) return list;
    return list.filter(item => {
      if (!item?.url) return false;
      if (!/^(?:video|audio)\//i.test(item.contentType || '') && !/\.(?:mp4|m4a|webm|m3u8|mpd)(?:$|[?#])/i.test(item.url)) return true;
      if (observed.has(item.url)) return true;
      if ((item.contentType || '').toLowerCase().startsWith('audio/') && obviousStandaloneAudio(item)) return true;
      return false;
    });
  }

  const tabStates = new Map();
  const originalTabsSend = chrome.tabs.sendMessage.bind(chrome.tabs);
  const originalRuntimeSend = chrome.runtime.sendMessage.bind(chrome.runtime);

  chrome.tabs.sendMessage = function(tabId, message, options, callback) {
    let sendOptions = options, done = callback;
    if (typeof options === 'function') { done = options; sendOptions = undefined; }
    if (message?.type !== 'APOCALIPSE_SCAN' || typeof done !== 'function') {
      return sendOptions === undefined
        ? originalTabsSend(tabId, message, done)
        : originalTabsSend(tabId, message, sendOptions, done);
    }
    const wrapped = response => {
      const finish = pageUrl => {
        const scanned = cleanScanned(response?.media || [], pageUrl || '');
        tabStates.set(tabId, { pageUrl: pageUrl || '', scanned, at: Date.now() });
        done(response ? { ...response, media: scanned } : response);
      };
      try {
        const maybe = chrome.tabs.get(tabId);
        if (maybe?.then) maybe.then(tab => finish(tab?.url || '')).catch(() => finish(''));
        else finish('');
      } catch { finish(''); }
    };
    return sendOptions === undefined
      ? originalTabsSend(tabId, message, wrapped)
      : originalTabsSend(tabId, message, sendOptions, wrapped);
  };

  chrome.runtime.sendMessage = function(message, callback) {
    if (message?.type !== 'APOCALIPSE_RECENT_TAB_MEDIA' || typeof callback !== 'function') {
      return originalRuntimeSend(message, callback);
    }
    return originalRuntimeSend(message, response => {
      const state = tabStates.get(message.tabId);
      if (!response?.media || !state) return callback(response);
      callback({ ...response, media: filterNetwork(response.media, state) });
    });
  };

  globalThis.ADM_POPUP_MEDIA_GUARD = { cleanScanned, filterNetwork, socialIdentity };
})();
