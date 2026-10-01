const {test} = require('node:test');
const assert = require('node:assert/strict');
const {showDirectAudioConversion} = require('../apps/desktop/ui/audio-format-policy.js');
test('direct MP3 from the report offers conversion even without a media-kind hint', () => {
  assert.equal(showDirectAudioConversion('https://serv2.y2dl.space/dl/fixture.mp3?token=redacted', 'song.mp3', null), true);
});
test('MP4 never shows the extra FFmpeg conversion control, including audio-only MP4', () => {
  assert.equal(showDirectAudioConversion('https://example.test/audio.mp4?title=song.mp3', 'song.mp4', 'audio'), false);
  assert.equal(showDirectAudioConversion('https://example.test/get', 'song.MP4', 'audio'), false);
  assert.equal(showDirectAudioConversion('https://example.test/song%2Emp4', 'song.mp3', 'audio'), false);
});
test('audio formats and captured opaque audio offer conversion; unrelated files do not', () => {
  for (const extension of ['m4a','aac','ogg','wav','flac','opus']) assert.equal(showDirectAudioConversion(`https://example.test/file.${extension}`, `file.${extension}`, null), true);
  assert.equal(showDirectAudioConversion('https://example.test/get', 'file', 'audio'), true);
  assert.equal(showDirectAudioConversion('https://example.test/file.zip?title=song.mp3', 'file.zip', null), false);
});
