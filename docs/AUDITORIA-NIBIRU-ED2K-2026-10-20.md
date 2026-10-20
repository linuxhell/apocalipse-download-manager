# Auditoria ADM, motores Nibiru e Apocalipse ED2K — 20/10/2026

## Resultado e alcance real

Esta continuação recuperou o checkpoint anterior, o código completo do Colapse
em `145e88bd027a9f25aa54fb55d4120aceae0ba7f8` e os quatro motores. Corrigiu
contratos incompatíveis que impediam Power e Colapse de funcionar pelo ADM,
regressões de cancelamento/publicação, preferências persistidas e atualização
em caminho personalizado. A sessão Kad ganhou HELLO com desafio de endpoint,
tratamento de pacotes comprimidos e coleta limitada de fontes em vários lotes.

Os novos commits dos motores foram preparados localmente; o manifesto de build
fixa essas revisões. Sua publicação no GitHub continua pendente de autorização
explícita após uma rejeição da revisão automática. Não houve merge nem release.
O pacote de teste e sua evidência de build identificam a revisão exata do ADM.
Os resultados desta etapa estão discriminados abaixo; os números do checkpoint
anterior não são apresentados como testes novos.

O Apocalipse ED2K avançou de bootstrap Kad isolado para sessão UDP persistente
com roteamento limitado e consulta de fontes diretas. **Não está concluído como
substituto do aMule.** Os testes locais não demonstram interoperabilidade com
clientes públicos, alcance de portas/NAT ou operação contínua na rede pública.

A auditoria semântica se concentrou em seleção de motores, inicialização/RPC,
retomada, integridade, subprocessos, instalação/empacotamento, ED2K/Kad e suas
superfícies de interface. Foram inventariados o backend e a interface, mas
**a leitura semântica linha por linha de todo o ADM ainda não foi concluída**.
Inventário, busca textual e aprovação de testes não são equivalentes a essa
leitura. Este relatório registra um checkpoint verificável, sem certificar o
repositório inteiro como auditado.

## Revisões dos motores

| Motor | Revisão de fonte | Situação |
| --- | --- | --- |
| Xtreme | `0a9fcdde7363aed6881c08f57e157564a0bebd04` | RPC nativo `nibiru.*`; correção do Clippy e preservação da identidade da tarefa |
| Power | `55e10437298111cddc1e92fca699db8afbb65806` | Envelope JSON com identidade; extração, download e HLS testados |
| M3U8 | `803cab28a64fdfbb2ae667ab987ce1951f8b0717` | Isolamento das fixtures concorrentes; produção igual a `b84f502…` |
| Colapse | `712c83af47695b44ae9afe430327e236da52d3f1` | Fonte recuperada; sete operações reais verificadas pelo adaptador ADM |

As revisões completas estão fixadas em `scripts/nibiru-engines.json`; Colapse
passou a ser obrigatório. O cliente ED2K embutido usa a mesma revisão de Xtreme
no Cargo.lock. Os commits novos ainda não estão disponíveis no GitHub: antes
de executar CI a partir do manifesto, é necessário publicar as branches dos
motores. O script aceita `--source-root` para compilar revisões idênticas a
partir de repositórios Git locais e preserva também `packaging/licenses`.
O workspace fixa Tokio 1.54.0 ou superior, com o lock em 1.54.0, para corrigir a
ordem de descarte da espera de processos no Windows.

## Correções herdadas do checkpoint e preservadas

