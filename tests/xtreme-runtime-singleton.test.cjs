const test = require("node:test"), assert = require("node:assert/strict"), fs = require("node:fs"), path = require("node:path");
const root = path.resolve(__dirname, "..");
const runtime = fs.readFileSync(path.join(root, "apps/desktop/src-tauri/src/xtreme.rs"), "utf8");
const main = fs.readFileSync(path.join(root, "apps/desktop/src-tauri/src/main.rs"), "utf8");
test("native runtime uses serve, loopback RPC and parent lifetime", () => {
  assert.match(runtime, /\.arg\("serve"\)/);
  assert.match(runtime, /--stop-with-process/);
  assert.match(runtime, /127\.0\.0\.1/);
  assert.match(runtime, /nibiru\.getVersion/);
  assert.match(runtime, /--session-file/);
  assert.match(main, /running_xtreme_endpoint/);
});
test("native torrent startup preserves peer port, peer cap and local discovery", () => {
  for (const flag of ["--listen-port", "6881-6999", "--bt-max-peers", "--enable-lpd", "--seed-ratio"]) assert.ok(runtime.includes(flag), flag);
});
test("direct HTTP keeps a 16-connection upper bound and native options", () => {
  assert.match(runtime, /clamp\(1, 16\)/);
  for (const option of ["max-connection-per-server", '"split"', "min-split-size"]) assert.ok(runtime.includes(option));
});
