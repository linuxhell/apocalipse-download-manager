const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');

const content = readFileSync(join(__dirname, '../browser-extension/content.js'), 'utf8');

test('Facebook Home revalidates recycled video elements before keeping an overlay', () => {
  assert.match(content, /const currentBindingId = playerIdentity\(overlay\.element\)/);
  assert.match(content, /overlay\.bindingId && overlay\.bindingId !== currentBindingId/);
  assert.match(content, /facebook\.video_reused/);
  assert.match(content, /overlay\.cleanup\(\)/);
  assert.match(content, /bindingId: isFacebookVideo \? playerIdentity\(element\) : null/);
});

test('Facebook overlay keeps sponsored and audio-only content filtered', () => {
  assert.match(content, /facebook\.overlay_skipped_sponsored/);
  assert.match(content, /isSponsoredFacebookPlayer\(element\)/);
  assert.match(content, /facebookPage && element\.tagName === "AUDIO"/);
  assert.match(content, /facebook\.overlay_skipped_audio_only/);
});

test('Facebook virtualized feed revalidates overlays while scrolling and on player lifecycle changes', () => {
  assert.match(content, /addEventListener\("scroll", scheduleOverlays/);
  assert.match(content, /\["loadedmetadata", "loadstart", "load", "emptied"\]/);
  assert.match(content, /document\.addEventListener\(event, scheduleOverlays, true\)/);
});

test('stale dataset marker cannot block overlay reinstall after DOM recycling', () => {
  assert.match(content, /element\.dataset\.apocalipseButton && !activeOverlays\.has\(element\)/);
  assert.match(content, /delete element\.dataset\.apocalipseButton/);
});
