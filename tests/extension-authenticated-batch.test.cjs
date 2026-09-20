const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const background = fs.readFileSync(path.join(__dirname, "../browser-extension/background.js"), "utf8");
const manifest = JSON.parse(fs.readFileSync(path.join(__dirname, "../browser-extension/manifest.json"), "utf8"));

test("0.3.162 isolates authenticated cookies for each batch item", () => {
  assert.equal(manifest.version, "0.3.162");
  assert.match(background, /const cookieHeader = await cookieHeaderFor\(\[downloadUrl, item\.audioUrl\]\)/);
  assert.doesNotMatch(background, /cookieHeaderFor\(\[\.\.\.items\.flatMap/);
  assert.match(background, /failures\.push/);
  assert.match(background, /partial: failures\.length > 0/);
});
