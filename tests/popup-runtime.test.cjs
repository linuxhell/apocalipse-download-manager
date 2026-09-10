const { test } = require('node:test');
const assert = require('node:assert/strict');
const vm = require('node:vm');
const fs = require('node:fs');
const path = require('node:path');
const source = fs.readFileSync(path.join(__dirname, '../browser-extension/popup-runtime.js'), 'utf8');
function client(sendMessage) {
  const c = vm.createContext({setTimeout, clearTimeout, console,
    chrome: {runtime:{getManifest:()=>({version:'0.3.103'}),sendMessage}}});
  vm.runInContext(source,c);return c.ADM_POPUP;
}
test('popup IPC settles even when a worker retains the channel forever', async () => {
  const ipc = client(() => {});
  await assert.rejects(ipc.runtime({type:'test'},20), e=>e.code==='response_timeout');
});
test('late worker replies do not complete or duplicate a timed-out operation', async () => {
  let callback;const ipc=client((m,cb)=>{callback=cb});let completed=0;
  const result=ipc.runtime({type:'test'},20).then(()=>completed++,()=>completed++);
  await result;callback({ok:true});assert.equal(completed,1);
});
test('no response is not a successful download',async()=>{
 const ipc=client((m,cb)=>cb(undefined));
 await assert.rejects(ipc.runtime({type:'test'}),e=>e.code==='empty_response');
});
test('malformed URL escapes are printable and cannot abort rendering',()=>{
 const ipc=client(()=>{});assert.equal(ipc.safeDecode('%E0%A4%A'),'%E0%A4%A');assert.equal(ipc.safeDecode('hello%20world'),'hello world');
});
test('sync extension-context errors settle and do not escape event handlers',async()=>{
 const ipc=client(()=>{throw new Error('Extension context invalidated')});
 await assert.rejects(ipc.runtime({type:'test'}),e=>e.code==='extension_context_unavailable');
});
