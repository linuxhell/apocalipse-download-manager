const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const root = path.resolve(__dirname, '..');
const html = fs.readFileSync(path.join(root,'apps/desktop/ui/ed2k.html'),'utf8');
const script = fs.readFileSync(path.join(root,'apps/desktop/ui/ed2k.js'),'utf8');

class Element {
  constructor(tag, attributes='') {
    this.tagName = tag.toUpperCase(); this.attributes={}; this.dataset={}; this.children=[]; this.textContent=''; this.disabled=false; this.hidden=false; this.value='';
    for (const match of attributes.matchAll(/([\w-]+)="([^"]*)"/g)) this.setAttribute(match[1],match[2]);
  }
  setAttribute(key,value) { this.attributes[key]=String(value); if(key==='id')this.id=value; if(key==='value')this.value=value; if(key.startsWith('data-'))this.dataset[key.slice(5).replace(/-([a-z])/g,(_,v)=>v.toUpperCase())]=value; }
  getAttribute(key) { return this.attributes[key]; }
  append(...children) { this.children.push(...children); }
  replaceChildren(...children) { this.children=children; }
  set innerHTML(_) { throw Error('Untrusted metadata must never be inserted as HTML'); }
}
async function windowFixture({language='pt-BR',serverName='Fixture',connected=false}={}) {
  const all=[...html.matchAll(/<([a-z][\w-]*)\b([^>]*)>/gi)].map(m=>new Element(m[1],m[2]));
  const ids=new Map(all.filter(e=>e.id).map(e=>[e.id,e]));
  const document={hidden:false,documentElement:{lang:'en',dataset:{}},createElement:tag=>new Element(tag),querySelector:selector=>ids.get(selector.slice(1)),querySelectorAll:selector=>{
    if(selector==='#ed2k-downloads button')return ids.get('ed2k-downloads').children.flatMap(r=>r.children.flatMap(td=>td.children));
    if(selector==='#ed2k-shared-files button'||selector==='#ed2k-share-entries button')return ids.get(selector.slice(1,selector.indexOf(' '))).children.flatMap(r=>r.children.flatMap(td=>td.children));
    if(selector==='#ed2k-results button')return ids.get('ed2k-results').children.flatMap(r=>r.children.flatMap(td=>td.children));
    if(selector==='[data-i18n]')return all.filter(e=>e.dataset.i18n);
    if(selector==='[data-tab]')return all.filter(e=>e.dataset.tab);
    if(selector==='[role=tabpanel]')return all.filter(e=>e.attributes.role==='tabpanel');
    if(selector==='.ed2k-setting button, #ed2k-servers button')return [...all.filter(e=>e.id?.startsWith('ed2k-update-')||e.id?.startsWith('ed2k-import-')),...ids.get('ed2k-servers').children.flatMap(r=>r.children.flatMap(td=>td.children))];
    throw Error('Unexpected selector '+selector);
  }};
  const sharedState={entries:[{id:'e1',kind:'directory',path:'/data/<b>x</b>',recursive:true,enabled:true,files:1}],files:[{name:'<img src=x>.bin',size:2048,hash:'b'.repeat(32),link:'ed2k://|file|x.bin|2048|'+'b'.repeat(32)+'|/',path:'/data/x.bin',parts:1,priority:2,requests:4,accepted:2,uploaded:1536}],scan:{running:false,done:0,total:0,current:'',errors:[]},uploads:{active:[{name:'peer-1',address:'192.0.2.9',file:'b'.repeat(32),uploaded:1024,rank:null,seconds:75}],waiting:[{name:'peer-2',address:'192.0.2.10',file:'b'.repeat(32),uploaded:0,rank:1,seconds:5}],totalUploaded:1536,maxActive:3,bytesPerSecond:null,files:[]},settings:{upload_slots:3,upload_kib:0},published:null};
  const overview={servers:[{host:'192.0.2.1',port:4661,name:serverName,description:'Test fixture'}],node_count:20,downloads:[],server:connected?{name:serverName}:null,kad_bootstrap:null};
  const calls=[]; let languageListener, modeListener; const listeners={};
  const window={addEventListener:(name,fn)=>{listeners[name]=fn;},__TAURI__:{core:{invoke:async(command,args)=>{
    calls.push({command,args});
    switch(command){
      case 'get_application_language':return language;
      case 'get_dark_mode':return false;
      case 'ed2k_overview':return overview;
      case 'ed2k_connect':overview.server=overview.servers[0];return null;
      case 'ed2k_disconnect':overview.server=null;return null;
      case 'ed2k_search':return [{name:'<script>untrusted</script>.txt',size:1024,sources:3,hash:'a'.repeat(32),link:'ed2k://|file|test|1024|'+'a'.repeat(32)+'|/'}];
      case 'ed2k_find_sources':return [{client_id:123,host:null,port:4662,requires_callback:true},{client_id:0x04030201,host:'1.2.3.4',port:4662,requires_callback:false}];
      case 'ed2k_add_download':overview.downloads.push({id:'test-id',file:{name:'fixture.bin',size:1024},directory:'C:/Downloads',status:'paused',verified:0,peer:null,queue_rank:null});return 'test-id';
      case 'ed2k_download_action':if(args.action==='resume')overview.downloads[0].status='queued';else if(args.action==='pause')overview.downloads[0].status='paused';else overview.downloads=[];return null;
      case 'ed2k_import_list':return null;
      case 'ed2k_shared_list':return sharedState;
      case 'ed2k_shared_add':return {id:'new',kind:args.kind};
      default:return null;
    }
  }},event:{listen:async(name,fn)=>{if(name==='apocalipse-language-changed')languageListener=fn;else if(name==='apocalipse-mode-changed')modeListener=fn;return()=>{};}}}};
  const cache=new Map([['apocalipse.language','en']]);
  const context={window,document,localStorage:{getItem:k=>cache.get(k),setItem:(k,v)=>cache.set(k,v)},setInterval:()=>1,clearInterval(){},URL,Intl,Date};
  vm.runInNewContext(script.replace('const catalogs =','const catalogs = globalThis.testCatalogs ='),context);
  await new Promise(resolve=>setImmediate(resolve));
  return {ids,all,calls,overview,sharedState,document,context,language:value=>languageListener({payload:value}),listeners,mode:value=>modeListener({payload:value})};
}

