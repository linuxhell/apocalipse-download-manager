const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');

const content = readFileSync(join(__dirname, '../browser-extension/content.js'), 'utf8');
const pageHook = readFileSync(join(__dirname, '../browser-extension/page-hook.js'), 'utf8');
const popup = readFileSync(join(__dirname, '../browser-extension/popup.html'), 'utf8');

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
  const bypass = handler.indexOf('shortcutPressed(event, shortcutKeys.bypass)');
  const prevented = handler.indexOf('event.preventDefault()');
  assert.ok(bypass >= 0 && prevented > bypass);
  assert.doesNotMatch(content, /APOCALIPSE_CHATGPT_LIBRARY_ARM_DENY/);
});


test('Insert is always available as a force shortcut in addition to the configurable shortcut', () => {
  assert.match(content, /heldShortcutKeys/);
  assert.match(content, /\["Alt", "Shift", "Control", "Insert"\]/);
  assert.match(content, /const forcePressed = \(event\) => shortcutPressed\(event, shortcutKeys\.force\) \|\| shortcutPressed\(event, "Insert"\)/);
  assert.match(content, /shortcutPressed\(event, shortcutKeys\.bypass\)/);
  assert.match(pageHook, /event\.key === "Insert"/);
  assert.match(pageHook, /held\.has\(shortcuts\.force \|\| "Shift"\) \|\| held\.has\("Insert"\)/);
  assert.equal((popup.match(/<option>Insert<\/option>/g) || []).length, 2);
});

test('a stuck Alt/Shift modifier is reset on focus change so capture on claude.ai and Rapidgator never silently stalls', () => {
  assert.match(content, /const clearHeldShortcutKeys = \(\) => {/);
  assert.match(content, /window\.addEventListener\("blur", clearHeldShortcutKeys\)/);
  assert.match(content, /window\.addEventListener\("focus", clearHeldShortcutKeys\)/);
  assert.match(content, /document\.addEventListener\("visibilitychange", \(\) => {\s*if \(document\.hidden\) clearHeldShortcutKeys\(\);/);
  assert.match(pageHook, /const clearHeld = \(\) => held\.clear\(\);/);
  assert.match(pageHook, /addEventListener\("blur", clearHeld, true\)/);
  assert.match(pageHook, /addEventListener\("focus", clearHeld, true\)/);
  assert.match(pageHook, /document\.addEventListener\("visibilitychange", \(\) => {\s*if \(document\.hidden\) clearHeld\(\);/);
});


test('ChatGPT generated-file normal clicks arm a short automatic force transaction', () => {
  assert.match(content, /chatgptDownloadGesture/);
  assert.match(content, /APOCALIPSE_FORCE_NEXT", ttlMs: 8000/);
  assert.match(pageHook, /chatgptDownloadControl/);
  assert.match(pageHook, /CHATGPT_AUTO_FORCE_ARMED/);
  assert.match(pageHook, /chatgptAutoForceUntil = Date\.now\(\) \+ 8000/);
  assert.match(pageHook, /forceActive\(\) \|\| chatgptAutoForceActive\(\)/);
  assert.match(pageHook, /\^sandbox:/);
});
