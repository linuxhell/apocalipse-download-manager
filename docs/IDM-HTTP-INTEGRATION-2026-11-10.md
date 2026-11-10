# Integração HTTP Nibiru 0.2.7 — ADM 0.8.16

O ADM fixa o motor em `52d77572084819001973e4ff8cf3d61ec5e9292b`, tanto na
dependência Rust quanto no Cargo.lock. O CI existente extrai esse mesmo commit
do lockfile para construir o backend incluído nos pacotes. Assim o aplicativo
e o executável de downloads usam a mesma implementação.

O [PR do motor](https://github.com/linuxhell/nibiru-xtreme/pull/7) contém o
escalonador de faixas HTTP, os testes e a
[análise do IDM com medições reproduzíveis](https://github.com/linuxhell/nibiru-xtreme/blob/52d77572084819001973e4ff8cf3d61ec5e9292b/docs/IDM-ANALYSIS-2026-11-10.md).

## Comportamento

- O início HTTP segmentado usa respostas de dados para validar as faixas,
  dispensando o GET descartável de um byte. O método explícito de sondagem
  preserva esse teste.
- Pedidos maiores, limitados a 32 MiB, reduzem negociações repetidas. Workers
  livres podem assumir parte da maior cauda ainda não reservada de outro worker.
- O bloco em recepção/gravação permanece com seu dono; cortes alinhados aos
  blocos preservam a retomada e evitam escritas sobrepostas.
- A publicação continua aguardando sincronização do arquivo e renomeação.
  O Nibiru já grava os dados em suas posições e não precisa concatenar uma
  segunda cópia completa ao final.

## Interface

“Finalizando arquivo…” aparece para uma tarefa ainda em `downloading`, com
tamanho positivo conhecido e `received >= total`. É uma indicação baseada
nos contadores do backend, não uma porcentagem de montagem/fsync. Tarefas
concluídas, pausadas, com falha, ao vivo ou torrent ficam fora dessa inferência.
O estado concluído continua dependendo do resultado bem-sucedido do backend.

Há traduções nos três idiomas e atributos de progresso para leitores de tela.
Os controles de conexões e banda continuam com seus limites existentes.

## Verificação e limites

Os 384 testes JavaScript passaram, incluindo os casos de finalização e suas
exclusões. O motor passou os testes de todos os recursos, Clippy e regressões
de concorrência, HTTP real, retomada, identidade, BitTorrent e ed2k.

As comparações locais do motor tiveram cinco repetições alternadas em cada
cenário, arquivos verificados por SHA-256 e reduções de 13,6% e 6,0% no tempo
total. Não medem a conexão do usuário, não executam IDM e não estabelecem ganho
de velocidade em torrent/ed2k. A disponibilidade e a taxa de upload das fontes
P2P continuam relevantes.

O ambiente local não tem GTK/WebKit de desenvolvimento; a compilação e os
testes da janela desktop e os pacotes para Windows, Linux e macOS dependem do CI.
Os PRs preparam as versões 0.2.7/0.8.16; a publicação ocorre pelo fluxo de
release do repositório após integração, não pela simples abertura do PR.