| Prioridade | Problema anterior | Correção e evidência |
| --- | --- | --- |
| Alta | O ADM ainda iniciava um daemon com parâmetros e métodos antigos | `xtreme.rs` usa `serve` tipado e namespace `nibiru.*`; teste inicia o processo real, baixa e remove o resultado |
| Alta | Retomada HTTP podia combinar conteúdos diferentes ou confiar em arquivo arbitrário | Origem/ETag forte ou Last-Modified persistidos, If-Range e validação de intervalos; testes de mudança de versão e HTTP 416 |
| Alta | Pausa/retomada rápida podia deixar gerações gravando juntas | Exclusão por tarefa e identidade da geração; teste real verifica bytes e contadores finais |
| Alta | M3U8 podia sinalizar sucesso sem produzir um arquivo final com o contêiner pedido | JSON exige resultado completo, tamanho real e saída não vazia; ausência de FFmpeg falha antes de remux |
| Alta | Fingerprint de retomada M3U8 não descrevia suficientemente os segmentos | SHA-256 da descrição da mídia/segmentos e identidade das chaves; checkpoint live retém lacunas |
| Alta | Processos externos podiam bloquear pipes ou continuar após pausa | Leitura concorrente limitada, timeout, cancelamento e encerramento da árvore/grupo; testes do módulo real |
| Alta | Copiar a saída M3U8 com hard-link direto falhava entre unidades | Arquivo temporário no volume de destino, sync e publicação sem substituir arquivo existente |
| Alta | Manifestos/segmentos e downloads de ferramentas não tinham todos os limites explícitos | Tetos de 8 MiB/64 MiB no M3U8 e 512 MiB no download de ferramenta; validação de tamanho/Content-Range |
| Média | Instalação podia aceitar binário antigo com `--version` válido | SHA-256, negociação de identidade/schema/capacidade antes da substituição do binário |
| Média | Atualização do Xtreme ficava bloqueada depois de qualquer uso | Bloqueia workers ativos; encerra o runtime ocioso antes da instalação |
| Média | Build portable continha caminho antigo de backend | Os três workflows usam o conjunto de fontes fixadas e copiam a árvore dos motores |
| Média | MPD e o plano de HLS ainda mostravam o motor antigo | Classificação DASH e estratégia `NibiruM3u8`, preservando aliases de leitura de estado antigo |
| Média | Kad era somente uma consulta bootstrap | Sessão persistente, contatos verificados, consultas iterativas e fontes diretas integradas à fila |

## Restrições que permanecem nos motores

- Power não equivale ao catálogo completo de sites do yt-dlp. O ADM usa o caminho
  nativo para fontes suportadas sem contexto especial; fluxos com credenciais,
  proxy ou opções não suportadas continuam na rota existente.
- M3U8 precisa de FFmpeg para remux e faixas separadas. O ADM rejeita limitação de
  banda não implementada e cabeçalhos Cookie/Authorization sem escopo de host.
  Isso evita sinalizar cumprimento de uma configuração ignorada ou enviar essas
  credenciais a todos os hosts citados pelo manifesto. A retomada VOD reutiliza
  os segmentos verificados no cache; uma gravação live com lacunas não é concluída.
- Xtreme mantém tarefas RPC em memória. O ADM detecta GIDs obsoletos e recria a
  transferência com os dados de retomada; não existe restauração automática da
  tabela RPC pelo motor.
- Colapse executa inspeção, WAV, Y4M e MP4 faststart em arquivos locais. As sete
  operações do adaptador foram exercitadas com o binário real e saídas
  verificadas. Isso não implementa decodificação de codecs comprimidos,
  transcodificação de vídeo, mux de faixas MP4 ou equivalência ao FFmpeg.
  O snapshot recuperado não inclui RF64/WAVE_FORMAT_EXTENSIBLE da retomada
  posterior que não foi recuperada. Power e Colapse preservam os avisos Rust
  existentes; nenhuma licença própria do projeto foi inventada.

## Comparação com aMule 3.1.0

