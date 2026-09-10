"""Real Chromium extension APIs, synthetic TikTok page and local ADM stub.
Compare the unmodified 0.3.102 baseline, a filename-only control, and candidate.
No real TikTok requests, user credentials or playable media are used.
"""
import argparse
import json
import pathlib
import shutil
import tempfile
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from playwright.sync_api import sync_playwright

ROOT = pathlib.Path(__file__).resolve().parents[1]
REQUESTS = []
TOKEN = 'synthetic-local-test-token'

class Bridge(BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def reply(self, value):
        data = json.dumps(value).encode()
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', self.headers.get('Origin', '*'))
        self.send_header('Access-Control-Allow-Headers', 'Authorization,Content-Type')
        self.send_header('Access-Control-Allow-Methods', 'GET,POST,OPTIONS')
        self.send_header('Content-Length', str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_OPTIONS(self):
        self.reply({})

    def do_GET(self):
        self.reply({'ok': True, 'active': False})

    def do_POST(self):
        payload = json.loads(self.rfile.read(int(self.headers.get('Content-Length', '0'))) or b'{}')
        REQUESTS.append({'path': self.path, 'body': payload})
        self.reply({'ok': True, 'taskId': 'synthetic-task', 'preparing': self.path.endswith('preview-media')})

HTML = '''<!doctype html><meta charset="utf-8"><title>Fixture</title>
<style>body{margin:0}article{margin:20px;width:420px;height:340px}video{display:block;width:400px;height:280px;background:#222}</style>
<article id="current"><video id="v"></video><a id="identity" href="https://www.tiktok.com/@fixture/video/111">Own permalink</a></article>
<aside><a id="decoy" href="https://www.tiktok.com/@unrelated/video/999">Unrelated link</a></aside>
<script>
window.fsChanges=0;document.addEventListener('fullscreenchange',()=>fsChanges++);
window.swap=(id)=>{const a=document.querySelector('#identity');if(a)a.href='https://www.tiktok.com/@fixture/video/'+id;
 document.querySelector('video').src=URL.createObjectURL(new Blob(['invalid synthetic media'],{type:'video/mp4'}));};swap('111');
</script>'''

STATE = """({id,world})=>chrome.scripting.executeScript({target:{tabId:id},world,func:()=>{
 const v=document.querySelector('video'),b=document.querySelector('.apocalipse-media-download');
 return {identity:!!globalThis.ApocalipseTikTokIdentity,handler:!!globalThis.ADM_TIKTOK_DOWNLOAD,
   buttonTitle:b?.title,resolved:v?globalThis.ApocalipseTikTokIdentity?.resolve(v):null,
   bound:!!globalThis.ApocalipseTikTokIdentity?.videoFor(b),ready:document.readyState};
}})"""

def until(predicate, page, seconds=8):
    end = time.monotonic() + seconds
    while time.monotonic() < end:
        if predicate():
            return
        page.wait_for_timeout(100)
    raise AssertionError('Expected synthetic handoff was not observed')


def exercise(p, extension, mode):
    REQUESTS.clear()
    console = []
    fixed = mode == 'candidate'
    with tempfile.TemporaryDirectory() as profile:
        c = p.chromium.launch_persistent_context(
            profile, channel='chromium', headless=True,
            args=[f'--disable-extensions-except={extension}', f'--load-extension={extension}'],
            viewport={'width': 1000, 'height': 700})
        c.on('console', lambda m: console.append({'type': m.type, 'text': m.text[:1000]}))
        try:
            c.route('https://www.tiktok.com/**', lambda r: r.fulfill(status=200, content_type='text/html', body=HTML))
            c.route('https://v16.tiktokcdn.com/**', lambda r: r.fulfill(
                status=200, headers={'content-type': 'video/mp4', 'Access-Control-Allow-Origin': '*'}, body=b'synthetic-response'))
            worker = c.service_workers[0] if c.service_workers else c.wait_for_event('serviceworker')
            worker.evaluate('token=>chrome.storage.local.set({pairingToken:token,language:"pt_BR"})', TOKEN)
            health = worker.evaluate("async()=>{try{return await bridgeRequest('/v1/health')}catch(e){return {error:String(e)}}}")
            assert health.get('ok'), health
            site = c.new_page()
            site.on('pageerror', lambda e: console.append({'pageerror': str(e)}))
            site.goto('https://www.tiktok.com/')
            site.bring_to_front()
            site.wait_for_selector('.apocalipse-media-download:not(.apocalipse-media-record)')
            tabid = worker.evaluate("async()=> (await chrome.tabs.query({url:'https://www.tiktok.com/*'}))[0].id")
            worlds = {world: worker.evaluate(STATE, {'id': tabid, 'world': world})[0]['result']
                      for world in ['MAIN', 'ISOLATED']}
            print(json.dumps({'mode': mode, 'bridgeHealth': health, 'worlds': worlds}), flush=True)
            assert worlds['MAIN']['identity'], 'Page-world resolver must exist'

            def downloads():
                return [r['body'] for r in REQUESTS if r['path'] == '/v1/download']

            def open_popup():
                site.bring_to_front()
                with c.expect_page() as pending:
                    worker.evaluate('url=>chrome.tabs.create({url,active:false})', worker.url.rsplit('/', 1)[0] + '/popup.html')
                page = pending.value
                page.wait_for_load_state('domcontentloaded')
                return page

            button = site.locator('.apocalipse-media-download:not(.apocalipse-media-record)')
            if mode == 'baseline':
                assert not worlds['ISOLATED']['identity'], 'Exact baseline must reproduce the missing isolated resolver'
                button.click()
                site.wait_for_timeout(1200)
                assert not downloads(), 'Baseline unexpectedly handed off a canonical reel'
                assert site.evaluate('fsChanges') > 0, 'Expected original fullscreen flash'
                print(json.dumps({'mode': mode, 'missingIsolatedResolver': True,
                                  'fullscreenChanges': site.evaluate('fsChanges'), 'handoffs': 0}), flush=True)
                return

            assert worlds['ISOLATED']['identity'], 'Separate resource must load in isolated world'
            assert worlds['ISOLATED']['resolved'] == 'https://www.tiktok.com/@fixture/video/111'
            if fixed:
                popup = open_popup()
                popup.wait_for_selector('.download-item')
                assert popup.locator('.media-select').first.is_enabled()
                assert popup.locator('.external-preview').first.is_enabled()
                print(json.dumps({'mode': mode, 'popupVideoBeforeOverlayClick': True}), flush=True)
                popup.close()
            button.click()
            until(lambda: len(downloads()) >= 1, site)
            assert downloads()[-1]['url'] == 'https://www.tiktok.com/@fixture/video/111'
            site.evaluate("swap('222')")
            site.wait_for_timeout(350)
            button.click()
            until(lambda: len(downloads()) >= 2, site)
            assert downloads()[-1]['url'] == 'https://www.tiktok.com/@fixture/video/222'
            assert site.evaluate('fsChanges') == 0
            if mode == 'filename-only-control':
                print(json.dumps({'mode': mode, 'sequentialReelHandoffs': ['111', '222'],
                                  'fullscreenChanges': 0, 'onlyChange': 'MAIN resolver resource path'}), flush=True)
                return

            # Exact-element callback survives a lost delegated WeakMap lookup.
            worker.evaluate("id=>chrome.scripting.executeScript({target:{tabId:id},func:()=>{globalThis.ApocalipseTikTokIdentity.videoFor=()=>null}})", tabid)
            site.evaluate("document.querySelector('#identity').remove();document.querySelector('#decoy').remove();swap('333')")
            site.wait_for_timeout(350)
            before = len(downloads())
            button.click()
            site.wait_for_timeout(900)
            assert len(downloads()) == before, 'Unidentified blob must not become an arbitrary download'
            assert site.evaluate('fsChanges') == 0
            site.evaluate("fetch('https://v16.tiktokcdn.com/track.mp4').then(r=>r.arrayBuffer())")
            site.wait_for_timeout(400)
            captured = worker.evaluate("id=>new Promise(resolve=>{const found=recentMediaResponses.filter(x=>x.tabId===id);resolve(found.length)})", tabid)
            print(json.dumps({'mode': mode, 'realWebRequestCaptured': captured}), flush=True)
            assert captured > 0, 'Synthetic network fixture must reach real webRequest'
            for page in list(c.pages):
                if page.url.startswith('chrome-extension:'):
                    page.close()
            popup = open_popup()
            errors = []
            popup.on('pageerror', lambda e: errors.append(str(e)))
            popup.wait_for_selector('.download-item', timeout=12000)
            checkbox = popup.locator('.media-select').first
            assert checkbox.is_enabled()
            popup.locator('#select-all').check()
            assert checkbox.is_checked()
            popup.locator('#download-selected').click()
            popup.wait_for_timeout(100)
            assert len(downloads()) == before, 'Unverified batch must retain its safeguards'
            assert popup.locator('#popup-notice').is_visible()
            popup.locator('nav [data-kind="audio"]').click()
            popup.locator('nav [data-kind="video"]').click()
            popup.select_option('#language', 'en')
            popup.select_option('#language', 'pt_BR')
            worker.evaluate("()=>chrome.runtime.onMessage.addListener(m=>m.type==='TEST_SILENT_REPLY'?true:undefined)")
            code = popup.evaluate("async()=>{try{await ADM_POPUP.runtime({type:'TEST_SILENT_REPLY'},40);return 'unexpected'}catch(e){return e.code}}")
            assert code == 'response_timeout', code
            assert not errors, errors
            print(json.dumps({'mode': mode, 'fullscreenChanges': site.evaluate('fsChanges'),
                              'sequentialReelHandoffs': ['111', '222'], 'unidentifiedBlobNotSent': True,
                              'checkboxUsable': True, 'batchGuardPreserved': True,
                              'silentWorkerTimeout': code, 'pageErrors': errors}), flush=True)
        except Exception:
            print(json.dumps({'mode': mode, 'console': console, 'bridgeRequests': REQUESTS}), flush=True)
            try:
                print(json.dumps({'mode': mode, 'isolated': worker.evaluate(STATE, {'id': tabid, 'world': 'ISOLATED'})}), flush=True)
            except Exception as error:
                print('State capture failed', str(error), flush=True)
            raise
        finally:
            c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--baseline')
    args = parser.parse_args()
    server = ThreadingHTTPServer(('127.0.0.1', 17654), Bridge)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    try:
        with sync_playwright() as p, tempfile.TemporaryDirectory() as control:
            runs = []
            if args.baseline:
                baseline = pathlib.Path(args.baseline).resolve()
                runs.append((str(baseline), 'baseline'))
                shutil.copytree(baseline, control, dirs_exist_ok=True)
                control = pathlib.Path(control)
                shutil.copyfile(control / 'tiktok-identity.js', control / 'tiktok-identity-main.js')
                manifest = json.loads((control / 'manifest.json').read_text())
                for entry in manifest['content_scripts']:
                    if entry.get('world') == 'MAIN':
                        entry['js'] = ['tiktok-identity-main.js' if x == 'tiktok-identity.js' else x for x in entry['js']]
                (control / 'manifest.json').write_text(json.dumps(manifest))
                runs.append((str(control), 'filename-only-control'))
            runs.append((str(ROOT / 'browser-extension'), 'candidate'))
            failures = []
            for extension, mode in runs:
                try:
                    exercise(p, extension, mode)
                except Exception as error:
                    failures.append({'mode': mode, 'error': str(error)})
            assert not failures, failures
    finally:
        server.shutdown()
