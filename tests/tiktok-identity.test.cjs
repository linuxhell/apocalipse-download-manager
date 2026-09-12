const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const vm = require('node:vm');
const script = readFileSync(join(__dirname, '../browser-extension/tiktok-identity.js'), 'utf8');
class Element {
  constructor(tag, attributes = {}, children = []) {
    this.tagName = tag.toUpperCase(); this.attributes = attributes;
    this.children = children; this.isConnected = true;
    for (const child of children) child.parentElement = this;
  }
  getAttribute(name) { return this.attributes[name] || null; }
  matches() { return this.tagName === 'ARTICLE' || this.attributes['data-e2e'] === 'feed-video'; }
  querySelectorAll(selector) {
    return this.children.flatMap(child => [
      ...((selector === 'video' && child.tagName === 'VIDEO') || (selector.includes('a[href') && child.tagName === 'A') ? [child] : []),
      ...child.querySelectorAll(selector),
    ]);
  }
}
function fixture(url = 'https://www.tiktok.com/', networkPayload = null) {
  const html = new Element('html'), body = new Element('body'); body.parentElement = html;
  const scripts = [];
  const document = { body, documentElement: html, querySelectorAll: selector => selector.startsWith('script') ? scripts : body.querySelectorAll(selector) };
  const context = vm.createContext({ URL, location: new URL(url), document,
    ...(networkPayload ? { fetch: async () => ({ headers: { get: () => 'application/json' },
      clone: () => ({ json: async () => networkPayload }) }) } : {}) });
  vm.runInContext(script, context);
  const card = (id, source, tag = 'article') => {
    const video = new Element('video'); video.currentSrc = video.src = source;
    const anchor = new Element('a'); anchor.href = `https://www.tiktok.com/@creator/video/${id}`;
    const root = new Element(tag, {}, [video, anchor]); root.parentElement = body; body.children.push(root);
    return { video, anchor, root };
  };
  return { api: context.ApocalipseTikTokIdentity, context, card, body, scripts };
}
test('TikTok current card wins over an address bar and first feed link pointing at another reel', () => {
  const f = fixture('https://www.tiktok.com/@creator/video/111');
  const a = f.card('111', 'https://v16.tiktok.com/video/tos/a/');
  const b = f.card('222', 'https://v16.tiktok.com/video/tos/b/');
  assert.equal(f.api.resolve(b.video), b.anchor.href);
  assert.equal(f.api.resolve(a.video), a.anchor.href);
});
test('unlabelled wrapper never traverses into an ancestor holding two players', () => {
  const f = fixture();
  const a = f.card('111', 'blob:a', 'div');
  const b = f.card('222', 'blob:b', 'div');
  b.root.children = [b.video];
  assert.equal(f.api.resolve(b.video), null);
  assert.equal(f.api.resolve(a.video), a.anchor.href);
});
test('a recycled card with stale permalink is rejected until metadata changes', () => {
  const f = fixture(); const a = f.card('111', 'https://v16.tiktok.com/video/tos/a/');
  assert.equal(f.api.resolve(a.video), a.anchor.href);
  a.video.currentSrc = 'https://v16.tiktok.com/video/tos/b/';
  assert.equal(f.api.resolve(a.video), null);
  a.anchor.href = 'https://www.tiktok.com/@creator/video/222';
  assert.equal(f.api.resolve(a.video), a.anchor.href);
});
test('exact player source selects the matching global record, not the first JSON item', () => {
  const f = fixture(); const a = f.card('111', 'https://v16.tiktok.com/video/tos/b/?signed=new');
  a.root.children = [a.video];
  f.scripts.push({ textContent: JSON.stringify({ ItemModule: {
    a: { id: '111', author: { uniqueId: 'creator' }, video: { playAddr: 'https://v16.tiktok.com/video/tos/a/?signed=old' } },
    b: { id: '222', author: { uniqueId: 'creator' }, video: { playAddr: 'https://v16.tiktok.com/video/tos/b/?signed=old' } },
  } }) });
  assert.equal(f.api.resolve(a.video), 'https://www.tiktok.com/@creator/video/222');
});
test('play endpoint video_id is part of the identity', () => {
  const f = fixture(); const a = f.card('111', 'https://www.tiktok.com/aweme/v1/play/?video_id=assetB');
  a.root.children = [a.video];
  f.scripts.push({ textContent: JSON.stringify([
    { id: '111', author: { uniqueId: 'creator' }, video: { playAddr: 'https://www.tiktok.com/aweme/v1/play/?video_id=assetA' } },
    { id: '222', author: { uniqueId: 'creator' }, video: { playAddr: 'https://www.tiktok.com/aweme/v1/play/?video_id=assetB' } },
  ]) });
  assert.equal(f.api.resolve(a.video), 'https://www.tiktok.com/@creator/video/222');
});
test('ambiguous card links and global state without a source match never become a guess', () => {
  const f = fixture(); const a = f.card('111', 'blob:current');
  const other = new Element('a'); other.href = 'https://www.tiktok.com/@creator/video/222'; a.root.children.push(other);
  assert.equal(f.api.resolve(a.video), null);
  a.root.children = [a.video];
  f.scripts.push({ textContent: JSON.stringify([{ id: '111', author: { uniqueId: 'creator' } }]) });
  assert.equal(f.api.resolve(a.video), null);
});
test('MAIN-world framework props bound to the card are resolved without climbing Fiber return links', () => {
  const f = fixture(); const a = f.card('111', 'blob:current'); a.root.children = [a.video];
  a.root.__reactProps$test = { item: { id: '222', author: { uniqueId: 'current' }, video: {} },
    return: { id: '111', author: { uniqueId: 'wrong' } } };
  assert.equal(f.api.resolveLocal(a.video, false), 'https://www.tiktok.com/@current/video/222');
});

