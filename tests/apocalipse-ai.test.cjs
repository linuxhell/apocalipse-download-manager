const assert = require('node:assert/strict');
const { test } = require('node:test');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');

const AI = require('../apps/desktop/ui/apocalipse-ai-core.js');
const LocalModel = require('../apps/desktop/ui/apocalipse-ai-local-model.js');
const html = readFileSync(join(__dirname, '../apps/desktop/ui/index.html'), 'utf8');
const app = readFileSync(join(__dirname, '../apps/desktop/ui/app.js'), 'utf8');
const aiUi = readFileSync(join(__dirname, '../apps/desktop/ui/apocalipse-ai-ui.js'), 'utf8');
const css = readFileSync(join(__dirname, '../apps/desktop/ui/styles.css'), 'utf8');
const desktop = readFileSync(join(__dirname, '../apps/desktop/src-tauri/src/main.rs'), 'utf8');
const downloadCore = readFileSync(join(__dirname, '../crates/apocalipse-core/src/download.rs'), 'utf8');

test('Apocalipse AI is a real local page loaded before its UI controller', () => {
  assert.match(html, /data-page="ai"/);
  assert.match(html, /id="ai-panel"/);
  assert.ok(html.indexOf('apocalipse-ai-core.js') < html.indexOf('apocalipse-ai-ui.js'));
  assert.match(app, /ai: "aiDescription"/);
});

test('Apocalipse AI answers in the configured language', () => {
  assert.equal(AI.respond('oi', { locale: 'pt-BR' }).text.startsWith('Olá!'), true);
  assert.equal(AI.respond('hello', { locale: 'en' }).text.startsWith('Hello!'), true);
  assert.equal(AI.respond('你好', { locale: 'zh-CN' }).text.startsWith('你好！'), true);
});

test('TikTok guidance uses the selected player instead of naming VLC', () => {
  const answer = AI.respond('como visualizar no tiktok?', { locale: 'pt-BR' }).text;
  assert.equal(answer, 'Para visualizar um vídeo do TikTok pela extensão, clique em “Visualizar”. Quando a janela de compartilhamento abrir, clique no botão “Copy”. O Apocalipse usará o endereço copiado para abrir o vídeo no player que você definiu.');
  assert.doesNotMatch(answer, /VLC/i);
});

test('natural spelling variants still diagnose a missing site button', () => {
  for (const question of [
    'por que o botão de download não apareceu no vídeo do site exemplo.com?',
    'porque o botao de baixar nao apareceu no site exemplo.com?',
    'pq o button download sumiu no site exemplo.com?',
  ]) {
    const result = AI.respond(question, { locale: 'pt-BR', events: '{"event":"page_opened","detail":"url=https://exemplo.com"}' });
    assert.equal(result.intent, 'diagnosis');
    assert.equal(result.prelude, 'Vou analisar esse problema.');
  }
});

test('Rapidgator 404 is explained from actual evidence without suggesting resume', () => {
  const events = JSON.stringify({ level: 'ERROR', event: 'http.failed', detail: 'url=https://s1.rapidgator.net/download/id error=404 Not Found' });
  const answer = AI.respond('por que não baixou no Rapidgator?', { locale: 'pt-BR', events }).text;
  assert.match(answer, /erro 404/);
  assert.match(answer, /retomar a mesma tarefa não funcionará/);
});

test('correction feedback closes one hypothesis before another analysis', () => {
  const corrections = [{ id: 'one', name: 'Site X – player tardio', status: 'testing' }];
  const failed = AI.respond('a correção não funcionou, continua igual', { locale: 'pt-BR', corrections });
  assert.equal(failed.status, 'rejected');
  assert.equal(failed.analyzeAgain, true);
  assert.match(failed.text, /vou analisar os novos registros/i);
  const confirmed = AI.respond('funcionou', { locale: 'pt-BR', corrections });
  assert.equal(confirmed.status, 'confirmed');
  const saved = [{ id: 'two', name: 'Site X – botão tardio', status: 'saved' }];
  assert.equal(AI.respond('aplique Site X – botão tardio', { locale: 'pt-BR', corrections: saved }).status, 'testing');
  assert.equal(AI.respond('remover Site X – botão tardio', { locale: 'pt-BR', corrections: saved }).remove, true);
});

