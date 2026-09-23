const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');

const root = join(__dirname, '..');
const aria2 = readFileSync(join(root, 'apps/desktop/src-tauri/src/aria2.rs'), 'utf8');
const rqbit = readFileSync(join(root, 'apps/desktop/src-tauri/src/rqbit.rs'), 'utf8');
const main = readFileSync(join(root, 'apps/desktop/src-tauri/src/main.rs'), 'utf8');

test('aria2 daemon is tied to the lifetime of the ADM parent process', () => {
  assert.match(aria2, /--stop-with-process=\{\}/);
  assert.match(aria2, /std::process::id\(\)/);
  assert.match(aria2, /pub fn pid\(&self\) -> u32/);
  assert.match(main, /aria2\.runtime_spawned/);
  assert.match(main, /stop_with_parent=true/);
});

test('opening Tools does not launch a second aria2c process for --version', () => {
  const start = main.indexOf('async fn get_tool_statuses');
  const end = main.indexOf('fn optional_path', start);
  assert.ok(start >= 0 && end > start);
  const block = main.slice(start, end);

  assert.doesNotMatch(block, /configured_aria2\(&settings\)[\s\S]{0,120}\["--version"\]/);
  assert.match(block, /aria2::Runtime::is_running/);
  assert.match(block, /endpoint\.version\(\)\.await/);
  assert.match(block, /found: aria2_path\.is_file\(\)/);
});

test('runtime stop diagnostics identify the daemon process and RPC port', () => {
  assert.match(main, /aria2\.runtime_stopped/);
  assert.match(main, /pid=\{pid\} port=\{port\}/);
});

test('rqbit_endpoint reuses a running server instead of spawning a second rqbit process', () => {
  const start = main.indexOf('async fn rqbit_endpoint');
  const end = main.indexOf('\nfn stop_rqbit_runtime', start);
  assert.ok(start >= 0 && end > start);
  const block = main.slice(start, end);
  assert.match(block, /let mut runtime = state\s*\.rqbit_runtime\s*\.lock\(\)/);
  assert.match(block, /let reuse = runtime\.as_mut\(\)\.is_some_and\(rqbit::Runtime::is_running\)/);
  assert.match(block, /if !reuse \{/);
  assert.match(block, /Same single-flight guarantee/);
});

test('opening Tools reuses the running rqbit endpoint for its version instead of launching a second process', () => {
  const start = main.indexOf('async fn get_tool_statuses');
  const end = main.indexOf('fn optional_path', start);
  assert.ok(start >= 0 && end > start);
  const block = main.slice(start, end);
  assert.match(block, /rqbit::Runtime::is_running/);
  assert.match(block, /let rqbit_version = match rqbit_endpoint \{/);
  assert.match(block, /endpoint\.version\(\)\.await\.ok\(\)/);
});

test('rqbit runtime is tied to the ADM parent process and logs its own pid/port', () => {
  assert.match(rqbit, /pub fn pid\(&self\) -> u32/);
  assert.match(rqbit, /pub fn terminate\(&mut self\)/);
  assert.match(main, /rqbit\.runtime_spawned/);
  assert.match(main, /rqbit\.runtime_stopped/);
});
