const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

test("all packaging workflows fetch xtreme and checksums from the same available source", () => {
  for (const workflow of ["test-build.yml", "validate-portable.yml", "release.yml"]) {
    const source = fs.readFileSync(path.join(__dirname, "../.github/workflows", workflow), "utf8");
    const urls = source.match(/https:\/\/api\.github\.com\/repos\/[^\s"']+\/releases\/[^\s"']+/g) || [];
    const xtreme = urls.filter(url => /xtreme/.test(url));
    assert.ok(xtreme.length >= 2, workflow);
    assert.match(source, /repos\/linuxhell\/nibiru-xtreme\/releases["\s]/);
    for (const url of xtreme) {
      assert.ok(url.startsWith("https://api.github.com/repos/linuxhell/nibiru-xtreme/releases/"), `${workflow}: ${url}`);
    }
    assert.doesNotMatch(source, /api\.github\.com\/repos\/linuxhell\/xtreme-ultra\//);
    assert.match(source, /hashlib\.sha256|sha256sum|shasum|Get-FileHash/i, workflow);
  }
});
