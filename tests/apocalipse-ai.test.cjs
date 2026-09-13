const assert = require('node:assert/strict');
const { test } = require('node:test');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');

const AI = require('../apps/desktop/ui/apocalipse-ai-core.js');
const html = readFileSync(join(__dirname, '../apps/desktop/ui/index.html'), 'utf8');
const app = readFileSync(join(__dirname, '../apps/desktop/ui/app.js'), 'utf8');
const aiUi = readFileSync(join(__dirname, '../apps/desktop/ui/apocalipse-ai-ui.js'), 'utf8');
const css = readFileSync(join(__dirname, '../apps/desktop/ui/styles.css'), 'utf8');
const desktop = readFileSync(join(__dirname, '../apps/desktop/src-tauri/src/main.rs'), 'utf8');

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
