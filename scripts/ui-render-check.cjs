// Renders the ADM UI pages in every language and colour mode against a stubbed
// Tauri bridge, then reports raw i18n keys, JS errors and horizontal overflow and
// writes screenshots. Usage: node scripts/ui-render-check.cjs [outDir]
// Needs Playwright with a Chromium (set CHROMIUM_PATH if it is not auto-detected).
let chromium;
try { ({ chromium } = require("playwright")); } catch { ({ chromium } = require(process.env.PLAYWRIGHT_MODULE || "/opt/node22/lib/node_modules/playwright")); }
const http = require("http"), fs = require("fs"), path = require("path");
const UI = path.join(__dirname, "..", "apps", "desktop", "ui");
const OUT = path.resolve(process.argv[2] || path.join(require("os").tmpdir(), "adm-ui-render"));
fs.mkdirSync(OUT, { recursive: true });
const types = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".png": "image/png", ".svg": "image/svg+xml", ".jpg": "image/jpeg", ".webp": "image/webp", ".mp3": "audio/mpeg", ".woff2": "font/woff2" };
const server = http.createServer((req, res) => {
  const file = path.join(UI, decodeURIComponent(req.url.split("?")[0]).replace(/^\/$/, "/index.html"));
  if (!file.startsWith(UI) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) { res.writeHead(404); return res.end(); }
  res.writeHead(200, { "Content-Type": types[path.extname(file)] || "application/octet-stream" });
  fs.createReadStream(file).pipe(res);
});
const now = Math.floor(Date.now() / 1000);
const task = (id, name, state, extra = {}) => ({ id, source: `https://example.com/${name}`, destination: `/home/u/Downloads/${name}`, state, received: 0, total: null, created_at: now - 100, priority: 0, mirrors: [], torrent_selection: [], auto_extract: false, ...extra });
const tasks = [
  task("00000000-0000-4000-8000-000000000001", "ubuntu.iso", "downloading", { received: 3e9, total: 5e9, progress_percent: 60, download_speed: 12e6, resume_supported: true }),
  task("00000000-0000-4000-8000-000000000002", "movie.mp4", "paused", { received: 1e8, total: 4e8, progress_percent: 25, resume_supported: true }),
  task("00000000-0000-4000-8000-000000000003", "archive.zip", "completed", { received: 5e7, total: 5e7, progress_percent: 100, completed_at: now - 5 }),
  task("00000000-0000-4000-8000-000000000004", "clash.bin", { failed: { message: "destination already exists" } }),
  task("00000000-0000-4000-8000-000000000005", "stream.m3u8", { failed: { message: "hls_requires_nibiru_m3u8" } }),
  task("00000000-0000-4000-8000-000000000006", "a-very-long-file-name-that-keeps-going-and-going-and-going-to-test-ellipsis-handling-in-the-row-layout.tar.gz", "queued"),
];
const stub = (tasksJson) => `
  window.__calls = [];
  window.__TAURI__ = { core: { invoke: async (cmd, args) => {
    window.__calls.push(cmd);
    const T = ${tasksJson};
    switch (cmd) {
      case "list_downloads": return T;
      case "default_download_directory": return "/home/u/Downloads";
      case "list_download_directories": return [{ path: "/home/u/Downloads", available: true }, { path: "/mnt/big/Videos", available: false }];
      case "get_tool_statuses": return [{id:"xtreme",path:"/x",found:true,version:"0.2.0"},{id:"power",path:"/p",found:false,version:null},{id:"ffmpeg",path:"/f",found:true,version:"ffmpeg 7.0"},{id:"yt-dlp",path:"/y",found:false,version:null}];
      case "get_transfer_limits": return { maxActiveDownloads: 3, connectionsPerDownload: 8, globalBandwidthLimit: 0, adaptiveEfficiency: true, downloadBandwidthLimit: 0 };
      case "get_autostart": return { enabled: false, supported: true };
      case "get_clipboard_monitor": return false;
      case "get_associations": return ["m3u8","torrent","magnet","ftp","sftp"].map(id => ({ id, enabled: false, supported: true }));
      case "get_proxy_setting": return { enabled:false, url:"", username:"", hasPassword:false };
      case "get_dns_setting": return { enabled:false, servers:[] };
      case "get_user_agent": return { userAgent: "" };
      case "get_log_editor": return "";
      case "get_bridge_pairing": return { token: "x".repeat(32), connected: true };
      case "get_xtreme_rpc_settings": return { enabled:true, autoStart:true, configuredPort:null, connected:true, activePort:6800, version:"0.2.0" };
      case "get_torrent_store_preference": return true;
      case "get_application_language": return window.__LANG;
      case "get_dark_mode": return window.__MODE === "dark";
      case "get_app_version": return "0.6.1";
      case "get_link_transport_options": return { enabled:false, localIps:[] };
      case "get_link_identity": return { name:"PC-TEST", ip:"192.168.0.10", port:17655, id:"pc-test@192.168.0.10:17655" };
      case "list_link_shares": return [{ id:"s1", name:"Docs", path:"/home/u/Docs", allowWrite:false, directory:true }];
      case "list_local_link_files": return [];
      case "ed2k_overview": return { ip_filter:{ranges:3,skipped_lines:1}, server:{connected:false}, kad:{running:false}, downloads:[], servers:[], search:[], uploads:[] };
      case "ed2k_shared_list": return [];
      case "list_host_rules": return [];
      case "read_general_log": return "2026-10-21 INFO sample log line";
      default: return null;
    }
  }}, event: { listen: async () => () => {} } };
  localStorage.setItem("apocalipse.language", window.__LANG);
  localStorage.setItem("apocalipse.mode", window.__MODE);
`;
(async () => {
  await new Promise(r => server.listen(0, r));
  const port = server.address().port;
  const browser = await chromium.launch({ ...(process.env.CHROMIUM_PATH ? { executablePath: process.env.CHROMIUM_PATH } : {}), args: ["--no-sandbox"] });
  const report = [];
  for (const lang of ["en", "pt-BR", "zh-CN"]) for (const mode of ["light", "dark"]) for (const page of ["index.html", "link.html", "ed2k.html"]) {
    const ctx = await browser.newContext({ viewport: { width: 1180, height: 760 } });
    const p = await ctx.newPage();
    const errors = [];
    p.on("response", r => { if (r.status() === 404) errors.push("404: " + r.url()); });
    p.on("pageerror", e => errors.push("pageerror: " + e.message));
    p.on("console", m => { if (m.type() === "error") errors.push("console: " + m.text().slice(0, 200)); });
    await p.addInitScript(`window.__LANG=${JSON.stringify(lang)};window.__MODE=${JSON.stringify(mode)};` + stub(JSON.stringify(tasks)));
    await p.goto(`http://127.0.0.1:${port}/${page}`);
    await p.waitForTimeout(900);
    const findings = await p.evaluate(() => {
      const out = { rawKeys: [], overflow: [], clipped: [] };
      const vw = document.documentElement.clientWidth;
      for (const el of document.querySelectorAll("body *")) {
        const cs = getComputedStyle(el);
        if (cs.display === "none" || cs.visibility === "hidden") continue;
        const r = el.getBoundingClientRect();
        if (r.width === 0 || r.height === 0) continue;
        if (el.children.length === 0) {
          const txt = (el.textContent || "").trim();
          if (/^[a-z]+(?:[A-Z][a-z0-9]+)+$/.test(txt) && txt.length > 5) out.rawKeys.push(txt);
        }
        if (r.right > vw + 1 && cs.position !== "fixed") out.overflow.push(`${el.tagName.toLowerCase()}.${(el.className || "").toString().slice(0, 30)} right=${Math.round(r.right)}`);
      }
      out.overflow = out.overflow.slice(0, 5);
      out.scrollW = document.documentElement.scrollWidth; out.vw = vw;
      return out;
    });
    const shot = path.join(OUT, `${page.replace(".html", "")}-${lang}-${mode}.png`);
    await p.screenshot({ path: shot });
    // contrast: sample body text/background colors
    const colors = await p.evaluate(() => { const b = getComputedStyle(document.body); return { fg: b.color, bg: b.backgroundColor }; });
    const tour = [];
    if (page === "ed2k.html") {
      const scan = () => p.evaluate(() => { const raw = []; for (const el of document.querySelectorAll("body *")) { const cs = getComputedStyle(el); if (cs.display === "none" || cs.visibility === "hidden") continue; const r = el.getBoundingClientRect(); if (!r.width || !r.height || el.children.length) continue; const t = (el.textContent || "").trim(); if (/^[a-z]+(?:[A-Z][a-z0-9]+)+$/.test(t) && t.length > 5) raw.push(t); } return [...new Set(raw)]; });
      for (const tabName of await p.$$eval("[data-tab]", nodes => nodes.map(n => n.dataset.tab))) {
        await p.click(`[data-tab="${tabName}"]`).catch(() => {});
        await p.waitForTimeout(300);
        await p.screenshot({ path: path.join(OUT, `tour-ed2k-${tabName}-${lang}-${mode}.png`) });
        tour.push({ target: `ed2k-${tabName}`, raw: await scan() });
      }
    }
    if (page === "index.html") {
      const scan = () => p.evaluate(() => { const raw = []; for (const el of document.querySelectorAll("body *")) { const cs = getComputedStyle(el); if (cs.display === "none" || cs.visibility === "hidden") continue; const r = el.getBoundingClientRect(); if (!r.width || !r.height || el.children.length) continue; const t = (el.textContent || "").trim(); if (/^[a-z]+(?:[A-Z][a-z0-9]+)+$/.test(t) && t.length > 5) raw.push(t); } return [...new Set(raw)]; });
      for (const target of ["torrents", "recordings", "settings", "about", "downloads"]) {
        await p.click(`[data-page="${target}"]`).catch(() => {});
        await p.waitForTimeout(500);
        await p.screenshot({ path: path.join(OUT, `tour-${target}-${lang}-${mode}.png`) });
        tour.push({ target, raw: await scan() });
        if (target === "settings") {
          for (const tab of ["language", "tools", "network", "extension", "logs"]) {
            await p.click(`[data-settings-tab="${tab}"]`).catch(() => {});
            await p.waitForTimeout(300);
            await p.screenshot({ path: path.join(OUT, `tour-settings-${tab}-${lang}-${mode}.png`) });
            tour.push({ target: `settings-${tab}`, raw: await scan() });
          }
        }
      }
      await p.click("#add-download, [data-action='add'], button:has-text('Add download'), button:has-text('Adicionar download'), button:has-text('添加下载')").catch(() => {});
      await p.waitForTimeout(400);
      await p.screenshot({ path: path.join(OUT, `tour-add-${lang}-${mode}.png`) });
      tour.push({ target: "add-dialog", raw: await scan() });
    }
    report.push({ page, lang, mode, errors, ...findings, colors, tour });
    await ctx.close();
  }
  await browser.close(); server.close();
  fs.writeFileSync(path.join(OUT, "report.json"), JSON.stringify(report, null, 1));
  let failed = false;
  for (const r of report) {
    const issues = [];
    if (r.errors.length) issues.push("errors=" + JSON.stringify(r.errors.slice(0, 3)));
    if (r.rawKeys.length) issues.push("rawKeys=" + JSON.stringify([...new Set(r.rawKeys)].slice(0, 8)));
    if (r.overflow.length || r.scrollW > r.vw) issues.push(`overflow scrollW=${r.scrollW}/${r.vw} ${JSON.stringify(r.overflow)}`);
    for (const t of r.tour || []) if (t.raw.length) issues.push(`tour:${t.target} raw=${JSON.stringify(t.raw)}`);
    if (r.errors.some(e => !/favicon|404/.test(e)) || r.rawKeys.length || (r.tour || []).some(t => t.raw.length)) failed = true;
    console.log(`${r.page} ${r.lang} ${r.mode} fg=${r.colors.fg} bg=${r.colors.bg} ${issues.length ? "ISSUES " + issues.join(" ") : "ok"}`);
  }
  if (failed) process.exit(1);
})().catch(e => { console.error(e); process.exit(1); });
