from pathlib import Path
import json
import re

# 0.3.50 test: no CDP, strong Normal/Force/Bypass capture, powerful logs,
# dedicated Themes/Language/Logs navigation, and localized tray menu.

# ---------------- Extension: generic force capture + trace logs ----------------
p = Path('browser-extension/background.js')
s = p.read_text(encoding='utf-8')
s = s.replace('const recognizedDownload = (value) => {', '''const genericHttpDownload = (value) => {
  try {
    const url = new URL(value);
    return /^(https?):$/i.test(url.protocol) ? { url: url.href, kind: "forced" } : null;
  } catch { return null; }
};

const recognizedDownload = (value) => {''', 1)
s = s.replace('  const recognized = recognizedDownload(request?.url || "");\n  if (!recognized) throw new Error("unsupported_pre_download_url");', '  const recognized = recognizedDownload(request?.url || "") || (request?.force ? genericHttpDownload(request?.url || "") : null);\n  if (!recognized) throw new Error("unsupported_pre_download_url");', 1)
s = s.replace('  const recognized = recognizedDownload(item?.finalUrl || item?.url || "");\n  if (!recognized || bypassIsActive()) return;', '''  const forced = (() => { try { return forceHeld || Date.now() < forceUntil; } catch { return false; } })();
  const recognized = recognizedDownload(item?.finalUrl || item?.url || "") || (forced ? genericHttpDownload(item?.finalUrl || item?.url || "") : null);
  if (!recognized || bypassIsActive()) return;''', 1)
s = s.replace('      source: "chrome.downloads.fallback",\n    }))', '      source: forced ? "force.chrome-downloads-fallback" : "chrome.downloads.fallback",\n      force: forced,\n    }))', 1)
s = s.replace('    source: message.source || "main-world",\n  };', '    source: message.source || "main-world",\n    force: Boolean(message.force),\n  };', 1)
s = s.replace('      const recognized = recognizedDownload(request.url);', '      const recognized = recognizedDownload(request.url) || (request.force ? genericHttpDownload(request.url) : null);', 1)
# trace messages from the MAIN-world hook
needle = 'chrome.runtime.onMessage.addListener((message, sender, reply) => {\n  if (message?.type !== "APOCALIPSE_PRE_DOWNLOAD_URL") return;\n'
if needle in s:
    s = s.replace(needle, '''chrome.runtime.onMessage.addListener((message, sender, reply) => {
  if (message?.type === "APOCALIPSE_CAPTURE_TRACE") {
    const state = { traceId: message.traceId || crypto.randomUUID(), pageUrl: message.pageUrl || sender.tab?.url || null, startedAt: Number(message.at || Date.now()), bytes: 0 };
    const detail = Object.entries(message.detail || {}).map(([k,v]) => `${k}=${String(v ?? "").slice(0,180)}`).join(" ");
    void diagnostic(`capture.${String(message.eventName || "event")}`, state, { detail: `mode=${message.mode || "normal"} ${detail}`.trim() });
    reply({ ok: true });
    return;
  }
  if (message?.type !== "APOCALIPSE_PRE_DOWNLOAD_URL") return;
''', 1)
p.write_text(s, encoding='utf-8')

p = Path('browser-extension/content.js')
s = p.read_text(encoding='utf-8')
needle = '  window.addEventListener("message", (event) => {\n    if (event.source !== window) return;\n    const data = event.data;\n'
if needle in s and 'APOCALIPSE_CAPTURE_TRACE' not in s:
    s = s.replace(needle, '''  window.addEventListener("message", (event) => {
    if (event.source !== window) return;
    const data = event.data;
    if (data?.source === "apocalipse-page-hook" && data.type === "capture-trace") {
      chrome.runtime.sendMessage({ type: "APOCALIPSE_CAPTURE_TRACE", eventName: data.eventName, mode: data.mode, detail: data.detail || {}, traceId: data.traceId, pageUrl: location.href, at: data.at || Date.now() }).catch(() => {});
      return;
    }
''', 1)
s = s.replace('      source: data.kind || "main-world",\n    }).then', '      source: data.kind || "main-world",\n      force: Boolean(data.force),\n    }).then', 1)
p.write_text(s, encoding='utf-8')

