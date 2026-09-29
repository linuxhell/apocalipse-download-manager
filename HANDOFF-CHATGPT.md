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

## Investigação em andamento, não resolvida: ADM mais lento que aria2 puro pra mesmo arquivo/servidor

O usuário rodou o **mesmo** aria2-ultra.exe via `cmd`, direto (sem ADM), baixando a mesma URL do ISO do Windows 11 (`software.download.prss.microsoft.com`), e me mandou o `--log-level=debug` completo (`teste.log`, ~4 milhões de linhas). Comparação real que eu fiz (não é chute, são números do log):

- **CLI puro**: início `19:09:31.531`, fim (`Download complete`) `19:10:45.213` → **73.68s** pra 8172068864 bytes → **110.9 MB/s médio**.
- **ADM** (bundle anterior, mesmo arquivo): início `18:55:32.082`, fim `18:57:03.044` → **90.96s** → **89.8 MB/s médio**. Cerca de **24% mais lento**.

Em ambos os casos as 16 conexões TCP abrem quase instantaneamente (CLI: todas as 16 entre `19:09:31.552` e `19:09:31.644`, ~90ms; ADM: `connections=16` já aos 0.71s). **Não é diferença na contagem de conexões nem velocidade de abrir socket.** A diferença está em como a vazão agregada sobe depois disso:
- ADM (amostras reais de `performance/transfer-engine.jsonl`, taxa instantânea `computedDeltaBytesPerSecond`): 0.9 MB/s aos 0.71s → 11.2 MB/s aos 1.43s → 54.1 MB/s aos 1.78s → 73.9 MB/s aos 2.83s → só estabiliza perto de 85-100 MB/s por volta dos 5-7s.
- CLI: não tem telemetria de bytes/s no log bruto do aria2 (é log `--log-level=debug` do binário, sem barra de progresso capturada), mas a densidade de eventos `socket: read:1` por segundo (proxy grosseiro de atividade) já está em ~9000/s no segundo seguinte à conexão (`19:09:32`), sem o mesmo período prolongado de quase-zero que o ADM mostra.

**Suspeito mais concreto ainda não confirmado**: `apps/desktop/src-tauri/src/aria2.rs`, `add_download()`, linha ~298: `options.insert("continue".into(), Value::String("true".into()));` — enviado incondicionalmente em toda chamada `aria2.addUri` do ADM. Se o destino já tinha um arquivo parcial/`.aria2` de uma tentativa anterior (bem provável nesta sessão especificamente, já que o mesmo ISO foi tentado várias vezes por causa do bug do `not_paired`), aria2 faria trabalho de retomada antes de puxar bytes novos — o que bateria com uma rampa de alguns segundos. Isso **não foi confirmado**, só é a explicação mais plausível que encontrei até agora batendo com a diferença real medida.

**Próximos passos pra quem pegar isso**:
1. Perguntar ao usuário o comando `aria2c` exato usado no teste do `cmd` (flags de `--continue`, `--file-allocation`, etc.) e se a pasta de destino do teste do `cmd` estava vazia ou já tinha um arquivo/`.aria2` de tentativa anterior.
2. Reproduzir o teste do ADM apontando pra um destino garantidamente limpo (sem `.aria2`/parcial prévio) e comparar de novo.
3. Se `continue=true` for mesmo a causa, considerar: só enviar `continue=true` quando existir de fato um `.aria2`/arquivo parcial no destino (checar antes de montar as options), em vez de sempre.
4. **Não afirmar conclusão nenhuma sobre isso sem novo log real comparando as duas condições.**

## Regras gerais / avisos que continuam valendo

- **Toda vez que qualquer coisa em `browser-extension/` mudar, subir a versão em `manifest.json`** (`version` e `version_name`) e manter os testes que hardcodam a versão em sincronia (`grep -rl "0\.3\.NNN" tests/`). Instrução permanente do usuário.
- `.github/workflows/release.yml` tem um job `release` sem guarda de `if:` que cria tag/Release real e apaga tag antiga. **Nunca disparar sem confirmação explícita do usuário.** Pra artefato de teste, o PR (`#101`) já dispara `test-build.yml`/`validate-portable.yml` automaticamente a cada push.
- Não publicar nada como Release real do aria2-ultra sem confirmação explícita nova (mesmo que já tenha havido aprovação num momento anterior da conversa — confirmar de novo).
- Consultar o handoff do `aria2-ultra` antes de assumir comportamento default do motor.
- Rodar `node --test tests/*.test.cjs` (raiz do repo) e, pra mudanças em Rust, `cargo test -p apocalipse-core` + `cargo fmt --all -- --check` antes de cada commit.
- Este arquivo deve ser mantido atualizado ou apagado quando ficar obsoleto.
