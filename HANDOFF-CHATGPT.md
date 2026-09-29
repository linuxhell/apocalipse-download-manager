# Handoff para o ChatGPT — apocalipse-download-manager (retomada 2026-09-29)

Documento pra dar contexto completo a um chat novo do ChatGPT sem histórico. Leia inteiro antes de mexer em qualquer coisa. Se divergir do estado real (`git log`, código), **confie no repositório**, não neste texto.

Existe um documento irmão, `HANDOFF-CHATGPT.md`, no repositório `linuxhell/aria2-ultra` (branch `chatgpt/bep52-phase4`) — leia os dois, são complementares. Lá é o fork do aria2 que este app consome como motor de download; aqui é o app desktop em si.

## O que é este projeto

App desktop (Tauri: backend Rust em `apps/desktop/src-tauri/src/main.rs`, ~14800+ linhas, + `aria2.rs` pra falar com o aria2 via RPC) com extensão de navegador Manifest V3 (`browser-extension/*.js`) que intercepta downloads do navegador e os entrega ao aria2 pra baixar com múltiplas conexões (download direto) ou BitTorrent v1/v2/híbrido.

## Branch de trabalho: `prep-aria2-ultra`

Preparando o app pra usar o **aria2-ultra** (`linuxhell/aria2-ultra`) como motor no lugar do aria2 vanilla/FerroDownload que era usado antes.

## O que já foi feito nesta leva de trabalho (mais recente primeiro)

### 1. Corrigido bug real: `--file-allocation=none` forçado anulava o novo default do aria2-ultra (commit `0e2d5b2`)
O aria2-ultra (fork irmão) mudou o default de `-a/--file-allocation` pra `trunc` em downloads diretos (mais rápido, ver handoff de lá). Mas este app forçava `--file-allocation=none` em dois lugares:
- `apps/desktop/src-tauri/src/aria2.rs`, `Runtime::spawn_once()` — flag global no daemon.
- `apps/desktop/src-tauri/src/aria2.rs`, `add_download()` — por download HTTP/FTP.

Os dois overrides foram removidos. Agora o daemon herda o default do próprio binário aria2-ultra (`trunc` pra direto, `none` automático pra BT via lógica própria do fork — não precisa fazer nada aqui pra BT, `add_bittorrent()` ainda passa `none` explícito, redundante mas inofensivo, não foi mexido).

**`connections_per_download` já era 16 por padrão** (`main.rs`, `default_connections()`) antes desta leva de trabalho — isso já estava certo, não precisou de mudança. O usuário achou que o app tava usando só 1 conexão porque testou o `aria2c.exe` **direto por `.cmd`**, sem passar pelo app; isso foi corrigido no lado do fork (aria2-ultra agora também usa 16 conexões por padrão quando invocado sem flags).

### 2. Logs de diagnóstico corrigidos (mesmo commit `0e2d5b2`)
Antes:
- `aria2.runtime_spawned` logava `backend=classic` fixo (era vestígio de antes da troca pro aria2-ultra) e nunca logava a versão real do binário.
- `torrent.engine_selected`/`http.engine_selected` logava `"fileAllocation":"none"` como string literal, sempre, mesmo que o valor real fosse outro.
- Quando um download/torrent falhava (`status=error/removed` no RPC), a mensagem de erro do aria2 só ia pro `DownloadState::Failed` da UI, sem `diagnostic_log`/`state.diagnostics.record` — diferente dos ramos vizinhos de mismatch de tamanho, que já faziam isso. Ou seja, esse motivo de erro só sobrevivia numa bundle de diagnóstico se por acaso estivesse no tail do `aria2.log`.

Depois: `aria2.runtime_spawned` agora chama `endpoint.version()` (RPC `aria2.getVersion`) depois do `wait_ready()` e loga a versão real + `backend=aria2-ultra`; `torrent.engine_selected`/`http.engine_selected` loga `fileAllocation` real (`if is_bittorrent {"none"} else {"trunc"}`); o `match "error"|"removed"` ganhou um `diagnostic_log` com `status`+`reason` antes de remover a task de `state.aria2_tasks`.

**Gap que ainda falta** (identificado, não resolvido): nenhum log distingue torrent v1/v2/híbrido — nenhum campo tipo `metaVersion` do RPC é capturado em lugar nenhum. Então mesmo com os logs corrigidos, provar que o BEP52 (a razão de existir do aria2-ultra) está funcionando certo — ou achar um bug específico de v2/híbrido — não dá só pelos logs, precisa de teste manual com torrent v2/híbrido conhecido, ou instrumentar isso.

### 3. Interceptação do Rapidgator via Shift + declarativeNetRequest (commit `fc303b8`)
Pedido do usuário: deixar o navegador baixar normalmente do Rapidgator (e sites parecidos) por padrão, mas se o usuário segurar Shift, o app deve ser o primeiro e único a "consumir" o link de download de uso único.

O mecanismo de "force" (Shift) via `page-hook.js` **já existia** antes desta sessão (hooks em `<a>.click()`, `fetch()`, `window.open()`) e já funciona pra maioria dos casos. O problema específico do Rapidgator: o link real de download (`s<N>.rapidgator.net/download/<uuid>`) é gerado pelo próprio JS do site e a navegação até ele acontece via `location.href = ...` — que é uma propriedade **"unforgeable"** no Chrome por design de segurança, nenhum script de página consegue interceptar isso, em nenhum navegador Chromium-based. Confirmado analisando um diagnóstico real fornecido pelo usuário (trace mostrando `disposable=true force=true` seguido de `browser_already_owns_response` — ou seja, o Chrome já tinha consumido o link antes do app conseguir agir).