p = Path('browser-extension/page-hook.js')
s = p.read_text(encoding='utf-8')
s = s.replace('  let activeLibraryFileName = "";\n', '  let activeLibraryFileName = "";\n  let forceGestureUntil = 0;\n  let activeTraceId = "";\n', 1)
s = s.replace('  const bypassPressed = () => held.has(shortcuts.bypass || "Alt");\n', '''  const bypassPressed = () => held.has(shortcuts.bypass || "Alt");
  const forcePressed = () => held.has(shortcuts.force || "Shift");
  const forceActive = () => forcePressed() || Date.now() < forceGestureUntil;
  const safeUrl = (value) => { try { const u = new URL(String(value || ""), location.href); u.search = ""; u.hash = ""; return u.href; } catch { return ""; } };
  const trace = (eventName, detail = {}) => window.postMessage({ source: "apocalipse-page-hook", type: "capture-trace", eventName, mode: bypassPressed() ? "bypass" : (forceActive() ? "force" : "normal"), detail, traceId: activeTraceId, at: Date.now() }, "*");
  const forceClassify = (value) => { if (!forceActive() || bypassPressed()) return null; try { const u = new URL(String(value || ""), location.href); return /^(https?):$/i.test(u.protocol) ? { url: u.href, kind: "forced" } : null; } catch { return null; } };
''', 1)
s = s.replace('  const emit = (candidate, primitive) => {\n    if (!candidate || bypassPressed()) return false;', '''  const emit = (candidate, primitive) => {
    if (!candidate || bypassPressed()) { if (candidate && bypassPressed()) trace("BYPASS", { primitive, url: safeUrl(candidate.url) }); return false; }''', 1)
s = s.replace('    const requestId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;\n', '    const requestId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;\n    trace(forceActive() ? "FORCE_CAPTURE" : "AUTO_ACCEPT", { primitive, kind: candidate.kind, url: safeUrl(candidate.url), requestId });\n', 1)
s = s.replace('      fileName: candidate.kind === "chatgpt-library" ? activeLibraryFileName : "",\n', '      fileName: candidate.kind === "chatgpt-library" ? activeLibraryFileName : "",\n      force: forceActive(),\n', 1)
# Gesture correlation: only the consequences of the user's force-click get aggressive capture.
s = s.replace('  document.addEventListener("pointerdown", (event) => {\n    const button = event.target?.closest?.', '''  document.addEventListener("pointerdown", (event) => {
    activeTraceId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
    if (forcePressed()) forceGestureUntil = Date.now() + 5000;
    const target = event.target instanceof Element ? event.target : null;
    const clickable = target?.closest?.("a[href],button,[role=button],[role=menuitem]");
    trace(forcePressed() ? "FORCE_ARMED" : (bypassPressed() ? "BYPASS_ARMED" : "AUTO_GESTURE"), { tag: clickable?.tagName || target?.tagName || "", role: clickable?.getAttribute?.("role") || "", text: String(clickable?.innerText || clickable?.textContent || "").trim().slice(0,120), href: safeUrl(clickable?.href || "") });
    const button = event.target?.closest?.''', 1)
s = s.replace('    const candidate = classify(this.href);', '    const candidate = classify(this.href) || forceClassify(this.href);', 1)
s = s.replace('    const candidate = classify(url);\n    if (emit(candidate, "window.open"))', '    const candidate = classify(url) || forceClassify(url);\n    if (emit(candidate, "window.open"))', 1)
# For fetch, never swallow arbitrary API requests. Known downloads are intercepted; force only observes file-like responses.
old_fetch = '''  const originalFetch = window.fetch;
  window.fetch = function(input, init) {
    const value = typeof input === "string" || input instanceof URL ? String(input) : input?.url;
    const candidate = classify(value);
    if (emit(candidate, "window.fetch")) {
      return Promise.resolve(new Response("", { status: 204, statusText: "Handled by Apocalipse" }));
    }
    return originalFetch.call(this, input, init);
  };'''
