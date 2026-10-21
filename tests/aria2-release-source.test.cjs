const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

test("all packaging workflows fetch aria2 and checksums from the same available source", () => {
  for (const workflow of ["validate-portable.yml", "release.yml"]) {
    const source = fs.readFileSync(path.join(__dirname, "../.github/workflows", workflow), "utf8");
    const urls = source.match(/https:\/\/api\.github\.com\/repos\/[^\s"']+\/releases\/[^\s"']+/g) || [];
    const aria2 = urls.filter(url => /aria2/.test(url));
    assert.ok(aria2.length >= 3, workflow);
    for (const url of aria2) {
      assert.ok(url.startsWith("https://api.github.com/repos/FerroDownload/aria2-static-builds/releases/"), `${workflow}: ${url}`);
    }
    assert.doesNotMatch(source, /api\.github\.com\/repos\/linuxhell\/aria2-ultra\//);
    assert.match(source, /hashlib\.sha256|sha256sum|shasum|Get-FileHash/i, workflow);
  }
});
