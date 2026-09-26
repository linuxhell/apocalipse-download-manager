const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const main = fs.readFileSync(path.join(root, "apps/desktop/src-tauri/src/main.rs"), "utf8");
const ui = fs.readFileSync(path.join(root, "apps/desktop/ui/app.js"), "utf8");
const html = fs.readFileSync(path.join(root, "apps/desktop/ui/index.html"), "utf8");

test("torrent metadata is not saved persistently by default", () => {
  assert.match(main, /save_torrent_metadata: false/);
  assert.match(main, /if save \{\s*let path = persist_torrent_bytes/);
  assert.match(main, /load_torrent_metadata_without_saving/);
});

test("storage setting and cleanup live in the torrent section", () => {
  assert.match(html, /id="torrent-storage-controls"/);
  assert.match(html, /id="save-torrent-files"/);
  assert.match(html, /id="remove-saved-torrent-files"/);
  assert.match(ui, /set_torrent_storage_setting/);
  assert.match(ui, /remove_saved_torrent_metadata/);
  assert.doesNotMatch(ui, /deleteTorrentMetadataConfirm/);
});

test("saved files still used by existing tasks are protected from manual cleanup", () => {
  assert.match(main, /referenced\.contains\(&path\)/);
});
