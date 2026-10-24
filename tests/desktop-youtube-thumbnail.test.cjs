const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs'), vm = require('node:vm');
const app = fs.readFileSync('apps/desktop/ui/app.js', 'utf8');
const worker = fs.readFileSync('browser-extension/background.js', 'utf8');
const tick = () => new Promise(resolve => setImmediate(resolve));
test('desktop tries YouTube fallback after an image fails to decode', async () => {
  const calls = [];
  const context = vm.createContext({ Date, Math, Set, thumbnailDataCache: new Map(), thumbnailRetryAfter: new Map(), resolveCachedThumbnail: async url => { calls.push(url); return `data:image/png;base64,${url}`; } });
  vm.runInContext(app.slice(app.indexOf('function loadPreviewThumbnail('), app.indexOf('let lastDownloadRenderSignature')), context);
  const image = { dataset: {}, removeAttribute(key) { delete this[key]; } };
  context.loadPreviewThumbnail(image, 'unsupported', 'youtube'); await tick();
  image.onerror(); await tick();
  assert.deepEqual(calls, ['unsupported', 'youtube']);
  image.onload(); assert.equal(image.hidden, false);
});
test('stale thumbnail resolution cannot replace a newer video cover', async () => {
  let release;
  const context = vm.createContext({ Date, Math, Set, thumbnailDataCache: new Map(), thumbnailRetryAfter: new Map(), resolveCachedThumbnail: url => url === 'old' ? new Promise(resolve => { release = resolve; }) : Promise.resolve('new-image') });
  vm.runInContext(app.slice(app.indexOf('function loadPreviewThumbnail('), app.indexOf('let lastDownloadRenderSignature')), context);
  const image = { dataset: {}, removeAttribute(key) { delete this[key]; } };
  context.loadPreviewThumbnail(image, 'old'); context.loadPreviewThumbnail(image, 'new'); await tick();
  release('old-image'); await tick(); assert.equal(image.src, 'new-image');
});
test('extension retains remote URL when the portable image exceeds the bridge limit', async () => {
  const context = vm.createContext({ fetchThumbnailDataUrl: async () => 'data:image/jpeg;base64,' + 'A'.repeat(190*1024) });
  vm.runInContext(worker.slice(worker.indexOf('const portableThumbnail'), worker.indexOf('const blobDataUrl')) + '\nthis.portable = portableThumbnail;', context);
  const url = 'https://i.ytimg.com/vi/abcdefghijk/hqdefault.jpg';
  assert.equal(await context.portable(url), url);
  assert.equal(await context.portable('data:image/jpeg;base64,' + 'A'.repeat(190*1024)), null);
});