new_fetch = '''  const originalFetch = window.fetch;
  window.fetch = function(input, init) {
    const value = typeof input === "string" || input instanceof URL ? String(input) : input?.url;
    const candidate = classify(value);
    if (emit(candidate, "window.fetch")) return Promise.resolve(new Response("", { status: 204, statusText: "Handled by Apocalipse" }));
    const forcedAtCall = forceActive() && !bypassPressed();
    return originalFetch.call(this, input, init).then((response) => {
      if (forcedAtCall) {
        const disposition = response.headers?.get?.("content-disposition") || "";
        const type = response.headers?.get?.("content-type") || "";
        const fileLike = /attachment|filename=/i.test(disposition) || /application\/(octet-stream|zip|x-rar|pdf)|video\//i.test(type);
        if (fileLike) emit(forceClassify(response.url || value), "window.fetch.response");
        else trace("FORCE_OBSERVED_RESPONSE", { status: response.status, type: type.slice(0,80), url: safeUrl(response.url || value) });
      }
      return response;
    });
  };'''
if old_fetch in s: s = s.replace(old_fetch, new_fetch, 1)
s = s.replace('    const candidate = classify(anchor?.href);', '    const candidate = classify(anchor?.href) || forceClassify(anchor?.href);', 1)
s = s.replace('        const candidate = classify(url);\n        if (emit(candidate, `location.${method}`))', '        const candidate = classify(url) || forceClassify(url);\n        if (emit(candidate, `location.${method}`))', 1)
p.write_text(s, encoding='utf-8')

# ---------------- Desktop UI organization ----------------
p = Path('apps/desktop/ui/index.html')
h = p.read_text(encoding='utf-8')
h = h.replace('<button data-page="matrix"><span>◆</span><b>Matrix Ultimate v3 AI</b><i id="matrix-alert" class="matrix-alert" hidden>0</i></button>', '<button data-page="themes"><span>◐</span><b data-i18n="themes">Themes</b></button>\n        <button data-page="language"><span>文</span><b data-i18n="language">Language</b></button>\n        <button data-page="matrix"><span>▤</span><b data-i18n="logs">Logs</b><i id="matrix-alert" class="matrix-alert" hidden>0</i></button>', 1)
h = re.sub(r'\n\s*<small class="matrix-powered"[^>]*>.*?</small>', '', h, count=1, flags=re.S)
h = re.sub(r'<select id="language" aria-label="Language">.*?</select\s*>\s*><button id="add"', '<button id="add"', h, count=1, flags=re.S)
# Replace Matrix visual page with clean Logs page; hidden compatibility nodes keep old JS harmless until backend cleanup.
h = re.sub(r'<section id="matrix-panel" class="matrix-panel" hidden>.*?</section>', '''<section id="themes-panel" class="customization-panel" hidden>
        <div class="customization-grid"><section class="customization-card"><header><div><b data-i18n="themes">Themes</b><small data-i18n="themesHint">Choose the visual style of Apocalipse.</small></div></header><label for="theme" data-i18n="appearanceTheme">Interface theme</label><select id="theme"><option value="void">Alien Void</option><option value="inferno">Inferno</option><option value="toxic">Toxic Reactor</option><option value="synthwave">Synthwave</option><option value="royal">Royal Nebula</option><option value="crimson">Blood Moon</option><option value="arctic">Arctic Signal</option><option value="obsidian">Obsidian Black</option><option value="monochrome">Monochrome</option><option value="midnight">Midnight Blue</option><option value="forest">Dark Forest</option><option value="graphite">Graphite Steel</option><option value="deepsea">Deep Sea</option><option value="eclipse">Purple Eclipse</option><option value="hazard">Hazard Black &amp; Yellow</option><option value="cyberstorm">Cyber Storm</option><option value="ultraviolet">Ultraviolet Pulse</option><option value="emeraldgold">Emerald &amp; Gold</option><option value="scarletice">Scarlet Ice</option><option value="coppernavy">Copper Navy</option><option value="matrixcode">Matrix Code</option><option value="solarizednight">Solarized Night</option></select></section>
        <section class="customization-card"><header><div><b data-i18n="customize">Customize</b><small data-i18n="customizeHint">Fine tune the interface.</small></div></header><label class="switch-row"><span><b data-i18n="windowTransparency">Window transparency</b><small data-i18n="windowTransparencyHint">Soft translucency.</small></span><input id="window-transparency" type="checkbox" /></label><label class="range-title" for="transparency-level"><span data-i18n="transparencyLevel">Transparency level</span><output id="transparency-level-value">12%</output></label><input id="transparency-level" type="range" min="0" max="35" value="12" /><label class="switch-row"><span><b data-i18n="roundedCorners">Rounded corners</b><small data-i18n="roundedCornersHint">Rounded panels and controls.</small></span><input id="rounded-corners" type="checkbox" checked /></label><label class="range-title" for="corner-radius"><span data-i18n="cornerRadius">Corner radius</span><output id="corner-radius-value">14 px</output></label><input id="corner-radius" type="range" min="0" max="28" value="14" /><label class="range-title" for="interface-scale"><span data-i18n="interfaceSize">Interface size</span><output id="interface-scale-value">100%</output></label><input id="interface-scale" type="range" min="85" max="125" step="5" value="100" /></section></div></section>
      <section id="language-panel" class="customization-panel" hidden><section class="customization-card language-card"><header><div><b data-i18n="language">Language</b><small data-i18n="languageHint">Applies to the app and tray menu.</small></div></header><div class="language-choices"><label><input type="radio" name="ui-language" value="pt-BR" /><span><b>Português (Brasil)</b><small>Português brasileiro</small></span></label><label><input type="radio" name="ui-language" value="en" /><span><b>English</b><small>English</small></span></label><label><input type="radio" name="ui-language" value="zh-CN" /><span><b>简体中文</b><small>简体中文</small></span></label></div><select id="language" hidden><option value="en">English</option><option value="pt-BR">Português (Brasil)</option><option value="zh-CN">简体中文</option></select></section></section>
      <section id="matrix-panel" class="matrix-panel logs-panel" hidden><header><div><b data-i18n="logs">Logs</b><small data-i18n="logsHint">Powerful capture diagnostics.</small></div><span class="matrix-actions"><button id="export-diagnostics" type="button" data-i18n="exportDiagnostics">Export diagnostics</button><button id="clear-internal-logs" type="button" data-i18n="clearInternalLogs">Clear internal logs</button></span></header><p data-i18n="logsDescription">Export diagnostics after a failed capture to inspect the complete path quickly.</p><div class="logs-guide"><code>AUTO</code><span data-i18n="logsAuto">Automatic capture</span><code>FORCE</code><span data-i18n="logsForce">Forced capture</span><code>BYPASS</code><span data-i18n="logsBypass">Browser bypass</span></div><small id="matrix-summary" hidden></small><div id="matrix-proposals" hidden></div><button id="matrix-import" hidden></button><button id="matrix-export" hidden></button><button id="matrix-scan" hidden></button></section>''', h, count=1, flags=re.S)
