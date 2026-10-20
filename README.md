<p align="center"><img src="assets/branding/apocalipse-a-blue.svg" width="180" alt="Apocalipse Download Manager"></p>

# Apocalipse Download Manager

[Português do Brasil](README.md) · [English](README.en.md) · [简体中文](README.zh-CN.md)

Gerenciador de downloads portátil com interface Rust/Tauri, extensão de navegador, captura de mídia, torrents, transferências entre computadores e integração nativa com os motores Nibiru.

**Estado desta árvore: ADM `0.6.1`, extensão `0.3.206`, integração Nibiru/ED2K em desenvolvimento em 20/10/2026.** O número da aplicação não significa que todas as alterações desta branch já estejam publicadas. Este README descreve o código atual; versões publicadas têm suas próprias notas no [changelog](CHANGELOG.md) e em [Releases](https://github.com/linuxhell/apocalipse-download-manager/releases).

## Recursos atuais

| Área | Implementado nesta árvore |
| --- | --- |
| HTTP/HTTPS | Retomada com identidade do arquivo, segmentação, espelhos verificados, SHA-256, fila e concorrência adaptativa |
| Torrents/magnets | Nibiru Xtreme, seleção de arquivos, informações de peers/sessão e prévia de mídia |
| Navegador | Captura de vídeo, áudio e imagens; envio ao ADM; idiomas `pt-BR`, `en` e `zh-CN` |
| Extração de páginas | Nibiru Power para fontes suportadas e rotas yt-dlp para os demais fluxos existentes |
| HLS/DASH | Integração por capacidades com Nibiru M3U8; rotas legadas conforme o fluxo |
| Mídia local | Operações tipadas Colapse: inspeção, WAV, FLAC, QOA, QOIR experimental com PPM/PAM, Y4M e MP4 faststart |
| Gravações | Gravação progressiva e exportação pelos motores de mídia configurados |
| Apocalipse Link | Transferências autenticadas com TLS; QUIC é uma opção separada, desativada por padrão |
| ED2K/Kad | Janela independente, busca em servidor, fontes, partes verificadas, compartilhamento explícito e sessão Kad limitada |
| Interface | Janela principal de 980×620, modo claro/escuro, bandeja e troca de idioma durante a execução |

<p align="center"><img src="assets/screenshots/downloads.png" width="800" alt="Interface de downloads com dados de exemplo"></p>

As capturas ilustram a interface; não comprovam conectividade pública nem execução do checkpoint mais recente em cada sistema.

## Começar

1. Obtenha um pacote em [Releases](https://github.com/linuxhell/apocalipse-download-manager/releases) ou compile esta branch. Extraia o pacote inteiro para uma pasta gravável.
2. Execute o ADM. No Windows, mantenha as DLLs distribuídas junto do executável; a interface precisa do WebView2. No Linux, o desktop usa GTK/WebKit.
3. Em **Configurações → Ferramentas**, confira caminhos e capacidades. Os binários Nibiru usam subpastas em `data/tools/`; caminhos próprios também são aceitos.
4. Instale a extensão correspondente para Chrome, Edge ou Firefox. O manifesto desta árvore é `0.3.206`; gerar um XPI localmente não lhe dá assinatura Mozilla automaticamente.
5. Escolha `Português do Brasil`, `English` ou `简体中文`. A extensão e a janela ED2K acompanham o idioma configurado.

`data/`, ao lado do executável, guarda configurações, fila, logs e ferramentas. Senhas e tokens usam o cofre de credenciais do sistema; copiar a pasta portátil não garante a migração desses segredos. Use somente conteúdo que você tenha autorização para baixar.

## Motores e integração

| Motor | Função no ADM | Contrato atual |
| --- | --- | --- |
| [Nibiru Xtreme](https://github.com/linuxhell/nibiru-xtreme) | Transferências/torrents e biblioteca ED2K embutida | RPC nativo `nibiru.*`, identidade e capacidades verificadas |
| [Nibiru Power](https://github.com/linuxhell/nibiru-power) | Descoberta de mídia em páginas suportadas | `extract URL --json --timeout 30`; supervisor ADM com prazo de 40 s |
| [Nibiru M3U8](https://github.com/linuxhell/nibiru-m3u8) | Playlists e segmentos suportados | CLI nativa, capacidades e eventos JSONL |
| [Nibiru Colapse](https://github.com/linuxhell/nibiru-colapse) | Arquivos de mídia locais | Operações tipadas; JSON com `schema_version=1`, `engine`, `operation`, `result` |

Os quatro executáveis respondem a `capabilities --json`. O ADM recusa identidade/schema incompatível. Power não recebe comandos de yt-dlp e Colapse não recebe comandos de FFmpeg. FFmpeg, FFprobe, yt-dlp e QuickJS ainda atendem rotas não cobertas pelos motores nativos; confira os binários incluídos no seu pacote.

As chaves antigas `aria2_rpc_*` são aceitas para migração; a integração atual inicia Xtreme e usa RPC `nibiru.*`. Revisões dos executáveis estão em [scripts/nibiru-engines.json](scripts/nibiru-engines.json). A biblioteca ED2K tem seu próprio pin no [Cargo.toml do desktop](apps/desktop/src-tauri/Cargo.toml) e `Cargo.lock`; commits posteriores contendo apenas documentação não mudam o código já fixado.

## Parâmetros atuais

### Aplicação e CLI

| Comando | Efeito |
| --- | --- |
| `apocalipse-desktop` | Abre a aplicação |
| `apocalipse-desktop --hidden` | Inicia com a janela principal oculta |
| `apocalipse-desktop --open URL` | Abre uma fonte reconhecida; usado também pelas associações do sistema |
| `apocalipse-desktop --verify-media-routing REPORT.json` | Diagnóstico da CI; grava o relatório e encerra antes de abrir a interface |
| `apocalipse-cli URL DESTINO` | Download pelo core Rust, solicita 8 conexões e preserva destinos existentes |
| `apocalipse-cli benchmark URL DESTINO [CONEXOES] [ESPELHO ...]` | Padrão 8 conexões, limite 1–32, adaptação ativa e resumo JSON |

São nomes dos alvos Cargo; o pacote pode renomear o desktop. No Windows, acrescente `.exe`. A CLI é uma ferramenta separada do core e não controla todas as funções da interface. Não expõe flags para cabeçalhos, checksum ou sobrescrita.

### Configurações da interface

| Campo em `settings.json` | Padrão e interpretação |
| --- | --- |
| `max_active_downloads` | `3`; ajustável de 1 a 20 tarefas |
| `connections_per_download` | `16`; limite solicitado de 1 a 32; streams multiplexados não equivalem necessariamente a conexões TCP |
| `adaptive_efficiency` | `true`; adapta a concorrência conforme motor/transferência |
| `global_bandwidth_limit` | `0` sem limite; bytes/s, máximo de 10 GiB/s |
| `download_directory` | Inicialmente `null`; diretório escolhido na interface |
| `xtreme_path`, `power_path`, `m3u8_path`, `colapse_path` | Opcionais; padrão `data/tools/NOME/NOME[.exe]` |
| `xtreme_rpc_enabled`, `xtreme_rpc_auto_start` | `true`; porta automática quando `xtreme_rpc_port` é `null` |
| `ffmpeg_path`, `yt_dlp_path`, `qjs_path`, `n_m3u8dl_re_path` | Caminhos opcionais para as rotas correspondentes |
| `proxy_enabled`, `proxy_url`, `dns_enabled`, `dns_servers` | Proxy/DNS configuráveis; habilitação inicia em `false` |
| `language`, `dark_mode` | Idiomas `en`, `pt-BR`, `zh-CN`; padrões `en` e `false` |
| `link_quic_enabled` | `false`; opção experimental independente do Link TLS |

Prefira a interface para mudar configurações. O bridge escuta em `127.0.0.1:17654`; a porta-base do Link é `17655`. Esses números não são flags CLI. Segredos não devem aparecer nos exemplos de configuração.

### Mídia local

O adaptador Nibiru aceita 13 operações tipadas, das quais 12 aparecem no painel. A concatenação está disponível pelo contrato, sem seletor múltiplo na interface atual. Caminhos devem ser absolutos; entradas precisam existir e saídas precisam ser novas. Campos desconhecidos ou pertencentes a outra operação são rejeitados.

| Operação | Campos do pedido |
| --- | --- |
| `inspect` | `input` |
| `wav-cut`, `y4m-cut` | `input`, `output`, `start`, `frames` |
| `wav-gain` | `input`, `output`, `db` |
| `wav-normalize` | `input`, `output`, `peak_db` |
| `wav-concat` | `inputs`, `output` |
| `flac-encode`, `flac-decode`, `qoa-encode`, `qoa-decode` | `input`, `output` |
| `qoir-encode` | `input`, `output`; `input_color` opcional: `bt709` ou `srgb` |
| `qoir-decode` | `input`, `output` |
| `mp4-faststart` | `input`, `output` |

`start`/`frames` contam sample frames em WAV e frames de vídeo em Y4M. Ganho usa dB; normalização é de pico em dBFS, sem LUFS. FLAC é sem perdas nos perfis inteiros suportados. QOA usa PCM16 com perdas e não contém checksum; não equivale a uma cópia exata do áudio original. Inspeção MP4 não decodifica os codecs identificados. Consulte capacidades e limites no README do Colapse.

QOIR é um formato de imagem experimental, diferente de QOI. A codificação aceita uma imagem P6 PPM ou P7 PAM, RGB/RGBA de 8 bits, e produz QOIR sem perdas; a decodificação exporta PAM e também aceita os níveis com perdas do formato. O limite é 67.108.864 pixels. A cor padrão de entrada é BT.709, salvo comentário de cor reconhecido ou `input_color` explícito; selecionar sRGB não converte os pixels. A exportação PAM recusa alpha pré-multiplicado e metadados que não consegue representar, incluindo perfis ICC. QOIR não possui checksum obrigatório. O painel identifica o formato como experimental e mostra a escolha de cor somente na codificação.

No contrato ADM, cortes exigem `frames >= 1` e soma `start + frames` sem overflow; ganho fica entre −120 e +120 dB, e normalização pelo ADM aceita de −120 a 0 dBFS, combinando o limite do adaptador com o teto do Colapse. Concatenação aceita 1–256 entradas. O supervisor de mídia tem prazo de seis horas. Esses limites do adaptador são adicionais aos limites do core Colapse.

## ED2K/Kad: estado real

Há login/busca em servidor, fontes, downloads diretos com verificação MD4/partes, retomada, compartilhamento escolhido pelo usuário, fila de uploads e publicação da lista no servidor. Kad usa HELLO, desafio de endpoint, consultas iterativas e fontes diretas. O [documento ED2K/Kad](docs/ed2k-kad-native.md) detalha limites de pacotes, contatos, filas e cancelamento.

Dois downloads ED2K podem trabalhar em paralelo; cada transferência usa um peer por vez e tenta até 16 fontes diretas por tentativa. Uploads permitem 1–64 slots e limite total em KiB/s. O listener TCP prefere 4662, usando outra porta se ocupada. O ID atribuído pelo servidor é a referência: uma porta local aberta não comprova acesso através de NAT/firewall.

Kad tem até 512 contatos, 32 consultas pendentes, três segundos por consulta e provas de endpoint com validade de 20 minutos. Fontes diretas podem alimentar a fila sem servidor ED2K. Busca/publicação por palavras-chave Kad, créditos, AICH, ofuscação e paridade completa com aMule continuam pendentes. Testes locais entre componentes próprios não comprovam interoperabilidade pública com eMule/aMule.

## Compilar, testar e empacotar

Use Rust estável e execute na raiz para carregar `.cargo/config.toml`, que habilita a API HTTP/3 de reqwest. A CI JavaScript usa Node 22; scripts de build usam Python 3.11 ou posterior.

```sh
cargo test --locked -p apocalipse-core
cargo build --locked --release -p apocalipse-cli
cargo build --locked --release -p apocalipse-desktop
node --test tests/*.test.cjs
python3 scripts/build_nibiru_engines.py --output dist/nibiru-tools --require-all
```

Em Debian/Ubuntu, o desktop usa `build-essential`, `pkg-config`, `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `libpam0g-dev`, `libssl-dev`. A [matriz de CI](.github/workflows/validate-portable.yml) cobre os comandos Fedora/Arch e jobs Windows/macOS. Compilar só core/CLI não exige GTK/WebKit.

| Opção de `build_nibiru_engines.py` | Comportamento |
| --- | --- |
| `--output PASTA` | Obrigatória; saída dos motores e de `build.json` |
| `--require-all` | Falha se qualquer revisão necessária estiver ausente |
| `--source-root PASTA` | Usa clones Git locais `PASTA/nibiru-*`, sempre na revisão fixada; aceita commits ainda não publicados |

O script compila para o sistema onde é executado, verifica versão/capacidades, copia avisos de licença e registra SHA-256. Não é um cross-compiler. Pins ainda não publicados exigem clones locais ou publicação prévia dos commits.

### Validação e limites

O checkpoint atual passou nos 341 testes JavaScript de 44 arquivos, executados diretamente com Node e contagem TAP, e nos 33 testes comuns do harness nativo. O teste separado executou as 13 operações tipadas com o Colapse real. Dentro da suíte JavaScript, quatro testes do painel em Node/VM conferem a mudança de idioma e 36 pedidos simulados nas três línguas, incluindo as escolhas de cor. Os três workflows passaram pela validação YAML, sintaxe dos blocos Bash e conferência do filtro de integração. O checkpoint anterior de auditoria também executou 52 testes do core e os quatro motores reais, e compilou o desktop para Windows GNU x64. São escopos distintos, sem soma de repetições; validação de sintaxe não equivale à execução dos jobs remotos.

O checkpoint da rodada 4 do Colapse, com QOIR e decoder FLAC paralelo, passou em 172 testes Rust (quatro testes externos/de corpus ignorados nessa invocação), 53 testes Python, formatação, Clippy e smoke das 13 operações/21 direções de codec. O mesmo binário Linux congelado foi usado no teste real do adaptador ADM. As [correções posteriores orientadas pela leitura do FFmpeg](https://github.com/linuxhell/nibiru-colapse/blob/main/docs/SOURCE_GUIDED_DEVELOPMENT_2026-10-20.md), as comparações independentes de áudio/pixels e a matriz de desempenho têm registros próprios no repositório do motor. Logs, comandos, hashes e limites vinculam cada execução ao binário testado; as contagens não somam repetições nem demonstram cobertura completa dos codecs. Os testes de interface em VM não comprovam o funcionamento do WebView no Windows.

Compilar Windows não equivale a executar sua interface. Execução nativa Windows, conectividade pública ED2K/Kad, empacotamento AppImage e auditoria semântica integral ainda exigem evidência específica. A [auditoria delimitada](docs/AUDITORIA-NIBIRU-ED2K-2026-10-20.md) registra a cobertura real. O instalador suporta manifestos assinados, mas a lista de chaves confiáveis está vazia; essa assinatura ainda não é exigida. Veja [atualizações de ferramentas](docs/tool-update-signing.md).

## Evolução e contribuição

Tecnologias novas são avaliadas pelo ganho mensurável em velocidade, integridade, qualidade, compatibilidade ou manutenção. HTTP/3, codecs nativos e paralelismo devem manter resultados reproduzíveis, fallbacks aplicáveis e documentação sincronizada. Experimentos não são apresentados como melhorias comprovadas. Consulte [desempenho](docs/PERFORMANCE.md), [roadmap](ROADMAP.md), [segurança](SECURITY.md) e [issues](https://github.com/linuxhell/apocalipse-download-manager/issues); documentos históricos identificam o snapshot a que se referem.

Você pode [apoiar o desenvolvimento pelo PayPal](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL).

## Licença

ADM usa [GPL-3.0-or-later](LICENSE). Motores e ferramentas mantêm suas próprias licenças e avisos; a licença do ADM não substitui esses termos.
