const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { webcrypto } = require('node:crypto');
const { test } = require('node:test');
const vm = require('node:vm');
function worker(rejectPreview = false) {
  const listeners = [], previews = [], lookups = [];
  const event = { addListener() {} };
  const storage = { get: async () => ({ pairingToken: 'test-token' }), set: async () => {}, remove: async () => {} };
  const context = vm.createContext({ URL, crypto: webcrypto, console, navigator: { userAgent: 'SyntheticBrowser/1.0' },
    chrome: {
      runtime: { onMessage: { addListener(fn) { listeners.push(fn); } }, onInstalled: event, onStartup: event, getManifest: () => ({ version: 'test' }) },
      alarms: { create() {}, onAlarm: event }, webRequest: { onResponseStarted: event },
      downloads: { onDeterminingFilename: event, onChanged: event }, storage: { local: storage, session: storage },
      cookies: { getAll: async details => { lookups.push(details); await new Promise(resolve => setImmediate(resolve)); return [{ name: 'sid', value: new URL(details.url).hostname }]; } },
      tabs: { query: async () => [{ url: 'https://unrelated.example/' }] },
    },
    fetch: async (url, options = {}) => {
      const preview = url.endsWith('/v1/preview-media');
      if (preview) previews.push(JSON.parse(options.body));
      return { ok: !(preview && rejectPreview), status: rejectPreview ? 400 : 202,
        json: async () => preview && rejectPreview ? { ok: false, error: 'preview_player_start_failed:NotFound' } : { ok: true } };
    },
  });
  vm.runInContext(readFileSync(join(__dirname, '../browser-extension/background.js'), 'utf8'), context);
  return { previews, lookups, send: message => new Promise(resolve => listeners[0](message, { tab: { url: 'https://www.tiktok.com/' } }, resolve)) };
}
const signed = 'https://v16-webapp-prime.tiktok.com/video/tos/synthetic/?a=1988&&signature=a%2Bb%3D&mime_type=video_mp4';
test('preserves the exact TikTok URL and only requests matching media cookies', async () => {
  const { send, previews, lookups } = worker();
  const result = await send({ type: 'APOCALIPSE_PREVIEW_MEDIA', url: signed, pageUrl: 'https://www.tiktok.com/', contentType: 'video/mp4' });
  assert.equal(result.ok, true); assert.equal(previews[0].url, signed);
  assert.equal(previews[0].referer, 'https://www.tiktok.com/');
  assert.equal(previews[0].cookieHeader, 'sid=v16-webapp-prime.tiktok.com');
  assert.equal(previews[0].contentType, 'video/mp4');
  assert.equal(lookups.length, 1); assert.equal(lookups[0].url, signed); assert.equal(lookups[0].domain, undefined);
});
test('overlapping previews keep snapshots when the popup changes', async () => {
  const { send, previews } = worker();
  const first = { type: 'APOCALIPSE_PREVIEW_MEDIA', url: signed, pageUrl: 'https://www.tiktok.com/?page=a' };
  const promise = send(first); first.url = 'https://wrong.example/'; first.pageUrl = 'https://wrong.example/';
  await Promise.all([promise, send({ type: 'APOCALIPSE_PREVIEW_MEDIA', url: 'https://v19.tiktokcdn.com/video.mp4', pageUrl: 'https://www.tiktok.com/?page=b' })]);
  assert.equal(previews.length, 2);
  const a = previews.find(item => item.url === signed);
  const b = previews.find(item => item.url === 'https://v19.tiktokcdn.com/video.mp4');
  assert.equal(a.referer, 'https://www.tiktok.com/?page=a'); assert.equal(a.cookieHeader, 'sid=v16-webapp-prime.tiktok.com');
  assert.equal(b.referer, 'https://www.tiktok.com/?page=b'); assert.equal(b.cookieHeader, 'sid=v19.tiktokcdn.com');
});
test('rejects query-only and unsafe URLs before starting a player', async () => {
  const { send, previews, lookups } = worker();
  for (const url of ['?a=1988&mime_type=video_mp4', 'file:///C:/secret', 'https://user:pass@v16.tiktok.com/', 'https://v16.tiktok.com/v\n--flag']) {
    const result = await send({ type: 'APOCALIPSE_PREVIEW_MEDIA', url });
    assert.equal(result.ok, false); assert.match(result.error, /invalid_preview_url/);
  }
  assert.equal(previews.length, 0); assert.equal(lookups.length, 0);
});
test('reports the native player error instead of a disconnected desktop', async () => {
  const { send } = worker(true);
  const result = await send({ type: 'APOCALIPSE_PREVIEW_MEDIA', url: signed });
  assert.equal(result.ok, false); assert.match(result.error, /preview_player_start_failed:NotFound/);
});
test('does not gather cookies for unrelated sites or spoofed TikTok hosts', async () => {
  const { send, previews, lookups } = worker();
  for (const url of ['https://example.org/video.mp4', 'https://tiktok.com.unrelated.example/video.mp4']) {
    assert.equal((await send({ type: 'APOCALIPSE_PREVIEW_MEDIA', url })).ok, true);
  }
  assert.equal(lookups.length, 0); assert.ok(previews.every(item => item.cookieHeader === null));
});