Solução implementada em `browser-extension/background.js`: uma regra `declarativeNetRequest` de sessão (`chrome.declarativeNetRequest.updateSessionRules`), armada só enquanto o gesto de force (Shift) está ativo numa aba do `rapidgator.net` (TTL ≤30s), escopada só àquela aba (`tabIds`), que redireciona a requisição real de volta pra própria página (não gera tela de erro `ERR_BLOCKED`) antes do Chrome completá-la. Em paralelo, um observador `webRequest.onBeforeRequest` não-bloqueante captura a URL real no mesmo instante e entrega pro aria2-ultra via `/v1/download` do bridge, como único requisitante. A regra se autolimpa no primeiro match, ao soltar Shift, com Alt (bypass), ou por timeout. `manifest.json` ganhou a permissão `declarativeNetRequest`.

**⚠️ Não testado contra o site real** — não há navegador/Rapidgator disponível neste ambiente de sandbox, só revisão cuidadosa de código. **Validar manualmente antes de confiar 100%.**

### 4. Auditoria completa feita, sem mais achados além do que já foi corrigido acima
Verificado nesta sessão (por um agente de auditoria dedicado, evidência por arquivo/linha):
- Download/verificação de binário do aria2-ultra: nomes de asset esperados (`aria2c-{windows-x64.exe,linux-x64,macos-x64}` + `.sha256`), lógica de instalação direta (sem extração de arquivo, já que são binários crus, não tarballs) — **tudo bate certo** com o que o aria2-ultra publica/vai publicar.
- Nenhum outro código morto, `#[allow(dead_code)]`, bloco comentado ou TODO/FIXME relevante encontrado em `main.rs`/`aria2.rs`.
- Só resíduo cosmético: nomes de arquivo de teste (`tests/aria2-classic-metadata.test.cjs`, `tests/aria2-runtime-singleton.test.cjs`) e alguns comentários ainda dizem "classic" — não quebra nada, mas seria bom renomear num momento de limpeza.

### 5. Antes disso (commit `36a036e`)
- `aria2_release_repo` (setting em `main.rs`) adicionado, **ainda com default `"FerroDownload/aria2-static-builds"`** — precisa mudar pra `"linuxhell/aria2-ultra"` **depois** que uma Release de verdade for publicada lá (ver handoff do outro repo, passo 2 da lista de pendências). Não decidido ainda se muda só esse default ou também o fetch hardcoded em `.github/workflows/release.yml`/`test-build.yml` deste repo (que hoje ainda apontam pro `FerroDownload/aria2-static-builds` explicitamente na URL da API do GitHub).
- `save_torrent_metadata` (setting), controles de UI pra `.torrent` em `data/torrents` (checkbox "salvar" + botão "limpar"), path já correto via `PathBuf::join` (não precisou de correção, já funcionava certo em qualquer SO).
- Checkbox de "extrair ao terminar" só aparece quando o download é detectado como arquivo compactado (antes aparecia sempre).
- `browser-extension/facebook-media-fix.js` removido por estar órfão (confirmado via grep no repo inteiro).
- `.github/workflows/test-build.yml` criado — **só** o job de build (Windows/Linux tar.gz+AppImage/macOS, x64), sem o job de `release` perigoso que cria tag/publica Release real. Ainda **não foi disparado** (`workflow_dispatch`) — o usuário pediu artefato de teste mas o foco mudou pro trabalho do aria2-ultra antes de rodar.

## O que falta fazer

1. **Testar manualmente a interceptação do Rapidgator** (item 3 acima) contra o site real — não dá pra validar em sandbox.
2. **Esperar a Release do aria2-ultra ser publicada** (ver handoff do outro repo) e então decidir + aplicar: mudar `aria2_release_repo` default, e/ou o fetch hardcoded em `release.yml`/`test-build.yml`.
3. **Disparar `test-build.yml`** via `workflow_dispatch` pra gerar os artefatos de teste pedidos (Windows x64, Linux x64 tar.gz+AppImage, macOS x64) — só depois do item 2, senão o build baixa o binário errado (FerroDownload, não aria2-ultra).
4. **Considerar instrumentar distinção v1/v2/híbrido nos logs** (gap identificado na auditoria) se for preciso diagnosticar um bug específico de BEP52 no futuro.
5. Cosmético, não urgente: renomear os arquivos/comentários que ainda dizem "classic" pra "aria2-ultra".

## ⚠️ Nunca disparar sem confirmação explícita do usuário

`.github/workflows/release.yml` deste repo tem um job `release` **sem guarda de `if:`**, que roda incondicionalmente depois do `build` (inclusive em `workflow_dispatch` manual numa branch), cria uma tag git real, publica/edita uma Release real via `gh release`, e deleta uma tag antiga (`old_tag="v0.4.32"`). **Nunca dispare esse workflow sem o usuário pedir explicitamente uma release de verdade.** Pra artefato de teste, use `test-build.yml`.

## Regras gerais

- Não commitar segredos, binários grandes nem lixo de build.
- Consultar o handoff do `aria2-ultra` antes de assumir qual é o comportamento default do motor — ele muda com frequência nesta fase do projeto.
- Este arquivo deve ser mantido atualizado ou apagado quando ficar obsoleto.
