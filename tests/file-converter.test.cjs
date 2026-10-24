const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs'), vm = require('node:vm');
const { webcrypto } = require('node:crypto');
const source = fs.readFileSync('apps/desktop/ui/file-converter.js', 'utf8');
function fixture(respond = () => {}) {
  const elements = new Map(), calls = [], events = {};
  function element(id) {
    if (!elements.has(id)) elements.set(id, { id, value: '', hidden: false, disabled: false, children: [],
      replaceChildren(...children) { this.children = children; this.value = children[0]?.value ?? ''; },
      removeAttribute(key) { delete this[key]; }, addEventListener(name, fn) { this[name] = fn; },
      showModal() { this.open = true; }, close() { this.open = false; }, click() { this.onclick?.(); } });
    return elements.get(id);
  }
  const context = vm.createContext({ window: { __TAURI__: { event: { listen(name, fn) { events[name] = fn; } } } }, crypto: webcrypto, document: { getElementById: element, createElement: () => ({}) }, Set, console });
  vm.runInContext(source, context);
  const api = context.window.createAdmFileConverter({ t: key => key, resize() {}, report() {}, invoke: async (command, args) => { calls.push([command, args]); return respond(command, args) ?? (command === 'default_download_directory' ? '/downloads' : undefined); } });
  return { api, element, calls, events, catalogs: context.window.AdmConversionCatalogs };
}
const audio = { source: '/downloads/black.mp4', kind: 'audio', black_video: true, tracks: [{ index: 1, kind: 'audio', codec: 'aac', language: 'por' }, { index: 0, kind: 'video', codec: 'h264' }], formats: ['flac', 'mp3'] };
test('converter uses inspected formats, selected audio track, progress and original source', async () => {
  const f = fixture((cmd) => cmd === 'inspect_conversion_file' ? audio : cmd === 'convert_local_file' ? '/downloads/black.converted.mp3' : undefined);
  await f.api.open(audio.source);
  assert.deepEqual(f.element('conversion-format').children.map(n => n.value), ['flac', 'mp3']);
  assert.equal(f.element('conversion-black').hidden, false);
  assert.equal(f.element('conversion-start').disabled, false);
  f.element('conversion-format').value = 'mp3';
  await f.element('conversion-start').onclick();
  const request = f.calls.find(([cmd]) => cmd === 'convert_local_file')[1];
  assert.equal(request.audioTrack, 1); assert.equal(request.format, 'mp3'); assert.equal(request.directory, '/downloads');
  assert.equal(f.element('conversion-result').hidden, false);
  assert.equal(f.element('conversion-source').value, audio.source);
  assert.equal(f.element('conversion-progress').value, 100);
  f.element('conversion-source').oninput();
  assert.equal(f.element('conversion-start').disabled, true);
});
test('analysis cancellation prevents conversion and closing interrupts active analysis', async () => {
  let finish;
  const f = fixture(cmd => cmd === 'inspect_conversion_file' ? new Promise(resolve => { finish = resolve; }) : undefined);
  const opening = f.api.open(audio.source);
  assert.equal(f.element('conversion-close').disabled, true);
  let prevented = false;
  f.element('conversion-dialog').cancel({ preventDefault() { prevented = true; } });
  assert.ok(prevented);
  assert.equal(f.calls.at(-1)[0], 'cancel_file_conversion');
  finish(audio); await opening;
  assert.equal(f.element('conversion-start').disabled, true);
  assert.equal(f.element('conversion-status').textContent, 'conversionCancelled');
});
test('all converter messages have the same keys in three languages', () => {
  const f = fixture();
  for (const locale of ['pt-BR', 'zh-CN']) assert.deepEqual(Object.keys(f.catalogs[locale]).sort(), Object.keys(f.catalogs.en).sort());
});
