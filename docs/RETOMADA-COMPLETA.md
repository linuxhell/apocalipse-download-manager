# Apocalipse Download Manager — Retomada Completa

Atualizado em: 2026-09-22  
Base pública atual: desktop **v0.4.72** / extensão **0.3.169**

## Objetivo deste arquivo

Este é o ponto único de retomada do desenvolvimento. Ele reúne:
- estado atual validado;
- correções recentes que não devem regredir;
- diagnóstico e correção do aria2 duplicado;
- suporte explícito a ISO e retomada de arquivos grandes;
- todas as melhorias técnicas propostas para as próximas etapas;
- ordem recomendada de implementação.

---

## Estado atual publicado

Release v0.4.72:
- Windows x64 portable;
- Linux x64 portable TAR.GZ;
- Linux x64 AppImage;
- macOS x64 portable;
- Chrome 0.3.169;
- Edge 0.3.169;
- Firefox 0.3.169 assinado;
- SHA256SUMS.txt.

### Ferramentas
A tela Ferramentas possui fluxo portátil de **Procurar / Baixar / Atualizar**:
- FFmpeg;
- yt-dlp;
- QuickJS;
- N_m3u8DL-RE;
- aria2.

Player e Extrator permanecem, por decisão de UX, apenas com:
- Procurar;
- Baixar.

Sem botão Atualizar para Player/Extrator.

Downloads automáticos usam a pasta portátil `tools`, ajustam o campo de caminho e só persistem o caminho após **Salvar**. A janela de Ferramentas foi ampliada para não cortar botões/caminhos.

Plataformas:
- Windows: mpv + 7-Zip;
- Linux: mpv AppImage + 7zz;
- macOS: mpv + 7zz.

### Extensão / captura
Preservar obrigatoriamente:
- autorrecuperação de content script/page-hook em abas já abertas;
- clique normal do ChatGPT entregue ao ADM;
- Insert como atalho fixo de força;
- bypass configurável;
- `capture.layer_healthy`;
- `capture.layer_main_hook_repaired` somente após resposta verificada;
- `capture.layer_main_hook_repair_unverified` quando o reparo não puder ser confirmado;
- recuperação de nomes genéricos de mídia;
- SoundCloud usando metadata/título/slug como fallback;
- `browser_download.filename_resolved` e `browser_download.filename_fallback`.

### Downloads nativos
O motor Rust já possui:
- HTTP/HTTPS segmentado;
- probing HTTP/3 com fallback HTTP;
- retomada segura baseada em identidade remota;
- ETag / Last-Modified;
- journals duplos para retomada;
- escalonamento adaptativo;
- range stealing;
- mirror striping somente após identidade comprovada;
- descoberta/verificação SHA-256;
- limitação global e por download;
- DNS customizado;
- proxy.

---

## Hotfix prioritário — processos aria2 duplicados

### Causas encontradas

Há dois casos diferentes:

1. **Processo curto de versão ao abrir Ferramentas**  
   O status de ferramentas executava `aria2c --version`. Se o daemon RPC já estivesse ativo, o sistema mostrava temporariamente dois processos aria2.

2. **aria2 órfão após encerramento anormal do ADM**  
   O runtime era morto no `Drop` durante encerramento normal, mas o processo filho não estava ligado à vida do PID pai. Se o ADM fosse finalizado à força, travasse ou fosse substituído durante atualização, o aria2 podia continuar vivo. Na próxima inicialização o ADM escolhia outra porta livre e criava um novo aria2.

### Correção definida

- iniciar o daemon com `--stop-with-process=<PID do Apocalipse>`;
- abrir Ferramentas não executa `aria2c --version`;
- se o runtime RPC estiver vivo, a versão é obtida pelo próprio RPC;
- se estiver parado, Ferramentas apenas verifica se o executável existe;
- registrar:
  - `aria2.runtime_spawned` com PID, PID pai e porta;
  - `aria2.runtime_stopped` com PID e porta.

### Próximo reforço opcional
Se ainda houver evidência de órfãos antigos:
- persistir `pid/port/runtime-id` do daemon;
- no boot, validar o runtime anterior pelo RPC com o segredo do ADM;
- encerrar/reaproveitar apenas o daemon comprovadamente pertencente ao ADM;
- nunca matar um aria2 externo do usuário apenas pelo nome do processo.

