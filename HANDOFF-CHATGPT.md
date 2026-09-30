# Handoff para o ChatGPT — apocalipse-download-manager (retomada 2026-09-29, sessão 2)

Documento pra dar contexto completo a um chat novo sem histórico. Leia inteiro antes de mexer em qualquer coisa. Se divergir do estado real (`git log`, código), **confie no repositório**, não neste texto. A versão anterior deste arquivo (auditoria do aria2-ultra como motor, Rapidgator/Shift) está preservada no histórico do git (`git log -- HANDOFF-CHATGPT.md`) — o que segue é uma reescrita completa cobrindo uma sessão de debugging bem mais longa e concreta.

Existe um documento irmão em `linuxhell/aria2-ultra` (branch `chatgpt/bep52-phase4`) — leia os dois.

## O que é este projeto

App desktop (Tauri: backend Rust em `apps/desktop/src-tauri/src/main.rs`, ~14800+ linhas, + `aria2.rs` pra falar com o aria2 via RPC) com extensão de navegador Manifest V3 (`browser-extension/*.js`) que intercepta downloads/magnet links do navegador e os entrega ao aria2-ultra.

## Branch de trabalho: `prep-aria2-ultra`

## Contexto crítico: como o usuário testa

O usuário reconstrói o app via CI (GitHub Actions, workflow `test-build.yml`, artefato de teste) e recarrega manualmente em `chrome://extensions` + reinstala o app desktop a cada rodada. Ele manda bundles de diagnóstico reais (exportados pelo "Apocalipse Forensic Debugger V4", um zip com `RELATORIO_PARA_IA.txt`, `summary.json`, `logs/by-component/*.jsonl` etc.) depois de cada teste. **Trate cada bundle como evidência primária — não adivinhe causa raiz sem ler os eventos reais.** O usuário já foi enganado por uma explicação minha não-verificada nesta sessão (eu disse "é o escalonador adaptativo, comportamento normal" sem checar os números — era mentira, os dados mostravam uma rampa real). Ele está com razão em cobrar isso. **Nunca afirme uma causa sem ter a evidência de log na mão.**

Também importante: `Build:` no `RELATORIO_PARA_IA.txt` costuma ser um hash de merge commit sintético do GitHub Actions (evento `pull_request`), **não bate com `git log` da branch**. Pra saber se o build testado já tem uma correção específica, compare o `createdAt` do `manifest.json` do bundle contra o horário de conclusão do run do CI correspondente (`mcp__github__actions_list` → `list_workflow_runs`, filtrar por branch), não o hash.

## Bugs reais encontrados e corrigidos nesta sessão (ordem cronológica, todos já commitados e enviados a `origin/prep-aria2-ultra`)

### 1. `bridgeConnected` global quebrando downloads saudáveis (fix antigo desta sessão, antes do que segue)
Duas checagens prematuras `if (!bridgeConnected) return false` em `background.js`/`takeBrowserDownload()` faziam downloads saudáveis falharem por causa de uma flag global mutável sendo pisada por chamadas de bridge concorrentes não relacionadas (heartbeat, diagnóstico). Removidas — a real chamada `bridgeRequest("/v1/download", ...)` já tinha fallback correto.

### 2. Causa raiz real do "botão de download/magnet não funciona" — `Extension context invalidated` (commits `d288447`, `f6a8e26`)
**Esta era a causa raiz de verdade**, confirmada com um print real do console do DevTools do usuário mostrando `Uncaught Error: Extension context invalidated. content.js:1098` — não era o clique em si, não era `downloadableLink()`.

Mecanismo: toda vez que o usuário reconstrói e recarrega a extensão pra testar um build novo, as abas **já abertas no navegador** continuam rodando a instância ANTIGA do content script, cujo `chrome.runtime` está morto. O handler de clique em `content.js` já chamava `event.preventDefault()`/`stopImmediatePropagation()` antes de tentar `chrome.runtime.sendMessage(...)` pro handoff de magnet/arquivo — quando essa chamada passou a lançar exceção síncrona em vez de falhar assincronamente, o clique era engolido por completo: sem navegação, sem handoff, sem log (porque até o diagnóstico usa o mesmo `chrome.runtime` morto). É por isso que 3 rodadas de diagnóstico anteriores a esta sessão não mostravam nada.

Correção em duas partes:
- Todos os handoffs do click handler (bypass, force, magnet, arquivo) passaram a usar o `sendRuntimeMessageQuietly()`/`extensionContextActive()` que já existiam no arquivo (usados só pelos timers de heartbeat/appearance-sync antes) — em vez de chamar `chrome.runtime.sendMessage` direto e sem guarda.
- **Mais importante**: o `background.js` já tinha um sistema de auto-reparo (`repairOpenCaptureTabs`, disparado em `chrome.runtime.onInstalled`) que reinjeta um content script novo em toda aba aberta assim que a extensão recarrega. Mas o script ANTIGO continuava com o listener de clique grudado no `document`, e como ele registra primeiro, sempre vencia a corrida contra o script novo (mesma fase de captura, mesmo elemento, ordem de registro) e ainda chamava `preventDefault`/`stopImmediatePropagation` antes de descobrir que estava morto. Agora o listener antigo checa `extensionContextActive()` **logo no início**, e se estiver morto, remove a si mesmo e retorna **sem tocar em preventDefault/stopPropagation** — deixando o evento passar pro listener novo (já reinjetado) resolver o clique.

