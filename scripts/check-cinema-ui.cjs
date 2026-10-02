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
          progress_percent: 56, resume_supported: true, connections_override: 16 }];
        if (command === 'get_app_version') return '0.4.80';
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
    await page.locator('.download-row').waitFor();
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
    await page.setViewportSize({ width: 1000, height: 700 });
    await page.screenshot({ path: path.join(output, 'themes-small-window.png'), fullPage: true });
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true);
    assert.deepEqual(errors, []);
    console.log('Cinema UI: gallery, apply/cancel, search, transparency, titlebar actions and small window passed.');
  } finally { await browser.close(); server.close(); }
})().catch(error => { console.error(error); server.close(); process.exitCode = 1; });