---

## ISO e retomada completa

### Estado
ISO não precisa de um motor separado. O motor nativo de HTTP é neutro à extensão.

Arquivos `.iso` devem usar exatamente a mesma infraestrutura de retomada segura dos demais arquivos grandes:
- destino `arquivo.iso`;
- parcial `arquivo.iso.part`;
- journals duplos `resume-a.json` / `resume-b.json`;
- journals segmentados `segments-a.json` / `segments-b.json`;
- chunks em diretório exclusivo por destino;
- offsets e tamanhos em `u64`, inclusive acima de 4 GiB;
- retomada somente quando identidade remota e suporte a range forem compatíveis;
- verificação final de integridade quando hash remoto estiver disponível.

### Regressões obrigatórias
Manter testes para:
- ISO de 8 GiB ou maior;
- nome parcial preservando `.iso.part`;
- journals duplos;
- diretórios de chunks sem colisão;
- limpeza exata sem apagar arquivos parecidos;
- restart/crash + retomada;
- servidor que muda ETag entre tentativas;
- servidor que deixa de aceitar Range;
- download concluído sem deixar journals/chunks órfãos.

### UX futura
Para ISOs grandes:
- mostrar claramente “retomada segura disponível”;
- mostrar hash esperado/verificado quando existir;
- opção “Verificar integridade”;
- manter o arquivo parcial após erro recuperável;
- evitar pré-alocação cara por padrão.

---

# Melhorias propostas — ordem técnica

## P1 — Cache de capacidade de transporte + HTTP Happy Eyeballs

Criar cache versionado por origem com TTL:
- HTTP/3 / HTTP/2 / HTTP/1.1;
- suporte a Range;
- ETag/Last-Modified estáveis;
- RTT;
- first-byte latency;
- goodput sustentado;
- faixa útil de concorrência;
- taxa de erro transitório.

Para HTTPS:
- iniciar HTTP/3;
- após pequeno atraso, iniciar caminho HTTP normal;
- manter o primeiro caminho saudável;
- cachear temporariamente;
- reprovar periodicamente.

Objetivo: início mais rápido e menos espera quando QUIC estiver bloqueado/intermitente.

## P2 — Scheduler adaptativo v2 baseado em streams

Distinguir:
- conexões TCP HTTP/1.1;
- streams multiplexados HTTP/2;
- streams QUIC HTTP/3.

Decisões por:
- ganho marginal de goodput;
- RTT;
- erros;
- pressão de escrita em disco;
- limites globais;
- fairness por origem.

Não tratar “16 workers” como “16 conexões” em HTTP/2/3.

## P3 — Banco de tarefas durável SQLite/WAL

Migrar estado operacional de `queue.json` para SQLite WAL.

Tabelas sugeridas:
- tasks;
- attempts;
- sources/mirrors;
- transfer_checkpoints;
- media_metadata;
- engine_decisions;
- history;
- diagnostic_refs.

Manter JSON para import/export portátil.

Benefícios:
- transições atômicas;
- recuperação melhor após crash;
- histórico seguro;
- atualizações frequentes baratas;
- base para RSS/playlist/subscriptions.

## P4 — Supply chain assinada para ferramentas

O download “Baixar” de Ferramentas deve evoluir para manifests assinados contendo:
- tool id;
- versão;
- plataforma/arquitetura;
- URL;
- tamanho exato;
- SHA-256;
- expiração;
- sequência monotônica.

Adicionar:
- verificação obrigatória;
- rollback de um clique para binário anterior verificado.

O probe de versão continua apenas como validação secundária.

## P5 — Camada unificada de estratégia de mídia

Roteamento automático recomendado:
1. arquivo direto capturado pelo navegador -> motor Rust;
2. página/extrator -> yt-dlp;
3. HLS/DASH/MSS -> N_m3u8DL-RE quando mais adequado;
4. áudio/vídeo separados -> aquisição paralela + FFmpeg remux;
5. live/URL efêmera -> Gravações;
6. fallback somente após classificar a falha.