test('independent ED2K window uses the saved ADM language and updates while open',async()=>{
  const f=await windowFixture();
  assert.equal(f.document.documentElement.lang,'pt-BR');
  assert.equal(f.ids.get('ed2k-connect').textContent,'Conectar ED2K');
  f.language('zh-CN');assert.equal(f.ids.get('ed2k-connect').textContent,'连接 ED2K');
  f.language('en');assert.equal(f.ids.get('ed2k-connect').textContent,'Connect ED2K');
  const keys=Object.keys(f.context.testCatalogs.en).sort();
  for(const catalog of Object.values(f.context.testCatalogs))assert.deepEqual(Object.keys(catalog).sort(),keys);
  for(const node of f.all.filter(e=>e.dataset.i18n))assert.ok(f.context.testCatalogs.en[node.dataset.i18n],node.dataset.i18n);
});
test('list update rejects HTTP and credentials before invoking the backend',async()=>{
  const f=await windowFixture();
  for(const url of ['http://example.test/server.met','https://user:secret@example.test/server.met']){
    f.ids.get('ed2k-server-url').value=url;f.ids.get('ed2k-update-servers').onclick();
    assert.equal(f.ids.get('ed2k-feedback').dataset.error,'true');
  }
  assert.ok(!f.calls.some(c=>c.command==='ed2k_update_list'));
});
test('ED2K server and result names are plain text and search uses the native command',async()=>{
  const f=await windowFixture({serverName:'<img src=x onerror=bad()>'});
  assert.equal(f.ids.get('ed2k-servers').children[0].children[0].textContent,'<img src=x onerror=bad()>');
  assert.equal(f.ids.get('ed2k-search-submit').disabled,true);
  await f.ids.get('ed2k-connect').onclick();
  assert.equal(f.ids.get('ed2k-search-submit').disabled,false);
  f.ids.get('ed2k-query').value='example';
  f.ids.get('ed2k-search-form').onsubmit({preventDefault(){}});
  await new Promise(resolve=>setImmediate(resolve));
  assert.equal(f.ids.get('ed2k-results').children[0].children[0].textContent,'<script>untrusted</script>.txt');
  assert.equal(f.calls.find(c=>c.command==='ed2k_search').args.query,'example');
});
test('Kad bootstrap is identified as a probe rather than a connected routing session',async()=>{
  const f=await windowFixture();
  assert.match(f.context.testCatalogs.en.kadHint,/does not yet keep a Kad session/);
  assert.match(f.context.testCatalogs['pt-BR'].scopeHint,/Kad persistente ainda está em desenvolvimento/);
  f.ids.get('ed2k-bootstrap').onclick();await new Promise(resolve=>setImmediate(resolve));
  assert.ok(f.calls.some(c=>c.command==='ed2k_bootstrap'));
});

