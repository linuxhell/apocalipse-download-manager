const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');
const vm = require('node:vm');

const source = readFileSync(join(__dirname, '../browser-extension/background.js'), 'utf8');
const start = source.indexOf('function mp4TrackInfo');
const end = source.indexOf('\nfunction mediaStartUrl', start);
const context = vm.createContext({ Uint8Array, DataView, BigInt, Number, Set, String });
vm.runInContext(`${source.slice(start, end)}\nglobalThis.inspect = mp4TrackInfo;`, context);

const box = (type, ...payloads) => {
  const size = 8 + payloads.reduce((total, value) => total + value.length, 0);
  const output = Buffer.alloc(size);
  output.writeUInt32BE(size, 0); output.write(type, 4, 4, 'ascii');
  let offset = 8;
  for (const payload of payloads) { payload.copy(output, offset); offset += payload.length; }
  return output;
};
const mvhd = (seconds) => {
  const body = Buffer.alloc(20);
  body.writeUInt32BE(1000, 12); body.writeUInt32BE(seconds * 1000, 16);
  return box('mvhd', body);
};
const track = (handler) => {
  const body = Buffer.alloc(12);
  body.write(handler, 8, 4, 'ascii');
  return box('trak', box('mdia', box('hdlr', body)));
};

test('MP4 inspector distinguishes video, audio and complete muxed resources', () => {
  const video = context.inspect(box('moov', mvhd(30), track('vide')));
  const audio = context.inspect(box('moov', mvhd(30), track('soun')));
  const muxed = context.inspect(box('moov', mvhd(44), track('vide'), track('soun')));
  assert.deepEqual(JSON.parse(JSON.stringify(video)), { kind: 'video', duration: 30 });
  assert.deepEqual(JSON.parse(JSON.stringify(audio)), { kind: 'audio', duration: 30 });
  assert.deepEqual(JSON.parse(JSON.stringify(muxed)), { kind: 'muxed', duration: 44 });
});
