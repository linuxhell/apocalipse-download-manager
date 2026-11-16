const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { webcrypto } = require('node:crypto');
const { test } = require('node:test');
const vm = require('node:vm');
const script = readFileSync(process.env.ADM_CONTENT_SCRIPT || join(__dirname, '../browser-extension/content.js'), 'utf8');

test('Facebook unresolved download falls back to the exact player recording path', () => {
  assert.match(script, /overlay_download_stream_capture/);
  assert.match(script, /if \(isFacebookVideo && canRecord && recordButton\)/);
  assert.match(script, /facebook_srcobject_without_complete_resource/);
  assert.match(script, /facebook_unresolved_recording_fallback/);
  assert.match(script, /recordButton\.click\(\)/);
});

test('Facebook extractor parse failures expose a localized recording fallback', () => {
  const rust = readFileSync(join(__dirname, '../apps/desktop/src-tauri/src/main.rs'), 'utf8');
  const app = readFileSync(join(__dirname, '../apps/desktop/ui/app.js'), 'utf8');
  assert.match(rust, /cannot parse data/);
  assert.match(rust, /facebook_direct_download_unavailable_use_recording/);
  assert.match(app, /The Facebook extractor failed to read this Reel/);
  assert.match(app, /O extrator do Facebook falhou ao ler este Reel/);
  assert.match(app, /Facebook 提取器无法读取此 Reel/);
  assert.match(app, /t\("facebookRecordingFallback"\)/);
  assert.match(app, /warnedFacebookRecordingFallbacks/);
});

// Execute the real content script and its installed click handler. Only browser
// APIs/DOM geometry are mocked; URL selection and the outgoing payload are real.
function page({ url = 'https://www.facebook.com/reel/123456789', source = 'https://video.fbcdn.net/track.mp4?bytestart=0&byteend=999', permalink = null, network = [], readableBlob = false, recordable = false, shadow = false, clipping = false, theme = "void", extraVideos = [], redditAttributes = {}, redditPostPresent = true, redditPlayerAttributes = {}, redditComment = false, mediaState = {}, recordingClass = null, nowMs = 0, messageHook = null } = {}) {
  const sent = [], fetched = [], appended = [], listeners = [];
  const timers = new Map();
  const clock = { now: nowMs };
  let nextTimer = 0;
  const location = new URL(url);
  const rect = { left: 20, top: 40, right: 500, bottom: 600, width: 480, height: 560 };
  const post = { parentElement: null, getBoundingClientRect: () => rect, querySelectorAll: () => [] };
  const video = {
    tagName: 'VIDEO', dataset: {}, isConnected: true, currentSrc: source, src: source,
    title: 'Synthetic complete reel', poster: 'https://images.example/poster.jpg', duration: 30,
    parentElement: post, innerHTML: '', getBoundingClientRect: () => rect,
    getAttribute: () => null,
    ...(recordable ? { captureStream: () => ({ getTracks: () => [] }) } : {}),
    querySelector: selector => selector.includes('a[href') && permalink ? { href: permalink } : null,
    querySelectorAll: () => [], closest: selector => selector.includes('article') ? post : null,
    addEventListener() {}, removeEventListener() {},
    ...mediaState,
  };
  const videos = [video, ...extraVideos.map(extra => ({ ...video, dataset: {}, ...extra }))];
  const postAttributes = { permalink, 'post-title': 'Exact Reddit post title', ...redditAttributes };
  const redditPost = { getAttribute: name => postAttributes[name] ?? null };
  const player = { getAttribute: name => redditPlayerAttributes[name] ?? null,
    closest: selector => selector === 'shreddit-comment' && redditComment ? {}
      : selector === 'shreddit-post' && redditPostPresent ? redditPost : selector === 'shreddit-player' ? player : null };
  player.shadowRoot = { host: player, querySelectorAll: selector => selector === 'video' || selector === 'video,audio' ? [video] : [] };
  if (shadow) { video.getRootNode = () => player.shadowRoot; post.getRootNode = () => player.shadowRoot; }
  const element = tag => ({ tagName: tag.toUpperCase(), style: { setProperty(name, value) { this[name] = value; } }, dataset: {}, offsetWidth: 80,
    addEventListener(type, handler) { this[type] = handler; }, remove() { this.removed = true; } });
  const document = {
    title: 'Synthetic complete reel', fullscreenElement: null,
    documentElement: { append(node) { appended.push(node); } },
    createElement: element, addEventListener() {}, elementFromPoint: () => video,
    querySelector: selector => selector === 'video' ? video : null,
    querySelectorAll: selector => selector === 'shreddit-player' && shadow ? [player]
      : selector === 'video' || selector === 'video,audio' ? (shadow ? videos.slice(1) : videos)
      : selector === '.apocalipse-media-download' ? appended.filter(node => !node.removed && String(node.className).includes('apocalipse-media-download')) : [],
  };
  const context = vm.createContext({
    URL, Blob, Uint8Array, crypto: webcrypto, console, Date, document, location,
    ...(recordable ? { MediaRecorder: recordingClass || class {
      static isTypeSupported() { return true; }
      constructor() { this.state = 'inactive'; }
      start() { this.state = 'recording'; }
      stop() { this.state = 'inactive'; void this.onstop?.(); }
    } } : {}),
    navigator: { language: 'pt-BR', userAgent: 'Synthetic browser' },
    innerWidth: 1200, innerHeight: 800, scrollX: 0, scrollY: 0,
    ...(clipping ? { getComputedStyle: () => ({ overflowX: "visible", overflowY: "auto" }) } : {}),
    addEventListener(type, handler) { if (type === "scroll") listeners.push({ scroll: handler }); }, removeEventListener() {}, postMessage() {},
    setTimeout(fn, delay) { const id = ++nextTimer; timers.set(id, { fn, delay }); return id; },
    clearTimeout(id) { timers.delete(id); }, setInterval: () => 1, clearInterval() {},
    MutationObserver: class { observe() {} },
    performance: { now: () => clock.now, getEntriesByType: () => network.map(item => ({ ...item, name: item.url })), getEntriesByName: () => [] },
    fetch: async target => {
      fetched.push(target);
      if (!readableBlob) throw new Error('synthetic unreadable blob');
      return { ok: true, blob: async () => new Blob(['synthetic track'], { type: 'video/mp4' }) };
    },
    chrome: {
      storage: { local: { get(defaults, callback) { callback({ ...defaults, desktopTheme: theme }); } }, onChanged: { addListener() {} } },
      runtime: {
        id: 'synthetic-extension-id',
        onMessage: { addListener(handler) { listeners.push(handler); } },
        sendMessage(message, callback) {
          sent.push(message);
          const injected = messageHook?.(message, { video, postAttributes });
          const result = injected ?? (message.type === 'APOCALIPSE_RECENT_TAB_MEDIA' ? { media: network }
            : message.type === 'APOCALIPSE_BLOB_BEGIN' ? { uploadId: 'synthetic-upload' }
              : message.type === 'APOCALIPSE_SELECT_HLS' ? { url: message.urls[0], requestUrls: message.urls }
              : { ok: true, target: 'desktop' });
          if (callback) callback(result);
          return Promise.resolve(result);
        },
      },
    },
  });
  context.window = context; context.top = context;
  vm.runInContext(readFileSync(join(__dirname, '../browser-extension/tiktok-identity.js'), 'utf8'), context);
  vm.runInContext(script.replace(/\}\)\(\);\s*$/, 'globalThis.testHooks = { installOverlays, collect, titleInfoFor, activeOverlays, waitForRecordingPlayback: typeof waitForRecordingPlayback === "function" ? waitForRecordingPlayback : null };\n})();'), context);
  context.testHooks.installOverlays();
  const button = appended.find(node => node.className === 'apocalipse-media-download'
    && context.ApocalipseTikTokIdentity.videoFor(node) === video)
    || appended.find(node => node.className === 'apocalipse-media-download');
  assert.ok(button, 'the actual overlay must be installed');
  return {
    sent, fetched, location, video, appended, player, setNow: value => { clock.now = value; },
    setRedditAttributes: attributes => Object.assign(postAttributes, attributes),
    activePlayers: () => Array.from(context.testHooks.activeOverlays.keys()),
    button,
    recordButton: button[Symbol.for('apocalipse.recordButton')]
      || appended.find(node => node.className === 'apocalipse-media-download apocalipse-media-record'),
    refresh: () => context.testHooks.installOverlays(),
    waitForPlayback: ms => context.testHooks.waitForRecordingPlayback(video, ms),
    fireTimeout: ms => {
      const matching = [...timers].filter(([, timer]) => timer.delay === ms);
      assert.ok(matching.length, `expected pending ${ms} ms timer`);
      for (const [id, timer] of matching) { timers.delete(id); timer.fn(); }
    },
    hasTimeout: ms => [...timers.values()].some(timer => timer.delay === ms),
    zeroWrapper: () => { post.getBoundingClientRect = () => ({ left: 0, right: 0, top: 0, bottom: 0 }); for (const entry of listeners) entry.scroll?.(); },
    clipTo: top => { post.getBoundingClientRect = () => ({ left: 0, right: 800, top, bottom: top + 200 }); for (const entry of listeners) entry.scroll?.(); },
    scrollTo: top => { rect.top = top; rect.bottom = top + rect.height; for (const entry of listeners) entry.scroll?.(); },
    imageTitle: () => context.testHooks.titleInfoFor({ tagName: "IMG", closest: () => redditPost }),
    click: () => button.click({ preventDefault() {}, stopPropagation() {} }),
    scan: () => context.testHooks.collect(),
    downloads: () => sent.filter(message => message.type === 'APOCALIPSE_DOWNLOAD'),
  };
}
const tracks = [
  { url: 'https://video.fbcdn.net/audio-disguised.mp4?bytestart=0&byteend=999', contentType: 'video/mp4', capturedAt: 1000 },
  { url: 'https://audio.fbcdn.net/another-reel.mp4', contentType: 'audio/mp4', capturedAt: 1001 },
];

