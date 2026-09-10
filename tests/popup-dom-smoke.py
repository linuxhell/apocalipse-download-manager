"""Browser DOM regression tests. Chrome transport is mocked; no real websites or credentials."""
import argparse
import json
import pathlib
import re
import threading
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from functools import partial
from playwright.sync_api import sync_playwright

ROOT = pathlib.Path(__file__).resolve().parents[1]
MOCK = r"""
window.testCalls = []; window.testScan = []; window.testNetwork = [];
window.hangTypes = new Set();
const storage = {language:'pt_BR',pairingToken:''};
const changed=[];
const resolve=(cb,v)=>{if(cb)queueMicrotask(()=>cb(v));return Promise.resolve(v)};
window.chrome={
 runtime:{getManifest:()=>({version:'0.3.103'}),getURL:p=>'https://fixture.invalid/'+p,
 sendMessage(m,cb){testCalls.push(m);if(hangTypes.has(m.type))return new Promise(()=>{});
 let v=m.type==='APOCALIPSE_RECENT_TAB_MEDIA'?{media:testNetwork}:m.type.startsWith('ADM_DIAG')?{active:false,ok:true}:m.type==='APOCALIPSE_WORKER_PING'?{ok:true,version:'0.3.103'}:{ok:true,target:'desktop'};
 return resolve(cb,v);},onMessage:{addListener(){}}},
 storage:{local:{get(v,cb){return resolve(cb,{...v,...storage})},set(v){Object.assign(storage,v);return Promise.resolve()}},onChanged:{addListener(fn){changed.push(fn)}}},
 tabs:{query(q,cb){return resolve(cb,[{id:7,url:'https://www.tiktok.com/'}])},sendMessage(id,m,o,cb){return resolve(cb,{media:testScan})}},
 scripting:{executeScript(){return Promise.resolve([{frameId:0,result:{media:testScan}}])}}
};
"""
def run(executable=None, memory_dom=False):
    with sync_playwright() as p:
        b=p.chromium.launch(headless=True, **({'executable_path':executable} if executable else {'channel':'chromium'}))
        page=b.new_page(viewport={'width':640,'height':600}); errors=[]
        page.on('pageerror',lambda e: errors.append(str(e)))
        server=None
        init=MOCK+"\nwindow.testNetwork=[{url:'https://v16.tiktokcdn.com/clip.mp4',contentType:'video/mp4',capturedAt:123,frameId:0}];"
        if memory_dom:
            # For restricted local browsers only: transport AND secure UUID API
            # are mocked. CI uses a real localhost secure context instead.
            html=(ROOT/'browser-extension/popup.html').read_text()
            scripts=re.findall(r'<script src="([^"]+)"></script>',html)
            html=re.sub(r'<script.*?</script>','',html,flags=re.S)
            html=html.replace('<link rel="stylesheet" href="popup.css">','<style>'+(ROOT/'browser-extension/popup.css').read_text()+'</style>')
            page.set_content(html)
            page.add_script_tag(content=init+"\nlet uid=0;crypto.randomUUID=()=> '10000000-1000-4000-8000-'+String(++uid).padStart(12,'0');")
            for name in scripts: page.add_script_tag(content=(ROOT/'browser-extension'/name).read_text())
        else:
            server=ThreadingHTTPServer(('127.0.0.1',0),partial(SimpleHTTPRequestHandler,directory=str(ROOT)))
            threading.Thread(target=server.serve_forever,daemon=True).start()
            page.add_init_script(init)
            page.goto(f'http://127.0.0.1:{server.server_port}/browser-extension/popup.html')
        page.wait_for_selector('.download-item');page.wait_for_timeout(100)
        assert page.locator('.download-item').inner_text()=='Analisar m\u00eddia'
        page.locator('#select-all').check()
        assert page.locator('.media-select').is_checked()
        page.locator('#download-selected').click()
        assert page.locator('#popup-notice').is_visible()
        assert not page.evaluate("testCalls.some(x=>x.type==='APOCALIPSE_DOWNLOAD_BATCH')")
        page.locator('nav [data-kind="audio"]').click();page.locator('nav [data-kind="video"]').click()
        page.select_option('#language','en');page.select_option('#language','pt_BR')
        page.evaluate("testScan=[{url:'https://www.tiktok.com/@fixture/video/111',kind:'video',pageExtractor:true,title:'Fixture'}, {url:'https://image.example/%E0%A4%A',kind:'image'}]")
        page.locator('#refresh-media').click()
        page.wait_for_function("document.querySelector('.download-item')?.textContent==='Download'")
        page.locator('.download-item').click();page.locator('.external-preview').click()
        assert page.evaluate("testCalls.filter(x=>x.type==='APOCALIPSE_DOWNLOAD').at(-1).item.url")=='https://www.tiktok.com/@fixture/video/111'
        assert page.evaluate("testCalls.filter(x=>x.type==='APOCALIPSE_PREVIEW_MEDIA').at(-1).url")=='https://www.tiktok.com/@fixture/video/111'
        page.locator('nav [data-kind="image"]').click()
        page.wait_for_selector('.media-select');page.locator('#select-all').check()
        assert page.locator('.media-select').is_checked()
        for width in [640,420]:
            page.set_viewport_size({'width':width,'height':600})
            page.locator('.download-item').scroll_into_view_if_needed()
            r=page.locator('.download-item').bounding_box()
            assert r['x']>=0 and r['x']+r['width']<=width, r
            assert page.evaluate("document.querySelector('main').scrollWidth<=document.querySelector('main').clientWidth")
        page.set_viewport_size({'width':640,'height':600})
        page.screenshot(path='/mnt/data/adm-runtime-tests/popup-fixed.png') if pathlib.Path('/mnt/data/adm-runtime-tests').exists() else None
        page.locator('#copy-extension-report').click()
        page.wait_for_function("!document.querySelector('#copy-extension-report').disabled")
        assert not errors, errors
        print(json.dumps({'ok':True,'checks':['ambiguous items selectable without unsafe handoff','bulk feedback','navigation and language','download and preview same identity','invalid URL escape handled','buttons fit 640 and 420 px','recovery report control'],'pageErrors':errors}))
        b.close()
        if server: server.shutdown()
if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--browser-executable');parser.add_argument('--memory-dom',action='store_true');a=parser.parse_args();run(a.browser_executable,a.memory_dom)
