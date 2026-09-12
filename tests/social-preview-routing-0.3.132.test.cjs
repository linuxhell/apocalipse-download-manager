const assert = require('node:assert/strict');
const test = require('node:test');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const ext = path.join(__dirname, '../browser-extension');
const src = name => fs.readFileSync(path.join(ext, name), 'utf8');

function rect(left=0, top=0, width=400, height=700) {
  return { left, top, width, height, right:left+width, bottom:top+height };
}
function node({tag='DIV', r=rect(), parent=null, attrs={}, videos=[], click=null}={}) {
  return {
    tagName:tag, parentElement:parent, isConnected:true, hidden:false, textContent:'', title:'', currentSrc:'', src:'', duration:12,
    getBoundingClientRect(){ return {...r}; },
    getAttribute(name){ return attrs[name] ?? null; },
    setAttribute(){}, removeAttribute(){}, dispatchEvent(){},
    querySelectorAll(selector){ if (selector==='video') return videos; return []; },
    matches(selector){ return tag==='ARTICLE' && selector.includes('article'); },
    closest(){ return null; }, addEventListener(){}, removeEventListener(){}, click(){ if(click) click(); },
  };
}

test('0.3.139 manifest version is exact', () => {
  const manifest = JSON.parse(src('manifest.json'));
  assert.equal(manifest.version, '0.3.139');
  assert.match(manifest.version_name, /Resolve and close TikTok share dialogs/);
});

test('0.3.132 resolved Preview and Download are dispatched explicitly, never synthetic re-click', () => {
  const popup = src('popup-social-player-resolution.js');
  assert.match(popup, /dispatchResolvedAction/);
  assert.match(popup, /actionIntent: 'preview'/);
  assert.match(popup, /workerRoute: 'preview_media'/);
  assert.match(popup, /actionIntent: 'download'/);
  assert.match(popup, /type: 'APOCALIPSE_DOWNLOAD'/);
  assert.doesNotMatch(popup, /forwardClick\s*=/);
  assert.doesNotMatch(popup, /button\.click\(\);\s*\n\s*};\s*\n\s*document\.addEventListener\('click'/);
});

test('0.3.132 V3 resolver uses the fresh scan source even when visibility geometry would reject it', async () => {
  const video = node({tag:'VIDEO', r:rect(20,1200,400,700)});
  video.currentSrc = 'blob:https://www.tiktok.com/current';
  const listeners=[];
  const context = vm.createContext({
    globalThis:null, URL, Number, String, Object, RegExp, Promise, Math,
    location:{hostname:'www.tiktok.com',href:'https://www.tiktok.com/'},
    document:{title:'Feed',querySelectorAll(s){return s==='video'?[video]:[]}}, innerHeight:900, innerWidth:1000,
    chrome:{runtime:{onMessage:{addListener(fn){listeners.push(fn)}}}},
    ADM_SOCIAL_PLAYER_RESOLVER_V2:{exactPlayer(){ throw new Error('old visible-only matcher must not run'); },tiktokDom(){return{url:'https://www.tiktok.com/@owner/video/7676540110162136322',reason:'tiktok_scoped_anchor',scope:{nodes:[1]}}}},
    ADM_SOCIAL_HOME_FEED_V3_CORE:{rect:v=>v.getBoundingClientRect(),videoVis:()=>false,stable:async(v,fn)=>({value:await fn(),ok:true,reason:'player_stable',delta:0})},
    ADM_SOCIAL_HOME_FEED_V3_FB:{dom(){throw new Error('not facebook')},info(){return{url:null}},menu(){throw new Error('not facebook')}},
    ADM_SOCIAL_HOME_FEED_V3_TT:{menu(){throw new Error('menu not needed')},deep(){return null}},
  });
  context.globalThis=context;
  vm.runInContext(src('social-home-feed-resolution-v3.js'),context);
  const r=video.getBoundingClientRect();
  const out=await context.ADM_SOCIAL_HOME_FEED_V3.resolve({playerId:'player-9',playerBindingValidated:true,previewUrl:video.currentSrc,rect:r,duration:12,thumbnail:''},'11111111-1111-4111-8111-111111111111');
  assert.equal(out.item.url,'https://www.tiktok.com/@owner/video/7676540110162136322');
});

test('0.3.132 TikTok identity scopes ignore hidden preload but stop at visible competitor', () => {
  const body=node({tag:'BODY'}), current=node({tag:'VIDEO',r:rect(10,10,400,700)}), preload=node({tag:'VIDEO',r:rect(10,1100,400,700)});
  current.currentSrc='blob:https://www.tiktok.com/current'; preload.currentSrc='blob:https://www.tiktok.com/preload';
  const card=node({tag:'ARTICLE',r:rect(0,0,500,800),videos:[current,preload],parent:body}); current.parentElement=card; preload.parentElement=card;
  const context=vm.createContext({
    globalThis:null,URL,Set,WeakMap,WeakSet,Object,String,Number,RegExp,JSON,
    location:{hostname:'www.tiktok.com',href:'https://www.tiktok.com/'},document:{body,documentElement:node({tag:'HTML'}),querySelectorAll(s){return s==='video'?[current,preload]:[]}},
    innerHeight:900,innerWidth:1000,getComputedStyle(){return{display:'block',visibility:'visible',opacity:'1'}},Event:class{},window:{},
  });
  context.window=context; context.window.top=context; context.globalThis=context;
  vm.runInContext(src('tiktok-identity.js'),context);
  const scopes=context.ApocalipseTikTokIdentity.scopesFor(current);
  assert.ok(scopes.includes(card),'hidden preload must not stop at card');
  const competitor=node({tag:'VIDEO',r:rect(500,20,300,600)}); competitor.currentSrc='blob:https://www.tiktok.com/other'; competitor.parentElement=card; card.querySelectorAll=s=>s==='video'?[current,preload,competitor]:[];
  const scopes2=context.ApocalipseTikTokIdentity.scopesFor(current);
  assert.ok(!scopes2.includes(card),'visible competitor must remain a boundary');
});

