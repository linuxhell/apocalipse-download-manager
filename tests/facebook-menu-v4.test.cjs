const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const source = readFileSync(join(__dirname, '../browser-extension/facebook-menu-v4.js'), 'utf8');

test('Facebook friend posts retry Copy Link through the exact Share control', () => {
  assert.match(source, /facebook_copy_link_item_not_found/);
  assert.match(source, /shareRe/);
  assert.match(source, /facebook_share_copy_link_clicked/);
  assert.match(source, /candidates\.length !== 1/);
});

test('Facebook Share resolver only returns identity or clipboard discovery, never Download', () => {
  assert.doesNotMatch(source, /APOCALIPSE_DOWNLOAD|actionIntent:\s*['"]download/);
  assert.match(source, /clipboardRequested:\s*true/);
});