test('a unique parent Fiber identity is accepted when the current TikTok layout has no card marker', () => {
  const f = fixture();
  const video = new Element('video'); video.currentSrc = video.src = 'blob:current';
  const wrapper = new Element('div', {}, [video]); wrapper.parentElement = f.body; f.body.children.push(wrapper);
  wrapper.__reactFiber$test = { return: { id: '333', author: { uniqueId: 'parent' }, video: {} } };
  assert.equal(f.api.resolveLocal(video, false), 'https://www.tiktok.com/@parent/video/333');
});

test('unresolved TikTok identity reports safe structure counts without page content', () => {
  assert.match(script, /parentRecordCount/);
  assert.match(script, /parentDistinctIds/);
  assert.match(script, /explicitIdCount/);
  assert.match(script, /sourceIdentityPresent/);
  assert.match(script, /networkRecordCount/);
});

test('TikTok preserves item identities from feed responses before Blob rendering discards them', () => {
  assert.match(script, /rememberNetworkPayload/);
  assert.match(script, /matched_feed_response/);
  assert.match(script, /__apocalipseTikTokFetchIdentity/);
  assert.match(script, /__apocalipseTikTokXhrIdentity/);
});

test('a unique TikTok feed-response duration and size resolves the current Blob player', async () => {
  const payload = { itemList: [
    { id: '777', author: { uniqueId: 'right' }, video: { duration: 9, width: 1080, height: 1920 } },
    { id: '888', author: { uniqueId: 'other' }, video: { duration: 20, width: 1080, height: 1920 } },
  ] };
  const f = fixture('https://www.tiktok.com/', payload);
  await f.context.fetch('https://www.tiktok.com/api/recommend/item_list/');
  await Promise.resolve();
  const current = f.card('111', 'blob:current', 'div');
  current.root.children = [current.video]; current.video.duration = 9;
  current.video.videoWidth = 1080; current.video.videoHeight = 1920;
  assert.equal(f.api.resolveLocal(current.video, false), 'https://www.tiktok.com/@right/video/777');
});
test('buttons keep their exact video reference, not the nearest player geometry', () => {
  const f = fixture(); const a = f.card('111', 'blob:a'), b = f.card('222', 'blob:b');
  const button = {}; f.api.bind(button, b.video);
  assert.equal(f.api.videoFor(button), b.video); assert.notEqual(f.api.videoFor(button), a.video);
  b.video.isConnected = false; assert.equal(f.api.videoFor(button), null);
});
test('button binding survives a resolver realm restart without guessing another player', () => {
  const f = fixture(); const a = f.card('111', 'blob:a'), b = f.card('222', 'blob:b');
  const button = {}; f.api.bind(button, b.video);
  vm.runInContext(script, f.context);
  const restarted = f.context.ApocalipseTikTokIdentity;
  assert.equal(restarted.videoFor(button), b.video);
  assert.notEqual(restarted.videoFor(button), a.video);
});
test('unsafe integer IDs and spoofed TikTok hosts are rejected', () => {
  const f = fixture(); const a = f.card('111', 'blob:current'); a.root.children = [a.video];
  a.root.__reactProps$test = { item: { id: 9007199254740993, author: { uniqueId: 'creator' } } };
  assert.equal(f.api.resolve(a.video), null);
  assert.equal(f.api.validUrl('https://tiktok.com.evil.example/@x/video/1'), null);
});