test('0.3.132 TikTok fresh menu accepts unique Copy control and requests popup clipboard read', async () => {
  let opened=false, copied=false;
  const body=node({tag:'BODY',r:rect(0,0,1200,900)});
  const video=node({tag:'VIDEO',r:rect(250,60,500,760)}); video.currentSrc='blob:https://www.tiktok.com/current';
  const share=node({tag:'BUTTON',r:rect(790,550,48,48),attrs:{'data-e2e':'share-icon','aria-label':'Share'},click:()=>{opened=true}});
  const copy=node({tag:'BUTTON',r:rect(720,300,160,45),attrs:{'data-e2e':'share-copy'},click:()=>{copied=true}}); copy.textContent='Copy';
  const card=node({tag:'ARTICLE',r:rect(180,20,700,820),parent:body,videos:[video]}); video.parentElement=card;
  card.querySelectorAll=selector=>{
    if(selector==='video') return [video];
    if(selector.includes('button')||selector.includes('[data-e2e]')||selector.includes('[aria-label]')) return [share];
    return [];
  };
  const document={body,documentElement:node({tag:'HTML'}),querySelectorAll(selector){
    if(selector.includes('[data-e2e*="share"]')) return [share];
    if(opened && (selector.includes('[data-e2e]')||selector.includes('button')||selector.includes('[role="button"]'))) return [copy];
    return [];
  }};
  const context=vm.createContext({
    globalThis:null,URL,Set,Map,Promise,decodeURIComponent,location:{hostname:'www.tiktok.com',href:'https://www.tiktok.com/'},document,
    innerWidth:1200,innerHeight:900,getComputedStyle(){return{display:'block',visibility:'visible',opacity:'1'}},setTimeout(fn){fn();return 1},clearTimeout(){},
  }); context.globalThis=context;
  vm.runInContext(src('social-home-feed-v3-core.js'),context);
  vm.runInContext(src('social-home-feed-v3-tiktok.js'),context);
  const result=await context.ADM_SOCIAL_HOME_FEED_V3_TT.menu(video);
  assert.equal(result.clipboardRequested,true);
  assert.equal(result.reason,'tiktok_copy_link_clicked');
  assert.equal(copied,true);
});

test('0.3.134 TikTok keeps a short Share URL as a canonical-resolution candidate', async () => {
  let opened=false;
  const body=node({tag:'BODY',r:rect(0,0,1200,900)});
  const video=node({tag:'VIDEO',r:rect(250,60,500,760)}); video.currentSrc='blob:https://www.tiktok.com/current';
  const share=node({tag:'BUTTON',r:rect(790,550,48,48),attrs:{'data-e2e':'share-icon','aria-label':'Share'},click:()=>{opened=true}});
  const copy=node({tag:'BUTTON',r:rect(720,300,160,45),attrs:{'data-e2e':'share-copy','data-url':'https://vm.tiktok.com/ZSynthetic/'}}); copy.textContent='Copy link';
  const card=node({tag:'ARTICLE',r:rect(180,20,700,820),parent:body,videos:[video]}); video.parentElement=card;
  card.querySelectorAll=selector=>selector==='video'?[video]:selector.includes('button')||selector.includes('[data-e2e]')||selector.includes('[aria-label]')?[share]:[];
  const document={body,documentElement:node({tag:'HTML'}),querySelectorAll(selector){
    if(selector.includes('[data-e2e*="share"]')) return [share];
    if(opened && (selector.includes('[data-e2e]')||selector.includes('button')||selector.includes('[role="button"]'))) return [copy];
    return [];
  }};
  const context=vm.createContext({globalThis:null,URL,Set,Map,Promise,decodeURIComponent,location:{hostname:'www.tiktok.com',href:'https://www.tiktok.com/'},document,
    innerWidth:1200,innerHeight:900,getComputedStyle(){return{display:'block',visibility:'visible',opacity:'1'}},setTimeout(fn){fn();return 1},clearTimeout(){}});
  context.globalThis=context;
  vm.runInContext(src('social-home-feed-v3-core.js'),context);
  vm.runInContext(src('social-home-feed-v3-tiktok.js'),context);
  const result=await context.ADM_SOCIAL_HOME_FEED_V3_TT.menu(video);
  assert.equal(result.clipboardRequested,true);
  assert.equal(result.clipboardCandidate,'https://vm.tiktok.com/ZSynthetic/');
});

test('0.3.132 Facebook and TikTok content scripts no longer read clipboard after Copy Link', () => {
  assert.doesNotMatch(src('social-home-feed-v3-facebook.js'),/navigator\.clipboard\.readText/);
  assert.doesNotMatch(src('social-home-feed-v3-tiktok.js'),/navigator\.clipboard\.readText/);
  assert.match(src('popup-social-player-resolution.js'),/navigator\.clipboard\.readText/);
});