test('opening an independent ED2K window does not show or resize the main window',()=>{
  const code=fs.readFileSync(path.join(root,'apps/desktop/ui/color-mode.js'),'utf8');
  for(const bodyClass of ['link-window','ed2k-window','']){
    const calls=[];let loaded;
    const window={addEventListener:(_,fn)=>{loaded=fn;},__TAURI__:{core:{invoke:async name=>{calls.push(name);}}}};
    const document={documentElement:{dataset:{}},body:{classList:{contains:name=>name===bodyClass}}};
    vm.runInNewContext(code,{window,document,localStorage:{getItem:()=>null},requestAnimationFrame:fn=>fn()});
    loaded();assert.deepEqual(calls,bodyClass?[]:['main_window_ready']);
  }
});

test('ED2K window follows live theme changes and filters source results without remote requests',async()=>{
  const f=await windowFixture({connected:true});
  f.mode(true);assert.equal(f.document.documentElement.dataset.mode,'dark');
  f.mode(false);assert.equal(f.document.documentElement.dataset.mode,'light');
  f.ids.get('ed2k-query').value='example';
  f.ids.get('ed2k-search-form').onsubmit({preventDefault(){}});
  await new Promise(resolve=>setImmediate(resolve));
  const row=f.ids.get('ed2k-results').children[0];
  await row.children[4].children[1].onclick();
  assert.equal(f.calls.find(c=>c.command==='ed2k_find_sources').args.hash,'a'.repeat(32));
  assert.equal(f.ids.get('ed2k-source-panel').hidden,false);
  const sources=f.ids.get('ed2k-source-results').children;
  assert.equal(sources[0].children[0].textContent,'ID 123');
  assert.equal(sources[0].children[2].textContent,'Exige callback do servidor');
  assert.equal(sources[1].children[0].textContent,'1.2.3.4');
  f.ids.get('ed2k-result-filter').value='not found';f.ids.get('ed2k-result-filter').oninput();
  assert.equal(f.ids.get('ed2k-results').children.length,0);
  f.ids.get('ed2k-result-filter').value='UNTRUSTED';f.ids.get('ed2k-result-filter').oninput();
  assert.equal(f.ids.get('ed2k-results').children.length,1);
  assert.equal(f.calls.filter(c=>c.command==='ed2k_search').length,1);
});

