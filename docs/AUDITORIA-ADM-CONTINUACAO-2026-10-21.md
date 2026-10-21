# Continuação da auditoria do ADM — 21/10/2026

Base: checkpoint `8ababf01` (recuperado do bundle completo, árvore
`fb7bd5d1…` conferida). Branch: `claude/optimistic-heisenberg-cozrt2`.
Este documento separa **compilado**, **testado automaticamente** e **validado em
uso real**. Nada abaixo foi validado em uso real nem em Windows/macOS.

## Ambiente de validação e limite conhecido

As revisões fixadas dos motores (Xtreme `0a9fcdde` no `Cargo.lock`; Xtreme
`dd92f6ee`, Power `6d5f207e`, M3U8 `2f1ae000` no manifesto) **não estão
publicadas** no GitHub; só o Colapse `6f92c7e` está. O `cargo` do workspace
completo, portanto, não resolve neste ambiente e a CI do ADM não pode passar
até a publicação. Os testes do desktop foram executados em uma cópia de
rascunho com o Xtreme apontado para a branch publicada `work/native-rpc-ed2k`;
isso prova a compilação e a lógica do ADM, não a compatibilidade com a revisão
fixada. O Colapse real foi compilado de `6f92c7e` para o teste de operações.

## Defeitos confirmados e corrigidos

| Área | Defeito | Correção | Commit |
| --- | --- | --- | --- |
| Ferramentas | Atualizar N_m3u8DL-RE, 7-Zip ou mpv apagava a instalação antes de validar o download | Diretório de staging e troca somente após validação | `16e42cc7` |
| Ferramentas | Troca de executável interrompida apagava o único backup | Backup é restaurado primeiro | `16e42cc7` |
| Colapse | Operações não anunciadas eram executadas | `features` do contrato filtram a operação; `get_nibiru_capabilities` lista as disponíveis | `16e42cc7` |
| Fila | Falha ao criar o motor HTTP mantinha a vaga de worker | Vaga liberada e fila avança | `badd4997` |
| HTTP nativo | `rename` final sobrescrevia arquivo criado no destino durante a transferência | `hard_link` sem substituição, com alternativa | `badd4997` |
| Remoção | "Apagar arquivos" removia arquivo preexistente que a tarefa nunca produziu | Preservado quando anterior à tarefa e não concluída | `badd4997` |
| Xtreme | Pause atrasado de geração anterior deixava tarefa retomada em Pausada | Retoma o motor se o usuário não pausou (máx. 3) | `f6bce7f8` |
| Fila salva | Uma entrada ilegível descartava a fila inteira | Leitura entrada por entrada; original em `*.corrupt-<hora>` | `028befd4` |
| Processos | yt-dlp/ffmpeg sem supervisão de árvore: descendente prendia os pipes e travava a tarefa; sobrava processo após pausa/queda | Grupo de processos / Job Object do Nibiru; pipes drenados com limite de 5 s | `3579e23d` |
| Segredos | Pausa deixava cookie jar e config de senha do yt-dlp em disco | Removidos ao pausar | `3579e23d` |
| HLS áudio | ffmpeg gravava direto no destino com `-y` | Parcial privado, saída vazia rejeitada, publicação sem substituir | `3579e23d` |
| yt-dlp mídia | Cópia final direta e não atômica | Cópia ao lado do destino e publicação sem substituir | `ef86ba33` |
| Mux social | `rename` sobrescrevia; worker cancelado removia o registro do worker novo | Publicação sem substituir; remoção só se não cancelado | `ef86ba33` |
| Metalink | Tarefa-mãe era removida antes dos filhos (sumia se a gravação falhasse ou todos fossem rejeitados); manifesto sem limite e não cancelável | Mãe só sai após filhos aceitos, senão falha visível; limite de 8 MiB e cancelamento | `ef86ba33` |
| yt-dlp | `--update-to` bloqueava um worker e sem timeout | `tokio::process`, timeout de 300 s | `ef86ba33` |
| Link | Um symlink dentro de uma pasta compartilhada que aponta para fora permitia ler, listar e gravar fora do compartilhamento | Caminho resolvido (parte existente mais profunda) deve permanecer sob a raiz canônica do compartilhamento | `04bc362f` |
| Link | Login por conta do sistema sem limite compartilhado: conexões paralelas permitiam adivinhar a senha da conta pela LAN | Orçamento global de 8 falhas por 60 s, depois 429 | `04bc362f` |
| Link | A listagem mantinha o lock das configurações durante a leitura do disco, travando a interface e a bridge em compartilhamentos lentos | Lock liberado antes de listar | `04bc362f` |
| Configurações | Um campo ilegível em `settings.json` descartava todas as preferências | Leitura campo a campo; original em `*.corrupt-<hora>` | `247ab185` |
| Bridge | Upload de blob abandonado (navegador/extensão morreu) deixava a tarefa em Downloading e um parcial para sempre | Expira após 10 min sem atividade (tarefa falha, parcial removido) | `0b53eaf3` |
| Bridge | Falha no fim do upload (incompleto, destino ocupado) deixava tarefa e parcial órfãos; o `rename` final substituía arquivo existente | Falha a tarefa, remove o parcial, publica sem substituir | `0b53eaf3` |
| Bridge | `browser-download-complete` aceitava qualquer arquivo existente | Só arquivos modificados nos últimos 15 min | `0b53eaf3` |
| Extensão | Falha no envio deixava o desktop esperando | Novo `/v1/blob/abort`; gravação com falha ainda finaliza para manter a parte recebida | `e2d1904e` |
| Extensão | `waitWhilePaused` consultava `/v1/blob/status` a cada 16 KiB por um campo `paused` que o desktop nunca envia | Removido (código morto comprovado) | `e2d1904e` |

