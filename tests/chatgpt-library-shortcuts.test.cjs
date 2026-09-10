const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');

const content = readFileSync(join(__dirname, '../browser-extension/content.js'), 'utf8');

test('ChatGPT Library normal and Force clicks use the real ADM download channel', () => {
  const start = content.indexOf('const interceptChatgptLibrary');
  const end = content.indexOf('document.addEventListener("pointerdown", interceptChatgptLibrary', start);
  const handler = content.slice(start, end);
  assert.ok(start >= 0 && end > start);
  assert.match(handler, /type:\s*"APOCALIPSE_DOWNLOAD"/);
  assert.doesNotMatch(handler, /APOCALIPSE_CHATGPT_LIBRARY_DIRECT/);
  assert.match(handler, /requestUrls:\s*\[found\.url\]/);
});

test('ChatGPT Library Bypass returns before preventing the native browser click', () => {
  const start = content.indexOf('const interceptChatgptLibrary');
  const end = content.indexOf('document.addEventListener("pointerdown", interceptChatgptLibrary', start);
  const handler = content.slice(start, end);
  const bypass = handler.indexOf('modifierPressed(event, shortcutKeys.bypass)');
  const prevented = handler.indexOf('event.preventDefault()');
  assert.ok(bypass >= 0 && prevented > bypass);
  assert.doesNotMatch(content, /APOCALIPSE_CHATGPT_LIBRARY_ARM_DENY/);
});