test('transfer queue accepts an ED2K link and routes resume, pause and removal to native commands',async()=>{
  const f=await windowFixture({connected:true});
  const link='ed2k://|file|fixture.bin|1024|'+'a'.repeat(32)+'|/';
  f.ids.get('ed2k-file-link').value=link;
  f.ids.get('ed2k-add-form').onsubmit({preventDefault(){}});
  await new Promise(resolve=>setImmediate(resolve));
  assert.equal(f.calls.find(c=>c.command==='ed2k_add_download').args.link,link);
  assert.equal(f.ids.get('panel-transfers').hidden,false);
  let row=f.ids.get('ed2k-downloads').children[0];
  assert.equal(row.children[3].textContent,'Pausado');
  await row.children[5].children[0].onclick();
  assert.equal(f.calls.filter(c=>c.command==='ed2k_download_action').at(-1).args.action,'resume');
  row=f.ids.get('ed2k-downloads').children[0];
  assert.equal(row.children[3].textContent,'Na fila');
  await row.children[5].children[0].onclick();
  assert.equal(f.calls.filter(c=>c.command==='ed2k_download_action').at(-1).args.action,'pause');
  row=f.ids.get('ed2k-downloads').children[0];
  await row.children[5].children[1].onclick();
  assert.equal(f.ids.get('ed2k-downloads').children.length,0);
});

const tick=()=>new Promise(resolve=>setImmediate(resolve));
const plain=value=>JSON.parse(JSON.stringify(value));
function cells(row){return row.children.map(td=>td.textContent);}

test('Shared tab lists what the user chose and treats names as text, never HTML',async()=>{
  const f=await windowFixture({language:'en'});
  f.all.find(e=>e.id==='tab-shared').onclick();await tick();
  assert.ok(f.calls.some(c=>c.command==='ed2k_shared_list'));
  const entry=f.ids.get('ed2k-share-entries').children[0];
  assert.equal(entry.children[0].textContent,'/data/<b>x</b>');
  assert.match(entry.children[1].textContent,/Folder · Include subfolders/);
  const file=f.ids.get('ed2k-shared-files').children[0];
  assert.equal(file.children[0].textContent,'<img src=x>.bin');
  assert.equal(file.children[6].children.length,2,'copy link and open folder');
  assert.equal(f.ids.get('shared-count').textContent,'1 shared files');
  assert.match(f.ids.get('ed2k-publish-status').textContent,/Not offered yet/);
  assert.equal(f.ids.get('panel-shared').hidden,false);
});

test('Shared tab actions call the native commands with the chosen options',async()=>{
  const f=await windowFixture({language:'en'});
  f.all.find(e=>e.id==='tab-shared').onclick();await tick();
  f.ids.get('ed2k-share-recursive').checked=false;
  f.ids.get('ed2k-share-file').onclick();await tick();
  f.ids.get('ed2k-share-recursive').checked=true;
  f.ids.get('ed2k-share-folder').onclick();await tick();
  const adds=f.calls.filter(c=>c.command==='ed2k_shared_add').map(c=>c.args);
  assert.deepEqual(plain(adds),[{kind:'file',recursive:false},{kind:'directory',recursive:true}]);
  f.ids.get('ed2k-share-rescan').onclick();await tick();
  assert.ok(f.calls.some(c=>c.command==='ed2k_shared_rescan'));
  const [pause,stop]=f.ids.get('ed2k-share-entries').children[0].children[4].children;
  pause.onclick();await tick();stop.onclick();await tick();
  assert.deepEqual(plain(f.calls.find(c=>c.command==='ed2k_shared_set_enabled').args),{id:'e1',enabled:false});
  assert.deepEqual(plain(f.calls.find(c=>c.command==='ed2k_shared_remove').args),{id:'e1'});
  const row=f.ids.get('ed2k-shared-files').children[0];
  const select=row.children[3].children[0];
  assert.equal(select.children.length,5);
  select.value='4';select.onchange();await tick();
  assert.deepEqual(plain(f.calls.find(c=>c.command==='ed2k_shared_set_priority').args),{hash:'b'.repeat(32),priority:4});
  row.children[6].children[0].onclick();await tick();
  assert.match(f.calls.find(c=>c.command==='ed2k_copy_link').args.link,/^ed2k:\/\/\|file\|x\.bin\|2048\|/);
  row.children[6].children[1].onclick();await tick();
  assert.deepEqual(plain(f.calls.find(c=>c.command==='ed2k_shared_open_folder').args),{hash:'b'.repeat(32)},'only the hash is sent, never a path');
});

