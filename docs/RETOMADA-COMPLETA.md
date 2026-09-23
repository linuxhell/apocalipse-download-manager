# Apocalipse Download Manager — Retomada Completa

Atualizado em: 2026-09-23
Base pública atual: desktop **v0.4.74** (branch de trabalho `claude/stoic-ramanujan-icyww8`)
Último commit desta branch no momento desta atualização: `cd4d8ce` (aria2 `--log-level=debug` para diagnóstico do bug de metadados de magnet, ver seção dedicada abaixo).

## Objetivo deste arquivo

Este é o ponto único de retomada do desenvolvimento — escrito para que qualquer assistente (ChatGPT, Claude ou outro) consiga continuar o trabalho sem precisar re-investigar o que já foi resolvido. Ele reúne:
- estado atual validado;
- a migração completa de rqbit para aria2 em torrents/magnet, já concluída e em produção;
- as últimas features novas implementadas e testadas;
- um bug ainda **aberto e não resolvido** (metadados de magnet travando mesmo com peers conectados), com todo o histórico de diagnóstico para não repetir hipóteses já descartadas;
- todas as melhorias técnicas propostas para as próximas etapas;
- ordem recomendada de implementação.

---

## Estado atual publicado

Release mais recente conhecida: v0.4.72 (Windows x64 portable, Linux x64 portable TAR.GZ + AppImage, macOS x64 portable, extensões Chrome/Edge/Firefox 0.3.169, SHA256SUMS.txt). A branch de trabalho está em v0.4.74 e ainda não foi lançada como release pública — apenas builds de teste via `validate-portable.yml` (o workflow `release.yml` **nunca** deve ser disparado sem pedido explícito do usuário).

### Ferramentas
A tela Ferramentas possui fluxo portátil de **Procurar / Baixar / Atualizar**:
- FFmpeg;
- yt-dlp;
- QuickJS;
- N_m3u8DL-RE;
- aria2 (agora também usado para torrent/magnet, ver seção seguinte).

Player e Extrator permanecem, por decisão de UX, apenas com Procurar/Baixar (sem Atualizar).

### Extensão / captura
Preservar obrigatoriamente (não regredir):
- autorrecuperação de content script/page-hook em abas já abertas;
- clique normal do ChatGPT entregue ao ADM;
- Insert como atalho fixo de força;
- bypass configurável;
- `capture.layer_healthy`, `capture.layer_main_hook_repaired`, `capture.layer_main_hook_repair_unverified`;
- recuperação de nomes genéricos de mídia;
- SoundCloud usando metadata/título/slug como fallback;
- `browser_download.filename_resolved` e `browser_download.filename_fallback`.

### Downloads nativos (HTTP/HTTPS)
O motor Rust já possui: segmentação, probing HTTP/3 com fallback HTTP, retomada segura por identidade remota, ETag/Last-Modified, journals duplos, escalonamento adaptativo, range stealing, mirror striping pós-identidade, verificação SHA-256, limitação global/por download, DNS customizado, proxy.

---

## Migração completa: rqbit removido, torrent/magnet agora via aria2

**Concluída e em produção nesta branch.** Não é mais um item de avaliação — a decisão foi tomada e implementada por completo.

### O que mudou
- `apps/desktop/src-tauri/src/rqbit.rs` foi **deletado por completo**, junto com todos os comandos Tauri (`get_rqbit_settings`, `set_rqbit_settings`, `test_rqbit`, `regenerate_rqbit_credentials`), campos de `UserSettings`, download automático do binário rqbit (workflows de CI) e toda a UI relacionada (3 idiomas).
- `apps/desktop/src-tauri/src/aria2.rs` passou a lidar com Torrent/Magnet além de HTTP/FTP, com as extensões BitTorrent pedidas:
  - DHT (IPv4 + IPv6), Peer Exchange (PEX), Local Peer Discovery (LPD): flags nativas do aria2, sempre ativas.
  - Fast Extension e UDP tracker: nativos do aria2, sem flag necessária.
  - Multi-Tracker: suportado nativamente via lista de trackers do magnet/torrent.
  - MSE/PSE (criptografia de peer): `--bt-min-crypto-level=arc4` preferindo conexões criptografadas sem recusar peers em texto puro (`--bt-require-crypto=false`).
  - WebSeeding (BEP 19): automático a partir da própria lista `url-list`/`httpseeds` do torrent — sem configuração adicional.
  - Preview do primeiro/último bloco: `--bt-prioritize-piece=head=2M,tail=2M` (somente para downloads reais, não durante o preview de metadados).
  - Sem seeding após concluir: `--seed-time=0 --seed-ratio=0.0` (é um gerenciador de download, não uma seedbox).
