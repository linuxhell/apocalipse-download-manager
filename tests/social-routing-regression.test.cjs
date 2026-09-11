const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { webcrypto } = require('node:crypto');
const { test } = require('node:test');
const vm = require('node:vm');

const content = readFileSync(join(__dirname, '../browser-extension/content.js'), 'utf8');
const tiktok = readFileSync(join(__dirname, '../browser-extension/tiktok-media-fix.js'), 'utf8');

// Run the scripts in manifest order, including TikTok's document-capture listener.
// Browser APIs are mocked; installed click handlers and outgoing payloads are real.
function page({ url = 'https://www.tiktok.com/', source = 'https://v16.tiktok.com/video/a.mp4', sourceObject = null, permalink = null, network = [], inspections = [], shipped = false, readableBlob = false, onMediaQuery = null } = {}) {
  const sent = [], fetched = [], appended = [], clickListeners = [];
  const location = new URL(url);
  const state = { permalink, network };
  const rect = { left: 20, top: 40, right: 500, bottom: 600, width: 480, height: 560 };
  const anchors = () => state.permalink ? [{ href: state.permalink, getBoundingClientRect: () => rect }] : [];
  const post = {
    parentElement: null, innerHTML: '', getBoundingClientRect: () => rect,
    querySelectorAll: selector => selector.includes('a[href') ? anchors() : [],
    querySelector: selector => selector.includes('a[href') ? anchors()[0] || null : null,
  };
  const video = {
    tagName: 'VIDEO', dataset: {}, isConnected: true, currentSrc: source, src: source, srcObject: sourceObject,
    title: 'Synthetic current video', poster: 'https://images.example/poster.jpg', duration: 30,
    parentElement: post, innerHTML: '', getBoundingClientRect: () => rect,
    getAttribute: () => null,
    querySelector: selector => selector.includes('a[href') ? anchors()[0] || null : null,
    querySelectorAll: () => [], closest: selector => selector.includes('article') ? post : null,
  };
  const element = tag => ({
    tagName: tag.toUpperCase(), style: {}, dataset: {}, offsetWidth: 80,
    addEventListener(type, handler) { this[type] = handler; },
    remove() { this.removed = true; },
    getBoundingClientRect: () => ({ ...rect, width: 80, height: 25 }),
    closest(selector) { return selector.startsWith('.apocalipse-media-download') ? this : null; },
  });
  const document = {
    title: 'Synthetic current video', fullscreenElement: null,
    documentElement: { dataset: {}, append(node) { appended.push(node); } },
    createElement: element, elementFromPoint: () => video, dispatchEvent() {},
    addEventListener(type, handler, capture) { if (type === 'click') clickListeners.push({ handler, capture }); },
    querySelector: selector => selector === 'video' ? video : null,
    querySelectorAll: selector => selector === 'video' || selector === 'video,audio' ? [video]
      : selector === '.apocalipse-media-download' ? appended.filter(node => !node.removed && String(node.className).includes('apocalipse-media-download')) : [],
  };
  const context = vm.createContext({
    URL, Blob, Uint8Array, crypto: webcrypto, console, Date, document, location,
    navigator: { language: 'pt-BR', userAgent: 'Synthetic browser' },
    innerHeight: 800, scrollX: 0, scrollY: 0,
    addEventListener() {}, removeEventListener() {}, postMessage() {},
    setTimeout(callback, delay) { if (delay === 75 || delay === 100) queueMicrotask(callback); return 1; },
    clearTimeout() {}, setInterval: () => 1, clearInterval() {},
    MutationObserver: class { observe() {} },
    performance: { getEntriesByType: () => state.network.map(item => ({ name: item.url })), getEntriesByName: () => [] },
    fetch: async target => {
      fetched.push(target);
      if (!readableBlob) throw new Error('synthetic unreadable blob');
      return { ok: true, blob: async () => new Blob(['synthetic track'], { type: 'video/mp4' }) };
    },
    chrome: {
      storage: {
        local: { get(defaults, callback) { if (callback) return callback(defaults); return Promise.resolve(defaults); } },
        onChanged: { addListener() {} },
      },
      runtime: {
        onMessage: { addListener() {} },
        sendMessage(message, callback) {
          sent.push(message);
          if (message.type === 'APOCALIPSE_RECENT_TAB_MEDIA') onMediaQuery?.(video, state);
          const result = message.type === 'APOCALIPSE_RECENT_TAB_MEDIA' ? { media: state.network }
            : message.type === 'APOCALIPSE_INSPECT_MEDIA_TRACKS' ? { media: inspections }
            : message.type === 'APOCALIPSE_BLOB_BEGIN' ? { uploadId: 'synthetic-upload' }
              : { ok: true, target: 'desktop' };
          if (callback) callback(result);
          return Promise.resolve(result);
        },
      },
    },
  });
  context.window = context; context.top = context;
  vm.runInContext(readFileSync(join(__dirname, '../browser-extension/tiktok-identity.js'), 'utf8'), context);
  vm.runInContext(content.replace(/\}\)\(\);\s*$/, 'globalThis.testHooks = { installOverlays, collect };\n})();'), context);
  if (shipped) vm.runInContext(tiktok, context);
  context.testHooks.installOverlays();
  const button = appended.find(node => node.className === 'apocalipse-media-download');
  assert.ok(button, 'the actual overlay must be installed');
  return {
    sent, fetched, state, location, video, button,
    async click() {
      let stopped = false;
      const event = {
        button: 0, target: button, composedPath: () => [button],
        preventDefault() { this.defaultPrevented = true; }, stopPropagation() {},
        stopImmediatePropagation() { stopped = true; },
      };
      if (shipped) for (const listener of clickListeners.filter(value => value.capture)) {
        await listener.handler(event);
        if (stopped) return;
      }
      await button.click(event);
    },
    downloads: () => sent.filter(message => message.type === 'APOCALIPSE_DOWNLOAD').map(message => message.item),
  };
}

