(function (root) {
  "use strict";
  const copy = {
    en: { title: "File parts", loading: "Reading confirmed blocks…", complete: "Confirmed", partial: "Partly confirmed", pending: "Not confirmed", unavailable: "This engine has not reported a block map. Overall progress does not identify byte positions.", sequential: "Continuous recording. Final size is unknown; there are no parallel file parts to display.", torrent_unavailable: "This torrent engine does not expose piece positions. A piece map cannot be inferred from the total percentage.", unknown_total: "Waiting for the file size and a confirmed block map.", summary: "{done} of {count} parts confirmed", hint: "Engine checkpoint · grouped by byte position · updates may lag the transfer", range: "{start}–{end}: {done} confirmed", error: "Could not read the block map. The download continues normally." },
    "pt-BR": { title: "Partes do arquivo", loading: "Lendo blocos confirmados…", complete: "Confirmado", partial: "Parcialmente confirmado", pending: "Sem confirmação", unavailable: "Este motor ainda não informou o mapa de blocos. O progresso geral não identifica a posição dos bytes.", sequential: "Gravação contínua. O tamanho final é desconhecido; não há partes paralelas do arquivo para exibir.", torrent_unavailable: "Este motor de torrent não expõe a posição das peças. Não é possível deduzir o mapa pela porcentagem total.", unknown_total: "Aguardando o tamanho do arquivo e um mapa de blocos confirmado.", summary: "{done} de {count} partes confirmadas", hint: "Registro do motor · agrupado por posição dos bytes · pode haver atraso na atualização", range: "{start}–{end}: {done} confirmado", error: "Não foi possível ler o mapa de blocos. O download continua normalmente." },
    "zh-CN": { title: "文件分块", loading: "正在读取已确认的数据块…", complete: "已确认", partial: "部分确认", pending: "未确认", unavailable: "此引擎尚未提供分块位置。总进度无法确定字节在文件中的位置。", sequential: "连续录制，最终大小未知，没有可显示的并行文件分块。", torrent_unavailable: "此种子引擎未提供分片位置，无法根据总百分比推断分片图。", unknown_total: "正在等待文件大小及已确认的分块图。", summary: "已确认 {done} / {count} 个分块", hint: "引擎检查点 · 按字节位置分组 · 更新可能稍有延迟", range: "{start}–{end}：已确认 {done}", error: "无法读取分块图，下载将正常继续。" },
  };
  const expanded = new Set();
  const snapshots = new Map();
  const pending = new Set();
  const requestedAt = new Map();
  function cellState(cell) {
    if (!(cell.end > cell.start) || cell.confirmed <= 0) return "pending";
    return cell.confirmed >= cell.end - cell.start ? "complete" : "partial";
  }
  const text = (template, values) => template.replace(/\{(\w+)\}/g, (_, key) => values[key]);
  function createTransferParts(task, { invoke, language, formatBytes }) {
    const t = copy[language] || copy.en;
    const details = document.createElement("details");
    details.className = "task-parts";
    details.dataset.partsId = task.id;
    details.open = expanded.has(task.id);
    const summary = document.createElement("summary");
    summary.textContent = t.title;
    const body = document.createElement("div");
    body.className = "task-parts-body";
    function draw() {
      const data = snapshots.get(task.id);
      body.replaceChildren();
      if (!data?.available) {
        body.textContent = data ? t[data.reason] || t.unavailable : t.loading;
        return;
      }
      const label = document.createElement("span");
      label.textContent = text(t.summary, { done: data.completed_count, count: data.parts_count });
      const map = document.createElement("div");
      map.className = "parts-map";
      map.setAttribute("role", "img");
      map.setAttribute("aria-label", label.textContent);
      for (const cell of data.cells || []) {
        const mark = document.createElement("span");
        const state = cellState(cell);
        mark.dataset.complete = String(state === "complete");
        mark.dataset.partial = String(state === "partial");
        mark.title = text(t.range, { start: formatBytes(cell.start), end: formatBytes(cell.end), done: formatBytes(cell.confirmed) });
        mark.setAttribute("aria-hidden", "true");
        map.append(mark);
      }
      const legend = document.createElement("div");
      legend.className = "parts-legend";
      for (const kind of ["complete", "partial", "pending"]) {
        const item = document.createElement("span");
        const swatch = document.createElement("i");
        swatch.dataset.kind = kind;
        item.append(swatch, document.createTextNode(t[kind]));
        legend.append(item);
      }
      const hint = document.createElement("small");
      hint.className = "parts-caption";
      hint.textContent = t.hint;
      body.append(label, map, legend, hint);
    }
    async function refresh() {
      if (!details.open || !details.isConnected || pending.has(task.id) || Date.now() - (requestedAt.get(task.id) || 0) < 2000) return;
      requestedAt.set(task.id, Date.now());
      pending.add(task.id);
      try {
        const data = await invoke("get_transfer_parts", { id: task.id });
        snapshots.set(task.id, data);
      } catch {
        snapshots.set(task.id, { available: false, reason: "error" });
      } finally {
        pending.delete(task.id);
        // The list can have been rebuilt during the request. Update the live row.
        for (const live of document.querySelectorAll(".task-parts[open]")) {
          if (live.dataset.partsId === task.id) live.drawParts?.();
        }
        if (snapshots.size > 200) {
          const oldest = snapshots.keys().next().value;
          snapshots.delete(oldest);
          requestedAt.delete(oldest);
        }
      }
    }
    details.drawParts = draw;
    details.refreshParts = refresh;
    details.addEventListener("toggle", () => {
      if (!details.isConnected) return;
      if (details.open) { expanded.add(task.id); draw(); void refresh(); }
      else expanded.delete(task.id);
    });
    details.append(summary, body);
    draw();
    return details;
  }
  root.createTransferParts = createTransferParts;
  if (typeof document !== "undefined") {
    let cursor = 0;
    setInterval(() => {
      if (document.visibilityState === "hidden") return;
      const open = [...document.querySelectorAll(".task-parts[open]")];
      for (let i = 0; i < Math.min(4, open.length); i++) {
        void open[(cursor + i) % open.length].refreshParts?.();
      }
      cursor = open.length ? (cursor + 4) % open.length : 0;
    }, 1000);
  }
  if (typeof module === "object" && module.exports) module.exports = { cellState, copy };
})(globalThis);
