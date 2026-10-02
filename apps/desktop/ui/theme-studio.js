(function (root) {
  const themes = [
    ['cyberpunk', 'Cyberpunk', '#25d9ef', ['Neon, chuva e cidade futurista', 'Neon, rain and a futuristic city', '霓虹、雨与未来城市']],
    ['bladerunner', 'Blade Runner', '#ffb347', ['Âmbar e névoa na metrópole', 'Amber and mist in the metropolis', '琥珀色与都市迷雾']],
    ['sexy', 'Sexy', '#f16a9a', ['Retrato cinematográfico em vinho', 'A cinematic portrait in burgundy', '酒红色电影肖像']],
    ['samurai', 'Samurai', '#ff596d', ['Templo, névoa e cerejeiras', 'Temple, mist and cherry blossoms', '寺庙、薄雾与樱花']],
    ['future', 'Futuro', '#69caff', ['Horizonte de uma nova era', 'The horizon of a new era', '新时代的地平线']],
    ['fantasy', 'Fantasia', '#77e8b0', ['Elfa e fada no bosque encantado', 'Elf and fairy in an enchanted forest', '魔法森林中的精灵与仙子']],
    ['pandora', 'Pandora', '#62ddf5', ['Montanhas flutuantes e cascatas', 'Floating mountains and waterfalls', '浮山与瀑布']],
    ['void', 'Alien Void'], ['nebula', 'Nebula'], ['ember', 'Ember'], ['jade', 'Jade'],
    ['plasma', 'Plasma'], ['glacier', 'Glacier'], ['amber', 'Amber'], ['abyss', 'Abyss'],
    ['rust', 'Rust'], ['venom', 'Venom'], ['wine', 'Wine'], ['linen', 'Linho'],
    ['sky', 'Céu'], ['blossom', 'Flor de Cerejeira'], ['sage', 'Sálvia'], ['sand', 'Areia'],
    ['lilac', 'Lilás'], ['mist', 'Neblina'], ['citrus', 'Cítrico'], ['coral', 'Coral'], ['frost', 'Geada'],
  ];
  const defaults = { transparencyEnabled: true, transparencyLevel: 65, roundedEnabled: true,
    cornerRadius: 12, interfaceSize: 'normal', backgroundBrightness: 75, aeroEnabled: true, animationsEnabled: true };
  function normalize(value = {}) {
    const input = { ...defaults, ...value };
    const number = (key, min, max) => Number.isFinite(Number(input[key]))
      ? Math.max(min, Math.min(max, Number(input[key]))) : defaults[key];
    return { ...input, transparencyLevel: number('transparencyLevel', 0, 70),
      cornerRadius: number('cornerRadius', 0, 28), backgroundBrightness: number('backgroundBrightness', 25, 100),
      interfaceSize: ['compact', 'normal', 'large'].includes(input.interfaceSize) ? input.interfaceSize : 'normal' };
  }
  const asset = (id, thumb = false) => `assets/themes/${id}${thumb ? '-thumb' : ''}.webp`;
  function applyPresentation(id, settings = defaults, target = document.documentElement) {
    const theme = themes.find(item => item[0] === id) || themes.find(item => item[0] === 'void');
    settings = normalize(settings);
    target.style.setProperty('--theme-background', `url("${asset(theme[0])}")`);
    target.style.setProperty('--background-brightness', settings.backgroundBrightness / 100);
    // Keep a dark glass backing even at maximum transparency, rather than fading text.
    const light = ['linen','sky','blossom','sage','sand','lilac','mist','citrus','coral','frost'].includes(theme[0]);
    const opacity = settings.transparencyEnabled ? Math.max(light ? 0.74 : 0.46, 0.92 - settings.transparencyLevel * 0.0065) : 0.98;
    target.style.setProperty('--glass-opacity', opacity.toFixed(3));
    target.style.setProperty('--glass-blur', settings.aeroEnabled ? '8px' : '0px');
    target.style.setProperty('--corner-radius', settings.roundedEnabled ? `${settings.cornerRadius}px` : '0px');
    target.dataset.animations = settings.animationsEnabled ? 'on' : 'off';
    target.dataset.scenic = 'on';
  }
  function init({ translate, language, applyTheme, applyAppearance, readAppearance, commit, downloads }) {
    const panel = document.querySelector('#themes-panel');
    const select = document.querySelector('#theme');
    const grid = document.querySelector('#theme-gallery');
    let selected = localStorage.getItem('apocalipse.theme') || 'void';
    let draft = normalize(readAppearance());
    let saving = false;
    const savedTheme = () => localStorage.getItem('apocalipse.theme') || 'void';
    const description = theme => theme[3]?.[language() === 'pt-BR' ? 0 : language() === 'zh-CN' ? 2 : 1] || translate('themeHint');
    const preview = () => {
      const theme = themes.find(item => item[0] === selected) || themes[7];
      const box = document.querySelector('#theme-live-preview');
      box.style.backgroundImage = `url("${asset(theme[0])}")`;
      box.style.setProperty('--preview-accent', getComputedStyle(document.documentElement).getPropertyValue('--accent'));
      box.style.setProperty('--glass-opacity', (draft.transparencyEnabled ? 0.92 - draft.transparencyLevel * 0.0065 : 0.98).toFixed(3));
      box.style.setProperty('--glass-blur', draft.aeroEnabled ? '5px' : '0px');
      document.querySelector('#theme-preview-name').textContent = theme[1];
      document.querySelector('#theme-preview-description').textContent = description(theme);
      // Preview uses the first actual task when available; a labelled example otherwise.
      const task = downloads()[0];
      document.querySelector('#theme-preview-file').textContent = task
        ? task.display_title || task.destination.split(/[\\/]/).pop() : translate('themeExampleFile');
      const percent = task ? Math.max(0, Math.min(100, task.progress_percent ?? (task.total ? task.received / task.total * 100 : 0))) : 56;
      document.querySelector('#theme-preview-progress').style.width = `${percent}%`;
      document.querySelector('#theme-preview-percent').textContent = `${Math.round(percent)}%`;
      document.querySelector('#theme-preview-example').hidden = Boolean(task);
      for (const node of grid.querySelectorAll('[data-theme-choice]')) {
        node.classList.toggle('selected', node.dataset.themeChoice === selected);
        node.setAttribute('aria-pressed', String(node.dataset.themeChoice === selected));
        node.classList.toggle('applied', node.dataset.themeChoice === savedTheme());
      }
    };
    function syncControls() {
      for (const [id, key] of [['transparency-enabled','transparencyEnabled'], ['rounded-enabled','roundedEnabled'], ['aero-enabled','aeroEnabled'], ['animations-enabled','animationsEnabled']]) panel.querySelector(`#${id}`).checked = draft[key];
      for (const [id, key] of [['transparency-level','transparencyLevel'], ['corner-radius','cornerRadius'], ['background-brightness','backgroundBrightness']]) panel.querySelector(`#${id}`).value = draft[key];
      panel.querySelector('#interface-size').value = draft.interfaceSize;
      panel.querySelector('#transparency-value').textContent = `${draft.transparencyLevel}%`;
      panel.querySelector('#corner-radius-value').textContent = `${draft.cornerRadius} px`;
      panel.querySelector('#brightness-value').textContent = `${draft.backgroundBrightness}%`;
      panel.querySelector('#transparency-level').disabled = !draft.transparencyEnabled;
      panel.querySelector('#corner-radius').disabled = !draft.roundedEnabled;
    }
    function showDraft() { select.value = selected; applyTheme(selected); applyAppearance(draft); syncControls(); preview(); }
    function render() {
      const query = panel.querySelector('#theme-search').value.trim().toLocaleLowerCase();
      grid.replaceChildren();
      for (const theme of themes) {
        if (!`${theme[1]} ${description(theme)}`.toLocaleLowerCase().includes(query)) continue;
        const card = document.createElement('button');
        card.type = 'button'; card.className = 'theme-card'; card.dataset.themeChoice = theme[0];
        card.setAttribute('aria-label', theme[1]);
        const img = new Image(); img.src = asset(theme[0], true); img.alt = ''; img.loading = 'lazy';
        const name = document.createElement('strong'); name.textContent = theme[1];
        const check = document.createElement('span'); check.className = 'theme-check'; check.textContent = '✓'; check.setAttribute('aria-hidden', 'true');
        card.append(img, name, check); card.onclick = () => { selected = theme[0]; showDraft(); };
        grid.append(card);
      }
      panel.querySelector('#theme-count').textContent = `${themes.length} ${translate('themes').toLocaleLowerCase()}`;
      panel.querySelector('#theme-no-results').hidden = grid.childElementCount > 0;
      preview();
    }
    function rollback() { selected = savedTheme(); draft = normalize(readAppearance()); showDraft(); }
    panel.querySelector('#theme-search').oninput = render;
    select.onchange = () => { selected = select.value; showDraft(); };
    for (const node of panel.querySelectorAll('.appearance-options input, .appearance-options select')) {
      node.oninput = node.onchange = () => {
        draft = normalize({
          transparencyEnabled: panel.querySelector('#transparency-enabled').checked,
          transparencyLevel: Number(panel.querySelector('#transparency-level').value),
          roundedEnabled: panel.querySelector('#rounded-enabled').checked,
          cornerRadius: Number(panel.querySelector('#corner-radius').value),
          interfaceSize: panel.querySelector('#interface-size').value,
          backgroundBrightness: Number(panel.querySelector('#background-brightness').value),
          aeroEnabled: panel.querySelector('#aero-enabled').checked,
          animationsEnabled: panel.querySelector('#animations-enabled').checked,
        }); showDraft();
      };
    }
    panel.querySelector('#theme-reset').onclick = () => { selected = 'void'; draft = { ...defaults }; showDraft(); };
    panel.querySelector('#theme-cancel').onclick = () => { rollback(); panel.querySelector('#theme-save-status').textContent = ''; };
    panel.querySelector('#theme-apply').onclick = async () => {
      if (saving) return;
      saving = true; panel.querySelector('#theme-apply').disabled = true;
      const theme = selected, appearance = { ...draft };
      try {
        await commit(theme, appearance);
        // A successful commit updates the applied badge but does not discard a newer draft.
        panel.querySelector('#theme-save-status').textContent = translate('themeApplied');
        preview();
      } catch (error) { panel.querySelector('#theme-save-status').textContent = `${translate('themeSaveFailed')}: ${error}`; }
      finally { saving = false; panel.querySelector('#theme-apply').disabled = false; }
    };
    syncControls(); render();
    return { refresh: render, open: () => { rollback(); render(); }, leave: rollback };
  }
  root.ThemeStudio = { themes, defaults, normalize, asset, applyPresentation, init };
  if (typeof module !== 'undefined') module.exports = root.ThemeStudio;
})(typeof window !== 'undefined' ? window : globalThis);
