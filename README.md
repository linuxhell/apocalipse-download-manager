<p align="center"><img src="assets/branding/apocalipse-a-blue.svg" width="160" alt="Apocalipse"></p>

[Português](README.md) · [English](README.en.md) · [简体中文](README.zh-CN.md)

# Apocalipse Download Manager

Gerenciador de downloads portátil em Rust/Tauri, com extensão para navegador.

## Estado do projeto

O código desta base é **0.6.1**, com extensão **0.3.206**. A auditoria do código está em andamento; a versão do fonte não significa que as alterações desta branch já tenham sido publicadas.

O ADM continua usando **aria2**, **yt-dlp nightly**, **N_m3u8DL-RE**, **FFmpeg**, **QuickJS**, extrator e player externos. O Nibiru Xtreme está em desenvolvimento separado: a substituição do aria2 pelo Xtreme depende da conclusão do motor e da validação da integração. Esta branch não integra Nibiru M3U8, Power ou Colapse.

## Instalação

Baixe os arquivos na [página de releases](https://github.com/linuxhell/apocalipse-download-manager/releases/latest). Confira a versão efetivamente publicada antes de escolher a extensão.

| Plataforma | Pacote |
| --- | --- |
| Windows x64 | ZIP portátil |
| Linux x64 | AppImage ou tar.gz portátil |
| macOS x64 | ZIP portátil |
| Chrome / Edge | ZIP da extensão |
| Firefox | XPI assinado pela Mozilla, quando fornecido na release |

Extraia o pacote, abra o aplicativo e configure as ferramentas em **Configurações → Ferramentas**. Instale a extensão correspondente se desejar enviar downloads do navegador. Um XPI de teste sem assinatura não equivale ao pacote assinado para Firefox estável.

## Recursos e limites

- HTTP/HTTPS com retomada, fila, limites de banda, mirrors e verificação SHA-256.
- Torrents e magnets via aria2, seleção de arquivos e pré-visualização pelo player configurado.
- Detecção de mídia na extensão e extração por yt-dlp; HLS pelo N_m3u8DL-RE e fluxos de áudio/conversão por FFmpeg.
- QuickJS como runtime JavaScript do yt-dlp; mantenha seu caminho configurado para sites que exigem desafios JavaScript.
- Gravações progressivas e exportação de mídia.
- Apocalipse Link: compartilhamentos escolhidos, TLS e autenticação por conta do sistema para acesso remoto. A descoberta de compartilhamentos SMB depende do sistema.
- Interface e extensão em português brasileiro, inglês e chinês simplificado.

A compatibilidade depende do site, das ferramentas instaladas e do sistema operacional. Protótipos de transporte não representam suporte universal ou validação em rede pública. A verificação de manifesto assinado para ferramentas depende de chaves confiáveis configuradas; veja [assinatura de atualizações](docs/tool-update-signing.md).

## Estrutura e desenvolvimento

| Caminho | Responsabilidade |
| --- | --- |
| `crates/apocalipse-core` | Modelos, classificação, transferências e políticas |
| `crates/apocalipse-cli` | Interface de linha de comando |
| `apps/desktop/src-tauri` | Desktop, integração de ferramentas, persistência e bridge |
| `apps/desktop/ui` | Interface do aplicativo e do Link |
| `browser-extension` | Captura e envio pelo navegador |
| `experiments/transport-lab` | Transportes experimentais; dependência utilizada pelo desktop |
| `tests` | Regressões JavaScript e contratos |

Requer Rust estável e Node.js. O desktop Linux também requer WebKitGTK 4.1, GTK/AppIndicator, librsvg, PAM e ferramentas de compilação; os workflows registram os pacotes por distribuição.

```sh
cargo fmt --all -- --check
cargo test --locked -p apocalipse-core -p apocalipse-cli
cargo test --workspace --locked
cargo build --release --locked -p apocalipse-desktop
node --test tests/*.test.cjs
```

Os testes do workspace completo precisam das dependências nativas do desktop. Compilar e passar testes não equivale à validação em uso real em Windows, Linux e macOS.

## CI e documentação

- `validate-portable.yml`: testes, compatibilidade Linux e pacotes candidatos nas três plataformas; não publica release.
- `transport-lab.yml`: regressões dos transportes usados pelo desktop.
- `release.yml`: compilação e publicação; não deve ser disparado para tratar uma auditoria como versão final.

Consulte [histórico](CHANGELOG.md), [roadmap](ROADMAP.md), [segurança](SECURITY.md) e [desempenho](docs/PERFORMANCE.md). Arquivos de downloads, parciais e checkpoints não são lixo de repositório.

## Apoio e licença

[Doar pelo PayPal](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL).

GPL-3.0-or-later; consulte [LICENSE](LICENSE).
