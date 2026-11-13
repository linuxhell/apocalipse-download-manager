const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { webcrypto } = require('node:crypto');
const vm = require('node:vm');
const root = process.env.ADM_EXTENSION_ROOT || join(__dirname, '../browser-extension');
const source = file => readFileSync(join(root, file), 'utf8');
const id = '11111111-2222-4333-8444-555555555555';
const signed = `http://tlu.dl.delivery.mp.microsoft.com/filestreamingservice/files/${id}?P1=123&P2=404&P3=2&P4=a%2Bb%3D&&x=a+b`;
const fileName = 'Windows11.0-KB1234567-x64.msu';
const pageUrl = `https://uupdump.net/getfile.php?id=synthetic&file=${encodeURIComponent(fileName)}`;

function worker({ offline = false, firefox = false } = {}) {
  const requests = [], cancelled = [], session = {}, handlers = { messages: [] };
  const event = { addListener() {} };
  const storage = { get: async () => ({ pairingToken: 'synthetic' }), set: async () => {}, remove: async () => {} };
  const context = vm.createContext({ URL, crypto: webcrypto, console, navigator: { userAgent: 'Synthetic browser' },
    Date, setTimeout: () => 1, clearTimeout() {},
    chrome: {
      runtime: { onMessage: { addListener(fn) { handlers.messages.push(fn); } }, onInstalled: event, onStartup: event,
        getManifest: () => ({ version: 'test' }) },
      storage: { local: storage, session: { ...storage, set: async values => Object.assign(session, values) } },
      alarms: { create() {}, onAlarm: event }, cookies: { getAll: async () => [] },
      webRequest: { onResponseStarted: event }, tabs: { query: async () => [], get: async () => ({ url: pageUrl }) },
      downloads: {
        onDeterminingFilename: firefox ? undefined : { addListener(fn) { handlers.filename = fn; } },
        onCreated: { addListener(fn) { handlers.created = fn; } }, onChanged: event,
        cancel(id, cb) { cancelled.push(id); cb(); }, erase(_args, cb) { cb(); },
      },
    },
    fetch: async (url, options = {}) => {
      assert.ok(url.startsWith('http://127.0.0.1:17654/'), 'extension must not consume the signed CDN response');
      requests.push({ url, body: options.body && JSON.parse(options.body) });
      if (offline) throw new Error('bridge unavailable');
      return { ok: true, status: 202, json: async () => ({ ok: true, taskId: 'synthetic-task' }) };
    },
  });
  if (firefox) vm.runInContext(source('download-url-policy.js'), context);
  else context.importScripts = (...files) => files.filter(f => f === 'download-url-policy.js')
    .forEach(file => vm.runInContext(source(file), context));
  vm.runInContext(source('background.js'), context);
  return { context, requests, cancelled, session,
    send: message => new Promise(resolve => {
      let handled = false, replied = false;
      for (const listener of handlers.messages) {
        handled = listener(message, { tab: { id: 1, url: pageUrl } }, value => { replied = true; resolve(value); }) === true || handled;
      }
      if (!handled && !replied) resolve(undefined);
    }),
    determine: item => firefox ? (handlers.created(item), tick())
      : new Promise(resolve => handlers.filename(item, resolve)),
  };
}

function page(w, url = pageUrl) {
  const handlers = {}, navigations = [];
  const context = vm.createContext({ URL, crypto: webcrypto, console, location: Object.assign(new URL(url), {
    assign(value) { navigations.push(value); },
  }), navigator: { language: 'en', userAgent: 'Synthetic browser' },
    document: { querySelectorAll: () => [], querySelector: () => null,
      addEventListener(type, handler) { handlers[type] = handler; }, removeEventListener() {},
      documentElement: { append() {} }, createElement: () => ({}) },
    MutationObserver: class { observe() {} },
    addEventListener() {}, postMessage() {}, setTimeout: () => 1, clearTimeout() {}, setInterval: () => 1,
    performance: { getEntriesByType: () => [] },
    chrome: { storage: { local: { get(defaults, cb) { cb(defaults); } }, onChanged: { addListener() {} } },
      runtime: { id: 'synthetic-extension', onMessage: { addListener() {} }, sendMessage: w.send } },
  });
  context.window = context; context.top = context;
  vm.runInContext(source('download-url-policy.js'), context);
  vm.runInContext(source('content.js'), context);
  return { navigations, click(href = signed, modifiers = {}) {
    let prevented = false;
    const anchor = { tagName: 'A', href, textContent: 'Download update', hasAttribute: () => false,
      getAttribute(name) { return name === 'href' ? href : null; } };
    handlers.click({ button: 0, target: { closest: selector => selector.includes('magnet:') ? null : anchor },
      composedPath: () => [anchor], preventDefault() { prevented = true; }, stopImmediatePropagation() {}, ...modifiers });
    return prevented;
  } };
}
const tick = () => new Promise(resolve => setImmediate(resolve));
const downloads = w => w.requests.filter(r => r.url.endsWith('/v1/download'));

