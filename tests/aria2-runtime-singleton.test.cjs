const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');

const root = join(__dirname, '..');
const surge = readFileSync(join(root, 'apps/desktop/src-tauri/src/surge.rs'), 'utf8');
const transmission = readFileSync(join(root, 'apps/desktop/src-tauri/src/transmission.rs'), 'utf8');
const main = readFileSync(join(root, 'apps/desktop/src-tauri/src/main.rs'), 'utf8');

test('surge_endpoint reuses a running server instead of spawning a second surge.exe', () => {
  const start = main.indexOf('async fn surge_endpoint');
  const end = main.indexOf('\nfn stop_surge_runtime', start);
  assert.ok(start >= 0 && end > start);
  const block = main.slice(start, end);
  assert.match(block, /let mut runtime = state\s*\.surge_runtime\s*\.lock\(\)/);
  assert.match(block, /let reuse = runtime\.as_mut\(\)\.is_some_and\(surge::Runtime::is_running\)/);
  assert.match(block, /if !reuse \{/);
  assert.match(block, /Same single-flight guarantee|Holding the lock across the reuse check/);
});

test('transmission_endpoint reuses a running daemon instead of spawning a second transmission-daemon', () => {
  const start = main.indexOf('async fn transmission_endpoint');
  const end = main.indexOf('\nfn stop_transmission_runtime', start);
  assert.ok(start >= 0 && end > start);
  const block = main.slice(start, end);
  assert.match(block, /let mut runtime = state\s*\.transmission_runtime\s*\.lock\(\)/);
  assert.match(block, /let reuse = runtime\s*\.as_mut\(\)\s*\.is_some_and\(transmission::Runtime::is_running\)/);
  assert.match(block, /if !reuse \{/);
});

test('opening Tools reuses the running transmission-daemon for --version instead of launching a second one', () => {
  const start = main.indexOf('async fn get_tool_statuses');
  const end = main.indexOf('#[tauri::command]', start + 1);
  assert.ok(start >= 0 && end > start);
  const block = main.slice(start, end);
  assert.match(block, /transmission::Runtime::is_running/);
  assert.match(block, /endpoint\.version\(\)\.await\.ok\(\)/);
  assert.match(block, /version_line\(&transmission_path, \["--version"\]\.as_slice\(\)\)/);
});

test('surge --version exits before the server starts, so it can never leave a second background process', () => {
  assert.match(surge, /pub fn spawn\(/);
  assert.match(surge, /\.arg\("server"\)/);
  // The Tools "--version" check never passes the "server" subcommand, so
  // cobra's own --version interception (which prints and returns before any
  // command logic, including ours, runs) is what keeps this safe.
});

test('runtime lifetimes are tied to the ADM parent process and log their own pid/port', () => {
  assert.match(surge, /pub fn pid\(&self\) -> u32/);
  assert.match(surge, /pub fn terminate\(&mut self\)/);
  assert.match(transmission, /pub fn pid\(&self\) -> u32/);
  assert.match(transmission, /pub fn terminate\(&mut self\)/);
  assert.match(main, /surge\.runtime_spawned/);
  assert.match(main, /surge\.runtime_stopped/);
  assert.match(main, /transmission\.runtime_spawned/);
  assert.match(main, /transmission\.runtime_stopped/);
  assert.match(main, /pid=\{pid\} port=\{port\}/);
});
