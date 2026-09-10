const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const vm = require('node:vm');
const { webcrypto } = require('node:crypto');
const root = join(__dirname, '../browser-extension');
const read = name => readFileSync(join(root, name), 'utf8');
const clone = value => JSON.parse(JSON.stringify(value));
function harness(initial = {}, failing = false) {
  const values = clone(initial), changed = [], listeners = [], network = [], calls = [];
  const noopEvent = { addListener() {} };
  let failStorage = false, failDelivery = failing;
  const storage = {
    async get(defaults) { return typeof defaults === 'string' ? { [defaults]: clone(values[defaults] ?? null) } : { ...defaults, ...clone(values) }; },
    async set(update) {
      if (failStorage) throw new Error('synthetic storage failure');
      const changes = {};
      for (const [key, value] of Object.entries(update)) { changes[key] = { oldValue: values[key], newValue: clone(value) }; values[key] = clone(value); }
      for (const fn of changed) fn(changes, 'local');
    },
    async remove(key) { delete values[key]; },
  };
  const context = vm.createContext({ URL, TextEncoder, crypto: webcrypto, console, AbortController,
    performance: { now: () => 123 }, navigator: { userAgent: 'SyntheticBrowser' },
    setTimeout: () => 1, clearTimeout() {},
    chrome: {
      storage: { local: storage, session: storage, onChanged: { addListener(fn) { changed.push(fn); } } },
      runtime: { getManifest: () => ({ version: '0.3.102' }), getURL: path => `chrome-extension://synthetic/${path}`,
        onMessage: { addListener(fn) { listeners.push(fn); } }, onInstalled: noopEvent, onStartup: noopEvent },
      alarms: { create() {}, onAlarm: noopEvent },
      webRequest: { onResponseStarted: { addListener(fn) { network.push(fn); } } },
      downloads: { onDeterminingFilename: noopEvent, onChanged: noopEvent },
      cookies: { getAll: async () => [] },
      tabs: { query: async () => [{ id: 7, url: 'https://www.tiktok.com/' }], sendMessage: async () => ({ ok: true }) },
    },
    fetch: async (url, options = {}) => {
      calls.push({ url, body: options.body ? JSON.parse(options.body) : null });
      if (url.endsWith('/v1/diagnostic-v3')) {
        if (failDelivery) throw new Error('offline');
        return { ok: true, json: async () => ({ accepted: JSON.parse(options.body).events.map(e => e.eventId) }) };
      }
      return { ok: true, json: async () => ({ ok: true, taskId: 'synthetic-task' }) };
    },
  });
  context.importScripts = (...files) => files.forEach(file => vm.runInContext(read(file), context));
  vm.runInContext(read('background.js'), context);
  const send = (message, sender = { url: 'chrome-extension://synthetic/popup.html' }) => new Promise((resolve, reject) => {
    let handled = false, answered = false;
    for (const listener of listeners) {
      const asyncReply = listener(message, sender, value => { answered = true; resolve(value); });
      if (asyncReply === true) handled = true;
      if (answered || handled) break;
    }
    if (!handled && !answered) reject(new Error('no handler'));
  });
  return { context, values, calls, network, send, failStorage: () => { failStorage = true; },
    online: () => { failDelivery = false; }, D: context.ApocalipseDiagnostics, W: context.ApocalipseDiagnosticWorker };
}
const settle = () => new Promise(resolve => setImmediate(resolve));

test('privacy: nested secrets, URL paths and signed parameters are removed before outbox storage', async () => {
  const h = harness(); await h.D.ready;
  const action = webcrypto.randomUUID();
  await h.D.emit('test.secret_scrub', { url: 'https://alice:password@cdn.example/private-file?sig=CANARY1',
    headers: { Authorization: 'Bearer CANARY2', Cookie: 'id=CANARY3' }, title: 'CANARY4',
    nested: { error: 'token=CANARY5' }, path: 'C:\\Users\\private\\CANARY6' }, action);
  const text = JSON.stringify(h.values.admDiagnosticOutboxV3);
  for (let n = 1; n <= 6; n++) assert.ok(!text.includes(`CANARY${n}`), `canary ${n} leaked`);
  assert.ok(!text.includes('alice')); assert.ok(!text.includes('private-file'));
  assert.ok(text.includes(action)); assert.ok(text.includes('cdn.example'));
});

test('identical resources correlate within a capture, without exporting the salt', async () => {
  const h = harness(); await h.send({ type: 'APOCALIPSE_DIAGNOSTIC_CONTROL', command: 'start' });
  const a = await h.D.resource('https://cdn.example/video?secret=x');
  const b = await h.D.resource('https://cdn.example/video?secret=x');
  const c = await h.D.resource('https://cdn.example/video?secret=y');
  assert.equal(a.resourceId, b.resourceId); assert.notEqual(a.resourceId, c.resourceId);
  assert.ok(!JSON.stringify(a).includes('secret')); assert.ok(!JSON.stringify(a).includes(h.values.admDiagnosticSession.salt));
});