Referência oficial: [aMule 3.1.0](https://github.com/amule-org/amule/releases/tag/3.1.0),
publicado em 21/09/2026, commit `909d304d993ee07df6c6f6acf501a6d791d53666`.
A comparação usa o código dessa revisão, não somente a aparência da interface.

| Recurso | Apocalipse ED2K neste checkpoint | Referência no aMule |
| --- | --- | --- |
| Login, ID e busca em servidor ED2K | Implementados; fixtures TCP locais | `ServerConnect.cpp`, `SearchList.cpp` |
| Consulta de fontes e download direto | Implementados; um peer por arquivo de cada vez | `DownloadQueue.cpp`, `PartFile.cpp` |
| Verificação MD4, hashsets e retomada | Implementados e testados | `PartFile.cpp` |
| Callback de Low-ID pelo servidor | Parcial: requer solicitante High-ID e porta acessível | `DownloadClient.cpp`, `ListenSocket.cpp` |
| Compartilhamento, fila e limite de upload | Implementados; sem créditos persistentes | `SharedFileList.cpp`, `UploadQueue.cpp` |
| Publicação dos compartilhados no servidor | Implementada | `ServerSocket.cpp` |
| Kad persistente e lookup por hash | Parcial: tabela limitada, contatos/refresco e fontes diretas | `kademlia/routing`, `kademlia/kademlia/Search.cpp` |
| Kad HELLO e desafio de endpoint | Implementados em protocolo plaintext v5; fixtures UDP locais | `kademlia/net/KademliaUDPListener.cpp` |
| Kad pacotes comprimidos e lotes de fontes | E5/zlib e coleta limitada; não é ofuscação | `Packet.cpp`, `kademlia/kademlia/Indexed.cpp` |
| Kad busca por palavras, índices e publicação | Não implementados | `kademlia/kademlia/Indexed.cpp` |
| Firewall Kad, buddy e callbacks UDP | Não implementados | `kademlia/kademlia/UDPFirewallTester.cpp` |
| Ofuscação TCP/UDP | Não implementada | `EncryptedStreamSocket.cpp`, `EncryptedDatagramSocket.cpp` |
| Identificação segura e créditos RSA | Não implementados | `ClientCredits.cpp`, `ClientCreditsList.cpp` |
| AICH e recuperação por subblocos | Não implementados | `SHAHashSet.cpp` |
| Múltiplas fontes simultâneas do mesmo arquivo | Não implementadas | `PartFile.cpp`, `DownloadQueue.cpp` |
| Troca de fontes entre clientes | Não implementada | `DownloadClient.cpp` |
| IP filter, banimento e controles equivalentes | Sem equivalência completa | `IPFilter.cpp`, `ClientList.cpp` |
| Amigos, mensagens, notas e avaliações | Não implementados | `FriendList.cpp`, `CommentDialog.cpp` |
| Estatísticas persistentes e API externa equivalente | Sem equivalência completa | `Statistics.cpp`, `ExternalConn.cpp` |

O próximo critério é executar os testes em Windows nativo e demonstrar
interoperabilidade controlada com aMule. HELLO/challenge, E5 e lotes estão
cobertos por fixtures locais, sem prova de alcance público ou NAT. Identificação
segura/créditos, AICH, busca/publicação Kad, callbacks/ofuscação e transferência
multifonte permanecem trabalhos distintos. Nenhuma bandeira de capacidade
anuncia esses recursos enquanto estiverem ausentes.

## Defeitos adicionais corrigidos nesta continuação

| Achado | Efeito observado ou demonstrado | Correção |
| --- | --- | --- |
| Power sem `engine` no resultado | Processo retornava sucesso, mas o adaptador rejeitava o JSON e voltava ao fluxo legado | Envelope comum schema/engine/operation/result; regressão com motor real |
| Colapse sem `engine` e concatenação divergente | Contrato ADM rejeitava resposta; operação com uma entrada não era aceita pela CLI | Identidade explícita e `wav-concat` compatível com uma ou mais entradas |
| Colisão de fixture M3U8 | Limpeza de um teste podia remover o cache de outro no Windows | `tempfile::TempDir` exclusivo e regressão concorrente com dois processos |
| Filho órfão após saída do pai | Timeout retornava com descendente ainda executando | Grupo Unix ou Job Object Windows mantido durante toda a operação |
| Publicação após cancelamento/remoção | Hash/cópia bloqueante continuava depois do sinal e podia criar saída após a limpeza | Leases de conclusão, cópia/hash canceláveis e gate antes da publicação |
| Cache nativo não removido | `data/m3u8-work/<id>` permanecia após apagar a tarefa e os arquivos | Remoção do workspace exato depois da barreira de término |
| Preferências RPC antigas perdidas | Desativação, autostart, porta e segredo voltavam aos defaults | Aliases de leitura; o caminho `aria2_path` nunca seleciona o Xtreme |
| Update no caminho errado | Uma cópia portable era atualizada enquanto o runtime continuava usando um caminho personalizado | Instalador utiliza a mesma resolução de caminho do runtime |
| Pausa depois da conclusão | `Completed` podia voltar para `Paused`, deixando um destino final já existente | Pausa preserva estado concluído |
| Reconexão durante teardown lento | Intenção de retomar se perdia; evento antigo também podia despachar depois de um evento novo | Espera de leases e coordenação entre geração da rede, fila e registro de workers |
| Tokio Windows com espera pendente | Ordem de Drop fechava handle antes de retirar a espera | Tokio 1.54.0 no workspace e no lock |
| Refresh Kad disputando pending com fontes | Fixture real devolveu zero fontes por `kad_query_busy` | Foreground mantém lock durante lookup/coleta; manutenção cede por contato |
| Respostas Kad E5 e múltiplos lotes | Respostas válidas do aMule podiam ser ignoradas | Decodificação zlib limitada e coleta de vários datagramas correlacionados |
| UDP Windows com datagrama grande ou peer offline | Erro recuperável podia encerrar a sessão inteira | Buffer de datagrama completo e tratamento limitado de erros transitórios |

## Verificação desta etapa

- JavaScript: **337 testes**, nos **43 arquivos**, passaram sem falhas. O modo
  agregado `node --test` reportou arquivos nesta execução; o log de execução
  direta registra os 337 casos individuais.
- Core ADM: **52 testes Rust** passaram após a atualização para Tokio 1.54.0.
- Xtreme: Clippy `--all-targets --all-features -- -D warnings` passou em Linux;
  **17 regressões selecionadas** de identidade HTTP, contrato e RPC passaram.
- Power: **35 testes Rust e 11 testes Python** passaram; Clippy passou. O teste
  adicional ADM executa o Power real e verifica a conversão de mídia.
- M3U8: **52 testes Rust** passaram; os testes compilaram para Windows GNU.
  O executável Windows foi reutilizado do checkpoint, com hash e fontes de
  produção conferidos; somente fixtures/dependências de desenvolvimento mudaram.
- Colapse: **49 testes Rust e 15 testes Python** passaram; Clippy passou.
  O adaptador ADM exercita as sete operações com verificações de bytes WAV,
  Y4M e estrutura/offsets MP4. A fixture MP4 não demonstra vídeo decodificado.
- A suíte dos módulos nativos do ADM passou com **33 testes comuns e quatro
  integrações com motores reais**. Os 33 incluem **16 casos Kad**, também
  repetidos por uma revisão independente. A fixture interna de subprocesso é
  acionada pelos próprios testes de supervisão. O harness inclui diretamente
  os módulos de produção, com UDP em loopback, subprocessos e publicação;
  não exercita a GUI Tauri. Comandos constam no resumo de validação do pacote.
- O check Windows compila também os testes, incluindo migração das preferências
  e caminhos dos motores. A verificação não executa os testes do binário Tauri
  em Windows. Os cinco avisos já presentes de depreciação/variáveis de outros
  módulos não foram tratados como uma auditoria ou correção global.
- Os três workflows passaram por validação de YAML e dos blocos Bash. Foram
  incluídas as integrações reais Power/Colapse após a compilação dos motores.
  Não há novo resultado remoto de CI para os commits ainda não publicados.

## Limites de aceitação

Os executáveis Windows GNU x64 e o loader WebView2 são artefatos de teste,
sem execução nativa de Windows ou assinatura Authenticode nesta etapa. Os
testes nativos adicionais deverão exercer processo/job, I/O e GUI no sistema
de destino; os workflows atuais incluem testes e selfcheck sem interface.
O AppImage não foi validado com os motores embutidos: somente a árvore portable
recebe o conjunto Nibiru. Os caminhos graváveis na montagem AppImage também
precisam de validação específica.
Morte abrupta do processo do ADM entre criar um filho suspenso e atribuí-lo ao
Job Object permanece uma janela distinta de cancelamento normal da future.

Não foi demonstrada interoperabilidade pública ED2K/Kad, abertura de portas,
NAT ou estabilidade prolongada. Não foi concluída a leitura semântica linha
por linha de todo o ADM. Os relatórios de revisão registram os arquivos e
intervalos efetivamente lidos; busca textual e aprovação dos testes não
substituem essa cobertura.

## Referências de verificação

- aMule: revisão `909d304d993ee07df6c6f6acf501a6d791d53666`, fontes de
  `KademliaUDPListener.cpp`, `Indexed.cpp`, `Packet.cpp` e `SafeFile.cpp`.
- [RegisterWaitForSingleObject](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-registerwaitforsingleobject):
  o handle não pode ser fechado enquanto a espera estiver pendente.
- [recvfrom](https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-recvfrom):
  condições Windows de datagrama grande e ICMP Port Unreachable em UDP.
- Tokio local 1.53.1 e 1.54.0: `src/process/windows.rs`; ordem de campos
  `waiting`/`child` conferida diretamente nas duas versões.

Os PRs existentes continuam rascunhos. As branches e revisões propostas estão
identificadas no pacote de revisão. Nenhum merge ou release foi realizado.
