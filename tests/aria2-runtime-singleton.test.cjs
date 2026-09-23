const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');

const root = join(__dirname, '..');
const aria2 = readFileSync(join(root, 'apps/desktop/src-tauri/src/aria2.rs'), 'utf8');
const main = readFileSync(join(root, 'apps/desktop/src-tauri/src/main.rs'), 'utf8');

test('aria2 daemon is tied to the lifetime of the ADM parent process', () => {
  assert.match(aria2, /--stop-with-process=\{\}/);
  assert.match(aria2, /std::process::id\(\)/);
  assert.match(aria2, /pub fn pid\(&self\) -> u32/);
  assert.match(main, /aria2\.runtime_spawned/);
  assert.match(main, /stop_with_parent=true/);
});

test('opening Tools does not launch a second aria2next process for --version', () => {
  const start = main.indexOf('async fn get_tool_statuses');
  const end = main.indexOf('fn optional_path', start);
  assert.ok(start >= 0 && end > start);
  const block = main.slice(start, end);

  assert.doesNotMatch(block, /configured_aria2next\(&settings\)[\s\S]{0,120}\["--version"\]/);
  assert.match(block, /aria2::Runtime::is_running/);
  assert.match(block, /endpoint\.version\(\)\.await/);
  assert.match(block, /found: aria2_path\.is_file\(\)/);
});

test('runtime stop diagnostics identify the daemon process and RPC port', () => {
  assert.match(main, /aria2\.runtime_stopped/);
  assert.match(main, /pid=\{pid\} port=\{port\}/);
});

test('torrent and magnet downloads reuse the same aria2next endpoint/runtime singleton as HTTP', () => {
  assert.match(main, /matches!\(kind, DownloadKind::Torrent \| DownloadKind::Magnet\)/);
  assert.doesNotMatch(main, /rqbit/i);
});

test('the old upstream aria2 static-build sources are fully gone, replaced by AnInsomniacy/aria2-next', () => {
  assert.doesNotMatch(main, /FerroDownload\/aria2-static-builds/);
  assert.doesNotMatch(main, /Kenshin9977\/aria2/);
  assert.doesNotMatch(main, /"aria2c(\.exe)?"/);
  assert.match(main, /AnInsomniacy\/aria2-next/g);
  assert.match(aria2, /aria2-next/);
});

test('aria2-next persistent state stays inside the ADM portable data directory', () => {
  assert.match(aria2, /runtime_root\.join\("state"\)/);
  assert.match(aria2, /--state-dir=\{\}/);
  assert.match(aria2, /state_dir\.display\(\)/);
  assert.match(aria2, /--state-save-interval=30/);
});

test('aria2-next receives global and per-download bandwidth limits, including live changes', () => {
  assert.match(aria2, /"max-overall-download-limit"\.into\(\)/);
  assert.match(aria2, /"max-download-limit"\.into\(\)/);
  assert.match(aria2, /pub async fn set_global_download_limit/);
  assert.match(aria2, /pub async fn set_download_limit/);
  assert.match(main, /async fn set_transfer_limits\(/);
  assert.match(main, /set_global_download_limit\(result\.global_bandwidth_limit\)/);
  assert.match(main, /async fn set_download_bandwidth_limit\(/);
  assert.match(main, /endpoint\.set_download_limit\(&gid, limit\)\.await/);
});