- `crates/apocalipse-core/src/strategy.rs`: removida a variante `Engine::RqbitTorrent`; `Torrent`/`Magnet` agora roteiam para `Engine::Aria2Rpc`.
- Handoff de GID: quando o aria2 resolve os metadados de um magnet e (com `follow-torrent=true` global) inicia automaticamente o download do conteúdo real sob um **novo GID**, isso só é detectável pelo campo `followedBy` do `tellStatus`. O poller de downloads reais (`run_aria2_download` em `main.rs`) já trata isso corretamente, trocando de GID em vez de marcar a tarefa como concluída assim que só o metadado termina.
- Testes: `tests/aria2-runtime-singleton.test.cjs` e `tests/download-speed-display.test.cjs` têm asserts explícitos de que **nenhum traço de "rqbit"** sobra no código (`assert.doesNotMatch(main, /rqbit/i)`), além de cobertura de cada extensão BT, do handoff de GID e das correções abaixo.

### Verificação
`grep -rli "rqbit"` em todo o repositório (excluindo `target`/`node_modules`/`.git`) só retorna os próprios testes de guarda que garantem a ausência do termo.

---

## BUG ABERTO — preview de metadados de magnet trava mesmo com peers conectados

**Este é o item mais importante para quem retomar o trabalho.** Não está resolvido. Não repita as hipóteses já descartadas abaixo sem novas evidências.

### Sintoma relatado pelo usuário
Ao adicionar o magnet de "O Resgate do Soldado Ryan 1998" (`magnet:?xt=urn:btih:de4cfc1352d2c797b19c7f400b622af0729502a2&...`), a tela "Nova Tarefa" mostra "Nenhum par respondeu com os metadados deste torrent/magnet a tempo" após o timeout. O comando Tauri `inspect_torrent_metadata` sempre expira (`durationMs` ≈ o timeout configurado).

### Linha do tempo de diagnóstico (evidência real, de bundles enviados pelo usuário)

1. **Antes da migração (rqbit)**: o mesmo magnet resolvia metadados em ~1,1s. Prova de que o swarm tem peers ativos e a rede do usuário não bloqueia BitTorrent.
2. **Primeira tentativa aria2 (`follow-torrent=false`)**: sempre expirava em 90s. Hipótese: `follow-torrent=false` impedia o GID de metadado-apenas de alcançar status `"complete"`.
3. **Fix `follow-torrent=mem` (commit `e38064f`)**: aplicado por ser o valor documentado do aria2 para modo metadata-only. **Não resolveu** — mesmo magnet, mesmo build, continuou expirando (confirmado por bundle de diagnóstico subsequente).
4. **Diagnóstico com visibilidade de peers adicionada (commit `32cf1f5`)**: adicionamos ao `tellStatus` os campos `connections`/`numSeeders` e log periódico. Resultado real observado: **35 conexões de peers e 1 seeder confirmados**, ainda assim expirou os 150s (timeout já estendido de 90s→150s nesse commit). Isso **descarta definitivamente** a hipótese de rede/firewall bloqueando DHT/tracker — os peers estão alcançáveis.
5. **Hipótese `followedBy` (commit `0623eed`)**: suspeitamos que o aria2 estivesse "seguindo" silenciosamente para um download de conteúdo real (ignorando `bt-metadata-only=true`) por causa do `follow-torrent=true` global sobrepondo o override por request. Implementamos detecção de `followedBy` durante o polling do preview, tratando seu aparecimento como prova de que os metadados já foram resolvidos, e removendo (`forceRemove`) tanto o GID de metadado quanto o de conteúdo para não vazar um download em segundo plano.
6. **Bundle de diagnóstico seguinte (build `0623eed`, com a correção do item 5 já ativa)**: **nenhum evento `followedBy` ocorreu**. A hipótese do item 5 não é a causa (ao menos não nesta captura). Porém revelou um dado novo e mais preciso:
   - `totalLength` chegou a **14489 bytes** — ou seja, o aria2 recebeu de pelo menos um peer o handshake estendido (BEP 10) com o `metadata_size` do BEP 9. O tamanho é conhecido.
   - `completedLength` **ficou em 0 o tempo inteiro**, em **duas tentativas completas de 150s**, com até **40 conexões e 1 seeder**.
   - Conclusão: peers alcançáveis ✅, tamanho do metadado conhecido ✅, mas a peça do metadado em si (BEP 9 é tipicamente uma única peça de até 16 KiB — aqui caberia numa só) **nunca é entregue**. O travamento é dentro da própria troca `ut_metadata`, não em descoberta de peers nem em roteamento de GID.