FFmpeg deve ser principalmente verificador/remuxer/transcoder, não downloader de rede padrão.

Mostrar na UI:
- motor escolhido;
- motivo da escolha;
- fallback utilizado.

## P6 — Download Context Envelope extensão -> desktop

Criar estrutura única e versionada contendo:
- resource URL;
- page URL;
- método HTTP;
- headers sanitizados;
- content type;
- content length;
- suggested filename;
- filename source;
- media title;
- referer/origin scope;
- capture timestamp;
- indício de URL expiráveI;
- identificador de escopo de autenticação;
- relação áudio/vídeo;
- trace id.

Não transportar senha bruta ou segredo de longa duração.

Usar o mesmo envelope em:
- clique normal;
- popup;
- requests capturadas;
- downloads assistidos pelo navegador.

## P7 — Scoring e rebalanceamento de mirrors

Somente após comprovar conteúdo idêntico:
- medir first-byte latency;
- goodput;
- falhas/timeouts;
- desempenho por faixa.

Reatribuir ranges em tempo real quando um mirror degradar.

Persistir score de curto prazo por origem para ordenar melhor o próximo download.

## P8 — Integridade forte de arquivo parcial

Manter SHA-256 final para interoperabilidade.

Adicionar hash interno rápido por chunk, por exemplo BLAKE3:
- validar chunks já gravados após crash;
- evitar rehash completo do parcial a cada restart;
- detectar corrupção localizada.

Não substituir hash/assinatura publicada pelo fornecedor.

## P9 — Avaliação de motor torrent moderno

Manter aria2 em produção por enquanto.

Criar protótipo lado a lado com libtorrent e medir:
- tempo de metadata magnet;
- fast resume;
- prioridade por peça/arquivo;
- preview/streaming;
- uTP;
- DHT/PEX;
- I/O;
- RAM;
- tamanho do pacote;
- portabilidade.

Migrar somente se o ganho medido compensar o peso/dependência.

## P10 — Modularização do backend desktop

Reduzir responsabilidades de `main.rs`.

Separar em:
- task_manager;
- transfer_router;
- browser_bridge;
- media_pipeline;
- tools_manager;
- torrent_manager;
- archive_manager;
- apocalipse_link;
- credentials;
- diagnostics;
- persistence.

Tauri commands devem ficar finos.

---

# Melhorias competitivas de mídia

## Autenticação / cookies
- fonte de cookies configurável;
- browser/cookies.txt;
- health check por site;
- fallback controlado;
- mensagens claras de cookie expirado/login necessário.

## Erros e retry
- taxonomia estável de erros;
- retry automático com backoff;
- estado “retry scheduled”;
- diferenciar:
  - auth;
  - rate limit;
  - URL expirada;
  - DRM;
  - extractor bug;
  - rede;
  - arquivo removido;
  - geo restriction.

## Subtítulos e metadados
- subtitles;
- automatic captions;
- thumbnail embedding;
- metadata;
- chapters;
- container MP4/MKV/WebM/original;
- templates de nome;
- intervalos de tempo.

## Playlist / canal
- inspecionar playlist/canal;
- selecionar filhos;
- criar child tasks;
- limites de quantidade/data.

## Twitch
- UX de VOD/live;
- reconnect;
- política de chat/subtitle;
- health de autenticação.

## Bilibili
- health de SESSDATA/DedeUserID;
- aliases de AI subtitles;
- danmaku;
- fallback de formato premium;
- playlist/canal.

## Biblioteca e automação
- RSS/subscriptions;
- importação de mídia local;
- histórico pesquisável;
- transcrição local;
- seleção GPU-aware de ASR;
- diarização opcional;
- IA opcional sobre transcrições.

---

# Distribuição e arquitetura futuras

- WebDAV nativo;
- Windows ARM64;
- Linux ARM64;
- Apple Silicon nativo;
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
- Não anunciar “mais rápido que X” sem benchmark reproduzível + hashes de saída.

---

# Sequência recomendada

## Hotfix / confiabilidade imediata
1. eliminar processos aria2 duplicados;
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
1. estratégia unificada de mídia;
2. controles ricos de subtitle/metadata/container;
3. protótipo libtorrent;
4. ARM64.

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
