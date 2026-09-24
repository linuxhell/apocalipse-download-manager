const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');

const root = join(__dirname, '..', 'browser-extension');
const background = readFileSync(join(root, 'background.js'), 'utf8');
const contentScript = readFileSync(join(root, 'content.js'), 'utf8');
const popup = readFileSync(join(root, 'popup.js'), 'utf8');
const manifest = JSON.parse(readFileSync(join(root, 'manifest.json'), 'utf8'));

test('0.3.169 rejects generic media labels before deriving a file name', () => {
  assert.equal(manifest.version, '0.3.176');
  assert.match(contentScript, /genericMediaTitle/);
  assert.match(contentScript, /normalize\("NFD"\)/);
  assert.match(contentScript, /pageMediaTitle/);
  assert.match(contentScript, /soundCloudSlugTitle/);
  assert.match(contentScript, /titleSource/);
  assert.match(contentScript, /generic_element/);
});

test('SoundCloud popup rows prefer the tab title or track slug over Captured resource', () => {
  assert.match(popup, /pageTitleHint/);
  assert.match(popup, /soundCloudSlugTitle/);
  assert.match(popup, /repairMediaTitle/);
  assert.match(popup, /browser_title/);
  assert.match(popup, /soundcloud_slug/);
  assert.match(popup, /genericTitle/);
  assert.match(popup, /titleSource/);
});

test('browser-assisted downloads replace generic Video.mp4-style names before handoff', () => {
  assert.match(background, /genericDownloadStem/);
  assert.match(background, /normalizedDownloadTitle/);
  assert.match(background, /resolveBrowserDownloadFileName/);
  assert.match(background, /soundcloud_page/);
  assert.match(background, /browser_download\.filename_resolved/);
  assert.match(background, /browser_download\.filename_fallback/);
  assert.match(background, /suggest\(\{ filename: decision\.fileName, conflictAction: "uniquify" \}\)/);
  assert.match(background, /fileName: effectiveFileName/);
  assert.match(background, /markAssistedDownload\(\{ \.\.\.item, filename: effectiveFileName \|\| item\.filename \}/);
});

test('a consumed disposable download link is never resent, even with force capture active', () => {
  assert.match(background, /const browserAssisted = disposable;/);
  assert.doesNotMatch(background, /const browserAssisted = disposable && !forced;/);
});

test('the network capture filter matches SoundCloud media from soundcloud.com and sndcdn.com', () => {
  assert.match(background, /soundcloud\\\.com\|sndcdn\\\.com/);
});

test('filename diagnostics report the source without logging the page title itself', () => {
  assert.match(background, /filename_source=\$\{fileNameDecision\.source\}/);
  assert.match(background, /source=\$\{fileNameDecision\.source\} ext=/);
  assert.doesNotMatch(background, /browser_download\.filename_resolved[\s\S]{0,400}title=/);
});
