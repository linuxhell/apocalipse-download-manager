const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');

const root = join(__dirname, '..');
const content = readFileSync(join(root, 'browser-extension', 'content.js'), 'utf8');
const appJs = readFileSync(join(root, 'apps', 'desktop', 'ui', 'app.js'), 'utf8');

test('inactive feed previews are reported once per element, not on every scan', () => {
  assert.match(content, /const skippedFeedPreviews = new WeakSet\(\)/);
  assert.match(content, /if \(!skippedFeedPreviews\.has\(element\)\) \{\s*skippedFeedPreviews\.add\(element\);\s*trace\("overlay_skipped_feed_preview"/);
});

test('the saved application language is restored from the backend like the theme', () => {
  assert.match(appJs, /invoke\("get_application_language"\)/);
  assert.match(appJs, /saved === "en" && locale !== "en"/);
  assert.match(appJs, /if \(saved !== locale\) selectLanguage\(saved\)/);
});
