const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const path = require('node:path');
const app = fs.readFileSync(path.join(__dirname, '../apps/desktop/ui/app.js'), 'utf8');
const popup = fs.readFileSync(path.join(__dirname, '../browser-extension/popup.js'), 'utf8');
const restore = app.slice(app.indexOf('const applicationThemeReady ='), app.indexOf('invoke("get_app_version")'));

for (const cache of [null, 'void', 'samurai']) {
  test(`startup restores persisted theme over webview cache ${cache}`, async () => {
    const stored = new Map(cache === null ? [] : [['apocalipse.theme', cache]]);
    const calls = [], applied = [];
    let refreshed = 0;
    const context = vm.createContext({
      valid: ['void', 'samurai', 'cyberpunk'],
      localStorage: { setItem: (key, value) => stored.set(key, value) },
      invoke: async (command) => { calls.push(command); return 'cyberpunk'; },
      applyTheme: theme => applied.push(theme), applyAppearance: () => {},
      themeStudio: { leave: () => {} }, syncAppearanceControls: () => refreshed++, reportUiError: () => assert.fail('unexpected error'),
    });
    vm.runInContext(restore, context);
    await vm.runInContext('applicationThemeReady', context);
    assert.deepEqual(calls, ['get_application_theme']);
    assert.equal(stored.get('apocalipse.theme'), 'cyberpunk');
    assert.deepEqual(applied, ['cyberpunk']);
    assert.equal(refreshed, 1);
  });
}

test('failed theme restoration does not overwrite saved configuration or cached theme', async () => {
  let writes = 0, errors = 0;
  const context = vm.createContext({
    valid: ['void'], localStorage: { setItem: () => writes++ },
    invoke: async () => { throw new Error('unavailable'); },
    applyTheme: () => assert.fail('must not apply a fallback'), applyAppearance: () => {},
    syncAppearanceControls: () => {}, reportUiError: () => errors++,
  });
  vm.runInContext(restore, context);
  await vm.runInContext('applicationThemeReady', context);
  assert.equal(writes, 0);
  assert.equal(errors, 1);
});

const recover = popup.slice(popup.indexOf('const recoverPairedWorker ='), popup.indexOf('const showBridgeError ='));
for (const healthy of [true, false]) {
  test(`automatic worker recovery preserves pairing token (healthy=${healthy})`, async () => {
    const messages = [], warnings = [];
    const context = vm.createContext({
      workerSelfTest: async () => ({ ok: healthy, error: healthy ? undefined : 'service_worker_timeout' }),
      chrome: { runtime: { sendMessage: message => messages.push(message) } },
      showWorkerWarning: async worker => warnings.push(worker),
    });
    vm.runInContext(recover, context);
    await vm.runInContext('recoverPairedWorker("existing-token")', context);
    assert.equal(messages.length, healthy ? 0 : 1);
    assert.equal(warnings.length, healthy ? 0 : 1);
    if (!healthy) {
      assert.equal(messages[0].type, 'APOCALIPSE_PAIR');
      assert.equal(messages[0].token, 'existing-token');
    }
  });
}

test('both initial health check and reconnect polling invoke worker recovery', () => {
  const startup = popup.slice(popup.indexOf('chrome.storage.local.get({ pairingToken: "" }, async'), popup.indexOf('const saveShortcuts ='));
  const polling = popup.slice(popup.indexOf('setInterval(async () => {'), popup.indexOf('document.querySelector("#connect").onclick'));
  assert.match(startup, /await recoverPairedWorker\(pairingToken\)/);
  assert.match(polling, /if \(reconnected\) await recoverPairedWorker\(pairingToken\)/);
});
