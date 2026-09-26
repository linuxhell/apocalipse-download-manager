const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const aria2 = fs.readFileSync(path.join(root, "apps/desktop/src-tauri/src/aria2.rs"), "utf8");
const main = fs.readFileSync(path.join(root, "apps/desktop/src-tauri/src/main.rs"), "utf8");

test("aria2 obtains magnet metadata in a disposable staging directory", () => {
  assert.match(aria2, /"bt-metadata-only"/);
  assert.match(aria2, /"bt-save-metadata"/);
  assert.match(main, /join\("metadata-staging"\)/);
  assert.match(main, /remove_dir_all\(&temporary\)/);
});

test("ADM retains metadata in memory for selection without a persistent torrent file", () => {
  assert.match(main, /pending_torrent_metadata: Mutex<HashMap<String, Vec<u8>>>/);
  assert.match(main, /active_torrent_metadata: Mutex<HashMap<DownloadId, Vec<u8>>>/);
  assert.match(main, /inspect_torrent_bytes\(&bytes\)/);
  assert.match(main, /endpoint\.add_bittorrent\(&bytes, &task\.destination, &task\.torrent_selection/);
});