# Remove appearance, site-rules and diagnostics from Settings. They now live in Themes/Logs.
h = re.sub(r'<section class="pairing-settings theme-settings">.*?</section>', '', h, count=1, flags=re.S)
h = re.sub(r'<section class="pairing-settings">\s*<b data-i18n="siteRules">.*?</section>', '', h, count=1, flags=re.S)
h = re.sub(r'<section class="pairing-settings">\s*<b data-i18n="diagnostics">.*?</section>', '', h, count=1, flags=re.S)
p.write_text(h, encoding='utf-8')

# CSS for dedicated customization pages and true soft transparency.
p = Path('apps/desktop/ui/styles.css')
s = p.read_text(encoding='utf-8')
s = s.replace('  --accent-2: #45a9ff;\n}', '  --accent-2: #45a9ff;\n  --window-alpha: 100%;\n  --corner-radius: 12px;\n  --ui-scale: 1;\n}', 1)
s = s.replace('  background: #070a0f;\n', '  background: transparent;\n', 1)
s = s.replace('    radial-gradient(circle at 75% 0, var(--app-glow) 0, transparent 32%), var(--app-bg);', '    radial-gradient(circle at 75% 0, color-mix(in srgb, var(--app-glow) var(--window-alpha), transparent) 0, transparent 32%), color-mix(in srgb, var(--app-bg) var(--window-alpha), transparent);\n  zoom: var(--ui-scale);')
s += '''
.customization-panel { flex:1; min-height:0; }
.customization-grid { display:grid; grid-template-columns:minmax(280px,1fr) minmax(340px,1fr); gap:18px; }
.customization-card { border:1px solid var(--line); background:color-mix(in srgb,var(--surface) 92%,transparent); border-radius:var(--corner-radius); padding:22px; display:grid; gap:16px; }
.customization-card header { margin:0 0 4px; align-items:start; }
.customization-card header b { font-size:18px; }
.customization-card header small,.switch-row small { display:block; color:var(--muted); margin-top:5px; }
.customization-card select { width:100%; background:var(--input); color:var(--text); border:1px solid var(--line); border-radius:var(--corner-radius); padding:11px; }
.switch-row,.language-choices label { display:flex; align-items:center; justify-content:space-between; gap:20px; border:1px solid var(--line); border-radius:var(--corner-radius); background:color-mix(in srgb,var(--surface-2) 88%,transparent); padding:14px 16px; }
.switch-row input { width:20px; height:20px; }
.range-title { display:flex; justify-content:space-between; align-items:center; color:var(--text); }
.customization-card input[type=range] { width:100%; accent-color:var(--accent); }
.language-card { max-width:760px; }
.language-choices { display:grid; gap:12px; }
.language-choices label { justify-content:flex-start; cursor:pointer; }
.language-choices input { width:20px; height:20px; }
.language-choices span { display:grid; gap:3px; }
.language-choices small { color:var(--muted); }
.logs-panel { border:1px solid var(--line); border-radius:var(--corner-radius); background:color-mix(in srgb,var(--surface) 92%,transparent); padding:22px; }
.logs-guide { display:grid; grid-template-columns:auto 1fr; gap:10px 14px; align-items:center; margin-top:18px; }
.logs-guide code { color:var(--accent); border:1px solid var(--line); border-radius:8px; padding:5px 8px; background:var(--input); }
:root[data-rounded="false"] { --corner-radius:0px; }
nav button,.panel,.matrix-panel,.link-panel>*,dialog,.pairing-settings,.metrics article,input,select,button { border-radius:var(--corner-radius); }
@media(max-width:900px){.customization-grid{grid-template-columns:1fr;}}
'''
p.write_text(s, encoding='utf-8')

