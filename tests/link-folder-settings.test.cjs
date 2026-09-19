const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const app = fs.readFileSync(path.join(root, "apps/desktop/ui/app.js"), "utf8");
const html = fs.readFileSync(path.join(root, "apps/desktop/ui/index.html"), "utf8");
const css = fs.readFileSync(path.join(root, "apps/desktop/ui/styles.css"), "utf8");
const linkHtml = fs.readFileSync(path.join(root, "apps/desktop/ui/link.html"), "utf8");
const linkJs = fs.readFileSync(path.join(root, "apps/desktop/ui/link.js"), "utf8");
const rust = fs.readFileSync(path.join(root, "apps/desktop/src-tauri/src/main.rs"), "utf8");
const workflow = fs.readFileSync(path.join(root, ".github/workflows/validate-portable.yml"), "utf8");

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

test("Windows SMB shared folders are discovered automatically for Link", () => {
  assert.match(rust, /fn windows_shared_link_shares\(\)/);
  assert.match(rust, /NetShareEnum/);
  assert.match(rust, /STYPE_SPECIAL/);
  assert.match(rust, /effective_link_shares\(settings\)/);
  assert.match(rust, /stable_link_share_id\("windows"/);
  assert.match(linkJs, /Windows SMB shared folders are also discovered automatically/);
  assert.match(linkJs, /Pastas compartilhadas pelo Windows \(SMB\) também aparecem automaticamente/);
});

test("Linux Samba shares are discovered from smb.conf and usershares", () => {
  assert.match(rust, /fn linux_smb_config_entries\(contents: &str\)/);
  assert.match(rust, /\/etc\/samba\/smb\.conf/);
  assert.match(rust, /\/var\/lib\/samba\/usershares/);
  assert.match(rust, /fn linux_smb_usershare_entry/);
  assert.match(rust, /stable_link_share_id\("linux-smb"/);
  assert.match(rust, /chain\(linux_shared_link_shares\(\)\)/);
  assert.match(linkJs, /Windows and Linux SMB shared folders are also discovered automatically/);
  assert.match(linkJs, /Windows ou Linux via SMB também aparecem automaticamente/);
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

test("Link keeps system-account fields for future encrypted remote login without requiring them for loopback", () => {
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
  assert.match(app, /passwordField\.value = "";/);
  assert.doesNotMatch(app, /invoke\("authenticate_local_link_account"/);
  assert.doesNotMatch(linkJs, /invoke\("authenticate_local_link_account"/);
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
  assert.doesNotMatch(rust, /struct LinkIdentity\s*\{[^}]*password:/);
});

test("Loopback Link opens explicit shares without a Windows password round-trip", () => {
  assert.match(app, /function isLocalLinkTarget\(value\)/);
  assert.match(app, /host === "127\.0\.0\.1"/);
  assert.match(app, /if \(\!isLocalLinkTarget\(id\)\)[\s\S]*linkNativeAuthPending/);
  assert.match(app, /linkRemoteId = id;[\s\S]*linkLocalAccountSession = true;[\s\S]*await openRemoteLink\(""\)/);
  assert.match(linkJs, /linkRemoteId = id;[\s\S]*linkLocalAccountSession = true;[\s\S]*await openRemoteLink\(""\)/);
  assert.doesNotMatch(app, /invoke\("authenticate_local_link_account"/);
  assert.doesNotMatch(linkJs, /invoke\("authenticate_local_link_account"/);
  assert.match(rust, /fn list_local_link_files\([\s\S]*list_shared_link_directory/);
  assert.match(rust, /fn list_shared_link_directory\([\s\S]*link_share_entries\(settings\)/);
});

test("Native system-account authentication remains available for encrypted remote Link transport", () => {
  assert.match(rust, /fn authenticate_local_link_account\([\s\S]*password\.zeroize\(\)/);
  assert.match(rust, /#\[cfg\(windows\)\][\s\S]*fn verify_system_account\([\s\S]*LogonUserW/);
  assert.match(rust, /Some\("\."\.to_owned\(\)\)/);
  assert.match(rust, /MicrosoftAccount/);
  assert.match(rust, /#\[cfg\(unix\)\][\s\S]*#\[link\(name = "pam"\)\]/);
  assert.match(rust, /pam_start\(/);
  assert.match(rust, /pam_authenticate\(/);
  assert.match(rust, /pam_acct_mgmt\(/);
  assert.match(rust, /data\.password\.zeroize\(\)/);
  assert.match(workflow, /libpam0g-dev/);
});

test("Loopback Link file operations bypass the legacy remote transport", () => {
  assert.match(app, /linkLocalAccountSession[\s\S]*delete_local_shared_link_item/);
  assert.match(app, /linkLocalAccountSession[\s\S]*download_local_shared_link_item/);
  assert.match(app, /linkLocalAccountSession[\s\S]*upload_local_shared_link_item/);
  assert.match(rust, /fn delete_local_shared_link_item/);
  assert.match(rust, /fn download_local_shared_link_item/);
  assert.match(rust, /fn upload_local_shared_link_item/);
  assert.match(rust, /copy_local_link_directory/);
  assert.match(rust, /link_write_not_allowed/);
});


test("About page is localized, sits immediately below PayPal and keeps the main window size", () => {
  assert.match(html, /id="donate-paypal"[\s\S]*data-page="about"/);
  assert.match(html, /id="about-panel"/);
  assert.match(html, /assets\/about-creator\.jpg/);
  assert.match(html, /assets\/about-theme\.mp4/);
  assert.match(app, /about: "About"/);
  assert.match(app, /about: "Sobre"/);
  assert.match(app, /about: "关于"/);
  assert.match(app, /aboutAudio\.pause\(\)/);
  assert.match(app, /aboutAudio\.currentTime = 0/);
  assert.match(app, /aboutAudio\.play\(\)/);
  assert.match(app, /document\.querySelector\("#add"\)\.hidden = activePage === "about"/);
  assert.match(css, /\.about-creator-line[\s\S]*font-size: 20px/);
  assert.match(css, /nav \{ min-height: 0; overflow-y: auto;/);
});

test("Apocalipse Link opens as a dedicated maximized native window with normal controls", () => {
  assert.match(app, /invoke\("open_link_window"\)/);
  assert.match(rust, /async fn open_link_window/);
  assert.match(rust, /WebviewUrl::App\("link\.html"\.into\(\)\)/);
  assert.match(rust, /\.maximized\(true\)/);
  assert.match(rust, /\.decorations\(true\)/);
  assert.match(rust, /\.resizable\(true\)/);
  assert.match(rust, /window\.label\(\) == "main"/);
  assert.match(linkHtml, /class="link-window"/);
  assert.match(linkJs, /loadLinkIdentity\(\)/);
  assert.match(css, /body\.link-window[\s\S]*height: 100vh/);
});

test("Windows Link retries Microsoft and Azure account forms after ERROR_LOGON_FAILURE", () => {
  assert.match(rust, /fn windows_logon_candidates/);
  assert.match(rust, /Some\("MicrosoftAccount"\.to_owned\(\)\)/);
  assert.match(rust, /Some\("AzureAD"\.to_owned\(\)\)/);
  assert.match(rust, /for logon_type in \[3_u32, 2_u32\]/);
  assert.match(linkJs, /system_auth_failed:1326/);
  assert.match(linkJs, /Windows Hello PIN/);
});

test("Linux validation covers Debian Fedora Arch and publishes an AppImage", () => {
  assert.match(workflow, /Debian 12/);
  assert.match(workflow, /fedora:latest/);
  assert.match(workflow, /archlinux:latest/);
  assert.match(workflow, /cargo check --locked -p apocalipse-desktop/);
  assert.match(workflow, /@tauri-apps\/cli@2 build --bundles appimage/);
  assert.match(workflow, /apocalipse-download-manager-linux-x64\.AppImage/);
  assert.match(workflow, /ubuntu-22\.04/);
});