test('Reel overlay uses the same page URL as popup scan, not a misleading CDN track', async () => {
  const p = page({ network: tracks });
  const popup = p.scan().find(item => item.url === p.location.href);
  assert.ok(popup.pageExtractor);
  assert.equal(popup.previewUrl, p.video.currentSrc);
  assert.ok(!p.scan().some(item => item.url === p.video.currentSrc));
  await p.click();
  assert.equal(p.downloads().length, 1);
  const overlay = p.downloads()[0].item;
  assert.equal(overlay.url, popup.url);
  assert.equal(overlay.title, popup.title);
  assert.equal(overlay.kind, 'video');
  assert.equal(overlay.audioUrl, null);
  assert.equal(overlay.requestUrls.length, 0);
  assert.ok(!p.sent.some(message => message.type === 'APOCALIPSE_RECENT_TAB_MEDIA'));
});

test('even a readable Facebook blob cannot bypass the complete-page extractor', async () => {
  const p = page({ source: 'blob:https://www.facebook.com/synthetic', readableBlob: true, network: tracks });
  await p.click();
  assert.equal(p.downloads().length, 1);
  assert.equal(p.downloads()[0].item.url, p.location.href);
  assert.equal(p.fetched.length, 0);
  assert.ok(!p.sent.some(message => message.type.startsWith('APOCALIPSE_BLOB_')));
});

test('feed-card permalink wins over unrelated recent network traffic', async () => {
  const permalink = 'https://www.facebook.com/reel/222333444/';
  const p = page({ url: 'https://www.facebook.com/', permalink, network: tracks });
  await p.click();
  assert.equal(p.downloads()[0].item.url, permalink);
  assert.equal(p.downloads()[0].item.audioUrl, null);
});

test('a reused Reel player resolves the current permalink on every click', async () => {
  const p = page({ network: tracks });
  await p.click();
  p.location.href = 'https://www.facebook.com/reel/999888777';
  await p.click();
  assert.deepEqual(p.downloads().map(message => message.item.url), [
    'https://www.facebook.com/reel/123456789', 'https://www.facebook.com/reel/999888777',
  ]);
});

test('watch and shared-video page URLs keep the extractor route', async () => {
  for (const url of ['https://www.facebook.com/watch/?v=123456789', 'https://www.facebook.com/share/r/SyntheticId/']) {
    const p = page({ url, network: tracks });
    await p.click();
    assert.equal(p.downloads()[0].item.url, url);
    assert.equal(p.downloads()[0].item.audioUrl, null);
  }
});

test('TikTok permalink wins over a potentially incomplete captured media track', async () => {
  const url = 'https://v16.tiktok.com/video/tos/synthetic/?mime_type=video_mp4';
  const p = page({ url: 'https://www.tiktok.com/@synthetic/video/123456789', source: url, network: [{ url, contentType: 'video/mp4' }] });
  await p.click();
  assert.equal(p.downloads()[0].item.url, 'https://www.tiktok.com/@synthetic/video/123456789');
  assert.equal(p.downloads()[0].item.audioUrl, null);
});

