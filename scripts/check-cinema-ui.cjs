// Headless integration checks. The Tauri bridge is mocked only for UI QA.
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const http = require('node:http');
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const root = path.resolve(__dirname, '../apps/desktop/ui');
const output = path.resolve(process.env.CINEMA_QA_OUTPUT || 'dist/cinema-ui');
fs.mkdirSync(output, { recursive: true });
const types = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.webp': 'image/webp', '.png': 'image/png' };
const server = http.createServer((request, response) => {
  const file = path.resolve(root, '.' + decodeURIComponent(request.url.split('?')[0] === '/' ? '/index.html' : request.url.split('?')[0]));
  if (!file.startsWith(root + path.sep) || !fs.existsSync(file)) { response.writeHead(404); response.end(); return; }
  response.setHeader('Content-Type', types[path.extname(file)] || 'application/octet-stream');
  fs.createReadStream(file).pipe(response);
});
(async () => {
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: { width: 1600, height: 1000 } });
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.addInitScript(() => {
      localStorage.setItem('apocalipse.language', 'pt-BR');
      localStorage.setItem('apocalipse.theme', 'cyberpunk');
      localStorage.removeItem('apocalipse.appearance');
      window.qaCommands = [];
      window.__TAURI__ = { event: { listen: async () => () => {} }, core: { invoke: async (command, args) => {
        window.qaCommands.push({ command, args });
        if (command === 'list_downloads') return [{ id: 'qa-task', source: 'https://example.test/windows.iso', destination: 'D:\\Downloads\\Windows_11.iso',
          state: 'downloading', received: 3640000000, total: 6500000000, download_speed: 42300000,
          progress_percent: 56, resume_supported: true, connections_override: 16 },
          { id: 'qa-torrent', source: 'magnet:?xt=urn:btih:0123456789012345678901234567890123456789', destination: 'D:\\Downloads\\Movie.mkv',
            state: 'downloading', received: 3640000000, total: 6500000000, download_speed: 42300000, progress_percent: 56, resume_supported: true },
          { id: 'qa-recording', source: 'https://example.test/live', destination: 'D:\\Downloads\\Live.recording.webm',
            state: 'downloading', received: 3640000000, total: 6500000000, download_speed: 42300000, progress_percent: 56, resume_supported: true }].flatMap(task => Array.from({ length: 8 }, (_, index) => ({ ...task, id: `${task.id}-${index}` })));
        if (command === 'can_preview_download') return args.id.startsWith('qa-torrent') || args.id.startsWith('qa-recording');
        if (command === 'get_app_version') return '0.4.83';
        if (command === 'default_download_directory') return 'D:\\Downloads';
        if (command === 'get_application_theme') return 'cyberpunk';
        if (command === 'get_bridge_pairing') return { connected: false, paired: false };
        if (command === 'read_general_log') return '';
        if (command === 'get_about_background') return '';
        if (command === 'get_about_media') return {};
        if (command === 'diagnostics_status') return {};
        if (command.startsWith('take_') || command === 'read_clipboard_link') return null;
        return null;
      } } };
    });
    await page.goto(`http://127.0.0.1:${server.address().port}/`);
    await page.locator('.download-row').first().waitFor();
    await page.screenshot({ path: path.join(output, 'downloads-cyberpunk.png'), fullPage: true });
    await page.locator('nav [data-page="themes"]').click();
    await page.locator('#theme-gallery .theme-card').first().waitFor();
    assert.equal(await page.locator('#theme-gallery .theme-card').count(), 28);
    await page.locator('[data-theme-choice="samurai"]').click();
    assert.equal(await page.evaluate(() => localStorage.getItem('apocalipse.theme')), 'cyberpunk');
    await page.locator('#theme-cancel').click();
    assert.equal(await page.evaluate(() => document.documentElement.dataset.theme), 'cyberpunk');
    await page.locator('[data-theme-choice="fantasy"]').click();
    await page.locator('#theme-apply').click();
    await page.waitForFunction(() => localStorage.getItem('apocalipse.theme') === 'fantasy');
    await page.locator('[data-theme-choice="cyberpunk"]').click();
    await page.evaluate(() => { document.querySelector("main").scrollTop = 0; document.querySelector(".theme-detail-column").scrollTop = 0; });
    await page.screenshot({ path: path.join(output, 'themes-cinema-premium.png'), fullPage: true });
    await page.locator('#theme-search').fill('samurai');
    assert.equal(await page.locator('#theme-gallery .theme-card').count(), 1);
    await page.locator('#theme-search').fill('');
    // Slider changes the glass backing, not text opacity; native controls remain fixed colors.
    await page.locator('#transparency-level').fill('70');
    const presentation = await page.evaluate(() => ({
      glass: getComputedStyle(document.querySelector('aside')).backgroundColor,
      text: getComputedStyle(document.querySelector('h1')).opacity,
      buttons: [...document.querySelectorAll('.window-controls button')].map(node => ({
        background: getComputedStyle(node).backgroundColor, color: getComputedStyle(node).color,
      })),
    }));
    assert.equal(presentation.text, '1');
    for (const button of presentation.buttons) {
      assert.equal(button.background, 'rgb(5, 5, 5)'); assert.equal(button.color, 'rgb(255, 255, 255)');
    }
    for (const action of ['minimize', 'maximize', 'close']) await page.locator(`[data-window-action="${action}"]`).click();
    await page.locator('#window-drag-region').dispatchEvent('mousedown', { button: 0, detail: 1 });
    const actions = await page.evaluate(() => window.qaCommands.filter(item => item.command === 'control_main_window').map(item => item.args.action));
    for (const action of ['minimize', 'maximize', 'close', 'drag']) assert.ok(actions.includes(action), action);
    await page.locator('[data-theme-choice="cyberpunk"]').click();
    await page.locator('#theme-apply').click();
    await page.waitForFunction(() => localStorage.getItem('apocalipse.theme') === 'cyberpunk');
    // Queue controls and all three task types leave the right-hand scenery exposed.
    await page.setViewportSize({ width: 1280, height: 850 });
    for (const section of ['downloads', 'torrents', 'recordings']) {
      await page.locator(`nav [data-page="${section}"]`).click();
      await page.locator('.download-row').first().waitFor();
      const compact = await page.evaluate(() => {
        const main = document.querySelector('main').getBoundingClientRect();
        const panel = document.querySelector('main > .panel').getBoundingClientRect();
        return { ratio: panel.width / (main.width - 40), bars: [...document.querySelectorAll('.task-progress')].map(n => n.getBoundingClientRect().width) };
      });
      assert.ok(compact.ratio >= .95, `${section} uses the full available width`);
      const queue = await page.evaluate(() => {
        const list = document.querySelector('#download-list');
        const widths = ['main > .panel', '.metrics', 'footer'].map(selector => document.querySelector(selector).getBoundingClientRect().width);
        list.scrollTop = 100;
        renderDownloads(true);
        return { widths, scrollHeight: list.scrollHeight, height: list.clientHeight, scrollTop: list.scrollTop, overflow: getComputedStyle(list).overflowY };
      });
      assert.ok(queue.widths.every(width => Math.abs(width - queue.widths[0]) < 2), `${section} uses matching panel widths`);
      assert.ok(queue.scrollHeight > queue.height && queue.overflow === 'auto', `${section} scrolls inside its list`);
      assert.equal(queue.scrollTop, 100, `${section} preserves scroll when progress refreshes`);
      await page.locator('#download-list').evaluate(node => { node.scrollTop = 0; });
      const folder = page.locator('.task-action[data-command="reveal_download"]').first();
      const taskId = await folder.evaluate(node => node.closest('.download-row').dataset.taskId);
      await folder.click();
      assert.equal(await page.evaluate(() => window.qaCommands.filter(item => item.command === 'reveal_download').at(-1).args.id), taskId);
      const details = page.locator('.task-details').first();
      await details.locator('summary').click();
      await page.evaluate(() => renderDownloads(true));
      assert.equal(await details.evaluate(node => node.open), true);
      await details.locator('summary').click();
      await page.locator('.task-remove-action').first().click();
      assert.equal(await page.locator('#clear-dialog').evaluate(node => node.open), true);
      await page.locator('#clear-dialog').evaluate(node => node.close());
      assert.ok(compact.bars.every(width => width > 100), `${section} has compact progress bars`);
      await page.screenshot({ path: path.join(output, `${section}-compact-cyberpunk.png`), fullPage: true });
    }
    await page.locator('nav [data-page="themes"]').click();
    await page.locator('[data-theme-choice="sky"]').click();
    await page.locator('#theme-apply').click();
    await page.waitForFunction(() => localStorage.getItem('apocalipse.theme') === 'sky');
    await page.locator('nav [data-page="recordings"]').click();
    await page.locator('#moq-capture-panel').evaluate(node => { node.open = true; });
    const light = await page.locator('#moq-capture-panel').evaluate(node => ({ color: getComputedStyle(node).color, backing: getComputedStyle(node).backgroundColor }));
    assert.equal(light.color, 'rgb(16, 24, 32)');
    assert.ok(Number(light.backing.match(/, ([\d.]+)\)$/)?.[1] || 1) >= .88, 'Light capture form has a contrast backing');
    const lightButton = await page.locator('.download-row .task-action').first().evaluate(node => ({ color: getComputedStyle(node).color, background: getComputedStyle(node).backgroundColor }));
    assert.equal(lightButton.color, 'rgb(16, 24, 32)');
    assert.ok(lightButton.background.includes('250, 252, 255'), 'Light task buttons use a light backing');
    await page.screenshot({ path: path.join(output, 'recordings-compact-light.png'), fullPage: true });
    await page.locator('nav [data-page="themes"]').click();
    for (const selector of ['.theme-card strong', '.preview-task strong']) {
      assert.equal(await page.locator(selector).first().evaluate(node => getComputedStyle(node).color), 'rgb(255, 255, 255)', 'Photo labels remain white in light themes');
    }
    await page.locator('[data-theme-choice="fantasy"]').click();
    await page.locator('#theme-apply').click();
    await page.waitForFunction(() => localStorage.getItem('apocalipse.theme') === 'fantasy');
    await page.locator('nav [data-page="downloads"]').click();
    await page.screenshot({ path: path.join(output, 'downloads-compact-fantasy.png'), fullPage: true });
    await page.locator('nav [data-page="about"]').click();
    assert.equal(await page.evaluate(() => getComputedStyle(document.body, '::before').backgroundImage), 'none');
    await page.screenshot({ path: path.join(output, 'about-theme-colors.png'), fullPage: true });
    await page.locator('nav [data-page="logs"]').click();
    const panes = await page.evaluate(() => ['.diagnostics-panel', '.log-event-list'].map(selector => document.querySelector(selector).getBoundingClientRect().height));
    assert.ok(panes[0] > 100 && Math.abs(panes[0] - panes[1]) < 2, `Logs panes share the height: ${panes}`);
    await page.screenshot({ path: path.join(output, 'logs-equal-panes.png'), fullPage: true });
    await page.locator('nav [data-page="themes"]').click();
    await page.setViewportSize({ width: 1000, height: 700 });
    await page.screenshot({ path: path.join(output, 'themes-small-window.png'), fullPage: true });
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true);
    const bounds = await page.locator(".app-titlebar").boundingBox();
    assert.equal(bounds.y, 0, "Native titlebar stays at the top");
    assert.equal(await page.evaluate(() => document.documentElement.scrollHeight <= innerHeight), true);
    assert.deepEqual(errors, []);
    console.log('Cinema UI: gallery, apply/cancel, search, transparency, titlebar actions and small window passed.');
  } finally { await browser.close(); server.close(); }
})().catch(error => { console.error(error); server.close(); process.exitCode = 1; });
