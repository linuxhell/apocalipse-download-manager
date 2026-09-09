const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { webcrypto } = require('node:crypto');
const { test } = require('node:test');
const vm = require('node:vm');
function worker(rejectPreview = false, tabUrl = 'https://www.tiktok.com/') {
  const listeners = [], previews = [], downloads = [], lookups = [];
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
      if (url.endsWith('/v1/download')) downloads.push(JSON.parse(options.body));
      return { ok: !(preview && rejectPreview), status: rejectPreview ? 400 : 202,
        json: async () => preview && rejectPreview ? { ok: false, error: 'preview_player_start_failed:NotFound' } : { ok: true } };
    },
  });
  vm.runInContext(readFileSync(join(__dirname, '../browser-extension/background.js'), 'utf8'), context);
  return { previews, downloads, lookups, send: message => new Promise(resolve => listeners[0](message, { tab: { url: tabUrl } }, resolve)) };
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


test('Facebook and Instagram use separate exact-media cookie scopes and page context', async () => {
  const { send, previews, lookups } = worker();
  const entries = [
    ['https://video.xx.fbcdn.net/clip.mp4?sig=a%2Bb%3D&bytestart=1024&byteend=2047', 'https://www.facebook.com/reel/123'],
    ['https://scontent.cdninstagram.com/clip.mp4?sig=a%2Bb%3D&&x=%20&x=a+b', 'https://www.instagram.com/reel/123/'],
  ];
  const results = await Promise.all(entries.map(([url, pageUrl]) => send({ type: 'APOCALIPSE_PREVIEW_MEDIA', url, pageUrl, contentType: 'video/mp4' })));
  assert.ok(results.every(result => result.ok));
  for (const [url, pageUrl] of entries) {
    const payload = previews.find(item => item.url === url);
    assert.equal(payload.referer, pageUrl);
    assert.equal(payload.contentType, 'video/mp4');
    assert.equal(payload.cookieHeader, `sid=${new URL(url).hostname}`);
    assert.ok(lookups.some(item => item.url === url && !item.domain));
  }
  assert.equal(lookups.length, 2);
});
test('spoofed Facebook and Instagram CDN names never receive cookies', async () => {
  const { send, previews, lookups } = worker();
  for (const url of ['https://fbcdn.net.evil.example/video.mp4', 'https://cdninstagram.com.evil.example/video.mp4']) {
    assert.equal((await send({ type: 'APOCALIPSE_PREVIEW_MEDIA', url })).ok, true);
  }
  assert.equal(lookups.length, 0);
  assert.ok(previews.every(item => item.cookieHeader === null));
});

test('Instagram Reel popup downloads use the page extractor instead of a silent CDN video track', async () => {
  const pageUrl = 'https://www.instagram.com/reel/SyntheticId/';
  const directVideo = 'https://scontent.cdninstagram.com/clip.mp4?sig=synthetic';
  const { send, downloads, lookups } = worker(false, pageUrl);
  const result = await send({
    type: 'APOCALIPSE_DOWNLOAD',
    item: { url: directVideo, kind: 'video', title: 'Synthetic Reel' },
  });
  assert.equal(result.ok, true);
  assert.equal(downloads.length, 1);
  assert.equal(downloads[0].url, pageUrl);
  assert.equal(downloads[0].audioUrl, null);
  assert.ok(lookups.some(item => item.url === pageUrl));
  assert.ok(!lookups.some(item => item.url === directVideo));
});

test('popup routing leaves non-Instagram videos and Instagram images on their direct URLs', async () => {
  for (const [tabUrl, item] of [
    ['https://www.facebook.com/reel/123/', { url: 'https://video.fbcdn.net/clip.mp4', kind: 'video' }],
    ['https://www.instagram.com/reel/123/', { url: 'https://scontent.cdninstagram.com/poster.jpg', kind: 'image' }],
    ['https://instagram.com.evil.example/reel/123/', { url: 'https://cdn.example/clip.mp4', kind: 'video' }],
  ]) {
    const { send, downloads } = worker(false, tabUrl);
    assert.equal((await send({ type: 'APOCALIPSE_DOWNLOAD', item })).ok, true);
    assert.equal(downloads[0].url, item.url);
  }
});

test('popup rejects a social CDN track explicitly marked incomplete', async () => {
  const { send, downloads } = worker(false, 'https://www.tiktok.com/');
  const result = await send({
    type: 'APOCALIPSE_DOWNLOAD',
    item: { url: signed, kind: 'video', ambiguousSocialTrack: true },
  });
  assert.equal(result.ok, false);
  assert.match(result.error, /incomplete_social_media_track/);
  assert.equal(downloads.length, 0);
});

test('companion audio receives only cookies selected for its own exact URL', async () => {
  const { send, previews, lookups } = worker();
  const audioUrl = 'https://a.tiktokcdn.com/audio.m4a?token=audio';
  const result = await send({ type: 'APOCALIPSE_PREVIEW_MEDIA', url: signed, audioUrl, mediaKind: 'video' });
  assert.equal(result.ok, true);
  assert.equal(previews[0].audioUrl, audioUrl);
  assert.equal(previews[0].cookieHeader, 'sid=v16-webapp-prime.tiktok.com');
  assert.equal(previews[0].audioCookieHeader, 'sid=a.tiktokcdn.com');
  assert.equal(lookups.length, 2);
});
test('invalid companion URLs are rejected before looking up credentials', async () => {
  const { send, previews, lookups } = worker();
  assert.equal((await send({ type: 'APOCALIPSE_PREVIEW_MEDIA', url: signed, audioUrl: 'file:///secret' })).ok, false);
  assert.equal(previews.length, 0); assert.equal(lookups.length, 0);
});
test('Instagram popup preview and Download use the same page extraction identity', async () => {
  const url = 'https://scontent.cdninstagram.com/clip.mp4?signature=test';
  const pageUrl = 'https://www.instagram.com/reel/SyntheticId/';
  const { send, previews, downloads } = worker(false, pageUrl);
  await send({ type: 'APOCALIPSE_DOWNLOAD', item: { url, kind: 'video' } });
  await send({ type: 'APOCALIPSE_PREVIEW_MEDIA', url, pageUrl, mediaKind: 'video', contentType: 'video/mp4' });
  assert.equal(previews[0].url, downloads[0].url);
  assert.equal(previews[0].pageExtractor, true); assert.equal(previews[0].contentType, null);
});
