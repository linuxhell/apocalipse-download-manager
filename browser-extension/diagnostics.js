// Diagnostic v3. Never changes routing, plays media, reads cookies or uploads bodies.
(() => {
  if (globalThis.ApocalipseDiagnostics) return;
  const producerId = crypto.randomUUID();
  let sequence = 0, salt = producerId, capture = null;
  const actionIds = new WeakMap(), players = new WeakMap();
  const sensitive = /(?:cookie|authorization|password|passwd|secret|token|credential|body|html|caption|title|filename|filepath)/i;
  const uuid = value => /^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$/i.test(value || '');
  const loaded = chrome.storage.local.get({ admDiagnosticSession: null }).then(value => {
    capture = value.admDiagnosticSession;
    if (capture?.salt) salt = capture.salt;
  }).catch(() => {});
  chrome.storage.onChanged.addListener((changes, area) => {
    if (area === 'local' && changes.admDiagnosticSession) {
      capture = changes.admDiagnosticSession.newValue;
      if (capture?.salt) salt = capture.salt;
    }
  });
  async function resource(value) {
    const text = String(value || '');
    let parsed;
    try { parsed = new URL(text); } catch { return { scheme: text ? 'unparseable' : 'empty' }; }
    const hash = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(`${salt}\0${text}`));
    const id = [...new Uint8Array(hash)].slice(0, 12).map(b => b.toString(16).padStart(2, '0')).join('');
    let host = parsed.hostname;
    if (parsed.protocol === 'blob:') { try { host = new URL(parsed.pathname).hostname; } catch {} }
    return { scheme: parsed.protocol.slice(0, -1), host, resourceId: id, queryParameterCount: [...parsed.searchParams].length };
  }
  async function safe(value, key = '', depth = 0) {
    if (depth > 5) return '[depth-limit]';
    if (value == null || typeof value === 'boolean') return value;
    if (sensitive.test(key)) return '[redacted]';
    if (typeof value === 'number') return Number.isFinite(value) ? value : null;
    if (Array.isArray(value)) return Promise.all(value.slice(0, 24).map(item => safe(item, key, depth + 1)));
    if (typeof value === 'object') {
      const output = {};
      for (const [name, item] of Object.entries(value).slice(0, 48)) {
        if (!/^[A-Za-z][A-Za-z0-9_]{0,60}$/.test(name)) continue;
        output[name] = await safe(item, name, depth + 1);
      }
      return output;
    }
    let text = String(value).slice(0, 1200);
    if (/^(?:https?|blob|file|data):/i.test(text)) return resource(text);
    if (/(?:cookie|authorization|password|passwd|secret|token|credential)\s*[:=]|\bBearer\s+/i.test(text)) return '[redacted-sensitive-text]';
    text = text.replace(/(?:https?|blob|file|data):[^\s"'<>]+/gi, '[resource-redacted]');
    text = text.replace(/[A-Za-z]:[\\/][^\s"'<>]*/g, '[local-path]')
      .replace(/\/(?:home|Users|mnt|tmp)\/[^\s"'<>]*/g, '[local-path]');
    return text.slice(0, 512);
  }
  const beginAction = element => { const id = crypto.randomUUID(); if (element) actionIds.set(element, { id, at: Date.now() }); return id; };
  const actionFor = element => {
    if (!element || (typeof element !== 'object' && typeof element !== 'function')) return crypto.randomUUID();
    const previous = actionIds.get(element);
    if (previous && Date.now() - previous.at < 800) return previous.id;
    const id = crypto.randomUUID(); actionIds.set(element, { id, at: Date.now() }); return id;
  };
  const player = element => {
    if (!element) return null;
    const source = String(element.currentSrc || element.src || '');
    let current = players.get(element);
    if (!current) current = { id: crypto.randomUUID(), source, revision: 0 };
    if (source !== current.source) { current.source = source; current.revision++; }
    players.set(element, current);
    return { playerId: current.id, revision: current.revision, source, connected: Boolean(element.isConnected),
      readyState: element.readyState, networkState: element.networkState, paused: element.paused,
      duration: Number.isFinite(element.duration) ? element.duration : null,
      width: element.videoWidth, height: element.videoHeight, mediaErrorCode: element.error?.code || null };
  };
  async function emit(event, data = {}, actionId = null, level = 'INFO', component = 'extension.content') {
    const seq = ++sequence;
    const clientTimestamp = new Date().toISOString();
    const monotonicMs = globalThis.performance?.now?.() || 0;
    try {
      await loaded;
      const record = { schemaVersion: 3, eventId: crypto.randomUUID(), producerId, sequence: seq,
        clientTimestamp, monotonicMs, sessionId: capture?.id || producerId,
        actionId: uuid(actionId) ? actionId : null, event: String(event).replace(/[^A-Za-z0-9_.-]/g, '_').slice(0, 100),
        level: ['DEBUG', 'INFO', 'WARN', 'ERROR'].includes(level) ? level : 'INFO', component,
        extensionVersion: chrome.runtime.getManifest().version, data: await safe(data) };
      if (globalThis.ApocalipseDiagnosticSink) return await globalThis.ApocalipseDiagnosticSink(record);
      return await chrome.runtime.sendMessage({ type: 'APOCALIPSE_DIAGNOSTIC_EVENT_V3', record });
    } catch { /* Diagnostics must never prevent downloads or interact with the page. */ }
  }
  const active = () => Boolean(capture && capture.expiresAt > Date.now());
  globalThis.ApocalipseDiagnostics = { emit, safe, resource, actionFor, beginAction, player, active, uuid,
    ready: loaded, producerId, setSession: value => { capture = value; if (value?.salt) salt = value.salt; }, session: () => capture };
})();
