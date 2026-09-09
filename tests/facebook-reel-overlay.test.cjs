const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { webcrypto } = require('node:crypto');
const { test } = require('node:test');
const vm = require('node:vm');
const script = readFileSync(process.env.ADM_CONTENT_SCRIPT || join(__dirname, '../browser-extension/content.js'), 'utf8');

// Execute the real content script and its installed click handler. Only browser
// APIs/DOM geometry are mocked; URL selection and the outgoing payload are real.
function page({ url = 'https://www.facebook.com/reel/123456789', source = 'https://video.fbcdn.net/track.mp4?bytestart=0&byteend=999', permalink = null, network = [], readableBlob = false } = {}) {
  const sent = [], fetched = [], appended = [], listeners = [];
  const location = new URL(url);
  const rect = { left: 20, top: 40, right: 500, bottom: 600, width: 480, height: 560 };
  const post = { parentElement: null, getBoundingClientRect: () => rect, querySelectorAll: () => [] };
  const video = {
    tagName: 'VIDEO', dataset: {}, isConnected: true, currentSrc: source, src: source,
    title: 'Synthetic complete reel', poster: 'https://images.example/poster.jpg', duration: 30,
    parentElement: post, innerHTML: '', getBoundingClientRect: () => rect,
    getAttribute: () => null,
    querySelector: selector => selector.includes('a[href') && permalink ? { href: permalink } : null,
    querySelectorAll: () => [], closest: selector => selector.includes('article') ? post : null,
  };
  const element = tag => ({ tagName: tag.toUpperCase(), style: {}, dataset: {}, offsetWidth: 80,
    addEventListener(type, handler) { this[type] = handler; }, remove() { this.removed = true; } });
  const document = {
    title: 'Synthetic complete reel', fullscreenElement: null,
    documentElement: { append(node) { appended.push(node); } },
    createElement: element, addEventListener() {}, elementFromPoint: () => video,
    querySelector: selector => selector === 'video' ? video : null,
    querySelectorAll: selector => selector === 'video' || selector === 'video,audio' ? [video]
      : selector === '.apocalipse-media-download' ? appended.filter(node => !node.removed && String(node.className).includes('apocalipse-media-download')) : [],
  };
  const context = vm.createContext({
    URL, Blob, Uint8Array, crypto: webcrypto, console, Date, document, location,
    navigator: { language: 'pt-BR', userAgent: 'Synthetic browser' },
    innerHeight: 800, scrollX: 0, scrollY: 0,
    addEventListener() {}, removeEventListener() {}, postMessage() {},
    setTimeout: () => 1, clearTimeout() {}, setInterval: () => 1, clearInterval() {},
    MutationObserver: class { observe() {} },
    performance: { getEntriesByType: () => network.map(item => ({ name: item.url })), getEntriesByName: () => [] },
    fetch: async target => {
      fetched.push(target);
      if (!readableBlob) throw new Error('synthetic unreadable blob');
      return { ok: true, blob: async () => new Blob(['synthetic track'], { type: 'video/mp4' }) };
    },
    chrome: {
      storage: { local: { get(defaults, callback) { callback(defaults); } }, onChanged: { addListener() {} } },
      runtime: {
        onMessage: { addListener(handler) { listeners.push(handler); } },
        sendMessage(message, callback) {
          sent.push(message);
          const result = message.type === 'APOCALIPSE_RECENT_TAB_MEDIA' ? { media: network }
            : message.type === 'APOCALIPSE_BLOB_BEGIN' ? { uploadId: 'synthetic-upload' }
              : { ok: true, target: 'desktop' };
          if (callback) callback(result);
          return Promise.resolve(result);
        },
      },
    },
  });
  context.window = context; context.top = context;
  vm.runInContext(readFileSync(join(__dirname, '../browser-extension/tiktok-identity.js'), 'utf8'), context);
  vm.runInContext(script.replace(/\}\)\(\);\s*$/, 'globalThis.testHooks = { installOverlays, collect };\n})();'), context);
  context.testHooks.installOverlays();
  const button = appended.find(node => node.className === 'apocalipse-media-download');
  assert.ok(button, 'the actual overlay must be installed');
  return {
    sent, fetched, location, video,
    click: () => button.click({ preventDefault() {}, stopPropagation() {} }),
    scan: () => context.testHooks.collect(),
    downloads: () => sent.filter(message => message.type === 'APOCALIPSE_DOWNLOAD'),
  };
}
const tracks = [
  { url: 'https://video.fbcdn.net/audio-disguised.mp4?bytestart=0&byteend=999', contentType: 'video/mp4', capturedAt: 1000 },
  { url: 'https://audio.fbcdn.net/another-reel.mp4', contentType: 'audio/mp4', capturedAt: 1001 },
];

test('Reel overlay uses the same page URL as popup scan, not a misleading CDN track', async () => {
  const p = page({ network: tracks });
  const popup = p.scan().find(item => item.url === p.location.href);
  assert.ok(popup.pageExtractor);
  assert.equal(popup.previewUrl, p.video.currentSrc);
  assert.ok(!p.scan().some(item => item.url === p.video.currentSrc));
  await p.click();
  assert.equal(p.downloads().length, 1);
  const overlay = p.downloads()[0].item;
  assert.equal(overlay.url, popup.url);
  assert.equal(overlay.title, popup.title);
  assert.equal(overlay.kind, 'video');
  assert.equal(overlay.audioUrl, null);
  assert.equal(overlay.requestUrls.length, 0);
  assert.ok(!p.sent.some(message => message.type === 'APOCALIPSE_RECENT_TAB_MEDIA'));
});

