/* Local-file converter: all DOM text is translated or assigned through textContent. */
window.AdmConversionCatalogs = {
  en: {
    convertFile: "Convert file", conversionChoose: "Choose file…", conversionSource: "Source file", conversionAnalyze: "Analyze file", conversionAnalyzing: "Analyzing streams and checking for black-only video…", conversionOriginal: "The original file is kept. A separate file is created; existing files are never replaced.", conversionImage: "Image", conversionAudio: "Audio", conversionVideo: "Video", conversionTrack: "Audio track", conversionTracks: "Detected tracks", conversionCover: "Cover art", conversionBlack: "All decoded video frames are black. Only audio formats are offered.", conversionFormat: "Convert to", conversionStart: "Convert", conversionStop: "Cancel operation", conversionStopping: "Cancelling…", conversionWorking: "Converting…", conversionDone: "Saved: ", conversionCancelled: "Operation cancelled. The original file was kept.", conversionLossless: "Lossless encoding; it cannot restore quality already lost in the source. Output may be much larger.", conversionHigh: "High quality; re-encoding may lose a small amount of detail.", conversionFrame: "Still image conversion. PNG and TIFF support lossless conversion; lossless WebP supports up to 8 bits per channel. JPEG is lossy; BMP can remove transparency.", conversionNeedTools: "Install FFmpeg (including FFprobe) in Settings → Tools, or select their installed location.", conversionError: "Conversion failed: ", conversionUnsupported: "This file has no supported image, audio or video streams.", conversionChanged: "The source changed. Analyze it again before converting.", conversionNoEncoders: "This FFmpeg build has no compatible output encoders.", conversionPickFirst: "Choose a file and analyze it first.", conversionLosslessMkv: "MKV — lossless video", conversionAnimated: "Animated images are handled as video to preserve their frames."
  },
  "pt-BR": {
    convertFile: "Converter arquivo", conversionChoose: "Escolher arquivo…", conversionSource: "Arquivo original", conversionAnalyze: "Analisar arquivo", conversionAnalyzing: "Analisando trilhas e verificando se o vídeo tem apenas tela preta…", conversionOriginal: "O original é mantido. Um arquivo separado é criado; arquivos existentes nunca são substituídos.", conversionImage: "Imagem", conversionAudio: "Áudio", conversionVideo: "Vídeo", conversionTrack: "Trilha de áudio", conversionTracks: "Trilhas identificadas", conversionCover: "Capa", conversionBlack: "Todos os quadros de vídeo decodificados são pretos. Apenas formatos de áudio são oferecidos.", conversionFormat: "Converter para", conversionStart: "Converter", conversionStop: "Cancelar operação", conversionStopping: "Cancelando…", conversionWorking: "Convertendo…", conversionDone: "Salvo: ", conversionCancelled: "Operação cancelada. O arquivo original foi mantido.", conversionLossless: "Codificação sem perdas; não recupera qualidade já perdida no original. O arquivo pode ficar muito maior.", conversionHigh: "Alta qualidade; a recodificação pode perder um pouco de detalhe.", conversionFrame: "Conversão de imagem estática. PNG e TIFF permitem conversão sem perdas; WebP sem perdas suporta até 8 bits por canal. JPEG tem perdas; BMP pode remover transparência.", conversionNeedTools: "Instale FFmpeg (incluindo FFprobe) em Configurações → Ferramentas ou selecione onde estão instalados.", conversionError: "Falha na conversão: ", conversionUnsupported: "Este arquivo não contém trilhas de imagem, áudio ou vídeo compatíveis.", conversionChanged: "O original foi alterado. Analise novamente antes de converter.", conversionNoEncoders: "Esta versão do FFmpeg não possui codificadores de saída compatíveis.", conversionPickFirst: "Escolha um arquivo e analise primeiro.", conversionLosslessMkv: "MKV — vídeo sem perdas", conversionAnimated: "Imagens animadas são tratadas como vídeo para preservar seus quadros."
  },
  "zh-CN": {
    convertFile: "转换文件", conversionChoose: "选择文件…", conversionSource: "源文件", conversionAnalyze: "分析文件", conversionAnalyzing: "正在分析轨道并检查视频是否全为黑屏…", conversionOriginal: "保留原文件，另存为新文件，不会覆盖已有文件。", conversionImage: "图像", conversionAudio: "音频", conversionVideo: "视频", conversionTrack: "音轨", conversionTracks: "检测到的轨道", conversionCover: "封面", conversionBlack: "所有解码的视频帧均为黑屏，仅提供音频格式。", conversionFormat: "转换为", conversionStart: "转换", conversionStop: "取消操作", conversionStopping: "正在取消…", conversionWorking: "正在转换…", conversionDone: "已保存：", conversionCancelled: "操作已取消，原文件已保留。", conversionLossless: "无损编码无法恢复源文件已损失的质量，输出文件可能大得多。", conversionHigh: "高质量；重新编码可能损失少量细节。", conversionFrame: "静态图像转换。PNG 和 TIFF 支持无损转换；无损 WebP 支持每通道最多 8 位。JPEG 为有损格式，BMP 可能丢失透明度。", conversionNeedTools: "请在设置 → 工具中安装 FFmpeg（含 FFprobe），或选择安装位置。", conversionError: "转换失败：", conversionUnsupported: "此文件没有支持的图像、音频或视频轨道。", conversionChanged: "源文件已更改，请重新分析后再转换。", conversionNoEncoders: "此 FFmpeg 版本没有兼容的输出编码器。", conversionPickFirst: "请先选择并分析文件。", conversionLosslessMkv: "MKV — 无损视频", conversionAnimated: "动态图像按视频处理，以保留全部帧。"
  }
};
window.createAdmFileConverter = ({ invoke, t, resize, report }) => {
  const $ = id => document.getElementById(id);
  const dialog = $("conversion-dialog");
  let id = null, info = null, busy = false, stopped = false, output = null;
  const audioFormats = new Set(["mp3", "m4a", "opus", "flac", "wav", "ogg"]);
  const status = text => { $("conversion-status").textContent = text; };
  function lock(value) {
    busy = value;
    for (const name of ["conversion-pick", "conversion-analyze", "conversion-source", "conversion-directory", "conversion-directory-pick", "conversion-format", "conversion-track"]) $(name).disabled = value;
    $("conversion-start").disabled = value || !info;
    $("conversion-stop").hidden = !value;
    $("conversion-close").disabled = value;
  }
  function updateFormat() {
    const format = $("conversion-format").value;
    $("conversion-track-row").hidden = !audioFormats.has(format);
    $("conversion-quality").textContent = t(info?.kind === "image" ? "conversionFrame" : ["mkv-lossless", "flac", "wav"].includes(format) ? "conversionLossless" : "conversionHigh");
  }
  function errorText(error) {
    const raw = String(error);
    if (raw.includes("conversion_cancelled")) return t("conversionCancelled");
    if (raw.includes("conversion_tool_unavailable")) return t("conversionNeedTools");
    if (raw.includes("conversion_unsupported")) return t("conversionUnsupported");
    if (raw.includes("conversion_source_changed")) return t("conversionChanged");
    if (raw.includes("conversion_no_encoders")) return t("conversionNoEncoders");
    return t("conversionError") + raw;
  }
  async function analyze() {
    if (busy) return;
    const source = $("conversion-source").value.trim();
    info = null; output = null;
    $("conversion-result").hidden = true; $("conversion-report").hidden = true;
    if (!source) { status(t("conversionPickFirst")); return; }
    id = crypto.randomUUID(); stopped = false; lock(true);
    $("conversion-progress").removeAttribute("value");
    status(t("conversionAnalyzing"));
    try {
      const result = await invoke("inspect_conversion_file", { id, source });
      if (stopped) { status(t("conversionCancelled")); return; }
      info = result;
      $("conversion-source").value = info.source;
      const kinds = { image: "conversionImage", audio: "conversionAudio", video: "conversionVideo" };
      $("conversion-kind").textContent = t(kinds[info.kind]);
      $("conversion-black").hidden = !info.black_video;
      $("conversion-tracks").textContent = info.tracks.map(track => `${track.index}: ${t(track.attached_picture ? "conversionCover" : track.kind === "audio" ? "conversionAudio" : track.kind === "video" ? "conversionVideo" : "conversionTracks")} · ${track.codec}${track.language ? ` · ${track.language}` : ""}`).join("\n");
      $("conversion-format").replaceChildren(...info.formats.map(format => {
        const node = document.createElement("option"); node.value = format;
        node.textContent = format === "mkv-lossless" ? t("conversionLosslessMkv") : format.toUpperCase(); return node;
      }));
      $("conversion-track").replaceChildren(...info.tracks.filter(track => track.kind === "audio").map(track => {
        const node = document.createElement("option"); node.value = track.index;
        node.textContent = `${track.index} · ${track.codec}${track.language ? ` · ${track.language}` : ""}`; return node;
      }));
      $("conversion-report").hidden = false;
      updateFormat(); status("");
    } catch (error) { status(errorText(error)); report("main", "conversion-inspect", error); }
    finally { lock(false); $("conversion-progress").value = 0; }
  }
  async function open(source = "") {
    if (dialog.open || busy) return;
    info = null; output = null; id = null;
    $("conversion-source").value = source;
    $("conversion-report").hidden = true; $("conversion-result").hidden = true;
    $("conversion-progress").value = 0; status(""); lock(false);
    dialog.showModal(); resize(true);
    $("conversion-directory").value = source ? source.replace(/[\\/][^\\/]+$/, "") : await invoke("default_download_directory");
    if (source) await analyze();
  }
  $("conversion-pick").onclick = async () => {
    try { const source = await invoke("pick_conversion_file"); if (source) { $("conversion-source").value = source; $("conversion-directory").value = source.replace(/[\\/][^\\/]+$/, ""); await analyze(); } }
    catch (error) { status(errorText(error)); }
  };
  $("conversion-directory-pick").onclick = async () => {
    try { const path = await invoke("pick_directory", { initialDirectory: $("conversion-directory").value || null }); if (path) $("conversion-directory").value = path; }
    catch (error) { status(errorText(error)); }
  };
  $("conversion-source").oninput = () => { info = null; $("conversion-report").hidden = true; lock(false); };
  $("conversion-analyze").onclick = analyze;
  $("conversion-format").onchange = updateFormat;
  $("conversion-start").onclick = async () => {
    if (busy || !info) return;
    stopped = false; lock(true); $("conversion-progress").value = 0;
    $("conversion-result").hidden = true; status(t("conversionWorking"));
    try {
      output = await invoke("convert_local_file", { id, format: $("conversion-format").value, audioTrack: audioFormats.has($("conversion-format").value) ? Number($("conversion-track").value) : null, directory: $("conversion-directory").value });
      status(t("conversionDone") + output); $("conversion-result").hidden = false; $("conversion-progress").value = 100;
    } catch (error) { status(errorText(error)); report("main", "conversion-run", error); }
    finally { lock(false); }
  };
  $("conversion-stop").onclick = async () => {
    if (!busy || !id) return;
    stopped = true; status(t("conversionStopping"));
    try { await invoke("cancel_file_conversion", { id }); } catch (error) { status(errorText(error)); }
  };
  $("conversion-result").onclick = () => invoke("reveal_converted_file", { path: output }).catch(error => status(errorText(error)));
  $("conversion-close").onclick = () => { if (!busy) dialog.close(); };
  dialog.addEventListener("cancel", event => { if (busy) { event.preventDefault(); $("conversion-stop").click(); } });
  dialog.addEventListener("close", () => resize(false));
  window.__TAURI__?.event?.listen?.("file-conversion-progress", event => { if (event.payload.id === id && busy) $("conversion-progress").value = event.payload.percent; });
  return { open };
};
