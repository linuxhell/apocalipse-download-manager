const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');

const html = readFileSync(join(__dirname, '../apps/desktop/ui/index.html'), 'utf8');
const js = readFileSync(join(__dirname, '../apps/desktop/ui/app.js'), 'utf8');
const css = readFileSync(join(__dirname, '../apps/desktop/ui/styles.css'), 'utf8');
const rust = readFileSync(join(__dirname, '../apps/desktop/src-tauri/src/main.rs'), 'utf8');

test('Tools offers localized Download beside Browse for every configurable item', () => {
  for (const id of ['ffmpeg', 'yt-dlp', 'qjs', 'n-m3u8dl-re', 'aria2next', 'extractor', 'player']) {
    assert.match(html, new RegExp(`data-tool-download="${id}"`));
    assert.match(html, new RegExp(`data-tool-pick="${id}"`));
  }
  assert.doesNotMatch(html, /data-tool-update="extractor"/);
  assert.doesNotMatch(html, /data-tool-update="player"/);
  assert.match(js, /downloadTool: "Download"/);
  assert.match(js, /downloadTool: "Baixar"/);
  assert.match(js, /downloadTool: "下载"/);
});

test('Downloaded tools remain pending until Save and player uses the same path workflow', () => {
  assert.match(js, /invoke\("download_tool", \{ id \}\)/);
  assert.match(js, /input\.value = path/);
  assert.match(js, /set_media_player", \{ path: document\.querySelector\("#tool-player"\)\.value \}/);
  assert.match(rust, /fn portable_tools_directory\(\)/);
  assert.match(rust, /\.parent\(\)[\s\S]*\.join\("tools"\)/);
  assert.match(rust, /async fn download_tool/);
  assert.match(rust, /download_tool,\s*update_tool/);
});

test('Extractor and player auto-download are platform-specific', () => {
  assert.match(rust, /ip7z\/7zip/);
  assert.match(rust, /linux-x64\.tar\.xz/);
  assert.match(rust, /-mac\.tar\.xz/);
  assert.match(rust, /7z\.exe/);
  assert.match(rust, /mpv-player\/mpv/);
  assert.match(rust, /pkgforge-dev\/mpv-AppImage/);
  assert.match(rust, /APPIMAGE_EXTRACT_AND_RUN/);
});

test('Tools dialog expands instead of squeezing four controls into the old width', () => {
  assert.match(css, /#tools-dialog \{ width: min\(1460px, calc\(100vw - 20px\)\); max-width: none;/);
  assert.match(css, /grid-template-columns: minmax\(0, 1fr\) auto auto auto/);
  assert.match(css, /#tools-dialog \.tool-settings \{ min-height: 0; overflow: auto; \}/);
});


test('Opening Tools never launches an already configured media player', () => {
  const playerStatusStart = rust.indexOf('let player = settings.media_player_path.clone().unwrap_or_default();');
  const playerStatusEnd = rust.indexOf('Ok(statuses)', playerStatusStart);
  const playerStatus = rust.slice(playerStatusStart, playerStatusEnd);
  assert.ok(playerStatusStart >= 0 && playerStatusEnd > playerStatusStart);
  assert.doesNotMatch(playerStatus, /version_line\(&player/);
  assert.match(playerStatus, /version:\s*None/);
});
