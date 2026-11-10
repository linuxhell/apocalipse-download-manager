const assert = require('node:assert/strict');
const { webcrypto } = require('node:crypto');
const { readFileSync } = require('node:fs');
const http = require('node:http');
const { join } = require('node:path');
const { test } = require('node:test');
const vm = require('node:vm');
const { gzipSync } = require('node:zlib');

const pageUrl = 'https://uupdump.example.test/download.php';
const downloadUrl = 'https://uupdump.example.test/get.php?id=synthetic';
const formBody = 'edition=professional&language=pt-br&option=a%2Bb';
const contentType = 'application/x-www-form-urlencoded;charset=UTF-8';
const payload = Buffer.from(Array.from({ length: 48 * 1024 + 17 }, (_, i) => i % 256));

async function downloadServer(t, { compressed = false, chunked = false, status = 200, dropResponse = false } = {}) {
  const requests = [];
  const server = http.createServer(async (req, res) => {
    const parts = [];
    for await (const part of req) parts.push(part);
    requests.push({ method: req.method, headers: req.headers, body: Buffer.concat(parts).toString() });
    if (dropResponse) { req.socket.destroy(); return; }
    // A generated download is available once; a repeated POST is rate limited.
    res.statusCode = requests.length === 1 ? status : 429;
    if (res.statusCode !== 200) { res.end('rate limited'); return; }
    const bytes = compressed ? gzipSync(payload) : payload;
    res.setHeader('Content-Type', 'archive/zip');
    res.setHeader('Content-Disposition', "attachment; filename*=UTF-8''test%20download.zip");
    if (compressed) res.setHeader('Content-Encoding', 'gzip');
    if (!chunked) res.setHeader('Content-Length', bytes.length);
    res.write(bytes.subarray(0, 123));
    res.end(bytes.subarray(123));
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  t.after(() => new Promise(resolve => { server.close(resolve); server.closeAllConnections(); }));
  return { requests, url: `http://127.0.0.1:${server.address().port}/get.php` };
}

function worker(server, { failBridgePath = null, pausedOnce = false } = {}) {
  const listeners = [], bridge = [], remoteOptions = [];
  const event = { addListener() {} };
  const storage = { get: async () => ({ pairingToken: 'synthetic-token' }), set: async () => {}, remove: async () => {} };
  let statusRequests = 0;
  const context = vm.createContext({
    URL, crypto: webcrypto, console, AbortSignal, setTimeout, clearTimeout,
    chrome: {
      runtime: { onMessage: { addListener(fn) { listeners.push(fn); } }, onInstalled: event, onStartup: event, getManifest: () => ({ version: 'test' }) },
      alarms: { create() {}, onAlarm: event }, webRequest: { onResponseStarted: event },
      downloads: { onDeterminingFilename: event, onChanged: event },
      storage: { local: storage, session: storage }, cookies: { getAll: async () => [] },
      tabs: { query: async () => [] },
    },
    fetch: async (url, options = {}) => {
      if (new URL(url).hostname === 'uupdump.example.test') {
        remoteOptions.push(options);
        return fetch(server.url, options);
      }
      assert.equal(new URL(url).origin, 'http://127.0.0.1:17654');
      const path = new URL(url).pathname;
      const body = options.body ? JSON.parse(options.body) : null;
      bridge.push({ path, body });
      if (path === failBridgePath) return { ok: false, status: 503 };
      const response = path === '/v1/blob/begin' ? { uploadId: 'synthetic-upload' }
        : path === '/v1/blob/status' ? { paused: pausedOnce && ++statusRequests === 1 }
        : { ok: true };
      return { ok: true, status: 200, json: async () => response };
    },
  });
  vm.runInContext(readFileSync(join(__dirname, '../browser-extension/background.js'), 'utf8'), context);
  const send = (method = 'POST', sourceUrl = pageUrl) => new Promise((resolve, reject) => {
    const message = { type: 'APOCALIPSE_PRE_DOWNLOAD_URL', url: downloadUrl, force: true,
      source: 'post-download-form', method, body: method === 'POST' ? formBody : null,
      contentType: method === 'POST' ? contentType : null };
    for (const listener of listeners) {
      if (listener(message, { tab: { id: 1, url: sourceUrl } }, resolve) === true) return;
    }
    reject(new Error('pre-download message not handled'));
  });
  const captured = () => Buffer.concat(bridge.filter(x => x.path === '/v1/blob/chunk').map(x => Buffer.from(x.body.data, 'hex')));
  return { context, bridge, remoteOptions, send, captured };
}

test('generated POST ZIP is streamed once into the ADM destination workflow', async t => {
  const server = await downloadServer(t);
  const { send, bridge, remoteOptions, captured } = worker(server);
  const result = await send();
  assert.equal(result.ok, true);
  assert.equal(result.bytes, payload.length);
  assert.equal(server.requests.length, 1);
  assert.equal(server.requests[0].method, 'POST');
  assert.equal(server.requests[0].body, formBody);
  assert.equal(server.requests[0].headers['content-type'], contentType);
  assert.equal(remoteOptions[0].credentials, 'include');
  assert.equal(remoteOptions[0].referrer, pageUrl);
  assert.deepEqual(captured(), payload);
  const begin = bridge.find(x => x.path === '/v1/blob/begin').body;
  assert.equal(begin.fileName, 'test download.zip');
  assert.equal(begin.total, payload.length);
  assert.equal(begin.promptForDestination, true);
  assert.equal(begin.recording, false);
  assert.equal(bridge.filter(x => x.path === '/v1/blob/end').length, 1);
  assert.ok(bridge.filter(x => x.path === '/v1/blob/chunk').every(x => x.body.data.length <= 32 * 1024));
  assert.ok(!bridge.some(x => x.path === '/v1/download'));
});

test('chunked POST and existing GET captures preserve all bytes and pause handling', async t => {
  for (const method of ['POST', 'GET']) {
    await t.test(method, async t => {
      const server = await downloadServer(t, { chunked: true });
      const { send, bridge, captured } = worker(server, { pausedOnce: true });
      const result = await send(method);
      assert.equal(result.ok, true);
      assert.equal(result.bytes, payload.length);
      assert.equal(server.requests.length, 1);
      assert.deepEqual(captured(), payload);
      assert.equal(bridge.find(x => x.path === '/v1/blob/begin').body.streaming, true);
      assert.ok(bridge.filter(x => x.path === '/v1/blob/status').length > 1);
    });
  }
});

test('decoded HTTP response uses the decoded byte count rather than compressed Content-Length', async t => {
  const server = await downloadServer(t, { compressed: true });
  const { send, bridge, captured } = worker(server);
  assert.equal((await send()).ok, true);
  assert.deepEqual(captured(), payload);
  const begin = bridge.find(x => x.path === '/v1/blob/begin').body;
  assert.equal(begin.total, 0);
  assert.equal(begin.streaming, true);
});

test('a failure after sending POST never permits automatic form replay', async t => {
  for (const failBridgePath of ['/v1/blob/begin', '/v1/blob/chunk', '/v1/blob/end']) {
    await t.test(failBridgePath, async t => {
      const server = await downloadServer(t);
      const { send, bridge } = worker(server, { failBridgePath });
      const result = await send();
      assert.equal(result.ok, false);
      assert.equal(result.replay, false);
      assert.match(result.error, /503/);
      assert.equal(server.requests.length, 1);
      assert.ok(!bridge.some(x => x.path === '/v1/download'));
    });
  }
});

test('a rate limited POST is reported without replay or another HTTP request', async t => {
  const server = await downloadServer(t, { status: 429 });
  const { send, bridge } = worker(server);
  const result = await send();
  assert.equal(result.ok, false);
  assert.equal(result.replay, false);
  assert.match(result.error, /http_429/);
  assert.equal(server.requests.length, 1);
  assert.ok(!bridge.some(x => x.path === '/v1/blob/begin'));
});

test('a lost response after the server receives POST never resubmits the form', async t => {
  const server = await downloadServer(t, { dropResponse: true });
  const { send, bridge } = worker(server);
  const result = await send();
  assert.equal(result.ok, false);
  assert.equal(result.replay, false);
  assert.equal(server.requests.length, 1);
  assert.ok(!bridge.some(x => x.path === '/v1/blob/begin'));
});

test('an offline ADM permits browser fallback before sending any POST', async t => {
  const server = await downloadServer(t);
  const { send } = worker(server, { failBridgePath: '/v1/health' });
  const result = await send();
  assert.equal(result.ok, false);
  assert.equal(result.replay, true);
  assert.equal(server.requests.length, 0);
});

function formHook() {
  const listeners = {}, posted = [];
  class HTMLElement {}
  class HTMLFormElement extends HTMLElement {
    method = 'POST';
    action = downloadUrl;
    isConnected = true;
    submitted = 0;
    requestSubmit() { this.submitted++; }
  }
  const context = vm.createContext({
    URL, URLSearchParams, HTMLElement, HTMLFormElement, File: class {}, queueMicrotask,
    location: { href: pageUrl, origin: new URL(pageUrl).origin },
    FormData: class { values() { return ['professional']; } entries() { return [['edition', 'professional']]; } },
    addEventListener(type, fn) { listeners[type] = fn; },
    document: { addEventListener(type, fn) { listeners[type] = fn; } },
    postMessage(message) { posted.push(message); },
  });
  context.window = context;
  vm.runInContext(readFileSync(join(__dirname, '../browser-extension/uupdump-post-fix.js'), 'utf8'), context);
  const form = new HTMLFormElement();
  listeners.submit({ target: form, preventDefault() {}, stopImmediatePropagation() {} });
  const result = value => listeners.message({ source: vm.runInContext('window', context), data: {
    source: 'apocalipse-extension', type: 'pre-download-result', requestId: posted[0].requestId, result: value,
  } });
  return { form, posted, result };
}

test('UUP form fallback honors consumed POST and explicit browser fallback responses', () => {
  const consumed = formHook();
  consumed.result({ ok: false, error: 'bridge_http_503', replay: false });
  assert.equal(consumed.form.submitted, 0);
  const offline = formHook();
  offline.result({ ok: false, replay: true });
  assert.equal(offline.form.submitted, 1);
  const bypass = formHook();
  bypass.result({ ok: false, bypass: true });
  assert.equal(bypass.form.submitted, 1);
  const success = formHook();
  success.result({ ok: true });
  assert.equal(success.form.submitted, 0);
});

test('the content relay forbids POST replay when the worker message channel closes', async () => {
  const listeners = {}, posted = [];
  const context = vm.createContext({
    URL, crypto: webcrypto, console, navigator: { language: 'en' }, location: new URL(pageUrl),
    document: { querySelectorAll: () => [], querySelector: () => null, addEventListener() {},
      documentElement: { append() {} }, createElement: () => ({}) },
    MutationObserver: class { observe() {} },
    addEventListener(type, fn) { listeners[type] = fn; },
    postMessage(message) { posted.push(message); },
    setTimeout: () => 1, clearTimeout() {}, setInterval: () => 1, clearInterval() {},
    chrome: {
      storage: { local: { get(defaults, callback) { callback(defaults); } }, onChanged: { addListener() {} } },
      runtime: { id: 'synthetic-extension', onMessage: { addListener() {} },
        sendMessage: message => message.type === 'APOCALIPSE_PRE_DOWNLOAD_URL'
          ? Promise.reject(new Error('message channel closed')) : Promise.resolve({ ok: true }) },
    },
  });
  context.window = context;
  context.top = context;
  vm.runInContext(readFileSync(join(__dirname, '../browser-extension/content.js'), 'utf8'), context);
  for (const method of ['POST', 'GET']) {
    listeners.message({ source: vm.runInContext('window', context), data: {
      source: 'apocalipse-page-hook', type: 'pre-download-url', requestId: method, url: downloadUrl, method,
    } });
    await new Promise(resolve => setImmediate(resolve));
    const reply = posted.find(x => x.type === 'pre-download-result' && x.requestId === method);
    assert.equal(reply.result.ok, false);
    assert.equal(reply.result.replay, method !== 'POST');
  }
});
