const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const root = path.join(__dirname, '..', 'browser-extension');

test('TikTok uses one deterministic isolated-world chain in every frame', () => {
  const manifest = JSON.parse(fs.readFileSync(path.join(root, 'manifest.json'), 'utf8'));
  const tiktok = manifest.content_scripts.find((item) => Array.isArray(item.js)
    && item.js.includes('tiktok-media-fix.js') && item.js.includes('content.js'));
  assert.ok(tiktok, 'TikTok chain must be declared');
  assert.equal(tiktok.all_frames, true);
  assert.deepEqual(tiktok.js, ['diagnostics-core.js', 'diagnostics.js', 'tiktok-identity.js', 'tiktok-media-fix.js', 'content.js']);
  const generic = manifest.content_scripts.find((item) => Array.isArray(item.js)
    && item.js.includes('content.js') && !item.js.includes('tiktok-media-fix.js'));
  assert.ok(generic?.exclude_matches?.some((value) => value.includes('tiktok.com')),
    'generic content handler must not also be injected into TikTok');
});

test('TikTok fixer claims overlay click before legacy button handler', () => {
  const source = fs.readFileSync(path.join(root, 'tiktok-media-fix.js'), 'utf8');
  assert.match(source, /async function handleDownload\(button, video, event\)/);
  assert.match(source, /ADM_TIKTOK_DOWNLOAD/);
  const content = fs.readFileSync(path.join(root, 'content.js'), 'utf8');
  assert.match(content, /ADM_TIKTOK_DOWNLOAD\.handle\(button, element, event\)/);
  assert.doesNotMatch(content, /requestFullscreen|enteredTikTokFullscreen/);
  assert.match(source, /event\.stopImmediatePropagation\(\)/);
  assert.match(source, /tiktok_overlay_handler_claimed/);
});

test('TikTok identity can climb from a player frame into its parent card', () => {
  const source = fs.readFileSync(path.join(root, 'tiktok-identity.js'), 'utf8');
  assert.match(source, /const frameAncestorUrl = \(\) => \{/);
  assert.match(source, /currentWindow\.frameElement/);
  assert.match(source, /if \(framed\) return explain\(video, "parent_frame_candidate", framed\)/);
});

test('popup discovers the visible TikTok reel across all frames', () => {
  const manifest = JSON.parse(fs.readFileSync(path.join(root, 'manifest.json'), 'utf8'));
  const html = fs.readFileSync(path.join(root, 'popup.html'), 'utf8');
  const source = fs.readFileSync(path.join(root, 'popup-tiktok.js'), 'utf8');
  assert.ok(manifest.permissions.includes('scripting'));
  assert.match(html, /popup-tiktok\.js/);
  assert.match(source, /target:\s*\{\s*tabId,\s*allFrames:\s*true\s*\}/);
  assert.match(source, /ADM_MEDIA_SCAN\.snapshot\(\)/);
  assert.doesNotMatch(source, /visibleScore|anchors\[0\]|best\.url/);
});
