const assert = require('node:assert/strict');
const { test } = require('node:test');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { webcrypto } = require('node:crypto');
const vm = require('node:vm');

function worker({ duration = 19.933333, audioOnly = false } = {}) {
  const listeners = [], responses = [], navigations = [], updates = [], decisions = [], fetched = [];
  const event = { addListener() {} };
  const storage = { get: async () => ({ pairingToken: 'test' }), set: async () => {}, remove: async () => {} };
  const context = vm.createContext({ URL, crypto: webcrypto, console, navigator: { userAgent: 'Test' },
    ADM_DIAG_WORKER: { network: (details, accepted) => decisions.push({ url: details.url, accepted }), emit() {}, message() { return false; } },
    chrome: {
      runtime: { onMessage: { addListener(f) { listeners.push(f); } }, onInstalled: event, onStartup: event, getManifest: () => ({ version: 'test' }) },
      alarms: { create() {}, onAlarm: event }, storage: { local: storage, session: storage },
      webRequest: { onResponseStarted: { addListener(f) { responses.push(f); } }, onBeforeRequest: { addListener(f) { navigations.push(f); } } },
      downloads: { onDeterminingFilename: event, onChanged: event },
      tabs: { query: async () => [], onUpdated: { addListener(f) { updates.push(f); } } },
    },
    fetch: async (url) => {
      fetched.push(url);
      if (url.includes('/HLSPlaylist.m3u8')) return { ok: true, text: async () => '#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=100000\nHLS_720.m3u8\n' };
      if (url.includes('/HLS_720.m3u8')) return { ok: true, text: async () => `#EXTM3U\n#EXTINF:${duration},\nsegment.mp4\n#EXT-X-ENDLIST\n` };
      if (url.includes('/segment.mp4')) return new Response(new Uint8Array(16), { headers: { 'content-type': audioOnly ? 'audio/mp4' : 'video/mp4' } });
      return { ok: true, json: async () => ({ ok: true }) };
    },
  });
  vm.runInContext(readFileSync(join(__dirname, '../browser-extension/background.js'), 'utf8'), context);
  return {
    decisions, fetched,
    response(url, contentType) {
      responses[0]({ tabId: 11, frameId: 0, url, initiator: 'https://www.reddit.com/',
        responseHeaders: [{ name: 'Content-Type', value: contentType }], statusCode: 200 });
    },
    navigate() { for (const f of navigations) f({ tabId: 11, type: 'main_frame', url: 'https://www.reddit.com/other' }); },
    send(message, senderUrl = 'https://www.reddit.com/comments/abc123/') {
      return new Promise(resolve => listeners[0](message, { url: senderUrl, frameId: 0, tab: { id: 11, url: senderUrl } }, resolve));
    },
  };
}
const master = 'https://v.redd.it/commentvideo123/HLSPlaylist.m3u8?a=signed';

test('Reddit HLS MIME and MP4 responses enter the capture inventory; lookalike domains do not', async () => {
  const w = worker();
  w.response(master, 'application/x-mpegurl');
  w.response('https://v.redd.it/commentvideo123/DASH_720.mp4', 'video/mp4');
  w.response('https://v.redd.it.evil.example/commentvideo123/HLSPlaylist.m3u8', 'application/x-mpegurl');
  assert.deepEqual(w.decisions.map(x => x.accepted), [true, true, false]);
  const inventory = await w.send({ type: 'APOCALIPSE_RECENT_TAB_MEDIA' });
  assert.equal(inventory.media.length, 2);
  assert.ok(inventory.media.some(x => x.url === master));
});

test('all HLS MIME aliases are remembered with their exact URL and reset on navigation', async () => {
  const w = worker();
  for (const [index, mime] of ['application/x-mpegurl', 'application/vnd.apple.mpegurl', 'audio/x-mpegurl'].entries()) {
    w.response(`https://v.redd.it/commentvideo123/manifest${index}?signature=exact`, mime);
  }
  const before = await w.send({ type: 'APOCALIPSE_RECENT_HLS_MANIFESTS' });
  assert.equal(before.urls.length, 3);
  assert.ok(before.urls.every(x => x.endsWith('?signature=exact#adm.m3u8')));
  w.navigate();
  assert.equal((await w.send({ type: 'APOCALIPSE_RECENT_HLS_MANIFESTS' })).urls.length, 0);
});

test('Reddit master verification follows its video playlist and matches the actual player duration', async () => {
  const w = worker();
  const result = await w.send({ type: 'APOCALIPSE_VERIFY_REDDIT_HLS', url: master, duration: 19.933333 });
  assert.equal(result.ok, true);
  assert.equal(result.url, master);
  assert.equal(result.duration, 19.933333);
  assert.deepEqual(Array.from(result.requestUrls), [master, 'https://v.redd.it/commentvideo123/HLS_720.m3u8']);
  assert.ok(w.fetched.includes(master));
});

test('Reddit master verification rejects wrong durations and audio-only playlists', async () => {
  for (const options of [{ duration: 40 }, { audioOnly: true }]) {
    const w = worker(options);
    assert.equal((await w.send({ type: 'APOCALIPSE_VERIFY_REDDIT_HLS', url: master, duration: 19.933333 })).ok, false);
  }
});

test('Reddit verification accepts only native master URLs requested from a Reddit frame', async () => {
  const w = worker();
  for (const [url, sender] of [
    ['https://v.redd.it.evil.example/a/HLSPlaylist.m3u8', 'https://www.reddit.com/'],
    ['https://user:secret@v.redd.it/a/HLSPlaylist.m3u8', 'https://www.reddit.com/'],
    [master, 'https://example.com/'],
    ['https://v.redd.it/a/DASH_720.mp4', 'https://www.reddit.com/'],
  ]) assert.equal((await w.send({ type: 'APOCALIPSE_VERIFY_REDDIT_HLS', url, duration: 19.933333 }, sender)).ok, false);
  assert.ok(!w.fetched.some(x => x.includes('HLSPlaylist')));
});


test('Reddit worker accepts the linked asset master and resolves child URLs inside its asset path', async () => {
  const w = worker();
  const url = 'https://v.redd.it/link/1xwx0jm/asset/eu9esac1sz1i1/HLSPlaylist.m3u8?f=hd%2Cvp9&a=synthetic%3D%3D';
  const result = await w.send({ type: 'APOCALIPSE_VERIFY_REDDIT_HLS', url, duration: 19.933333 });
  assert.equal(result.ok, true);
  assert.equal(result.reason, 'video_duration_verified');
  assert.equal(result.url, url);
  assert.deepEqual(Array.from(result.requestUrls), [url, 'https://v.redd.it/link/1xwx0jm/asset/eu9esac1sz1i1/HLS_720.m3u8']);
  assert.ok(w.fetched.includes(url));
});

test('Reddit worker rejects malformed linked asset routes', async () => {
  const w = worker();
  for (const url of [
    'https://v.redd.it/link/abc123/asset/HLSPlaylist.m3u8',
    'https://v.redd.it/link/abc123/asset/def456/extra/HLSPlaylist.m3u8',
    'https://v.redd.it/link/abc123/asset/def456/CMAF_AUDIO_128.m3u8',
    'https://v.redd.it.evil.example/link/abc123/asset/def456/HLSPlaylist.m3u8',
  ]) assert.equal((await w.send({ type: 'APOCALIPSE_VERIFY_REDDIT_HLS', url, duration: 19.933333 })).ok, false);
});