test('UUP MSU click opens the normal desktop destination with the exact signed URL and file name', async () => {
  const w = worker(), p = page(w);
  assert.equal(p.click(), true);
  await tick();
  assert.equal(downloads(w).length, 1);
  assert.equal(downloads(w)[0].body.url, signed);
  assert.equal(downloads(w)[0].body.fileName, fileName);
  assert.equal(downloads(w)[0].body.startImmediately, false);
  assert.equal(p.navigations.length, 0);
});

test('MSU filename is recovered when the browser supplies the opaque Microsoft UUID', async () => {
  const w = worker();
  await w.determine({ id: 7, url: signed, finalUrl: signed, filename: id, referrer: pageUrl });
  assert.equal(downloads(w).length, 1);
  assert.equal(downloads(w)[0].body.url, signed);
  assert.equal(downloads(w)[0].body.fileName, fileName);
  assert.deepEqual(w.cancelled, [7]);
});

test('real MSU filename from browser headers takes precedence over the page hint', async () => {
  const w = worker();
  await w.determine({ id: 8, url: signed, filename: 'server-name.msu', referrer: pageUrl });
  assert.equal(downloads(w).length, 1);
  assert.equal(downloads(w)[0].body.fileName, 'server-name.msu');
});

test('ordinary direct .msu links are captured, with bypass and same-origin navigation preserved', async () => {
  const w = worker(), p = page(w);
  assert.equal(p.click('https://downloads.example/update.msu'), true);
  assert.equal(p.click('https://downloads.example/update.msu', { altKey: true }), false);
  assert.equal(p.click('https://uupdump.net/update.msu'), false);
  await tick();
  assert.equal(downloads(w).length, 1);
});

test('bridge failure preserves the exact Microsoft link in browser fallback without cancelling it', async () => {
  const w = worker({ offline: true }), p = page(w);
  assert.equal(p.click(), true);
  await tick();
  assert.deepEqual(p.navigations, [signed]);
  await w.determine({ id: 9, url: signed, filename: id, referrer: pageUrl });
  assert.deepEqual(w.cancelled, []);
});

test('genuinely disposable UUID links still keep their original browser response, even when forced', async () => {
  const w = worker();
  await w.send({ type: 'APOCALIPSE_FORCE_NEXT', ttlMs: 20000 });
  await w.determine({ id: 10, url: `https://files.example/download/${id}`, filename: 'file.zip' });
  assert.equal(downloads(w).length, 0);
  assert.deepEqual(w.cancelled, []);
  assert.equal(Object.keys(w.session).length, 1);
});

test('Microsoft exception rejects lookalike hosts, other endpoints, credentials and unusual ports', () => {
  const w = worker();
  for (const url of [signed.replace('microsoft.com', 'microsoft.com.evil.example'),
    signed.replace('/filestreamingservice/files/', '/download/'), signed.replace('http://', 'ftp://'),
    signed.replace('http://', 'http://user:pass@'), signed.replace('microsoft.com/', 'microsoft.com:8080/')]) {
    assert.equal(w.context.ADM_DOWNLOAD_URLS.isMicrosoftUpdateFile(url), false, url);
  }
  assert.equal(w.context.ADM_DOWNLOAD_URLS.isMicrosoftUpdateFile(signed), true);
  assert.equal(w.context.ADM_DOWNLOAD_URLS.isMicrosoftUpdateFile(signed.replace('http:', 'https:')), true);
});

test('UUP hints reject unsafe names and unrelated pages or destinations', () => {
  const w = worker(), hint = w.context.ADM_DOWNLOAD_URLS.uupFileName;
  assert.equal(hint(pageUrl, signed), fileName);
  for (const name of ['../bad.msu', 'CON.msu', 'bad.msu\r\nHeader:x', 'wrong.exe', 'a'.repeat(241) + '.msu']) {
    assert.equal(hint(`https://uupdump.net/getfile.php?file=${encodeURIComponent(name)}`, signed), null);
  }
  assert.equal(hint(pageUrl.replace('uupdump.net', 'uupdump.net.evil.example'), signed), null);
  assert.equal(hint(pageUrl.replace('getfile.php', 'get.php'), signed), null);
  assert.equal(hint(pageUrl, signed.replace('microsoft.com', 'example.com')), null);
});

test('normal injection and recovery load the URL policy before the content handler', () => {
  const manifest = JSON.parse(source('manifest.json'));
  for (const chain of manifest.content_scripts.filter(s => s.js.includes('content.js'))) {
    assert.ok(chain.js.indexOf('download-url-policy.js') >= 0);
    assert.ok(chain.js.indexOf('download-url-policy.js') < chain.js.indexOf('content.js'));
  }
  assert.match(source('background.js'), /"download-url-policy.js"/);
});


test('Firefox classic background captures MSU through onCreated without onDeterminingFilename', async () => {
  for (const workflow of ['ci.yml', 'release.yml']) {
    assert.match(readFileSync(join(__dirname, '../.github/workflows', workflow), 'utf8'),
      /manifest\["background"\] = \{"scripts": \["download-url-policy\.js", worker\]\}/);
  }
  const w = worker({ firefox: true });
  await w.determine({ id: 11, url: signed, filename: id, referrer: pageUrl });
  assert.equal(downloads(w).length, 1);
  assert.equal(downloads(w)[0].body.fileName, fileName);
  assert.deepEqual(w.cancelled, [11]);
});