test('hashing progress and scan problems are shown while files are prepared',async()=>{
  const f=await windowFixture({language:'en'});
  Object.assign(f.sharedState.scan,{running:true,done:3,total:10,current:'/data/big/movie.mkv',errors:['/x/gone.bin: not found']});
  f.all.find(e=>e.id==='tab-shared').onclick();await tick();
  assert.equal(f.ids.get('ed2k-scan').hidden,false);
  assert.equal(f.ids.get('ed2k-scan-bar').max,10);assert.equal(f.ids.get('ed2k-scan-bar').value,3);
  assert.match(f.ids.get('ed2k-scan-text').textContent,/Preparing 3 of 10 files… Now: movie\.mkv/);
  assert.equal(f.ids.get('ed2k-scan-errors').hidden,false);
  assert.match(f.ids.get('ed2k-scan-errors').textContent,/1 problem\(s\): \/x\/gone\.bin/);
});

test('Uploads tab shows clients and queue, and validates settings before saving',async()=>{
  const f=await windowFixture({language:'pt-BR'});
  f.all.find(e=>e.id==='tab-uploads').onclick();await tick();
  assert.deepEqual(plain(cells(f.ids.get('ed2k-upload-active').children[0])),['peer-1','192.0.2.9','<img src=x>.bin','1 KiB','1m 15s']);
  assert.deepEqual(plain(cells(f.ids.get('ed2k-upload-waiting').children[0])),['1','peer-2','192.0.2.10','<img src=x>.bin','5s']);
  assert.equal(f.ids.get('upload-total').textContent,'Total enviado: 1,5 KiB');
  assert.equal(f.ids.get('ed2k-upload-slots').value,'3');
  // Out-of-range values never reach the backend.
  f.ids.get('ed2k-upload-slots').value='0';
  f.ids.get('ed2k-upload-form').onsubmit({preventDefault(){}});await tick();
  assert.ok(!f.calls.some(c=>c.command==='ed2k_upload_settings'));
  assert.equal(f.ids.get('ed2k-feedback').dataset.error,'true');
  f.ids.get('ed2k-upload-slots').value='5';f.ids.get('ed2k-upload-kib').value='256';
  f.ids.get('ed2k-upload-form').onsubmit({preventDefault(){}});await tick();
  assert.deepEqual(plain(f.calls.find(c=>c.command==='ed2k_upload_settings').args),{slots:5,kib:256});
});

test('empty upload lists say so, and every new label exists in all three languages',async()=>{
  const f=await windowFixture({language:'en'});
  f.sharedState.uploads.active=[];f.sharedState.uploads.waiting=[];
  f.all.find(e=>e.id==='tab-uploads').onclick();await tick();
  assert.equal(f.ids.get('ed2k-upload-active-summary').textContent,'No active uploads');
  assert.equal(f.ids.get('ed2k-upload-waiting-summary').textContent,'Nobody is waiting');
  const keys=Object.keys(f.context.testCatalogs.en);
  for(const language of ['pt-BR','zh-CN'])assert.deepEqual(keys.filter(k=>!(k in f.context.testCatalogs[language])),[],language+' is missing labels');
  const used=[...html.matchAll(/data-i18n="([^"]+)"/g)].map(m=>m[1]);
  assert.deepEqual([...new Set(used)].filter(k=>!(k in f.context.testCatalogs.en)),[],'HTML labels without a translation');
});
