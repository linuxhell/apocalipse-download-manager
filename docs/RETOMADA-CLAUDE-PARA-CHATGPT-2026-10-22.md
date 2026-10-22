# Retomada: Claude → ChatGPT (22/10/2026) — terminar o que falta para o usuário testar

Escrito pelo Claude a pedido do usuário (Juliano). **Este arquivo continua o de 21/10**
(`docs/RETOMADA-CLAUDE-PARA-CHATGPT-2026-10-21.md`, no repositório do Xtreme, branch
`audit/code-review-2026-10-21`) e o substitui no que aconteceu **depois dele**. As decisões e regras daquele
arquivo (seção 1) continuam valendo; onde este contradiz, este vale. Leia tudo antes de editar.

Vocabulário (não misture):

- **Compilado** = compila. **Testado automaticamente** = há teste que passa (diga onde).
- **Validado em uso real** = rodou com cliente/servidor/rede/roteador de verdade. **Pouquíssimo está neste nível.**
  O que o usuário rodou no Windows está listado na seção 4 e é o único uso real.

---

## 0. Onde está cada coisa (abrir estes dois repositórios)

| Repo | Branch | Cabeça | O que é |
|---|---|---|---|
| `linuxhell/nibiru-xtreme` | `audit/code-review-2026-10-21` | `f8f65b2` + o commit que adiciona este arquivo | Motor. Fonte de verdade do Xtreme. |
| `linuxhell/apocalipse-download-manager` (ADM) | **`claude/ed2k-share`** | `2b496395` + o commit deste arquivo | ADM com **tudo junto** (ver abaixo). **É a branch que deve virar o artefato de teste.** |
| ADM | `claude/adm-xtreme-tools` | `f53695bd` | Só a parte "aba Ferramentas + threads por site", a partir de `main` (`a945667a`). **Já está incluída** em `claude/ed2k-share` por merge. |
| ADM | `chatgpt/ed2k-kad-client` | (do ChatGPT) | Base da janela ED2K/Kad. `claude/ed2k-share` parte dela. |
| ADM | `claude/optimistic-heisenberg-cozrt2` | `074f7a42` | Pesquisa antiga (compõe 4 motores, removeu o N_m3u8DL-RE). **Não usar como base.** |

- Nada foi mesclado em `main`. **Nenhum PR foi aberto.** Nenhuma tag foi criada por mim.
- Releases do Xtreme publicadas **com autorização expressa do usuário** (pré-releases): `v0.2.0-alpha.2`
  (commit `9188245`) e `v0.2.0-alpha.3` (commit `692b98a`). **A `0.2.0-alpha.4` (UPnP) NÃO foi publicada**:
  `Cargo.toml` já diz `0.2.0-alpha.4`, o release não existe. Publicar exige autorização do usuário **e** CI +
  dry-run verdes.
- `claude/ed2k-share` = `chatgpt/ed2k-kad-client` + (compartilhamento, Kad, UPnP, ofuscação) + merge de
  `claude/adm-xtreme-tools`. **Conflito resolvido no merge** (importante, ver 3.4).
- Clones locais do Claude são efêmeros; só vale o que está no remoto.

---

## 1. O que foi feito desde a retomada de 21/10 (em ordem)

### 1.1 Xtreme (`audit/code-review-2026-10-21`)

| Commit | O quê |
|---|---|
| `9188245` | **`rust-toolchain.toml` dizia `stable`** e anulava o Rust 1.97.0 fixado nos workflows. O build musl do Linux rodava no stable do runner, sem o alvo musl (`can't find crate for core`); Windows/macOS só passavam porque o alvo é o do próprio runner. Agora `1.97.0`. (O dry-run do `release.yml` revelou isso; CI normal nunca pegou.) |
| `36edabf` | **Retomada real de download HTTP segmentado** + `diagnose` (ver 1.1.1 e 1.1.2). |
| `692b98a` | Versão `0.2.0-alpha.3` + notas. |
| `f8f65b2` | **UPnP** (`src/upnp.rs`, `serve --upnp`, `tests/upnp.rs`), versão `0.2.0-alpha.4`. |

