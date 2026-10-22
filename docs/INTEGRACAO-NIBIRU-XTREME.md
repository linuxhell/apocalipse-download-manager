# Integração nativa do Nibiru Xtreme no ADM

## Estado em 22/10/2026 — branch `claude/ed2k-share`

A implementação atual está em `c6b57cc`; as seções históricas abaixo descrevem
etapas anteriores. O resumo atual das correções e das verificações está em
[CORRECOES-TESTE-WINDOWS-2026-10-22.md](CORRECOES-TESTE-WINDOWS-2026-10-22.md).

- Runtime em `src/xtreme.rs`, lançamento por `serve`, sessão nativa e chamadas
  `nibiru.*`. Pausa/retomada usam `pause`/`resume`; limpeza usa `remove`/`removeResult`.
- Código, comandos e modelos usam identificadores Xtreme. Os campos de versões
  anteriores são migrados por um dicionário JSON para preservar fila e configurações;
  o enum salvo passa a `xtreme_rpc`, validado por desserialização no enum real.
- Os três workflows de empacotamento usam os assets nativos com SHA256. No portátil,
  o motor está em `data/tools/nibiru-xtreme/nibiru-xtreme(.exe)`.
- O atualizador usa releases publicadas, incluindo pré-releases. O pacote de teste
  de hoje usa o artefato da CI alpha.4, ainda distinto de uma release publicada.
- Player, extrator e demais ferramentas mantêm suas configurações. Torrent exibe
  `nibiru-xtreme` na seleção de arquivos. A janela ED2K usa a biblioteca embutida,
  fixada em `15d2537`, e oferece pasta por transferência e prévia de partes verificadas.
- Passaram localmente 343 testes Node, 50 testes unitários + 2 de integração do core,
  3 testes de migração e 3 de streaming ED2K. Um teste com o cliente RPC e motor
  reais baixou 16 MiB, pausou, retomou, comparou cada byte e removeu a tarefa.
  A validação de aplicativo nativo e o empacotamento são registrados na CI.

## Registro histórico da integração inicial

Escopo (decisão do usuário): **só o aria2-ultra é substituído**. yt-dlp (nightly), N_m3u8DL-RE, FFmpeg, QuickJS,
extrator e player ficam como estão. Os IDs internos (`aria2`, `aria2_path`, `aria2_release_repo`, comandos
`*_aria2_*`) **não foram renomeados**: o Xtreme fala o RPC do aria2, então o resto do ADM continua igual.

## O que mudou (branch `claude/adm-xtreme-tools`, base `main` @ `a945667a`)

- **Repositório padrão do slot**: `linuxhell/nibiru-xtreme`. Configurações que ainda traziam o padrão antigo
  (`linuxhell/aria2-ultra`) passam a seguir o novo. Um repositório de fork do aria2 digitado à mão continua
  funcionando (assets `aria2c-<sufixo>`, executável `aria2c`).
- **Assets do Xtreme**: `nibiru-xtreme-<versão>-windows-x64.exe | linux-x64 | macos-x64` + `.sha256`
  (mesmo formato do `release.yml` do Xtreme; o ADM lê o primeiro token hexadecimal de 64 caracteres).
- **Pasta do motor**: `tools/nibiru-xtreme/nibiru-xtreme(.exe)`. Só essa subpasta é escrita; os outros motores
  nunca são tocados.
- **Baixar**: baixa a última release, confere o SHA-256, valida `--version`, instala e **preenche o campo** com o
  caminho novo (o usuário só clica em Salvar). Para o runtime em execução antes de trocar o arquivo.
- **Atualizar**:
  1. sem motor no caminho (arquivo ausente e nada no PATH) → baixa o mais recente para `tools/…` e devolve o
     caminho (`<id> installed: <caminho>`); a interface preenche o campo;
  2. o slot ainda aponta para um `aria2c` → instala o Xtreme ao lado e deixa o `aria2c` intocado (reversível);
  3. caso contrário **consulta a release primeiro**: se a tag é a versão já instalada (comparação exata, `v0.2.0`
     não casa com `0.2.0-alpha.2`) responde "já está atualizado" **sem baixar**; senão baixa, valida e troca
     somente aquele executável, com backup e restauração se a validação falhar.
