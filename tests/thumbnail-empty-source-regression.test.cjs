const assert = require('node:assert/strict');
const { test } = require('node:test');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { webcrypto } = require('node:crypto');
const vm = require('node:vm');

// Executes the production content.js. Browser/DOM APIs are simulated; its URL,
// thumbnail and inventory code is not rewritten or copied into the tests.
const sourceFile = process.env.ADM_CONTENT_SCRIPT || join(__dirname, '../browser-extension/content.js');
const script = readFileSync(sourceFile, 'utf8');

function page({ url = 'https://www.facebook.com/', source = '', poster = '', image = '', count = 1 } = {}) {
  const sent = [], listeners = [], videos = [];
  const location = new URL(url);
  const rect = { left: 20, top: 40, right: 500, bottom: 600, width: 480, height: 560 };
  const cover = { tagName: 'IMG', currentSrc: image, src: image, getBoundingClientRect: () => rect };
  for (let i = 0; i < count; i += 1) {
    const parent = { tagName: 'DIV', parentElement: null, getBoundingClientRect: () => rect,
      querySelectorAll: selector => selector === 'video' ? [videos[i]] : selector === 'img' && image ? [cover] : [] };
    videos.push({ tagName: 'VIDEO', dataset: {}, isConnected: true, currentSrc: source, src: source,
      srcObject: source ? null : {}, title: `Synthetic video ${i}`, poster, duration: 30 + i,
      parentElement: parent, innerHTML: '', getBoundingClientRect: () => rect,
      getAttribute: name => name === 'poster' ? poster || null : null,
      querySelector: () => null, querySelectorAll: () => [], closest: () => parent });
  }
  const document = {
    title: 'Synthetic feed', fullscreenElement: null,
    documentElement: { clientWidth: 1000, clientHeight: 800, append() {} },
    createElement: tag => ({ tagName: tag.toUpperCase(), style: {}, dataset: {}, addEventListener() {}, remove() {} }),
    addEventListener() {}, elementFromPoint: () => videos[0],
    querySelector: selector => selector === 'video' ? videos[0] : null,
    querySelectorAll: selector => selector === 'video' || selector === 'video,audio' ? videos : [],
  };
  const context = vm.createContext({
    URL, Blob, Uint8Array, crypto: webcrypto, console, Date, document, location,
    navigator: { language: 'pt-BR', userAgent: 'Synthetic browser' },
    innerWidth: 1000, innerHeight: 800, scrollX: 0, scrollY: 0,
    addEventListener() {}, removeEventListener() {}, postMessage() {},
    setTimeout: () => 1, clearTimeout() {}, setInterval: () => 1, clearInterval() {},
    MutationObserver: class { observe() {} },
    getComputedStyle: () => ({ backgroundImage: 'none' }),
    performance: { getEntriesByType: () => [], getEntriesByName: () => [] },
    chrome: {
      storage: { local: { get(defaults, callback) { callback(defaults); } }, onChanged: { addListener() {} } },
      runtime: {
        onMessage: { addListener(handler) { listeners.push(handler); } },
        sendMessage(message, callback) {
          sent.push(message);
          const result = message.type === 'APOCALIPSE_CAPTURE_VISIBLE_THUMBNAIL'
            ? { dataUrl: 'data:image/jpeg;base64,c3ludGhldGlj' } : { ok: true };
          if (callback) callback(result);
          return Promise.resolve(result);
        },
      },
    },
  });
  context.window = context; context.top = context;
  const instrumented = script.replace(/\}\)\(\);\s*$/, 'globalThis.testHooks = { absolute, collect, pageThumbnail, captureThumbnailFor };\n})();');
  assert.notEqual(instrumented, script, 'production closure must expose test hooks');
  vm.runInContext(instrumented, context);
  return { hooks: context.testHooks, video: videos[0], sent, location };
}

test('missing URL values never resolve into the document URL or /undefined', () => {
  const p = page();
  for (const value of ['', '  ', null, undefined]) assert.equal(p.hooks.absolute(value), null, `missing value: ${String(value)}`);
});

test('empty poster does not hide a valid cover from the exact player region', () => {
  const p = page({ image: 'https://images.example/current-cover.jpg' });
  assert.equal(p.hooks.pageThumbnail(p.video), 'https://images.example/current-cover.jpg');
});

for (const url of ['https://www.facebook.com/', 'https://www.tiktok.com/']) {
  test(`${new URL(url).hostname}: no cover yields an empty thumbnail, not HTML`, () => {
    const p = page({ url, source: 'blob:https://example.test/synthetic' });
    assert.equal(p.hooks.pageThumbnail(p.video), '');
  });
  test(`${new URL(url).hostname}: no cover reaches the exact-player screenshot fallback`, async () => {
    const p = page({ url, source: 'blob:https://example.test/synthetic' });
    assert.match(await p.hooks.captureThumbnailFor(p.video), /^data:image\/jpeg;base64,/);
    const capture = p.sent.filter(m => m.type === 'APOCALIPSE_CAPTURE_VISIBLE_THUMBNAIL');
    assert.equal(capture.length, 1);
    assert.equal(capture[0].rect.left, 20);
    assert.equal(capture[0].rect.width, 480);
  });
}

test('Facebook srcObject without a source stays visual-only instead of a Home-page video', () => {
  const p = page();
  const rows = p.hooks.collect().filter(item => item.kind === 'video');
  assert.equal(rows.length, 1);
  assert.equal(rows[0].visualOnly, true);
  assert.equal(rows[0].thumbnail, '');
  assert.notEqual(rows[0].url, p.location.href);
});

test('two Facebook srcObject players do not collapse onto the same Home URL', () => {
  const p = page({ count: 2 });
  const rows = p.hooks.collect().filter(item => item.kind === 'video');
  assert.equal(rows.length, 2);
  assert.notEqual(rows[0].url, rows[1].url);
  assert.ok(rows.every(item => item.visualOnly));
});

test('a valid poster is preserved and avoids unnecessary screenshot capture', async () => {
  const p = page({ poster: 'https://images.example/real-cover.jpg?signature=abc' });
  assert.equal(await p.hooks.captureThumbnailFor(p.video), 'https://images.example/real-cover.jpg?signature=abc');
  assert.equal(p.sent.filter(m => m.type === 'APOCALIPSE_CAPTURE_VISIBLE_THUMBNAIL').length, 0);
});

test('valid relative and signed absolute resource URLs retain their behavior', () => {
  const p = page();
  assert.equal(p.hooks.absolute('/images/cover.jpg'), 'https://www.facebook.com/images/cover.jpg');
  const signed = 'https://cdn.example/video.mp4?sig=a%2Fb&range=0-100#part';
  assert.equal(p.hooks.absolute(signed), signed);
});
