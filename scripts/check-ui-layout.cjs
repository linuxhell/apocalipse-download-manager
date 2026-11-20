// Real production row renderer + CSS in an isolated browser, with I/O replaced.
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const assert = require('node:assert/strict');
const {pathToFileURL} = require('node:url');
const {chromium} = require('playwright');
const root = path.resolve(__dirname, '..');
const ui = path.join(root,'apps/desktop/ui');
const out = path.join(root,'test-results/ui-layout');
const app = fs.readFileSync(path.join(ui,'app.js'),'utf8');
const catalogs = app.slice(0,app.indexOf('let locale ='));
const renderer = app.slice(app.indexOf('function renderDownloads('),app.indexOf('function updateSelectionControls('));
const helpers = app.slice(app.indexOf('function stateName('),app.indexOf('function updateSpeeds('));
assert.ok(renderer.includes('createTransferParts('));
fs.mkdirSync(out,{recursive:true});
const temporary = fs.mkdtempSync(path.join(os.tmpdir(),'adm-ui-layout-'));
let css = fs.readFileSync(path.join(ui,'classic.css'),'utf8');
for (const name of ['inter-latin-ext-wght-normal.woff2','inter-latin-wght-normal.woff2']) {
  const bytes=fs.readFileSync(path.join(ui,'assets/fonts',name));
  css=css.replace('url("assets/fonts/'+name+'")','url("data:font/woff2;base64,'+bytes.toString('base64')+'")');
}
let html=fs.readFileSync(path.join(ui,'index.html'),'utf8').replace(/<script\b[^>]*>[\s\S]*?<\/script>/g,'');
html=html.replace('<link href="classic.css" rel="stylesheet"/>','<style>'+css+'</style>');
const fixture=path.join(temporary,'fixture.html');fs.writeFileSync(fixture,html);
const bootstrap = `
${catalogs}
let locale='en',activePage='downloads',activeFilter='all',historyQuery='',downloads=[],lastDownloadRenderSignature='',overallSpeed=0,overallUploadSpeed=0,selectionPointerActive=false;
const selectedIds=new Set(),busyIds=new Set(),speedSamples=new Map();
const t=key=>catalogs[locale]?.[key]||key;
const visibleDownloads=()=>downloads;
const isTorrent=task=>task.source.startsWith('magnet:');
const youtubeThumbnailFallback=()=>null;
const loadPreviewThumbnail=image=>{image.src='data:image/gif;base64,R0lGODlhAQABAAD/ACwAAAAAAQABAAACADs=';};
const updateSelectionControls=()=>{};
const invoke=async name=>name==='get_transfer_parts'?{available:true,parts_count:8,completed_count:3,cells:Array.from({length:8},(_,i)=>({start:i*500,end:(i+1)*500,confirmed:i<3?500:0}))}:false;
window.__TAURI__={core:{invoke}};
${helpers}
${renderer}
window.layoutScenario=(theme,language,page,long,thumbnail)=>{
  locale=language;activePage=page;document.documentElement.lang=language;
  ADM_THEMES.apply(document.documentElement,theme);
  const title=long?('非常长的文件名称_Sem_espacos_4K_documentacao_'+'x'.repeat(1700)): 'arquivo';
  downloads=['downloading','paused','completed',{failed:{message:'network_waiting_for_reconnect'}},'downloading'].map((state,i)=>({
    id:'layout-'+i,state,source:page==='torrents'?'magnet:?xt=urn:btih:example':'https://example.com/file',
    destination:'/Downloads/'+title+(page==='recordings'?'.recording.webm':'.iso'),display_title:title,
    received:i===4?4000:2440,total:page==='recordings'?null:4000,
    thumbnail:thumbnail?'data:image/gif;base64,R0lGODlhAQABAAD/ACwAAAAAAQABAAACADs=':null,
    torrent_seeders:page==='torrents'?12345:null,torrent_leechers:6789,torrent_eta:'123456 s',resume_supported:true
  }));
  downloads.forEach(task=>speedSamples.set(task.id,{speed:123456789}));
  renderDownloads(true);
};
`;