const track = (url, contentType = 'video/mp4', capturedAt = 1000, frameId = 0) => ({ url, contentType, capturedAt, frameId, ageMs: 10 });
const videoA = 'https://v16.tiktok.com/video/a.mp4';
const audioA = 'https://v16.tiktok.com/audio/a.mp4';
const videoB = 'https://v16.tiktok.com/video/b.mp4';
const audioB = 'https://v16.tiktok.com/audio/b.mp4';
const pageA = 'https://www.tiktok.com/@synthetic/video/123456789';
const pageB = 'https://www.tiktok.com/@synthetic/video/987654321';

for (const shipped of [false, true]) {
  const route = shipped ? 'shipped TikTok capture listener' : 'generic overlay';
  test(`${route}: an isolated social video is marked incomplete`, async () => {
    const p = page({ shipped, source: videoA, network: [track(videoA)] });
    await p.click();
    assert.equal(p.downloads().length, 1);
    assert.equal(p.downloads()[0].url, videoA);
    assert.equal(p.downloads()[0].ambiguousSocialTrack, true);
    assert.equal(p.downloads()[0].audioUrl, null);
  });
  test(`${route}: a specific permalink wins even over a readable blob`, async () => {
    const p = page({ shipped, url: pageA, source: 'blob:https://www.tiktok.com/synthetic', readableBlob: true, network: [track(videoB)] });
    await p.click();
    assert.equal(p.downloads().length, 1);
    assert.equal(p.downloads()[0].url, pageA);
    assert.equal(p.downloads()[0].audioUrl, null);
    assert.equal(p.downloads()[0].requestUrls.length, 0);
    assert.equal(p.fetched.length, 0);
  });
  test(`${route}: generic MSE media cannot adopt another buffered video's URL`, async () => {
    const p = page({ shipped, source: 'blob:https://www.tiktok.com/synthetic', network: [track(videoB)] });
    await p.click();
    assert.equal(p.downloads().length, 0);
    assert.ok(!p.sent.some(message => message.type.startsWith('APOCALIPSE_BLOB_')));
  });
  test(`${route}: a recycled player uses its new source and matching audio`, async () => {
    const p = page({ shipped, source: videoA, network: [track(videoA), track(audioA, 'audio/mp4', 1001)] });
    await p.click();
    p.video.currentSrc = videoB; p.video.src = videoB;
    p.state.network.unshift(track(videoB, 'video/mp4', 5000), track(audioB, 'audio/mp4', 5001));
    await p.click();
    assert.deepEqual(p.downloads().map(item => [item.url, item.audioUrl]), [[videoA, audioA], [videoB, audioB]]);
    assert.ok(p.downloads().every(item => item.ambiguousSocialTrack === false));
  });
  test(`${route}: audio from another frame is never attached`, async () => {
    const p = page({ shipped, source: videoA, network: [track(videoA), track(audioA, 'audio/mp4', 1001, 2)] });
    await p.click();
    assert.equal(p.downloads()[0].audioUrl, null);
    assert.equal(p.downloads()[0].ambiguousSocialTrack, true);
  });
  test(`${route}: missing capture timestamps do not establish a pair`, async () => {
    const p = page({ shipped, source: videoA, network: [{ url: videoA, contentType: 'video/mp4' }, { url: audioA, contentType: 'audio/mp4' }] });
    await p.click();
    assert.equal(p.downloads()[0].audioUrl, null);
    assert.equal(p.downloads()[0].ambiguousSocialTrack, true);
  });
  test(`${route}: audio closer to another buffered video is rejected`, async () => {
    const p = page({ shipped, source: videoA, network: [track(videoA), track(videoB, 'video/mp4', 5000), track(audioB, 'audio/mp4', 5001)] });
    await p.click();
    assert.equal(p.downloads()[0].audioUrl, null);
    assert.equal(p.downloads()[0].ambiguousSocialTrack, true);
  });
  test(`${route}: equally close audio candidates are ambiguous`, async () => {
    const p = page({ shipped, source: videoA, network: [track(videoA), track(audioA, 'audio/mp4', 999), track(audioB, 'audio/mp4', 1001)] });
    await p.click();
    assert.equal(p.downloads()[0].audioUrl, null);
    assert.equal(p.downloads()[0].ambiguousSocialTrack, true);
  });
  test(`${route}: a player changing during lookup cancels the stale handoff`, async () => {
    const p = page({ shipped, source: videoA, network: [track(videoA)], onMediaQuery(video) { video.currentSrc = videoB; video.src = videoB; } });
    await p.click();
    assert.equal(p.downloads().length, 0);
  });
}

