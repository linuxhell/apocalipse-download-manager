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
  assert.match(rust, /fn list_local_link_files\([\s\S]*list_shared_link_directory/);
  assert.match(app, /linkNoShares/);
  assert.match(app, /refreshVisibleLinkPanels/);
  assert.match(rust, /resolve_link_share/);
  assert.match(rust, /DELETE \/v1\/link\/item/);
  assert.match(rust, /link_write_not_allowed/);
  assert.match(html, /linkRemoteAuthPlan/);
  assert.match(html, /linkRemoteAccountFormats/);
  assert.match(html, /linkRemoteSecurityNotice/);
});

test("Link lists remain readable and settings use the available window", () => {
  assert.match(css, /#settings-dialog \{ width: min\(1360px, calc\(100vw - 20px\)\)/);
  assert.match(css, /height:clamp\(300px,42vh,460px\)/);
  assert.match(css, /text-overflow:ellipsis/);
});

test("per-site rules own new credentials and expose removal", () => {
  assert.doesNotMatch(html, /id="website-credential-/);
  assert.doesNotMatch(app, /invoke\("list_website_credentials"/);
  assert.doesNotMatch(rust, /fn list_website_credentials/);
  assert.match(rust, /legacy_website_credentials/);
  assert.match(app, /hostRuleRemoveConfirm/);
  assert.match(app, /invoke\("remove_host_rule"/);
  assert.match(app, /remove\.className = "danger-action"/);
});

test("Link share mutations refresh visible panels at the share root", () => {
  assert.match(app, /refreshVisibleLinkPanels\(\{ resetToRoot: true \}\)/);
  assert.match(app, /const localPath = resetToRoot \? "" : linkLocalPath;/);
  assert.match(app, /const remotePath = resetToRoot \? "" : linkRemotePath;/);
});

test("explicit Link shares remain visible even if metadata is temporarily unavailable", () => {
  assert.match(rust, /struct LinkShare[\s\S]*directory: bool/);
  assert.match(rust, /fn link_share_entries[\s\S]*\.map\(\|share\|/);
  assert.doesNotMatch(rust, /fn link_share_entries[\s\S]*\.filter_map\(\|share\|/);
  assert.match(rust, /map_or\(share\.directory, \|value\| value\.is_dir\(\)\)/);
  assert.match(rust, /share\.directory = metadata\.is_dir\(\);/);
  assert.match(rust, /directory: true/);
  assert.match(rust, /directory: false/);
});

test("Link remote guidance follows the selected language", () => {
  assert.match(html, /data-i18n="linkAccessNotice"/);
  assert.match(html, /data-i18n="linkRemoteAuthPlan"/);
  assert.match(html, /data-i18n="linkRemoteAccountFormats"/);
  assert.match(html, /data-i18n="linkRemoteSecurityNotice"/);
  assert.equal((app.match(/linkRemoteAuthPlan:/g) || []).length, 3);
  assert.equal((app.match(/linkRemoteAccountFormats:/g) || []).length, 3);
  assert.equal((app.match(/linkRemoteSecurityNotice:/g) || []).length, 3);
  assert.doesNotMatch(app, /linkWindowsLoginNotice:/);
  assert.doesNotMatch(app, /Authorized access shows all drives and folders/);
  assert.doesNotMatch(app, /O acesso autorizado mostra todas as unidades e pastas/);
});

test("Link exposes only system-account fields in the remote login", () => {
  assert.match(html, /id="link-remote-username"/);
  assert.match(html, /data-i18n="linkRemoteUsername"/);
  assert.match(html, /data-i18n="linkRemoteSystemPassword"/);
  assert.doesNotMatch(html, /id="link-own-password"/);
  assert.doesNotMatch(html, /id="link-new-password"/);
  assert.doesNotMatch(html, /linkCurrentPassword/);
  assert.doesNotMatch(html, /linkNewPassword/);
  assert.doesNotMatch(rust, /fn regenerate_link_password/);
  assert.doesNotMatch(rust, /regenerate_link_password,/);
  assert.equal((app.match(/linkRemoteUsername:/g) || []).length, 3);
  assert.equal((app.match(/linkRemoteSystemPassword:/g) || []).length, 3);
  assert.equal((app.match(/linkCredentialsRequired:/g) || []).length, 3);
  assert.equal((app.match(/linkNativeAuthPending:/g) || []).length, 3);
  assert.match(app, /const username = document\.querySelector\("#link-remote-username"\)\.value\.trim\(\);/);
  assert.match(app, /passwordField\.value = "";/);
  assert.doesNotMatch(app, /linkRemotePassword/);
});

test("Link has one address-based connection flow for loopback, LAN and Internet", () => {
  assert.doesNotMatch(html, /id="link-self-test"/);
  assert.doesNotMatch(app, /linkSelfTestMode/);
  assert.doesNotMatch(app, /linkSelfTest:/);
  assert.doesNotMatch(rust, /fn get_local_link_capabilities/);
  assert.match(html, /127\.0\.0\.1:17655/);
  assert.match(html, /data-i18n="linkRemoteAddressExamples"/);
  assert.equal((app.match(/linkRemoteAddressExamples:/g) || []).length, 3);
  assert.match(rust, /struct LinkIdentity\s*\{\s*id: String,\s*\}/);
  assert.doesNotMatch(rust, /struct LinkIdentity[\s\S]*password:/);
});
