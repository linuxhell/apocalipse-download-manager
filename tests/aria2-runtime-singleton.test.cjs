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

test('aria2-next honors HTTP and BitTorrent proxy settings without leaking unsupported routes', () => {
  assert.match(aria2, /"all-proxy"\.into\(\)/);
  assert.match(aria2, /"all-proxy-user"\.into\(\)/);
  assert.match(aria2, /"all-proxy-passwd"\.into\(\)/);
  assert.match(aria2, /"bt-proxy"\.into\(\)/);
  assert.match(main, /fn aria2_bt_proxy_url\(/);
  assert.match(main, /"socks5h"[\s\S]*set_scheme\("socks5"\)/);
  assert.match(main, /aria2_proxy_failed/);
  assert.match(main, /ed2k_proxy_unsupported/);
});

test('custom DNS and proxy schemes unsupported by aria2 HTTP route fall back to the native network engine', () => {
  assert.match(main, /let aria2_http_network_compatible\s*=\s*!limits\.dns_enabled/);
  assert.match(main, /aria2_http_proxy_url\(&limits\)\.is_some\(\)/);
  assert.match(main, /\|\| !aria2_http_network_compatible/);
});



test('aria2-next GIDs survive ADM restart and stale restored GIDs fall back to recreation', () => {
  const model = readFileSync(join(root, 'crates/apocalipse-core/src/model.rs'), 'utf8');
  assert.match(model, /pub aria2_gid: Option<String>/);
  assert.match(main, /fn restored_aria2_task_map\(queue: &\[DownloadTask\]\)/);
  assert.match(main, /aria2_tasks: Mutex::new\(initial_aria2_tasks\)/);
  assert.match(main, /aria2\.restored_gid_stale/);
  assert.match(main, /item\.aria2_gid = Some\(gid\.clone\(\)\)/);
  assert.match(main, /item\.aria2_gid = None/);
});

test('transient queue states become paused after process restart instead of appearing active without a worker', () => {
  const start = main.indexOf('fn load_queue(path: &Path)');
  const end = main.indexOf('\nfn copy_directory_if_missing', start);
  assert.ok(start >= 0 && end > start);
  const block = main.slice(start, end);
  assert.match(block, /DownloadState::Downloading \| DownloadState::Inspecting \| DownloadState::Verifying/);
  assert.match(block, /task\.state = DownloadState::Paused/);
  assert.match(block, /task\.download_speed = Some\(0\)/);
  assert.match(block, /fs::write\(path, data\)/);
});

test('network runtime restart preserves task to GID mappings for aria2-next session recovery', () => {
  const start = main.indexOf('fn reconnect_active_downloads_after_network_change');
  const end = main.indexOf('\nfn run_network_change_monitor', start);
  assert.ok(start >= 0 && end > start);
  const block = main.slice(start, end);
  assert.match(block, /stop_aria2_runtime\(&state\)/);
  assert.doesNotMatch(block, /aria2_tasks\.clear\(\)/);
});

test('redownload preserves torrent selection and transfer options while allocating a fresh GID', () => {
  const start = main.indexOf('fn redownload_downloads(');
  const end = main.indexOf('\n\#\[tauri::command\]\nfn get_clipboard_monitor', start);
  assert.ok(start >= 0 && end > start);
  const block = main.slice(start, end);
  for (const field of [
    'torrent_selection',
    'companion_audio_url',
    'mirrors',
    'priority',
    'bandwidth_limit',
    'connections_override',
    'expected_size',
    'sha256',
    'auto_extract',
  ]) {
    assert.match(block, new RegExp(`task\\.${field} = original\\.${field}`));
  }
  assert.doesNotMatch(block, /task\.aria2_gid = original\.aria2_gid/);
});

test('invisible restored Magnet previews are pruned before they can block the same info-hash', () => {
  assert.match(aria2, /pub async fn prune_orphan_bittorrent_transfers/);
  assert.match(aria2, /"aria2\.tellActive"/);
  assert.match(aria2, /"aria2\.tellWaiting"/);
  assert.match(aria2, /"infoHash"/);
  assert.match(aria2, /"aria2\.forceRemove"/);
  assert.match(aria2, /matches!\(status\.as_str\(\), "active" \| "waiting" \| "paused"\)/);
  assert.match(main, /fn protected_aria2_gids\(/);
  assert.match(main, /async fn prune_orphan_aria2_torrents\(/);
  assert.match(main, /aria2\.orphan_bittorrent_removed/);
  assert.match(main, /reason=\{reason\}/);
  assert.match(main, /prune_orphan_aria2_torrents\(state, &endpoint, "runtime_start"\)\.await/);
});

test('metadata Analyze reconciles orphan engine torrents before adding a temporary Magnet preview', () => {
  const start = main.indexOf('async fn inspect_torrent_metadata(');
  const end = main.indexOf('\nasync fn fetch_torrent_file_bytes', start);
  assert.ok(start >= 0 && end > start);
  const block = main.slice(start, end);
  const endpoint = block.indexOf('aria2_endpoint(&state, true).await?');
  const prune = block.indexOf('prune_orphan_aria2_torrents(&state, &endpoint, "metadata_preview").await?');
  const preview = block.indexOf('.preview_magnet_metadata(');
  assert.ok(endpoint >= 0 && prune > endpoint && preview > prune);
});

test('Magnet analysis gives restored aria2-next session entries one bounded attach grace before retry', () => {
  const start = main.indexOf('async fn inspect_torrent_metadata(');
  const end = main.indexOf('\nasync fn fetch_torrent_file_bytes', start);
  assert.ok(start >= 0 && end > start);
  const block = main.slice(start, end);
  assert.match(block, /session_may_restore/);
  assert.match(block, /metadata\.len\(\) > 0/);
  assert.match(block, /Duration::from_millis\(650\)/);
  assert.match(block, /metadata_preview_restore_grace/);
});



test('aria2-next uses a dynamically selected BitTorrent listen port instead of fixed 6881', () => {
  assert.match(aria2, /fn reserve_bittorrent_port\(\)/);
  assert.match(aria2, /UdpSocket::bind\(\("0\.0\.0\.0", port\)\)/);
  assert.match(aria2, /--listen-port=\{bt_port\}/);
  assert.match(aria2, /aria2_bt_listen_failed/);
  assert.doesNotMatch(aria2, /--listen-port=6881/);
  assert.match(main, /bt_listen_port=\{bt_port\}/);
});

test('discarded Magnet metadata previews wait for removal and collect only orphan fastresume state', () => {
  assert.match(aria2, /async fn cleanup_temporary_magnet_preview/);
  assert.match(aria2, /"aria2\.forceRemove"/);
  assert.match(aria2, /"aria2\.removeDownloadResult"/);
  assert.match(aria2, /"aria2\.saveSession"/);
  assert.match(aria2, /live_bittorrent_info_hash/);
  assert.match(aria2, /state_dir\.join\("bittorrent"\)\.join\("torrents"\)/);
  assert.match(aria2, /\.fastresume/);
  assert.match(aria2, /magnet_info_hash\(magnet\)/);
});

test('aria2-next HTTP downloads use native stream-max-connections without legacy split aliases', () => {
  const start = aria2.indexOf('pub async fn add_download(');
  const end = aria2.indexOf('\n    pub async fn status(', start);
  assert.ok(start >= 0 && end > start);
  const block = aria2.slice(start, end);
  assert.match(block, /"stream-max-connections"/);
  assert.doesNotMatch(block, /"max-connection-per-server"/);
  assert.doesNotMatch(block, /"min-split-size"/);
  assert.doesNotMatch(block, /options\.insert\("split"/);
  assert.match(main, /"streamMaxConnections": connections/);
});

test('aria2 startup diagnostics expose first payload and worker ramp milestones', () => {
  for (const event of [
    'aria2.first_payload_byte',
    'aria2.workers_2',
    'aria2.workers_4',
    'aria2.workers_8',
    'aria2.workers_16',
  ]) {
    assert.match(main, new RegExp(event.replace('.', '\\.')));
  }
});


test('aria2-next working directory is isolated under the portable runtime tree', () => {
  assert.match(aria2, /command\.current_dir\(runtime_root\)/);
  assert.match(aria2, /--state-dir=\{\}/);
  assert.match(aria2, /state_dir\.display\(\)/);
});

test('new BitTorrent tasks warm preview sequentially and release back to rarest-first after 64 MiB', () => {
  assert.match(aria2, /"force-sequential"\.into\(\),\s*Value::String\("true"\.into\(\)\)/);
  assert.match(aria2, /pub async fn set_bittorrent_sequential/);
  assert.match(aria2, /"aria2\.changeOption"/);
  assert.match(main, /preview_warmup_bytes = 64_u64 \* 1024 \* 1024/);
  assert.match(main, /set_bittorrent_sequential\(&gid, false\)\.await/);
  assert.match(main, /torrent\.preview_window_ready/);
});

test('aria2 startup polling is temporarily accelerated for direct-download UI responsiveness', () => {
  assert.match(main, /tokio::time::interval\(Duration::from_millis\(100\)\)/);
  assert.match(main, /transfer_started_at\.elapsed\(\) >= Duration::from_secs\(2\)/);
  assert.match(main, /tokio::time::interval\(Duration::from_millis\(350\)\)/);
});
