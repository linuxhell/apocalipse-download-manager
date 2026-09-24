const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const main = fs.readFileSync(path.join(root, "apps/desktop/src-tauri/src/main.rs"), "utf8");
const ui = fs.readFileSync(path.join(root, "apps/desktop/ui/app.js"), "utf8");
const model = fs.readFileSync(path.join(root, "crates/apocalipse-core/src/model.rs"), "utf8");

test("torrent metadata path persists with the task", () => {
  assert.match(model, /pub torrent_metadata_path: Option<PathBuf>/);
  assert.match(main, /runtime_root\.join\("torrents"\)/);
  assert.match(main, /item\.torrent_metadata_path = Some\(path\.clone\(\)\)/);
});

test("completed torrent disk removal asks about saved torrent metadata", () => {
  assert.match(ui, /activePage === "torrents"/);
  assert.match(ui, /stateKey\(task\.state\) === "completed"/);
  assert.match(ui, /window\.confirm\(t\("deleteTorrentMetadataConfirm"\)\)/);
  assert.match(ui, /deleteTorrentMetadata/);
  assert.match(main, /delete_torrent_metadata && task\.state == DownloadState::Completed/);
});

test("torrent metadata deletion prompt is localized", () => {
  const matches = ui.match(/deleteTorrentMetadataConfirm:/g) || [];
  assert.equal(matches.length, 3);
  assert.match(ui, /Deseja apagar também o\(s\) arquivo\(s\) \.torrent/);
  assert.match(ui, /Also delete the saved \.torrent file\(s\)/);
  assert.match(ui, /是否同时删除保存在 data\/torrents 中的 \.torrent 文件/);
});


test("pausing keeps the managed torrent copy so resume does not depend on the original file", () => {
  const pauseStart = main.indexOf("fn pause_download(");
  const resumeStart = main.indexOf("fn resume_download(", pauseStart);
  const pause = main.slice(pauseStart, resumeStart);
  assert.ok(pauseStart >= 0 && resumeStart > pauseStart);
  assert.doesNotMatch(pause, /torrent_metadata_path\s*=\s*None|remove_path_with_retry\([^\n]*torrent_metadata/i);
  assert.match(main, /task\s*\.torrent_metadata_path[\s\S]*add_bittorrent[\s\S]*&bytes/);
  assert.match(main, /persist_torrent_bytes\(state, &bytes\)/);
});


test("shared torrent metadata is preserved while another task still references it", () => {
  assert.match(main, /retained_torrent_metadata_paths/);
  assert.match(main, /torrent\.metadata_preserved_shared/);
  assert.match(main, /deleted_torrent_metadata_paths\.insert\(path\.to_path_buf\(\)\)/);
});
