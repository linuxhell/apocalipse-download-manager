const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

const ui = path.join(__dirname, "..", "apps", "desktop", "ui");
const read = (file) => fs.readFileSync(path.join(ui, file), "utf8");
const LOCALES = ["en", "pt-BR", "zh-CN"];

function loadCatalogs(file) {
  const text = read(file);
  let code;
  if (file === "ed2k.js") {
    code = `${text.slice(text.indexOf("const catalogs = {"), text.indexOf("  let locale"))}\nthis.catalogs = catalogs;`;
  } else {
    const lines = text.split("\n");
    const stop = lines.findIndex((line) => /^window\.apocalipseCatalogs|^const t = |^let locale/.test(line));
    const head = lines.slice(0, stop < 0 ? lines.length : stop);
    const assigns = lines.filter((line) => /^Object\.assign\(catalogs/.test(line));
    code = `${[...head.filter((line) => !/^Object\.assign\(catalogs/.test(line)), ...assigns].join("\n")}\nthis.catalogs = catalogs;`;
  }
  const context = { window: {} };
  vm.createContext(context);
  vm.runInContext(code, context);
  return context.catalogs;
}

const pages = {
  "app.js": ["index.html", "app.js", "nibiru-media.js", "diagnostics-panel.js"],
  "link.js": ["link.html", "link.js"],
  "ed2k.js": ["ed2k.html", "ed2k.js"],
};

for (const [catalogFile, sources] of Object.entries(pages)) {
  test(`${catalogFile}: English, Portuguese and Chinese define the same keys and placeholders`, () => {
    const catalogs = loadCatalogs(catalogFile);
    const reference = Object.keys(catalogs.en).sort();
    for (const locale of LOCALES.slice(1)) {
      assert.deepEqual(Object.keys(catalogs[locale]).sort(), reference, `${locale} keys differ from en`);
      for (const key of reference) {
        const placeholders = (value) => (String(value).match(/\{[a-zA-Z]+\}/g) || []).sort().join();
        assert.equal(placeholders(catalogs[locale][key]), placeholders(catalogs.en[key]), `${locale}.${key} placeholders`);
        assert.ok(String(catalogs[locale][key]).trim(), `${locale}.${key} is empty`);
      }
    }
    // Every static key the page asks for must exist (dynamic prefixes and the
    // product name set programmatically are excluded).
    const used = new Set();
    for (const file of sources) {
      const text = read(file);
      for (const m of text.matchAll(/data-i18n(?:-[a-z-]+)?="([^"]+)"/g)) used.add(m[1]);
      for (const m of text.matchAll(/(?<![A-Za-z0-9_$.])t\(\s*["'`]([A-Za-z0-9_.-]+)["'`]/g)) used.add(m[1]);
    }
    used.delete("ed2kKad");
    used.delete("stage_");
    const missing = [...used].filter((key) => !(key in catalogs.en));
    assert.deepEqual(missing, []);
  });
}

test("technical failure codes are explained in every language and keep the raw code", () => {
  const text = read("app.js");
  const catalogs = loadCatalogs("app.js");
  const block = text.slice(text.indexOf("const failureRules = ["));
  const end = block.indexOf("};\n", block.indexOf("const failureText")) + 3;
  const source = block.slice(0, end);
  for (const locale of LOCALES) {
    const context = { t: (key) => catalogs[locale][key] || catalogs.en[key] || key };
    vm.createContext(context);
    vm.runInContext(`${source}\nthis.failureText = failureText;`, context);
    for (const raw of [
      "destination already exists",
      "yt_dlp_destination_already_exists",
      "sha256 mismatch",
      "download_checksum_mismatch",
      "nibiru_m3u8_output_size_mismatch",
      "incomplete download: received 5 of 10",
      "hls_requires_nibiru_m3u8",
      "metalink_children_rejected",
      "blob_upload_abandoned",
      "signed_stream_url_expired",
      "xtreme_unavailable:boom",
      "external_engine_unavailable: missing",
    ]) {
      const shown = context.failureText(raw);
      assert.notEqual(shown, raw, `${raw} is not translated`);
      assert.ok(shown.includes(raw), "raw detail kept");
      assert.ok(!/^failure[A-Z]/.test(shown), "unresolved catalog key");
    }
    assert.equal(context.failureText("some_unknown_code"), "some_unknown_code");
  }
});