Código morto removido: a linha de comando do N_m3u8DL-RE em
`run_external_download` (HLS não-áudio vai ao Nibiru M3U8 em `start_download`);
`clean_directory`. O campo `n_m3u8dl_re_path` e a gestão da ferramenta na
interface foram mantidos por compatibilidade com configurações antigas; não são
mais usados por nenhum fluxo e são candidatos a remoção com ajuste de interface
e traduções.

## Testes automáticos adicionados

`failed_tool_directory_replacement_keeps_the_working_installation`,
`colapse_operations_follow_the_advertised_features`,
`publishing_never_replaces_a_destination_created_during_the_transfer`,
`one_unreadable_queue_entry_does_not_discard_the_other_downloads`,
`external_output_is_published_without_replacing_and_must_not_be_empty`,
`supervised_external_process_leaves_no_straggler_holding_its_pipes` (processo
real, Linux), `link_paths_cannot_escape_a_share_through_a_symlink` (Unix),
`link_account_guessing_is_limited_by_a_shared_failure_budget`,
`one_unreadable_setting_keeps_the_other_preferences`,
`only_recently_finished_files_count_as_browser_downloads`, e o teste JS
`extension-blob-abort`. O teste JS `link-folder-settings` foi atualizado: ele
ainda esperava a linha de comando do N_m3u8DL-RE removida antes (falha que
não havia sido detectada porque a suíte JS não foi reexecutada naquele commit).

## Pendências reais

- Leitura semântica restante de `main.rs`, servidor Link, extensão e interface.
- Código Windows alterado (`nibiru_cli::spawn_supervised` em yt-dlp/ffmpeg) não
  foi compilado em Windows aqui.
- Cookies e credenciais continuam na linha de comando do ffmpeg
  (`-headers`) na rota de áudio HLS e do mux social: visíveis a outros
  processos do usuário. Sem alternativa sem arquivo de cabeçalhos.
- ED2K/Kad: lacunas do relatório anterior inalteradas.
- CI, builds Windows/macOS e validação em uso real: pendentes.
