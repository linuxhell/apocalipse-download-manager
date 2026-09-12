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
  assert.match(source, /identity\.tiktok_share_probe/);
  assert.match(source, /clipboardCandidatePresent/);
  assert.match(source, /key !== 'stateNode'/);
  assert.match(source, /dialog\.querySelectorAll\('\*'\)/);
  assert.match(source, /C\.ev\(node\).*C\.ev\(C\.clickTarget\(node\)\)/);
  assert.match(source, /tiktok_copy_handler_identity/);
  assert.match(source, /data-apocalipse-tiktok-copy-probe/);
  assert.match(source, /data-apocalipse-tiktok-copy-result/);
  assert.match(source, /tiktok_share_dialog_dom_identity/);
  assert.match(source, /start\?\.value/);
  assert.match(source, /data-clipboard-text/);
  assert.match(source, /input,textarea/);
  assert.match(source, /markup\.length <= 250000/);
  assert.match(source, /apocalipse-tiktok-share-identity-request/);
  assert.match(source, /tiktok_share_main_world_control_identity/);
  assert.match(source, /data-apocalipse-tiktok-share-main-result/);
});

test('TikTok share V4 always dismisses the dialog it opened', () => {
  assert.match(source, /const dismiss = fresh =>/);
  assert.match(source, /setTimeout\(\(\) => dismiss\(fresh\), 100\)/);
  assert.match(source, /key: 'Escape'/);
  assert.match(source, /topRight\.click/);
});
