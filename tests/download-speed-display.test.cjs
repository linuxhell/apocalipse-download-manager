const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");

const root = path.resolve(__dirname, "..");
const ui = fs.readFileSync(path.join(root, "apps/desktop/ui/app.js"), "utf8");
const desktop = fs.readFileSync(
  path.join(root, "apps/desktop/src-tauri/src/main.rs"),
  "utf8",
);
const aria2 = fs.readFileSync(
  path.join(root, "apps/desktop/src-tauri/src/aria2.rs"),
  "utf8",
);
const cargo = fs.readFileSync(path.join(root, "Cargo.toml"), "utf8");
const model = fs.readFileSync(
  path.join(root, "crates/apocalipse-core/src/model.rs"),
  "utf8",
);
const tauri = JSON.parse(
  fs.readFileSync(path.join(root, "apps/desktop/src-tauri/tauri.conf.json"), "utf8"),
);
const desktopHtml = fs.readFileSync(path.join(root, "apps/desktop/ui/index.html"), "utf8");
const desktopCss = fs.readFileSync(path.join(root, "apps/desktop/ui/styles.css"), "utf8");

test("native HTTP speed uses a multi-second EWMA instead of noisy quarter-second jumps", () => {
  assert.match(ui, /const SPEED_EWMA_SECONDS = 2\.0/);
  assert.match(ui, /1 - Math\.exp\(-elapsed \/ SPEED_EWMA_SECONDS\)/);
  assert.match(desktop, /smoothedBytesPerSecond/);
  assert.match(desktop, /bytes_per_second as f64 >= display_rate_ewma/);
  assert.match(desktop, /0\.80/);
  assert.match(desktop, /0\.35/);
  assert.match(desktop, /task\.download_speed = Some\(smoothed_bytes_per_second\)/);
});

test("list-and-files removal passes a fixed true flag instead of a dataset-derived mode", () => {
  assert.match(desktopHtml, /id="clear-list-and-files"[^>]*data-clear-mode="files"/);
  assert.match(ui, /removeSelectedDownloads\(event\.currentTarget, true\)/);
  assert.match(desktop, /"task\.removal_disk_cleanup"/);
  assert.match(desktop, /chunkArtifactsRemaining/);
});

test("active downloads keep the engine-reported speed visible", () => {
  assert.match(
    ui,
    /const externalSpeed = active \? Number\(task\.download_speed\) \|\| 0 : 0;/,
  );
  assert.doesNotMatch(
    ui,
    /const externalSpeed = now - changedAt < 2000/,
  );
});

test("yt-dlp publishes bytes, total, percent and speed to the ADM interface", () => {
  assert.match(desktop, /download:ADM_PROGRESS\|%\(progress\.downloaded_bytes\)s\|%\(progress\.total_bytes\)s\|%\(progress\.total_bytes_estimate\)s\|%\(progress\.speed\)s/);
  assert.match(desktop, /fn parse_yt_dlp_progress/);
  assert.match(desktop, /task\.download_speed = Some\(speed\)/);
  assert.match(desktop, /task\.received = received/);
  assert.match(desktop, /task\.total = total/);
  assert.match(desktop, /task_connections\.max\(16\)/);
});

test("YouTube live downloads start from the beginning inside an isolated workspace", () => {
  assert.match(desktop, /status == "is_live"/);
  assert.match(desktop, /task\.is_live = context\.is_live/);
  assert.match(desktop, /command\.args\(\["--live-from-start", "--hls-use-mpegts"\]\)/);
  assert.match(desktop, /let media_work_directory = \(kind == DownloadKind::MediaPage\)/);
  assert.match(desktop, /parent\.join\("media-work"\)\.join\(task\.id\.to_string\(\)\)/);
  assert.match(desktop, /remove_path_with_retry\(&workspace, true\)\.await/);
  assert.match(ui, /pendingIsLive = media\.isLive === true/);
  assert.match(ui, /isLive: pendingIsLive/);
});

test("split yt-dlp progress lines remain buffered until speed can be parsed", () => {
  assert.match(desktop, /let mut progress_buffer = String::new\(\)/);
  assert.match(desktop, /progress_buffer\.push_str\(&text\)/);
  assert.match(desktop, /parse_yt_dlp_progress\(&progress_buffer\)/);
});

