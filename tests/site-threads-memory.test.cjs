const test = require("node:test"), assert = require("node:assert/strict"), fs = require("node:fs"), path = require("node:path");
const ui = (name) => fs.readFileSync(path.join(__dirname, "..", "apps/desktop/ui", name), "utf8");
const app = ui("app.js"), html = ui("index.html");
const rust = fs.readFileSync(path.join(__dirname, "..", "apps/desktop/src-tauri/src/main.rs"), "utf8");

test("the download window offers 1-16 threads, defaulting to 8, with a per-site memory box", () => {
  assert.match(html, /id="task-connections" max="16" min="1" type="range" value="8"/);
  assert.match(html, /id="remember-site-threads" type="checkbox"/);
  assert.match(html, /id="connections" max="16" min="1" type="range" value="8"/);
  assert.match(html, /id="host-rule-connections" max="16"/);
  assert.match(rust, /const MAX_DOWNLOAD_CONNECTIONS: usize = 16;/);
  assert.match(rust, /const fn default_connections\(\) -> usize \{\s*8\s*\}/);
});

test("every language has every site-thread string", () => {
  for (const key of ["rememberSiteThreads", "rememberSiteThreadsHint", "rememberSiteThreadsFor", "rememberSiteThreadsOffer", "siteThreadsForced"]) {
    assert.equal((app.match(new RegExp(`^\\s{4}${key}: "`, "gm")) || []).length, 3, `${key} must exist in en, pt-BR and zh-CN`);
  }
  for (const key of ["rememberSiteThreadsFor", "rememberSiteThreadsOffer", "siteThreadsForced"]) {
    for (const line of app.split("\n").filter((l) => l.trimStart().startsWith(key + ":"))) assert.ok(line.includes("{host}"), line);
  }
});

test("the dialog saves, updates or forgets the count only when the download starts", () => {
  assert.match(app, /await applySiteThreadsChoice\(url\.value\);/);
  assert.match(app, /if \(box\.checked\) await invoke\("set_site_threads", \{ url: source, threads \}\);/);
  assert.match(app, /else if \(siteThreads\.remembered\) await invoke\("set_site_threads", \{ url: source, threads: null \}\);/);
  assert.match(rust, /fn get_site_threads/);
  assert.match(rust, /fn set_site_threads/);
});
