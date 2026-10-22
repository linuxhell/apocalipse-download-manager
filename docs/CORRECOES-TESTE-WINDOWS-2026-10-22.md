# Continuação do teste Windows: correções HTTP, motor nativo e ED2K

Base: `claude/ed2k-share`, após `c241c740`. Integra o avanço remoto da retomada.
Nenhuma release, tag, PR ou merge em main faz parte deste trabalho.

## Evidência do usuário

- HTTP: ISO concluída; pausas e retomadas funcionaram. O diagnóstico confirma seis
  reutilizações de parcial, inclusive uma com 5.075.107.840 bytes preservados.
- Ao excluir da lista e do disco, sobraram `.nibiru-http.part`,
  `.nibiru-http.part.json` e `.nibiru-http.lock`; após conclusão, sobrou o lock.
- A velocidade apresentada oscilava, incluindo amostras de zero. O motor publica
  deltas de bytes gravados em blocos de 4 MiB a cada 250 ms; o ADM os exibia diretamente.
- A captura mostra a velocidade truncada e uma coluna de miniatura vazia antes do nome.
- O usuário relatou metadados de torrent rápidos, download e visualização funcionando.
  Exclusão do torrent concluído ainda aguarda o resultado do usuário.

## Correções

- `nibiru_artifacts.rs`: remove apenas os quatro nomes exatos do destino (parcial,
  registro, temporário do registro e lock). Espera o motor liberar o arquivo de
  coordenação antes da limpeza, inclusive enquanto o cancelamento grava o registro.
  Recusa diretórios e symlinks. No Windows, abre exclusivamente o lock e o remove
  por delete-on-close. A ISO e arquivos vizinhos não são removidos por esse helper;
  a exclusão do arquivo principal permanece no fluxo explícito de remover do disco.
- O resultado da exclusão agora também verifica e registra sobras do Xtreme.
- `transfer_speed.rs`: somente transferências diretas do Xtreme usam a taxa calculada
  no ADM sobre uma janela de aproximadamente 2 s de bytes cumulativos. O primeiro
  contador é a referência, portanto os bytes restaurados não viram pico de velocidade.
  Quedas do contador reiniciam a medição; uma parada real zera a janela. O diagnóstico
  preserva a velocidade bruta do motor e identifica `xtreme-byte-window-2s`.
- Torrent mantém a taxa nativa. O ADM usa exclusivamente o backend Nibiru Xtreme.
- Lista: não reserva coluna de miniatura em transferências sem miniatura; espaçamento
  menor junto à seleção; velocidade em coluna própria sem reticências. Torrent mantém
  as taxas de download e upload; detalhes permanecem no menu existente.
- Pedido adicional do usuário, restrito à escolha de arquivos do torrent: espaçamento
  entre título e lista; o resumo técnico `Aria2Rpc · peer_to_peer` passa a mostrar
  `nibiru-xtreme`, conforme o nome solicitado para o slot do motor. Não muda metadados
  nem streaming. O nome do produto é o mesmo nos três idiomas.
- `test-build.yml` também gera artefatos em pushes para a branch integrada, sem release.

## Integração nativa e pedidos adicionais

- Cliente, comandos, modelo e empacotamento usam Nibiru Xtreme e RPC `nibiru.*`.
  O lançamento usa `serve` com argumentos nativos. O atualizador usa releases
  publicadas do repositório nativo, incluindo pré-releases, com verificação SHA256.
- Os nomes antigos de campos de configurações e fila ficam apenas em um dicionário
  de migração JSON, necessário para preservar a instalação anterior. O código
  executável não referencia o motor antigo. Documentos históricos permanecem.
- Logs sai de Configurações e ganha página própria, logo abaixo dela no painel esquerdo.
- ED2K: campo **Salvar em** sugere a pasta do ADM; **Escolher pasta…** abre o seletor.
  O botão Baixar da busca leva à transferência para conferir link e destino antes
  de adicionar. A pasta fica registrada por tarefa e não muda ao retomar.
- ED2K: removido o selo de versão de prévia em todos os idiomas.
- ED2K: **Visualizar** aparece ao lado de Pausar/Retomar para vídeos e usa o player
  configurado no ADM. Um servidor HTTP privado de loopback suporta HEAD e Range.
  Só lê partes marcadas pelo motor após hash e gravação; o total percentual não
  autoriza a leitura de lacunas. A primeira parte precisa estar verificada para
  habilitar o botão. Após reiniciar, a retomada precisa verificar novamente o parcial.
  Ao buscar trecho ainda ausente, aguarda até 90 s; não há priorização ED2K de partes
  pedidas pelo player. Alguns MP4 dependem do fim do arquivo para começar a tocar.
- Biblioteca Xtreme fixada no commit `15d253733707dd4d3b32c1d05350d9a45c3c0e7f`:
  progresso inclui mapa de partes verificadas; lock de escritor fica separado dos
  dados para permitir leitores no Windows. O lock Windows usa delete-on-close;
  Unix conserva o arquivo de coordenação para não criar uma corrida de unlink.

## Verificação local

- Rust 1.97.0: 2 testes funcionais da janela de velocidade e 3 da limpeza passaram.
  A limpeza foi testada com lock ativo, gravação tardia de registro, diretório no lugar
  do parcial, ISO concluída e arquivo vizinho que precisam ser preservados.
- 343 testes Node passaram; `cargo fmt --all -- --check` e `git diff --check` passaram.
- Os testes Rust locais acima compilam os módulos reais com `rustc --test`; não são
  uma compilação Tauri completa. A CI Windows/Linux/macOS é o gate do aplicativo.
- O navegador de teste não pôde ser instalado neste ambiente (downloads inválidos).
  O ajuste CSS ainda precisa da conferência visual no aplicativo Windows.
- Core Rust: 50 testes unitários e 2 de integração passaram antes da alteração ED2K.
- Migração: 2 testes do módulo real passaram. Streaming ED2K: 3 testes do módulo
  real passaram, incluindo Range, token/Host e espera por partes não verificadas.
- Xtreme: 7 testes de transferência ED2K passaram com peer local real, incluindo
  leitura das partes indicadas no progresso; clippy da biblioteca sem features passou.
- A CI e o novo portátil devem ser conferidos após o push; não presumir sucesso.

Não usar os relatórios com pausas para calcular a média dividindo o tamanho total
pelo tempo apenas da última retomada. Não publicar links assinados temporários.
