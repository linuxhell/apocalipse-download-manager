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

test("ed2k:// links classify and actually dispatch through the shared aria2next engine", () => {
  assert.match(classifier, /Ed2k,?\s*\n?\s*}/s);
  assert.match(classifier, /input\.starts_with\("ed2k:\/\/"\)/);
  assert.match(strategy, /DownloadKind::Torrent \| DownloadKind::Magnet \| DownloadKind::Ed2k/);
  // Regression: the original ED2K commit claimed this route existed, but the
  // real dispatcher only matched HTTP/AcceleratedHttp/FTP. Keep Ed2k in the
  // actual run_aria2_download branch.
  assert.match(main, /DownloadKind::Ftp\s*\n\s*\| DownloadKind::Ed2k/);
  const ed2kCommandsBlock = main.slice(main.indexOf("fn ed2k_connect"), main.indexOf("fn ed2k_search_results") + 500);
  assert.match(ed2kCommandsBlock, /aria2_endpoint\(&state,/g);
  assert.doesNotMatch(ed2kCommandsBlock, /aria2::Runtime::spawn/);
});

test("ED2K server sources are attached to real aria2-next download/search requests, not ignored global options", () => {
  assert.match(main, /fn normalize_ed2k_server/);
  assert.match(main, /ed2k_servers: Vec<String>/);
  assert.match(aria2, /pub async fn add_ed2k_download/);
  assert.match(aria2, /options\.insert\("ed2k-server"\.into\(\)/);
  assert.match(aria2, /options\.insert\(\s*"ed2k-server-list"\.into\(\)/s);
  assert.match(main, /\.add_ed2k_download\(/);
  assert.match(aria2, /pub async fn ed2k_search/);
  assert.match(aria2, /"aria2\.ed2kSearch"/);
  assert.match(aria2, /Value::Object\(options\)/);
  assert.match(aria2, /"aria2\.getEd2kSearchResults"/);
  assert.doesNotMatch(aria2, /fn set_ed2k_servers/);
  assert.doesNotMatch(aria2, /fn set_ed2k_server_list_file/);
});

test("ed2k link download extracts its filename from the pipe-delimited link, not a bogus path segment", () => {
  assert.match(main, /classify_url\(source\) == Some\(DownloadKind::Ed2k\)/);
  assert.match(main, /source\.split\('\|'\)\.nth\(2\)/);
});

test("ed2k: system association is available in settings and platform handlers", () => {
  assert.match(main, /ASSOCIATION_IDS: \[&str; 6\] = \["m3u8", "torrent", "magnet", "ed2k", "ftp", "sftp"\]/);
  assert.match(main, /lower\.starts_with\("ed2k:"\)/);
  assert.match(main, /\("ed2k", "x-scheme-handler\/ed2k"\)/);
  assert.match(html, /data-association="ed2k"/);
});

test("ED2K server list auto-updates safely from a configurable server.met URL", () => {
  // aria2-next's ed2k-server-list accepts a local server.met path, so ADM
  // downloads it into the portable data tree and supplies that path on each
  // ED2K download/search request.
  assert.match(main, /fn default_ed2k_server_list_url\(\) -> String \{\s*\n\s*"https:\/\/upd\.emule-security\.org\/server\.met"\.to_owned\(\)/);
  assert.match(main, /ed2k_server_list_url: String,/);
  assert.match(main, /fn validate_ed2k_server_met\(bytes: &\[u8\]\)/);
  assert.match(main, /count > 100_000/);
  assert.match(main, /async fn ed2k_update_server_list\(/);
  assert.match(main, /fs::write\(&staged, &bytes\)/);
  assert.match(main, /ed2k_server_list_payload_too_large/);
  assert.match(main, /met\.backup/);
  assert.match(aria2, /"ed2k-server-list"\.into\(\)/);

  // "Connect" prepares the engine and refreshes server.met. aria2-next has
  // no standalone ED2K connect/disconnect RPC; real handshakes begin with a
  // search/download task carrying these options.
  const connectBlock = main.slice(main.indexOf("async fn ed2k_connect("), main.indexOf("async fn ed2k_disconnect("));
  assert.match(connectBlock, /ed2k_update_server_list\(state\.clone\(\)\)\.await/);
  assert.match(connectBlock, /aria2_endpoint\(&state, true\)\.await/);
  assert.match(main, /aria2-next exposes no independent ED2K disconnect RPC/);

  // Backend and UI both expose the source URL as editable.
  assert.match(main, /fn ed2k_get_server_list_url/);
  assert.match(main, /fn ed2k_set_server_list_url/);
  const ed2kJs = fs.readFileSync(path.join(root, "apps/desktop/ui/ed2k.js"), "utf8");
  assert.match(ed2kJs, /invoke\("ed2k_get_server_list_url"\)/);
  assert.match(ed2kJs, /invoke\("ed2k_update_server_list"\)/);
  const ed2kHtml = fs.readFileSync(path.join(root, "apps/desktop/ui/ed2k.html"), "utf8");
  assert.match(ed2kHtml, /id="ed2k-server-list-url"/);
});
