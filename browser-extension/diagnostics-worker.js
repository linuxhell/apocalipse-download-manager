// The worker alone owns the durable outbox. It persists BEFORE attempting delivery.
(() => {
  if (globalThis.ADM_DIAG_WORKER || !globalThis.ADM_DIAG_CORE) return;
  const core = ADM_DIAG_CORE, KEY = 'admDiagnosticsV3';
  const contextId = crypto.randomUUID();
  let state = { config: null, outbox: [], dropped: 0, storageErrors: 0, transportErrors: 0, accepted: 0 };
  let serial = Promise.resolve(), sequence = 0, flushing = false, configChecked = 0;
  let rateAt = 0, rateCount = 0;
  const MAX_OUTBOX_BYTES = 2 * 1024 * 1024;
  const load = chrome.storage.local.get({ [KEY]: null }).then(saved => {
    const value = saved[KEY];
    if (value && typeof value === 'object') state = { ...state, ...value, outbox: Array.isArray(value.outbox) ? value.outbox.slice(-core.MAX_EVENTS) : [] };
  }).catch(() => { state.storageErrors++; });
  const persist = async () => {
    try { await chrome.storage.local.set({ [KEY]: state }); }
    catch { state.storageErrors++; }
  };
  const transact = fn => {
    const next = serial.then(() => load).then(fn);
    serial = next.catch(() => {});
    return next;
  };
  async function bridge(path, payload) {
    const { pairingToken = '' } = await chrome.storage.local.get({ pairingToken: '' });
    if (!pairingToken) throw new Error('not_paired');
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), 5000);
    try {
      const response = await fetch(`http://127.0.0.1:17654/v1/diagnostics-v3/${path}`, {
        method: payload === undefined ? 'GET' : 'POST',
        headers: { Authorization: `Bearer ${pairingToken}`, 'Content-Type': 'application/json' },
        ...(payload === undefined ? {} : { body: JSON.stringify(payload) }), signal: controller.signal,
      });
      if (!response.ok) throw new Error(`diagnostics_http_${response.status}`);
      return await response.json();
    } finally { clearTimeout(timer); }
  }
  const sameSession = config => config?.sessionId === state.config?.sessionId;
  const scoped = sender => core.isActive(state.config) && (
    !sender?.tab || sender.tab.id === state.config.tabId);
  async function refreshConfig() {
    await load;
    if (Date.now() - configChecked < 1500) return;
    configChecked = Date.now();
    try {
      const config = await bridge('status');
      await transact(async () => {
        if (!sameSession(config)) { state.dropped += state.outbox.length; state.outbox = []; }
        state.config = config;
        await persist();
      });
    } catch { /* Status is optional; an unexpired last configuration remains usable. */ }
  }
  function allowed(input, sender) {
    if (!input || typeof input !== 'object' || !scoped(sender) || input.sessionId !== state.config.sessionId) return false;
    return !sender.tab || sender.tab.id === state.config.tabId;
  }
  async function enqueue(inputs, sender = {}) {
    return transact(async () => {
      const now = Date.now();
      if (now - rateAt >= 1000) { rateAt = now; rateCount = 0; }
      let accepted = 0;
      for (const input of inputs.slice(0, core.MAX_BATCH)) {
        if (!allowed(input, sender)) continue;
        if (++rateCount > 80) { state.dropped++; continue; }
        const event = await core.record(input, state.config, {
          component: sender.tab ? 'content' : input.component || 'popup', contextId,
          tabId: sender.tab?.id ?? state.config.tabId, frameId: sender.frameId,
        });
        if (!event) continue;
        if (state.outbox.length >= core.MAX_EVENTS) { state.outbox.shift(); state.dropped++; }
        state.outbox.push(event); accepted++;
        // Bound bytes as well as record count, below the browser storage quota.
        while (JSON.stringify(state.outbox).length > MAX_OUTBOX_BYTES) { state.outbox.shift(); state.dropped++; }
      }
      await persist();
      return { ok: true, accepted, dropped: state.dropped };
    });
  }
  async function emit(event, detail = {}, traceId = null, level = 'INFO', tabId = null) {
    await load;
    if (!core.isActive(state.config) || (tabId !== null && state.config.tabId !== tabId)) return;
    const result = await enqueue([{ sessionId: state.config.sessionId, id: crypto.randomUUID(), contextId,
      sequence: ++sequence, traceId, event, detail, level, component: 'worker',
      version: chrome.runtime.getManifest().version, clientTimestamp: Date.now() }]);
    void flush();
    return result;
  }
  async function emitForSender(sender, event, detail = {}, traceId = null, level = 'INFO') {
    await load;
    if (!core.isActive(state.config)) return;
    let tabId = sender.tab?.id;
    if (!Number.isInteger(tabId)) {
      const [tab] = await chrome.tabs.query({ active: true, currentWindow: true }).catch(() => []);
      tabId = tab?.id;
    }
    if (Number.isInteger(tabId)) return emit(event, detail, traceId, level, tabId);
  }
  async function flush() {
    if (flushing) return;
    flushing = true;
    try {
      for (let n = 0; n < 8; n++) {
        const batch = await transact(async () => state.outbox.slice(0, core.MAX_BATCH));
        if (!batch.length) break;
        let result;
        try { result = await bridge('events', { sessionId: batch[0].sessionId, events: batch,
          health: { dropped: state.dropped, storageErrors: state.storageErrors, transportErrors: state.transportErrors,
            workerContextId: contextId, queued: state.outbox.length } }); }
        catch { await transact(async () => { state.transportErrors++; await persist(); }); break; }
        if (!result?.ok || !Array.isArray(result.ackIds)) break;
        const acknowledged = new Set(result.ackIds);
        await transact(async () => {
          state.outbox = state.outbox.filter(item => !acknowledged.has(item.id));
          state.accepted += result.accepted || 0;
          await persist();
        });
        if (!acknowledged.size) break;
      }
    } finally { flushing = false; }
  }
  async function control(action, tabId) {
    if (!['start','stop','mark','clear'].includes(action)) throw new Error('invalid_diagnostics_action');
    await flush();
    const config = await bridge('control', { action, tabId: Number.isInteger(tabId) ? tabId : null });
    await transact(async () => {
      if (!sameSession(config)) {
        state = { config, outbox: [], dropped: 0, storageErrors: 0, transportErrors: 0, accepted: 0 };
      } else state.config = config;
      await persist();
    });
    configChecked = Date.now();
    if (action === 'start') await emit('worker.session_ready', { webRequestAvailable: Boolean(chrome.webRequest?.onResponseStarted),
      storageAvailable: true, cacheVisibilityLimited: true });
    return config;
  }
  function message(message, sender, reply) {
    if (!String(message?.type || '').startsWith('ADM_DIAG_')) return false;
    const extensionPage = !sender.tab && String(sender.url || '').startsWith(chrome.runtime.getURL(''));
    (async () => {
      if (message.type === 'ADM_DIAG_CONFIG') {
        await refreshConfig();
        if (!sender.tab) {
          const [tab] = await chrome.tabs.query({ active: true, currentWindow: true }).catch(() => []);
          if (!tab || tab.id !== state.config?.tabId) return { active: false };
        }
        return scoped(sender) ? state.config : { active: false };
      }
      if (message.type === 'ADM_DIAG_BATCH') {
        const result = await enqueue(Array.isArray(message.events) ? message.events : [], sender);
        void flush(); return result;
      }
      if (message.type === 'ADM_DIAG_STATUS' && extensionPage) {
        await refreshConfig();
        return { ...state.config, queued: state.outbox.length, dropped: state.dropped, storageErrors: state.storageErrors };
      }
      if (message.type === 'ADM_DIAG_CONTROL' && extensionPage) {
        // Only an extension-owned page may start collection, never a webpage relay.
        const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
        if (message.action === 'start' && (!tab?.id || !/^https?:/.test(tab.url || ''))) throw new Error('select_http_tab');
        return await control(message.action, tab?.id);
      }
      if (message.type === 'ADM_DIAG_FLUSH' && extensionPage) { await flush(); return { ok: true }; }
      return { ok: false, error: 'diagnostics_not_authorized' };
    })().then(reply).catch(error => reply({ ok: false, error: String(error?.message || 'diagnostics_failed').slice(0,100) }));
    return true;
  }
  async function network(details, accepted, reason) {
    await load;
    if (!core.isActive(state.config) || details.tabId !== state.config.tabId) return;
    const header = name => (details.responseHeaders || []).find(h => h.name?.toLowerCase() === name)?.value || '';
    const mime = header('content-type').split(';')[0].trim().toLowerCase();
    const relevant = /^(?:video|audio)\//.test(mime) || details.type === 'media'
      || /\.(?:mp4|webm|m3u8|mpd)(?:[?#]|$)|mime_type=(?:video|audio)/i.test(details.url || '');
    if (!relevant) return;
    return emit('capture.network_decision', { url: details.url, requestRef: String(details.requestId),
      contentType: mime || 'unknown', kind: details.type || 'unknown', method: details.method || 'GET',
      statusCode: details.statusCode, accepted, reason, fromCache: Boolean(details.fromCache),
      bytes: parseInt(header('content-length'),10) || null, partial: details.statusCode === 206,
      frameId: details.frameId, browserTimestamp: details.timeStamp }, null, accepted ? 'INFO' : 'WARN', details.tabId);
  }
  globalThis.ADM_DIAG_WORKER = { message, emit, emitForSender, network, flush, refreshConfig,
    health: async () => { await load; return { queued: state.outbox.length, dropped: state.dropped, storageErrors: state.storageErrors }; } };
  chrome.alarms?.create('adm-diagnostics-v3', { periodInMinutes: 0.5 });
  chrome.alarms?.onAlarm?.addListener(alarm => { if (alarm.name === 'adm-diagnostics-v3') { void refreshConfig().then(flush); } });
  chrome.webRequest?.onErrorOccurred?.addListener(details => {
    if (details.type === 'media') void emit('capture.network_error', { url: details.url, errorRef: details.error,
      frameId: details.frameId, requestRef: String(details.requestId) }, null, 'ERROR', details.tabId);
  }, { urls: ['http://*/*','https://*/*'] });
  void load.then(() => emit('worker.restarted', { queued: state.outbox.length })).then(flush);
})();