# App JS translations, page switching, dedicated settings behavior.
p = Path('apps/desktop/ui/app.js')
s = p.read_text(encoding='utf-8')
insert = '''
Object.assign(catalogs.en,{themes:"Themes",language:"Language",logs:"Logs",themesHint:"Choose the visual style of Apocalipse.",customize:"Customize",customizeHint:"Fine tune the interface.",windowTransparency:"Window transparency",windowTransparencyHint:"Soft translucency.",transparencyLevel:"Transparency level",roundedCorners:"Rounded corners",roundedCornersHint:"Rounded panels and controls.",cornerRadius:"Corner radius",interfaceSize:"Interface size",languageHint:"Applies to the application and tray menu.",logsHint:"Powerful diagnostics for capture decisions.",logsDescription:"Export diagnostics after a failed capture to inspect the complete click → detection → bridge → download path.",logsAuto:"Automatic capture",logsForce:"Forced capture shortcut",logsBypass:"Browser bypass shortcut",themesPageDescription:"Themes, transparency, rounded corners and interface size.",languagePageDescription:"Choose the language used by Apocalipse and the tray menu.",logsPageDescription:"Powerful diagnostics for quickly locating capture and download failures."});
Object.assign(catalogs["pt-BR"],{themes:"Temas",language:"Idioma",logs:"Logs",themesHint:"Escolha o estilo visual do Apocalipse.",customize:"Personalizar",customizeHint:"Ajuste a interface.",windowTransparency:"Transparência da janela",windowTransparencyHint:"Transparência suave.",transparencyLevel:"Nível de transparência",roundedCorners:"Cantos arredondados",roundedCornersHint:"Cantos arredondados nos painéis e controles.",cornerRadius:"Raio dos cantos",interfaceSize:"Tamanho da interface",languageHint:"Aplica-se ao aplicativo e ao menu do tray.",logsHint:"Diagnóstico poderoso das decisões de captura.",logsDescription:"Exporte o diagnóstico após uma falha para analisar rapidamente todo o caminho clique → detecção → bridge → download.",logsAuto:"Captura automática",logsForce:"Captura forçada por atalho",logsBypass:"Ignorar e usar o navegador",themesPageDescription:"Temas, transparência, cantos arredondados e tamanho da interface.",languagePageDescription:"Escolha o idioma usado pelo Apocalipse e pelo menu do tray.",logsPageDescription:"Diagnóstico poderoso para localizar rapidamente falhas de captura e download."});
Object.assign(catalogs["zh-CN"],{themes:"主题",language:"语言",logs:"日志",themesHint:"选择 Apocalipse 的视觉风格。",customize:"自定义",customizeHint:"微调界面。",windowTransparency:"窗口透明度",windowTransparencyHint:"柔和透明效果。",transparencyLevel:"透明度级别",roundedCorners:"圆角",roundedCornersHint:"为面板和控件应用圆角。",cornerRadius:"圆角半径",interfaceSize:"界面大小",languageHint:"应用于程序和托盘菜单。",logsHint:"强大的捕获诊断。",logsDescription:"捕获失败后导出诊断，以快速检查完整路径。",logsAuto:"自动捕获",logsForce:"快捷键强制捕获",logsBypass:"浏览器绕过",themesPageDescription:"主题、透明度、圆角和界面大小。",languagePageDescription:"选择 Apocalipse 和托盘菜单的语言。",logsPageDescription:"强大的诊断，可快速定位捕获和下载故障。"});
'''
s = s.replace('\nlet locale = localStorage.getItem("apocalipse.language") || "en";', insert + '\nlet locale = localStorage.getItem("apocalipse.language") || "en";', 1)
# appearance helpers
marker = 'applyTheme(localStorage.getItem("apocalipse.theme") || "void");\n'
appearance = '''applyTheme(localStorage.getItem("apocalipse.theme") || "void");
const applyAppearance = () => {
  const enabled = localStorage.getItem("apocalipse.transparency.enabled") === "true";
  const level = Math.max(0, Math.min(35, Number(localStorage.getItem("apocalipse.transparency.level") || 12)));
  const rounded = localStorage.getItem("apocalipse.rounded") !== "false";
  const radius = Math.max(0, Math.min(28, Number(localStorage.getItem("apocalipse.corner.radius") || 14)));
  const scale = Math.max(85, Math.min(125, Number(localStorage.getItem("apocalipse.ui.scale") || 100)));
  document.documentElement.style.setProperty("--window-alpha", `${enabled ? 100-level : 100}%`);
  document.documentElement.dataset.rounded = String(rounded);
  document.documentElement.style.setProperty("--corner-radius", `${rounded ? radius : 0}px`);
  document.documentElement.style.setProperty("--ui-scale", String(scale/100));
};
applyAppearance();
'''
s = s.replace(marker, appearance, 1)
# descriptions maps
s = s.replace('matrix: "matrixPageDescription"', 'matrix: "logsPageDescription", themes: "themesPageDescription", language: "languagePageDescription"')
# page switching panels/hiding
s = s.replace('document.querySelector("#matrix-panel").hidden = activePage !== "matrix";\n    document.querySelector(".metrics").hidden = ["link", "matrix"].includes(activePage);\n    document.querySelector(".panel").hidden = ["link", "matrix"].includes(activePage);', 'document.querySelector("#matrix-panel").hidden = activePage !== "matrix";\n    document.querySelector("#themes-panel").hidden = activePage !== "themes";\n    document.querySelector("#language-panel").hidden = activePage !== "language";\n    document.querySelector(".metrics").hidden = ["link", "matrix", "themes", "language"].includes(activePage);\n    document.querySelector(".panel").hidden = ["link", "matrix", "themes", "language"].includes(activePage);', 1)
# language change persists to tray backend too
s = s.replace('  localStorage.setItem("apocalipse.language", locale);\n  translate();', '  localStorage.setItem("apocalipse.language", locale);\n  document.querySelectorAll(\'input[name="ui-language"]\').forEach((input)=>input.checked=input.value===locale);\n  invoke("set_ui_locale", { locale }).catch(console.error);\n  translate();', 1)
# old settings references to #theme: keep Settings independent from appearance
s = s.replace('    document.querySelector("#theme").value = document.documentElement.dataset.theme;\n', '', 1)
s = s.replace('    applyTheme(localStorage.getItem("apocalipse.theme") || "void");\n    settingsDialog.close();', '    settingsDialog.close();', 1)
s = s.replace('document.querySelector("#theme").onchange = (event) => applyTheme(event.target.value);\n', '', 1)
s = s.replace('    const theme = document.querySelector("#theme").value;\n    localStorage.setItem("apocalipse.theme", theme);\n', '', 1)
s = s.replace('    applyTheme(theme);\n', '', 1)
# disable Matrix polling/AI behavior but keep hidden compatibility nodes safe.
s = s.replace('if (activePage === "matrix") refreshMatrix().catch(console.error);', '')
s = s.replace('document.querySelector(\'[data-page="matrix"]\').addEventListener("click", () => refreshMatrix().catch(console.error));', '')
s = s.replace('refreshMatrix().catch(console.error);\nsetInterval(() => refreshMatrix().catch(console.error), 5000);', '')
# site rules UI handlers can stay only if nodes exist; guard them after UI removal.
s = s.replace('document.querySelector("#manage-site-rules").onclick = async () => {', 'if (document.querySelector("#manage-site-rules")) document.querySelector("#manage-site-rules").onclick = async () => {')
s = s.replace('document.querySelector("#save-site-rules").onclick = async () => {', 'if (document.querySelector("#save-site-rules")) document.querySelector("#save-site-rules").onclick = async () => {')
s = s.replace('document.querySelector("#reset-site-rules").onclick = async () => {', 'if (document.querySelector("#reset-site-rules")) document.querySelector("#reset-site-rules").onclick = async () => {')
# log editor nodes removed from Settings, so guard their handlers.
for ident in ['#open-log','#clear-log','#refresh-log','#open-log-external','#pick-log-editor','#remove-log-editor']:
    s = s.replace(f'document.querySelector("{ident}").onclick =', f'if (document.querySelector("{ident}")) document.querySelector("{ident}").onclick =')
