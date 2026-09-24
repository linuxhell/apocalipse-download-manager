const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const main = fs.readFileSync(path.join(root, "apps/desktop/src-tauri/src/main.rs"), "utf8");

test("a resolved page/media title is used as the file name before falling back to the raw URL", () => {
  // Regression: enqueue_download_impl used to decide the file name purely
  // from the last URL path segment (suggested_name), ignoring a perfectly
  // good title the caller already resolved (e.g. a SoundCloud track name).
  // For a signed CDN stream URL like
  // playback.media-streaming.soundcloud.cloud/...?token=..., that path
  // segment is meaningless, so the file was silently named after the URL
  // instead of the known track title.
  assert.match(main, /fn sanitize_title_for_filename\(title: &str\) -> String/);
  const enqueueBody = main.slice(
    main.indexOf(") -> Result<DownloadTask, String> {", main.indexOf("fn enqueue_download_impl(")),
    main.indexOf("fn enqueue_download_impl(") + 4000,
  );
  assert.match(enqueueBody, /let title_based_name = context/);
  assert.match(enqueueBody, /\.map\(sanitize_title_for_filename\)/);
  assert.match(enqueueBody, /let proposed = file_name\s*\n\s*\.or\(title_based_name\)/);
});

test("bridge downloads with neither a file name nor a title probe Content-Disposition before falling back to a generic name", () => {
  // Regression: a direct bridge.download for a URL like
  // gopeed.com/api/download?tpl=... (no useful path, no query filename) was
  // always saved as the literal file "download" with no extension, because
  // nothing ever inspected the server's own Content-Disposition header.
  assert.match(main, /async fn probe_content_disposition_filename\(url: &str\) -> Option<String>/);
  assert.match(main, /fn parse_content_disposition_filename\(value: &str\) -> Option<String>/);
  // Extended RFC 6266 form (filename*=UTF-8''...) must be preferred over the
  // plain filename= form, same precedence as the extension's own parser.
  assert.match(main, /strip_prefix\("filename\*=UTF-8''"\)/);
  assert.match(main, /CONTENT_DISPOSITION/);

  const bridgeBody = main.slice(
    main.indexOf("fn queue_from_bridge("),
    main.indexOf("fn queue_from_bridge(") + 1200,
  );
  // Only probes when both a file name and a title are missing — never an
  // extra request on the common, already-working path — and only for real
  // http(s) URLs (never local file paths or magnet links).
  assert.match(bridgeBody, /request\s*\n\s*\.file_name/);
  assert.match(bridgeBody, /request\.title\.as_deref\(\)\.is_none_or/);
  assert.match(bridgeBody, /request\.url\.starts_with\("http:\/\/"\) \|\| request\.url\.starts_with\("https:\/\/"\)/);
  assert.match(bridgeBody, /tauri::async_runtime::block_on\(probe_content_disposition_filename\(&request\.url\)\)/);
  assert.match(bridgeBody, /request\.file_name = Some\(name\);/);
});

test("the Content-Disposition probe runs on a plain OS thread, not inside the tokio runtime", () => {
  // block_on from within an already-running tokio task can panic or stall a
  // worker thread. queue_from_bridge must only be reachable from contexts
  // that are NOT tokio tasks: the extension bridge and Apocalipse Link
  // servers, both spawned as dedicated std::thread workers.
  assert.match(main, /\.name\("apocalipse-extension-bridge"\.into\(\)\)/);
  assert.match(main, /\.name\("apocalipse-link-client"\.into\(\)\)/);
});