#### 1.1.1 Retomada de HTTP segmentado (bug real achado pelo usuário no Windows)
Antes: cada execução criava `.nibiru-http-XXXX.part` novo; Ctrl+C reiniciava do zero e deixava um órfão do
tamanho do arquivo. Agora: `<destino>.nibiru-http.part` + `<destino>.nibiru-http.part.json` com os blocos de
4 MiB já gravados (o parcial é descarregado em disco **antes** do registro ser trocado). A próxima execução
pergunta ao servidor (`If-Range`, mesmo ETag/Last-Modified e tamanho; a URL é comparada **sem a query**, então
link assinado renovado serve) e baixa só o que falta. Pausa, queda e Ctrl+C mantêm o parcial; arquivo trocado
ou registro inválido recomeçam. Sem ETag/Last-Modified o parcial é apagado. `download` agora trata Ctrl+C como
pausa (segundo Ctrl+C sai). Testes: `tests/http_resume.rs` (5). `tests/http_identity.rs` ajustado ao novo nome.
**Não testado:** retomada pelo RPC/ADM (usa o mesmo caminho, só exercitado em teste), queda de energia.

#### 1.1.2 `nibiru-xtreme diagnose <URL> [--dir D] [--full] [--engine-connections N] [--connections 1,8,16,32]`
Mede a rede **sem disco** (N conexões), o disco e o motor no mesmo link, e escreve
`nibiru-diagnose-report.txt`. **O veredito automático tem falhas conhecidas (corrigir):**
- diz "gravar em segundo plano ajudaria" mesmo com o motor a 99% do teto (o tempo de espera por disco não
  implica perda de velocidade);
- diz "N segundos abaixo de 90% da mediana" por causa da alternância 85/135 MB/s (seção 4.3), que não
  reduz a média;
- a linha "MB/s por segundo" do disco sai vazia (a gravação termina em <1 s e a descarga final domina);
- `--engine-connections` só **reduz** (teto interno 16). Não dá para testar 32 conexões no motor.

#### 1.1.3 UPnP (`nibiru_xtreme::upnp::Upnp`, feature `upnp`, ligada por padrão; `serve --upnp`)
Descobre o IGD (SSDP), mapeia **só** as portas pedidas (externa = local, IPv4, concessão 1 h, renova a cada
20 min, remove no fim/Ctrl+C/`--stop-with-process`); concessão permanente (erro 725) tratada; roteador que
anuncia URL de outro host é ignorado (`igd-next` recusa também loopback, **exceto** se listado em
`allowed_gateway_ips`, o que os testes fazem). Em `serve --upnp`: BitTorrent TCP/UDP, DHT UDP e, com `--ed2k`,
TCP do ED2K e UDP do Kad. **`Ed2kService::announced_port` virou `pub`.** Testes `tests/upnp.rs` (6) contra
roteador falso em loopback que fala SSDP, XML e SOAP. **Nenhum roteador real foi testado.** **No modo
compatível com aria2 (`--enable-rpc…`, o que o ADM usa para downloads comuns) não existe `--upnp`.**
`Cargo.lock` ganhou só `igd-next 0.18.0`, `xml-rs`, `xmltree`.

#### 1.1.4 Otimizações de velocidade **descartadas** (nunca commitadas)
Gravação em segundo plano (um bloco em voo por conexão) e faixas de 64 MiB. Medidas num servidor local
(90 ms de latência por requisição, 115 MB/s compartilhados): empate dentro do ruído (≈107–109 MB/s os dois).
Sem ganho medido, foi revertido. A medição **no link real do usuário** é o que decide (seção 4).

### 1.2 ADM — `claude/adm-xtreme-tools` (já incluída em `claude/ed2k-share`)

- **Slot do motor de downloads**: continua com ids/comandos `aria2` (RPC compatível), mas o repositório
  padrão é `linuxhell/nibiru-xtreme`; assets `nibiru-xtreme-<versão>-<windows-x64.exe|linux-x64|macos-x64>`
  + `.sha256`; pasta `tools/nibiru-xtreme/`. **Baixar** instala e preenche o caminho; **Atualizar** sem motor
  baixa para a pasta de ferramentas; se o slot aponta para um `aria2c`, instala o Xtreme ao lado **sem
  tocar** no `aria2c`; senão consulta a release **antes** e só baixa se a tag difere da versão instalada.
  Releases só-pré-lançamento são achadas pela lista (o `/releases/latest` do GitHub as ignora).