test('TikTok generic feed pairs a captured video track with its audio track', async () => {
  const video = 'https://v16.tiktok.com/video/tos/synthetic/?mime_type=video_mp4';
  const audio = 'https://v16.tiktok.com/audio/tos/synthetic/';
  const unrelatedVideo = 'https://v16.tiktok.com/video/tos/another/?mime_type=video_mp4';
  const unrelatedAudio = 'https://v16.tiktok.com/audio/tos/another/';
  const p = page({ url: 'https://www.tiktok.com/', source: video, network: [
    { url: unrelatedVideo, contentType: 'video/mp4', capturedAt: 5000 },
    { url: unrelatedAudio, contentType: 'audio/mp4', capturedAt: 5001 },
    { url: video, contentType: 'video/mp4', capturedAt: 1000 },
    { url: audio, contentType: 'audio/mp4', capturedAt: 1001 },
  ] });
  await p.click();
  assert.equal(p.downloads()[0].item.url, video);
  assert.equal(p.downloads()[0].item.audioUrl, audio);
});

test('a scrolled TikTok feed rebinds the same overlay to the new player source', async () => {
  const firstVideo = 'https://v16.tiktok.com/video/tos/first/?mime_type=video_mp4';
  const firstAudio = 'https://v16.tiktok.com/audio/tos/first/';
  const secondVideo = 'https://v16.tiktok.com/video/tos/second/?mime_type=video_mp4';
  const secondAudio = 'https://v16.tiktok.com/audio/tos/second/';
  const network = [
    { url: firstVideo, contentType: 'video/mp4', capturedAt: 1000 },
    { url: firstAudio, contentType: 'audio/mp4', capturedAt: 1001 },
  ];
  const p = page({ url: 'https://www.tiktok.com/', source: firstVideo, network });
  await p.click();
  p.video.currentSrc = secondVideo;
  p.video.src = secondVideo;
  network.unshift(
    { url: secondVideo, contentType: 'video/mp4', capturedAt: 5000 },
    { url: secondAudio, contentType: 'audio/mp4', capturedAt: 5001 },
  );
  await p.click();
  assert.deepEqual(p.downloads().map(message => [message.item.url, message.item.audioUrl]), [
    [firstVideo, firstAudio], [secondVideo, secondAudio],
  ]);
});

test('YouTube page extraction and ordinary direct HTTP downloads are unchanged', async () => {
  for (const [url, source, expected] of [
    ['https://www.youtube.com/watch?v=synthetic', 'https://cdn.example/track.mp4', 'https://www.youtube.com/watch?v=synthetic'],
    ['https://site.example/page', 'https://cdn.example/complete.mp4', 'https://cdn.example/complete.mp4'],
  ]) {
    const p = page({ url, source });
    await p.click();
    assert.equal(p.downloads()[0].item.url, expected);
  }
});

test('every supported YouTube player immediately exposes only yt-dlp Download', async () => {
  for (const url of [
    'https://www.youtube.com/watch?v=synthetic123',
    'https://m.youtube.com/watch?v=synthetic123',
    'https://music.youtube.com/watch?v=synthetic123',
    'https://www.youtube.com/shorts/synthetic123',
    'https://www.youtube.com/live/synthetic123',
  ]) {
    const p = page({ url, source: 'blob:https://www.youtube.com/player', recordable: true });
    const visible = p.appended.filter(node => !node.removed && String(node.className).includes('apocalipse-media-download'));
    assert.equal(visible.length, 1, `${url} must have one action`);
    assert.equal(visible[0].className, 'apocalipse-media-download');
    assert.equal(visible[0].hidden, false);
    await p.click();
    assert.equal(p.downloads()[0].item.url, url);
    assert.equal(p.fetched.length, 0, `${url} must not probe the internal blob`);
  }
});