### Estado atual do código (branch `claude/stoic-ramanujan-icyww8`, commit `cd4d8ce`)
- `apps/desktop/src-tauri/src/aria2.rs::preview_magnet_metadata`: timeout de 150s; opções `bt-metadata-only=true`, `bt-save-metadata=false`, `follow-torrent=mem`; polling de `status`, `errorMessage`, `bittorrent`, `files`, `connections`, `numSeeders`, `totalLength`, `completedLength`, `followedBy`; callback de progresso a cada 5s (ou imediatamente se `followedBy` aparecer) repassado para `diagnostic_log` em `main.rs` como `aria2.metadata_preview_progress` (ou `aria2.metadata_preview_followed_unexpectedly` se o followedBy ocorrer); em timeout, o erro carrega `connections`/`seeders` de pico (`aria2_metadata_timeout:connections=N:seeders=M`).
- `--log-level` do processo aria2 foi elevado de `notice` → `info` → **`debug`** (commit `cd4d8ce`, ainda não confirmado em build) especificamente para capturar o protocolo BitTorrent peça a peça (handshake estendido, requisições/respostas `ut_metadata`) no próximo bundle de diagnóstico, já que `info` não tem essas linhas.
- UI (`app.js`): mensagem diferenciada quando `connections=0:seeders=0` (provável firewall/VPN) vs. timeout genérico (peers presentes, mas sem seeds/lento). **Esta mensagem "sem peers" não é o caso atual** — o caso real tem peers e precisa de uma terceira mensagem/diagnóstico mais preciso quando `completedLength` fica travado em 0 com `totalLength` conhecido (ainda não implementada; é um bom próximo passo de UX quando a causa raiz for confirmada).

### Hipóteses já testadas e descartadas (não repetir sem evidência nova)
- ❌ Rede/firewall bloqueando UDP/DHT — descartado pelo item 4 (peers conectados).
- ❌ `follow-torrent=false` vs `mem` sendo a causa — ambos os valores produzem o mesmo sintoma de timeout; a diferença não muda o resultado observado.
- ❌ DHT frio (sem tabela de roteamento cacheada) precisando de mais tempo — descartado: mesmo com conexões estabelecidas rapidamente (peak_connections crescendo nos primeiros 20-25s) e size conhecido, o travamento persiste pelos 150s inteiros, não é questão de esperar mais bootstrap.
- ❌ `followedBy` (aria2 seguindo para download de conteúdo sem avisar) — corrigido no código, mas o bundle mais recente não mostrou esse evento ocorrendo, então não era a causa desta captura específica (pode ainda ser uma causa válida em outros cenários; a correção foi mantida por segurança/vazamento de download).
- ❌ `bt-prioritize-piece=head=2M,tail=2M` interferindo no preview — na verdade essa opção só é setada em `add_bittorrent` (downloads reais), nunca em `preview_magnet_metadata`; não se aplica a este caso.

### Hipóteses ainda não testadas (candidatas para a próxima rodada)
- A negociação de criptografia (`bt-min-crypto-level=arc4` + `bt-require-crypto=false`) pode estar interagindo mal especificamente com a extensão `ut_metadata` para o único peer que anunciou o `metadata_size` — vale tentar remover a preferência de criptografia (deixar plano) só para o preview e comparar.
- O(s) peer(s) alcançados podem ser entradas "mortas"/obsoletas na DHT (torrent antigo, trackers majoritariamente mortos como rarbg/glotorrents) que completam o handshake estendido mas nunca respondem à requisição de peça do metadado — comum em swarms decadentes. Os logs em `debug` (commit `cd4d8ce`) devem provar ou refutar isso mostrando se a requisição de peça é sequer enviada e se algum peer responde com `reject`.
- Perguntar ao usuário se **outros** magnets/torrents (não só este specimen antigo) também travam da mesma forma — decisivo para saber se é um bug sistêmico do nosso código ou uma característica deste swarm específico.

