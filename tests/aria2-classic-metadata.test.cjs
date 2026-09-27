const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const aria2 = fs.readFileSync(path.join(root, "apps/desktop/src-tauri/src/aria2.rs"), "utf8");
const main = fs.readFileSync(path.join(root, "apps/desktop/src-tauri/src/main.rs"), "utf8");
const app = fs.readFileSync(path.join(root, "apps/desktop/ui/app.js"), "utf8");

const nativeStart = aria2.indexOf("pub async fn inspect_magnet_metadata");
const nativeEnd = aria2.indexOf("pub async fn prepare_magnet_download", nativeStart);
const nativeMetadata = aria2.slice(nativeStart, nativeEnd);
const classicStart = aria2.indexOf("pub async fn save_magnet_metadata");
const classicEnd = aria2.indexOf("pub async fn set_download_limit", classicStart);
const classicMetadata = aria2.slice(classicStart, classicEnd);

test("aria2-ultra reads Magnet metadata from the paused RPC task without a torrent file", () => {
  assert.ok(nativeStart >= 0 && nativeEnd > nativeStart);
  assert.match(nativeMetadata, /"pause-metadata"\.into\(\),\s*Value::String\("true"\.into\(\)\)/s);
  assert.match(nativeMetadata, /"bittorrent"/);
  assert.match(nativeMetadata, /fileSelectionState/);
  assert.match(nativeMetadata, /metadata_ready/);
  assert.match(nativeMetadata, /aria2\.getFiles/);
  assert.doesNotMatch(nativeMetadata, /bt-save-metadata|\.torrent/);
});

test("classic aria2 metadata file flow remains only as compatibility fallback", () => {
  assert.ok(classicStart >= 0 && classicEnd > classicStart);
  assert.match(classicMetadata, /"bt-metadata-only"/);
  assert.match(classicMetadata, /"bt-save-metadata"/);
  assert.match(classicMetadata, /\.torrent/);
  assert.match(main, /metadata_native_unsupported/);
  assert.match(main, /fallback=classic/);
});

test("ADM carries the paused Magnet gid from inspection into the queued task", () => {
  assert.match(main, /torrent_gid:\s*Option<String>/);
  assert.match(main, /torrent_metadata_gid:\s*Option<String>/);
  assert.match(main, /prepare_magnet_download/);
  assert.match(main, /item\.torrent_metadata_gid = None/);
  assert.match(app, /pendingTorrentMetadataGid = torrent\.torrentGid \|\| null/);
  assert.match(app, /torrentMetadataGid: pendingTorrentMetadataGid/);
});

test("aria2 raw logs are bounded and rotated instead of growing forever", () => {
  assert.match(aria2, /ARIA2_LOG_MAX_BYTES:\s*u64 = 32 \* 1024 \* 1024/);
  assert.match(aria2, /with_extension\("log\.1"\)/);
  assert.match(aria2, /fs::remove_file\(&previous\)/);
  assert.match(aria2, /fs::rename\(&path, &previous\)/);
  assert.match(aria2, /\.arg\("--log=-"\)/);
  assert.doesNotMatch(aria2, /\.arg\("--log-level=debug"\)/);
});

test("legacy arc4 flags no longer force aria2-next encryption-required mode", () => {
  assert.doesNotMatch(aria2, /--bt-min-crypto-level=arc4/);
  assert.doesNotMatch(aria2, /--bt-require-crypto=false/);
});
