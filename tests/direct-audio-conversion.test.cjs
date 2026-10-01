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

const fs = require('node:fs');
const vm = require('node:vm');
function analyzeHlsAudio(name) {
  const source = fs.readFileSync(require.resolve('../apps/desktop/ui/app.js'), 'utf8');
  const marker = '    } else if (plan.reason === "hls_manifest") {';
  const start = source.indexOf(marker) + marker.length;
  const end = source.indexOf('    } else if (showDirectAudioConversion(', start);
  assert.ok(start > marker.length && end > start);
  const controls = new Map();
  const node = id => {
    if (!controls.has(id)) controls.set(id, {hidden:false, checked:true, value:'mp3', disabled:false, dataset:{}, options:[], replaceChildren(){this.options=[];}});
    return controls.get(id);
  };
  const select = node('#media-format');
  const fileName = {value:name};
  const context = {
    document:{querySelector:node}, url:{value:'https://example.test/playlist.m3u8'}, fileName,
    pendingMediaKind:'audio', pendingTitle:'Song', pendingThumbnail:null, pendingDuration:null, pendingExpectedSize:null,
    showDirectAudioConversion, t:key=>key,
    option:(control,value,label)=>control.options.push({value,label}),
    updateHlsAudioConversion(){select.value='original';},
    showCapturedPreview(){},
  };
  vm.runInNewContext(source.slice(start,end), context);
  return {select, conversion:node('#hls-audio-conversion')};
}
test('HLS producing MP4 retains its audio choices without the extra conversion checkbox', () => {
  const {select,conversion}=analyzeHlsAudio('song.mp4');
  assert.equal(conversion.hidden,true);
  assert.equal(select.hidden,false);
  assert.ok(select.options.some(option=>option.value==='audio:mp3'));
});
test('HLS producing other audio formats retains the FFmpeg conversion control', () => {
  const {select,conversion}=analyzeHlsAudio('song.m4a');
  assert.equal(conversion.hidden,false);
  assert.equal(select.hidden,true);
});
