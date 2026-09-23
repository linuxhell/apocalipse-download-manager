const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const html = fs.readFileSync(path.join(root, "apps/desktop/ui/index.html"), "utf8");
const app = fs.readFileSync(path.join(root, "apps/desktop/ui/app.js"), "utf8");
const css = fs.readFileSync(path.join(root, "apps/desktop/ui/styles.css"), "utf8");
const main = fs.readFileSync(path.join(root, "apps/desktop/src-tauri/src/main.rs"), "utf8");
const aria2 = fs.readFileSync(path.join(root, "apps/desktop/src-tauri/src/aria2.rs"), "utf8");
const classifier = fs.readFileSync(path.join(root, "crates/apocalipse-core/src/classifier.rs"), "utf8");
const strategy = fs.readFileSync(path.join(root, "crates/apocalipse-core/src/strategy.rs"), "utf8");

test("Apocalipse AI is fully removed: no files, no nav entry, no orphaned markup/CSS", () => {
  for (const file of ["apocalipse-ai-core.js", "apocalipse-ai-ui.js", "apocalipse-ai-local-model.js"]) {
    assert.ok(!fs.existsSync(path.join(root, "apps/desktop/ui", file)), `${file} should be deleted`);
  }
  assert.doesNotMatch(html, /apocalipse-ai|data-page="ai"|id="ai-panel"|ai-corrections-dialog/i);
  assert.doesNotMatch(app, /window\.ApocalipseAI|#ai-panel|apocalipse-ai-opened/);
  assert.doesNotMatch(css, /\.ai-panel|\.ai-corrections|\.ai-nav-icon|\.ai-message/);
  assert.ok(!fs.existsSync(path.join(root, "tests/apocalipse-ai.test.cjs")));
});

test("Apocalipse ED2K replaces Apocalipse AI's nav slot and opens its own themed window", () => {
  assert.match(html, /data-page="ed2k"/);
  assert.doesNotMatch(html, /Apocalipse AI/);
  assert.match(app, /button\.dataset\.page === "ed2k"/);
  assert.match(app, /invoke\("open_ed2k_window"\)/);
  assert.match(main, /async fn open_ed2k_window\(app: tauri::AppHandle\)/);
  assert.match(main, /get_webview_window\("apocalipse-ed2k"\)/);
  assert.match(main, /WebviewUrl::App\("ed2k\.html"\.into\(\)\)/);
  assert.ok(fs.existsSync(path.join(root, "apps/desktop/ui/ed2k.html")));
  assert.ok(fs.existsSync(path.join(root, "apps/desktop/ui/ed2k.js")));
  const ed2kJs = fs.readFileSync(path.join(root, "apps/desktop/ui/ed2k.js"), "utf8");
  // Same theme-follow fix as the Link window: pull the real value from the
  // backend and listen for live changes, instead of trusting a possibly
  // stale/empty per-window localStorage copy.
  assert.match(ed2kJs, /invoke\("get_application_theme"\)/);
  assert.match(ed2kJs, /listen\?\.\("theme-changed",/);
});

test("ed2k:// links classify to their own DownloadKind and route through the shared aria2next engine, not a second instance", () => {
  assert.match(classifier, /Ed2k,?\s*\n?\s*}/s);
  assert.match(classifier, /input\.starts_with\("ed2k:\/\/"\)/);
  assert.match(strategy, /DownloadKind::Torrent \| DownloadKind::Magnet \| DownloadKind::Ed2k/);
  // Every ED2K command reuses aria2_endpoint(), the same singleton HTTP and
  // BitTorrent downloads already share - never a second spawned runtime.
  const ed2kCommandsBlock = main.slice(main.indexOf("fn ed2k_connect"), main.indexOf("fn ed2k_search_results") + 500);
  assert.match(ed2kCommandsBlock, /aria2_endpoint\(&state,/g);
  assert.doesNotMatch(ed2kCommandsBlock, /aria2::Runtime::spawn/);
});

test("ED2K server list is validated and persisted, and search/connect use the documented aria2-next RPC surface", () => {
  assert.match(main, /fn normalize_ed2k_server/);
  assert.match(main, /ed2k_servers: Vec<String>/);
  assert.match(aria2, /pub async fn set_ed2k_servers/);
  assert.match(aria2, /"aria2\.changeGlobalOption"/);
  assert.match(aria2, /pub async fn ed2k_search/);
  assert.match(aria2, /"aria2\.ed2kSearch"/);
  assert.match(aria2, /"aria2\.getEd2kSearchResults"/);
});

test("ed2k link download extracts its filename from the pipe-delimited link, not a bogus path segment", () => {
  assert.match(main, /classify_url\(source\) == Some\(DownloadKind::Ed2k\)/);
  assert.match(main, /source\.split\('\|'\)\.nth\(2\)/);
});

test("ED2K server list auto-updates from a configurable server.met URL, mirroring aMule's own Ed2kServersUrl default", () => {
  // aria2-next's --ed2k-server-list only accepts a local file path (its own
  // docs confirm this, not a remote URL), so the configured URL must be
  // downloaded to a local file first and that file's path applied via RPC.
  assert.match(main, /fn default_ed2k_server_list_url\(\) -> String \{\s*\n\s*"https:\/\/upd\.emule-security\.org\/server\.met"\.to_owned\(\)/);
  assert.match(main, /ed2k_server_list_url: String,/);
  assert.match(main, /async fn ed2k_update_server_list\(/);
  assert.match(main, /fs::write\(&path, &bytes\)/);
  assert.match(aria2, /pub async fn set_ed2k_server_list_file/);
  assert.match(aria2, /"ed2k-server-list"\.into\(\)/);
  // Connect refreshes server.met first, matching aMule's "update at
  // startup" behavior, but falls back to whatever's already cached (or to
  // aria2-next's own built-in bootstrap servers) instead of hard-failing
  // when the network fetch itself fails.
  const connectBlock = main.slice(main.indexOf("async fn ed2k_connect("), main.indexOf("async fn ed2k_disconnect("));
  assert.match(connectBlock, /ed2k_update_server_list\(state\.clone\(\)\)\.await/);
  assert.match(connectBlock, /set_ed2k_server_list_file\(&cached\)/);
  // Backend and UI both expose the URL as editable, not hardcoded only.
  assert.match(main, /fn ed2k_get_server_list_url/);
  assert.match(main, /fn ed2k_set_server_list_url/);
  const ed2kJs = fs.readFileSync(path.join(root, "apps/desktop/ui/ed2k.js"), "utf8");
  assert.match(ed2kJs, /invoke\("ed2k_get_server_list_url"\)/);
  assert.match(ed2kJs, /invoke\("ed2k_update_server_list"\)/);
  const ed2kHtml = fs.readFileSync(path.join(root, "apps/desktop/ui/ed2k.html"), "utf8");
  assert.match(ed2kHtml, /id="ed2k-server-list-url"/);
});
