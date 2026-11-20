const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const root = path.resolve(__dirname, '..');
const themes = require('../browser-extension/themes.js');
const parts = require('../apps/desktop/ui/transfer-parts.js');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');
function luminance(hex) {
  const [r,g,b] = hex.slice(1).match(/../g).map(n => parseInt(n,16)/255).map(c => c <= .04045 ? c/12.92 : ((c+.055)/1.055)**2.4);
  return r*.2126+g*.7152+b*.0722;
}
function contrast(a,b) { const x=luminance(a),y=luminance(b);return (Math.max(x,y)+.05)/(Math.min(x,y)+.05); }
test('all ten complete palettes preserve readable text on their actual surfaces', () => {
  assert.equal(themes.palettes.length, 10);
  for (const {id,colors:c,names} of themes.palettes) {
    for (const locale of ['en','pt-BR','zh-CN']) assert.ok(names[locale], `${id}: ${locale}`);
    for (const [label,fg,bg] of [['text',c.ink,c.bg],['panel',c.ink,c.panel],['secondary',c.muted,c.panel],['selected',c.muted,c.soft],['button',c.on,c.accent],['failed',c.error,c.panel],['paused',c.warning,c.panel],['done',c.done,c.panel]]) {
      assert.ok(contrast(fg,bg)>=4.5, `${id} ${label}: ${contrast(fg,bg)}`);
    }
  }
  assert.equal(read('apps/desktop/ui/themes.js'), read('browser-extension/themes.js'));
});
test('bridge values select trusted palettes, never arbitrary CSS', () => {
  for (const id of ['url(https://example.com)',{accent:'red'},null,'unknown']) assert.equal(themes.resolve(id).id,'white');
  assert.equal(themes.resolve(null,true).id,'black');
  const variables = new Map(), element = {dataset:{},style:{setProperty:(k,v)=>variables.set(k,v)}};
  themes.apply(element,'sapphire');
  assert.equal(element.dataset.mode,'dark');
  assert.equal(variables.get('--text'),themes.resolve('sapphire').colors.ink);
  themes.apply(element,'pearl',true);
  assert.equal(element.dataset.mode,'light');
  assert.equal(variables.get('--p-on-accent'),themes.resolve('pearl').colors.on);
});
test('a late saved-theme reply cannot replace a live desktop theme event', async () => {
  let resolveSaved,changed;
  const saved = new Promise(resolve=>resolveSaved=resolve);
  const document = {documentElement:{dataset:{},style:{setProperty(){}}},body:{classList:{contains:()=>false}}};
  const window = {addEventListener(){},dispatchEvent(){},__TAURI__:{event:{listen:async(_name,fn)=>{changed=fn;}},core:{invoke:()=>saved}}};
  const context = {window,document,ADM_THEMES:themes,localStorage:{getItem:()=>null,setItem(){}},Event:class{}};
  vm.runInNewContext(read('apps/desktop/ui/color-mode.js'),context);
  await Promise.resolve();
  changed({payload:'emerald'});
  resolveSaved('white');
  await new Promise(resolve=>setImmediate(resolve));
  assert.equal(document.documentElement.dataset.theme,'emerald');
});
test('popup restores and updates the theme while retaining the last choice offline', async () => {
  const stored={desktopTheme:'sapphire'},writes=[];let changed;
  const document={documentElement:{dataset:{},style:{setProperty(){}}}};
  const chrome={storage:{local:{get(defaults,callback){const result={...defaults,...stored};if(callback)callback(result);return Promise.resolve(result);},async set(v){Object.assign(stored,v);writes.push(v);}},onChanged:{addListener(fn){changed=fn;}}}};
  const context={document,chrome,ADM_THEMES:themes,localStorage:{getItem:()=>null,setItem(){}}};
  vm.runInNewContext(read('browser-extension/popup-theme.js'),context);
  assert.equal(document.documentElement.dataset.theme,'sapphire');
  await context.syncPopupDesktopTheme({theme:'sand'});
  assert.equal(document.documentElement.dataset.theme,'sand');
  assert.equal(writes[0].desktopTheme,'sand');
  await context.syncPopupDesktopTheme({language:'en'});
  assert.equal(document.documentElement.dataset.theme,'sand');
  changed({desktopTheme:{newValue:'copper'}},'local');
  assert.equal(document.documentElement.dataset.theme,'copper');
});
test('grouped part cells distinguish confirmed bytes without inventing active connections', () => {
  assert.equal(parts.cellState({start:0,end:10,confirmed:10}),'complete');
  assert.equal(parts.cellState({start:0,end:10,confirmed:3}),'partial');
  assert.equal(parts.cellState({start:10,end:10,confirmed:10}),'pending');
  assert.equal(parts.cellState({start:0,end:10,confirmed:0}),'pending');
  const keys=Object.keys(parts.copy.en).sort();
  for (const locale of ['pt-BR','zh-CN']) {
    assert.deepEqual(Object.keys(parts.copy[locale]).sort(),keys);
    for (const key of keys) assert.ok(parts.copy[locale][key]);
  }
});
test('themes are isolated from webpage main-world scripts and included in recovery', () => {
  const manifest=JSON.parse(read('browser-extension/manifest.json'));
  for (const item of manifest.content_scripts) {
    if (item.world==='MAIN') assert.ok(!item.js.includes('themes.js'));
    if (item.js.includes('content.js')) assert.ok(item.js.indexOf('themes.js')<item.js.indexOf('content.js'));
  }
  assert.match(read('browser-extension/background.js'),/const recoveryFiles = \[\s*"themes.js"/);
});
