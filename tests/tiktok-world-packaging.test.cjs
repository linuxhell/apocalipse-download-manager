const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.join(__dirname, '..', 'browser-extension');

test('MAIN and isolated worlds use distinct URLs for identical TikTok resolver code', () => {
  const manifest = JSON.parse(fs.readFileSync(path.join(root, 'manifest.json'), 'utf8'));
  const main = manifest.content_scripts.filter(entry => entry.world === 'MAIN').flatMap(entry => entry.js);
  const isolated = manifest.content_scripts.filter(entry => entry.world !== 'MAIN').flatMap(entry => entry.js);
  assert.ok(main.includes('tiktok-identity-main.js'));
  assert.ok(isolated.includes('tiktok-identity.js'));
  assert.deepEqual(main.filter(file => isolated.includes(file)), [], 'do not reuse static script resource URLs across execution worlds');
  // Keep one resolver implementation, shipped under distinct resource names.
  // On resolver edits, copy tiktok-identity.js to tiktok-identity-main.js.
  assert.equal(fs.readFileSync(path.join(root, 'tiktok-identity-main.js'), 'utf8'),
    fs.readFileSync(path.join(root, 'tiktok-identity.js'), 'utf8'));
});
