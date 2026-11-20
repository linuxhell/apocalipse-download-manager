// Apply a cached allowlisted palette before paint; settings.json remains authoritative.
window.applyAdmTheme = (theme) => {
  const palette = globalThis.ADM_THEMES?.apply(document.documentElement, theme)
    || { id: theme === "black" ? "black" : "white", dark: theme === "black" };
  document.documentElement.dataset.mode = palette.dark ? "dark" : "light";
  try {
    localStorage.setItem("apocalipse.theme", palette.id);
    localStorage.setItem("apocalipse.mode", palette.dark ? "dark" : "light");
  } catch {}
  window.dispatchEvent?.(new Event("apocalipse-theme-applied"));
};
try {
  window.applyAdmTheme(localStorage.getItem("apocalipse.theme") || (localStorage.getItem("apocalipse.mode") === "dark" ? "black" : "white"));
} catch { window.applyAdmTheme("white"); }
if (window.__TAURI__?.event?.listen) {
  // Register before reading settings so an older response cannot undo a live change.
  let revision = 0;
  const subscription = window.__TAURI__.event.listen("apocalipse-theme-changed", event => {
    revision++;
    window.applyAdmTheme(event.payload);
  });
  subscription.then(() => {
    const requestedRevision = revision;
    return window.__TAURI__.core.invoke("get_ui_theme").then(theme => {
      if (requestedRevision === revision) window.applyAdmTheme(theme);
    });
  }).catch(() => {});
}
// The main window is created invisible; show it once the first frame is painted
// so there is no blank white flash (hidden autostart never shows it). The
// Independent Link and ED2K/Kad windows must not trigger it.
window.addEventListener("load", () => {
  if (document.body.classList.contains("link-window") || document.body.classList.contains("ed2k-window")) return;
  requestAnimationFrame(() => requestAnimationFrame(() => {
    try { window.__TAURI__?.core?.invoke("main_window_ready").catch(() => {}); } catch {}
  }));
});