test('Facebook srcObject alone never classifies an ordinary Reel as recording-only', () => {
  assert.match(script, /recordingOnly:\s*Boolean\(element\.srcObject && isSponsoredFacebookPlayer\(element\)\)/);
  assert.doesNotMatch(script, /recordingOnly:\s*Boolean\(element\.srcObject && \/\(\^\|\\\.\)facebook/);
});

test('ordinary direct HTTP players avoid Record while Facebook keeps its recording fallback', () => {
  assert.match(script, /const hasDirectHttpMedia = \/\^https\?:\/i\.test\(liveMediaUrl\)/);
  assert.match(script, /\(!hasDirectHttpMedia \|\| isFacebookVideo\)/);
});

test('Facebook sponsored detection is strict and only filters popup inventory', () => {
  assert.match(script, /const facebookSponsoredEvidence = \(element\) =>/);
  assert.match(script, /exact_header_label_same_article/);
  assert.match(script, /explicit_attribute_same_article/);
  assert.match(script, /rect\.bottom <= playerRect\.top \+ 18/);
  assert.match(script, /facebook\.popup_sponsored_filtered/);
  assert.match(script, /facebook\.sponsored_home_allowed/);
  assert.doesNotMatch(script, /facebook\.overlay_skipped_sponsored/);
});

test('recording follows player pauses without writing dead timeline gaps', () => {
  assert.match(script, /recording_paused_with_player/);
  assert.match(script, /recorder\.pause\(\)/);
  assert.match(script, /recording_resumed_with_player/);
  assert.match(script, /recorder\.resume\(\)/);
  assert.match(script, /recorder && recorder\.state !== "inactive"/);
});

test('recording keeps the source player alive and stops at its real end', () => {
  assert.match(script, /playbackWatch = setInterval/);
  assert.match(script, /resumeRecorderOnRealProgress/);
  assert.match(script, /element\.addEventListener\("timeupdate", resumeRecorderOnRealProgress\)/);
  assert.match(script, /currentTime <= pausedAtMediaTime \+ 0\.04/);
  assert.match(script, /currentTime >= duration - 0\.25/);
  assert.match(script, /recording_reached_media_end/);
  assert.match(script, /if \(playbackWatch\) clearInterval\(playbackWatch\)/);
});

test('late HTTP sources replace Record-only state with a working direct Download', async () => {
  const p = page({ url: 'https://media.example.test/watch/123', source: '', recordable: true });
  assert.equal(p.button.hidden, true);
  assert.equal(p.recordButton.hidden, false);
  const direct = 'https://cdn.example.test/movie.mp4?signature=synthetic';
  p.video.currentSrc = direct;
  p.refresh();
  assert.equal(p.button.hidden, false);
  assert.equal(p.recordButton.hidden, true);
  await p.click();
  assert.equal(p.downloads().length, 1);
  assert.equal(p.downloads()[0].item.url, direct);
  assert.equal(p.sent.filter(m => m.type === 'APOCALIPSE_BLOB_BEGIN').length, 0);
});

test('source changes refresh both actions without duplicating the overlay', () => {
  const p = page({ url: 'https://media.example.test/watch/123', recordable: true });
  const original = p.appended.filter(node => String(node.className).includes('apocalipse-media-download'));
  assert.equal(p.recordButton.hidden, true);
  p.video.currentSrc = ''; p.video.src = '';
  p.refresh();
  assert.equal(p.button.hidden, true);
  assert.equal(p.recordButton.hidden, false);
  assert.deepEqual(p.appended.filter(node => String(node.className).includes('apocalipse-media-download')), original);
});

test('the exact HTTP player source outranks an unrelated expired HLS playlist', async () => {
  const direct = 'https://media.example.test/get-video?id=123&token=synthetic';
  const p = page({ url: 'https://media.example.test/watch/123', source: direct,
    network: [{ url: 'https://ads.example.test/old-preview.m3u8', contentType: 'application/vnd.apple.mpegurl' }] });
  await p.click();
  assert.equal(p.downloads().length, 1);
  assert.equal(p.downloads()[0].item.url, direct);
  assert.equal(p.sent.filter(m => m.type === 'APOCALIPSE_SELECT_HLS').length, 0);
});

test('an empty player does not inherit another video manifest', () => {
  const p = page({ url: 'https://media.example.test/watch/123', source: '', recordable: true,
    network: [{ url: 'https://ads.example.test/preview.m3u8', contentType: 'application/vnd.apple.mpegurl' }] });
  assert.equal(p.button.hidden, true);
  assert.equal(p.recordButton.hidden, false);
});

test('a direct HLS player analyzes only its own manifest', async () => {
  const direct = 'https://media.example.test/current.m3u8';
  const p = page({ url: 'https://media.example.test/watch/123', source: direct,
    network: [{ url: 'https://ads.example.test/other.m3u8', contentType: 'application/vnd.apple.mpegurl' }] });
  await p.click();
  const analysis = p.sent.find(m => m.type === 'APOCALIPSE_SELECT_HLS');
  assert.deepEqual(Array.from(analysis.urls), [direct]);
  assert.equal(p.downloads()[0].item.url, direct);
});

test('recording startup blocks repeated clicks and recovers from unresolved playback', async () => {
  const p = page({ url: 'https://media.example.test/watch/123', source: 'blob:https://media.example.test/synthetic', recordable: true });
  let playCalls = 0;
  p.video.loop = true;
  p.video.play = () => { playCalls++; return new Promise(() => {}); };
  const event = { preventDefault() {}, stopPropagation() {} };
  const starting = p.recordButton.click(event);
  assert.equal(p.recordButton.disabled, true);
  assert.match(p.recordButton.textContent, /Starting|Iniciando|正在启动/);
  await p.recordButton.click(event);
  assert.equal(playCalls, 1);
  p.fireTimeout(8000);
  await starting;
  assert.equal(p.recordButton.disabled, false);
  assert.equal(p.video.loop, true);
  assert.equal(p.recordButton.textContent, '⚠');
  await new Promise(setImmediate);
  assert.ok(p.sent.some(m => m.eventName === 'recording_start_failed'));
  assert.equal(p.sent.filter(m => m.type === 'APOCALIPSE_BLOB_BEGIN').length, 0);
});

test('successful playback clears its recording-start timeout', async () => {
  const p = page();
  p.video.play = async () => {};
  await p.waitForPlayback(37);
  assert.equal(p.hasTimeout(37), false);
});

test('a playing video outranks a centered empty placeholder and paused previews', async () => {
  const direct = 'https://cdn.example.test/actual-video.mp4';
  const p = page({ url: 'https://media.example.test/watch/123', source: '', recordable: true,
    extraVideos: [
      { currentSrc: direct, src: direct, readyState: 4, paused: false, muted: false, volume: 1 },
      { currentSrc: 'https://cdn.example.test/preview.mp4', src: '', readyState: 4, paused: true, muted: true },
    ] });
  assert.equal(p.activePlayers().length, 1);
  assert.equal(p.activePlayers()[0].currentSrc, direct);
  assert.equal(p.button.hidden, false);
  assert.equal(p.recordButton.hidden, true);
  await p.click();
  assert.equal(p.downloads()[0].item.url, direct);
});

test('recording a playing stream does not rewind it or wait on a redundant play promise', async () => {
  const p = page({ url: 'https://media.example.test/watch/123', source: 'blob:https://media.example.test/current', recordable: true });
  let playCalls = 0;
  let stopped = false;
  p.video.play = () => { playCalls++; return new Promise(() => {}); };
  p.video.currentTime = 32;
  p.video.readyState = 4;
  p.video.paused = false;
  p.video.ended = false;
  p.video.captureStream = () => ({ getTracks: () => [{ kind: 'video', stop() { stopped = true; }, addEventListener() {} }] });
  const starting = p.recordButton.click({ preventDefault() {}, stopPropagation() {} });
  assert.equal(playCalls, 0);
  assert.equal(p.video.currentTime, 32);
  await starting;
  assert.equal(p.recordButton.disabled, false);
  assert.match(p.recordButton.textContent, /Stop and save|Parar e salvar|停止并保存/);
  assert.equal(p.sent.filter(m => m.type === 'APOCALIPSE_BLOB_BEGIN').length, 1);
  assert.equal(stopped, false);
});

test('page CSS cannot expose media controls marked hidden', () => {
  assert.match(script, /\.apocalipse-media-download\[hidden\]\{display:none!important\}/);
});

test('a large paused main player outranks small autoplay previews and empty overlays', async () => {
  const direct = 'https://cdn.example.test/main.mp4';
  const p = page({ url: 'https://media.example.test/watch/123', source: '', recordable: true,
    extraVideos: [
      { currentSrc: direct, src: direct, readyState: 4, paused: true,
        getBoundingClientRect: () => ({ left: 20, top: 100, right: 860, bottom: 580, width: 840, height: 480 }) },
      { currentSrc: 'https://cdn.example.test/preview.mp4', src: '', readyState: 4, paused: false, muted: true,
        getBoundingClientRect: () => ({ left: 20, top: 300, right: 140, bottom: 380, width: 120, height: 80 }) },
    ] });
  assert.equal(p.activePlayers().length, 1);
  assert.equal(p.activePlayers()[0].currentSrc, direct);
  await p.click();
  assert.equal(p.downloads()[0].item.url, direct);
});

test('recording seals each segment and resumes only after real media progress', () => {
  assert.match(script, /recorder\.requestData\?\.\(\)/);
  assert.match(script, /pausedAtMediaTime = Number\(element\.currentTime\)/);
  assert.match(script, /recorder\.state !== "paused" \|\| element\.paused/);
  assert.match(script, /recorder\.resume\(\)/);
});


test('Facebook sponsored videos hide both Download and Record', () => {
  assert.match(script, /button\.hidden = !canDownload \|\| Boolean\(isFacebookVideo && facebookSponsoredEvidence\(element\)\)/);
  assert.match(script, /const sponsoredHomeVideo = Boolean\(isFacebookVideo && facebookSponsoredEvidence\(element\)\)/);
  assert.match(script, /button\.hidden = sponsoredHomeVideo \|\| !liveCanDownload/);
  assert.match(script, /recordButton\.hidden = sponsoredHomeVideo \|\| outsideVisibleArea/);
  assert.match(script, /sponsoredRecordOnly:/);
});

test('normal Facebook srcObject Reels resolve Download before any recording fallback', () => {
  assert.doesNotMatch(script, /facebook_stream_without_permalink/);
  const immediate = script.indexOf('const immediateFacebookUrl = isFacebookVideo ? facebookUrlFor(element) : null;');
  const reveal = script.indexOf('await revealFacebookUrl(element)');
  const fallback = script.indexOf('facebook_unresolved_recording_fallback');
  assert.ok(immediate >= 0 && reveal > immediate, 'normal Facebook stream must attempt permalink discovery');
  assert.ok(fallback > reveal, 'recording fallback must happen only after permalink resolution fails');
});

test('generic video-feed pages (3+ visible videos) keep one selected player', () => {
  assert.match(script, /let activeGenericFeedVideo = null;/);
  assert.match(script, /socialPlatform\(\) === "generic" && !isFacebookReelsPage && !isInstagramReelsPage && !isTikTokPage/);
  assert.match(script, /if \(visibleVideos\.length >= 3\) \{/);
  assert.match(script, /if \(activeGenericFeedVideo && element\.tagName === "VIDEO" && element !== activeGenericFeedVideo\) \{/);
  assert.match(script, /overlay_skipped_feed_preview/);
});

test('generic feed suppression never engages on Facebook, Instagram or TikTok, whose own reel logic is unchanged', () => {
  const start = script.indexOf('let activeGenericFeedVideo = null;');
  const end = script.indexOf('queryMediaElements("video,audio").forEach', start);
  assert.ok(start >= 0 && end > start);
  const block = script.slice(start, end);
  assert.match(block, /!isFacebookReelsPage/);
  assert.match(block, /!isInstagramReelsPage/);
  assert.match(block, /!isTikTokPage/);
});


test('Reddit shadow video uses the exact native video HLS instead of a silent DASH track', async () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOs/', shadow: true,
    permalink: '/r/UFOs/comments/abc123/exact_post/', source: 'https://v.redd.it/exact/DASH_720.mp4' });
  const expected = 'https://v.redd.it/exact/HLSPlaylist.m3u8';
  assert.ok(p.scan().find(item => item.url === expected && item.pageExtractor === false));
  await p.click();
  const item = p.downloads()[0].item;
  assert.equal(item.url, expected);
  assert.equal(item.pageExtractor, false);
  assert.equal(item.title, "Exact Reddit post title");
  assert.equal(item.audioUrl, null);
  assert.deepEqual(Array.from(item.requestUrls), [expected]);
});

