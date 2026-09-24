const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const aria2 = fs.readFileSync(path.join(root, "apps/desktop/src-tauri/src/aria2.rs"), "utf8");
const main = fs.readFileSync(path.join(root, "apps/desktop/src-tauri/src/main.rs"), "utf8");
const start = aria2.indexOf("pub async fn save_magnet_metadata");
const end = aria2.indexOf("pub async fn set_download_limit", start);
const metadata = aria2.slice(start, end);

test("classic aria2 saves Magnet metadata as a real torrent file", () => {
  assert.ok(start >= 0 && end > start);
  assert.match(metadata, /"bt-metadata-only"\.into\(\),\s*Value::String\("true"\.into\(\)\)/s);
  assert.match(metadata, /"bt-save-metadata"\.into\(\),\s*Value::String\("true"\.into\(\)\)/s);
  assert.match(metadata, /"infoHash"/);
  assert.match(metadata, /\.torrent/);
  assert.doesNotMatch(metadata, /pause-metadata|followedBy|aria2\.getFiles/);
});

test("ADM stores and parses torrent files from data/torrents before payload", () => {
  assert.match(main, /runtime_root\.join\("torrents"\)/);
  assert.match(main, /materialize_torrent_metadata_file/);
  assert.match(main, /inspect_torrent_file\(&path\)/);
  assert.match(main, /torrent_metadata_path = Some\(path\.clone\(\)\)/);
  assert.match(main, /add_bittorrent[\s\S]*&bytes/);
});

test("retired followedBy metadata preview code is absent", () => {
  for (const retired of [
    "preview_magnet_metadata",
    "MetadataProbe",
    "pause-metadata",
    "followedBy",
    "aria2-metadata-inspection",
  ]) {
    assert.equal(aria2.includes(retired) || main.includes(retired), false, retired);
  }
});
