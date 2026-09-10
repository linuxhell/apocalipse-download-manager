const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const vm = require('node:vm');
const { webcrypto, createHash } = require('node:crypto');

test('normal-mode resource salt is independent of the exported producer ID', async () => {
  const context = vm.createContext({ crypto: webcrypto, URL, TextEncoder,
    chrome: { storage: { local: { get: async () => ({ admDiagnosticSession: null }) }, onChanged: { addListener() {} } } } });
  vm.runInContext(readFileSync(join(__dirname, '../browser-extension/diagnostics.js'), 'utf8'), context);
  const d = context.ApocalipseDiagnostics; await d.ready;
  const source = 'https://cdn.example/private/fixture.mp4?signature=TEST';
  const a = await d.resource(source), b = await d.resource(source);
  const publicSaltHash = createHash('sha256').update(`${d.producerId}\0${source}`).digest('hex').slice(0, 24);
  assert.equal(a.resourceId, b.resourceId);
  assert.notEqual(a.resourceId, publicSaltHash);
  assert.equal(d.session(), null);
});
