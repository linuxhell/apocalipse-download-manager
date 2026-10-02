(() => {
  const encoded = globalThis.__ADM_THEME_SCENE_B64 || "";
  if (!encoded) return;
  document.documentElement.style.setProperty(
    "--theme-scene-sprite",
    `url("data:image/webp;base64,${encoded}")`
  );
  delete globalThis.__ADM_THEME_SCENE_B64;
})();
