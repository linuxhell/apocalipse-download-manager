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


test('Facebook sponsored filtering requires a visible explicit marker and avoids generic ads links', () => {
  assert.match(content, /const visibleMarker = marker =>/);
  assert.match(content, /style\.display !== "none"/);
  assert.match(content, /style\.visibility !== "hidden"/);
  assert.doesNotMatch(content, /a\[href\*="\/ads\/"\]/);
  assert.doesNotMatch(content, /a\[href\*="ads\/about"\]/);
});

test('Facebook direct HTTP players keep recording available', () => {
  assert.match(content, /\(!hasDirectHttpMedia \|\| isFacebookVideo\)/);
});

test('Facebook unresolved Download falls back to recording even without srcObject', () => {
  assert.match(content, /facebook_unresolved_recording_fallback/);
  assert.match(content, /if \(isFacebookVideo && canRecord && recordButton\)/);
  assert.doesNotMatch(content, /if \(isFacebookVideo && element\.srcObject && canRecord && recordButton\)/);
});


test('universal social debugger records player decisions, missing overlays and scan summaries', () => {
  assert.match(content, /social\.player_decision/);
  assert.match(content, /social\.overlay_missing/);
  assert.match(content, /social\.scan_summary/);
  assert.match(content, /visible_player_without_overlay_after_reconcile/);
  assert.match(content, /no_supported_action/);
  assert.match(content, /platform: socialPlatform\(\)/);
});

test('social debugger supports Facebook, Instagram, TikTok and future generic handling', () => {
  assert.match(content, /return "facebook"/);
  assert.match(content, /return "instagram"/);
  assert.match(content, /return "tiktok"/);
  assert.match(content, /return "x"/);
  assert.match(content, /return "generic"/);
});
