const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { webcrypto } = require('node:crypto');
const { test } = require('node:test');
const vm = require('node:vm');

function worker() {
  const listeners = [];
  const requests = [];
  const event = { addListener() {} };
  const storage = { get: async () => ({ pairingToken: 'synthetic-test-token' }), set: async () => {}, remove: async () => {} };
  const context = vm.createContext({
    URL, crypto: webcrypto, console, navigator: { userAgent: 'Test browser' },
    chrome: {
      runtime: { onMessage: { addListener(fn) { listeners.push(fn); } }, onInstalled: event, onStartup: event, getManifest: () => ({ version: 'test' }) },
      alarms: { create() {}, onAlarm: event }, webRequest: { onResponseStarted: event },
      downloads: { onDeterminingFilename: event, onChanged: event },
      storage: { local: storage, session: storage },
      cookies: { getAll: async () => [] },
      tabs: { query: async () => [{ url: 'https://page.example/' }] },
    },
    fetch: async (url, options = {}) => {
      let taskId;
      if (url.endsWith('/v1/download')) {
        const payload = JSON.parse(options.body);
        requests.push(payload);
        taskId = `task-${requests.length}`;
        await new Promise(resolve => setImmediate(resolve));
      }
      return { ok: true, json: async () => ({ ok: true, taskId }) };
    },
  });
  vm.runInContext(readFileSync(join(__dirname, '../browser-extension/background.js'), 'utf8'), context);
  const send = (message, pageUrl = 'https://page.example/') => new Promise((resolve, reject) => {
    if (listeners[0](message, { tab: { id: 1, url: pageUrl } }, resolve) !== true) reject(new Error('message not handled'));
  });
  return { context, requests, send, name: item => context.mediaDownloadFileName(item) };
}

const signed = index => `https://media.example/video/opaque-${index}/?a=1988&&mime_type=video_mp4&signature=a%2Bb%3D`;

test('media names preserve container, supplied names and signed URLs', () => {
  const { name } = worker();
  const item = { url: signed(1), kind: 'video', title: 'Clip' };
  assert.equal(name(item), 'Clip.mp4');
  assert.equal(item.url, signed(1));
  assert.equal(name({ ...item, fileName: 'chosen.mkv' }), 'chosen.mkv');
  assert.equal(name({ url: 'https://media.example/opaque/', contentType: 'video/webm; codecs=vp9', title: 'Clip' }), 'Clip.webm');
  assert.equal(name({ url: 'https://media.example/opaque/', contentType: 'audio/mp4', kind: 'audio' }), 'audio.m4a');
  assert.equal(name({ url: 'https://media.example/file', kind: 'video' }), null);
  assert.equal(name({ ...item, title: '../bad: name?' }), '.._bad_ name_.mp4');
  assert.equal(name({ ...item, title: 'CON' }), '_CON.mp4');
});

test('batch handoff retains a separate name, source and metadata for every item', async () => {
  const { requests, send } = worker();
  const items = [0, 1, 2].map(index => ({ url: signed(index), title: 'Clip', kind: 'video', size: 1000 + index, audioUrl: `https://media.example/audio/${index}`, thumbnail: `https://media.example/image/${index}.jpg` }));
  const original = JSON.stringify(items);
  const result = await send({ type: 'APOCALIPSE_DOWNLOAD_BATCH', items });
  assert.equal(result.ok, true);
  assert.equal(result.taskIds.length, 3);
  assert.equal(new Set(result.taskIds).size, 3);
  assert.equal(requests.length, 3);
  requests.forEach((payload, index) => {
    assert.equal(payload.url, items[index].url);
    assert.equal(payload.fileName, 'Clip.mp4'); // desktop atomically reserves suffixes
    assert.equal(payload.audioUrl, items[index].audioUrl);
    assert.equal(payload.thumbnail, items[index].thumbnail);
    assert.equal(payload.expectedSize, items[index].size);
    assert.equal(payload.startImmediately, true);
  });
  assert.equal(JSON.stringify(items), original);
});

test('overlapping batches and a single request do not share mutable request state', async () => {
  const { requests, send } = worker();
  const group = offset => [0, 1].map(i => ({ url: signed(offset + i), kind: 'video', title: `Clip ${offset + i}` }));
  const results = await Promise.all([
    send({ type: 'APOCALIPSE_DOWNLOAD_BATCH', items: group(0) }, 'https://page.example/a'),
    send({ type: 'APOCALIPSE_DOWNLOAD_BATCH', items: group(2) }, 'https://page.example/b'),
    send({ type: 'APOCALIPSE_DOWNLOAD', item: { url: signed(4), title: 'Single', kind: 'video' } }, 'https://page.example/c'),
  ]);
  assert.ok(results.every(result => result.ok));
  assert.equal(requests.length, 5);
  assert.equal(new Set(requests.map(payload => payload.url)).size, 5);
  for (let index = 0; index < 5; index++) {
    const payload = requests.find(item => item.url === signed(index));
    assert.equal(payload.fileName, index === 4 ? 'Single.mp4' : `Clip ${index}.mp4`);
    assert.equal(payload.pageUrl, `https://page.example/${index < 2 ? 'a' : index < 4 ? 'b' : 'c'}`);
    assert.equal(payload.startImmediately, index !== 4);
  }
});
