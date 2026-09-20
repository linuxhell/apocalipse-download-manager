const test = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');

const root = join(__dirname, '..');
const source = readFileSync(join(root, 'browser-extension/diagnostics-media-replay.js'), 'utf8');
const manifest = JSON.parse(readFileSync(join(root, 'browser-extension/manifest.json'), 'utf8'));
const native = readFileSync(join(root, 'apps/desktop/src-tauri/src/diagnostics_v3.rs'), 'utf8');
const popup = readFileSync(join(root, 'browser-extension/popup.js'), 'utf8');
const desktop = readFileSync(join(root, 'apps/desktop/src-tauri/src/main.rs'), 'utf8');

test('0.3.164 loads the opt-in replay before media handlers in every isolated content chain', () => {
  assert.equal(manifest.version, '0.3.164');
  for (const entry of manifest.content_scripts.filter(item => item.js.includes('content.js'))) {
    assert.ok(entry.js.includes('diagnostics-media-replay.js'));
    assert.ok(entry.js.indexOf('diagnostics.js') < entry.js.indexOf('diagnostics-media-replay.js'));
    assert.ok(entry.js.indexOf('diagnostics-media-replay.js') < entry.js.indexOf('content.js'));
    assert.notEqual(entry.world, 'MAIN');
  }
});

test('replay observes left and right media interaction without page text or typed values', () => {
  assert.match(source, /\['click', 'dblclick', 'auxclick'\]/);
  assert.match(source, /event\.button === 2 \? 'right'/);
  assert.match(source, /document\.addEventListener\('contextmenu'/);
  assert.match(source, /editable\(event\.target\)/);
  assert.doesNotMatch(source, /innerText|\.value\b|keydown|keypress|keyup|\btext\s*:/);
});

test('clipboard probe is short-lived and emits only canonical social media links', () => {
  assert.match(source, /Date\.now\(\) \+ 8000/);
  assert.match(source, /if \(!identity\) return; \/\/ Arbitrary clipboard content is never emitted/);
  assert.match(source, /tiktok_video/);
  for (const kind of ['facebook_watch_id', 'facebook_share_video', 'facebook_profile_video', 'facebook_reel']) assert.match(source, new RegExp(kind));
  assert.match(source, /clearInterval\(pending\.timer\)/);
  assert.match(source, /canonicalUrl: identity\.url/);
  assert.doesNotMatch(source, /url: identity\.url[\s\S]{0,120}\.\.\.state\.player/);
  assert.match(source, /ApocalipseTikTokIdentity\?\.learn/);
});

test('Facebook replay distinguishes link routes and exact sponsored-card evidence', () => {
  assert.match(source, /mediaCardClass/);
  assert.match(source, /facebook_sponsored/);
  assert.match(source, /facebook_ordinary/);
  assert.match(source, /videos\.some\(item => item !== video\)/);
});

test('desktop ZIP exports the dedicated chronological media replay file', () => {
  assert.match(native, /traces\/replay-de-midia\.jsonl/);
  assert.match(native, /name\.starts_with\("media_replay\."\)/);
  assert.match(native, /name\.starts_with\("handoff\."\)/);
  assert.match(desktop, /Apocalipse Forensic Debugger V4/);
  assert.match(desktop, /timeline\/events-local\.jsonl/);
  assert.match(desktop, /correlation\/index\.json/);
  assert.match(desktop, /incidents\/problem-windows\.jsonl/);
});

test('an already-open tab receives the replay collector during extension recovery', () => {
  assert.match(popup, /"diagnostics\.js", "diagnostics-media-replay\.js", "tiktok-identity\.js"/);
});


test('desktop ZIP exports dedicated universal social debugger files', () => {
  assert.match(native, /social\/player-debugger\.jsonl/);
  assert.match(native, /social\/summary\.json/);
  assert.match(native, /social\.overlay_missing/);
  assert.match(native, /structured_observations_not_guesses/);
});


test('Debugger V4 exports sanitized engine logs and subsystem index', () => {
  assert.match(desktop, /engines\/aria2-runtime\.log/);
  assert.match(desktop, /join\("logs"\)\.join\("engines"\)/);
  assert.match(desktop, /debugger-index\.json/);
  assert.match(desktop, /warnings-errors\.jsonl/);
  assert.match(desktop, /"formatVersion": 4/);
  assert.match(desktop, /redacted-sensitive-line/);
});

test('forensic timeline preserves local time and authoritative server sequence', () => {
  assert.match(native, /receivedAtLocal/);
  assert.match(native, /clientTimestampLocal/);
  assert.match(native, /timeline\/events-local\.jsonl/);
  assert.match(native, /serverSequence/);
  assert.match(native, /receivedAt_then_serverSequence/);
});

test('marked incidents preserve before-and-after windows instead of stopping capture', () => {
  assert.match(native, /session\.problem_marked/);
  assert.match(native, /saturating_sub\(90_000\)/);
  assert.match(native, /saturating_add\(45_000\)/);
  assert.match(native, /events_near_user_mark_not_automatic_root_cause/);
});