test('Reddit MSE shadow video keeps Record and sends the permalink before reading blobs', async () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOs/', shadow: true, recordable: true,
    permalink: '/r/UFOs/comments/def456/second_post/', source: 'blob:https://www.reddit.com/player' });
  assert.ok(p.appended.some(node => String(node.className).includes('apocalipse-media-record')));
  await p.click();
  assert.equal(p.downloads()[0].item.url, 'https://www.reddit.com/r/UFOs/comments/def456/second_post/');
  assert.equal(p.fetched.length, 0);
});


test('Reddit images use their post title instead of subreddit page title', () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOs/', shadow: true, permalink: '/r/UFOs/comments/abc123/title/' });
  assert.equal(p.imageTitle().title, 'Exact Reddit post title');
  assert.equal(p.imageTitle().source, 'reddit_post');
});

 test('media overlay hides offscreen players and returns when scrolled back', () => {
  const p = page({ recordable: true });
  const controls = p.appended.filter(node => String(node.className).includes('apocalipse-media-download'));
  assert.equal(controls.length, 2);
  assert.ok(controls.every(node => !node.hidden));
  p.scrollTo(-700);
  assert.ok(controls.every(node => node.hidden));
  p.scrollTo(850);
  assert.ok(controls.every(node => node.hidden));
  p.scrollTo(40);
  assert.ok(controls.every(node => !node.hidden));
});

test('media overlay follows clipping by an internal scroll container', () => {
  const p = page({ recordable: true, clipping: true });
  const controls = p.appended.filter(node => String(node.className).includes('apocalipse-media-download'));
  p.clipTo(650);
  assert.ok(controls.every(node => node.hidden));
  p.clipTo(40);
  assert.ok(controls.every(node => !node.hidden));
});

test('overlays are always white with black text, whatever theme an older version stored', () => {
  const p = page({ recordable: true, theme: 'samurai' });
  const controls = p.appended.filter(node => String(node.className).includes('apocalipse-media-download'));
  assert.equal(controls.length, 2);
  for (const control of controls) {
    assert.equal(control.style['--apocalipse-accent'], '#ffffff');
    assert.equal(control.style['--apocalipse-foreground'], '#111111');
  }
});

test('visible media stays available inside layout wrappers without a clipping box', () => {
  const p = page({ recordable: true, clipping: true });
  const controls = p.appended.filter(node => String(node.className).includes('apocalipse-media-download'));
  p.zeroWrapper();
  assert.ok(controls.every(node => !node.hidden));
  p.scrollTo(-700);
  assert.ok(controls.every(node => node.hidden));
});


test('Reddit post without permalink uses its own content-href even on a different post page', async () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOB/comments/other/address_bar_post/', shadow: true,
    redditAttributes: { 'content-href': '/r/UFOB/comments/target/current_player/' },
    source: 'blob:https://www.reddit.com/current', recordable: true });
  await p.click();
  assert.equal(p.downloads()[0]?.item.url, 'https://www.reddit.com/r/UFOB/comments/target/current_player/');
  assert.equal(p.downloads()[0].item.pageExtractor, true);
  assert.equal(p.downloads()[0].item.audioUrl, null);
  assert.equal(p.fetched.length, 0);
});

test('Reddit post fullname resolves its own extractor page when permalink metadata is absent', async () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOB/comments/other/address_bar_post/', shadow: true,
    redditAttributes: { id: 't3_abc123' }, source: 'blob:https://www.reddit.com/current', recordable: true });
  await p.click();
  assert.equal(p.downloads()[0]?.item.url, 'https://www.reddit.com/comments/abc123/');
  assert.ok(p.scan().some(item => item.url === 'https://www.reddit.com/comments/abc123/' && item.pageExtractor));
});

test('Reddit standalone shadow player uses a dedicated post page only when it is the sole player', async () => {
  const url = 'https://www.reddit.com/r/UFOB/comments/1xwx0jm/revisiting_1000_uap096/';
  const p = page({ url, shadow: true, redditPostPresent: false, source: 'blob:https://www.reddit.com/current', recordable: true });
  assert.equal(p.button.hidden, false);
  await p.click();
  assert.equal(p.downloads()[0]?.item.url, url);
  assert.equal(p.downloads()[0].item.pageExtractor, true);
  assert.deepEqual(Array.from(p.downloads()[0].item.requestUrls), []);
  assert.equal(p.fetched.length, 0);
});

