const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const vm = require('node:vm');
const source = readFileSync(require('node:path').join(__dirname, '../browser-extension/content.js'), 'utf8');
const handler = source.slice(source.indexOf('const handleDocumentClick ='), source.indexOf('document.addEventListener("click", handleDocumentClick'));

for (const [target, outcome] of [['desktop', 'click_handoff_acknowledged'], [null, 'click_handoff_failed']]) {
  test(`magnet records correlated click and ${outcome} without changing fallback`, async () => {
    const events = [], navigations = [];
    const anchor = { tagName: 'A', getAttribute: () => 'magnet:?xt=urn:btih:abc&dn=test' };
    const context = {
      URL, crypto: { randomUUID: () => 'click-id' },
      extensionContextActive: () => true, shortcutPressed: () => false,
      shortcutKeys: {}, document: { removeEventListener() {} },
      traceDiagnostic: (...args) => events.push(args),
      sendRuntimeMessageQuietly: () => Promise.resolve(target ? { target } : null),
      location: { assign: url => navigations.push(url) },
    };
    vm.createContext(context);
    vm.runInContext(`${handler}; globalThis.handleClick = handleDocumentClick;`, context);
    let prevented = false;
    context.handleClick({ button: 0, target: { closest: () => anchor }, preventDefault() { prevented = true; }, stopImmediatePropagation() {} });
    await new Promise(resolve => setImmediate(resolve));
    assert.equal(prevented, true);
    assert.deepEqual(events.map(x => x[0]), ['click_observed', 'click_handoff_started', outcome]);
    assert.ok(events.every(x => x[3] === 'click-id'));
    assert.equal(navigations.length, target ? 0 : 1);
  });
}