test('Pixeldrain compatibility is exact-host-only and visible to diagnostics', () => {
  assert.match(desktop, /site_connection_override/);
  assert.match(desktop, /value == "pixeldrain\.com" \|\| value\.ends_with\("\.pixeldrain\.com"\)/);
  assert.match(desktop, /site_rule\.pixeldrain_single_connection/);
  assert.match(desktop, /connections=1/);
  assert.match(AI.respond('qual regra de conexão do pixeldrain?', { locale: 'pt-BR' }).text, /uma conexão/);
});

test('direct downloads use automatic high-speed defaults without bypassing site rules', () => {
  assert.match(app, /connectionsOverride: null/);
  assert.match(app, /taskConnectionsManuallyChanged\s*\?\s*Number/);
  assert.match(app, /taskConnectionsManuallyChanged = false/);
  assert.match(desktop, /limits\.connections_per_download\.max\(16\)/);
  assert.match(downloadCore, /MIN_SEGMENT_CHUNK_SIZE: u64 = 4 \* 1024 \* 1024/);
  assert.match(downloadCore, /MAX_SEGMENT_CHUNK_SIZE: u64 = 32 \* 1024 \* 1024/);
  assert.match(downloadCore, /adaptive_chunk_size/);
  assert.match(downloadCore, /download_from_sources/);
  assert.match(downloadCore, /WORKER_START_INTERVAL_MS: u64 = 35/);
  assert.match(desktop, /task\.connections_override = site_connection_override\(&url, connections_override\)/);
});

test('Apocalipse AI uses theme variables and includes light-theme readability', () => {
  const aiCss = css.slice(css.indexOf('/* Apocalipse AI'));
  for (const token of ['var(--text)', 'var(--muted)', 'var(--surface)', 'var(--input)', 'var(--line)', 'var(--accent)']) assert.match(aiCss, new RegExp(token.replace(/[()]/g, '\\$&')));
  assert.match(css, /data-theme="pearlblue"/);
  assert.match(css, /data-theme="polarmint"/);
});

test('diagnostic details remove common secret query fields before an AI answer', () => {
  const events = JSON.stringify({ level: 'ERROR', event: 'http.failed', detail: 'https://example.test/file?token=secret&sig=private failed' });
  const answer = AI.respond('qual foi o erro?', { locale: 'pt-BR', events }).text;
  assert.doesNotMatch(answer, /secret|private/);
});

test('common typing mistakes are understood locally', () => {
  const cases = [
    ['pq a extenção não conecta?', /extensão detecta mídias/i],
    ['meu dowload está muito devagar', /download lento/i],
    ['como visualisar um vídeo?', /player externo/i],
    ['onde fica a pasta do donwload?', /pasta de destino/i],
    ['o tik tok abre como?', /janela de compartilhamento/i],
  ];
  for (const [question, expected] of cases) assert.match(AI.respond(question, { locale: 'pt-BR' }).text, expected);
});

test('follow-up questions inherit the previous site and feature', () => {
  const messages = [
    { role: 'user', text: 'O visualizar do TikTok não abriu o vídeo.' },
    { role: 'assistant', text: 'Vou analisar.' },
    { role: 'user', text: 'Como faço com ele?' },
  ];
  const result = AI.respond('Como faço com ele?', { locale: 'pt-BR', messages });
  assert.match(result.text, /clique no botão “Copy”/);
});

test('major Apocalipse areas have deterministic local answers', () => {
  const cases = [
    ['para que serve gravar?', /captura a mídia/i],
    ['o que tem em ferramentas?', /FFmpeg/],
    ['como funciona torrent?', /pares/i],
    ['como funciona ed2k?', /Kad/i],
    ['o que é Apocalipse Link?', /computadores autorizados/i],
    ['onde configuro proxy e dns?', /Configurações contém/i],
    ['a conversa vai para modelo externo?', /funciona localmente/i],
  ];
  for (const [question, expected] of cases) assert.match(AI.respond(question, { locale: 'pt-BR' }).text, expected);
});

test('task totals are calculated from the current application state', () => {
  const downloads = [
    { state: 'downloading' }, { state: 'paused' }, { state: 'failed' }, { state: 'completed' },
  ];
  const answer = AI.respond('quantas tarefas de download tenho?', { locale: 'pt-BR', downloads }).text;
  assert.match(answer, /4 tarefa/);
  assert.match(answer, /1 ativa/);
  assert.match(answer, /1 pausada/);
  assert.match(answer, /1 com falha/);
  assert.match(answer, /1 concluída/);
});

