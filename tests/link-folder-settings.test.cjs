const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const app = fs.readFileSync(path.join(root, "apps/desktop/ui/app.js"), "utf8");
const html = fs.readFileSync(path.join(root, "apps/desktop/ui/index.html"), "utf8");
const css = fs.readFileSync(path.join(root, "apps/desktop/ui/styles.css"), "utf8");
const rust = fs.readFileSync(path.join(root, "apps/desktop/src-tauri/src/main.rs"), "utf8");

test("Apocalipse Link selects and transfers both files and folders", () => {
  assert.match(app, /linkSelectedLocal = entry;/);
  assert.match(app, /linkSelectedRemote = entry;/);
  assert.match(app, /directory: linkSelectedRemote\.directory/);
  assert.match(rust, /PUT \/v1\/link\/directory\?path=/);
  assert.match(rust, /tokio::fs::create_dir_all\(&destination\)/);
  assert.match(rust, /send_link_directory\(&id, &password, &remote_path\)/);
});

test("Link exposes only explicit shares with per-share write permission", () => {
  assert.doesNotMatch(html, /id="link-allow-write"/);
  assert.match(html, /id="link-share-file"/);
  assert.match(html, /id="link-share-folder"/);
  assert.match(app, /update_link_share/);
  assert.match(app, /delete_remote_link_item/);
  assert.match(rust, /list_shared_link_directory/);
  assert.match(rust, /resolve_link_share/);
  assert.match(rust, /DELETE \/v1\/link\/item/);
  assert.match(rust, /link_write_not_allowed/);
  assert.match(html, /linkWindowsLoginNotice/);
});

test("Link lists remain readable and settings use the available window", () => {
  assert.match(css, /#settings-dialog \{ width: min\(1360px, calc\(100vw - 20px\)\)/);
  assert.match(css, /height:clamp\(300px,42vh,460px\)/);
  assert.match(css, /text-overflow:ellipsis/);
});

test("per-site rules own new credentials and expose removal", () => {
  assert.match(html, /website-credentials-settings" hidden aria-hidden="true"/);
  assert.match(app, /hostRuleRemoveConfirm/);
  assert.match(app, /invoke\("remove_host_rule"/);
  assert.match(app, /remove\.className = "danger-action"/);
});
