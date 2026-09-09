const assert = require('node:assert/strict');
const { test } = require('node:test');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const vm = require('node:vm');
const source = readFileSync(join(__dirname, '../browser-extension/popup.js'), 'utf8');
const start = source.indexOf('function previewRequestFor(');
const end = source.indexOf('\nconst render =', start);
const context = vm.createContext({ URL });
vm.runInContext(source.slice(start, end), context);
const preview = context.previewRequestFor;
test('player uses the same explicit row identity as Download, never previewUrl feed hint', () => {
  for (const previewUrl of ['https://www.facebook.com/', 'blob:partial', 'https://cdn.example/wrong.mp4']) {
    const item = { url: 'https://www.facebook.com/reel/123', previewUrl, pageExtractor: true, kind: 'video', contentType: 'video/mp4' };
    const result = preview(item, 'https://www.facebook.com/');
    assert.equal(result.url, item.url); assert.equal(result.pageExtractor, true); assert.equal(result.contentType, null);
  }
});
test('paired direct preview preserves video AND audio identity', () => {
  const item = { url: 'https://v16.tiktok.com/video/a.mp4?s=1', audioUrl: 'https://v19.tiktokcdn.com/audio/a.m4a?s=2', kind: 'video', contentType: 'video/mp4' };
  const result = preview(item, 'https://www.tiktok.com/');
  assert.equal(result.url, item.url); assert.equal(result.audioUrl, item.audioUrl); assert.equal(result.mediaKind, 'video');
});
test('isolated incomplete videos stay visible but disabled; explicit audio remains playable', () => {
  const item = { url: 'https://v16.tiktok.com/track.mp4', ambiguousSocialTrack: true, kind: 'video' };
  assert.equal(preview(item, 'https://www.tiktok.com/'), null);
  assert.ok(preview({ ...item, kind: 'audio' }, 'https://www.tiktok.com/'));
  assert.ok(source.includes('previewButton.hidden = item.kind === "image"'));
  assert.ok(source.includes('previewButton.disabled = !previewRequest'));
});
test('invalid preview sources cannot be passed to the desktop player', () => {
  for (const url of ['file:///secret', 'https://user:pass@host.test/a.mp4', 'blob:x', '?onlyquery=1']) {
    assert.equal(preview({ url, kind: 'video' }, 'https://www.tiktok.com/'), null);
  }
});