# Dedicated Themes interactions.
s += '''
const themeSelect = document.querySelector("#theme");
if (themeSelect) {
  themeSelect.value = document.documentElement.dataset.theme;
  themeSelect.onchange = (event) => { localStorage.setItem("apocalipse.theme", event.target.value); applyTheme(event.target.value); };
}
const appearanceInputs = ["window-transparency","transparency-level","rounded-corners","corner-radius","interface-scale"];
function syncAppearanceControls(){
  const enabled=localStorage.getItem("apocalipse.transparency.enabled")==="true", level=Number(localStorage.getItem("apocalipse.transparency.level")||12), rounded=localStorage.getItem("apocalipse.rounded")!=="false", radius=Number(localStorage.getItem("apocalipse.corner.radius")||14), scale=Number(localStorage.getItem("apocalipse.ui.scale")||100);
  document.querySelector("#window-transparency").checked=enabled; document.querySelector("#transparency-level").value=level; document.querySelector("#rounded-corners").checked=rounded; document.querySelector("#corner-radius").value=radius; document.querySelector("#interface-scale").value=scale;
  document.querySelector("#transparency-level-value").value=`${level}%`; document.querySelector("#corner-radius-value").value=`${radius} px`; document.querySelector("#interface-scale-value").value=`${scale}%`;
}
function saveAppearanceControls(){ localStorage.setItem("apocalipse.transparency.enabled",String(document.querySelector("#window-transparency").checked)); localStorage.setItem("apocalipse.transparency.level",document.querySelector("#transparency-level").value); localStorage.setItem("apocalipse.rounded",String(document.querySelector("#rounded-corners").checked)); localStorage.setItem("apocalipse.corner.radius",document.querySelector("#corner-radius").value); localStorage.setItem("apocalipse.ui.scale",document.querySelector("#interface-scale").value); syncAppearanceControls(); applyAppearance(); }
appearanceInputs.forEach((id)=>{ const el=document.getElementById(id); if(el) el.oninput=saveAppearanceControls; });
syncAppearanceControls();
document.querySelectorAll('input[name="ui-language"]').forEach((input)=>{ input.checked=input.value===locale; input.onchange=()=>{ if(!input.checked)return; locale=input.value; document.querySelector("#language").value=locale; localStorage.setItem("apocalipse.language",locale); invoke("set_ui_locale",{locale}).catch(console.error); translate(); }; });
'''
p.write_text(s, encoding='utf-8')

