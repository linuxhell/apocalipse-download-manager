const {test}=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const vm=require('node:vm');
const source=fs.readFileSync(require('node:path').join(__dirname,'../apps/desktop/ui/app.js'),'utf8');
const helper=source.slice(source.indexOf('function reportUiError('),source.indexOf('const recordStructuredUi ='));
function harness(bridge) {
 const logs=[],c=vm.createContext({freshUiTrace:()=> 'test-trace',console:{error:(...args)=>logs.push(args),warn:(...args)=>logs.push(args)},window:{__TAURI__:{core:{invoke:bridge}}}});
 vm.runInContext(helper,c);return {report:c.reportUiError,logs};
}
test('caught UI errors go to both diagnostic sinks with the same trace',async()=>{
 const calls=[],h=harness(async(command,args)=>{calls.push({command,args});});
 await h.report('main','save_settings',new Error('write failed'));
 assert.equal(h.logs.length,1);assert.equal(calls.length,2);
 assert.equal(calls[0].command,'record_diagnostics_ui');
 assert.equal(calls[0].args.detail.traceId,'test-trace');
 assert.equal(calls[0].args.detail.operation,'save_settings');
 assert.equal(calls[0].args.detail.level,'ERROR');
 assert.equal(calls[1].command,'record_ui_diagnostic');
 assert.match(calls[1].args.detail,/trace=test-trace/);
});
test('failed diagnostic bridges never recurse or reject the error handler',async()=>{
 for(const bridge of [()=>{throw new Error('sync failure')},async()=>{throw new Error('async failure')},undefined]){
  let calls=0;const h=harness(bridge? (...args)=>{calls++;return bridge(...args)}:undefined);
  await assert.doesNotReject(h.report('main','refresh',new Error('original')));
  assert.equal(calls,bridge?2:0);assert.equal(h.logs.length,1);
 }
});
test('warning severity is retained and caught errors no longer stay console-only',async()=>{
 const calls=[],h=harness(async(command,args)=>calls.push({command,args}));
 await h.report('main','thumbnail',new Error('unavailable'),'WARN');
 assert.equal(calls[0].args.detail.level,'WARN');assert.equal(calls[1].args.level,'WARN');
 assert.doesNotMatch(source,/\.catch\(console\.error\)|console\.(error|warn)\(error\)/);
});
