const assert = require('node:assert/strict');
const { test } = require('node:test');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');

const content = readFileSync(join(__dirname, '../browser-extension/content.js'), 'utf8');
const popup = readFileSync(join(__dirname, '../browser-extension/popup.js'), 'utf8');
const resolver = readFileSync(join(__dirname, '../browser-extension/popup-social-player-resolution.js'), 'utf8');
const preview = readFileSync(join(__dirname, '../apps/desktop/src-tauri/src/prepared_preview.rs'), 'utf8');

test('YouTube watch, live and Shorts use one canonical extractor row', () => {
  assert.match(content, /\(\?:shorts\|live\)/);
  assert.match(content, /const youtubeUrl = youtubeExtractorUrl\(\);/);
  assert.match(content, /if \(youtubeUrl\) return;/);
  assert.match(content, /pageExtractor: true,[\s\S]*recommended: true/);
});

test('bulk selection never selects disabled visual-only identities', () => {
  assert.match(popup, /value\.kind === selected && !value\.visualOnly/);
});

test('Facebook and TikTok resolver cannot intercept YouTube rows', () => {
  assert.match(resolver, /\(\?:facebook\|tiktok\)\\\.com/);
  assert.match(resolver, /socialResolverPage\(\) && item\?\.kind === 'video'/);
});

test('page preview is bounded so an ongoing live can open', () => {
  assert.match(preview, /"--download-sections",\s*"\*0-30"/);
});

test('YouTube prefers the current video id thumbnail over stale page metadata', () => {
  assert.match(content, /const thumbnail = videoId[\s\S]*i\.ytimg\.com\/vi\/\$\{videoId\}\/hqdefault\.jpg/);
  assert.doesNotMatch(content, /add\(youtubeUrl, "video", video, document\.querySelector\('meta\[property="og:image"\]'\)/);
});