test('Reddit standalone player never inherits the address bar post when multiple players exist', async () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOB/comments/abc123/page_post/', shadow: true,
    redditPostPresent: false, source: 'blob:https://www.reddit.com/current', recordable: true,
    extraVideos: [{ currentSrc: 'blob:https://www.reddit.com/other' }] });
  assert.equal(p.button.hidden, true);
  p.refresh();
  assert.equal(p.button.hidden, true);
  await p.click();
  assert.equal(p.downloads().length, 0);
});

test('Reddit unresolved feed player keeps Download hidden until its own identity becomes available', () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOB/', shadow: true,
    mediaState: { readyState: 4, videoWidth: 480, videoHeight: 360 },
    source: 'blob:https://www.reddit.com/current', recordable: true });
  assert.equal(p.button.hidden, true);
  assert.equal(p.recordButton.hidden, false);
  p.refresh();
  assert.equal(p.button.hidden, true);
  p.setRedditAttributes({ permalink: '/r/UFOB/comments/abc123/now_resolved/' });
  p.refresh();
  assert.equal(p.button.hidden, false);
  p.setRedditAttributes({ permalink: null });
  p.refresh();
  assert.equal(p.button.hidden, true);
});

test('Reddit invalid owner URLs do not fall back to another post in the address bar', async () => {
  for (const permalink of ['https://reddit.com.evil.test/r/UFOB/comments/abc123/', 'https://example.com/r/UFOB/comments/abc123/', 'javascript:alert(1)', '/r/UFOB/']) {
    const p = page({ url: 'https://www.reddit.com/r/UFOB/comments/other/page_post/', permalink, shadow: true,
      source: 'blob:https://www.reddit.com/current', recordable: true });
    assert.equal(p.button.hidden, true, permalink);
    await p.click();
    assert.equal(p.downloads().length, 0, permalink);
  }
});

test('Reddit post-id identity is re-read after a player is recycled for another post', async () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOB/', shadow: true,
    redditAttributes: { id: 't3_abc123' }, source: 'blob:https://www.reddit.com/first', recordable: true });
  p.setRedditAttributes({ id: 't3_def456' });
  p.video.currentSrc = p.video.src = 'blob:https://www.reddit.com/second';
  p.refresh();
  await p.click();
  assert.equal(p.downloads()[0]?.item.url, 'https://www.reddit.com/comments/def456/');
});


test('Reddit post fullname takes precedence over stale permalink metadata after recycling', async () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOB/', shadow: true,
    permalink: '/r/UFOB/comments/abc123/stale_post/', redditAttributes: { id: 't3_def456' },
    source: 'blob:https://www.reddit.com/current', recordable: true });
  await p.click();
  assert.equal(p.downloads()[0]?.item.url, 'https://www.reddit.com/comments/def456/');
});

test('Reddit standalone feed player cannot use a subreddit as an extractor page', async () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOB/', shadow: true, redditPostPresent: false,
    source: 'blob:https://www.reddit.com/current', recordable: true });
  assert.equal(p.button.hidden, true);
  await p.click();
  assert.equal(p.downloads().length, 0);
});

test('Reddit short comments routes work while malformed fullnames cannot create a post identity', async () => {
  const p = page({ url: 'https://old.reddit.com/comments/abc123/', shadow: true,
    redditPostPresent: false, source: 'blob:https://www.reddit.com/current', recordable: true });
  await p.click();
  assert.equal(p.downloads()[0]?.item.url, 'https://old.reddit.com/comments/abc123/');
  for (const id of ['t1_abc123', 't3_abc123/../../other', 'abc123', 't3_']) {
    const invalid = page({ url: 'https://www.reddit.com/r/UFOB/', shadow: true, redditAttributes: { id },
      source: 'blob:https://www.reddit.com/current', recordable: true });
    assert.equal(invalid.button.hidden, true, id);
    await invalid.click();
    assert.equal(invalid.downloads().length, 0, id);
  }
});


test('Reddit changing posts during thumbnail capture cancels the stale handoff', async () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOB/', shadow: true,
    redditAttributes: { id: 't3_abc123' }, source: 'blob:https://www.reddit.com/current', recordable: true,
    messageHook(message, { postAttributes }) {
      if (message.type === 'APOCALIPSE_CAPTURE_VISIBLE_THUMBNAIL') postAttributes.id = 't3_def456';
    } });
  p.video.poster = '';
  await p.click();
  assert.equal(p.downloads().length, 0);
  await p.click();
  assert.equal(p.downloads()[0]?.item.url, 'https://www.reddit.com/comments/def456/');
});


const zeroVideoBox = () => ({ left: 0, top: 0, right: 0, bottom: 0, width: 0, height: 0 });

test('Reddit diagnostic regression: one visible empty player plus two zero-box videos keeps post Download', async () => {
  for (const redditPostPresent of [false, true]) {
    const url = 'https://www.reddit.com/r/UFOB/comments/1xwx0jm/revisiting_1000_uap096/';
    const p = page({ url, shadow: true, redditPostPresent, source: '', recordable: true,
      mediaState: { readyState: 0, videoWidth: 0, videoHeight: 0, duration: NaN, paused: false },
      extraVideos: [
        { currentSrc: '', src: '', getBoundingClientRect: zeroVideoBox },
        { currentSrc: 'https://v.redd.it/hidden/DASH_720.mp4', src: '', getBoundingClientRect: zeroVideoBox },
      ] });
    assert.equal(p.button.hidden, false, 'hidden implementation videos cannot suppress the dedicated post');
    assert.equal(p.recordButton.hidden, true, 'an empty player has no frames to record');
    await p.click();
    assert.equal(p.downloads()[0]?.item.url, url);
    assert.equal(p.downloads()[0].item.pageExtractor, true);
    assert.equal(p.downloads()[0].item.audioUrl, null);
    assert.ok(!p.sent.some(m => m.type === 'APOCALIPSE_RECENT_TAB_MEDIA'));
  }
});

test('Reddit empty player does not call play or start an eight-second recording timeout', async () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOB/comments/abc123/post/', shadow: true,
    redditPostPresent: false, source: '', recordable: true,
    mediaState: { readyState: 0, videoWidth: 0, videoHeight: 0, duration: NaN } });
  let playCalls = 0;
  p.video.play = () => { playCalls++; return new Promise(() => {}); };
  await p.recordButton.click({ preventDefault() {}, stopPropagation() {} });
  assert.equal(playCalls, 0);
  assert.equal(p.hasTimeout(8000), false);
  assert.equal(p.sent.filter(m => m.type === 'APOCALIPSE_BLOB_BEGIN').length, 0);
});

