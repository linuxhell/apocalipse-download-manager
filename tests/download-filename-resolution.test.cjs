const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const main = fs.readFileSync(path.join(root, "apps/desktop/src-tauri/src/main.rs"), "utf8");
const app = fs.readFileSync(path.join(root, "apps/desktop/ui/app.js"), "utf8");

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
  assert.match(main, /async fn probe_content_disposition_filename\(url: &str\) -> Result<String, String>/);
  assert.match(main, /fn parse_content_disposition_filename\(value: &str\) -> Option<String>/);
  // Extended RFC 6266 form (filename*=UTF-8''...) must be preferred over the
  // plain filename= form, same precedence as the extension's own parser.
  assert.match(main, /strip_prefix\("filename\*=UTF-8''"\)/);
  assert.match(main, /CONTENT_DISPOSITION/);

  const bridgeBody = main.slice(
    main.indexOf("fn queue_from_bridge("),
    main.indexOf("fn queue_from_bridge(") + 1800,
  );
  // Only probes when both a file name and a title are missing — never an
  // extra request on the common, already-working path — and only for real
  // http(s) URLs (never local file paths or magnet links).
  assert.match(bridgeBody, /request\s*\n\s*\.file_name/);
  assert.match(bridgeBody, /is_generic_download_name\(value\.trim\(\)\)/);
  assert.match(bridgeBody, /request\s*\n\s*\.title\s*\n\s*\.as_deref\(\)/);
  assert.match(bridgeBody, /request\.url\.starts_with\("http:\/\/"\) \|\| request\.url\.starts_with\("https:\/\/"\)/);
  // Logged unconditionally, before the (blocking) probe call, so a
  // diagnostic export can prove the probe was even attempted — earlier
  // exports showed the whole handoff completing in ~1ms, which is only
  // possible if the gate never actually let the probe run.
  assert.match(bridgeBody, /"handoff\.file_name_probe_started"/);
  assert.match(bridgeBody, /tauri::async_runtime::block_on\(probe_content_disposition_filename\(&request\.url\)\)/);
  assert.match(bridgeBody, /Ok\(name\) => \{/);
  assert.match(bridgeBody, /request\.file_name = Some\(name\);/);
  // A failed probe is logged with a reason instead of silently vanishing,
  // so the next diagnostic export can actually say why it failed (client
  // build error, network failure, no header, unparseable header, ...).
  assert.match(bridgeBody, /Err\(reason\) => \{/);
  assert.match(bridgeBody, /"handoff\.file_name_probe_failed"/);
});

test("the Content-Disposition probe runs on a plain OS thread, not inside the tokio runtime", () => {
  // block_on from within an already-running tokio task can panic or stall a
  // worker thread. queue_from_bridge must only be reachable from contexts
  // that are NOT tokio tasks: the extension bridge and Apocalipse Link
  // servers, both spawned as dedicated std::thread workers.
  assert.match(main, /\.name\("apocalipse-extension-bridge"\.into\(\)\)/);
  assert.match(main, /\.name\("apocalipse-link-client"\.into\(\)\)/);
});

test("the Content-Disposition probe falls back to a ranged GET with a real User-Agent, not just a bare HEAD", () => {
  // Regression: a HEAD-only probe against gopeed.com/api/download?tpl=...
  // came back with no Content-Disposition at all (many "download endpoint"
  // style servers only compute it while actually serving a body, and some
  // block requests without a browser-like User-Agent), so the file kept
  // falling back to the literal name "download" even after the probe was
  // added. A HEAD miss now retries with a minimal ranged GET.
  const probeBody = main.slice(
    main.indexOf("async fn probe_content_disposition_filename("),
    main.indexOf("async fn probe_content_disposition_filename(") + 3400,
  );
  assert.match(probeBody, /\.user_agent\(PROBE_USER_AGENT\)/);
  assert.match(probeBody, /client\s*\n\s*\.get\(url\)/);
  assert.match(probeBody, /header\(reqwest::header::RANGE, "bytes=0-0"\)/);
});

test("a Content-Disposition probe also fires when the extension already sent a generic name, not just when it sent nothing", () => {
  // Regression: even after the probe was added, gopeed.com/api/download
  // still saved as "download" — because the extension's own
  // resolveBrowserDownloadFileName already sends the browser's own generic
  // guess ("download") as request.fileName when it can't do better (no
  // usable page title), so the field was never actually empty. Confirmed
  // live: handoff.desktop_received fired 0ms after the request arrived,
  // meaning the probe's network call never ran at all. The backend must
  // treat a generic name the same as no name, mirroring the extension's own
  // genericDownloadStem list (background.js).
  assert.match(main, /fn is_generic_download_name\(name: &str\) -> bool/);
  const genericFnBody = main.slice(
    main.indexOf("fn is_generic_download_name("),
    main.indexOf("fn is_generic_download_name(") + 700,
  );
  for (const word of ["video", "audio", "media", "midia", "download", "file", "arquivo", "videoplayback"]) {
    assert.match(genericFnBody, new RegExp(`"${word}"`));
  }
});

test("a resolved audio title (not just video) fills the save-dialog file name for bridge downloads", () => {
  // Regression: the save dialog shown for a bridge.download with
  // start_immediately=false has its own, separate title-based file name
  // fallback in app.js — and it only ever applied to pendingMediaKind ===
  // "video", so a SoundCloud audio capture with a perfectly good resolved
  // track title still fell back to the raw (meaningless) CDN URL segment.
  assert.doesNotMatch(app, /pendingMediaKind === "video" && titleName/);
  assert.match(app, /\(pendingMediaKind === "video" \|\| pendingMediaKind === "audio"\) && titleName/);
  assert.match(app, /const titleExtension = pendingMediaKind === "audio" \? "m4a" : "mp4";/);
});
