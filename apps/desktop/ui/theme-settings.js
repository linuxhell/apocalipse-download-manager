(() => {
  const names = {
    en: { title: "Themes", hint: "Choose a complete palette. The browser extension follows automatically.", saved: "Theme saved", saving: "Saving theme…", failed: "Could not save the theme. Your previous theme was restored.", preview: "Download", light: "Light palette", dark: "Dark palette" },
    "pt-BR": { title: "Temas", hint: "Escolha uma paleta completa. A extensão do navegador acompanha automaticamente.", saved: "Tema salvo", saving: "Salvando tema…", failed: "Não foi possível salvar o tema. O tema anterior foi restaurado.", preview: "Baixar", light: "Paleta clara", dark: "Paleta escura" },
    "zh-CN": { title: "主题", hint: "选择完整配色。浏览器扩展会自动同步。", saved: "主题已保存", saving: "正在保存主题…", failed: "无法保存主题，已恢复之前的主题。", preview: "下载", light: "浅色配色", dark: "深色配色" },
  };
  const invoke = (...args) => window.__TAURI__.core.invoke(...args);
  let saving = false;
  function render() {
    const grid = document.getElementById("theme-choices");
    if (!grid) return;
    const language = names[document.documentElement.lang] ? document.documentElement.lang : "en";
    const copy = names[language];
    document.querySelectorAll("[data-themes-label]").forEach(node => { node.textContent = copy.title; });
    document.getElementById("theme-description").textContent = copy.hint;
    grid.replaceChildren();
    for (const palette of ADM_THEMES.palettes) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "theme-choice";
      button.dataset.themeChoice = palette.id;
      button.disabled = saving;
      button.setAttribute("aria-pressed", String(document.documentElement.dataset.theme === palette.id));
      ADM_THEMES.apply(button, palette.id);
      const preview = document.createElement("span");
      preview.className = "theme-choice-preview";
      const line = document.createElement("span");
      line.className = "theme-preview-line";
      line.textContent = copy.preview;
      const folder = document.createElement("span");
      folder.className = "theme-preview-folder";
      folder.setAttribute("aria-hidden", "true");
      const bar = document.createElement("span");
      bar.className = "theme-preview-progress";
      preview.append(line, folder, bar);
      const name = document.createElement("b");
      name.textContent = palette.names[language];
      const mode = document.createElement("small");
      mode.textContent = palette.dark ? copy.dark : copy.light;
      button.append(preview, name, mode);
      button.onclick = async () => {
        if (saving || palette.id === document.documentElement.dataset.theme) return;
        const previous = document.documentElement.dataset.theme;
        saving = true;
        window.applyAdmTheme(palette.id);
        const status = document.getElementById("theme-save-status");
        status.textContent = copy.saving;
        try {
          await invoke("set_ui_theme", { theme: palette.id });
          status.textContent = copy.saved;
        } catch {
          window.applyAdmTheme(previous);
          status.textContent = copy.failed;
        } finally {
          saving = false;
          render();
        }
      };
      grid.append(button);
    }
  }
  window.renderAdmThemes = render;
  window.addEventListener("apocalipse-theme-applied", render);
  render();
})();
