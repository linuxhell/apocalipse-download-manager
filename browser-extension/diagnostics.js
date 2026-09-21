// Loaded in ISOLATED / extension pages only. Detailed observation is opt-in and expires.
(() => {
  if (globalThis.ADM_DIAG || !globalThis.ADM_DIAG_CORE) return;
  const core = ADM_DIAG_CORE, contextId = crypto.randomUUID();
  const component = /^(chrome|moz)-extension:$/.test(location.protocol) ? 'popup' : 'content';
  let config = null, sequence = 0, outbox = [], dropped = 0, sendErrors = 0, busy = false, timer = null;
  let rateAt = 0, rateCount = 0, chain = Promise.resolve(), lastPlayers = '', snapshotTimer = null;
  let mutationTimer = null, mutationStats = { batches:0, addedNodes:0, removedNodes:0, addedVideos:0, removedVideos:0, sourceChanges:0 };
  const scripts = new Set(), players = new WeakMap(), resourceSeen = new Set();
  const send = message => new Promise((resolve,reject) => {
    let finished = false;
    const timeout = setTimeout(() => { if (!finished) { finished = true; reject(new Error('diagnostic_message_timeout')); } }, 6000);
    try {
      chrome.runtime.sendMessage(message, value => {
        if (finished) return; finished = true; clearTimeout(timeout);
        const error = chrome.runtime.lastError;
        error ? reject(new Error('diagnostic_channel_error')) : resolve(value);
      });
    } catch { clearTimeout(timeout); finished = true; reject(new Error('diagnostic_context_invalid')); }
  });
  const active = () => core.isActive(config);
  function emit(event, detail = {}, traceId = null, level = 'INFO') {
    if (!active()) return Promise.resolve();
    const now = Date.now();
    if (now - rateAt >= 1000) { rateAt = now; rateCount = 0; }
    if (++rateCount > 120) { dropped++; return Promise.resolve(); }
    const capturedConfig = config;
    const input = { id: crypto.randomUUID(), contextId, sequence: ++sequence, traceId, event, level, component,
      version: chrome.runtime.getManifest().version, clientTimestamp: now, monoMs: Math.round(performance.now()), detail };
    chain = chain.then(async () => {
      const record = await core.record(input, capturedConfig, { component, contextId });
      if (!record || config?.sessionId !== capturedConfig.sessionId) return;
      if (outbox.length >= 200) { outbox.shift(); dropped++; }
      outbox.push(record);
      if (!timer) timer = setTimeout(() => { timer = null; void flush(); }, 150);
    }).catch(() => { sendErrors++; });
    return chain;
  }
  async function flush() {
    await chain;
    if (busy || !outbox.length) return;
    busy = true;
    try {
      const events = outbox.slice(0, core.MAX_BATCH);
      const result = await send({ type: 'ADM_DIAG_BATCH', events });
      if (result?.ok) outbox.splice(0, events.length);
      else sendErrors++;
    } catch { sendErrors++; }
    finally {
      busy = false;
      if (outbox.length && active() && !timer) timer = setTimeout(() => { timer = null; void flush(); }, 800);
    }
  }
  function player(video) {
    let entry = players.get(video);
    const source = String(video?.currentSrc || video?.src || '');
    if (!entry) { entry = { playerId: crypto.randomUUID(), revision: 0, source }; players.set(video, entry); }
    if (entry.source !== source) { entry.source = source; entry.revision++; }
    let rect; try { rect = video.getBoundingClientRect(); } catch { rect = {}; }
    return { playerId: entry.playerId, revision: entry.revision, url: source,
      sourceScheme: source.startsWith('blob:') ? 'blob' : source.startsWith('http') ? 'http' : source ? 'other' : 'empty',
      duration: Number.isFinite(video?.duration) ? video.duration : null,
      currentTime: Number.isFinite(video?.currentTime) ? video.currentTime : null,
      videoWidth: video?.videoWidth || 0, videoHeight: video?.videoHeight || 0,
      readyState: String(video?.readyState ?? 'unknown'), networkState: video?.networkState,
      connected: Boolean(video?.isConnected), paused: Boolean(video?.paused), ended: Boolean(video?.ended),
      seeking: Boolean(video?.seeking), muted: Boolean(video?.muted), loop: Boolean(video?.loop),
      autoplay: Boolean(video?.autoplay), controls: Boolean(video?.controls),
      playbackRate: Number.isFinite(video?.playbackRate) ? video.playbackRate : null,
      volume: Number.isFinite(video?.volume) ? video.volume : null,
      sourceChildren: video?.querySelectorAll?.('source')?.length || 0,
      visible: rect?.width > 40 && rect?.height > 20 && rect?.bottom > 0 && rect?.top < innerHeight,
      rect: { left: Math.round(rect?.left || 0), top: Math.round(rect?.top || 0),
        width: Math.round(rect?.width || 0), height: Math.round(rect?.height || 0) },
      hasSrcObject: Boolean(video?.srcObject), errorCode: video?.error?.code || null };
  }
  async function snapshot() {
    if (!active()) { clearInterval(snapshotTimer); snapshotTimer = null; return; }
    const videos = [...document.querySelectorAll('video')].slice(0, 40);
    const snapshot = videos.map(player);
    const signature = JSON.stringify(snapshot);
    if (signature !== lastPlayers) {
      lastPlayers = signature;
      void emit('players.snapshot', { players: snapshot, videos: videos.length,
        frames: document.querySelectorAll('iframe').length, source: location.href });
    }
    let seen = 0;
    for (const resource of performance.getEntriesByType('resource').slice(-300)) {
      if (++seen > 300 || resourceSeen.has(resource.name)) continue;
      const relevant = /\.(?:mp4|webm|m3u8|mpd)(?:[?#]|$)|mime_type=(?:video|audio)/i.test(resource.name)
        || ['video','audio'].includes(resource.initiatorType);
      if (!relevant) continue;
      if (resourceSeen.size >= 600) { dropped++; break; }
      resourceSeen.add(resource.name);
      void emit('capture.performance_resource', { url: resource.name, kind: resource.initiatorType || 'unknown',
        durationMs: resource.duration, bytes: resource.encodedBodySize, transferBytes: resource.transferSize,
        startMs: resource.startTime, bindingProven: false });
    }
  }
  const countVideoNodes = node => {
    if (!node || node.nodeType !== 1) return 0;
    return (node.tagName === 'VIDEO' ? 1 : 0) + (node.querySelectorAll?.('video')?.length || 0);
  };
  const flushMutations = () => {
    mutationTimer = null;
    if (!active()) return;
    const stats = mutationStats;
    mutationStats = { batches:0, addedNodes:0, removedNodes:0, addedVideos:0, removedVideos:0, sourceChanges:0 };
    if (stats.batches) void emit('dom.media_mutation_batch', stats);
  };
  const mutationObserver = new MutationObserver(records => {
    if (!active()) return;
    mutationStats.batches += 1;
    for (const record of records) {
      if (record.type === 'attributes') {
        if (record.target?.tagName === 'VIDEO' || record.target?.tagName === 'SOURCE') mutationStats.sourceChanges += 1;
        continue;
      }
      mutationStats.addedNodes += record.addedNodes?.length || 0;
      mutationStats.removedNodes += record.removedNodes?.length || 0;
      for (const node of record.addedNodes || []) mutationStats.addedVideos += countVideoNodes(node);
      for (const node of record.removedNodes || []) mutationStats.removedVideos += countVideoNodes(node);
    }
    if (!mutationTimer) mutationTimer = setTimeout(flushMutations, 350);
  });
  try {
    mutationObserver.observe(document.documentElement, {
      subtree:true, childList:true, attributes:true, attributeFilter:['src','poster','class','style']
    });
  } catch {}

  async function refresh() {
    try {
      const next = await send({ type: 'ADM_DIAG_CONFIG' });
      if (!next?.active || !core.isActive(next)) {
        config = null; clearInterval(snapshotTimer); snapshotTimer = null; return;
      }
      const changed = config?.sessionId !== next.sessionId;
      config = next;
      if (changed) {
        dropped += outbox.length; outbox = []; lastPlayers = ''; resourceSeen.clear();
        void emit('collector.started', { scripts: [...scripts].map(script => ({ script })),
          readyState: document.readyState, world: 'ISOLATED', topFrame: window === window.top,
          dropped, sendErrors, cacheVisibilityLimited: true, workerPersistenceEnabled: true });
        if (component === 'content') {
          await snapshot();
          if (!snapshotTimer) snapshotTimer = setInterval(() => void snapshot(), 1200);
        }
      }
    } catch { sendErrors++; }
  }
  const begin = (event, detail = {}) => { const traceId = crypto.randomUUID(); void emit(event, detail, traceId); return traceId; };
  globalThis.ADM_DIAG = { emit, begin, player, flush, active, refresh,
    register(script) { scripts.add(script); void emit('script.loaded', { script, world: 'ISOLATED' }); },
    sessionId: () => active() ? config.sessionId : null,
  };
  chrome.storage?.onChanged?.addListener((changes, area) => {
    if (area === 'local' && changes.admDiagnosticsV3) {
      const previous = changes.admDiagnosticsV3.oldValue?.config;
      const current = changes.admDiagnosticsV3.newValue?.config;
      if (JSON.stringify(previous) !== JSON.stringify(current)) void refresh();
    }
  });
  addEventListener('pagehide', () => { void emit('collector.pagehide', { dropped, sendErrors, queued: outbox.length }); void flush(); });
  addEventListener('error', event => {
    // Never capture page text, message contents, stack traces or arguments.
    void emit('collector.javascript_error', { errorName: event.error?.name || 'Error', resourceRef: event.filename || '',
      line: event.lineno || 0, column: event.colno || 0 }, null, 'ERROR');
  });
  addEventListener('unhandledrejection', event => {
    void emit('collector.unhandled_rejection', { errorName: event.reason?.name || 'Error', errorRef: String(event.reason) }, null, 'ERROR');
  });
  for (const type of ['loadstart','loadedmetadata','loadeddata','canplay','play','playing','pause','waiting','stalled','suspend','ended','emptied','durationchange','seeking','seeked','ratechange','volumechange','resize','abort','error']) {
    document.addEventListener(type, event => {
      if (event.target?.tagName === 'VIDEO') void emit('player.lifecycle', { eventType: type, ...player(event.target) }, null, type === 'error' ? 'ERROR' : 'INFO');
    }, true);
  }
  addEventListener('online', () => { if (active()) void emit('collector.network_state', { status:'online' }); });
  addEventListener('offline', () => { if (active()) void emit('collector.network_state', { status:'offline' }, null, 'WARN'); });
  document.addEventListener('visibilitychange', () => {
    if (active()) void emit('collector.visibility', { state: document.visibilityState || 'unknown', hidden:Boolean(document.hidden) });
  });
  addEventListener('pageshow', event => {
    if (active()) void emit('collector.pageshow', { persisted:Boolean(event.persisted), readyState:document.readyState });
  });
  void refresh();
  // Covers opening the page before/after the session and expiry without keeping a worker alive.
  setInterval(() => { if (active()) void emit('collector.health', { dropped, sendErrors, queued: outbox.length }); }, 5000);
})();
