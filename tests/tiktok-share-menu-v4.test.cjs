const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');

const source = readFileSync(join(__dirname, '../browser-extension/tiktok-share-menu-v4.js'), 'utf8');

test('TikTok share V4 reads exact item identity from fresh framework data', () => {
  assert.match(source, /tiktok_share_dialog_framework_identity/);
  assert.match(source, /value\.video \|\| value\.videoInfo/);
  assert.match(source, /value\.author\?\.uniqueId/);
  assert.match(source, /frameworkValuesInspected/);
});

test('TikTok share V4 always dismisses the dialog it opened', () => {
  assert.match(source, /const dismiss = \(\) =>/);
  assert.match(source, /setTimeout\(dismiss, 600\)/);
  assert.match(source, /key: 'Escape'/);
});