### Próximo passo imediato
1. Aguardar o usuário testar novamente com o build do commit `cd4d8ce` (log em `debug`) e enviar um novo bundle de diagnóstico.
2. Ler `engines/aria2-runtime.log` no bundle procurando linhas de `[DEBUG]` relacionadas a `ut_metadata`, extended handshake e rejeições de peça.
3. Confirmar com o usuário se o problema é específico deste magnet ou generalizado.
4. Só então decidir entre: ajustar a preferência de criptografia, tentar um segundo `addUri` paralelo após N segundos sem progresso (novo peer via DHT), ou aceitar que é uma limitação do swarm específico e melhorar apenas a mensagem de erro para o usuário.

---

## Novidades implementadas nesta rodada (branch `claude/stoic-ramanujan-icyww8`)

Das 5 ideias de diferenciação frente a outros gerenciadores de download levantadas nesta sessão, 4 foram implementadas, testadas e commitadas; a 5ª foi deliberadamente adiada.

1. **"Por que está lento?" (explicação inline)** — link expansível em cada tarefa em andamento que roda `apocalipse-ai-core.js::performanceDiagnosis()` filtrado pela tarefa específica (`taskId`), explicando em linguagem natural o motivo da velocidade atual.
2. **"Localizar arquivo..." (retomar de novo caminho)** — novo comando Tauri `relocate_download`; permite apontar uma tarefa pausada/falha para um arquivo parcial movido manualmente para outra pasta/drive, sem re-baixar do zero. Não move o arquivo — confia na validação de identidade remota já existente do motor para retomar com segurança.
3. **Visibilidade de mirrors HTTP (WebSeeding)** — a UI agora mostra `+N mirror HTTP` ao lado das estatísticas de seeders/leechers quando o torrent usa WebSeeding (BEP 19) e o aria2 reporta URIs de arquivo com `status="used"` e esquema http(s).
4. **Apocalipse Link como fonte alternativa (versão simples)** — quando uma tarefa pausada/falha tem um arquivo com nome correspondente disponível via Apocalipse Link (LAN), a UI oferece preencher a tarefa copiando o arquivo local em vez de baixar da internet. Implementado tanto na página Link embutida (`app.js`) quanto na janela popup separada (`link.js`), já que a UI do Link existe duplicada nos dois lugares.
5. **(Adiada) Repositório comunitário de correções de site** — feature descartada por enquanto por risco real de segurança/confiança: aplicar automaticamente regras buscadas da internet é um vetor de supply-chain injection. Precisa de desenho de assinatura/curadoria antes de ser retomada.

Durante essa rodada também foram corrigidos dois bugs reais no Apocalipse AI (chat), encontrados a partir de screenshot do usuário:
- Perguntas de esclarecimento puramente contextuais ("e o que é isso?") não reexplicavam a última resposta do próprio assistente — corrigido com `clarifyPreviousAnswer()`.
- O placeholder interno `"<redacted-sensitive-line>"` (usado por `sanitize_log_detail()` para censurar linhas de log com aparência de segredo) estava sendo ecoado literalmente ao usuário em vez de traduzido para uma explicação — corrigido em `safeDetail()`.

Um bug de HTML inválido (`<button>` aninhado dentro de `<button>`) foi encontrado e corrigido proativamente durante a implementação do item 4, trocando as linhas de arquivo do Link de `<button>` para `<div role="button" tabindex="0">` com navegação por teclado manual, em ambos `app.js` e `link.js`.

---

## Hotfix histórico (já resolvido) — processos aria2 duplicados

Mantido apenas como referência; não regredir.

- Causa 1: `aria2c --version` era chamado ao abrir Ferramentas mesmo com o daemon RPC já ativo.
- Causa 2: aria2 órfão após encerramento anormal (processo filho não ligado à vida do PID pai).
- Correção: `--stop-with-process=<PID do Apocalipse>`; Ferramentas consulta a versão via RPC quando o runtime está vivo, ou apenas verifica a existência do executável quando parado; logs `aria2.runtime_spawned`/`aria2.runtime_stopped` com PID/porta.

