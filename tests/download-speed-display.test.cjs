const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");

const root = path.resolve(__dirname, "..");
const ui = fs.readFileSync(path.join(root, "apps/desktop/ui/app.js"), "utf8");
const desktop = fs.readFileSync(
  path.join(root, "apps/desktop/src-tauri/src/main.rs"),
  "utf8",
);
const cargo = fs.readFileSync(path.join(root, "Cargo.toml"), "utf8");
const tauri = JSON.parse(
  fs.readFileSync(path.join(root, "apps/desktop/src-tauri/tauri.conf.json"), "utf8"),
);
const desktopHtml = fs.readFileSync(path.join(root, "apps/desktop/ui/index.html"), "utf8");
const desktopCss = fs.readFileSync(path.join(root, "apps/desktop/ui/styles.css"), "utf8");

test("active downloads keep the engine-reported speed visible", () => {
  assert.match(
    ui,
    /const externalSpeed = active \? Number\(task\.download_speed\) \|\| 0 : 0;/,
  );
  assert.doesNotMatch(
    ui,
    /const externalSpeed = now - changedAt < 2000/,
  );
});

test("streamed browser recordings publish a global core speed", () => {
  assert.match(desktop, /speed_sample_at: Instant/);
  assert.match(desktop, /task\.download_speed = Some\(download_speed\)/);
  assert.match(desktop, /instantaneous as f64 \* 0\.65/);
  assert.match(desktop, /"blob-upload-progress"/);
  assert.match(ui, /listen\?\.\("blob-upload-progress"/);
  assert.match(ui, /renderDownloads\(true\)/);
});

test("quiet clipboard polling does not flood diagnostics", () => {
  assert.match(
    ui,
    /if \(!quiet\.has\(command\) && command !== "record_ui_diagnostic"\)/,
  );
  assert.match(
    desktop,
    /let Ok\(value\) = app\.clipboard\(\)\.read_text\(\) else \{\s*return Ok\(None\);/,
  );
});

test("desktop package and interface versions cannot diverge", () => {
  const packageVersion = cargo.match(/\[workspace\.package\][\s\S]*?version = "([^"]+)"/)?.[1];
  assert.ok(packageVersion, "workspace package version is missing");
  assert.equal(tauri.version, packageVersion);
  assert.match(ui, /invoke\("get_app_version"\)/);
});

test("five readable light themes are available", () => {
  for (const theme of ["pearlblue", "whiteaurora", "goldenivory", "crystalrose", "polarmint"]) {
    assert.match(desktopHtml, new RegExp(`value="${theme}"`));
    assert.match(desktopCss, new RegExp(`data-theme="${theme}"`));
  }
  assert.match(desktopCss, /color-scheme:light/);
  assert.match(desktopCss, /--panel:#fff/);
  assert.match(desktopCss, /\.logs-panel,\.link-panel>div,\.diagnostics-panel/);
  assert.match(desktop, /"theme": theme/);
  assert.match(ui, /invoke\("set_application_theme"/);
});
