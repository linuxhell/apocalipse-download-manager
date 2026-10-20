const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

const ui = path.resolve(__dirname, '../apps/desktop/ui');
const html = fs.readFileSync(path.join(ui, 'index.html'), 'utf8');
const script = fs.readFileSync(path.join(ui, 'nibiru-media.js'), 'utf8');
const plain = value => JSON.parse(JSON.stringify(value));

function mediaFixture() {
  const ids = new Map();
  for (const match of html.matchAll(/<([a-z][\w-]*)\b([^>]*\bid="(nibiru-[^"]+)"[^>]*)>/gi)) {
    ids.set(match[3], {
      value: match[2].match(/\bvalue="([^"]*)"/)?.[1] || '',
      textContent: '', hidden: false, disabled: false,
    });
  }
  const get = id => {
    assert.ok(ids.has(id), `Missing media element: ${id}`);
    return ids.get(id);
  };
  const select = get('nibiru-operation');
  const options = html.match(/<select id="nibiru-operation">([\s\S]*?)<\/select>/)[1];
  select.options = [...options.matchAll(/<option value="([^"]+)">([^<]*)<\/option>/g)]
    .map(match => ({ value: match[1], textContent: match[2] }));
  select.value = select.options[0].value;
  const dialog = get('nibiru-media-dialog');
  const dialogHtml = html.match(/<dialog id="nibiru-media-dialog">([\s\S]*?)<\/dialog>/)[1];
  const buttons = [...dialogHtml.matchAll(/<button\b[^>]*\bid="([^"]+)"/g)].map(match => get(match[1]));
  dialog.open = false;
  dialog.showModal = () => { dialog.open = true; };
  dialog.close = () => { dialog.open = false; };
  dialog.querySelectorAll = selector => {
    assert.equal(selector, 'button');
    return buttons;
  };

  const cache = new Map([['apocalipse.language', 'en']]);
  const document = { documentElement: { lang: 'en' }, getElementById: get };
  const listeners = new Map();
  const calls = [];
  const window = {
    addEventListener(name, handler) {
      if (!listeners.has(name)) listeners.set(name, []);
      listeners.get(name).push(handler);
    },
    dispatchEvent(event) {
      for (const handler of listeners.get(event.type) || []) handler(event);
    },
    __TAURI__: { core: { invoke: async (command, args) => {
      calls.push(plain({ command, args }));
      return { ok: true };
    } } },
  };
  vm.runInNewContext(script, {
    window, document, localStorage: { getItem: key => cache.get(key) || null },
  }, { filename: 'nibiru-media.js' });
  return {
    get, calls,
    choose(operation) {
      assert.ok(select.options.some(option => option.value === operation));
      select.value = operation;
      select.onchange();
    },
    language(locale) {
      // The main window saves the language before dispatching this DOM event.
      cache.set('apocalipse.language', locale);
      document.documentElement.lang = locale;
      window.dispatchEvent({ type: 'apocalipse-language-changed', detail: { language: locale } });
    },
  };
}

const translations = {
  en: { open: 'Nibiru media', start: 'Start frame', gain: 'Gain (dB)', peak: 'Target peak (dBFS)', normalize: 'WAV peak normalization' },
  'pt-BR': { open: 'Mídia Nibiru', start: 'Frame inicial', gain: 'Ganho (dB)', peak: 'Pico alvo (dBFS)', normalize: 'Normalizar pico WAV' },
  'zh-CN': { open: 'Nibiru 媒体', start: '起始帧', gain: '增益 (dB)', peak: '目标峰值 (dBFS)', normalize: 'WAV 峰值归一化' },
};

test('media language changes before opening and while open preserve the selected operation and input', () => {
  const f = mediaFixture();
  f.choose('wav-normalize');
  f.get('nibiru-input').value = '/media/input.wav';
  f.get('nibiru-output').value = '/media/output.wav';
  f.get('nibiru-start').value = '-1.5';
  f.get('nibiru-frames').value = '25';
  for (const locale of ['pt-BR', 'zh-CN', 'en']) {
    f.language(locale);
    assert.equal(f.get('nibiru-media-open').textContent, translations[locale].open);
    assert.equal(f.get('nibiru-start-label').textContent, translations[locale].peak);
    assert.equal(f.get('nibiru-operation').options.find(option => option.value === 'wav-normalize').textContent, translations[locale].normalize);
    assert.equal(f.get('nibiru-operation').value, 'wav-normalize');
    assert.equal(f.get('nibiru-input').value, '/media/input.wav');
    assert.equal(f.get('nibiru-output').value, '/media/output.wav');
    assert.equal(f.get('nibiru-start').value, '-1.5');
    assert.equal(f.get('nibiru-frames').value, '25');
    assert.equal(f.get('nibiru-media-dialog').open, locale !== 'pt-BR');
    if (locale === 'pt-BR') f.get('nibiru-media-open').onclick();
  }
  assert.deepEqual(f.calls, []);
});

for (const [locale, text] of Object.entries(translations)) {
  test(`media parameter labels and typed requests follow the operation in ${locale}`, async () => {
    const f = mediaFixture();
    f.language(locale);
    const cases = [
      { operation: 'wav-cut', value: '7', frames: '19', label: text.start, parameters: { start: 7, frames: 19 } },
      { operation: 'y4m-cut', value: '3', frames: '9', label: text.start, parameters: { start: 3, frames: 9 } },
      { operation: 'wav-gain', value: '2.5', frames: '19', label: text.gain, parameters: { db: 2.5 } },
      { operation: 'wav-normalize', value: '-1.5', frames: '19', label: text.peak, parameters: { peak_db: -1.5 } },
      ...['flac-encode', 'flac-decode', 'qoa-encode', 'qoa-decode'].map(operation => ({ operation, value: '-1.5', frames: '0', parameters: {} })),
    ];
    for (const row of cases) {
      f.choose(row.operation);
      const pcmExtension = row.operation === 'y4m-cut' ? 'y4m' : 'wav';
      const input = `/media/input.${row.operation.endsWith('-decode') ? row.operation.split('-')[0] : pcmExtension}`;
      const output = `/media/output.${row.operation.endsWith('-encode') ? row.operation.split('-')[0] : pcmExtension}`;
      f.get('nibiru-input').value = input;
      f.get('nibiru-output').value = output;
      f.get('nibiru-start').value = row.value;
      f.get('nibiru-frames').value = row.frames;
      if (row.label) assert.equal(f.get('nibiru-start-label').textContent, row.label, row.operation);
      for (const id of ['nibiru-start-label', 'nibiru-start']) assert.equal(f.get(id).hidden, !row.label, row.operation);
      for (const id of ['nibiru-frames-label', 'nibiru-frames']) assert.equal(f.get(id).hidden, !('frames' in row.parameters), row.operation);
      for (const id of ['nibiru-output-label', 'nibiru-output']) assert.equal(f.get(id).hidden, false, row.operation);
      const before = f.calls.length;
      await f.get('nibiru-media-run').onclick();
      assert.equal(f.calls.length, before + 1, row.operation);
      assert.deepEqual(f.calls.at(-1), { command: 'process_nibiru_media', args: { request: { operation: row.operation, input, output, ...row.parameters } } });
    }
  });
}