test('ambiguous Apocalipse questions ask a useful clarification', () => {
  assert.match(AI.respond('o botão não funciona', { locale: 'pt-BR' }).text, /sobre o vídeo|lista do Apocalipse/);
  assert.match(AI.respond('deu problema quando cliquei', { locale: 'pt-BR' }).text, /Baixar, Visualizar ou Gravar/);
});

test('answers stay in English and Simplified Chinese when selected', () => {
  assert.match(AI.respond('why is my download slow?', { locale: 'en' }).text, /^A slow download/);
  assert.match(AI.respond('下载为什么很慢？', { locale: 'zh-CN' }).text, /^下载缓慢/);
  assert.match(AI.respond('隐私怎么样？', { locale: 'zh-CN' }).text, /本地运行/);
});

test('Apocalipse AI always obeys the application-selected language, not the question language', () => {
  const engineEvents = [
    { event: 'http.transfer_started', detail: { activeConnections: 16 } },
    { event: 'http.engine_plan', detail: { activeConnections: 16, sourceCount: 2 } },
    { event: 'http.performance_sample', detail: { bytesPerSecond: 100 * 1024 * 1024, activeConnections: 16 } },
    { event: 'http.performance_sample', detail: { bytesPerSecond: 50 * 1024 * 1024, activeConnections: 16 } },
    { event: 'http.segment_completed', detail: { bytesPerSecond: 80 * 1024 * 1024, attempts: 2, sourceCount: 2, transport: 'HTTP/2' } },
  ];

  const englishUi = AI.respond('por que meu download está lento?', { locale: 'en', engineEvents });
  assert.match(englishUi.text, /^The transfer-engine telemetry/);
  assert.doesNotMatch(englishUi.text, /^A telemetria do motor/);

  const portugueseUi = AI.respond('why is my download slow?', { locale: 'pt-BR', engineEvents });
  assert.match(portugueseUi.text, /^A telemetria do motor/);

  const chineseUi = AI.respond('why is my download slow?', { locale: 'zh-CN', engineEvents });
  assert.match(chineseUi.text, /^传输引擎遥测/);
});

test('Apocalipse AI explains measured slowdown and mirror recovery from engine telemetry', () => {
  const engineEvents = [
    { event: 'http.engine_plan', detail: { activeConnections: 16, sourceCount: 3 } },
    { event: 'http.performance_sample', detail: { bytesPerSecond: 120 * 1024 * 1024, activeConnections: 16 } },
    { event: 'http.performance_sample', detail: { bytesPerSecond: 48 * 1024 * 1024, activeConnections: 16 } },
    { event: 'http.segment_completed', detail: { attempts: 2, sourceCount: 3, transport: 'HTTP/2' } },
  ];
  const answer = AI.respond('meu download está lento, qual o gargalo?', { locale: 'pt-BR', engineEvents }).text;
  assert.match(answer, /48\.0 MB\/s/);
  assert.match(answer, /120 MB\/s/);
  assert.match(answer, /16 conexão/);
  assert.match(answer, /3 fonte/);
  assert.match(answer, /HTTP\/2/);
  assert.match(answer, /40% do pico/);
  assert.match(answer, /fallback de mirrors/);
});

test('AI UI retrieves privacy-safe engine diagnostics and passes the selected language', () => {
  assert.match(aiUi, /invoke\("read_ai_diagnostics"\)/);
  assert.match(aiUi, /locale: language\(\)/);
  assert.match(aiUi, /application-selected language is authoritative/);
  assert.match(desktop, /fn read_ai_diagnostics/);
  assert.match(desktop, /ai_snapshot\(750\)/);
});