- **Janela de download**: slider de threads **1–16, padrão 8** (era 1–32/16); `MAX_DOWNLOAD_CONNECTIONS = 16`.
  Caixa **"Memorizar a quantidade de threads para este site"** (3 idiomas): marcada, ao Baixar grava na regra
  do **host exato** (`set_site_threads`/`get_site_threads`, `apply_site_threads` pura e testada); marcada com
  outro valor atualiza; desmarcada apaga **só** a contagem (velocidade/UA/credenciais da regra ficam; a
  regra some se só tinha a contagem). Downloads sem a janela (extensão) usam o valor memorizado. Pixeldrain
  segue fixo em 1 (controles travados). **A regra especial da Microsoft não força mais ≥16**: sem escolha
  usa 16; com escolha respeita.
- O teste `obsolete-engine-residue` proíbe o termo "ED2K" no código do produto da base `main`: o rótulo
  da aba Ferramentas **não** cita ED2K.
- Documento: `docs/INTEGRACAO-NIBIRU-XTREME.md`. Testes: `tests/site-threads-memory.test.cjs` (3) e 5 de Rust
  (versão/slot do motor, threads por site, prioridade da escolha); o de migração do repositório foi ajustado.

### 1.3 ADM — `claude/ed2k-share` (janela ED2K/Kad)

- **Aba "Compartilhados"** (como a do aMule): adicionar pasta (com/sem subpastas) e arquivos; prioridade por
  arquivo (muito baixa…lançamento); pedidos/enviado por arquivo; copiar link; "Parar de compartilhar" (arquivo
  avulso é esquecido; arquivo vindo de pasta é **excluído**); limites (vagas 1–20, KiB/s); fila de uploads.
  **Nada é compartilhado sem ação do usuário.** Nunca: ocultos (componente que começa com `.`), vazios,
  symlinks, nada dentro da pasta de dados do ADM; pasta que contém os dados do ADM ou raiz de disco é
  **recusada**. Persistência em `shares.json`, cache de hash em `share-cache.json`. Publicação ao servidor
  (`OP_OFFERFILES`) ao conectar e a cada mudança. Código: `ed2k_ui/shares.rs`.
- **Kad real** (`ed2k_ui/network.rs`): "Conectar Kad" inicia `KadSession` na porta TCP real do listener
  (UDP 4672 se livre); contatos em `kad-contacts.json`. Fontes de download e "Consultar fontes" vêm do
  servidor **e** do Kad; download funciona com só um dos dois; callbacks Low-ID por servidor, por buddy Kad ou
  ambos (`downloads.rs`). O comando antigo `ed2k_bootstrap` (sondagem) foi **removido**.
- **UPnP** no ADM (reusa o módulo do Xtreme): ligado por padrão (Preferências → Rede); encaminha TCP de
  entrada e UDP do Kad **só enquanto conectado**; remove ao desconectar, ao desligar e **ao sair** (gancho
  `RunEvent::Exit` em `main.rs`, `release_on_exit`). Estado na barra.
- **Ofuscação**: desligada/preferida (padrão)/obrigatória (servidor, peers, listener, Kad UDP).
  `prefs.json`.
- Dependência: `nibiru-xtreme` fixado em **`f8f65b2…`** com `features = ["upnp"]`
  (`apps/desktop/src-tauri/Cargo.toml` + `Cargo.lock`; ao mudar o Xtreme: publicar na branch de auditoria,
  trocar o `rev` e rodar `cargo update -p nibiru-xtreme --precise <sha>`).
- Documento: `docs/ed2k-kad-native.md` (seções "Shared files" e "Network"). Testes: 10 novos de Rust
  (6 em `shares.rs`, incluindo servir um arquivo escolhido a um cliente ED2K real e recusar um não escolhido;
  4 em `network.rs`: quais portas o UPnP encaminha, preferências, Kad iniciando e parando na porta do
  listener, Kad sem lista de nós) e 4 novos em `tests/ed2k-window.test.cjs`.