test('Reddit Record becomes available only after the player has a source and decoded frames', () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOB/comments/abc123/post/', shadow: true,
    redditPostPresent: false, source: '', recordable: true,
    mediaState: { readyState: 0, videoWidth: 0, videoHeight: 0 } });
  assert.equal(p.recordButton.hidden, true);
  p.video.currentSrc = p.video.src = 'blob:https://www.reddit.com/loaded';
  p.refresh();
  assert.equal(p.recordButton.hidden, true);
  Object.assign(p.video, { readyState: 4, videoWidth: 480, videoHeight: 360 });
  p.refresh();
  assert.equal(p.button.hidden, false);
  assert.equal(p.recordButton.hidden, false);
  p.video.currentSrc = p.video.src = '';
  p.refresh();
  assert.equal(p.recordButton.hidden, true);
});

test('Reddit player post-id resolves the owning post when the address bar points to another post', async () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOB/comments/other/address_bar/', shadow: true,
    redditPostPresent: false, redditPlayerAttributes: { 'post-id': 't3_def456' },
    source: '', recordable: true });
  await p.click();
  assert.equal(p.downloads()[0]?.item.url, 'https://www.reddit.com/comments/def456/');
});

test('Reddit fallback still rejects two sized players when the second is scrolled offscreen', async () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOB/comments/abc123/page/', shadow: true,
    redditPostPresent: false, source: '', recordable: true,
    extraVideos: [{ currentSrc: '', src: '', getBoundingClientRect: () => ({ left: 20, top: 2000, right: 500, bottom: 2360, width: 480, height: 360 }) }] });
  assert.equal(p.button.hidden, true);
  await p.click();
  assert.equal(p.downloads().length, 0);
});


test('Reddit loaded player records chunks and finalizes through the bridge without restarting playback', async () => {
  class Recorder {
    static isTypeSupported() { return true; }
    constructor() { this.state = 'inactive'; }
    start() {
      this.state = 'recording';
      this.ondataavailable?.({ data: new Blob(['synthetic-reddit-video-chunk'], { type: 'video/webm' }) });
    }
    stop() { this.state = 'inactive'; void this.onstop?.(); }
  }
  const p = page({ url: 'https://www.reddit.com/r/UFOB/comments/abc123/post/', shadow: true,
    redditPostPresent: false, source: 'blob:https://www.reddit.com/loaded', recordable: true,
    recordingClass: Recorder, mediaState: { readyState: 4, videoWidth: 480, videoHeight: 360, paused: false, currentTime: 12 } });
  let playCalls = 0, stopped = false;
  p.video.play = () => { playCalls++; throw new Error('must not restart playing video'); };
  const tracks = [{ kind: 'video', stop() { stopped = true; }, addEventListener() {} }];
  p.video.captureStream = () => ({ getTracks: () => tracks });
  const event = { preventDefault() {}, stopPropagation() {} };
  assert.equal(p.recordButton.hidden, false);
  await p.recordButton.click(event);
  assert.equal(playCalls, 0);
  assert.equal(p.video.currentTime, 12);
  await p.recordButton.click(event);
  await new Promise(setImmediate);
  const chunk = p.sent.find(m => m.type === 'APOCALIPSE_BLOB_CHUNK');
  assert.ok(chunk);
  assert.equal(Buffer.from(chunk.request.data, 'hex').toString(), 'synthetic-reddit-video-chunk');
  assert.equal(p.sent.filter(m => m.type === 'APOCALIPSE_BLOB_BEGIN').length, 1);
  assert.equal(p.sent.filter(m => m.type === 'APOCALIPSE_BLOB_END').length, 1);
  assert.ok(p.sent.some(m => m.eventName === 'recording_completed'));
  assert.equal(stopped, true);
});

test('Reddit conflicting owning-post and player identities never download the address bar post', async () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOB/comments/page123/page/', shadow: true,
    redditAttributes: { id: 't3_abc123' }, redditPlayerAttributes: { 'post-id': 't3_def456' },
    source: 'blob:https://www.reddit.com/current', recordable: true });
  assert.equal(p.button.hidden, true);
  await p.click();
  assert.equal(p.downloads().length, 0);
});

test('Reddit follows a slotted player to its owning post instead of using the unrelated page URL', async () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOB/comments/page123/page/', shadow: true,
    redditPostPresent: false, source: '', recordable: true });
  const owningPost = { getAttribute: name => name === 'id' ? 't3_def456' : null };
  p.video.assignedSlot = { closest: selector => selector === 'shreddit-post' ? owningPost : null };
  p.refresh();
  await p.click();
  assert.equal(p.downloads()[0]?.item.url, 'https://www.reddit.com/comments/def456/');
});


test('Reddit comment video never downloads its parent image post', async () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOB/comments/1xwx0jm/revisiting_1000_uap096/',
    shadow: true, redditComment: true, redditPostPresent: false,
    redditPlayerAttributes: { 'post-id': 't3_1xwx0jm' },
    source: 'https://v.redd.it/commentvideo123/DASH_720.mp4?source=fallback', recordable: true });
  await p.click();
  const item = p.downloads()[0]?.item;
  assert.equal(item?.url, 'https://v.redd.it/commentvideo123/HLSPlaylist.m3u8');
  assert.equal(item.pageExtractor, false);
  assert.equal(item.ambiguousSocialTrack, false);
  assert.equal(item.audioUrl, null);
  assert.deepEqual(Array.from(item.requestUrls), [item.url]);
  assert.ok(!p.downloads().some(m => m.item.url.includes('/comments/1xwx0jm/')));
});

test('Reddit MSE comment uses only the media identity of its own player component', async () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOB/comments/imagepost/title/', shadow: true,
    redditComment: true, redditPostPresent: false,
    redditPlayerAttributes: { 'post-id': 't3_imagepost', src: 'https://v.redd.it/owncomment123' },
    source: 'blob:https://www.reddit.com/current', recordable: true,
    network: [{ url: 'https://v.redd.it/unrelated/HLSPlaylist.m3u8', contentType: 'application/vnd.apple.mpegurl' }] });
  await p.click();
  assert.equal(p.downloads()[0]?.item.url, 'https://v.redd.it/owncomment123/HLSPlaylist.m3u8');
  assert.equal(p.downloads()[0].item.pageExtractor, false);
  assert.ok(!p.sent.some(m => m.type === 'APOCALIPSE_RECENT_TAB_MEDIA'));
});

test('Reddit preserves a signed HLS URL from the exact player', async () => {
  const url = 'https://v.redd.it/owncomment123/HLSPlaylist.m3u8?a=synthetic&f=hd';
  const p = page({ url: 'https://www.reddit.com/r/UFOB/comments/imagepost/title/', shadow: true,
    redditComment: true, source: url, recordable: true });
  await p.click();
  assert.equal(p.downloads()[0]?.item.url, url);
  assert.equal(p.downloads()[0].item.pageExtractor, false);
});

