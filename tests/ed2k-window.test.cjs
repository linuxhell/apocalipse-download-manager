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
    if(selector==='#ed2k-results button')return ids.get('ed2k-results').children.flatMap(r=>r.children.flatMap(td=>td.children));
    if(selector==='[data-i18n]')return all.filter(e=>e.dataset.i18n);
    if(selector==='[data-tab]')return all.filter(e=>e.dataset.tab);
    if(selector==='[role=tabpanel]')return all.filter(e=>e.attributes.role==='tabpanel');
    if(selector==='.ed2k-setting button, #ed2k-servers button')return [...all.filter(e=>e.id?.startsWith('ed2k-update-')||e.id?.startsWith('ed2k-import-')),...ids.get('ed2k-servers').children.flatMap(r=>r.children.flatMap(td=>td.children))];
    throw Error('Unexpected selector '+selector);
  }};
  const overview={servers:[{host:'192.0.2.1',port:4661,name:serverName,description:'Test fixture'}],node_count:20,server:connected?{name:serverName}:null,kad_bootstrap:null};
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
      case 'ed2k_import_list':return null;
      default:return null;
    }
  }},event:{listen:async(name,fn)=>{if(name==='apocalipse-language-changed')languageListener=fn;else if(name==='apocalipse-mode-changed')modeListener=fn;return()=>{};}}}};
  const cache=new Map([['apocalipse.language','en']]);
  const context={window,document,localStorage:{getItem:k=>cache.get(k),setItem:(k,v)=>cache.set(k,v)},setInterval:()=>1,clearInterval(){},URL,Intl,Date};
  vm.runInNewContext(script.replace('const catalogs =','const catalogs = globalThis.testCatalogs ='),context);
  await new Promise(resolve=>setImmediate(resolve));
  return {ids,all,calls,overview,document,context,language:value=>languageListener({payload:value}),listeners,mode:value=>modeListener({payload:value})};
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
  assert.match(f.context.testCatalogs['pt-BR'].scopeHint,/ainda estão em desenvolvimento/);
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
