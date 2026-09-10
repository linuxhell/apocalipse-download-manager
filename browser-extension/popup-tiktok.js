(() => {
  const isTikTok = (value) => {
    try { return /(^|\.)tiktok\.com$/i.test(new URL(value).hostname); } catch { return false; }
  };

  async function collectTikTokFromAllFrames(tabId) {
    const results = await chrome.scripting.executeScript({
      target: { tabId, allFrames: true },
      func: () => {
        const valid = (value) => {
          try {
            const url = new URL(String(value || '').replaceAll('\\/', '/'), location.href);
            const match = url.pathname.match(/^\/@([A-Za-z0-9._-]+)\/video\/(\d+)\/?$/);
            return /(^|\.)tiktok\.com$/i.test(url.hostname) && match
              ? `https://www.tiktok.com/@${match[1]}/video/${match[2]}` : null;
          } catch { return null; }
        };
        const visibleScore = (element) => {
          const rect = element?.getBoundingClientRect?.();
          if (!rect || rect.width < 40 || rect.height < 20 || rect.bottom <= 0 || rect.top >= innerHeight) return Infinity;
          const center = (rect.top + rect.bottom) / 2;
          return Math.abs(center - innerHeight / 2);
        };
        const unique = (values) => {
          const urls = [...new Set(values.filter(Boolean))];
          const ids = new Set(urls.map(url => url.match(/\/video\/(\d+)/)?.[1]).filter(Boolean));
          return ids.size === 1 ? urls[0] : null;
        };
        const found = [];
        const videos = [...document.querySelectorAll('video')]
          .map(video => ({ video, score: visibleScore(video) }))
          .filter(item => Number.isFinite(item.score))
          .sort((a, b) => a.score - b.score);
        for (const { video, score } of videos.slice(0, 3)) {
          const urls = [], explicit = new Set();
          for (let node = video, depth = 0; node && depth < 18; node = node.parentElement, depth += 1) {
            if (node === document.body || node === document.documentElement) break;
            const otherVideos = [...(node.querySelectorAll?.('video') || [])].filter(other => other !== video && Number.isFinite(visibleScore(other)));
            if (depth > 0 && otherVideos.length) break;
            for (const name of ['data-video-id', 'data-item-id', 'data-aweme-id']) {
              const id = node.getAttribute?.(name);
              if (id && /^\d+$/.test(id)) explicit.add(id);
            }
            for (const anchor of node.querySelectorAll?.('a[href*="/video/"]') || []) urls.push(valid(anchor.href));
            if (node.matches?.('article,[data-e2e="recommend-list-item-container"],[data-e2e="feed-video"],[data-e2e="browse-video"]')) break;
          }
          let url = unique(urls);
          if (!url && explicit.size === 1) {
            const id = [...explicit][0];
            url = unique(urls.filter(value => value?.endsWith(`/video/${id}`)));
          }
          if (!url && videos.length === 1) url = valid(location.href);
          if (url) found.push({
            url,
            score,
            title: String(document.title || 'TikTok').trim().slice(0, 180),
            thumbnail: String(video.poster || ''),
            duration: Number.isFinite(video.duration) && video.duration > 0 ? video.duration : null,
          });
        }
        // Some TikTok layouts keep the visible card/permalink in the parent
        // frame while the actual <video> lives in a child frame. In frames with
        // no visible video, use only the permalink nearest the viewport center.
        if (!found.length) {
          const anchors = [...document.querySelectorAll('a[href*="/video/"]')]
            .map(anchor => ({ url: valid(anchor.href), score: visibleScore(anchor) }))
            .filter(item => item.url && Number.isFinite(item.score))
            .sort((a, b) => a.score - b.score);
          if (anchors[0]) found.push({ url: anchors[0].url, score: anchors[0].score + 20, title: String(document.title || 'TikTok').trim().slice(0, 180), thumbnail: '', duration: null });
        }
        return found;
      },
    }).catch(error => {
      void globalThis.ApocalipseDiagnostics?.emit("popup.all_frames_failed", { error: String(error) }, popupScanTrace, "WARN", "extension.popup");
      return [];
    });
    void globalThis.ApocalipseDiagnostics?.emit("popup.all_frames_result", { frames: results.length,
      candidates: results.reduce((n, entry) => n + (entry.result?.length || 0), 0), frameIds: results.map(entry => entry.frameId) }, popupScanTrace, "INFO", "extension.popup");
    return results.flatMap(entry => Array.isArray(entry.result) ? entry.result : []);
  }

  async function refresh() {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true }).catch(() => []);
    if (!tab?.id || !isTikTok(tab.url || '')) return;
    const candidates = await collectTikTokFromAllFrames(tab.id);
    if (!candidates.length) return;
    const unique = new Map();
    for (const item of candidates.sort((a, b) => a.score - b.score)) {
      if (!unique.has(item.url)) unique.set(item.url, item);
    }
    const best = [...unique.values()][0];
    if (!best) return;
    const identified = {
      url: best.url,
      extractorUrl: best.url,
      kind: 'video',
      pageExtractor: true,
      recommended: true,
      ambiguousSocialTrack: false,
      title: best.title || `TikTok — ${best.url.match(/\/video\/(\d+)/)?.[1] || 'Reel'}`,
      thumbnail: best.thumbnail || '',
      duration: best.duration,
      size: null,
      ext: 'mp4',
    };
    try {
      if (typeof mergeDetectedMedia === 'function' && typeof render === 'function') {
        media = mergeDetectedMedia([identified, ...(Array.isArray(media) ? media : [])], [], tab.url || '');
        activePageUrl = tab.url || activePageUrl;
        render();
      }
    } catch (error) {
      console.debug('Apocalipse TikTok popup identity', error);
    }
  }

  setTimeout(refresh, 150);
  setTimeout(refresh, 900);
  setTimeout(refresh, 1800);
})();