test('Reddit packaged component metadata resolves one exact native video and ignores the JPEG preview', async () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOB/comments/imagepost/title/', shadow: true,
    redditComment: true, source: 'blob:https://www.reddit.com/current', recordable: true,
    redditPlayerAttributes: { 'packaged-media-json': JSON.stringify({ playback: { hls: 'https://v.redd.it/owncomment123/HLSPlaylist.m3u8' }, preview: 'https://preview.redd.it/923ltg54fs1i1.jpeg' }) } });
  await p.click();
  assert.equal(p.downloads()[0]?.item.url, 'https://v.redd.it/owncomment123/HLSPlaylist.m3u8');
});

test('Reddit unresolved comment never inherits the parent post or an arbitrary recent response', async () => {
  const p = page({ url: 'https://www.reddit.com/r/UFOB/comments/imagepost/title/', shadow: true,
    redditComment: true, redditPostPresent: false, redditPlayerAttributes: { 'post-id': 't3_imagepost' },
    source: 'blob:https://www.reddit.com/current', recordable: true,
    network: [{ url: 'https://v.redd.it/unrelated/HLSPlaylist.m3u8' }] });
  assert.equal(p.button.hidden, true);
  await p.click();
  assert.equal(p.downloads().length, 0);
});

test('Reddit comment rejects conflicting component media IDs and image or lookalike media sources', async () => {
  for (const attributes of [
    { src: 'https://preview.redd.it/923ltg54fs1i1.jpeg' },
    { src: 'https://v.redd.it.evil.test/owncomment123/HLSPlaylist.m3u8' },
    { src: 'https://user:secret@v.redd.it/owncomment123/HLSPlaylist.m3u8' },
    { src: 'https://v.redd.it/owncomment123', 'hls-url': 'https://v.redd.it/othercomment456/HLSPlaylist.m3u8' },
  ]) {
    const p = page({ url: 'https://www.reddit.com/r/UFOB/comments/imagepost/title/', shadow: true,
      redditComment: true, redditPlayerAttributes: attributes, source: 'blob:https://www.reddit.com/current', recordable: true });
    assert.equal(p.button.hidden, true);
    await p.click();
    assert.equal(p.downloads().length, 0);
  }
});

test('Reddit recycled comment re-reads its own component source on each click', async () => {
  const attrs = { src: 'https://v.redd.it/firstcomment123' };
  const p = page({ url: 'https://www.reddit.com/r/UFOB/comments/imagepost/title/', shadow: true,
    redditComment: true, redditPlayerAttributes: attrs, source: 'blob:https://www.reddit.com/first', recordable: true });
  await p.click();
  attrs.src = 'https://v.redd.it/secondcomment456';
  p.video.currentSrc = p.video.src = 'blob:https://www.reddit.com/second';
  p.refresh();
  await p.click();
  assert.deepEqual(p.downloads().map(m => m.item.url), [
    'https://v.redd.it/firstcomment123/HLSPlaylist.m3u8', 'https://v.redd.it/secondcomment456/HLSPlaylist.m3u8' ]);
});


const playingRedditBlob = { readyState: 4, videoWidth: 220, videoHeight: 480, duration: 19.933333, currentTime: 0.2, paused: false };
const redditMaster = 'https://v.redd.it/commentvideo123/HLSPlaylist.m3u8?a=signed';
function blobHlsPage(options = {}) {
  return page({ url: 'https://www.reddit.com/r/UFOB/comments/imagepost/title/', shadow: true,
    redditComment: true, redditPostPresent: false, source: 'blob:https://www.reddit.com/current',
    recordable: true, mediaState: { ...playingRedditBlob }, nowMs: 9070,
    network: [{ url: redditMaster, startTime: 8845 }],
    messageHook: m => m.type === 'APOCALIPSE_VERIFY_REDDIT_HLS' ? { ok: true, url: m.url, requestUrls: [m.url] } : undefined,
    ...options });
}

test('Reddit diagnostic 14:06: Blob playback binds its recent master after worker verification', async () => {
  const p = blobHlsPage({ extraVideos: [
    { currentSrc: '', src: '', getBoundingClientRect: zeroVideoBox },
    { currentSrc: 'https://v.redd.it/standby/DASH_720.mp4', src: '', getBoundingClientRect: zeroVideoBox },
  ] });
  assert.equal(p.button.hidden, true);
  await new Promise(setImmediate);
  assert.equal(p.button.hidden, false);
  assert.equal(p.recordButton.hidden, false);
  await p.click();
  assert.equal(p.downloads()[0]?.item.url, redditMaster);
  assert.equal(p.downloads()[0].item.pageExtractor, false);
  assert.equal(p.downloads()[0].item.ambiguousSocialTrack, false);
});

test('Reddit HLS duration rejection leaves Record available and Download unavailable', async () => {
  const p = blobHlsPage({ messageHook: m => m.type === 'APOCALIPSE_VERIFY_REDDIT_HLS' ? { ok: false, url: m.url } : undefined });
  await new Promise(setImmediate);
  assert.equal(p.button.hidden, true);
  assert.equal(p.recordButton.hidden, false);
});

test('Reddit stale or competing master playlists cannot bind to a Blob player', async () => {
  for (const network of [
    [{ url: redditMaster, startTime: 1000 }],
    [{ url: redditMaster, startTime: 8845 }, { url: 'https://v.redd.it/othercomment456/HLSPlaylist.m3u8', startTime: 8850 }],
  ]) {
    const p = blobHlsPage({ network });
    await new Promise(setImmediate);
    assert.equal(p.button.hidden, true);
    assert.ok(!p.sent.some(m => m.type === 'APOCALIPSE_VERIFY_REDDIT_HLS'));
  }
});

test('Reddit concurrent loaded players prevent network playlist association', async () => {
  const p = blobHlsPage({ extraVideos: [{ ...playingRedditBlob, currentSrc: 'blob:https://www.reddit.com/other' }] });
  await new Promise(setImmediate);
  assert.equal(p.button.hidden, true);
  assert.ok(!p.sent.some(m => m.type === 'APOCALIPSE_VERIFY_REDDIT_HLS'));
});

test('Reddit source replacement rejects an in-flight verification and old source requests', async () => {
  let finish;
  const p = blobHlsPage({ messageHook: m => m.type === 'APOCALIPSE_VERIFY_REDDIT_HLS' ? new Promise(resolve => { finish = () => resolve({ ok: true, url: m.url }); }) : undefined });
  assert.ok(finish);
  p.setNow(9500);
  p.video.currentSrc = p.video.src = 'blob:https://www.reddit.com/new';
  finish();
  await new Promise(setImmediate);
  p.refresh();
  assert.equal(p.button.hidden, true);
  assert.equal(p.sent.filter(m => m.type === 'APOCALIPSE_VERIFY_REDDIT_HLS').length, 1);
});
