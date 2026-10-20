const test=require("node:test"),assert=require("node:assert/strict"),fs=require("node:fs"),path=require("node:path");
const root=path.resolve(__dirname,"..");
test("native packaging builds the pinned Nibiru source set without the retired daemon",()=>{
 for(const name of ["release.yml","test-build.yml","validate-portable.yml"]){
  const text=fs.readFileSync(path.join(root,".github/workflows",name),"utf8");
  assert.match(text,/build_nibiru_engines\.py/);
  assert.match(text,/dist\/nibiru-tools/);
  assert.doesNotMatch(text,/FerroDownload|aria2c|aria2-backend/);
 }
});
