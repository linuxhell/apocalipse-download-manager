// Diagnostics v3: bounded, data-minimizing records. No cookies, page text or media bytes.
(() => {
  if (globalThis.ADM_DIAG_CORE) return;
  const MAX_EVENTS = 1200, MAX_BATCH = 40, MAX_DETAIL_BYTES = 6000;
  const uuid = value => typeof value === 'string' && /^[a-f0-9]{8}(?:-[a-f0-9]{4}){3}-[a-f0-9]{12}$/i.test(value);
  const code = value => typeof value === 'string' && /^[a-z0-9_.:-]{1,96}$/i.test(value);
  const enums = new Set(['reason','kind','mediaKind','contentType','method','component','world','readyState',
    'sourceScheme','eventType','stage','status','state','handler','result','engine','mode','command','script','errorName','decision','transport',
    'platform','action','route','source','bindingSource','failureStage','matchResult','candidateType','menuState','permalinkSource','canonicalClass','resolverReason','outcome']);
  const forbidden = /cookie|authorization|password|passwd|secret|token|header|body|title|text|html|fileName|stack|message/i;
  const digest = async (salt, value) => {
    const bytes = new TextEncoder().encode(`${salt}\u0000${String(value)}`);
    const hash = await crypto.subtle.digest('SHA-256', bytes);
    return Array.from(new Uint8Array(hash)).map(b => b.toString(16).padStart(2, '0')).join('').slice(0, 24);
  };
  const safeUrl = async (value, salt) => {
    if (!value) return { scheme: 'empty' };
    try {
      const url = new URL(String(value));
      return { ref: `u-${await digest(salt, value)}`, scheme: url.protocol.replace(':',''),
        // Domains only, never authority userinfo, paths, queries or fragments.
        host: ['http:','https:'].includes(url.protocol) ? url.hostname.slice(0, 200) : '',
        queryPresent: Boolean(url.search), fragmentPresent: Boolean(url.hash) };
    } catch { return { ref: `u-${await digest(salt, value)}`, scheme: 'invalid' }; }
  };
  async function clean(value, salt, key = '', depth = 0) {
    if (depth > 5) return '[depth-limit]';
    if (forbidden.test(key)) return '[redacted]';
    if (value === null || typeof value === 'boolean') return value;
    if (typeof value === 'number') return Number.isFinite(value) ? value : null;
    if (typeof value === 'string') {
      if (/url$|src$|sourceRef$|poster$|resourceRef$|pageRef$/i.test(key)) return safeUrl(value, salt);
      if (uuid(value) && /Id$|^id$|taskRefs/.test(key)) return value;
      if (/^(?:u|s)-[a-f0-9]{24}$/.test(value)) return value;
      if (key === 'contentType' && /^[a-z0-9.+-]+\/[a-z0-9.+-]+$/i.test(value)) return value;
      if (key === 'host' && /^[a-z0-9.-]{1,200}$/i.test(value)) return value;
      if ((enums.has(key) || key === 'scheme' || key === 'version') && code(value)) return value;
      return `s-${await digest(salt, value)}`;
    }
    if (Array.isArray(value)) return Promise.all(value.slice(0, 24).map(v => clean(v, salt, key, depth + 1)));
    if (value && typeof value === 'object') {
      const out = {};
      for (const [name, item] of Object.entries(value).slice(0, 48)) {
        if (/^[a-zA-Z][a-zA-Z0-9_]{0,63}$/.test(name)) out[name] = await clean(item, salt, name, depth + 1);
      }
      return out;
    }
    return null;
  }
  async function record(input, session, origin = {}) {
    if (!input || typeof input !== 'object' || !session || !uuid(session.sessionId) || !code(input.event)) return null;
    const detail = await clean(input.detail || {}, session.salt);
    const bounded = JSON.stringify(detail).length > MAX_DETAIL_BYTES ? { truncated: true } : detail;
    return { schemaVersion: 3, id: uuid(input.id) ? input.id : crypto.randomUUID(),
      sessionId: session.sessionId, contextId: uuid(input.contextId) ? input.contextId : origin.contextId,
      sequence: Number.isSafeInteger(input.sequence) && input.sequence >= 0 ? input.sequence : 0,
      traceId: uuid(input.traceId) ? input.traceId : null, event: input.event,
      level: ['DEBUG','INFO','WARN','ERROR'].includes(input.level) ? input.level : 'INFO',
      component: code(origin.component || input.component) ? origin.component || input.component : 'unknown',
      version: code(input.version) ? input.version : 'unknown',
      clientTimestamp: Number.isFinite(input.clientTimestamp) ? input.clientTimestamp : Date.now(),
      monoMs: Number.isFinite(input.monoMs) ? input.monoMs : null,
      tabId: Number.isInteger(origin.tabId) ? origin.tabId : null,
      frameId: Number.isInteger(origin.frameId) ? origin.frameId : null,
      detail: bounded };
  }
  const isActive = config => Boolean(config?.sessionId && config.active && config.expiresAt > Date.now());
  globalThis.ADM_DIAG_CORE = { MAX_EVENTS, MAX_BATCH, uuid, code, clean, record, digest, safeUrl, isActive };
})();
