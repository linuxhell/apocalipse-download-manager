const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');
const vm = require('node:vm');
const source = readFileSync(join(__dirname, '../browser-extension/background.js'), 'utf8');
const code = source.slice(source.indexOf('let thumbnailActivationSerial'), source.indexOf('const mediaPickerContexts'));
function worker({ active = 1, changeTab = false, cropFailure = false, noCanvas = false } = {}) {
  let captures = 0, closed = 0;
  const state = { active };
  const context = vm.createContext({ console, Date, Number, Math, Error, Promise, Map,
    tabNavigationEpochs: new Map(),
    blobDataUrl: async () => 'data:image/jpeg;base64,CROP',
    fetch: async () => ({ blob: async () => ({}) }),
    createImageBitmap: async () => ({ width: 1000, height: 800, close() { closed++; } }),
    OffscreenCanvas: noCanvas ? undefined : class {
      getContext() { return { drawImage() { if (cropFailure) throw Error('crop broke'); } }; }
      async convertToBlob() { return {}; }
    },
    chrome: { runtime: {}, tabs: {
      onActivated: { addListener(fn) { state.activation = fn; } },
      query: async () => [{ id: state.active, windowId: 9 }],
      get: async id => ({ id, windowId: 9 }),
      captureVisibleTab(id, options, callback) {
        captures++;
        if (changeTab) { state.active = 2; state.activation(); }
        callback('data:image/jpeg;base64,FULL_PAGE');
      },
    } },
  });
  vm.runInContext(code + '\nglobalThis.capture = captureVisibleThumbnail;', context);
  return { capture: (rect = { left: 20, top: 40, width: 480, height: 560 }, frameId = 0) => context.capture(
    { tab: { id: 1, windowId: 9 }, frameId }, rect, { width: 1000, height: 800 }),
    captures: () => captures, closed: () => closed };
}
test('worker captures only the requested active tab', async () => {
  const w = worker({ active: 2 });
  await assert.rejects(w.capture(), /thumbnail_tab_changed/);
  assert.equal(w.captures(), 0);
});
test('worker discards screenshot after a tab switch', async () => {
  const w = worker({ changeTab: true });
  await assert.rejects(w.capture(), /thumbnail_tab_changed/);
});
test('worker rejects unverified iframe coordinates before capture', async () => {
  const w = worker();
  await assert.rejects(w.capture(undefined, 1), /frame_coordinates_unverified/);
  assert.equal(w.captures(), 0);
});
test('worker never returns the full page when canvas cropping is unavailable', async () => {
  const w = worker({ noCanvas: true });
  await assert.rejects(w.capture(), /crop_unavailable/);
  assert.equal(w.captures(), 0);
});
test('worker closes the bitmap and rejects a failed crop instead of returning the whole page', async () => {
  const w = worker({ cropFailure: true });
  await assert.rejects(w.capture(), /crop broke/);
  assert.equal(w.closed(), 1);
});
for (const left of [-5, 1000, NaN, Infinity]) {
  test(`worker rejects invalid or clipped coordinates (${left})`, async () => {
    const w = worker();
    await assert.rejects(w.capture({ left, top: 0, width: 100, height: 80 }), /crop_not_fully_visible/);
    assert.equal(w.captures(), 0);
  });
}
test('worker returns a crop and bounds repeated capture requests', async () => {
  const w = worker();
  assert.equal(await w.capture(), 'data:image/jpeg;base64,CROP');
  assert.equal(w.closed(), 1);
  await assert.rejects(w.capture(), /capture_throttled/);
  assert.equal(w.captures(), 1);
});
