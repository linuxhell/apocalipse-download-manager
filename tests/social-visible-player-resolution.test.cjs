const assert = require('node:assert/strict');
const test = require('node:test');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

const extension = path.join(__dirname, '../browser-extension');

function loadGuard() {
  const source = fs.readFileSync(path.join(extension, 'popup-media-guard.js'), 'utf8');
  const context = vm.createContext({
    URL, Date, globalThis: null,
    chrome: {
      tabs: { sendMessage() {}, get() { return Promise.resolve({ url: 'https://www.tiktok.com/' }); } },
      runtime: { sendMessage() {} },
    },
  });
  context.globalThis = context;
  vm.runInContext(source, context);
  return context.ADM_POPUP_MEDIA_GUARD;
}

test('0.3.130 keeps only the visible unresolved social player', () => {
  const { cleanScanned } = loadGuard();
  const rows = cleanScanned([
    { url: 'https://www.tiktok.com/#apocalipse-player-1', kind: 'video', visualOnly: true, playerBound: true, recommended: true, thumbnail: 'data:image/jpeg;base64,A' },
    { url: 'https://www.tiktok.com/#apocalipse-player-2', kind: 'video', visualOnly: true, playerBound: true, recommended: false, thumbnail: 'data:image/jpeg;base64,B' },
  ], 'https://www.tiktok.com/');
  assert.equal(rows.length, 1);
  assert.match(rows[0].url, /player-1$/);
});

function fakeNode({ tagName = 'DIV', rect = null, hidden = false, ariaHidden = null, videos = [], anchors = [], parent = null, article = false } = {}) {
  return {
    tagName,
    hidden,
    isConnected: true,
    parentElement: parent,
    children: [],
    currentSrc: '', src: '', duration: 10, textContent: '', title: '', innerHTML: '',
    getBoundingClientRect: rect ? () => ({ ...rect }) : undefined,
    getAttribute(name) { return this.attributes?.[name] || (name === 'aria-hidden' ? ariaHidden : null); },
    querySelectorAll(selector) {
      if (selector === 'video') return videos;
      if (selector.includes('a[href*="/video/"]')) return anchors;
      return [];
    },
    matches(selector) { return article && selector.includes('article'); },
    closest() { return article ? this : null; },
    addEventListener() {}, removeEventListener() {}, click() {},
  };
}

function loadSocialResolver({ hostname = 'www.tiktok.com', href = 'https://www.tiktok.com/', documentFactory = null, globals = {} } = {}) {
  const source = fs.readFileSync(path.join(extension, 'social-player-resolution-v2.js'), 'utf8');
  const body = fakeNode({ tagName: 'BODY' });
  const html = fakeNode({ tagName: 'HTML' });
  const current = fakeNode({ tagName: 'VIDEO', rect: { left: 10, top: 10, right: 410, bottom: 710, width: 400, height: 700 } });
  current.currentSrc = `blob:https://${hostname}/current`;
  const preload = fakeNode({ tagName: 'VIDEO', rect: { left: 10, top: 1000, right: 410, bottom: 1700, width: 400, height: 700 } });
  preload.currentSrc = `blob:https://${hostname}/preload`;
  const anchor = { href: 'https://www.tiktok.com/@owner/video/7676540110162136322' };
  const card = fakeNode({ videos: [current, preload], anchors: [anchor], parent: body, article: true });
  current.parentElement = card; preload.parentElement = card;
  const baseDocument = {
    body, documentElement: html, title: 'Social',
    querySelectorAll(selector) {
      if (selector === 'video') return [current, preload];
      return [];
    },
  };
  const document = documentFactory ? documentFactory({ baseDocument, current, preload, card, anchor }) : baseDocument;
  const listeners = [];
  const context = vm.createContext({
    URL, Date, Promise, Number, String, Object, RegExp, Set, WeakSet,
    performance: { now: () => 100 },
    crypto: { randomUUID: () => '11111111-1111-4111-8111-111111111111' },
    globalThis: null,
    location: { href, hostname },
    document, navigator: { clipboard: { readText: async () => '' } },
    innerHeight: 800, innerWidth: 1000,
    getComputedStyle() { return { display: 'block', visibility: 'visible', opacity: '1' }; },
    setTimeout, clearTimeout,
    chrome: { runtime: { onMessage: { addListener(fn) { listeners.push(fn); } } } },
    ...globals,
  });
  context.globalThis = context;
  vm.runInContext(source, context);
  return { api: context.ADM_SOCIAL_PLAYER_RESOLVER_V2, context, current, preload, card, listeners };
}

test('0.3.130 TikTok DOM probe ignores hidden preload and resolves visible card', () => {
  const { api, current } = loadSocialResolver();
  const info = api.tiktokFromVisibleCardDetailed(current);
  assert.equal(info.url, 'https://www.tiktok.com/@owner/video/7676540110162136322');
  assert.equal(info.scope.hiddenPreloads, 1);
});

test('0.3.130 playerId preflight marks the refreshed geometry as an exact binding', () => {
  const { api, current } = loadSocialResolver();
  const found = api.findExactVisibleVideoDetailed({ playerId: 'player-7', playerBindingValidated: true,
    pageUrl: 'https://www.tiktok.com/', rect: current.getBoundingClientRect(), duration: 10 });
  assert.equal(found.video, current);
  assert.equal(found.bindingSource, 'player_id_scan');
  assert.equal(found.reason, 'exact_player_id_scan_match');
  const popup = fs.readFileSync(path.join(extension, 'popup-social-player-resolution.js'), 'utf8');
  assert.match(popup, /reason: 'player_id_stale'/);
  assert.match(popup, /type: 'APOCALIPSE_SCAN'/);
});

