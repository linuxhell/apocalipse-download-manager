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

test('0.3.129 keeps only the visible unresolved social player', () => {
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
    currentSrc: '', src: '', duration: 10,
    getBoundingClientRect: rect ? () => ({ ...rect }) : undefined,
    getAttribute(name) { return name === 'aria-hidden' ? ariaHidden : null; },
    querySelectorAll(selector) {
      if (selector === 'video') return videos;
      if (selector.includes('a[href*="/video/"]')) return anchors;
      return [];
    },
    matches(selector) { return article && selector.includes('article'); },
    closest() { return article ? this : null; },
    addEventListener() {}, removeEventListener() {},
  };
}

function loadSocialResolver() {
  const source = fs.readFileSync(path.join(extension, 'social-player-resolution.js'), 'utf8');
  const body = fakeNode({ tagName: 'BODY' });
  const html = fakeNode({ tagName: 'HTML' });
  const current = fakeNode({ tagName: 'VIDEO', rect: { left: 10, top: 10, right: 410, bottom: 710, width: 400, height: 700 } });
  current.currentSrc = 'blob:https://www.tiktok.com/current';
  const preload = fakeNode({ tagName: 'VIDEO', rect: { left: 10, top: 1000, right: 410, bottom: 1700, width: 400, height: 700 } });
  preload.currentSrc = 'blob:https://www.tiktok.com/preload';
  const anchor = { href: 'https://www.tiktok.com/@owner/video/7676540110162136322' };
  const card = fakeNode({ videos: [current, preload], anchors: [anchor], parent: body, article: true });
  current.parentElement = card; preload.parentElement = card;
  const document = {
    body, documentElement: html, title: 'TikTok',
    querySelectorAll(selector) {
      if (selector === 'video') return [current, preload];
      return [];
    },
  };
  const listeners = [];
  const context = vm.createContext({
    URL, Date, Promise, Number, String, Object, RegExp, Set, WeakSet,
    globalThis: null,
    location: { href: 'https://www.tiktok.com/', hostname: 'www.tiktok.com' },
    document, navigator: { clipboard: { readText: async () => '' } },
    innerHeight: 800, innerWidth: 1000,
    getComputedStyle() { return { display: 'block', visibility: 'visible', opacity: '1' }; },
    setTimeout, clearTimeout,
    chrome: { runtime: { onMessage: { addListener(fn) { listeners.push(fn); } } } },
  });
  context.globalThis = context;
  vm.runInContext(source, context);
  return { api: context.ADM_SOCIAL_PLAYER_RESOLVER, current, preload, listeners };
}

test('0.3.129 TikTok fallback ignores the hidden preload player and resolves the visible card', () => {
  const { api, current } = loadSocialResolver();
  assert.equal(api.tiktokFromVisibleCard(current), 'https://www.tiktok.com/@owner/video/7676540110162136322');
});

test('0.3.129 exact-player lookup selects the visible player, not its preload neighbor', () => {
  const { api, current } = loadSocialResolver();
  const found = api.findExactVisibleVideo({ rect: current.getBoundingClientRect(), duration: current.duration });
  assert.equal(found, current);
});

test('0.3.129 popup resolver can enable only a current visible visual-only item', () => {
  const source = fs.readFileSync(path.join(extension, 'popup-social-player-resolution.js'), 'utf8');
  const context = vm.createContext({
    globalThis: null, console,
    document: {
      querySelector() { return null; },
      querySelectorAll() { return []; },
      addEventListener() {},
    },
    MutationObserver: class { observe() {} },
    queueMicrotask() {}, setInterval() {},
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

test('0.3.129 manifest loads both resolution modules', () => {
  const manifest = JSON.parse(fs.readFileSync(path.join(extension, 'manifest.json'), 'utf8'));
  const html = fs.readFileSync(path.join(extension, 'popup.html'), 'utf8');
  assert.equal(manifest.version, '0.3.129');
  assert.ok(manifest.content_scripts.some(entry => entry.js?.includes('social-player-resolution.js')));
  assert.match(html, /popup-social-player-resolution\.js/);
});
