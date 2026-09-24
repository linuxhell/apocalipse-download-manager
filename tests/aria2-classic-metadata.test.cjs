const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const source = fs.readFileSync(
  path.resolve(__dirname, "../apps/desktop/src-tauri/src/aria2.rs"),
  "utf8",
);
const start = source.indexOf("pub async fn preview_magnet_metadata");
const end = source.indexOf("pub async fn set_download_limit", start);
const preview = source.slice(start, end);

test("classic aria2 magnet preview pauses followed payload before file inspection", () => {
  assert.ok(start >= 0 && end > start);
  assert.doesNotMatch(preview, /"bt-metadata-only"\.into\(\), Value::String\("true"\.into\(\)\)/);
  assert.match(preview, /"pause-metadata"\.into\(\), Value::String\("true"\.into\(\)\)/);
  assert.match(preview, /torrent_metadata_complete_without_followed_by/);
  assert.match(preview, /if let Some\(child_gid\) = followed\.as_deref\(\)/);
  assert.match(preview, /self\.pause\(child_gid\)\.await/);
  assert.match(preview, /if let Some\(inspect\) = followed\.as_deref\(\)/);
  assert.doesNotMatch(preview, /followed\.as_deref\(\)\.unwrap_or\(&gid\)/);
});

test("metadata pseudo-file is never exposed as a real torrent choice", () => {
  assert.match(preview, /to_ascii_uppercase\(\)\.starts_with\("\[METADATA\]"\)/);
});

test("metadata diagnostics expose parent, child and RPC state", () => {
  assert.match(source, /pub struct MetadataProbe/);
  assert.match(preview, /child_rpc_error/);
  assert.match(preview, /parent_metadata_file_count/);
  assert.match(preview, /child_real_file_count/);
});