test('0.3.130 Facebook canonical validation rejects generic Watch routes', () => {
  const { api } = loadSocialResolver({ hostname: 'www.facebook.com', href: 'https://www.facebook.com/' });
  assert.equal(api.facebookCanonicalInfo('https://www.facebook.com/watch/hashtag/funny').url, null);
  assert.equal(api.facebookCanonicalInfo('https://www.facebook.com/watch/live/').url, null);
  assert.equal(api.facebookCanonicalInfo('https://www.facebook.com/reel/123456789').url, 'https://www.facebook.com/reel/123456789');
  assert.equal(api.facebookCanonicalInfo('https://www.facebook.com/watch/?v=123456789').url, 'https://www.facebook.com/watch/?v=123456789');
});

test('0.3.130 TikTok Copy link fallback can recover canonical URL from exact share menu', async () => {
  let menuOpen = false, clipboard = 'old';
  const fixture = loadSocialResolver({
    documentFactory({ baseDocument, current, card }) {
      const share = fakeNode({ tagName: 'BUTTON', rect: { left: 350, top: 300, right: 390, bottom: 340, width: 40, height: 40 } });
      share.attributes = { 'data-e2e': 'share-icon', 'aria-label': 'Share' };
      share.textContent = 'Share'; share.click = () => { menuOpen = true; };
      const copy = fakeNode({ tagName: 'BUTTON', rect: { left: 500, top: 300, right: 650, bottom: 350, width: 150, height: 50 } });
      copy.attributes = { 'data-e2e': 'share-copy-link' };
      copy.textContent = 'Copy link';
      copy.click = () => { clipboard = 'https://www.tiktok.com/@owner/video/7676540110162136322'; };
      card.querySelectorAll = selector => {
        if (selector === 'video') return [current];
        if (selector.includes('a[href*="/video/"]')) return [];
        if (selector.includes('button') || selector.includes('data-e2e')) return [share];
        return [];
      };
      return {
        ...baseDocument,
        querySelectorAll(selector) {
          if (selector === 'video') return [current];
          if (selector.includes('copy-link') || selector.includes('copylink') || selector.includes('[data-e2e*="copy"]')) return menuOpen ? [copy] : [];
          return [];
        },
      };
    },
  });
  fixture.context.navigator.clipboard.readText = async () => clipboard;
  const result = await fixture.api.tiktokFromShareMenuDetailed(fixture.current);
  assert.equal(result.url, 'https://www.tiktok.com/@owner/video/7676540110162136322');
  assert.equal(result.reason, 'tiktok_copy_link_clipboard');
});

test('0.3.130 popup resolver can enable only a current visible visual-only item', () => {
  const source = fs.readFileSync(path.join(extension, 'popup-social-player-resolution.js'), 'utf8');
  const context = vm.createContext({
    globalThis: null, console,
    document: { querySelector() { return null; }, querySelectorAll() { return []; }, addEventListener() {} },
    MutationObserver: class { observe() {} }, queueMicrotask() {}, setInterval() {},
    chrome: { tabs: { sendMessage() { return Promise.resolve(null); } } },
  });
  context.globalThis = context;
  vm.runInContext("let media=[]; let selected='video'; let activeMediaTab={id:7}; let activePageUrl='https://www.tiktok.com/'; let locale='pt_BR'; const t=x=>x; const render=()=>{};", context);
  vm.runInContext(source, context);
  assert.equal(context.ADM_POPUP_SOCIAL_PLAYER_RESOLUTION.resolvable({
    kind: 'video', visualOnly: true, playerBound: true, recommended: true, retained: false,
    rect: { left: 1, top: 1, width: 10, height: 10 },
  }), true);
  assert.equal(context.ADM_POPUP_SOCIAL_PLAYER_RESOLUTION.resolvable({
    kind: 'video', visualOnly: true, playerBound: true, recommended: false, retained: false,
    rect: { left: 1, top: 1, width: 10, height: 10 },
  }), false);
});

test('0.3.130 diagnostics expose readable resolution stages and preserve one trace across popup/content', () => {
  const core = fs.readFileSync(path.join(extension, 'diagnostics-core.js'), 'utf8');
  const resolver = fs.readFileSync(path.join(extension, 'social-player-resolution-v2.js'), 'utf8');
  const popup = fs.readFileSync(path.join(extension, 'popup-social-player-resolution.js'), 'utf8');
  assert.match(core, /failureStage/);
  assert.match(core, /permalinkSource/);
  assert.match(resolver, /player\.resolve_stage/);
  assert.match(resolver, /player\.resolve_summary/);
  assert.match(popup, /type: 'APOCALIPSE_RESOLVE_VISIBLE_SOCIAL_MEDIA_V2',[\s\S]*traceId/);
});

test('0.3.130 manifest loads resolution modules', () => {
  const manifest = JSON.parse(fs.readFileSync(path.join(extension, 'manifest.json'), 'utf8'));
  const html = fs.readFileSync(path.join(extension, 'popup.html'), 'utf8');
  assert.equal(manifest.version, '0.3.130');
  assert.ok(manifest.content_scripts.some(entry => entry.js?.includes('social-player-resolution-v2.js')));
  assert.match(html, /popup-social-player-resolution\.js/);
});
