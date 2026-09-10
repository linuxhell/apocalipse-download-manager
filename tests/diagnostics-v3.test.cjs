const test = require('node:test');
const assert = require('node:assert/strict');
const vm = require('node:vm');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { webcrypto } = require('node:crypto');
const root = join(__dirname, '..', 'browser-extension');
const load = name => readFileSync(join(root, name), 'utf8');
const session = () => ({ active:true, sessionId:webcrypto.randomUUID(), salt:'test-salt', tabId:7, expiresAt:Date.now()+600000 });
const plain = value => JSON.parse(JSON.stringify(value));
function coreHarness() {
 const c=vm.createContext({ crypto:webcrypto, URL, TextEncoder, Date, console });
 vm.runInContext(load('diagnostics-core.js'),c);return c.ADM_DIAG_CORE;
}
function workerHarness(storage={}) {
 let config={active:false}, online=true, writes=0;const requests=[];
 const c=vm.createContext({crypto:webcrypto,URL,TextEncoder,Date,console,AbortController,setTimeout,clearTimeout,
 chrome:{runtime:{getManifest:()=>({version:'0.3.102'}),getURL:p=>'chrome-extension://test/'+p},
  storage:{local:{async get(defaults){return {...defaults,...storage}},async set(values){writes++;Object.assign(storage,plain(values))}}},
  alarms:{create(){},onAlarm:{addListener(){}}},webRequest:{onResponseStarted:{},onErrorOccurred:{addListener(){}}},
  tabs:{async query(){return [{id:7,url:'https://www.tiktok.com/'}]}}},
 fetch:async (url,options={})=>{
  const body=options.body?JSON.parse(options.body):null;requests.push({url,body});
  if(!online)throw new Error('offline');
  if(url.endsWith('/status'))return {ok:true,json:async()=>plain(config)};
  if(url.endsWith('/control')){if(body.action==='start')config=session();if(body.action==='stop')config.active=false;return {ok:true,json:async()=>plain(config)}}
  if(url.endsWith('/events'))return {ok:true,json:async()=>({ok:true,ackIds:body.events.map(e=>e.id),accepted:body.events.length})};
  throw new Error('unexpected URL');
 }});
 vm.runInContext(load('diagnostics-core.js'),c);vm.runInContext(load('diagnostics-worker.js'),c);
 const message=(value,sender={url:'chrome-extension://test/popup.html'})=>new Promise((resolve,reject)=>{
   const kept=c.ADM_DIAG_WORKER.message(value,sender,resolve);if(!kept)reject(new Error('message unhandled'));
 });
 return {c,storage,requests,message,setOnline:value=>{online=value},writes:()=>writes};
}
const tick=()=>new Promise(resolve=>setTimeout(resolve,30));
test('v3 URL refs are salted and stable; secrets are removed before persistence',async()=>{
 const c=coreHarness();const input={url:'https://user:password@video.example/PRIVATE_PATH?token=SECRET#FRAGMENT',cookie:'cookie-secret',headers:{Authorization:'bearer-secret'},title:'private caption',errorRef:'private text'};
 const a=await c.clean(input,'salt-a'),b=await c.clean(input,'salt-a'),d=await c.clean(input,'salt-b');
 assert.equal(a.url.ref,b.url.ref);assert.notEqual(a.url.ref,d.url.ref);assert.equal(a.url.host,'video.example');
 for(const secret of ['PRIVATE_PATH','SECRET','FRAGMENT','user:password','cookie-secret','bearer-secret','private caption','private text'])assert.ok(!JSON.stringify(a).includes(secret),secret);
 assert.deepEqual(plain(await c.clean(a,'salt-a')),plain(a));
});
test('v3 validates metadata and bounds detail cardinality and depth',async()=>{
 const c=coreHarness(),s=session();assert.equal(await c.record(null,s),null);assert.equal(await c.record({event:'secret sentence'},s),null);
 const traceId=webcrypto.randomUUID();const r=await c.record({event:'popup.click',traceId,detail:{values:Array(100).fill('private')},sequence:3},s,{tabId:7,frameId:0});
 assert.equal(r.traceId,traceId);assert.equal(r.detail.values.length,24);assert.equal(r.tabId,7);assert.equal(r.frameId,0);assert.equal(r.sequence,3);
 assert.equal(c.isActive({...s,expiresAt:Date.now()-1}),false);assert.equal(c.isActive({...s,active:false}),false);
});
test('only extension pages can start a diagnostic session; target tabs are enforced',async()=>{
 const h=workerHarness({pairingToken:'not-exported'});
 const denied=await h.message({type:'ADM_DIAG_CONTROL',action:'start'},{tab:{id:7},frameId:0,url:'https://www.tiktok.com/'});
 assert.equal(denied.ok,false);assert.equal(h.requests.filter(r=>r.url.endsWith('/control')).length,0);
 const config=await h.message({type:'ADM_DIAG_CONTROL',action:'start'});assert.equal(config.tabId,7);
 const make=()=>({sessionId:config.sessionId,id:webcrypto.randomUUID(),event:'capture.test',detail:{cookie:'DO-NOT-STORE'}});
 assert.equal((await h.message({type:'ADM_DIAG_BATCH',events:[make()]},{tab:{id:9},frameId:0})).accepted,0);
 const good=await h.message({type:'ADM_DIAG_BATCH',events:[make()]},{tab:{id:7},frameId:3});assert.equal(good.accepted,1);
 await tick();assert.ok(!JSON.stringify(h.storage.admDiagnosticsV3).includes('DO-NOT-STORE'));
});
test('worker persists before offline delivery and replays acknowledged IDs after restart',async()=>{
 const storage={pairingToken:'not-exported'};const h=workerHarness(storage);const config=await h.message({type:'ADM_DIAG_CONTROL',action:'start'});
 await tick();h.setOnline(false);const id=webcrypto.randomUUID(),traceId=webcrypto.randomUUID();
 await h.message({type:'ADM_DIAG_BATCH',events:[{id,traceId,sessionId:config.sessionId,event:'overlay.unresolved',level:'WARN',detail:{url:'https://video.example/path?secret=SECRET'}}]},{tab:{id:7},frameId:2});
 await tick();assert.ok(storage.admDiagnosticsV3.outbox.some(e=>e.id===id));assert.ok(!JSON.stringify(storage.admDiagnosticsV3).includes('SECRET'));
 const resumed=workerHarness(storage);await tick();await resumed.c.ADM_DIAG_WORKER.flush();
 assert.ok(resumed.requests.some(r=>r.body?.events?.some(e=>e.id===id && e.traceId===traceId)));
 assert.ok(!storage.admDiagnosticsV3.outbox.some(e=>e.id===id));
});
test('network diagnostics report a rejected video response without changing capture policy',async()=>{
 const h=workerHarness({pairingToken:'x'});await h.message({type:'ADM_DIAG_CONTROL',action:'start'});await tick();h.setOnline(false);
 await h.c.ADM_DIAG_WORKER.network({tabId:7,frameId:0,url:'https://cdn.example/private?sig=SECRET',requestId:'a12',type:'xmlhttprequest',statusCode:206,responseHeaders:[{name:'content-type',value:'video/mp4'},{name:'authorization',value:'SECRET'}]},false,'host_not_in_capture_filter');await tick();
 const r=h.storage.admDiagnosticsV3.outbox.find(e=>e.event==='capture.network_decision');
 assert.equal(r.detail.reason,'host_not_in_capture_filter');assert.equal(r.level,'WARN');assert.equal(r.detail.partial,true);assert.equal(r.detail.url.host,'cdn.example');
 assert.ok(!JSON.stringify(r).includes('SECRET'));
});
test('normal mode creates no detailed records or media requests',async()=>{
 const h=workerHarness({pairingToken:'x'});await h.c.ADM_DIAG_WORKER.emit('popup.click',{url:'https://example.com/'});await tick();
 assert.equal(h.requests.filter(r=>r.url.endsWith('/events')).length,0);assert.equal(h.storage.admDiagnosticsV3?.outbox?.length || 0,0);
});
test('diagnostic scripts are packaged in the same isolated contexts, not in MAIN',()=>{
 const manifest=JSON.parse(load('manifest.json'));
 for(const entry of manifest.content_scripts.filter(e=>e.js.includes('content.js'))){assert.deepEqual(entry.js.slice(0,2),['diagnostics-core.js','diagnostics.js']);assert.notEqual(entry.world,'MAIN')}
 assert.ok(!manifest.content_scripts.filter(e=>e.world==='MAIN').some(e=>e.js.includes('diagnostics.js')));
 const html=load('popup.html');assert.ok(html.indexOf('diagnostics.js')<html.indexOf('popup.js'));
});
