const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const legacy = JSON.parse(fs.readFileSync(path.join(root, 'apps/desktop/src-tauri/legacy-engine-fields.json'), 'utf8'));

test('production sources contain no retired engine identifier', () => {
  const prefix = Object.keys(legacy.keys)[0].split('_')[0].toLowerCase();
  const walk = dir => fs.readdirSync(dir, { withFileTypes: true }).flatMap(item => {
    const file = path.join(dir, item.name);
    return item.isDirectory() ? walk(file) : [file];
  });
  for (const directory of ['apps', 'crates', 'browser-extension', 'scripts', '.github']) {
    for (const file of walk(path.join(root, directory))) {
      if (!/\.(rs|js|html|ps1|yml)$/.test(file)) continue;
      assert.equal(fs.readFileSync(file, 'utf8').toLowerCase().includes(prefix), false, file);
    }
  }
});

test('the native RPC client uses only the engine namespace', () => {
  const source = fs.readFileSync(path.join(root, 'apps/desktop/src-tauri/src/xtreme.rs'), 'utf8');
  const methods = [...source.matchAll(/\.call\(\s*"([a-z]+\.[A-Za-z]+)"/g)].map(match => match[1]);
  assert.ok(methods.includes('nibiru.resume'));
  assert.ok(methods.includes('nibiru.removeResult'));
  assert.ok(methods.every(method => method.startsWith('nibiru.')), methods.join(', '));
});

test('Logs is directly below Settings and no longer a settings pane', () => {
  const html = fs.readFileSync(path.join(root, 'apps/desktop/ui/index.html'), 'utf8');
  assert.match(html, /data-page="settings"[^]*?<\/button><button data-page="logs"/);
  assert.doesNotMatch(html, /data-settings-tab="logs"|data-pane="logs"/);
  assert.match(html, /id="logs-page-panel"/);
  for (const id of ['export-logs', 'log-search', 'log-level', 'log-editor']) {
    assert.equal((html.match(new RegExp(`id="${id}"`, 'g')) || []).length, 1, id);
  }
});