# ---------------- Tauri backend: localized tray + no persistent site-rules file ----------------
p = Path('apps/desktop/src-tauri/src/main.rs')
s = p.read_text(encoding='utf-8')
s = s.replace('    link_password: String,\n}', '    link_password: String,\n    #[serde(default = "default_ui_locale")]\n    ui_locale: String,\n}', 1)
s = s.replace('fn default_link_password() -> String {', 'fn default_ui_locale() -> String { "en".to_owned() }\nfn default_link_password() -> String {', 1)
s = s.replace('            link_password: default_link_password(),\n', '            link_password: default_link_password(),\n            ui_locale: default_ui_locale(),\n', 1)
# command to persist locale and rebuild tray immediately
anchor = '#[tauri::command]\nfn get_bridge_pairing'
cmd = '''fn tray_labels(locale: &str) -> (&'static str, &'static str) {
    match locale { "pt-BR" => ("Mostrar Apocalipse", "Sair"), "zh-CN" => ("显示 Apocalipse", "退出"), _ => ("Show Apocalipse", "Quit") }
}

#[tauri::command]
fn set_ui_locale(app: tauri::AppHandle, state: State<'_, AppState>, locale: String) -> Result<(), String> {
    if !matches!(locale.as_str(), "en" | "pt-BR" | "zh-CN") { return Err("invalid_locale".to_owned()); }
    { let mut settings = state.settings.lock().map_err(|e| e.to_string())?; settings.ui_locale = locale.clone(); save_settings(&state, &settings)?; }
    let (show_text, quit_text) = tray_labels(&locale);
    let show = MenuItem::with_id(&app, "show", show_text, true, None::<&str>).map_err(|e| e.to_string())?;
    let quit = MenuItem::with_id(&app, "quit", quit_text, true, None::<&str>).map_err(|e| e.to_string())?;
    let menu = Menu::with_items(&app, &[&show, &quit]).map_err(|e| e.to_string())?;
    if let Some(tray) = app.tray_by_id("main-tray") { tray.set_menu(Some(menu)).map_err(|e| e.to_string())?; }
    Ok(())
}

#[tauri::command]
fn get_bridge_pairing'''
s = s.replace(anchor, cmd, 1)
# Stop persisting site-rules.json; remove stale file, retain in-memory compatibility until shortcut engine proves all cases.
s = s.replace('            let site_rules_path = app_data.join("site-rules.json");\n            let initial_site_rules = load_site_rules(&site_rules_path);\n            if !site_rules_path.exists() {\n                let data = serde_json::to_vec_pretty(&initial_site_rules)?;\n                fs::write(&site_rules_path, data)?;\n            }', '            let site_rules_path = app_data.join("site-rules.json");\n            let _ = fs::remove_file(&site_rules_path);\n            let initial_site_rules = default_site_rules();', 1)
s = s.replace('            app.manage(AppState {', '            let tray_locale = initial_settings.ui_locale.clone();\n            app.manage(AppState {', 1)
s = s.replace('            let show = MenuItem::with_id(app, "show", "Show Apocalipse", true, None::<&str>)?;\n            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;', '            let (show_text, quit_text) = tray_labels(&tray_locale);\n            let show = MenuItem::with_id(app, "show", show_text, true, None::<&str>)?;\n            let quit = MenuItem::with_id(app, "quit", quit_text, true, None::<&str>)?;', 1)
s = s.replace('            TrayIconBuilder::new()\n', '            TrayIconBuilder::with_id("main-tray")\n', 1)
s = s.replace('            get_bridge_pairing,\n', '            get_bridge_pairing,\n            set_ui_locale,\n', 1)
p.write_text(s, encoding='utf-8')

# Native window needs transparent backing for the user-controlled soft translucency.
p = Path('apps/desktop/src-tauri/tauri.conf.json')
conf = json.loads(p.read_text(encoding='utf-8'))
conf['app']['windows'][0]['transparent'] = True
p.write_text(json.dumps(conf, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

# Version bump and hard no-CDP assertion.
p = Path('browser-extension/manifest.json')
manifest = json.loads(p.read_text(encoding='utf-8'))
manifest['version'] = '0.3.50'
manifest['permissions'] = [x for x in manifest.get('permissions', []) if x != 'debugger']
p.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
for path in Path('browser-extension').glob('*.js'):
    text = path.read_text(encoding='utf-8')
    for forbidden in ['chrome.debugger','Fetch.enable','Fetch.requestPaused','Fetch.takeResponseBodyAsStream','IO.read','Page.setDownloadBehavior','Browser.setDownloadBehavior']:
        if forbidden in text: raise SystemExit(f'forbidden CDP token {forbidden} in {path}')