Resultado validado com bundle real do usuário (build 0.3.183): `browser_download.detected → handoff_acknowledged → chrome_cancelled` com sucesso, tanto pra ISO direto quanto pra um caso via magnet/forced-prehook. **Confirmado funcionando.**

### 3. uupdump.net: retry infinito por hint de tamanho obsoleto (commit `a3fedae`)
`crates/apocalipse-core/src/download.rs`, `finish_download()`. uupdump.net gera a resposta do `get.php` por requisição (embute um session id), então uma sondagem prévia da extensão (prehook) via `expected_size` pode diferir por alguns bytes do Content-Length real da resposta que o desktop baixa segundos depois — mesmo o download em si sendo completo e consistente (`received == total` da própria resposta). O código tratava esse hint obsoleto como autoritativo e falhava um download bem-sucedido repetidamente, até o uupdump.net bloquear por excesso de requisições (HTTP 429). Também explica a pasta `.apocalipse-parts` residual e a opção de extrair não aparecer — o download nunca terminava de verdade.

Fix: `request.expected_size` só é aplicado quando a resposta **não** informou seu próprio Content-Length (`total.is_none()`). Dois testes de regressão adicionados (`a_stale_prehook_size_hint_never_fails_a_self_consistent_download`, `expected_size_hint_still_gates_a_response_with_no_content_length`).

**⚠️ Ainda não confirmado com um build novo** — o teste do usuário que reproduziu o erro (`0949d201-uupdump.zip`) foi feito ANTES do CI terminar de buildar este commit (comparei `createdAt` do bundle 18:55:08 vs conclusão do run `36635944189` às 21:56:52 UTC = 18:56:52 -03). Precisa reteste com build pós-21:57 UTC de 29/09.

### 4. Diagnóstico aprofundado do `not_paired` — NÃO RESOLVIDO (commit `318b712`)
Downloads diretos (via `chrome.downloads.onDeterminingFilename`, fora do content script) às vezes falham com `error=Error: not_paired` em `bridgeRequest()`, mesmo com:
- Mesmo perfil do Chrome, janela normal (usuário confirmou explicitamente, não é anônimo).
- Heartbeat autenticando com sucesso no mesmo timeframe (`bridge.health`/`bridge.request` centenas de vezes OK).
- `bridgeConnected: true` no manifest do bundle.

Isso não devia ser possível: `pairingToken` vem do mesmo `chrome.storage.local`, que não é particionado por aba/página — só por perfil/instância de extensão. Instrumentação adicionada em `background.js`/`takeBrowserDownload()` (no catch do `bridgeRequest`) que, na próxima falha, vai reportar: se a chave `pairingToken` existe (não só truthy), o tipo bruto do valor, e a contagem total de chaves no storage naquele momento — pra separar "chave nunca existiu" de "chave virou null/corrompida" de "storage inteiro resetado".

**Ainda esperando um bundle que capture essa falha com o build 0.3.184+ pra ter a resposta definitiva.** Hipóteses já descartadas: janela anônima (não é), perfil diferente do Chrome (usuário confirmou que é o mesmo).

Dado novo, não conclusivo: nos dois testes controlados de velocidade mais recentes (item de performance abaixo, ISOs ARM64 e x64 Insider), o handoff do download direto pro ADM funcionou limpo (`extension.popup.download_handed_off`, sem `not_paired`). Isso **não prova que o bug foi corrigido** — só que essas duas tentativas específicas não bateram nele. Continue tratando como aberto até aparecer (ou deixar de aparecer) de forma consistente em mais rodadas.

## Investigação fechada: "ADM mais lento que aria2 puro" — era variância de rede/CDN, não bug

Histórico rápido (pra quem só olhar o `git log` deste arquivo): a hipótese começou com um teste não controlado (ISO x64, `teste.log`) que mostrou 73.68s no CLI vs 90.96s no ADM (~24% mais lento), e uma rampa de vazão real e mensurável nos primeiros ~5-7s do lado do ADM. Duas hipóteses foram levantadas e **descartadas** com evidência do próprio usuário:
- `continue=true` retomando um `.aria2`/parcial de tentativa anterior — descartado: usuário confirmou que sempre excluía o arquivo da lista **e do disco** a cada falha, e a pasta do teste do `cmd` estava vazia.
- Polling de status do ADM a 100ms competindo por CPU — descartado por inspeção do código: esse intervalo agressivo só dura até o primeiro byte chegar (frequentemente <1s), não cobre os 5-7s da rampa observada.

