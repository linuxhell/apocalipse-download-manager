const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

test("release packages reuse the validated build with its pinned native engine", () => {
  const ci = fs.readFileSync(path.join(__dirname, "../.github/workflows/ci.yml"), "utf8");
  const release = fs.readFileSync(path.join(__dirname, "../.github/workflows/release.yml"), "utf8");
  assert.match(ci, /cargo install --git https:\/\/github\.com\/linuxhell\/nibiru-xtreme --rev "\$rev"/);
  assert.match(ci, /Cargo\.lock/);
  assert.match(release, /uses: \.\/\.github\/workflows\/ci\.yml/);
  assert.match(release, /needs: validate/);
  assert.match(release, /pattern: apocalipse-download-manager-\*/);
  assert.match(release, /sha256sum apocalipse-\*/);
  assert.doesNotMatch(release, /Download xtreme backend|cargo build --release/);
});
