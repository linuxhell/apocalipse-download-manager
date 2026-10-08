const assert = require('node:assert/strict');
const { createHash } = require('node:crypto');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');

const manifest = JSON.parse(readFileSync(join(__dirname, '..', 'browser-extension', 'manifest.json'), 'utf8'));

// Chromium derives the extension ID from the manifest "key": the first 128 bits of
// SHA-256 of the DER public key, written with the letters a-p.
const idFromKey = (key) => [...createHash('sha256').update(Buffer.from(key, 'base64')).digest('hex').slice(0, 32)]
  .map((digit) => String.fromCharCode(97 + Number.parseInt(digit, 16))).join('');

test('the extension keeps a fixed Chromium ID so reinstalling does not drop the pairing token', () => {
  assert.equal(typeof manifest.key, 'string');
  assert.equal(idFromKey(manifest.key), 'lfgkfogkggkgacahaidkbhggdolpojjf');
});
