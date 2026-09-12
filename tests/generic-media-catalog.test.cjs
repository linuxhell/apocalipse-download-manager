const assert = require('node:assert/strict');
const { test } = require('node:test');
const { page } = require('./helpers/thumbnail-page.cjs');
const A = 'https://cdn.example/a.mp4?signature=exact%2Ftoken';
const B = 'https://cdn.example/b.mp4';
const coverA = 'https://cdn.example/a.jpg';
const coverB = 'https://cdn.example/b.jpg';

for (const url of ['https://stream.example/feed', 'https://news.example/reels', 'https://learning.example/lessons']) {
  test(`${url}: scroll down and back retains the same files and covers without duplicate URLs`, () => {
    const p = page({ url, source: A, poster: coverA });
    assert.equal(p.hooks.collect().length, 1);
    p.video.src = p.video.currentSrc = B; p.video.poster = coverB;
    let rows = p.hooks.collect();
    assert.equal(rows.length, 2);
    assert.equal(rows.find(i => i.url === A).thumbnail, coverA);
    assert.equal(rows.find(i => i.url === A).retained, true);
    assert.equal(rows.find(i => i.url === A).rect, null);
    p.video.src = p.video.currentSrc = A; p.video.poster = coverA;
    rows = p.hooks.collect();
    assert.equal(rows.length, 2);
    assert.equal(rows.find(i => i.url === B).thumbnail, coverB);
    assert.equal(rows.find(i => i.url === A).retained, false);
    assert.ok(rows.every(i => !i.downloaded), 'captured is not downloaded');
  });
}

test('a new document has a separate catalog', () => {
  const a = page({ url: 'https://a.example/feed', source: A });
  const b = page({ url: 'https://b.example/feed', source: B });
  a.hooks.collect();
  assert.deepEqual(Array.from(b.hooks.collect(), i => i.url), [B]);
});

test('catalog keys keep media categories and signed URLs intact', () => {
  const p = page();
  const rows = p.hooks.rememberMedia(['video', 'audio', 'image'].map(kind => ({ url: A, kind })));
  assert.equal(rows.length, 3);
  assert.ok(rows.every(i => i.url === A));
  assert.equal(new Set(rows.map(i => i.kind)).size, 3);
});

test('catalog evicts old items at the documented bound', () => {
  const p = page();
  const rows = p.hooks.rememberMedia(Array.from({ length: 310 }, (_, i) => ({ url: `https://cdn.example/${i}.jpg`, kind: 'image' })));
  assert.equal(rows.length, 300);
  assert.equal(rows[0].url, 'https://cdn.example/10.jpg');
});

test('unresolved Blob players are not preserved as permanent files', () => {
  const p = page({ url: 'https://stream.example/feed', source: 'blob:https://stream.example/one' });
  const first = p.hooks.collect();
  assert.ok(first[0].visualOnly);
  p.video.src = p.video.currentSrc = 'blob:https://stream.example/two';
  const second = p.hooks.collect();
  assert.equal(second.length, 1);
  assert.notEqual(second[0].url, first[0].url);
});

test('a source change and a media reload invalidate the player generation', () => {
  const p = page({ source: A });
  const first = p.hooks.playerIdentity(p.video);
  p.video.src = p.video.currentSrc = B;
  const second = p.hooks.playerIdentity(p.video);
  assert.notEqual(first, second);
  p.events.emptied[0]();
  assert.notEqual(p.hooks.playerIdentity(p.video), second);
});

for (const [name, mutate] of [
  ['source', ({ videos }) => { videos[0].src = videos[0].currentSrc = B; }],
  ['source object', ({ videos }) => { videos[0].srcObject = {}; }],
  ['removed element', ({ videos }) => { videos[0].isConnected = false; }],
  ['position', ({ rect }) => { rect.top += 60; }],
  ['viewport', ({ context }) => { context.innerHeight += 100; }],
  ['navigation', ({ context }) => { context.location.href = 'https://other.example/new'; }],
]) {
  test(`late screenshot is discarded after a change in ${name}`, async () => {
    const p = page({ url: 'https://stream.example/feed', source: A, beforeCapture: mutate });
    assert.equal(await p.hooks.captureThumbnailFor(p.video), '');
  });
}

