# Integração do Nibiru Xtreme no slot do aria2

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
- Rótulo da aba Ferramentas (pt/en/zh): "Nibiru Xtreme (HTTP/HTTPS, FTP/SFTP, torrent, magnet e ED2K)".
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
