const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');

const root = join(__dirname, '..');
const background = readFileSync(join(root, 'browser-extension/background.js'), 'utf8');
const contentScript = readFileSync(join(root, 'browser-extension/content.js'), 'utf8');
const pageHook = readFileSync(join(root, 'browser-extension/page-hook.js'), 'utf8');
const manifest = JSON.parse(readFileSync(join(root, 'browser-extension/manifest.json'), 'utf8'));

test('0.3.169 repairs capture scripts in tabs that were already open during an extension reload', () => {
  assert.equal(manifest.version, '0.3.175');
  assert.ok(manifest.permissions.includes('scripting'));
  assert.match(background, /APOCALIPSE_CONTENT_PING/);
  assert.match(background, /chrome\.scripting\.executeScript/);
  assert.match(background, /files:\s*recoveryFiles/);
  assert.match(background, /world:\s*"MAIN"[\s\S]*files:\s*\["page-hook\.js"\]/);
  assert.match(background, /runtime\.onInstalled\.addListener/);
  assert.match(background, /repairOpenCaptureTabs\(/);
  assert.match(background, /tabs\?\.onActivated\?\.addListener/);
  assert.match(background, /tabs\?\.onUpdated\?\.addListener/);
});

test('capture-layer health is explicit in forensic logs', () => {
  for (const marker of [
    'capture.layer_missing',
    'capture.layer_reinjected',
    'capture.layer_reinject_failed',
    'capture.layer_content_ready',
    'capture.layer_main_hook_ready',
    'capture.layer_healthy',
    'capture.layer_main_hook_repair_unverified',
  ]) assert.match(background, new RegExp(marker.replaceAll('.', '\\.')));
  assert.match(contentScript, /APOCALIPSE_CONTENT_READY/);
  assert.match(contentScript, /APOCALIPSE_MAIN_HOOK_READY/);
  assert.match(contentScript, /hookReady:\s*mainHookReady/);
  assert.match(pageHook, /type:\s*"hook-pong"/);
});

test('Insert remains a direct force path even when the configured force key is different', () => {
  assert.match(contentScript, /shortcutPressed\(event, shortcutKeys\.force\) \|\| shortcutPressed\(event, "Insert"\)/);
  assert.match(pageHook, /held\.has\(shortcuts\.force \|\| "Shift"\) \|\| held\.has\("Insert"\)/);
});


test('heartbeat distinguishes healthy hooks from verified repairs', () => {
  assert.match(background, /CAPTURE_LAYER_HEALTH_LOG_MS = 300_000/);
  assert.match(background, /logCaptureLayerHealthy/);
  assert.match(background, /capture\.layer_healthy/);
  assert.match(background, /capture\.layer_main_hook_repaired[\s\S]*verified=true/);
  assert.match(background, /capture\.layer_main_hook_repair_unverified/);
  assert.match(background, /waitForMainHookReady/);
  assert.match(background, /markCaptureLayerHealth/);
});
