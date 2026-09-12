const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');

const content = readFileSync(join(__dirname, '../browser-extension/content.js'), 'utf8');

test('continuous player mutations cannot postpone overlay installation forever', () => {
  assert.match(content, /if \(overlayTimer !== null && overlayTimer !== undefined\) return;/);
  assert.match(content, /overlayTimer = setTimeout\(\(\) => \{\s*overlayTimer = null;\s*installOverlays\(\);/);
  assert.doesNotMatch(content, /const scheduleOverlays = \(\) => \{\s*clearTimeout\(overlayTimer\)/);
});