test('per-site transfer rules are exposed in all three languages and wired to the vault-backed backend', () => {
  assert.match(app, /hostRules: "Per-site transfer rules"/);
  assert.match(app, /hostRules: "Regras de transferência por site"/);
  assert.match(app, /hostRules: "按网站传输规则"/);
  assert.match(app, /invoke\("list_host_rules"\)/);
  assert.match(app, /invoke\("save_host_rule"/);
  assert.match(app, /invoke\("remove_host_rule"/);
  assert.match(desktop, /fn host_rule_for_url/);
  assert.match(desktop, /fn effective_credential_for_download/);
  assert.match(desktop, /host_rule_vault_account/);
  assert.match(desktop, /task\.connections_override\s*\.or_else\(\|\| host_rule/);
});

test('thumbnails use the validated persistent cache instead of direct remote rendering', () => {
  assert.match(app, /invoke\("resolve_thumbnail", \{ url \}\)/);
  assert.match(app, /resolveCachedThumbnail\(requestedThumbnail\)/);
  assert.match(app, /loadPreviewThumbnail\(image, thumbnail\)/);
  assert.doesNotMatch(app, /thumbnail\.src = task\.thumbnail/);
  assert.doesNotMatch(app, /image\.src = thumbnail/);
  assert.match(desktop, /mod thumbnail_cache;/);
  assert.match(desktop, /async fn resolve_thumbnail_internal/);
  assert.match(desktop, /prefetch_thumbnail\(app\.clone\(\), thumbnail\)/);
  assert.match(desktop, /thumbnail\.cache_hit/);
  assert.match(desktop, /thumbnail\.cached/);
});

test('site credential commands use the existing secure settings action', () => {
  const pt = AI.respond('adicione uma regra para o site https://exemplo.com nome de usuário juliano e senha segredo forte', { locale: 'pt-BR' });
  assert.equal(pt.intent, 'credential_save');
  assert.deepEqual(pt.action, {
    type: 'save_website_credential', host: 'https://exemplo.com', username: 'juliano', password: 'segredo forte', valid: true,
  });
  assert.doesNotMatch(pt.text, /segredo forte/);
  assert.match(pt.text, /exemplo.com/);

  const en = AI.respond('add credentials for site example.org username john password secret', { locale: 'en' });
  assert.equal(en.action.host, 'example.org');
  assert.equal(en.action.username, 'john');
  assert.equal(en.action.password, 'secret');

  const zh = AI.respond('为网站 example.cn 添加凭据，用户名 ming，密码 mimamodelo', { locale: 'zh-CN' });
  assert.equal(zh.action.host, 'example.cn');
  assert.equal(zh.action.username, 'ming');
  assert.equal(zh.action.password, 'mimamodelo');
});

test('passwords are redacted before credential commands enter chat history', () => {
  const raw = 'adicione para o site exemplo.com usuário eu senha minha senha secreta';
  const redacted = AI.redactCredentialCommand(raw);
  assert.doesNotMatch(redacted, /minha senha secreta/);
  assert.match(redacted, /••••••••/);
  assert.ok(aiUi.indexOf('AI.redactCredentialCommand(text.trim())') < aiUi.indexOf('AI.respond(text, ctx)'));
  assert.doesNotMatch(aiUi, /record_ui_diagnostic[\s\S]{0,400}password/);
});

test('incomplete credential commands never trigger a save action', () => {
  const result = AI.respond('adicione credenciais para o site exemplo.com usuário juliano', { locale: 'pt-BR' });
  assert.equal(result.intent, 'credential_invalid');
  assert.equal(result.action, undefined);
});

test('the exact chat commands reported by the user are understood', () => {
  const clear = AI.respond('limpe essa tela do chat', { locale: 'pt-BR', events: '{"level":"ERROR","detail":"403 access denied"}' });
  assert.equal(clear.intent, 'chat_clear');
  assert.equal(clear.action.type, 'clear_chat');
  assert.match(clear.text, /Conversa limpa/);

  const log = AI.respond('como está o seu log? algum erro?', {
    locale: 'pt-BR', events: [
      { level: 'INFO', event: 'started', detail: 'ok' },
      { level: 'ERROR', event: 'http.failed', detail: 'site=example.test error=403' },
    ],
  });
  assert.match(log.text, /1 erro/);
  assert.doesNotMatch(log.text, /^O site recusou/);
});

test('welcome message follows language changes and the composer stays fixed', () => {
  assert.match(aiUi, /isGreeting \? AI\.say\(language\(\), "hello"\)/);
  assert.match(aiUi, /apocalipse-language-changed/);
  assert.match(app, /dispatchEvent\(new CustomEvent\("apocalipse-language-changed"/);
  assert.match(css, /grid-template-rows:auto minmax\(0,1fr\) auto auto/);
  assert.match(css, /body:has\(#ai-panel:not\(\[hidden\]\)\) main \{ height:100vh; overflow:hidden; \}/);
});

test('first launch and tray reopen use the compact full-height 1280 by 850 window', () => {
  assert.match(desktop, /show_main_window[\s\S]*set_size\(tauri::LogicalSize::new\(1280\.0, 850\.0\)\)/);
  assert.match(readFileSync(join(__dirname, '../apps/desktop/src-tauri/tauri.conf.json'), 'utf8'), /"width": 1280,[\s\S]*"height": 850/);
});

test('saved greeting follows the visible language instead of its original text', () => {
  assert.match(aiUi, /document\.documentElement\.lang/);
  assert.match(aiUi, /dictionary\.hello === message\.text/);
  assert.match(aiUi, /result\.intent === "greeting"/);
});

test('Media navigation is removed and recordings are exclusive to Recordings', () => {
  assert.doesNotMatch(html, /data-page="media"/);
  assert.match(app, /activePage === "recordings"[\s\S]*downloads\.filter\(isRecording\)[\s\S]*!isRecording\(task\) && !isTorrent\(task\)/);
  assert.doesNotMatch(app, /activePage === "media"/);
});

test('torrent tasks are exclusive to Torrents', () => {
  assert.match(app, /activePage === "torrents"[\s\S]*downloads\.filter\(isTorrent\)[\s\S]*!isRecording\(task\) && !isTorrent\(task\)/);
});

test('short natural acknowledgements never trigger stale log diagnosis', () => {
  for (const word of ['ok', 'certo', 'bacana', 'legal']) {
    const result = AI.respond(word, { locale: 'pt-BR', events: '{"level":"ERROR","detail":"403 access denied"}' });
    assert.equal(result.intent, 'acknowledgement');
    assert.match(result.text, /Certo/);
    assert.equal(result.prelude, undefined);
  }
});

test('common conversation remains local and natural in all supported languages', () => {
  const cases = [
    ['pt-BR', 'valeu', 'thanks'],
    ['pt-BR', 'como você está?', 'wellbeing'],
    ['pt-BR', 'o que você consegue fazer?', 'capabilities'],
    ['en', 'thank you', 'thanks'],
    ['en', 'can you help me?', 'help'],
    ['en', 'see you later', 'goodbye'],
    ['zh-CN', '谢谢', 'thanks'],
    ['zh-CN', '你能做什么？', 'capabilities'],
    ['zh-CN', '没问题', 'acknowledgement'],
  ];
  for (const [locale, input, intent] of cases) {
    const result = AI.respond(input, { locale, events: [{ level: 'ERROR', detail: 'stale 403' }] });
    assert.equal(result.intent, intent, `${locale}: ${input}`);
    assert.equal(result.prelude, undefined, `${locale}: ${input}`);
  }
});

test('conversation vocabulary lives in an extensible offline intent model', () => {
  assert.equal(LocalModel.classify('sounds good').intent, 'acknowledgement');
  assert.equal(LocalModel.classify('我需要帮助').intent, 'help');
  assert.equal(LocalModel.classify('qual é a previsão do tempo?'), null);
  assert.ok(html.indexOf('apocalipse-ai-local-model.js') < html.indexOf('apocalipse-ai-core.js'));
});

test('time questions use the computer clock and selected language locally', () => {
  const now = new Date(2026, 8, 13, 14, 7, 0);
  const pt = AI.respond('que horas são?', { locale: 'pt-BR', now });
  const en = AI.respond('what time is it?', { locale: 'en', now });
  const zh = AI.respond('现在几点？', { locale: 'zh-CN', now });
  assert.equal(pt.intent, 'current_time');
  assert.match(pt.text, /14:07/);
  assert.match(en.text, /2:07 PM/);
  assert.match(zh.text, /14:07/);
  for (const result of [pt, en, zh]) assert.equal(result.prelude, undefined);
});

test('update questions use the official checker and never guess', () => {
  const result = AI.respond('tem atualização?', { locale: 'pt-BR' });
  assert.equal(result.intent, 'update_check');
  assert.equal(result.action.type, 'check_app_update');
  assert.match(aiUi, /invoke\("check_app_update"\)/);
  assert.match(desktop, /releases\/latest/);
  assert.match(desktop, /timeout\(Duration::from_secs\(8\)\)/);
});

test('natural questions about a named site search its diagnostic records', () => {
  const found = AI.respond('alguma informação sobre o site rsload.net em seu log?', {
    locale: 'pt-BR', events: [
      { level: 'INFO', event: 'page.opened', detail: 'host=outro.test' },
      { level: 'INFO', event: 'credential.matched', detail: 'host=rsload.net result=available' },
    ],
  });
  assert.match(found.text, /1 registro/);
  assert.match(found.text, /rsload\.net/);
  assert.doesNotMatch(found.text, /especializada no Apocalipse/);

  const empty = AI.respond('há algo do site ausente.test nos registros?', { locale: 'pt-BR', events: [] });
  assert.match(empty.text, /Não encontrei registros/);
});