test('the shipped TikTok interceptor resolves each new card permalink', async () => {
  const p = page({ shipped: true, permalink: pageA, source: videoA, network: [track(videoA)] });
  await p.click();
  p.state.permalink = pageB; p.video.currentSrc = videoB; p.video.src = videoB;
  await p.click();
  assert.deepEqual(p.downloads().map(item => item.url), [pageA, pageB]);
});

test('TikTok blob feed selects the unique complete MP4 with the bound player duration', async () => {
  const current = track(videoA, 'video/mp4', 5000);
  const other = track(videoB, 'video/mp4', 6000);
  const p = page({ shipped: true, source: 'blob:https://www.tiktok.com/current', network: [other, current],
    inspections: [{ url: videoB, kind: 'muxed', duration: 44 }, { url: videoA, kind: 'muxed', duration: 30 }] });
  await p.click();
  assert.equal(p.downloads().length, 1);
  assert.equal(p.downloads()[0].url, videoA);
  assert.equal(p.downloads()[0].audioUrl, null);
  assert.equal(p.downloads()[0].ambiguousSocialTrack, false);
});

test('TikTok blob feed refuses tied duration matches from buffered cards', async () => {
  const p = page({ shipped: true, source: 'blob:https://www.tiktok.com/current', network: [track(videoA), track(videoB)],
    inspections: [{ url: videoA, kind: 'muxed', duration: 30 }, { url: videoB, kind: 'muxed', duration: 30.02 }] });
  await p.click();
  assert.equal(p.downloads().length, 0);
});

test('a vanished TikTok permalink cannot fall back to the button creation-time link', async () => {
  const p = page({ permalink: pageA, source: videoA, network: [track(videoA)] });
  await p.click();
  p.state.permalink = null; p.video.currentSrc = videoB; p.video.src = videoB;
  p.state.network = [track(videoB)];
  await p.click();
  assert.deepEqual(p.downloads().map(item => item.url), [pageA, videoB]);
  assert.equal(p.downloads()[1].ambiguousSocialTrack, true);
});

test('Facebook Home queries tracks for the current player without requiring a page permalink', async () => {
  const video = 'https://video.fbcdn.net/current.mp4';
  const audio = 'https://audio.fbcdn.net/current.mp4';
  const p = page({ url: 'https://www.facebook.com/', source: video, network: [track(video), track(audio, 'audio/mp4', 1001)] });
  await p.click();
  assert.equal(p.downloads()[0].url, video);
  assert.equal(p.downloads()[0].audioUrl, audio);
  assert.equal(p.downloads()[0].ambiguousSocialTrack, false);
});

