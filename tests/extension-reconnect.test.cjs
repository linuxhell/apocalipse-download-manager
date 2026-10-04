const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const path = require('node:path');
const popup = fs.readFileSync(path.join(__dirname, '../browser-extension/popup.js'), 'utf8');
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
