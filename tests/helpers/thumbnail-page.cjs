const assert = require('node:assert/strict');
const { test } = require('node:test');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { webcrypto } = require('node:crypto');
const vm = require('node:vm');

// Executes the production content.js. Browser/DOM APIs are simulated; its URL,
// thumbnail and inventory code is not rewritten or copied into the tests.
const sourceFile = process.env.ADM_CONTENT_SCRIPT || join(__dirname, '../../browser-extension/content.js');
const script = readFileSync(sourceFile, 'utf8');

function page({ url = 'https://www.facebook.com/', source = '', poster = '', image = '', count = 1, beforeCapture = null } = {}) {
  const sent = [], listeners = [], videos = [], timers = [];
  const events = {};
  const location = new URL(url);
  const rect = { left: 20, top: 40, right: 500, bottom: 600, width: 480, height: 560 };
  const cover = { tagName: 'IMG', currentSrc: image, src: image, getBoundingClientRect: () => rect };
  for (let i = 0; i < count; i += 1) {
    const parent = { tagName: 'DIV', parentElement: null, getBoundingClientRect: () => rect,
      querySelectorAll: selector => selector === 'video' ? [videos[i]] : selector === 'img' && image ? [cover] : [] };
    videos.push({ tagName: 'VIDEO', dataset: {}, isConnected: true, currentSrc: source, src: source,
      srcObject: source ? null : {}, title: `Synthetic video ${i}`, poster, duration: 30 + i,
      parentElement: parent, innerHTML: '', getBoundingClientRect: () => rect,
      addEventListener(name, cb) { (events[name] ||= []).push(cb); },
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
    setTimeout: (callback, delay) => { timers.push({ callback, delay }); return timers.length; }, clearTimeout() {}, setInterval: () => 1, clearInterval() {},
    MutationObserver: class { observe() {} },
    getComputedStyle: () => ({ backgroundImage: 'none' }),
    performance: { getEntriesByType: () => [], getEntriesByName: () => [] },
    chrome: {
      storage: { local: { get(defaults, callback) { callback(defaults); } }, onChanged: { addListener() {} } },
      runtime: {
        onMessage: { addListener(handler) { listeners.push(handler); } },
        sendMessage(message, callback) {
          sent.push(message);
          if (message.type === 'APOCALIPSE_CAPTURE_VISIBLE_THUMBNAIL') beforeCapture?.({ videos, rect, context });
          const result = message.type === 'APOCALIPSE_CAPTURE_VISIBLE_THUMBNAIL'
            ? { dataUrl: 'data:image/jpeg;base64,c3ludGhldGlj' } : { ok: true };
          if (callback) callback(result);
          return Promise.resolve(result);
        },
      },
    },
  });
  context.window = context; context.top = context;
  const instrumented = script.replace(/\}\)\(\);\s*$/, 'globalThis.testHooks = { absolute, collect, pageThumbnail, captureThumbnailFor, playerIdentity, rememberMedia, scheduleCatalog };\n})();');
  assert.notEqual(instrumented, script, 'production closure must expose test hooks');
  vm.runInContext(instrumented, context);
  return { hooks: context.testHooks, video: videos[0], videos, sent, location, document, context, events, rect, listeners, timers };
}


module.exports = { page };