---

## ISO e retomada completa

ISO não precisa de motor separado — usa a mesma infraestrutura de retomada segura dos demais arquivos grandes (destino/parcial, journals duplos, journals segmentados, chunks por destino, offsets `u64` acima de 4 GiB, retomada só com identidade remota + suporte a range compatíveis, verificação final quando hash remoto disponível). Investigado nesta sessão a oscilação de velocidade do ISO da Microsoft: **não é bug** — o CDN/URL assinada não suporta múltiplas conexões, o aria2 corretamente cai para 1 conexão, e throughput de fluxo único naturalmente tem mais variância.

Regressões a manter: ISO de 8 GiB+, nome parcial preservando `.iso.part`, journals duplos, diretórios de chunks sem colisão, limpeza exata, restart/crash + retomada, servidor trocando ETag entre tentativas, servidor que deixa de aceitar Range, download concluído sem journals/chunks órfãos.

---

# Melhorias propostas — ordem técnica

## P1 — Cache de capacidade de transporte + HTTP Happy Eyeballs
Cache versionado por origem com TTL (HTTP/3, HTTP/2, HTTP/1.1, suporte a Range, ETag/Last-Modified estáveis, RTT, first-byte latency, goodput sustentado, faixa útil de concorrência, taxa de erro transitório). Para HTTPS: iniciar HTTP/3, após pequeno atraso iniciar caminho HTTP normal, manter o primeiro caminho saudável, cachear temporariamente, reprovar periodicamente.

## P2 — Scheduler adaptativo v2 baseado em streams
Distinguir conexões TCP HTTP/1.1, streams multiplexados HTTP/2 e streams QUIC HTTP/3. Decisões por ganho marginal de goodput, RTT, erros, pressão de escrita em disco, limites globais, fairness por origem. Não tratar "16 workers" como "16 conexões" em HTTP/2/3.

## P3 — Banco de tarefas durável SQLite/WAL
Migrar estado operacional de `queue.json` para SQLite WAL. Tabelas sugeridas: tasks, attempts, sources/mirrors, transfer_checkpoints, media_metadata, engine_decisions, history, diagnostic_refs. Manter JSON para import/export portátil.

## P4 — Supply chain assinada para ferramentas
Manifests assinados (tool id, versão, plataforma/arquitetura, URL, tamanho exato, SHA-256, expiração, sequência monotônica) + verificação obrigatória + rollback de um clique. O probe de versão continua apenas como validação secundária.

## P5 — Camada unificada de estratégia de mídia
Roteamento automático: arquivo direto capturado → motor Rust; página/extrator → yt-dlp; HLS/DASH/MSS → N_m3u8DL-RE quando adequado; áudio/vídeo separados → aquisição paralela + FFmpeg remux; live/URL efêmera → Gravações; fallback só após classificar a falha. Mostrar na UI motor escolhido, motivo e fallback usado.

## P6 — Download Context Envelope extensão → desktop
Estrutura única versionada (resource URL, page URL, método, headers sanitizados, content type/length, filename sugerido + origem, media title, referer/origin scope, timestamp, indício de URL expirável, escopo de auth, relação áudio/vídeo, trace id). Sem senha bruta/segredo de longa duração. Mesmo envelope em clique normal, popup, requests capturadas e downloads assistidos.

## P7 — Scoring e rebalanceamento de mirrors
Só após comprovar conteúdo idêntico: medir first-byte latency, goodput, falhas/timeouts, desempenho por faixa. Reatribuir ranges em tempo real quando um mirror degradar. Persistir score de curto prazo por origem.

## P8 — Integridade forte de arquivo parcial
Manter SHA-256 final para interoperabilidade. Adicionar hash interno rápido por chunk (ex.: BLAKE3) para validar chunks já gravados após crash sem rehash completo, e detectar corrupção localizada. Não substituir hash/assinatura publicada pelo fornecedor.

## P9 — Avaliação de motor torrent moderno (atualizado)
**Decisão já tomada e implementada**: aria2 é o motor de torrent/magnet em produção (ver seção de migração acima), com DHT/PEX/LPD/MSE-PSE/WebSeeding/multi-tracker/UDP-tracker habilitados. Um protótipo lado a lado com libtorrent ainda pode valer a pena no futuro especificamente para comparar tempo de resolução de metadata magnet (dado o bug aberto acima) e fast resume, mas não é mais uma decisão pendente de arquitetura — é uma investigação de performance/confiabilidade pontual.