test('Facebook sponsored blob player identifies video and mislabeled audio MP4 tracks', async () => {
  const video = 'https://video.fbcdn.net/current.mp4';
  const audio = 'https://video.fbcdn.net/current-audio.mp4';
  const p = page({ url: 'https://www.facebook.com/', source: 'blob:https://www.facebook.com/current',
    network: [track(video, 'video/mp4', 1000), track(audio, 'video/mp4', 1001)],
    inspections: [{ url: video, kind: 'video', duration: 30 }, { url: audio, kind: 'audio', duration: 30 }] });
  await p.click();
  assert.equal(p.downloads()[0].url, video);
  assert.equal(p.downloads()[0].audioUrl, audio);
  assert.equal(p.downloads()[0].ambiguousSocialTrack, false);
});

test('Facebook sponsored srcObject player uses the same duration-bound track selection', async () => {
  const video = 'https://video.fbcdn.net/src-object-video.mp4';
  const audio = 'https://video.fbcdn.net/src-object-audio.mp4';
  const p = page({ url: 'https://www.facebook.com/', source: '', sourceObject: {},
    network: [track(video, 'video/mp4', 1000), track(audio, 'video/mp4', 1001)],
    inspections: [{ url: video, kind: 'video', duration: 30 }, { url: audio, kind: 'audio', duration: 30 }] });
  await p.click();
  assert.equal(p.downloads()[0].url, video);
  assert.equal(p.downloads()[0].audioUrl, audio);
  assert.equal(p.downloads()[0].ambiguousSocialTrack, false);
});

test('Instagram and ordinary direct HTTP videos are not marked ambiguous', async () => {
  for (const url of ['https://www.instagram.com/', 'https://example.com/video']) {
    const source = 'https://cdn.example/complete.mp4';
    const p = page({ url, source });
    await p.click();
    assert.equal(p.downloads()[0].url, source);
    assert.equal(p.downloads()[0].ambiguousSocialTrack, false);
  }
});

// Exercise the real popup pairing function without mocking away its decisions.
const popupSource = readFileSync(join(__dirname, '../browser-extension/popup.js'), 'utf8');
const popupContext = vm.createContext({ URL });
vm.runInContext(popupSource.slice(0, popupSource.indexOf('const messages ='))
  + '\nglobalThis.pairTracks = pairSocialTracks; globalThis.mergeDetected = mergeDetectedMedia; globalThis.mediaKind = networkMediaKind;', popupContext);
const popupTrack = (url, kind, capturedAt, frameId = 0) => ({ url, kind, capturedAt, frameId, networkCaptured: true });
test('popup does not pair one audio track to a different buffered video', () => {
  const result = popupContext.pairTracks([
    popupTrack(videoA, 'video', 1000), popupTrack(videoB, 'video', 5000), popupTrack(audioB, 'audio', 5001),
  ], 'https://www.tiktok.com/');
  assert.equal(result.find(item => item.url === videoA).ambiguousSocialTrack, true);
  assert.equal(result.find(item => item.url === videoB).audioUrl, audioB);
});
test('popup refuses invalid timestamps, uncertain frames and tied candidates', () => {
  for (const items of [
    [popupTrack(videoA, 'video', NaN), popupTrack(audioA, 'audio', 1001)],
    [popupTrack(videoA, 'video', 1000, null), popupTrack(audioA, 'audio', 1001, 2)],
    [popupTrack(videoA, 'video', 1000), popupTrack(audioA, 'audio', 999), popupTrack(audioB, 'audio', 1001)],
  ]) assert.equal(popupContext.pairTracks(items, 'https://www.facebook.com/')[0].ambiguousSocialTrack, true);
});

