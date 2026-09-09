const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const root = path.join(__dirname, '..', 'browser-extension');

test('TikTok click fixer runs in every frame', () => {
  const manifest = JSON.parse(fs.readFileSync(path.join(root, 'manifest.json'), 'utf8'));
  const entry = manifest.content_scripts.find((item) => Array.isArray(item.js) && item.js.includes('tiktok-media-fix.js'));
  assert.ok(entry, 'TikTok fixer must be declared');
  assert.equal(entry.all_frames, true);
});

test('TikTok fixer claims overlay click before legacy button handler', () => {
  const source = fs.readFileSync(path.join(root, 'tiktok-media-fix.js'), 'utf8');
  assert.match(source, /document\.addEventListener\("click",\s*async \(event\) => \{/);
  assert.match(source, /event\.stopImmediatePropagation\(\)/);
  assert.match(source, /tiktok_overlay_handler_claimed/);
});

test('TikTok identity can climb from a player frame into its parent card', () => {
  const source = fs.readFileSync(path.join(root, 'tiktok-identity.js'), 'utf8');
  assert.match(source, /const frameAncestorUrl = \(\) => \{/);
  assert.match(source, /currentWindow\.frameElement/);
  assert.match(source, /if \(framed\) return framed/);
});