test('offline outbox survives worker recreation and is removed only after acknowledged delivery', async () => {
  const h = harness({ pairingToken: 'test-pairing' }, true);
  const id = webcrypto.randomUUID(); await h.D.emit('overlay.test_failed', { reason: 'synthetic' }, id, 'WARN');
  await h.W.flush();
  assert.ok(h.W.status().pending > 0); assert.ok(h.W.status().deliveryErrors > 0);
  const restored = harness(h.values); await restored.D.ready; await settle();
  await restored.W.flush();
  const sent = restored.calls.filter(c => c.url.endsWith('/v1/diagnostic-v3')).flatMap(c => c.body.events);
  assert.ok(sent.some(e => e.actionId === id));
  assert.equal(restored.W.status().pending, 0);
});

test('network diagnostics observes a rejected CDN without changing the downloader filter', async () => {
  const h = harness(); await h.send({ type: 'APOCALIPSE_DIAGNOSTIC_CONTROL', command: 'start' });
  const request = { tabId: 7, frameId: 3, requestId: 'R1', url: 'https://v.tiktokcdn-eu.com/opaque',
    responseHeaders: [{ name: 'Content-Type', value: 'video/mp4' }], statusCode: 206, type: 'xmlhttprequest' };
  h.network.forEach(fn => fn(request)); await settle(); await settle();
  // Force serialization to finish before reading persisted evidence.
  await h.D.emit('test.barrier');
  const rows = h.values.admDiagnosticOutboxV3.pending;
  assert.ok(rows.some(e => e.event === 'network.media_response_observed'));
  assert.ok(rows.some(e => e.event === 'network.capture_filter_decision' && e.data.accepted === false));
  const selected = await h.send({ type: 'APOCALIPSE_RECENT_TAB_MEDIA', tabId: 7 });
  assert.equal(selected.media.length, 0, 'instrumentation must not silently change routing');
});

test('detailed collection is restricted to chosen tab and stops at expiry', async () => {
  const h = harness(); await h.send({ type: 'APOCALIPSE_DIAGNOSTIC_CONTROL', command: 'start' });
  assert.equal(h.W.scoped(7), true); assert.equal(h.W.scoped(8), false);
  const response = await h.send({ type: 'APOCALIPSE_DIAGNOSTIC_CONTROL', command: 'start' }, { tab: { id: 7 }, url: 'https://www.tiktok.com/' });
  assert.equal(response.ok, false);
  h.D.setSession({ ...h.D.session(), expiresAt: Date.now() - 1 }); assert.equal(h.W.scoped(7), false);
});

test('sender metadata overrides forged frame metadata and storage errors do not break handoff', async () => {
  const h = harness({ pairingToken: 'synthetic' }); await h.D.ready;
  await h.send({ type: 'APOCALIPSE_DIAGNOSTIC_EVENT_V3', record: {
    eventId: webcrypto.randomUUID(), producerId: webcrypto.randomUUID(), event: 'test.sender',
    component: 'desktop', tabId: 999, frameId: 999, level: 'INFO', data: {} } }, { tab: { id: 7 }, frameId: 2 });
  const row = h.values.admDiagnosticOutboxV3.pending.find(e => e.event === 'test.sender');
  assert.equal(row.tabId, 7); assert.equal(row.frameId, 2); assert.equal(row.component, 'extension.content');
  h.failStorage(); await h.D.emit('test.storage_error'); assert.ok(h.W.status().storageErrors > 0);
  const traceId = webcrypto.randomUUID();
  const result = await h.send({ type: 'APOCALIPSE_DOWNLOAD', item: { url: 'https://cdn.example/file.mp4', kind: 'video', traceId } });
  assert.equal(result.ok, true);
  const sent = h.calls.find(c => c.url.endsWith('/v1/download')); assert.equal(sent.body.traceId, traceId);
});

test('each click has its own action while handlers share that click identifier', async () => {
  const h = harness(); await h.D.ready;
  const button = {};
  const first = h.D.beginAction(button); assert.equal(h.D.actionFor(button), first);
  const second = h.D.beginAction(button); assert.notEqual(first, second); assert.equal(h.D.actionFor(button), second);
});

test('the complete manifest includes diagnostics before each isolated content chain', () => {
  const manifest = JSON.parse(read('manifest.json'));
  for (const entry of manifest.content_scripts.filter(e => e.js.includes('content.js'))) {
    assert.ok(entry.js.indexOf('diagnostics.js') < entry.js.indexOf('content.js'));
    assert.ok(entry.js.includes('diagnostics-content.js'));
  }
  for (const script of manifest.content_scripts.flatMap(e => e.js)) assert.doesNotThrow(() => read(script));
});