test('popup trusts audio MIME type over a misleading mp4 extension', () => {
  assert.equal(popupContext.mediaKind({ url: audioA, contentType: 'audio/mp4' }), 'audio');
});
test('popup keeps capture metadata when DOM and network report the same video', () => {
  const result = popupContext.mergeDetected([{ url: videoA, kind: 'video', title: 'Current card' }], [
    popupTrack(videoA, 'video', 1000), popupTrack(audioA, 'audio', 1001),
  ], 'https://www.tiktok.com/');
  assert.equal(result[0].audioUrl, audioA);
  assert.equal(result[0].title, 'Current card');
});
test('popup hides incidental CDN tracks for both social-page extractors', () => {
  for (const url of ['https://www.tiktok.com/', 'https://www.facebook.com/', 'https://www.instagram.com/']) {
    const result = popupContext.mergeDetected([{ url: pageA, kind: 'video', pageExtractor: true }], [
      popupTrack(videoA, 'video', 1000), popupTrack(audioA, 'audio', 1001),
    ], url);
    assert.equal(result.length, 1);
    assert.equal(result[0].pageExtractor, true);
  }
});
test('a visual social video suppresses unrelated CDN fragments globally', () => {
  for (const url of ['https://www.tiktok.com/', 'https://www.facebook.com/', 'https://www.instagram.com/']) {
    const visual = { url: pageA, kind: 'video', thumbnail: 'https://img.example/current.jpg', pageExtractor: true };
    const result = popupContext.mergeDetected([visual], [
      popupTrack(videoA, 'video', 1000), popupTrack(audioA, 'audio', 1001),
    ], url);
    assert.deepEqual(JSON.parse(JSON.stringify(result)), [visual]);
  }
});
test('a generic social Blob player resolves one CDN track only by unique duration', () => {
  const hint = { url: 'https://www.facebook.com/', kind: 'video', duration: 30,
    thumbnail: 'https://img.example/current.jpg', title: 'Visible card' };
  const current = { ...popupTrack(videoA, 'video', 1000), duration: 30, muxed: true };
  const other = { ...popupTrack(videoB, 'video', 2000), duration: 42, muxed: true };
  const result = popupContext.mergeDetected([hint], [current, other], 'https://www.facebook.com/');
  assert.equal(result.length, 1);
  assert.equal(result[0].url, videoA);
  assert.equal(result[0].thumbnail, hint.thumbnail);
  assert.equal(result[0].ambiguousSocialTrack, false);
});

test('duration fallback refuses tied social CDN tracks', () => {
  const hint = { url: 'https://www.facebook.com/', kind: 'video', duration: 30,
    thumbnail: 'https://img.example/current.jpg' };
  const result = popupContext.mergeDetected([hint], [
    { ...popupTrack(videoA, 'video', 1000), duration: 30, muxed: true },
    { ...popupTrack(videoB, 'video', 2000), duration: 30.2, muxed: true },
  ], 'https://www.facebook.com/');
  assert.ok(result.length > 1);
  assert.equal(result.find((item) => item.url === hint.url).ambiguousSocialTrack, true);
});
test('popup never assigns the viewport thumbnail to an anonymous CDN response', () => {
  const popup = readFileSync(join(__dirname, '../browser-extension/popup.js'), 'utf8');
  assert.match(popup, /item\.recommended\s*&&\s*!item\.networkCaptured\s*&&\s*!item\.thumbnail/);
  assert.doesNotMatch(popup, /sort\(\(left, right\).*capturedAt/);
});
test('popup disables Preview for a generic social homepage extractor', () => {
  const popup = readFileSync(join(__dirname, '../browser-extension/popup.js'), 'utf8');
  assert.match(popup, /const socialExtractor = pageExtractor/);
  assert.match(popup, /if \(!specific\) return null/);
});
test('popup refreshes video, audio and image inventory while it remains open', () => {
  const popup = readFileSync(join(__dirname, '../browser-extension/popup.js'), 'utf8');
  assert.match(popup, /async function refreshMediaInventory/);
  assert.match(popup, /setInterval\(\(\) => \{ if \(activeMediaTab\)/);
  assert.match(popup, /1500\)/);
});
test('popup does not flag complete HLS and DASH manifests as isolated social tracks', () => {
  for (const extension of ['m3u8', 'mpd']) {
    const result = popupContext.pairTracks([{ url: `https://cdn.example/master.${extension}`, kind: 'video' }], 'https://www.facebook.com/');
    assert.notEqual(result[0].ambiguousSocialTrack, true);
  }
});
test('popup keeps a muxed TikTok MP4 complete without inventing an audio pair', () => {
  const result = popupContext.pairTracks([{ ...popupTrack(videoA, 'video', 1000), muxed: true }], 'https://www.tiktok.com/');
  assert.equal(result[0].ambiguousSocialTrack, undefined);
  assert.equal(result[0].audioUrl, undefined);
});