let browser, page;
(async()=>{
  browser=await chromium.launch({headless:true});
  page=await browser.newPage({viewport:{width:980,height:620}});
  const errors=[];page.on('pageerror',error=>errors.push(error.message));
  await page.goto(pathToFileURL(fixture).href);
  for (const name of ['themes.js','color-mode.js','download-progress.js','transfer-parts.js']) await page.addScriptTag({path:path.join(ui,name)});
  await page.addScriptTag({content:bootstrap});
  await page.addScriptTag({path:path.join(ui,'theme-settings.js')});
  await page.evaluate(()=>document.fonts.ready);
  const themes=await page.evaluate(()=>ADM_THEMES.palettes.map(p=>p.id));
  const report=[];
  for (const theme of themes) for (const language of ['en','pt-BR','zh-CN']) for (const width of [980,1280]) for (const category of ['downloads','torrents','recordings']) {
    await page.setViewportSize({width,height:620});
    const result=await page.evaluate(({theme,language,category})=>{
      const rect=node=>{const r=node.getBoundingClientRect();return {x:r.x,y:r.y,right:r.right,bottom:r.bottom,width:r.width,height:r.height};};
      const failures=[];
      for(const thumbnail of [false,true]) {
        window.layoutScenario(theme,language,category,false,thumbnail);
        const baseline=[...document.querySelectorAll('.download-row')].map(row=>({height:rect(row).height,bar:rect(row.querySelector('.task-progress')).width}));
        window.layoutScenario(theme,language,category,true,thumbnail);
        for(const [i,row] of [...document.querySelectorAll('.download-row')].entries()) {
          const info=rect(row.querySelector('.download-info')),bar=rect(row.querySelector('.task-progress')),actions=rect(row.querySelector('.task-primary-actions')),bounds=rect(row),name=row.querySelector('.task-title-line strong');
          if(bar.width<80 || bar.x<info.x-1 || bar.right>info.right+1) failures.push('progress outside its own area');
          if(actions.right>bounds.right+1 || actions.x<info.right-1) failures.push('actions overlap or overflow');
          if(name.clientWidth<20 || getComputedStyle(name).textOverflow!=='ellipsis') failures.push('filename cannot be read or truncated safely');
          if(Math.abs(bounds.height-baseline[i].height)>1 || Math.abs(bar.width-baseline[i].bar)>1) failures.push('long name changes row or progress geometry');
          if(row.scrollWidth>row.clientWidth+1) failures.push('row has horizontal overflow');
          const folder=row.querySelector('.task-folder-action svg');
          if(getComputedStyle(folder).fill!=='rgb(246, 200, 74)') failures.push('folder lost its yellow color');
        }
      }
      return {failures:[...new Set(failures)],viewport:innerHeight};
    },{theme,language,category});
    assert.deepEqual(result.failures,[],`${theme} ${language} ${width} ${category}: ${result.failures.join(', ')}`);
    assert.equal(result.viewport,620);
    report.push({theme,language,width,category,passed:true});
  }
  await page.setViewportSize({width:980,height:620});
  for(const theme of themes) {
    await page.evaluate(theme=>window.layoutScenario(theme,'pt-BR','downloads',true,true),theme);
    if (await page.locator('.task-parts').first().getAttribute('open') === null) await page.locator('.task-parts > summary').first().click();
    await page.locator('.parts-map').first().waitFor();
    assert.equal(await page.evaluate(()=>innerHeight),620);
    const overflow=await page.locator('.task-parts-body').first().evaluate(node=>node.scrollWidth>node.clientWidth+1);
    assert.equal(overflow,false,`${theme}: open part map overflows`);
    await page.screenshot({path:path.join(out,theme+'.png')});
  }
  for(const language of ['en','pt-BR','zh-CN']) {
    await page.evaluate(language=>{document.documentElement.lang=language;window.renderAdmThemes();},language);
    assert.equal(await page.locator('[data-theme-choice]').count(),10);
    assert.ok((await page.locator('#theme-description').textContent()).length>10);
  }
  await page.evaluate(()=>{document.documentElement.lang='pt-BR';window.renderAdmThemes();document.querySelector('#settings-panel').hidden=false;document.querySelector('.panel').hidden=true;document.querySelectorAll('[data-pane]').forEach(n=>n.hidden=n.dataset.pane!=='themes');});
  await page.locator('[data-theme-choice="amethyst"]').click();
  assert.equal(await page.locator('html').getAttribute('data-theme'),'amethyst');
  await page.screenshot({path:path.join(out,'themes-tab.png')});
  assert.deepEqual(errors,[]);
  fs.writeFileSync(path.join(out,'report.json'),JSON.stringify({cases:report.length,report,errors},null,2));
  await browser.close();fs.rmSync(temporary,{recursive:true,force:true});
  console.log(`${report.length} layout scenarios passed; 10 theme screenshots and theme-picker interaction verified.`);
})().catch(async error=>{
  console.error(error);
  fs.writeFileSync(path.join(out,'failure.txt'),String(error.stack||error));
  try { await page?.screenshot({path:path.join(out,'failure.png'),timeout:5000}); } catch {}
  try { await browser?.close(); } catch {}
  process.exit(1);
});