**Teste controlado que resolveu a dúvida**: usuário gerou um link novo (ISO Windows 11 **ARM64**, 7937691648 bytes) e baixou pelo ADM e pelo `cmd` **em sequência, minutos um do outro**, mesmo link nos dois. Resultado:
- **ADM**: 79.51s → 99.9 MB/s médio.
- **CMD**: 76.12s → 104.3 MB/s médio.
- Diferença: **~4%**, não 24%.

E a rampa real do ADM neste teste (mesmas amostras de `performance/transfer-engine.jsonl`) foi rápida: 16 conexões já ativas em 1.10s, 51.2 MB/s aos 1.45s, **105.4 MB/s (praticamente o pico) aos 2.15s** — nada parecido com a rampa lenta de 5-7s do teste anterior.

**Conclusão**: o gap de 24% do primeiro teste não era um bug reproduzível do ADM — era a variável de confusão que eu levantei mas não tinha confirmado ainda (os dois links foram gerados ~14 minutos separados, a CDN da Microsoft/Akamai pode rotear pra bordas diferentes dependendo do momento). Controlando essa variável, a diferença cai pra ~4%, que é overhead normal de rodar por trás de RPC + coleta de diagnóstico, não vale a pena caçar mais.

**Segunda confirmação, independente** (ISO x64 Insider Preview, 8731574272 bytes, VPN desligada desta vez): ADM 85.83s → 101.7 MB/s; CMD 81.70s → 106.9 MB/s. **~4.8% de diferença** — bate com o resultado do ARM64. A rampa do ADM nesse teste foi ainda mais rápida: 72.6 MB/s já aos 0.66s, 119.9 MB/s aos 2.05s. Duas medições controladas e independentes convergindo pro mesmo resultado pequeno (~4-5%) fecha o caso. **Não reabrir sem um novo teste controlado mostrando gap grande de novo.**

## Regras gerais / avisos que continuam valendo

- **Toda vez que qualquer coisa em `browser-extension/` mudar, subir a versão em `manifest.json`** (`version` e `version_name`) e manter os testes que hardcodam a versão em sincronia (`grep -rl "0\.3\.NNN" tests/`). Instrução permanente do usuário.
- `.github/workflows/release.yml` tem um job `release` sem guarda de `if:` que cria tag/Release real e apaga tag antiga. **Nunca disparar sem confirmação explícita do usuário.** Pra artefato de teste, o PR (`#101`) já dispara `test-build.yml`/`validate-portable.yml` automaticamente a cada push.
- Não publicar nada como Release real do aria2-ultra sem confirmação explícita nova (mesmo que já tenha havido aprovação num momento anterior da conversa — confirmar de novo).
- Consultar o handoff do `aria2-ultra` antes de assumir comportamento default do motor.
- Rodar `node --test tests/*.test.cjs` (raiz do repo) e, pra mudanças em Rust, `cargo test -p apocalipse-core` + `cargo fmt --all -- --check` antes de cada commit.
- Este arquivo deve ser mantido atualizado ou apagado quando ficar obsoleto.

## Retomada 29/09 à noite — extensão 0.3.185 (artefato de teste)

Evidências: UUPDump terminou 8217/8217 bytes; Rapidgator terminou 4252486/4252486 bytes com nome `7-Zip_26.03.rar`. Faltava opção Extrair. TikTok mostrava mensagem de arquivo compactado para MP4. Corrigidos o caminho local antigo não limpo em `consumeBridgeDownload`, o filtro que excluía `mediaKind=file` e o refresh da opção de extração. Download HTTP simples limpa somente seus próprios resíduos; teste verifica que outras tarefas são preservadas.

TikTok: preview por Compartilhar + clique real em Copy funciona intermitentemente; popup cobria a instrução. Agora fecha quando espera Copy, e a instrução na página tem PT/EN/ZH. Resolução pode atender uma linha ambígua vinculada ao player, sem liberar faixas arbitrárias sem vínculo.

Cliques: eventos correlacionados click_observed / click_handoff_started / acknowledged ou failed, além dos motivos de clique ignorado. Runtime inválido continua sem poder enviar eventos: ausência de log não prova ausência de clique.

Metadados: teste registrou 40 conexões desde ~5s, mas metadados somente após ~31s. Novos campos incluem valores atuais (antes eram máximos), máximos, gid, tempo de addUri, primeira conexão, primeiro tamanho/byte e fase. RPC não expõe negociação ut_metadata ou causa de tracker/DHT: as fases observadas não devem ser anunciadas como causa definitiva. Motor usa log info, sem dump debug gigante.

Facebook Reel 2270956583757694: não patrocinado segundo usuário, yt-dlp falhou duas vezes com cannot parse data. A mensagem anterior sugeria indisponibilidade definitiva; agora descreve falha de extração e alternativas. external.failure_detail guarda a razão sanitizada e compacta. Download desse Reel ainda requer reteste; não afirmar que foi resolvido.

Validação local: node --test tests/*.test.cjs, cargo test -p apocalipse-core e cargo fmt passaram. Acompanhar test-build até o fim; não executar release.yml ou merge em main.
