// Bounded, serialized, sanitized outbox. At-least-once delivery; desktop deduplicates eventId.
(() => {
  const D = globalThis.ApocalipseDiagnostics;
  if (!D || globalThis.ApocalipseDiagnosticSink) return;
  const KEY = 'admDiagnosticOutboxV3', MAX_EVENTS = 600, MAX_BYTES = 1800000;
  let state = { pending: [], dropped: 0, delivered: 0, storageErrors: 0, deliveryErrors: 0, lastDeliveryError: null };
  let chain = Promise.resolve(), flushing = false, timer = null;
  const ready = chrome.storage.local.get({ [KEY]: null }).then(stored => {
    const old = stored[KEY];
    if (old && Array.isArray(old.pending)) state = { ...state, ...old, pending: old.pending.slice(-MAX_EVENTS) };
  }).catch(() => { state.storageErrors++; });
  const serial = fn => { const next = chain.then(fn); chain = next.catch(() => {}); return next; };
  const persist = async () => {
    try { await chrome.storage.local.set({ [KEY]: state }); }
    catch { state.storageErrors++; }
  };
  const append = async record => serial(async () => {
    await ready;
    const item = { ...record, data: await D.safe(record.data) };
    if (JSON.stringify(item).length > 10000) { item.data = { truncated: true }; state.dropped++; }
    state.pending.push(item);
    while (state.pending.length > MAX_EVENTS || JSON.stringify(state.pending).length > MAX_BYTES) {
      // Prefer retaining failed actions rather than debug snapshots under pressure.
      const low = state.pending.findIndex(e => e.level === 'DEBUG');
      state.pending.splice(low < 0 ? 0 : low, 1); state.dropped++;
    }
    await persist();
    if (!timer) timer = setTimeout(() => { timer = null; void flush(); }, 1000);
    return { ok: true };
  });
  globalThis.ApocalipseDiagnosticSink = append;
  async function flush() {
    if (flushing) return;
    flushing = true;
    try {
      await ready; await chain;
      const { pairingToken = '' } = await chrome.storage.local.get({ pairingToken: '' });
      if (!pairingToken || !state.pending.length) return;
      const batch = state.pending.slice(0, 40);
      const controller = new AbortController(), timeout = setTimeout(() => controller.abort(), 5000);
      try {
        const response = await fetch('http://127.0.0.1:17654/v1/diagnostic-v3', {
          method: 'POST', headers: { Authorization: `Bearer ${pairingToken}`, 'Content-Type': 'application/json' },
          body: JSON.stringify({ events: batch, health: { pending: state.pending.length, dropped: state.dropped,
            storageErrors: state.storageErrors, deliveryErrors: state.deliveryErrors, producerId: D.producerId } }), signal: controller.signal,
        });
        if (!response.ok) throw new Error(`diagnostic_http_${response.status}`);
        const result = await response.json();
        if (!Array.isArray(result.accepted)) throw new Error('diagnostic_ack_missing');
        const ids = new Set(result.accepted.filter(id => batch.some(item => item.eventId === id)));
        await serial(async () => {
          state.pending = state.pending.filter(item => !ids.has(item.eventId));
          state.delivered += ids.size; await persist();
        });
        if (ids.size && state.pending.length && !timer) timer = setTimeout(() => { timer = null; void flush(); }, 1000);
      } finally { clearTimeout(timeout); }
    } catch (error) { await serial(async () => { state.deliveryErrors++; state.lastDeliveryError = await D.safe(String(error)); await persist(); }); }
    finally { flushing = false; }
  }
  const trustedPopup = sender => !sender.tab && String(sender.url || '').startsWith(chrome.runtime.getURL(''));
  const scoped = tabId => D.active() && D.session()?.tabId === tabId;
  chrome.runtime.onMessage.addListener((message, sender, reply) => {
    if (message?.type === 'APOCALIPSE_DIAGNOSTIC_SCOPE') { reply({ active: scoped(sender.tab?.id) }); return; }
    if (message?.type === 'APOCALIPSE_DIAGNOSTIC_EVENT_V3') {
      const r = message.record;
      if (!r || !D.uuid(r.eventId) || !D.uuid(r.producerId)) { reply({ ok: false }); return; }
      if (r.level === 'DEBUG' && sender.tab && !scoped(sender.tab.id)) { reply({ ok: true, skipped: true }); return; }
      // Sender metadata is assigned by Chrome, never trusted from page payloads.
      append({ ...r, component: sender.tab ? 'extension.content' : 'extension.popup',
        tabId: sender.tab?.id ?? null, frameId: sender.frameId ?? null, documentId: sender.documentId || null,
        extensionVersion: chrome.runtime.getManifest().version }).then(reply, () => reply({ ok: false }));
      return true;
    }
    if (message?.type === 'APOCALIPSE_DIAGNOSTIC_STATUS') {
      Promise.all([ready, D.ready]).then(() => reply({ active: D.active(), session: D.session() ? {
        id: D.session().id, tabId: D.session().tabId, expiresAt: D.session().expiresAt } : null,
        pending: state.pending.length, delivered: state.delivered, dropped: state.dropped,
        storageErrors: state.storageErrors, deliveryErrors: state.deliveryErrors, lastDeliveryError: state.lastDeliveryError }));
      return true;
    }
    if (message?.type !== 'APOCALIPSE_DIAGNOSTIC_CONTROL') return;
    if (!trustedPopup(sender)) { reply({ ok: false, error: 'diagnostic_control_requires_popup' }); return; }
    (async () => {
      if (message.command === 'start') {
        const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
        if (!tab?.id || !/^https?:/.test(tab.url || '')) throw new Error('diagnostic_web_tab_required');
        const session = { id: crypto.randomUUID(), salt: crypto.randomUUID(), tabId: tab.id,
          startedAt: Date.now(), expiresAt: Date.now() + 5 * 60 * 1000 };
        await chrome.storage.local.set({ admDiagnosticSession: session }); D.setSession(session);
        await D.emit('diagnostic.capture_started', { tabId: tab.id, durationSeconds: 300 }, null, 'INFO', 'extension.background');
        // Responses from each frame are emitted independently by the content collector.
        await chrome.tabs.sendMessage(tab.id, { type: 'APOCALIPSE_DIAGNOSTIC_SNAPSHOT' }).catch(() => {
          void D.emit('collector.content_unreachable', { tabId: tab.id, reason: 'no_content_response_reload_tab' }, null, 'WARN', 'extension.background');
        });
      } else if (message.command === 'stop') {
        await D.emit('diagnostic.capture_stopped', {}, null, 'INFO', 'extension.background');
        await chrome.storage.local.set({ admDiagnosticSession: null }); D.setSession(null);
      } else if (message.command === 'mark') {
        await D.emit('diagnostic.user_mark', { tabId: D.session()?.tabId ?? null }, crypto.randomUUID(), 'WARN', 'extension.background');
        const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
        if (tab?.id) await chrome.tabs.sendMessage(tab.id, { type: 'APOCALIPSE_DIAGNOSTIC_SNAPSHOT' }).catch(() => {});
      } else throw new Error('diagnostic_invalid_command');
      void flush(); return { ok: true };
    })().then(reply, error => reply({ ok: false, error: String(error) }));
    return true;
  });
  // Observe relevant network responses BEFORE the downloader's host filter; never alter it.
  if (chrome.webRequest?.onResponseStarted) {
    chrome.webRequest.onResponseStarted.addListener(details => {
      if (!scoped(details.tabId)) return;
      const header = name => (details.responseHeaders || []).find(h => h.name.toLowerCase() === name)?.value || '';
      const mime = header('content-type').toLowerCase();
      if (!/^(?:video|audio)\//.test(mime) && !/(?:mpegurl|dash\+xml)/.test(mime)
        && !/\.(?:mp4|webm|m3u8|mpd)(?:$|[?#])|\/video\/tos\/|\/aweme\/v1\/play\//i.test(details.url)) return;
      void D.emit('network.media_response_observed', { tabId: details.tabId, frameId: details.frameId,
        requestId: details.requestId, url: details.url, initiator: details.initiator || '',
        contentType: mime, status: details.statusCode, fromCache: details.fromCache,
        contentLength: Number(header('content-length')) || null, contentRange: header('content-range'),
        browserTimestamp: details.timeStamp, type: details.type }, null, 'DEBUG', 'extension.background');
    }, { urls: ['http://*/*', 'https://*/*'] }, ['responseHeaders']);
  }
  chrome.alarms.create('adm-diagnostic-v3-flush', { periodInMinutes: 0.5 });
  chrome.alarms.onAlarm.addListener(alarm => { if (alarm.name === 'adm-diagnostic-v3-flush') void flush(); });
  void ready.then(() => D.emit('collector.worker_ready', { webRequest: Boolean(chrome.webRequest?.onResponseStarted),
    pendingRestored: state.pending.length, previousDeliveryErrors: state.deliveryErrors }, null, 'INFO', 'extension.background'));
  globalThis.ApocalipseDiagnosticWorker = { scoped, flush, status: () => ({ ...state, pending: state.pending.length }) };
})();