test('even a readable Facebook blob cannot bypass the complete-page extractor', async () => {
  const p = page({ source: 'blob:https://www.facebook.com/synthetic', readableBlob: true, network: tracks });
  await p.click();
  assert.equal(p.downloads().length, 1);
  assert.equal(p.downloads()[0].item.url, p.location.href);
  assert.equal(p.fetched.length, 0);
  assert.ok(!p.sent.some(message => message.type.startsWith('APOCALIPSE_BLOB_')));
});

test('feed-card permalink wins over unrelated recent network traffic', async () => {
  const permalink = 'https://www.facebook.com/reel/222333444/';
  const p = page({ url: 'https://www.facebook.com/', permalink, network: tracks });
  await p.click();
  assert.equal(p.downloads()[0].item.url, permalink);
  assert.equal(p.downloads()[0].item.audioUrl, null);
});

test('a reused Reel player resolves the current permalink on every click', async () => {
  const p = page({ network: tracks });
  await p.click();
  p.location.href = 'https://www.facebook.com/reel/999888777';
  await p.click();
  assert.deepEqual(p.downloads().map(message => message.item.url), [
    'https://www.facebook.com/reel/123456789', 'https://www.facebook.com/reel/999888777',
  ]);
});

test('watch and shared-video page URLs keep the extractor route', async () => {
  for (const url of ['https://www.facebook.com/watch/?v=123456789', 'https://www.facebook.com/share/r/SyntheticId/']) {
    const p = page({ url, network: tracks });
    await p.click();
    assert.equal(p.downloads()[0].item.url, url);
    assert.equal(p.downloads()[0].item.audioUrl, null);
  }
});

test('TikTok permalink wins over a potentially incomplete captured media track', async () => {
  const url = 'https://v16.tiktok.com/video/tos/synthetic/?mime_type=video_mp4';
  const p = page({ url: 'https://www.tiktok.com/@synthetic/video/123456789', source: url, network: [{ url, contentType: 'video/mp4' }] });
  await p.click();
  assert.equal(p.downloads()[0].item.url, 'https://www.tiktok.com/@synthetic/video/123456789');
  assert.equal(p.downloads()[0].item.audioUrl, null);
});

test('TikTok generic feed pairs a captured video track with its audio track', async () => {
  const video = 'https://v16.tiktok.com/video/tos/synthetic/?mime_type=video_mp4';
  const audio = 'https://v16.tiktok.com/audio/tos/synthetic/';
  const unrelatedVideo = 'https://v16.tiktok.com/video/tos/another/?mime_type=video_mp4';
  const unrelatedAudio = 'https://v16.tiktok.com/audio/tos/another/';
  const p = page({ url: 'https://www.tiktok.com/', source: video, network: [
    { url: unrelatedVideo, contentType: 'video/mp4', capturedAt: 5000 },
    { url: unrelatedAudio, contentType: 'audio/mp4', capturedAt: 5001 },
    { url: video, contentType: 'video/mp4', capturedAt: 1000 },
    { url: audio, contentType: 'audio/mp4', capturedAt: 1001 },
  ] });
  await p.click();
  assert.equal(p.downloads()[0].item.url, video);
  assert.equal(p.downloads()[0].item.audioUrl, audio);
});

test('a scrolled TikTok feed rebinds the same overlay to the new player source', async () => {
  const firstVideo = 'https://v16.tiktok.com/video/tos/first/?mime_type=video_mp4';
  const firstAudio = 'https://v16.tiktok.com/audio/tos/first/';
  const secondVideo = 'https://v16.tiktok.com/video/tos/second/?mime_type=video_mp4';
  const secondAudio = 'https://v16.tiktok.com/audio/tos/second/';
  const network = [
    { url: firstVideo, contentType: 'video/mp4', capturedAt: 1000 },
    { url: firstAudio, contentType: 'audio/mp4', capturedAt: 1001 },
  ];
  const p = page({ url: 'https://www.tiktok.com/', source: firstVideo, network });
  await p.click();
  p.video.currentSrc = secondVideo;
  p.video.src = secondVideo;
  network.unshift(
    { url: secondVideo, contentType: 'video/mp4', capturedAt: 5000 },
    { url: secondAudio, contentType: 'audio/mp4', capturedAt: 5001 },
  );
  await p.click();
  assert.deepEqual(p.downloads().map(message => [message.item.url, message.item.audioUrl]), [
    [firstVideo, firstAudio], [secondVideo, secondAudio],
  ]);
});

test('YouTube page extraction and ordinary direct HTTP downloads are unchanged', async () => {
  for (const [url, source, expected] of [
    ['https://www.youtube.com/watch?v=synthetic', 'https://cdn.example/track.mp4', 'https://www.youtube.com/watch?v=synthetic'],
    ['https://site.example/page', 'https://cdn.example/complete.mp4', 'https://cdn.example/complete.mp4'],
  ]) {
    const p = page({ url, source });
    await p.click();
    assert.equal(p.downloads()[0].item.url, expected);
  }
});