- **Releases só com pré-release**: `/releases/latest` do GitHub ignora pré-releases e responde 404. O ADM agora cai
  para a lista de releases e usa a mais recente não-rascunho, necessário enquanto o Xtreme só publicar `-alpha`.
- Rótulo da aba Ferramentas (pt/en/zh): "Nibiru Xtreme (HTTP/HTTPS, FTP/SFTP, torrent e magnet)" (o teste `obsolete-engine-residue` proíbe o termo ED2K no código do ADM).
- Log `aria2.runtime_spawned` informa o binário e o motor reais.

## O que NÃO mudou (de propósito)

- O **pacote** (`.github/workflows/release.yml`) ainda embute o `aria2c` de `FerroDownload/aria2-static-builds`
  em `data/tools/aria2/`. Instalações novas continuam apontando para ele até existir uma release do Xtreme que
  possa ser empacotada. Trocar o empacotamento antes disso quebraria as releases do ADM.
- Argumentos de inicialização (`aria2.rs`): inalterados.

## Evidência

| O quê | Como | Resultado |
| --- | --- | --- |
| Xtreme aceita **exatamente** os argumentos que `aria2.rs` passa | binário da branch `audit/code-review-2026-10-21` (`4eaa76c`), mesmos ~30 flags | sobe, responde `aria2.getVersion` (`0.2.0-alpha.2`) |
| Download HTTP com as opções do `add_download` (`dir`, `out`, `split`, `max-connection-per-server`, `header`…) | servidor local, 30 MB | `complete`, SHA-256 idêntico, `tellStatus`/`getGlobalStat`/`removeDownloadResult` ok |
| `--stop-with-process` | processo pai encerrado | o motor encerra junto (observado em Linux) |
| Lógica de versão/slot | testes unitários no ADM | ver abaixo |

**Não** foi validado nesta etapa: BitTorrent/magnet/ED2K através do ADM, retomada após reinício com o ADM,
`--input-file`/`--save-session` (o Xtreme ignora no modo aria2; o ADM já recria tarefas com GID obsoleto),
Windows/macOS em uso real, e o fluxo de download pela interface (a interface gráfica não foi aberta).

## Falta para a troca ficar completa

1. Existir uma release (pré-release) do `nibiru-xtreme` com os três binários e `.sha256`.
2. Trocar o empacotamento (`release.yml`/`validate-portable.yml`) para embutir o Xtreme.
3. Teste ponta a ponta no app: HTTP, pausa/retomada, magnet → seleção de arquivos, streaming, ED2K, fechar o ADM
   e confirmar que o motor encerra, atualizar o binário pela aba Ferramentas.

## Threads por download e memória por site

- O motor aceita de 1 a 16 conexões por download (teto interno); 1 roda em um único fluxo (testado no Xtreme:
  pico de 1 requisição simultânea). O ADM manda `split`, `max-connection-per-server` e `min-split-size=1M`, e o
  número é um **limite superior**: arquivos pequenos usam menos (até 32 MB: 2; até 128 MB: 4; até 512 MB: 8;
  até 2 GB: 12; acima: 16).
- Slider da janela de download: 1–16, padrão **8** (era 1–32, padrão 16). O padrão global novo também é 8 (quem já
  tinha um valor salvo mantém o dele). A regra especial dos hosts da Microsoft não força mais 16: sem escolha
  usa 16, com escolha respeita o valor. Pixeldrain continua fixo em 1.
- Caixa "Memorizar a quantidade de threads para este site" (en/pt-BR/zh-CN) logo abaixo do slider: marcada, ao
  clicar em Baixar grava a contagem na regra do **host exato** (`set_site_threads`); a janela lembra do valor e
  da marcação nas próximas vezes; marcada e com o slider em outro valor, **atualiza**; desmarcada, ao Baixar,
  **apaga só a contagem** (velocidade, User-Agent e credenciais da regra ficam; a regra some se só tinha a
  contagem). Downloads que não passam pela janela (extensão) também usam o valor memorizado.
- Testado: 3 testes de Rust (criar/atualizar/esquecer, preservar o resto da regra, prioridade da escolha), 3 de
  JavaScript e uma passada no Chromium com backend simulado nos 3 idiomas. **Não** foi testado no app Tauri real.
