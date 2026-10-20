(() => {
  "use strict";
  const $ = id => document.getElementById(id);
  const invoke = window.__TAURI__?.core?.invoke;
  const dialog = $("nibiru-media-dialog"), result = $("nibiru-media-result");
  const words = {
    en: ["Nibiru media", "Process supported WAV, Y4M and MP4 files. Choose a new output path; existing files are preserved.", "Operation", "Input file", "New output file", "Start frame / gain (dB)", "Frames", "Capabilities", "Run", "Working…"],
    "pt-BR": ["Mídia Nibiru", "Processe os formatos WAV, Y4M e MP4 suportados. Escolha um novo caminho de saída; arquivos existentes são preservados.", "Operação", "Arquivo de entrada", "Novo arquivo de saída", "Frame inicial / ganho (dB)", "Frames", "Capacidades", "Executar", "Processando…"],
    "zh-CN": ["Nibiru 媒体", "处理受支持的 WAV、Y4M 和 MP4 文件。请选择新的输出路径；现有文件不会被替换。", "操作", "输入文件", "新输出文件", "起始帧 / 增益 (dB)", "帧数", "功能", "执行", "处理中…"]
  };
  function labels() {
    let locale = document.documentElement.lang;
    try { locale = localStorage.getItem("apocalipse.language") || locale; } catch {}
    return words[locale] || words.en;
  }
  function translate() {
    const values=labels();
    ["nibiru-media-open", "nibiru-media-hint", "nibiru-operation-label", "nibiru-input-label", "nibiru-output-label", "nibiru-start-label", "nibiru-frames-label", "nibiru-capabilities", "nibiru-media-run"].forEach((id,i) => $(id).textContent=values[i]);
  }
  async function run(command,args) {
    const buttons=[...dialog.querySelectorAll("button")];
    buttons.forEach(b=>b.disabled=true);result.textContent=labels()[9];
    try {
      if (!invoke) throw new Error("desktop_required");
      result.textContent=JSON.stringify(await invoke(command,args),null,2);
    } catch(error) { result.textContent=String(error); }
    finally { buttons.forEach(b=>b.disabled=false); }
  }
  $("nibiru-media-open").onclick=()=>{translate();dialog.showModal();};
  $("nibiru-media-close").onclick=()=>dialog.close();
  $("nibiru-input-browse").onclick=async()=>{
    try { const path=await invoke("pick_executable",{initialPath:$("nibiru-input").value});if(path)$("nibiru-input").value=path; }
    catch(error){result.textContent=String(error);}
  };
  $("nibiru-capabilities").onclick=()=>run("get_nibiru_capabilities",{});
  $("nibiru-media-run").onclick=()=>{
    const operation=$("nibiru-operation").value,input=$("nibiru-input").value,output=$("nibiru-output").value;
    const value=Number($("nibiru-start").value),frames=Number($("nibiru-frames").value);
    const request={operation,input};
    if(operation!=="inspect") request.output=output;
    if(operation.endsWith("-cut")){
      if(!Number.isSafeInteger(value)||value<0||!Number.isSafeInteger(frames)||frames<1){result.textContent="invalid_media_range";return;}
      request.start=value;request.frames=frames;
    }
    if(operation==="wav-gain")request.db=value;
    if(operation==="wav-normalize")request.peak_db=value;
    return run("process_nibiru_media",{request});
  };
  translate();
})();