## P10 — Modularização do backend desktop
Reduzir responsabilidades de `main.rs`, separando em: task_manager, transfer_router, browser_bridge, media_pipeline, tools_manager, torrent_manager, archive_manager, apocalipse_link, credentials, diagnostics, persistence. Tauri commands devem ficar finos. (`main.rs` já passa de 9000 linhas nesta branch — prioridade crescente.)

---

# Melhorias competitivas de mídia

## Autenticação / cookies
Fonte de cookies configurável, browser/cookies.txt, health check por site, fallback controlado, mensagens claras de cookie expirado/login necessário.

## Erros e retry
Taxonomia estável de erros, retry automático com backoff, estado "retry scheduled", diferenciar auth/rate limit/URL expirada/DRM/extractor bug/rede/arquivo removido/geo restriction.

## Subtítulos e metadados
Subtitles, automatic captions, thumbnail embedding, metadata, chapters, container MP4/MKV/WebM/original, templates de nome, intervalos de tempo.

## Playlist / canal
Inspecionar playlist/canal, selecionar filhos, criar child tasks, limites de quantidade/data.

## Twitch
UX de VOD/live, reconnect, política de chat/subtitle, health de autenticação.

## Bilibili
Health de SESSDATA/DedeUserID, aliases de AI subtitles, danmaku, fallback de formato premium, playlist/canal.

## Biblioteca e automação
RSS/subscriptions, importação de mídia local, histórico pesquisável, transcrição local, seleção GPU-aware de ASR, diarização opcional, IA opcional sobre transcrições.

---

# Distribuição e arquitetura futuras

- WebDAV nativo;
- Windows ARM64, Linux ARM64, Apple Silicon nativo (nenhum planejado no curto prazo — não anunciar no README até haver build validado);
- instaladores opcionais além do modo portátil;
- relay criptografado e retomável para Apocalipse Link;
- updates de regras Matrix assinados;
- benchmark reproduzível com latência/perda de pacote;
- soak tests de arquivos muito grandes;
- fuzzing de parsers/metadata;
- acessibilidade;
- localização completa de todas as telas.

---

# O que não fazer agora

- Não substituir o motor HTTP Rust por aria2.
- Não criar outro parser próprio de HLS/DASH antes de esgotar yt-dlp/N_m3u8DL-RE.
- Não aumentar o número de regras específicas de site antes do Download Context Envelope e da taxonomia genérica de falhas.
- Não anunciar "mais rápido que X" sem benchmark reproduzível + hashes de saída.
- Não disparar `release.yml` sem pedido explícito do usuário — apenas `validate-portable.yml` para builds de teste.
- Não reintroduzir menção a builds ARM/Apple Silicon no README enquanto não existir build validado.

---

# Sequência recomendada

## Hotfix / confiabilidade imediata
1. **Resolver o bug aberto de metadados de magnet** (ver seção dedicada acima) — prioridade máxima no momento.
2. validar retomada de ISO grande;
3. soak test de downloads de muitas horas;
4. garantir limpeza de journals/processos após crash e restart.

## Milestone A
1. cache de capacidade de transporte;
2. HTTP Happy Eyeballs;
3. manifests assinados de ferramentas + rollback;
4. Download Context Envelope.

## Milestone B
1. SQLite/WAL;
2. scheduler adaptativo v2;
3. source scoring/rebalanceamento;
4. hash interno por chunk.

## Milestone C
1. controles ricos de subtitle/metadata/container;
2. protótipo libtorrent (comparação de resolução de metadata magnet);
3. ARM64/Apple Silicon.

## Milestone D
1. playlists/canais;
2. RSS/subscriptions;
3. biblioteca local;
4. transcrição/IA opcional.

---

## Critério permanente

Priorizar nesta ordem:
1. confiabilidade;
2. retomada sem corrupção;
3. observabilidade;
4. segurança da cadeia de ferramentas;
5. velocidade medida;
6. novas features.

O ADM deve continuar portátil, leve e simples para o usuário final, mesmo quando a arquitetura interna evoluir.
