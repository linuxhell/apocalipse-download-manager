# Continuação do teste Windows: exclusão HTTP, velocidade e largura da lista

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
- Torrent e aria2 continuam com a taxa nativa. A identificação vem do namespace RPC,
  não do nome do executável. O diagnóstico passa a registrar o backend correto.
- Lista: não reserva coluna de miniatura em transferências sem miniatura; espaçamento
  menor junto à seleção; velocidade em coluna própria sem reticências. Torrent mantém
  as taxas de download e upload; detalhes permanecem no menu existente.
- Pedido adicional do usuário, restrito à escolha de arquivos do torrent: espaçamento
  entre título e lista; o resumo técnico `Aria2 · motivo` vira a orientação localizada
  "Escolha os arquivos do torrent" (en/pt-BR/zh-CN). Não muda metadados nem streaming.
- `test-build.yml` também gera artefatos em pushes para a branch integrada, sem release.

## Verificação local

- Rust 1.97.0: 2 testes funcionais da janela de velocidade e 3 da limpeza passaram.
  A limpeza foi testada com lock ativo, gravação tardia de registro, diretório no lugar
  do parcial, ISO concluída e arquivo vizinho que precisam ser preservados.
- 338 testes Node passaram; `cargo fmt --all -- --check` e `git diff --check` passaram.
- Os testes Rust locais acima compilam os módulos reais com `rustc --test`; não são
  uma compilação Tauri completa. A CI Windows/Linux/macOS é o gate do aplicativo.
- O navegador de teste não pôde ser instalado neste ambiente (downloads inválidos).
  O ajuste CSS ainda precisa da conferência visual no aplicativo Windows.
- A CI e o novo portátil devem ser conferidos após o push; não presumir sucesso.

Não usar os relatórios com pausas para calcular a média dividindo o tamanho total
pelo tempo apenas da última retomada. Não publicar links assinados temporários.
