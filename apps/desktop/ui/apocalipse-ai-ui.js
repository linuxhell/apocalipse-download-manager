(function () {
  "use strict";
  const AI = window.ApocalipseAI;
  if (!AI) return;

  const CHAT_KEY = "apocalipse.ai.chat.v1";
  const FIX_KEY = "apocalipse.ai.corrections.v1";
  const UPDATE_KEY = "apocalipse.ai.update.v1";
  const UPDATE_ACK_KEY = "apocalipse.ai.update-ack.v1";
  const RELEASES_URL = "https://github.com/linuxhell/apocalipse-download-manager/releases";
  const UPDATE_INTERVAL_MS = 6 * 60 * 60 * 1000;
  const MAX_MESSAGES = 160;
  const read = (key, fallback) => {
    try { return JSON.parse(localStorage.getItem(key) || "") || fallback; }
    catch { return fallback; }
  };
  const write = (key, value) => localStorage.setItem(key, JSON.stringify(value));
  const language = () => AI.localeOf(document.documentElement.lang || localStorage.getItem("apocalipse.language") || "en");
  const invoke = (command, args = {}) => window.__TAURI__?.core?.invoke(command, args);
  let messages = read(CHAT_KEY, []);
  let corrections = read(FIX_KEY, []);
  let updateState = read(UPDATE_KEY, null);
  let busy = false;

  const root = document.querySelector("#ai-conversation");
  const input = document.querySelector("#ai-input");
  const form = document.querySelector("#ai-form");
  const correctionDialog = document.querySelector("#ai-corrections-dialog");

  function persistMessages() {
    messages = messages.slice(-MAX_MESSAGES);
    write(CHAT_KEY, messages);
  }
  function persistCorrections() {
    corrections = corrections.slice(-100);
    write(FIX_KEY, corrections);
    updateAlert();
  }
  function allowedReleaseUrl(value) {
    try {
      const parsed = new URL(value || RELEASES_URL);
      return parsed.protocol === "https:"
        && parsed.hostname === "github.com"
        && (parsed.pathname === "/linuxhell/apocalipse-download-manager/releases"
          || parsed.pathname.startsWith("/linuxhell/apocalipse-download-manager/releases/"))
        ? parsed.href
        : RELEASES_URL;
    } catch {
      return RELEASES_URL;
    }
  }
  function updateAlert() {
    const dot = document.querySelector("#ai-alert-dot");
    dot.hidden = !corrections.some(item => item.status === "proposed");
    const button = document.querySelector(".ai-nav-button");
    const acknowledged = read(UPDATE_ACK_KEY, "");
    const pendingUpdate = Boolean(updateState?.available
      && updateState.latest
      && updateState.latest !== acknowledged);
    button?.classList.toggle("ai-update-available", pendingUpdate);
    if (button) {
      button.title = pendingUpdate
        ? AI.say(language(), "updateNavTitle", { latest: updateState.latest })
        : "";
    }
  }
  function acknowledgeUpdate(version) {
    if (version) write(UPDATE_ACK_KEY, version);
    updateAlert();
  }
  function timeLabel(value) {
    try { return new Intl.DateTimeFormat(language(), { hour: "2-digit", minute: "2-digit" }).format(new Date(value)); }
    catch { return ""; }
  }
  function bubble(message) {
    const article = document.createElement("article");
    article.className = `ai-message ai-message-${message.role}`;
    const content = document.createElement("p");
    const isGreeting = ["welcome", "greeting"].includes(message.kind)
      || Object.values(AI.copy).some(dictionary => dictionary.hello === message.text);
    const isUpdate = message.kind === "update_available";
    content.textContent = isGreeting
      ? AI.say(language(), "hello")
      : isUpdate
        ? AI.say(language(), "updateAvailable", {
            current: message.currentVersion || "—",
            latest: message.updateVersion || "—",
          })
        : message.text;
    const meta = document.createElement("small");
    meta.textContent = message.role === "assistant" ? `Apocalipse AI · ${timeLabel(message.at)}` : timeLabel(message.at);
    article.append(content);
    if (isUpdate && message.linkUrl) {
      const link = document.createElement("a");
      link.className = "ai-message-link";
      link.href = allowedReleaseUrl(message.linkUrl);
      link.textContent = AI.say(language(), "updateReleaseLink");
      link.rel = "noreferrer";
      link.onclick = event => {
        event.preventDefault();
        acknowledgeUpdate(message.updateVersion);
        invoke("open_apocalipse_releases", { url: allowedReleaseUrl(message.linkUrl) }).catch(() => {});
      };
      article.append(link);
    }
    article.append(meta);
    return article;
  }
  function renderMessages() {
    root.replaceChildren(...messages.map(bubble));
    root.scrollTop = root.scrollHeight;
  }
  function addMessage(role, text, kind = null, extra = {}) {
    messages.push({ id: crypto.randomUUID(), role, text, kind, at: Date.now(), ...extra });
    persistMessages();
    renderMessages();
  }
  function welcome() {
    if (!messages.length) addMessage("assistant", AI.say(language(), "hello"), "welcome");
    else if (messages.length === 1 && messages[0].role === "assistant"
      && Object.values(AI.copy).some(dictionary => dictionary.hello === messages[0].text)) {
      messages[0].kind = "welcome";
      persistMessages();
    }
  }
  function parseExtensionVersion(events) {
    const versions = [...String(events || "").matchAll(/extension_version=([0-9.]+)/g)].map(match => match[1]);
    return versions.at(-1) || document.querySelector("#extension-version")?.textContent?.match(/[0-9]+(?:\.[0-9]+)+/)?.[0] || "—";
  }
  async function context() {
    const [events, engineEvents, tasks] = await Promise.all([
      invoke("read_general_log").catch(() => ""),
      invoke("read_ai_diagnostics").catch(() => []),
      invoke("list_downloads").catch(() => []),
    ]);
    return {
      // The application-selected language is authoritative. Apocalipse AI must
      // never switch output language by guessing from the user's sentence.
      locale: language(), events, engineEvents, downloads: tasks, corrections, messages,
      appVersion: document.querySelector("#app-version")?.textContent?.replace(/^v/, "") || "—",
      extensionVersion: parseExtensionVersion(events),
    };
  }
  function applyResult(result) {
    if (!result.correctionId) return;
    const index = corrections.findIndex(item => item.id === result.correctionId);
    if (index < 0) return;
    if (result.remove) corrections.splice(index, 1);
    else corrections[index] = { ...corrections[index], status: result.status, updatedAt: Date.now() };
    persistCorrections();
    renderCorrections();
  }
  function storeUpdateStatus(status) {
    if (!status) return;
    updateState = {
      available: Boolean(status.update_available),
      current: status.current_version || "",
      latest: status.latest_version || "",
      releaseUrl: allowedReleaseUrl(status.release_url),
      checkedAt: Date.now(),
    };
    write(UPDATE_KEY, updateState);
    updateAlert();
  }
  function announceUpdate(status) {
    if (!status?.update_available) return;
    const exists = messages.some(message =>
      message.kind === "update_available" && message.updateVersion === status.latest_version);
    if (exists) return;
    addMessage(
      "assistant",
      AI.say(language(), "updateAvailable", {
        current: status.current_version,
        latest: status.latest_version,
      }),
      "update_available",
      {
        currentVersion: status.current_version,
        updateVersion: status.latest_version,
        linkUrl: allowedReleaseUrl(status.release_url),
      },
    );
  }
  async function checkForUpdate({ announce = true } = {}) {
    try {
      const status = await invoke("check_app_update");
      storeUpdateStatus(status);
      if (announce) announceUpdate(status);
      return status;
    } catch {
      return null;
    }
  }

  async function submit(text) {
    if (busy || !text.trim()) return;
    busy = true;
    form.querySelector("button").disabled = true;
    addMessage("user", AI.redactCredentialCommand(text.trim()));
    input.value = "";
    try {
      const ctx = await context();
      const result = AI.respond(text, ctx);
      if (result.action?.type === "check_app_update") {
        const status = await checkForUpdate({ announce: false });
        if (status) {
          result.text = AI.say(language(), status.update_available ? "updateAvailable" : "upToDate", {
            current: status.current_version, latest: status.latest_version,
          });
          if (status.update_available) {
            result.kind = "update_available";
            result.currentVersion = status.current_version;
            result.updateVersion = status.latest_version;
            result.linkUrl = allowedReleaseUrl(status.release_url);
          }
        } else {
          result.text = AI.say(language(), "updateUnavailable", { current: ctx.appVersion });
        }
      }
      if (result.action?.type === "clear_chat") {
        messages = [];
        persistMessages();
        renderMessages();
      }
      if (result.action?.type === "save_website_credential") {
        const { host, username, password } = result.action;
        await invoke("save_host_rule", {
          pattern: host, username, password, userAgent: "",
          connections: null, bandwidthLimit: null, clearPassword: false,
        });
        result.action.password = "";
      }
      if (result.prelude) {
        addMessage("assistant", result.prelude);
        await new Promise(resolve => setTimeout(resolve, 180));
      }
      applyResult(result);
      addMessage(
        "assistant",
        result.text,
        result.kind || (result.intent === "greeting" ? "greeting" : null),
        result.kind === "update_available"
          ? {
              currentVersion: result.currentVersion,
              updateVersion: result.updateVersion,
              linkUrl: result.linkUrl,
            }
          : {},
      );
      invoke("record_ui_diagnostic", {
        level: "INFO", event: "apocalipse_ai.response",
        detail: `intent=${result.intent} evidence_events=${AI.parseEvents(ctx.events).length} local=true`,
      }).catch(() => {});
    } catch {
      addMessage("assistant", AI.say(language(), AI.parseCredentialCommand(text) ? "credentialFailed" : "noEvidence"));
    } finally {
      busy = false;
      form.querySelector("button").disabled = false;
      input.focus();
    }
  }

  function statusLabel(status) {
    const keys = { proposed: "aiStatusProposed", testing: "aiStatusTesting", saved: "aiStatusSaved", confirmed: "aiStatusConfirmed", rejected: "aiStatusRejected" };
    const locale = language();
    const dictionaries = window.apocalipseCatalogs || null;
    return dictionaries?.[locale]?.[keys[status]] || document.querySelector(`[data-i18n="${keys[status]}"]`)?.textContent || status;
  }
  function actionLabel(key, fallback) {
    return document.querySelector(`[data-i18n="${key}"]`)?.textContent || fallback;
  }
  function renderCorrections() {
    const list = document.querySelector("#ai-corrections-list");
    const empty = document.querySelector("#ai-corrections-empty");
    empty.hidden = corrections.length > 0;
    list.replaceChildren();
    for (const correction of [...corrections].reverse()) {
      const row = document.createElement("article");
      row.className = "ai-correction-row";
      const title = document.createElement("div");
      title.append(
        Object.assign(document.createElement("b"), { textContent: correction.name }),
        Object.assign(document.createElement("small"), { textContent: `${correction.site || "Apocalipse"} · ${statusLabel(correction.status)}` }),
      );
      const actions = document.createElement("span");
      const copyButton = Object.assign(document.createElement("button"), { type: "button", textContent: actionLabel("aiCopyName", "Copy name") });
      copyButton.onclick = () => navigator.clipboard.writeText(correction.name).catch(() => {});
      const applyButton = Object.assign(document.createElement("button"), { type: "button", textContent: actionLabel("aiApplyCorrection", "Apply for testing") });
      applyButton.disabled = correction.status === "testing";
      applyButton.onclick = () => submit(`${language() === "pt-BR" ? "aplique" : language() === "zh-CN" ? "应用" : "apply"} ${correction.name}`);
      const deleteButton = Object.assign(document.createElement("button"), { type: "button", textContent: actionLabel("aiDeleteCorrection", "Delete") });
      deleteButton.onclick = () => { corrections = corrections.filter(item => item.id !== correction.id); persistCorrections(); renderCorrections(); };
      actions.append(copyButton, applyButton, deleteButton);
      row.append(title, actions);
      list.append(row);
    }
  }

  form.addEventListener("submit", event => { event.preventDefault(); submit(input.value); });
  input.addEventListener("keydown", event => {
    if (event.key === "Enter" && !event.shiftKey) { event.preventDefault(); form.requestSubmit(); }
  });
  document.querySelector("#ai-open-corrections").onclick = () => { renderCorrections(); correctionDialog.showModal(); };
  document.querySelectorAll("[data-ai-corrections-close]").forEach(button => { button.onclick = () => correctionDialog.close(); });
  document.querySelector("#ai-delete-all-corrections").onclick = () => {
    if (!corrections.length) return;
    corrections = corrections.filter(item => ["testing", "confirmed"].includes(item.status));
    persistCorrections(); renderCorrections();
  };
  window.addEventListener("apocalipse-ai-opened", () => { welcome(); renderMessages(); updateAlert(); input.focus(); });
  window.addEventListener("apocalipse-language-changed", () => { welcome(); renderMessages(); renderCorrections(); updateAlert(); });
  window.addEventListener("storage", event => {
    if (event.key === FIX_KEY) {
      corrections = read(FIX_KEY, []);
      renderCorrections();
      updateAlert();
    }
    if (event.key === UPDATE_KEY || event.key === UPDATE_ACK_KEY) {
      updateState = read(UPDATE_KEY, null);
      updateAlert();
      renderMessages();
    }
  });
  updateAlert();
  window.setTimeout(() => checkForUpdate({ announce: true }), 1200);
  window.setInterval(() => checkForUpdate({ announce: true }), UPDATE_INTERVAL_MS);
})();