test("removing a media task terminates yt-dlp and every child process", () => {
  assert.match(desktop, /terminate_process_tree\(&mut child\)\.await/);
  assert.match(desktop, /taskkill\.exe/);
  assert.match(desktop, /"\/PID", &pid\.to_string\(\), "\/T", "\/F"/);
  assert.match(desktop, /process_group\(0\)/);
  assert.match(desktop, /format!\("-\{pid\}"\)/);
  assert.match(desktop, /child\.wait\(\)\.await/);
});

test("Facebook composite links stay canonicalized before transfer-engine dispatch", () => {
  assert.match(desktop, /fn canonical_facebook_video_url/);
  assert.match(desktop, /Some\(format!\("https:\/\/www\.facebook\.com\/watch\/\?v=\{video_id\}"\)\)/);
  assert.match(desktop, /run_aria2_download/);
  assert.match(desktop, /requires_native_http_compatibility/);
});

test("streamed browser recordings publish a global core speed", () => {
  assert.match(desktop, /speed_sample_at: Instant/);
  assert.match(desktop, /task\.download_speed = Some\(download_speed\)/);
  assert.match(desktop, /instantaneous as f64 \* 0\.65/);
  assert.match(desktop, /"blob-upload-progress"/);
  assert.match(ui, /listen\?\.\("blob-upload-progress"/);
  assert.match(ui, /renderDownloads\(true\)/);
});

test("quiet clipboard polling does not flood diagnostics", () => {
  assert.match(ui, /const quiet = new Set\(\["list_downloads", "read_general_log", "get_bridge_pairing", "read_clipboard_link"/);
  assert.match(ui, /if \(!quiet\.has\(command\) && command !== "record_ui_diagnostic" && command !== "record_diagnostics_ui"\)/);
  assert.match(
    desktop,
    /let Ok\(value\) = app\.clipboard\(\)\.read_text\(\) else \{\s*return Ok\(None\);/,
  );
});

test("clipboard suppression is checked again after an in-flight OS read", () => {
  const clipboardReader = desktop.match(/fn read_clipboard_link[\s\S]*?\n}\n\n#\[tauri::command\]/)?.[0] || "";
  assert.match(clipboardReader, /let clipboard_is_suppressed =/);
  assert.equal((clipboardReader.match(/clipboard_is_suppressed\(\)\?/g) || []).length, 2);
  assert.ok(clipboardReader.lastIndexOf("clipboard_is_suppressed()?") > clipboardReader.indexOf("read_text()"));
  assert.match(clipboardReader, /suppressed_value\.as_deref\(\) == Some\(value\)/);
  assert.match(desktop, /clipboard_suppressed_value: Mutex<Option<String>>/);
});

test("desktop package and interface versions cannot diverge", () => {
  const packageVersion = cargo.match(/\[workspace\.package\][\s\S]*?version = "([^"]+)"/)?.[1];
  assert.ok(packageVersion, "workspace package version is missing");
  assert.equal(tauri.version, packageVersion);
  assert.match(ui, /invoke\("get_app_version"\)/);
  assert.ok(ui.indexOf("const invoke =") < ui.indexOf('invoke("set_application_theme"'), "theme sync must run only after the desktop bridge is initialized");
});

test("ten readable light themes are available", () => {
  for (const theme of ["linen", "sky", "blossom", "sage", "sand", "lilac", "mist", "citrus", "coral", "frost"]) {
    assert.match(desktopHtml, new RegExp(`value="${theme}"`));
    assert.match(desktopCss, new RegExp(`data-theme="${theme}"`));
  }
  assert.match(desktopCss, /color-scheme:light/);
  assert.match(desktopCss, /--panel:#fff/);
  assert.match(desktopCss, /\.logs-panel,\.link-panel>div,\.diagnostics-panel/);
  assert.match(desktop, /"theme": theme/);
  assert.match(ui, /invoke\("set_application_theme"/);
  assert.match(desktopCss, /\.remove-options button:hover[\s\S]*color-mix\(in srgb, var\(--accent\) 14%, var\(--surface\)\)/);
  assert.match(desktopCss, /\.remove-options button\.danger:hover[\s\S]*color-mix\(in srgb, #ff5364 12%, var\(--surface\)\)/);
});

test("HLS uses the selected destination for process and temporary files", () => {
  assert.match(desktop, /command\.current_dir\(directory\)/);
  assert.match(desktop, /\.arg\("--tmp-dir"\)\s*\.arg\(directory\)/);
});

test("failed thumbnails restore the compact icon without covering progress", () => {
  assert.match(ui, /icon\.classList\.remove\("has-thumbnail"\)/);
  assert.doesNotMatch(ui, /else if \(task\.thumbnail\)\s*\{\s*icon\.classList\.add\("has-thumbnail"\)/);
  assert.match(desktopCss, /\.download-icon\s*\{[\s\S]*width:\s*32px;[\s\S]*height:\s*32px;/);
});

test("task state and action controls use the active theme instead of dark constants", () => {
  assert.match(desktopCss, /\.download-state\s*\{[\s\S]*background:\s*color-mix\(in srgb, var\(--accent\) 10%, var\(--surface\)\)/);
  assert.match(desktopCss, /\.task-action\s*\{[\s\S]*background:\s*var\(--surface-2\)/);
});

test("automatic thread selection stays readable without clipping", () => {
  assert.match(desktopCss, /\.task-connections output \{[^}]*min-width:96px/);
  assert.match(desktopCss, /\.task-connections output \{[^}]*white-space:nowrap/);
  assert.match(desktopCss, /\.task-connections output \{[^}]*color:var\(--text\)/);
});

test("bulk list actions are translated theme-aware buttons", () => {
  assert.match(desktopHtml, /id="import-list" class="list-action-button"[\s\S]*data-i18n="importList"/);
  assert.match(desktopHtml, /class="list-action-button list-select-button"[\s\S]*data-i18n="selectAll"/);
  assert.match(desktopHtml, /id="manage-list"[\s\S]*class="list-action-button list-action-danger"[\s\S]*data-i18n="removeSelected"/);
  assert.match(desktopHtml, /id="redownload-selected"[\s\S]*class="list-action-button"[\s\S]*data-i18n="redownloadSelected"/);
  assert.match(desktopCss, /\.list-action-button \{[\s\S]*var\(--surface-2\)/);
  assert.match(desktopCss, /\.list-action-danger:hover:not\(:disabled\)/);
});

test("every interface button has pointer and keyboard click feedback", () => {
  const feedback = fs.readFileSync(path.join(root, "apps/desktop/ui/button-feedback.js"), "utf8");
  assert.match(feedback, /document\.addEventListener\("pointerdown"/);
  assert.match(feedback, /event\.key === "Enter" \|\| event\.key === " "/);
  assert.match(feedback, /button-click-feedback/);
  assert.match(desktopCss, /@keyframes apocalipse-button-press/);
  assert.match(desktopCss, /@keyframes apocalipse-button-wave/);
  assert.match(desktopHtml, /<script src="button-feedback\.js"><\/script>/);
});

test("extension popup and video overlays provide visible click feedback", () => {
  const popup = fs.readFileSync(path.join(root, "browser-extension/popup.js"), "utf8");
  const popupCss = fs.readFileSync(path.join(root, "browser-extension/popup.css"), "utf8");
  const content = fs.readFileSync(path.join(root, "browser-extension/content.js"), "utf8");

  assert.match(popup, /addEventListener\("pointerdown"/);
  assert.match(popup, /event\.key !== "Enter" && event\.key !== " "/);
  assert.match(popup, /button\.disabled/);
  assert.match(popupCss, /apocalipse-extension-press/);
  assert.match(popupCss, /prefers-reduced-motion/);
  assert.match(content, /restartOverlayButtonFeedback/);
  assert.match(content, /apocalipse-overlay-press/);
  assert.match(content, /prefers-reduced-motion:reduce/);
});

test("stale Chrome content scripts stop quietly after an extension reload", () => {
  const content = fs.readFileSync(path.join(root, "browser-extension/content.js"), "utf8");
  assert.match(content, /const extensionContextActive =/);
  assert.match(content, /const sendRuntimeMessageQuietly =/);
  assert.match(content, /if \(!extensionContextActive\(\)\) return clearInterval\(overlayRefreshTimer\)/);
  assert.match(content, /if \(!extensionContextActive\(\)\) return clearInterval\(appearanceSyncTimer\)/);
  assert.match(content, /try \{ if \(button\.isConnected\) button\.classList\.remove/);
  assert.match(content, /if \(!extensionContextActive\(\)\) return;[\s\S]*window\.postMessage/);
});

test("protected thumbnails are cached for the desktop handoff", () => {
  const worker = fs.readFileSync(path.join(root, "browser-extension/background.js"), "utf8");
  const content = fs.readFileSync(path.join(root, "browser-extension/content.js"), "utf8");
  assert.match(worker, /const fetchThumbnailDataUrl = async/);
  assert.match(worker, /const thumbnail = await portableThumbnail\(item\.thumbnail\)/);
  assert.match(worker, /thumbnailDataCache/);
  assert.match(desktop, /value\.len\(\) <= 600_000/);
  assert.match(worker, /APOCALIPSE_CAPTURE_VISIBLE_THUMBNAIL/);
  assert.match(worker, /chrome\.tabs\.captureVisibleTab/);
  assert.match(worker, /new OffscreenCanvas/);
  assert.match(content, /const captureThumbnailFor = async/);
});

test("thumbnail diagnostics distinguish DOM, captured, fetched and fallback sources", () => {
  const popup = fs.readFileSync(path.join(root, "browser-extension/popup.js"), "utf8");
  assert.match(popup, /thumbnail\.source_selected/);
  assert.match(popup, /thumbnail\.fetch_succeeded/);
  assert.match(popup, /thumbnail\.fallback/);
  assert.match(popup, /thumbnails:\s*matches\.filter/);
});

test("feed thumbnails come from the exact visual player region before page metadata", () => {
  const content = fs.readFileSync(path.join(root, "browser-extension/content.js"), "utf8");
  assert.match(content, /const visualThumbnailFor = \(element\) =>/);
  assert.match(content, /if \(videos\.length > 1\) break/);
  assert.match(content, /if \(overlap < 0\.45\) continue/);
  assert.match(content, /element\?\.getAttribute\?\.\("poster"\),\s*visualThumbnailFor\(element\)/);
  assert.match(content, /const socialCardUrl = \(value\) =>/);
  assert.match(content, /const cardThumbnailFor = \(anchor\) =>/);
  assert.match(content, /Social feeds commonly expose the permalink and cover image before they/);
  assert.match(content, /const playerContext = \(element\) =>/);
  assert.match(content, /playerBound:\s*true/);
  assert.match(content, /visualOnly:\s*true/);
});

test("Facebook photo permalinks never leak into the Videos tab", () => {
  const content = fs.readFileSync(path.join(root, "browser-extension/content.js"), "utf8");
  assert.match(content, /\/\\\/\(\?:photo\|photos\)\(\?:\\\.php\|\\\/\|\$\)\/i/);
  assert.match(content, /Facebook uses `fbid` for both photos and videos/);
});

test("a closed browser becomes disconnected after the initial extension wait", () => {
  assert.match(ui, /bridgeDisconnected:\s*"Extension disconnected"/);
  assert.match(ui, /bridgeDisconnected:\s*"Extensão desconectada"/);
  assert.match(ui, /bridgeDisconnected:\s*"扩展已断开连接"/);
  assert.match(ui, /Date\.now\(\) - bridgeStatusStartedAt < 6000/);
});

test("download destinations and task copy follow the active theme palette", () => {
  assert.match(desktopCss, /\.destination-select\s*\{[\s\S]*color:\s*var\(--text\)/);
  assert.match(desktopCss, /\.destination-row\.unavailable \.destination-select span\s*\{\s*color:\s*var\(--muted\)/);
  assert.match(desktopCss, /\.download-info strong\s*\{[\s\S]*color:\s*color-mix/);
});

test("paused HLS cleanup includes its isolated segment workspace", () => {
  assert.match(desktop, /fn hls_workspace_path\(task: &DownloadTask\)/);
  assert.match(desktop, /torrent_root \|\| hls_workspace/);
});

test("automatic mirrors are server-advertised, identity-checked and latency-ranked", () => {
  const core = fs.readFileSync(path.join(root, "crates/apocalipse-core/src/download.rs"), "utf8");
  assert.match(core, /pub async fn verified_sources/);
  assert.match(core, /rel=\\"duplicate\\"/);
  assert.match(core, /same_download_identity/);
  assert.match(core, /verified\.sort_by_key\(\|\(_, elapsed\)\| \*elapsed\)/);
  assert.match(desktop, /engine\.verified_sources\(&request, &mirrors\)\.await/);
});


test("torrent and magnet downloads go through aria2 with BitTorrent extensions enabled, no rqbit anywhere", () => {
  assert.doesNotMatch(desktop, /rqbit/i);
  assert.doesNotMatch(aria2, /rqbit/i);
  assert.match(aria2, /pub async fn add_bittorrent\(/);
  assert.match(aria2, /--enable-dht=true/);
  assert.match(aria2, /--enable-peer-exchange=true/);
  assert.match(aria2, /--bt-enable-lpd=true/);
  assert.match(aria2, /--bt-encryption=preferred/);
  assert.match(aria2, /--follow-torrent=true/);
  assert.match(desktop, /matches!\(kind, DownloadKind::Torrent \| DownloadKind::Magnet\)/);
});

test("torrent/magnet downloads land inside their own folder, not loose files, and get cleaned up recursively", () => {
  assert.match(desktop, /\.add_bittorrent\(/);
  assert.match(desktop, /bytes\.as_deref\(\),\s*\n\s*&task\.destination,\s*\n\s*&task\.torrent_selection,/);
  assert.match(desktop, /torrent_root = matches!/);
  assert.match(desktop, /&& path == task\.destination/);
});

test("torrent/magnet previews prioritize the first and last pieces of every file", () => {
  assert.match(aria2, /"bt-first-last-piece-first"\.into\(\),\s*Value::String\("true"\.into\(\)\)/);
});


test("resolving a magnet's metadata uses aria2-next same-GID pause and a long metadata timeout", () => {
  assert.match(aria2, /pub async fn preview_magnet_metadata\(/);
  assert.match(aria2, /"pause-metadata"\.into\(\), Value::String\("true"\.into\(\)\)/);
  assert.doesNotMatch(aria2, /options\.insert\("bt-metadata-only"/);
  assert.match(aria2, /fileSelectionState/);
  assert.match(aria2, /Some\("awaiting"\)/);
  assert.match(aria2, /Duration::from_secs\(150\)/);
  const app = fs.readFileSync(path.join(root, "apps/desktop/ui/app.js"), "utf8");
  assert.match(app, /torrentMetadataTimeout:/);
  assert.ok(app.includes('t("torrentMetadataTimeout")'));
});


test("magnet metadata preview inspects real files on aria2-next's paused same GID", () => {
  const start = aria2.indexOf("pub async fn preview_magnet_metadata(");
  const end = aria2.indexOf("\n    }", start);
  assert.ok(start >= 0 && end > start);
  const block = aria2.slice(start, end);
  assert.match(block, /"pause-metadata"\.into\(\), Value::String\("true"\.into\(\)\)/);
  assert.doesNotMatch(block, /"bt-metadata-only"\.into\(\)/);
  assert.match(block, /file_selection_state == Some\("awaiting"\)/);
  assert.match(block, /has_bittorrent_info && has_real_files/);
  assert.match(block, /break Ok\(value\)/);
});

test("a magnet metadata timeout reports peers/seeders seen so far, and aria2 logs DHT/tracker activity", () => {
  // Regression: a bare "timeout" told nobody whether aria2 ever reached the
  // swarm at all (network/firewall blocking outbound BitTorrent) or reached
  // peers but got stuck resolving metadata (a real bug) - both looked
  // identical to the user and to us reading a diagnostic bundle.
  assert.match(aria2, /mut on_progress: impl FnMut\(u64, i64, i64, i64, i64, Option<&str>\)/);
  assert.match(aria2, /"connections"/);
  assert.match(aria2, /"numSeeders"/);
  assert.match(aria2, /aria2_metadata_timeout:connections=\{peak_connections\}:seeders=\{peak_seeders\}/);
  assert.match(aria2, /--log-level=debug/);
  const main = fs.readFileSync(path.join(root, "apps/desktop/src-tauri/src/main.rs"), "utf8");
  assert.match(main, /preview_magnet_metadata\(\s*\n\s*&source,\s*\n\s*&workspace,/);
  assert.match(main, /aria2\.metadata_preview_progress/);
  const app = fs.readFileSync(path.join(root, "apps/desktop/ui/app.js"), "utf8");
  assert.match(app, /torrentMetadataNoPeers:/);
  assert.match(app, /connections=0:seeders=0/);
});


test("magnet metadata preview prefers aria2-next same-GID files and keeps followedBy only as legacy fallback", () => {
  const start = aria2.indexOf("pub async fn preview_magnet_metadata(");
  const end = aria2.indexOf("\n    }", start);
  assert.ok(start >= 0 && end > start);
  const block = aria2.slice(start, end);
  assert.match(block, /file_selection_state == Some\("awaiting"\)/);
  assert.match(block, /has_bittorrent_info && has_real_files/);
  assert.match(block, /Compatibility fallback for legacy aria2 behavior/);
  assert.match(block, /"followedBy"/);
  assert.match(block, /removeDownloadResult/);
  const main = fs.readFileSync(path.join(root, "apps/desktop/src-tauri/src/main.rs"), "utf8");
  assert.match(main, /aria2\.metadata_preview_followed_unexpectedly/);
});


test("real magnet downloads commit select-file then resume the same aria2-next GID", () => {
  assert.match(aria2, /pub file_selection_state: Option<String>/);
  assert.match(aria2, /pub async fn set_selected_files/);
  assert.match(aria2, /"aria2\.changeOption"/);
  assert.match(desktop, /status\.file_selection_state\.as_deref\(\)/);
  assert.match(desktop, /endpoint\.set_selected_files\(&gid, &selected\)\.await/);
  assert.match(desktop, /endpoint\.resume\(&gid\)\.await/);
  assert.match(desktop, /aria2\.metadata_selection_applied/);
  // Legacy followedBy stays only as a compatibility fallback, never as the
  // primary aria2-next Magnet flow.
  assert.match(desktop, /Compatibility fallback for legacy aria2-style followedBy/);
});

test("torrent helper used by paused task actions is defined outside visibleDownloads", () => {
  // Regression from a live Windows diagnostic: pausing a torrent succeeded in
  // Rust and the queue still contained the paused task, but renderDownloads
  // cleared the list and then threw ReferenceError because isTorrent existed
  // only as a local inside visibleDownloads.
  const helper = ui.indexOf("const isTorrent = (task) =>");
  const visible = ui.indexOf("function visibleDownloads()");
  const render = ui.indexOf("function renderDownloads(");
  assert.ok(helper >= 0 && helper < visible && visible < render);
  const visibleEnd = ui.indexOf("\n}", visible);
  assert.ok(visibleEnd > visible);
  assert.doesNotMatch(ui.slice(visible, visibleEnd), /const isTorrent =/);
  assert.match(ui, /\(key === "paused" \|\| key === "failed"\) && !isTorrent\(task\)/);
});

test("a paused or failed download can be relocated to a partial file moved to another folder/drive", () => {
  const start = desktop.indexOf("fn relocate_download(");
  const end = desktop.indexOf("\n}", start);
  assert.ok(start >= 0 && end > start);
  const block = desktop.slice(start, end);
  assert.match(block, /DownloadState::Paused \| DownloadState::Failed \{ \.\. \}/);
  assert.match(block, /torrent_relocate_unsupported/);
  assert.match(block, /new_destination = new_directory\.join\(file_name\)/);
  assert.match(block, /mappings\.remove\(&id\)/);
  assert.match(desktop, /\n\s*relocate_download,\s*\n\s*redownload_downloads,/);
  assert.match(ui, /invoke\("relocate_download", \{ id: task\.id, newDirectory: selected \}\)/);
  assert.match(ui, /!isTorrent\(task\)/);
  assert.match(ui, /locateFile: "Locate file…",/);
});

test("a torrent using WebSeeding (BEP 19) shows its HTTP mirror count alongside the swarm", () => {
  assert.match(aria2, /pub web_seeds: u64,/);
  assert.match(aria2, /"files"\s*\n\s*\]\);/);
  assert.match(aria2, /uri\.get\("status"\)\.and_then\(Value::as_str\) == Some\("used"\)/);
  assert.match(aria2, /value\.starts_with\("http:\/\/"\) \|\| value\.starts_with\("https:\/\/"\)/);
  assert.match(model, /pub torrent_web_seeds: Option<u64>,/);
  assert.match(desktop, /item\.torrent_web_seeds = \(status\.web_seeds > 0\)\.then_some\(status\.web_seeds\);/);
  assert.match(ui, /webSeedStats/);
  assert.match(ui, /webMirror: "HTTP mirror",/);
});