test('an unchanged exact player reuses its captured thumbnail', async () => {
  const p = page({ source: A });
  const first = await p.hooks.captureThumbnailFor(p.video);
  assert.match(first, /^data:image/);
  assert.equal(await p.hooks.captureThumbnailFor(p.video), first);
  assert.equal(p.sent.filter(m => m.type === 'APOCALIPSE_CAPTURE_VISIBLE_THUMBNAIL').length, 1);
});

test('page-wide artwork does not prove a virtualized single-player binding', () => {
  const p = page({ source: A });
  p.document.querySelector = selector => selector.includes('og:image') ? { content: 'https://cdn.example/site-logo.jpg' } : null;
  assert.equal(p.hooks.pageThumbnail(p.video), '');
});

test('exact og:video URL allows matching page artwork on a static video page', () => {
  const p = page({ source: A });
  p.document.querySelector = selector => selector.includes('og:video') ? { content: A }
    : selector.includes('og:image') ? { content: coverA } : null;
  assert.equal(p.hooks.pageThumbnail(p.video), coverA);
});

test('unbound HLS performance entries cannot borrow the first video thumbnail', () => {
  const p = page({ source: A, poster: coverA, url: 'https://stream.example/feed' });
  p.context.performance.getEntriesByType = () => [{ name: 'https://cdn.example/unrelated.m3u8' }];
  const row = p.hooks.collect().find(i => i.url.endsWith('.m3u8'));
  assert.equal(row.thumbnail, '');
  assert.equal(row.playerBound, undefined);
});

test('scheduled document collection works without opening a popup', () => {
  const p = page({ source: A, poster: coverA });
  const collectTimer = p.timers.find(t => t.delay === 350);
  assert.ok(collectTimer);
  collectTimer.callback();
  p.video.src = p.video.currentSrc = B; p.video.poster = coverB;
  const rows = p.hooks.collect();
  assert.ok(rows.some(i => i.url === A && i.thumbnail === coverA));
  assert.equal(p.sent.filter(m => m.type === 'APOCALIPSE_PROBE').length, 0);
});

test('a request for the previous file cannot capture a reused player', () => {
  const p = page({ source: A });
  const item = p.hooks.collect()[0];
  p.video.src = p.video.currentSrc = B;
  const replies = [];
  for (const listener of p.listeners) listener({ type: 'APOCALIPSE_CAPTURE_PLAYER_THUMBNAIL',
    playerId: item.playerId, url: item.url, pageUrl: p.location.href }, {}, r => replies.push(r));
  assert.equal(replies.length, 1);
  assert.equal(replies[0].error, 'stale_player_binding');
  assert.equal(p.sent.filter(m => m.type === 'APOCALIPSE_CAPTURE_VISIBLE_THUMBNAIL').length, 0);
});

test('a stale unchanged poster is not attached to a recycled player new source', () => {
  const p = page({ source: A, poster: coverA, url: 'https://stream.example/feed' });
  assert.equal(p.hooks.pageThumbnail(p.video), coverA);
  p.video.src = p.video.currentSrc = B;
  assert.equal(p.hooks.pageThumbnail(p.video), '');
  p.video.poster = coverB;
  assert.equal(p.hooks.pageThumbnail(p.video), coverB);
});

test('a single-page navigation invalidates cached player thumbnails', async () => {
  const p = page({ source: A });
  const first = p.hooks.playerIdentity(p.video);
  await p.hooks.captureThumbnailFor(p.video);
  p.location.href = 'https://www.facebook.com/reel/456';
  assert.notEqual(p.hooks.playerIdentity(p.video), first);
  await p.hooks.captureThumbnailFor(p.video);
  assert.equal(p.sent.filter(m => m.type === 'APOCALIPSE_CAPTURE_VISIBLE_THUMBNAIL').length, 2);
});
