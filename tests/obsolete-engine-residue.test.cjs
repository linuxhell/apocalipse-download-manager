const test=require("node:test"),assert=require("node:assert/strict"),fs=require("node:fs"),path=require("node:path");const ROOT=path.resolve(__dirname,".."),FORBIDDEN=["gospeed","xtreme-next","xtremenext","aninsomniacy","libtorrent","pause-metadata","followedby","metadataprobe","preview_magnet_metadata","xtreme-metadata-inspection","fileselectionstate","bt-first-last-piece-first","stream-max-connections","rqbit","surgedm","transmission-daemon","transmissiontorrent","surgehttp"],EXT=new Set([".rs",".js",".cjs",".json",".html",".css",".yml",".yaml",".toml",".ps1",".sh",".plist"]),DIRS=[".github","apps","browser-extension","crates"];function walk(t,r=[]){if(!fs.existsSync(t))return r;const s=fs.statSync(t);if(s.isDirectory()){for(const n of fs.readdirSync(t)){if([".git","target","node_modules","dist"].includes(n))continue;walk(path.join(t,n),r)}}else r.push(t);return r}test("obsolete external engines and retired implementations are absent from product sources",()=>{const residue=[];for(const f of DIRS.flatMap(d=>walk(path.join(ROOT,d)))){const rel=path.relative(ROOT,f).replaceAll("\\","/");if(rel==="tests/obsolete-engine-residue.test.cjs"||!EXT.has(path.extname(f).toLowerCase()))continue;const c=fs.readFileSync(f,"utf8").toLowerCase();for(const t of FORBIDDEN)if(c.includes(t))residue.push(t+":"+rel)}assert.deepEqual(residue,[])});

// Native ED2K/Kad is explicitly requested again; it must use Nibiru rather than
// reintroducing an external aMule daemon or an old engine wrapper.
test("ED2K window uses native Nibiru commands",()=>{
 const integration=fs.readFileSync(path.join(ROOT,"apps/desktop/src-tauri/src/ed2k_ui.rs"),"utf8");
 assert.match(integration,/nibiru_xtreme::ed2k/);
 assert.doesNotMatch(integration,/Command::new|std::process|tokio::process/);
});
