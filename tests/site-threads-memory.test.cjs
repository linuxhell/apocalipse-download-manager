const test = require('node:test'), assert = require('node:assert/strict'), fs = require('node:fs'), path = require('node:path'), vm = require('node:vm');
const ui = n => fs.readFileSync(path.join(__dirname, '../apps/desktop/ui', n), 'utf8');
const app = ui('app.js'), html = ui('index.html');
const rust = fs.readFileSync(path.join(__dirname, '../apps/desktop/src-tauri/src/main.rs'), 'utf8');
function fixture(initial = {}, hook) {
  const elements = Object.fromEntries(['url','task-connections','task-connections-value','remember-site-threads','remember-site-threads-note'].map(id => [id, {value:'', checked:false, disabled:false, addEventListener(event, fn) {this[event]=fn;}}]));
  elements.url.value='https://files.example.test/a.zip'; elements['task-connections'].value='8';
  const rules = new Map(Object.entries(initial)), writes=[], errors=[];
  const context = vm.createContext({URL, Promise, taskConnectionsManuallyChanged:false, document:{querySelector:s=>elements[s.slice(1)]}, t:k=>k, reportUiError:(...a)=>errors.push(a), invoke:async (command, args)=>{
    if (hook) await hook(command,args);
    const host=new URL(args.url).hostname;
    if(command==='set_site_threads'){writes.push({...args});if(args.threads===null)rules.delete(host);else rules.set(host,args.threads);}
    return {host, threads:rules.get(host)||null,remembered:rules.has(host)};
  }});
  const start=app.indexOf('// Site writes are serialized'), end=app.indexOf('document.querySelectorAll("#add")', start);
  vm.runInContext(app.slice(start,end),context);
  const handlers=app.slice(app.indexOf('document.querySelector("#task-connections").oninput ='), app.indexOf('document.querySelector("#analyze").onclick ='));
  vm.runInContext(handlers,context);
  const settle=()=>vm.runInContext('siteThreadsSave',context);
  return {elements,rules,writes,errors,context,settle,refresh:()=>context.refreshSiteThreads(), change(n){const slider=elements['task-connections'];slider.value=String(n);slider.oninput({target:slider});slider.change();}, check(value){elements['remember-site-threads'].checked=value;elements['remember-site-threads'].change();}};
}
test('slider and memory box are visible alongside always-visible advanced options',()=>{
  const advanced=html.match(/<section class="advanced-download">[\s\S]*?<\/section>/)[0];
  assert.ok(!advanced.includes('task-connections'));
  assert.ok(html.indexOf('id="task-connections"')<html.indexOf('<section class="advanced-download">'));
  assert.match(html,/id="task-connections" max="16" min="1" type="range" value="8"/);
  assert.ok(html.indexOf('id="remember-site-threads"')>html.indexOf('id="task-connections"'));
});
test('every language explains immediate save, update and removal',()=>{
  for(const key of ['rememberSiteThreads','rememberSiteThreadsHint','rememberSiteThreadsFor','rememberSiteThreadsOffer']) assert.equal((app.match(new RegExp(`^\\s{4}${key}: "`,'gm'))||[]).length,3,key);
  assert.match(app,/Marque essa caixa para salvar quantas threads este site usará/);
  assert.ok(!app.includes('Untick and start the download to forget it.'));
});
test('checking saves 1, changing updates to 4 then 8, unchecking removes without enqueue',async()=>{
  const f=fixture();await f.refresh();f.change(1);f.check(true);await f.settle();assert.equal(f.rules.get('files.example.test'),1);
  f.change(4);await f.settle();assert.equal(f.rules.get('files.example.test'),4);
  f.change(8);await f.settle();assert.equal(f.rules.get('files.example.test'),8);
  f.check(false);await f.settle();assert.ok(!f.rules.has('files.example.test'));assert.equal(f.errors.length,0);
});
test('opening the site restores the saved slider and checked box',async()=>{
  const f=fixture({'files.example.test':4});await f.refresh();assert.equal(f.elements['task-connections'].value,'4');assert.equal(f.elements['remember-site-threads'].checked,true);
  f.change(8);await f.settle();await f.refresh();assert.equal(f.elements['task-connections'].value,'8');
});
test('rapid changes stay ordered, ending with removal rather than an older save',async()=>{
  let release;const gate=new Promise(r=>release=r);let first=true;
  const f=fixture({},async cmd=>{if(cmd==='set_site_threads'&&first){first=false;await gate;}});
  await f.refresh();f.change(1);f.check(true);f.change(4);f.change(8);f.check(false);release();await f.settle();assert.deepEqual(f.writes.map(x=>x.threads),[1,4,8,null]);assert.equal(f.rules.size,0);
});
test('a stale site lookup cannot replace a slider already edited by the user',async()=>{
  let release;const gate=new Promise(r=>release=r);const f=fixture({'files.example.test':1},async cmd=>{if(cmd==='get_site_threads')await gate;});
  const read=f.refresh();f.change(8);release();await read;assert.equal(f.elements['task-connections'].value,'8');assert.equal(f.elements['remember-site-threads'].checked,false);
});
test('switching hosts never loads the previous hosts preference',async()=>{
  const f=fixture({'files.example.test':1});await f.refresh();f.elements.url.value='https://other.test/b.zip';f.context.resetTaskConnections();await f.refresh();assert.equal(f.elements['task-connections'].value,'8');assert.equal(f.elements['remember-site-threads'].checked,false);
});
test('the selected count is always sent as the explicit download override',()=>{
  assert.match(app,/connectionsOverride: Number\(document.querySelector\("#task-connections"\)\.value\) \|\| 8,/);
  assert.match(rust,/if let Some\(chosen\) = requested \{\s*return Some\(chosen.clamp\(1, MAX_DOWNLOAD_CONNECTIONS\)\);/);
  assert.match(rust,/adaptive_connections: task\.connections_override\.is_none\(\)/);
});