### 1.4 O merge das duas branches do ADM (cuidado)
Único conflito: `apps/desktop/src-tauri/src/main.rs`. O ChatGPT (commit `2872f373`, "Repair aria2 packaging
source…") tinha **revertido o padrão para `FerroDownload/aria2-static-builds`** e criado
`migrate_aria2_release_repo`. Resolvi assim: **o Xtreme é o padrão**; `FerroDownload/aria2-static-builds` e o
extinto `linuxhell/aria2-ultra` migram para `linuxhell/nibiru-xtreme`; fonte personalizada é mantida; o teste
`aria2_release_source_migrates_deleted_default_and_preserves_custom_sources` foi atualizado. **O empacotamento
NÃO mudou**: `release.yml`, `test-build.yml` e `validate-portable.yml` ainda baixam o `aria2c` do
FerroDownload (o teste `tests/aria2-release-source.test.cjs` exige isso e passa). Se o ChatGPT preferir manter
Ferro como padrão, **reverta só o trecho do slot, não o resto**, e atualize o teste Rust de novo.

---

## 2. Estado da CI (confirmar, não presumir)

- **Xtreme**: `f8f65b2` — rodei localmente `cargo fmt`, `clippy --all-targets --all-features -D warnings`, e a
  suíte inteira (todas as suítes `ok`); a CI desse commit **não foi observada por mim**. Falha **intermitente**
  já vista: `tests/interop_amule.py` (`ED2K peer read timed out`) em `692b98a`; re-execução passou.
- **ADM**: `test-build.yml` (`workflow_dispatch`) disparado para `claude/ed2k-share`; o último disparo foi no
  commit `2b496395` (merge). **Resultado não observado até a escrita deste arquivo.** Esse workflow roda
  `cargo fmt --check`, `cargo update -p zip --precise 2.4.2`, `cargo test --workspace --locked`, build release,
  `--verify-media-routing` e empacota o portátil do Windows (artefato `apocalipse-download-manager-windows-x64-portable`,
  precisa de login no GitHub para baixar). Tenho um lembrete agendado para conferir; **se você assumir antes, confira
  e responda ao usuário com o link do artefato.**
- Regra: **acompanhar a CI até o fim**; se falhar, ler os logs dos jobs que falharam, achar a causa real, corrigir,
  repetir localmente, só então enviar. Um push que deixa a CI vermelha custa um ciclo.

---

## 3. O que falta (ordem sugerida)

### 3.1 Imediato
1. Conferir o `test-build.yml` do ADM em `claude/ed2k-share`; corrigir o que falhar (Windows é o importante);
   dar ao usuário o link do artefato. **Riscos de Windows que eu não pude testar** (Linux aqui):
   - caminhos compartilhados saem de `canonicalize` com prefixo `\\?\C:\…` (funciona, mas **aparece assim na
     UI** e fica assim no `shares.json`). Planejei um `simplify()` aplicado a tudo que é guardado/comparado;
     **não implementei**. Atenção: `scan_directory` do Xtreme canonicaliza de novo, então os caminhos que
     voltam têm o prefixo — simplificar **todos** os pontos de comparação (`wanted_files`, `known`/`keep`,
     `eligible`, `ed2k_share_remove_*`, `overview`);
   - "oculto" aqui é só nome começando com ponto; o atributo oculto do Windows **não** é verificado;
   - `rfd` (seletores de pasta/arquivos) e a janela Tauri nunca foram abertos.
2. Publicar a `0.2.0-alpha.4` do Xtreme **somente com autorização do usuário** e CI/dry-run verdes.

### 3.2 Xtreme (o que ainda não existe — não anunciar como pronto)
Metalink; busca por palavra-chave e **publicação** no Kad; créditos e identificação segura do ED2K;
ofuscação cliente↔servidor do ED2K; NAT-PMP/PCP; UPnP no modo compatível com aria2; **teste com roteador real**;
CI Windows/macOS com IPv6; desempenho contra o aria2-ultra no mesmo link; limite configurável de conexões
>16 (só para `download`/`diagnose`, **não** mudar o padrão do motor nem o RPC); explicar (ou provar que é
artefato de medida) a alternância 85/135 MB/s do motor (seção 4.3); corrigir o veredito do `diagnose`
(1.1.2). Itens históricos ainda abertos em `docs/CAPABILITY-MATRIX.md`.

### 3.3 ADM
- **Teste ponta a ponta no app Tauri real**: Baixar/Atualizar o Xtreme pela aba Ferramentas **com uma release
  real** (existe a `v0.2.0-alpha.3`; nunca rodou), download HTTP comum, pausa/retomada pelo RPC, magnet →
  seleção de arquivos, streaming, fechar o ADM e conferir que o motor encerra, caixa de threads por site,
  janela ED2K (compartilhar, Kad, UPnP, ofuscação).
- **Empacotamento ainda embute o `aria2c`** (decisão pendente do usuário: trocar para embutir o Xtreme só
  depois de haver release estável; mudar os 3 workflows e o teste `aria2-release-source`).
- ED2K da janela usa a **biblioteca embutida**, não o RPC (`addUri ed2k://` existe no RPC do Xtreme). Decisão
  pendente: manter embutido ou passar a usar o RPC.
- Faltam: compartilhar automaticamente o que o usuário baixa (padrão do aMule; **sugestão: opção opt-in,
  desligada**), créditos, AICH, mensagens, busca por palavra-chave no Kad, UI do filtro de IP (o motor tem
  `ip_filter`), teste de firewall/porta alcançável, limites de download no ED2K.
- Extensão do navegador: **inalterada** (0.3.206); nunca testada com o Xtreme como motor.
- `--session-file` do Xtreme não é usado pelo ADM (ele recria tarefas com GID obsoleto — não testado com o Xtreme).

---

## 4. Resultados reais do usuário no Windows (única validação real) — para você usar, não repetir

Link: ISO do Windows 11 pt-BR de 8,73 GB do CDN da Microsoft (**link assinado e temporário; não o inclua em
nada**). Referência do usuário: o IDM baixa essa ISO a ~**110 MB/s do começo ao fim**. Critério do usuário:
**≥110 MB/s sustentados é ótimo; acima disso estamos à frente do IDM.**

### 4.1 `download` direto (alpha.2)
Completou em 83,7 s: **104 MB/s** médios (16 conexões, HTTP/1.1, 367 requisições de faixa, picos ~120).
Soma do tempo por conexão: cabeçalhos ~2%, rede ~93%, gravação ~4%. Outra execução completa (também alpha.2):
75,6 s, **115 MB/s** (cabeçalhos 0,6%, rede ~95%, gravação ~4%). A variação entre execuções parece ser do
servidor/rota (a rede sem disco também variou), não do motor — mas é uma inferência, não foi isolada.

### 4.2 `diagnose` (alpha.3), mesma ISO, horários diferentes

| Rodada | 1 conexão | 8 | 16 | 32 | Motor |
|---|---|---|---|---|---|
| A (servidor rápido) | 106,0 | 116,0 | 115,9 | 115,8 | 16 → **114,8** média / 115,7 após rampa |
| B (servidor lento por conexão) | 45,6 | 93,5 | 109,5 | 114,4 | 16 → **102,4** média / 104,4 após rampa |
| C (servidor rápido) | 103,7 | 116,0 | 116,1 | 116,1 | 8 → **113,4** média / 114,2 após rampa |

(MB/s; colunas de conexões = rede sem gravar em disco, **após a rampa** de 2 s.) Disco: ~430 MB/s, nunca o gargalo. Conclusões **sustentadas pelos dados**:
com servidor rápido o motor fica em 98–100% do teto (8 e 16 iguais); com servidor que limita por conexão,
mais conexões ajudam (8 < 16 < 32 na rede). **Não sustentadas:** que 16 supere 8 no motor em servidor lento
(só medido na rede, uma rodada) e qualquer conclusão sobre 32 no motor (teto interno 16).
**Pendente:** o usuário prometeu o relatório com `--engine-connections 16` (a tentativa dele falhou por aspas
erradas no comando — o `--dir` ficou dentro da URL). Peça de novo.

### 4.3 Anomalia **não explicada**
A série "MB/s por segundo" do **motor** alterna ~85 e ~135 em todas as rodadas; a da **rede sem disco** é reta
(116). A média bate com o tempo total (8,73 GB / ~76 s ≈ 115 MB/s), então não há perda, mas a causa é
desconhecida (cadência do callback de progresso, escrita em blocos de 4 MiB em lote, agendamento de tarefas,
ou algo real). **Não concluir sem investigar**; uma investigação cega pode "consertar" o que é só medida.

---

## 5. Armadilhas aprendidas (economizam tempo)

- `rust-toolchain.toml` com `stable` **sobrescreve** o toolchain dos workflows (item 1.1, `9188245`).
- `igd-next` **recusa gateway em loopback** salvo `allowed_gateway_ips`; testes de UPnP precisam listar `127.0.0.1`.
- Em comandos Tauri, **não segurar `std::sync::MutexGuard` através de `.await`** (o future deixa de ser `Send`):
  ver `ed2k_overview`.
- O simulador de DOM de `tests/ed2k-window.test.cjs` **lança** em seletor desconhecido
  (`Unexpected selector`): ao usar um novo `querySelectorAll`, estenda o simulador.
- `assert.deepEqual` entre objetos do `vm` e do teste falha por reino diferente: normalize com `JSON.parse(JSON.stringify(..))`.
- O teste `obsolete-engine-residue` (base `main`) proíbe "ed2k" no código do produto; a branch do ChatGPT o
  ajustou. Ao mesclar `claude/ed2k-share` em `main` isso volta a importar.
- **Disco do ambiente encheu** durante a sessão (alvos de compilação de ~15 GB + 8 GB). Use um único
  `CARGO_TARGET_DIR`, limpe com `cargo clean -p <pacote>` e apague dados de benchmark. Sem isso, `No space left on device`.
- Falha intermitente da CI do Xtreme: interop com aMule (`ED2K peer read timed out`). Re-executar **uma vez**
  para confirmar; não tratar como regressão sem reprodução.
- `Playwright` no ambiente: `/opt/node-tools/node_modules/playwright` + `/opt/pw-browsers/chromium-1194/chrome-linux/chrome`.
  Dá para abrir `ed2k.html`/`index.html` com `window.__TAURI__.core.invoke` simulado e conferir a tela nos 3 idiomas.

---

## 6. Regras do usuário (valem sempre)

1. Terminar o que falta **sem declarar pronto** o que não foi provado; dizer exatamente o que falta e por quê.
2. **Nada de release, tag, PR, merge em `main` ou comentário externo sem autorização expressa.** Autorizados
   até aqui: pré-releases `alpha.2` e `alpha.3` do Xtreme; disparar `test-build.yml` para gerar artefato de teste.
3. Nunca compartilhar nada implicitamente no ED2K; caminhos canônicos, sem symlink, sem sobrescrever.
4. Textos de interface em **inglês, português do Brasil e chinês simplificado**.
5. Preservar downloads reais do usuário (inclusive `.part`, sidecars). **Não** apagar os `.nibiru-http-*.part`
   órfãos da alpha.2 por conta própria (o usuário pode apagar; estão na Área de Trabalho dele).
6. Não usar comandos globais destrutivos (`git clean -fdx`). Limpeza só de lixo comprovado.
7. O usuário não quer o Claude pedindo confirmação de rotina já autorizada, mas **quer ser avisado** do que não
   foi testado.

---

## 7. Como o usuário vai testar (roteiro para quando o artefato Windows existir)

1. Baixar e extrair `apocalipse-download-manager-windows-x64-portable` da execução do `test-build.yml`.
2. **Aba Ferramentas**: "Baixar" no slot do motor (deve trazer `nibiru-xtreme-0.2.0-alpha.3-windows-x64.exe`,
   conferir SHA-256, preencher o caminho); "Salvar"; "Atualizar" deve dizer "já está atualizado".
3. **Download HTTP** de um arquivo grande; slider de threads 1–16 (padrão 8); marcar "Memorizar…" e conferir que
   na próxima janela daquele site o valor e a marcação voltam; desmarcar + Baixar apaga.
4. **Pausa/retomada** de um download pelo ADM (a retomada via RPC é a parte nunca vista em uso real).
5. **Janela ED2K/Kad**: conectar ED2K; "Conectar Kad" (contatos verificados sobem); barra de UPnP
   (encaminhado/IP externo ou "roteador não encontrado" — **o primeiro teste com roteador real**); aba
   "Compartilhados": adicionar uma pasta de teste pequena, conferir lista, prioridade, "Parar de compartilhar",
   reiniciar o ADM e ver a escolha restaurada; baixar um link ED2K de teste; trocar a ofuscação.
6. **Fechar o ADM** e conferir no roteador (ou em `upnpc -l`) que os mapeamentos sumiram; conferir que o
   `nibiru-xtreme.exe` também terminou.
7. Enviar ao Claude/ChatGPT: relatório do `diagnose` (`nibiru-diagnose-report.txt`), logs e capturas de tela de
   qualquer erro.
