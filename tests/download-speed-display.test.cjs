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
  assert.ok(ui.indexOf("const invoke =") < ui.indexOf('invoke("set_application_theme"'), "theme sync must run only after the desktop bridge is initialized");
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
  assert.match(desktopCss, /\.remove-options button:hover[\s\S]*color-mix\(in srgb, var\(--accent\) 14%, var\(--surface\)\)/);
  assert.match(desktopCss, /\.remove-options button\.danger:hover[\s\S]*color-mix\(in srgb, #ff5364 12%, var\(--surface\)\)/);
});

test("HLS uses the selected destination for process and temporary files", () => {
  assert.match(desktop, /command\.current_dir\(directory\)/);
  assert.match(desktop, /\.arg\("--tmp-dir"\)\s*\.arg\(directory\)/);
});

test("failed thumbnails restore the compact icon without covering progress", () => {
  assert.match(ui, /icon\.classList\.remove\("has-thumbnail"\)/);
  assert.doesNotMatch(ui, /else if \(task\.thumbnail\)\s*\{\s*icon\.classList\.add\("has-thumbnail"\)/);
  assert.match(desktopCss, /\.download-icon\s*\{[\s\S]*width:\s*32px;[\s\S]*height:\s*32px;/);
});

test("task state and action controls use the active theme instead of dark constants", () => {
  assert.match(desktopCss, /\.download-state\s*\{[\s\S]*background:\s*color-mix\(in srgb, var\(--accent\) 10%, var\(--surface\)\)/);
  assert.match(desktopCss, /\.task-action\s*\{[\s\S]*background:\s*var\(--surface-2\)/);
});

test("protected thumbnails are cached for the desktop handoff", () => {
  const worker = fs.readFileSync(path.join(root, "browser-extension/background.js"), "utf8");
  const content = fs.readFileSync(path.join(root, "browser-extension/content.js"), "utf8");
  assert.match(worker, /const fetchThumbnailDataUrl = async/);
  assert.match(worker, /const thumbnail = await portableThumbnail\(item\.thumbnail\)/);
  assert.match(worker, /thumbnailDataCache/);
  assert.match(desktop, /value\.len\(\) <= 600_000/);
  assert.match(worker, /APOCALIPSE_CAPTURE_VISIBLE_THUMBNAIL/);
  assert.match(worker, /chrome\.tabs\.captureVisibleTab/);
  assert.match(worker, /new OffscreenCanvas/);
  assert.match(content, /const captureThumbnailFor = async/);
});

test("feed thumbnails come from the exact visual player region before page metadata", () => {
  const content = fs.readFileSync(path.join(root, "browser-extension/content.js"), "utf8");
  assert.match(content, /const visualThumbnailFor = \(element\) =>/);
  assert.match(content, /if \(videos\.length > 1\) break/);
  assert.match(content, /if \(overlap < 0\.45\) continue/);
  assert.match(content, /element\?\.getAttribute\?\.\("poster"\),\s*visualThumbnailFor\(element\)/);
});

test("a closed browser becomes disconnected after the initial extension wait", () => {
  assert.match(ui, /bridgeDisconnected:\s*"Extension disconnected"/);
  assert.match(ui, /bridgeDisconnected:\s*"Extensão desconectada"/);
  assert.match(ui, /bridgeDisconnected:\s*"扩展已断开连接"/);
  assert.match(ui, /Date\.now\(\) - bridgeStatusStartedAt < 6000/);
});

test("download destinations and task copy follow the active theme palette", () => {
  assert.match(desktopCss, /\.destination-select\s*\{[\s\S]*color:\s*var\(--text\)/);
  assert.match(desktopCss, /\.destination-row\.unavailable \.destination-select span\s*\{\s*color:\s*var\(--muted\)/);
  assert.match(desktopCss, /\.download-info strong\s*\{[\s\S]*color:\s*color-mix/);
});

test("paused HLS cleanup includes its isolated segment workspace", () => {
  assert.match(desktop, /fn hls_workspace_path\(task: &DownloadTask\)/);
  assert.match(desktop, /torrent_root \|\| hls_workspace/);
});
