const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const read = (...parts) => fs.readFileSync(path.join(__dirname, "..", ...parts), "utf8");
const background = read("browser-extension", "background.js");
const content = read("browser-extension", "content.js");
const rust = read("apps", "desktop", "src-tauri", "src", "main.rs");

test("a failed browser upload is aborted so the desktop task does not stay Downloading", () => {
  // Desktop route and its handler.
  assert.match(rust, /POST \/v1\/blob\/abort /);
  assert.match(rust, /fn abort_blob_upload/);
  assert.match(rust, /fn purge_stale_blob_uploads/);
  // The prehook uploader aborts in its catch path.
  assert.match(background, /bridgePost\("\/v1\/blob\/abort", \{ uploadId \}\)/);
  // The content-script uploader routes the abort through the background worker.
  assert.match(background, /APOCALIPSE_BLOB_ABORT[\s\S]*\/v1\/blob\/abort/);
  assert.match(content, /type: "APOCALIPSE_BLOB_ABORT"/);
});

test("the extension no longer polls a pause flag the desktop never sends", () => {
  assert.doesNotMatch(background, /waitWhilePaused/);
  assert.doesNotMatch(rust, /"paused"\s*:[^\n]*blob/i);
});
