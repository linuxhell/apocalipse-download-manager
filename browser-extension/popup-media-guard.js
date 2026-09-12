// 0.3.129 test guard: keep one logical media row per social post/reel and
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
      // Recording-only players stay available through the button placed over
      // the page, but must not become false Preview/Download rows.
      if (item.kind === 'video' && item.recordingOnly) continue;
      const social = socialHost(pageUrl) || socialHost(item.url) || socialHost(item.extractorUrl || '') || socialHost(item.playerPageUrl || '');
      // Keep only the exact currently visible unresolved social player. Hidden
      // preload players are implementation details and must not become rows.
      if (social && item.kind === 'video' && item.visualOnly
        && (!item.thumbnail || !item.playerBound || !item.recommended || item.retained)) continue;
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

  const rememberScan = async (tabId, response) => {
    let pageUrl = '';
    try { pageUrl = (await chrome.tabs.get(tabId))?.url || ''; } catch {}
    const scanned = cleanScanned(response?.media || [], pageUrl);
    tabStates.set(tabId, { pageUrl, scanned, at: Date.now() });
    return response ? { ...response, media: scanned } : response;
  };

  chrome.tabs.sendMessage = function(tabId, message, options, callback) {
    let sendOptions = options, done = callback;
    if (typeof options === 'function') { done = options; sendOptions = undefined; }
    const isScan = message?.type === 'APOCALIPSE_SCAN';
    if (!isScan) {
      return sendOptions === undefined
        ? originalTabsSend(tabId, message, done)
        : originalTabsSend(tabId, message, sendOptions, done);
    }
    if (typeof done === 'function') {
      const wrapped = response => { void rememberScan(tabId, response).then(done); };
      return sendOptions === undefined
        ? originalTabsSend(tabId, message, wrapped)
        : originalTabsSend(tabId, message, sendOptions, wrapped);
    }
    const pending = sendOptions === undefined
      ? originalTabsSend(tabId, message)
      : originalTabsSend(tabId, message, sendOptions);
    return pending?.then ? pending.then(response => rememberScan(tabId, response)) : pending;
  };

  const filterRecentResponse = (message, response) => {
    const state = tabStates.get(message?.tabId);
    if (!response?.media || !state) return response;
    const filtered = filterNetwork(response.media, state);
    if (filtered.length !== response.media.length) {
      void globalThis.ADM_DIAG?.emit('popup.guard_network_filtered', {
        before: response.media.length,
        after: filtered.length,
        removed: response.media.length - filtered.length,
        social: socialHost(state.pageUrl || ''),
      });
    }
    return { ...response, media: filtered };
  };

  chrome.runtime.sendMessage = function(message, callback) {
    const isRecent = message?.type === 'APOCALIPSE_RECENT_TAB_MEDIA';
    if (!isRecent) return originalRuntimeSend(message, callback);
    if (typeof callback === 'function') {
      return originalRuntimeSend(message, response => callback(filterRecentResponse(message, response)));
    }
    const pending = originalRuntimeSend(message);
    return pending?.then ? pending.then(response => filterRecentResponse(message, response)) : pending;
  };

  globalThis.ADM_POPUP_MEDIA_GUARD = { cleanScanned, filterNetwork, socialIdentity, tabStates };
})();
