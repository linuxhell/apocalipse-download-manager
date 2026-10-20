(() => {
  "use strict";
  const $ = id => document.getElementById(id);
  const invoke = window.__TAURI__?.core?.invoke;
  const dialog = $("nibiru-media-dialog"), result = $("nibiru-media-result");
  const words = {
    en: ["Nibiru media", "Process supported audio, images and video containers. FLAC preserves audio samples; QOA uses lossy PCM16 compression. Experimental QOIR losslessly encodes RGB/RGBA8 PPM/PAM and decodes to PAM. Choose a new output path; existing files are preserved.", "Operation", "Input file", "New output file", "Start frame", "Frames", "Capabilities", "Run", "Working…"],
    "pt-BR": ["Mídia Nibiru", "Processe áudio, imagens e contêineres de vídeo suportados. FLAC preserva as amostras; QOA usa compressão PCM16 com perdas. O QOIR experimental codifica PPM/PAM RGB/RGBA8 sem perdas e decodifica para PAM. Escolha uma nova saída; arquivos existentes são preservados.", "Operação", "Arquivo de entrada", "Novo arquivo de saída", "Frame inicial", "Frames", "Capacidades", "Executar", "Processando…"],
    "zh-CN": ["Nibiru 媒体", "处理受支持的音频、图像和视频容器。FLAC 无损保留采样；QOA 使用有损 PCM16 压缩。实验性 QOIR 可无损编码 RGB/RGBA8 PPM/PAM 图像，并解码为 PAM。请选择新输出路径；现有文件不会被替换。", "操作", "输入文件", "新输出文件", "起始帧", "帧数", "功能", "执行", "处理中…"]
  };
  const colorWords = {
    en: ["Input color interpretation", "File declaration (BT.709 when absent)", "BT.709", "sRGB"],
    "pt-BR": ["Interpretação de cor da entrada", "Declaração do arquivo (BT.709 se ausente)", "BT.709", "sRGB"],
    "zh-CN": ["输入颜色解释", "文件声明（未声明时为 BT.709）", "BT.709", "sRGB"]
  };
  const levelLabels = {
    en: {"wav-gain":"Gain (dB)", "wav-normalize":"Target peak (dBFS)"},
    "pt-BR": {"wav-gain":"Ganho (dB)", "wav-normalize":"Pico alvo (dBFS)"},
    "zh-CN": {"wav-gain":"增益 (dB)", "wav-normalize":"目标峰值 (dBFS)"}
  };
  const operationNames = {
    en: {inspect:"Inspect", "mp4-faststart":"MP4 faststart", "flac-encode":"WAV → FLAC (lossless)", "flac-decode":"FLAC → WAV", "qoa-encode":"WAV → QOA (lossy)", "qoa-decode":"QOA → WAV", "qoir-encode":"PPM/PAM → QOIR (experimental)", "qoir-decode":"QOIR → PAM", "wav-gain":"WAV gain", "wav-normalize":"WAV peak normalization", "wav-cut":"WAV cut", "y4m-cut":"Y4M cut"},
    "pt-BR": {inspect:"Inspecionar", "mp4-faststart":"MP4 faststart", "flac-encode":"WAV → FLAC (sem perdas)", "flac-decode":"FLAC → WAV", "qoa-encode":"WAV → QOA (com perdas)", "qoa-decode":"QOA → WAV", "qoir-encode":"PPM/PAM → QOIR (experimental)", "qoir-decode":"QOIR → PAM", "wav-gain":"Ganho WAV", "wav-normalize":"Normalizar pico WAV", "wav-cut":"Cortar WAV", "y4m-cut":"Cortar Y4M"},
    "zh-CN": {inspect:"检查", "mp4-faststart":"MP4 快速起播", "flac-encode":"WAV → FLAC（无损）", "flac-decode":"FLAC → WAV", "qoa-encode":"WAV → QOA（有损）", "qoa-decode":"QOA → WAV", "qoir-encode":"PPM/PAM → QOIR（实验性）", "qoir-decode":"QOIR → PAM", "wav-gain":"WAV 增益", "wav-normalize":"WAV 峰值归一化", "wav-cut":"剪切 WAV", "y4m-cut":"剪切 Y4M"}
  };
  function language() {
    let locale = document.documentElement.lang;
    try { locale = localStorage.getItem("apocalipse.language") || locale; } catch {}
    return Object.hasOwn(words, locale) ? locale : "en";
  }
  function labels() {
    return words[language()];
  }
  function translate() {
    const values=labels();
    ["nibiru-media-open", "nibiru-media-hint", "nibiru-operation-label", "nibiru-input-label", "nibiru-output-label", "nibiru-start-label", "nibiru-frames-label", "nibiru-capabilities", "nibiru-media-run"].forEach((id,i) => $(id).textContent=values[i]);
    const names = operationNames[language()];
    for (const option of $("nibiru-operation").options) option.textContent = names[option.value] || option.value;
    ["nibiru-input-color-label", "nibiru-input-color-auto", "nibiru-input-color-bt709", "nibiru-input-color-srgb"].forEach((id,i) => $(id).textContent=colorWords[language()][i]);
    updateFields();
  }
  function updateFields() {
    const operation = $("nibiru-operation").value;
    const cut = operation === "wav-cut" || operation === "y4m-cut";
    const gain = operation === "wav-gain" || operation === "wav-normalize";
    $("nibiru-start-label").textContent = levelLabels[language()][operation] || labels()[5];
    ["nibiru-start-label", "nibiru-start"].forEach(id => $(id).hidden = !(cut || gain));
    ["nibiru-frames-label", "nibiru-frames"].forEach(id => $(id).hidden = !cut);
    ["nibiru-output-label", "nibiru-output"].forEach(id => $(id).hidden = operation === "inspect");
    ["nibiru-input-color-label", "nibiru-input-color"].forEach(id => $(id).hidden = operation !== "qoir-encode");
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
  $("nibiru-operation").onchange=updateFields;
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
    if(operation==="qoir-encode"){
      const color=$("nibiru-input-color").value;
      if(!["", "bt709", "srgb"].includes(color)){result.textContent="invalid_media_color";return;}
      if(color)request.input_color=color;
    }
    return run("process_nibiru_media",{request});
  };
  window.addEventListener("apocalipse-language-changed", translate);
  translate();
})();
