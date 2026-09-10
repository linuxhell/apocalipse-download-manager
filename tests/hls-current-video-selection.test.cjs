const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');

const content = readFileSync(join(__dirname, '../browser-extension/content.js'), 'utf8');
const background = readFileSync(join(__dirname, '../browser-extension/background.js'), 'utf8');

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
