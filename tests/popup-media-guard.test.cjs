const assert = require('node:assert/strict');
const test = require('node:test');
const fs = require('node:fs');
const vm = require('node:vm');
const source = fs.readFileSync(require('node:path').join(__dirname, '../browser-extension/popup-media-guard.js'), 'utf8');

function makeContext({ pageUrl = 'https://www.tiktok.com/', scanMedia = [], recentMedia = [] } = {}) {
  const calls = [];
  const chrome = {
    tabs: {
      get() { return Promise.resolve({ url: pageUrl }); },
      sendMessage(tabId, message, options, callback) {
        if (typeof options === 'function') callback = options;
        calls.push(['tabs', message.type]);
        const response = message.type === 'APOCALIPSE_SCAN' ? { media: scanMedia } : { ok: true };
        if (callback) { callback(response); return undefined; }
        return Promise.resolve(response);
      },
    },
    runtime: {
      sendMessage(message, callback) {
        calls.push(['runtime', message.type]);
        const response = message.type === 'APOCALIPSE_RECENT_TAB_MEDIA' ? { media: recentMedia } : { ok: true };
        if (callback) { callback(response); return undefined; }
        return Promise.resolve(response);
      },
    },
  };
  const context = vm.createContext({ URL, Date, chrome, console, globalThis: null });
  context.globalThis = context;
  vm.runInContext(source, context);
  return { context, calls };
}

const reel = { url: 'https://www.tiktok.com/@u/video/123', kind: 'video', pageExtractor: true, thumbnail: 'thumb' };
const cdnVideo = { url: 'https://v16-webapp.tiktok.com/a.mp4?x=1', contentType: 'video/mp4' };
const cdnAudio = { url: 'https://v16-webapp.tiktok.com/a.m4a?x=1', contentType: 'audio/mp4' };

test('deduplicates canonical reels by logical identity', () => {
  const { context } = makeContext();
  const rows = context.ADM_POPUP_MEDIA_GUARD.cleanScanned([
    { url: 'https://www.tiktok.com/@u/video/123?x=1', kind: 'video', pageExtractor: true, thumbnail: '' },
    reel,
  ], 'https://www.tiktok.com/');
  assert.equal(rows.length, 1);
  assert.equal(rows[0].thumbnail, 'thumb');
  assert.equal(context.ADM_POPUP_MEDIA_GUARD.socialIdentity(rows[0].url), 'tiktok:123');
});

test('preserves real audio and images while filtering anonymous adaptive tracks', () => {
  const { context } = makeContext();
  const scan = [reel, { url: 'https://a/song.mp3', kind: 'audio' }, { url: 'https://a/photo.jpg', kind: 'image' }];
  const out = context.ADM_POPUP_MEDIA_GUARD.filterNetwork([
    cdnVideo, cdnAudio, { url: 'https://a/song.mp3', contentType: 'audio/mpeg' },
  ], { pageUrl: 'https://www.tiktok.com/', scanned: scan });
  assert.deepEqual(Array.from(out, x => x.url), ['https://a/song.mp3']);
  assert.deepEqual(Array.from(context.ADM_POPUP_MEDIA_GUARD.cleanScanned(scan, 'https://www.tiktok.com/'), x => x.kind), ['video', 'audio', 'image']);
});

test('Promise runtime.sendMessage path filters TikTok CDN tracks', async () => {
  const { context } = makeContext({ scanMedia: [reel], recentMedia: [cdnVideo, cdnAudio] });
  const scan = await context.chrome.tabs.sendMessage(7, { type: 'APOCALIPSE_SCAN' }, { frameId: 0 });
  assert.equal(scan.media.length, 1);
  const recent = await context.chrome.runtime.sendMessage({ type: 'APOCALIPSE_RECENT_TAB_MEDIA', tabId: 7 });
  assert.equal(recent.media.length, 0);
});

test('callback runtime.sendMessage path still filters Facebook CDN tracks', async () => {
  const fbReel = { url: 'https://www.facebook.com/reel/456', kind: 'video', pageExtractor: true, thumbnail: 'thumb' };
  const fbCdn = { url: 'https://scontent.fcpq4-1.fna.fbcdn.net/video.mp4?x=1', contentType: 'video/mp4' };
  const { context } = makeContext({ pageUrl: 'https://www.facebook.com/', scanMedia: [fbReel], recentMedia: [fbCdn] });
  await new Promise(resolve => context.chrome.tabs.sendMessage(8, { type: 'APOCALIPSE_SCAN' }, { frameId: 0 }, () => resolve()));
  const recent = await new Promise(resolve => context.chrome.runtime.sendMessage({ type: 'APOCALIPSE_RECENT_TAB_MEDIA', tabId: 8 }, resolve));
  assert.equal(recent.media.length, 0);
});

test('Promise scan path removes unresolved visual-only alien rows', async () => {
  const unresolved = { url: 'https://www.tiktok.com/#apocalipse-player-1', kind: 'video', visualOnly: true, thumbnail: '' };
  const { context } = makeContext({ scanMedia: [unresolved, reel] });
  const scan = await context.chrome.tabs.sendMessage(9, { type: 'APOCALIPSE_SCAN' });
  assert.equal(scan.media.length, 1);
  assert.equal(scan.media[0].url, reel.url);
});

test('Facebook recording-only sponsored players never become popup rows', () => {
  const { context } = makeContext({ pageUrl: 'https://www.facebook.com/' });
  const rows = context.ADM_POPUP_MEDIA_GUARD.cleanScanned([
    { url: 'https://www.facebook.com/#apocalipse-sponsored', kind: 'video', visualOnly: true,
      recordingOnly: true, playerBound: true, recommended: true, thumbnail: 'frame' },
    { url: 'https://www.facebook.com/reel/456', kind: 'video', pageExtractor: true, thumbnail: 'cover' },
  ], 'https://www.facebook.com/');
  assert.deepEqual(Array.from(rows, item => item.url), ['https://www.facebook.com/reel/456']);
});
