// Applied before the first paint: the saved colour mode (light is the default).
try { document.documentElement.dataset.mode = localStorage.getItem("apocalipse.mode") === "dark" ? "dark" : "light"; } catch { document.documentElement.dataset.mode = "light"; }
// The main window is created invisible; show it once the first frame is painted
// so there is no blank white flash (hidden autostart never shows it). The
// Apocalipse Link window shares this file but must not trigger it.
window.addEventListener("load", () => {
  if (document.body.classList.contains("link-window")) return;
  requestAnimationFrame(() => requestAnimationFrame(() => {
    try { window.__TAURI__?.core?.invoke("main_window_ready").catch(() => {}); } catch {}
  }));
});
