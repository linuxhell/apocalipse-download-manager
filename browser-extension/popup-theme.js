(() => {
  function apply(theme) {
    const palette = ADM_THEMES.apply(document.documentElement, theme, true);
    try { localStorage.setItem("apocalipse.theme", palette.id); } catch {}
    return palette.id;
  }
  try { apply(localStorage.getItem("apocalipse.theme")); } catch { apply("white"); }
  chrome.storage.local.get({ desktopTheme: "white" }, value => apply(value.desktopTheme));
  chrome.storage.onChanged.addListener((changes, area) => {
    if (area === "local" && changes.desktopTheme) apply(changes.desktopTheme.newValue);
  });
  globalThis.syncPopupDesktopTheme = async appearance => {
    // An older paired desktop has no theme field. Retain the last known choice.
    if (typeof appearance?.theme !== "string") return;
    const theme = apply(appearance.theme);
    const stored = await chrome.storage.local.get({ desktopTheme: "white" });
    if (stored.desktopTheme !== theme) await chrome.storage.local.set({ desktopTheme: theme });
  };
})();
