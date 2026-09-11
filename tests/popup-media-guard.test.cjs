const assert = require('node:assert/strict');
const test = require('node:test');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

const source = fs.readFileSync(path.join(__dirname, '../browser-extension/popup-media-guard.js'), 'utf8');
const context = vm.createContext({
  URL, Date, globalThis: null,
  chrome: {
    tabs: { sendMessage() {}, get() { return Promise.resolve({ url: 'https://www.tiktok.com/' }); } },
    runtime: { sendMessage() {} },
  },
});
context.globalThis = context;
vm.runInContext(source, context);
const { cleanScanned, filterNetwork, socialIdentity } = context.ADM_POPUP_MEDIA_GUARD;

test('0.3.127 deduplicates canonical social media identities', () => {
  const rows = cleanScanned([
    { url: 'https://www.tiktok.com/@u/video/123?x=1', kind: 'video', pageExtractor: true, thumbnail: '' },
    { url: 'https://www.tiktok.com/@u/video/123', kind: 'video', pageExtractor: true, thumbnail: 'x' },
  ], 'https://www.tiktok.com/');
  assert.equal(rows.length, 1);
  assert.equal(rows[0].thumbnail, 'x');
  assert.equal(socialIdentity(rows[0].url), 'tiktok:123');
});

test('0.3.127 hides unresolved alien rows', () => {
  assert.equal(cleanScanned([
    { url: 'https://www.instagram.com/#apocalipse-player-1', kind: 'video', visualOnly: true },
  ], 'https://www.instagram.com/').length, 0);
});

test('0.3.127 preserves real audio and images in their own categories', () => {
  const rows = cleanScanned([
    { url: 'https://a/x.mp3', kind: 'audio' },
    { url: 'https://a/x.jpg', kind: 'image' },
  ], 'https://www.facebook.com/');
  assert.deepEqual(Array.from(rows, item => item.kind), ['audio', 'image']);
});

test('0.3.127 hides adaptive CDN components but keeps observed standalone audio', () => {
  const scanned = [
    { url: 'https://www.instagram.com/reel/ABC/', kind: 'video', pageExtractor: true, thumbnail: 'x' },
    { url: 'https://a/song.mp3', kind: 'audio' },
  ];
  const state = { pageUrl: 'https://www.instagram.com/', scanned };
  const network = [
    { url: 'https://cdn/x.mp4', contentType: 'video/mp4' },
    { url: 'https://cdn/x.m4a', contentType: 'audio/mp4' },
    { url: 'https://a/song.mp3', contentType: 'audio/mpeg' },
  ];
  const output = filterNetwork(network, state);
  assert.deepEqual(Array.from(output, item => item.url), ['https://a/song.mp3']);
});
