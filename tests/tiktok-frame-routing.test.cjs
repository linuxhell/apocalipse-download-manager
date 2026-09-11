const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const root = path.join(__dirname, '..', 'browser-extension');

test('TikTok blob button falls back to its exact bound player stream', () => {
  const source = fs.readFileSync(path.join(root, 'tiktok-media-fix.js'), 'utf8');
  assert.match(source, /Symbol\.for\("apocalipse\.recordButton"\)/);
  assert.match(source, /tiktok_blob_without_complete_resource/);
  assert.match(source, /recordButton\.click\(\)/);
});

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
  const main = manifest.content_scripts.find((item) => item.world === 'MAIN'
    && item.js.includes('tiktok-page-identity.js'));
  assert.deepEqual(main.js, ['tiktok-main-identity.js', 'tiktok-page-identity.js']);
  assert.ok(!main.js.some((file) => tiktok.js.includes(file)),
    'Chrome must receive different file paths for MAIN and ISOLATED worlds');
  assert.equal(
    fs.readFileSync(path.join(root, 'tiktok-main-identity.js'), 'utf8'),
    fs.readFileSync(path.join(root, 'tiktok-identity.js'), 'utf8'),
    'both world-specific identity resolvers must stay byte-identical',
  );
});

test('TikTok fixer claims overlay click before legacy button handler', () => {
  const source = fs.readFileSync(path.join(root, 'tiktok-media-fix.js'), 'utf8');
  assert.match(source, /document\.addEventListener\("click",\s*async \(event\) => \{/);
  assert.match(source, /event\.stopImmediatePropagation\(\)/);
  assert.match(source, /tiktok_overlay_handler_claimed/);
});

test('generic TikTok handler never opens fullscreen to discover reel identity', () => {
  const source = fs.readFileSync(path.join(root, 'content.js'), 'utf8');
  assert.doesNotMatch(source, /legacy_identity_fallback/);
  assert.doesNotMatch(source, /requestFullscreen\(\)/);
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
  assert.match(source, /pageExtractor:\s*true/);
  assert.match(source, /extractorUrl:\s*best\.url/);
  assert.match(source, /ApocalipseTikTokIdentity\?\.resolve\?\.\(video\)/);
});

test('popup keeps a fixed browser viewport and scrolls only the media list', () => {
  const css = fs.readFileSync(path.join(root, 'popup.css'), 'utf8');
  assert.match(css, /html, body[^}]*height:\s*580px[^}]*overflow:\s*hidden/);
  assert.match(css, /body\s*\{[^}]*display:\s*flex[^}]*flex-direction:\s*column/);
  assert.match(css, /main\s*\{[^}]*flex:\s*1 1 auto[^}]*overflow-y:\s*scroll/);
  assert.doesNotMatch(css, /main\s*\{[^}]*height:\s*460px/);
});

test('compact popup gives media the main area and moves diagnostics into localized Logs', () => {
  const html = fs.readFileSync(path.join(root, 'popup.html'), 'utf8');
  const css = fs.readFileSync(path.join(root, 'popup.css'), 'utf8');
  const popup = fs.readFileSync(path.join(root, 'popup.js'), 'utf8');
  const diagnostics = fs.readFileSync(path.join(root, 'diagnostics-popup.js'), 'utf8');
  assert.match(html, /data-kind="logs"\s+data-i18n="logs"/);
  assert.match(html, /id="logs-panel"[^>]*hidden/);
  assert.match(html, /id="compact-settings"[^>]*hidden/);
  assert.match(diagnostics, /querySelector\('#logs-panel \.diag-controls'\)/);
  assert.doesNotMatch(diagnostics, /document\.createElement\('section'\)/);
  assert.match(popup, /pt_BR: \{ extension: "Extensão"/);
  assert.match(popup, /externalPreview: "Visualizar"/);
  assert.match(popup, /zh_CN: \{ extension: "扩展"/);
  assert.match(css, /article img, \.audio-icon \{ width: 104px; height: 62px; object-fit: cover/);
});
