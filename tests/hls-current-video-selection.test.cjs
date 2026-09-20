const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');

const content = readFileSync(join(__dirname, '../browser-extension/content.js'), 'utf8');
const background = readFileSync(join(__dirname, '../browser-extension/background.js'), 'utf8');
const desktopUi = readFileSync(join(__dirname, '../apps/desktop/ui/app.js'), 'utf8');

test('generic HLS overlays send the current player duration using the worker contract', () => {
  const messages = [...content.matchAll(/type:\s*"APOCALIPSE_(?:SELECT|ANALYZE)_HLS"[\s\S]*?\}\);/g)].map((match) => match[0]);
  assert.equal(messages.length, 3);
  for (const message of messages) {
    assert.match(message, /duration:\s*Number\.isFinite\(/);
    assert.doesNotMatch(message, /expectedDuration:/);
  }
  assert.match(background, /message\?\.type === "APOCALIPSE_SELECT_HLS"/);
  assert.match(background, /reply\(items\.find\(\(item\) => item\.recommended\) \|\| null\)/);
  assert.match(background, /message\?\.type === "APOCALIPSE_ANALYZE_HLS" \|\| message\?\.type === "APOCALIPSE_HLS_ANALYZE"/);
  assert.match(background, /message\.duration \?\? message\.expectedDuration/);
});

test('HLS analysis ranks playlists by distance from the visible player duration', () => {
  assert.match(background, /Math\.abs\(a\.duration - expected\) - Math\.abs\(b\.duration - expected\)/);
});


test('HLS analysis detects audio-only segment playlists and propagates the media kind', () => {
  assert.match(background, /response\.headers\.get\("content-type"\)/);
  assert.match(background, /mediaKind = inspected\.kind === "audio" \? "audio"/);
  assert.match(background, /mediaKind: result\.mediaKind \|\| null/);
  assert.match(content, /if \(hls\.length > 0\)/);
  assert.match(content, /kind: detail\.mediaKind \|\| item\.kind/);
  assert.match(content, /kind: resolved\?\.mediaKind \|\| element\.tagName\.toLowerCase\(\)/);
});


test('desktop exposes FFmpeg conversion for audio HLS handed off by the extension', () => {
  assert.match(desktopUi, /plan\.reason === "hls_manifest"/);
  assert.doesNotMatch(desktopUi, /plan\.primary === "NM3u8DlRe"/);
  assert.match(desktopUi, /const audioHls = pendingMediaKind === "audio"/);
  assert.match(desktopUi, /#hls-audio-conversion/);
  assert.match(desktopUi, /audio:\$\{format\}/);
});
