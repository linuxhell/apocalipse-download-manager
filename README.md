# Apocalipse Download Manager

[Português](README.md) · [English](README.en.md) · [简体中文](README.zh-CN.md)

Gerenciador de downloads livre e portátil para Windows, Linux e macOS x64.

[**Baixar**](https://github.com/linuxhell/apocalipse-download-manager/releases/latest)

| Plataforma | Pacote |
| --- | --- |
| Windows x64 | [Portable ZIP](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-download-manager-windows-x64-portable.zip) |
| Linux x64 | [Portable tar.gz](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-download-manager-linux-x64-portable.tar.gz) · [AppImage](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-download-manager-linux-x64.AppImage) |
| macOS x64 | [Portable ZIP](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-download-manager-macos-x64-portable.zip) |
| Chrome | [0.3.207 ZIP](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-chrome-0.3.207.zip) |
| Edge | [0.3.207 ZIP](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-edge-0.3.207.zip) |
| Firefox | [0.3.207 XPI](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-firefox-0.3.207.xpi) |

![Downloads](assets/screenshots/downloads.png)

O slider de threads fica visível na janela de novo download. Seu valor prevalece sobre os padrões do motor. A caixa logo abaixo salva imediatamente a preferência por site; mudar o slider atualiza o valor salvo e desmarcar remove essa preferência, preservando credenciais e outras configurações do site.

## Começar

1. Baixe o pacote portátil, extraia e abra o aplicativo. O pacote inclui Nibiru Xtreme, compilado da mesma revisão fixada usada pelo ADM.
2. Para integração com o navegador, instale a extensão correspondente. No Chrome/Edge, extraia o ZIP e use **Carregar sem compactação** no modo de desenvolvedor. No Firefox, abra o XPI assinado pela Mozilla.
3. Abra **Ferramentas** para instalar ou atualizar os motores necessários, como yt-dlp e FFmpeg. O AppImage não inclui o executável auxiliar Xtreme; instale-o nessa tela. No Linux, dê permissão de execução ao AppImage.

## Recursos

- HTTP/HTTPS com retomada, múltiplas conexões, espelhos e verificação SHA-256.
- Torrents, magnet e ED2K/Kad nativos, seleção de arquivos e prévia de vídeo.
- Captura de mídia pelo navegador, yt-dlp, HLS, FFmpeg e gravações progressivas.
- Apocalipse Link para transferências autenticadas com TLS entre computadores.
- Credenciais por site no cofre seguro do sistema; interface em português, inglês e chinês simplificado.

## Versão 0.8.3

Corrige nomes de filmes com espaços ou pontos finais no nome-base e mantém temporários de mídia em uma pasta própria, identificada por tarefa. A finalização seleciona somente a mídia produzida e preserva arquivos existentes. O número de conexões escolhido também vale para fragmentos do YouTube. Inclui correções de concorrência no cache de thumbnails e validação de torrents pelo Xtreme 0.2.2. Veja a [auditoria e seus limites](docs/AUDIT-2026-10-27.md).

Corrige a sobreposição do título com os controles de torrents na janela compacta. Os botões passam para a linha seguinte quando necessário, nos três idiomas.

A janela de download não usa mais números nas etapas e mostra até **12 pastas recentes**, com o caminho padrão primeiro. O release inclui a extensão Firefox **0.3.207 assinada pela Mozilla**, além de Chrome/Edge 0.3.207.

**Converter arquivo** está disponível na barra de downloads, na janela de adicionar download e nas ações dos downloads concluídos. Escolha um arquivo local ou use o arquivo já baixado. O FFprobe identifica o conteúdo e as trilhas reais, mesmo quando a extensão está errada. Capas embutidas não contam como vídeo. MP4 só com áudio oferece formatos de áudio; vídeo sem áudio oferece apenas formatos de vídeo. Quando há áudio e vídeo, todos os quadros são verificados antes de tratar um vídeo inteiramente preto como áudio; essa análise pode demorar em arquivos longos.

Formatos de saída: PNG, WebP sem perdas, JPEG, TIFF e BMP; FLAC, WAV, MP3, M4A, Opus e Ogg; MP4, MKV, MKV sem perdas e WebM. A lista depende dos codificadores disponíveis no FFmpeg instalado. Há seleção de trilha de áudio, progresso e cancelamento. A conversão preserva o original e salva outro arquivo sem sobrescrever arquivos existentes. Codificação sem perdas não recupera detalhes já perdidos; JPEG e os formatos com perdas usam alta qualidade. Imagens animadas reconhecidas são tratadas como vídeo para conservar os quadros.

Instale **FFmpeg com FFprobe** em **Configurações → Ferramentas**. A janela de download ocupa a área disponível da janela principal, com opções avançadas sempre visíveis e duas colunas quando há espaço. Miniaturas recebidas da extensão são preservadas; imagens inválidas tentam a alternativa do YouTube. Chrome/Edge: **0.3.207**; Firefox mantém o XPI assinado **0.3.207**.

A janela se amplia ao abrir **Adicionar download**, dando mais espaço ao formulário, ao controle de threads e às opções avançadas. Ao fechar, ela recupera o tamanho e a posição anteriores; em telas pequenas, a rolagem continua disponível.

O botão **Visualizar** do ED2K é habilitado após a primeira parte verificada por MD4: até **9.728.000 bytes**, cerca de 9,28 MiB. Não existe um limite fixo de 30–40 MB baixados. Os peers priorizam o começo e o final do arquivo antes do meio; MP4/AVI podem precisar do índice no final. Trechos ainda ausentes aguardam os dados.

Uma nova tentativa automática conserva a prévia já verificada durante a conferência desta execução. O resultado da nova conferência pode invalidar partes; reiniciar o aplicativo ou retomar manualmente exige nova verificação. Se nenhum peer tiver as partes restantes, o download continuará incompleto, preservando o parcial e tentando novamente.

Esta versão também corrige a liberação de bloqueios HTTP, a remoção da identidade de retomada ao excluir downloads comuns, o encerramento dos listeners do Xtreme e gravações simultâneas no cache criptografado. ADM **0.8.3**, Xtreme **0.2.2**, extensão **0.3.207**.

## Compilar e contribuir

Use o Rust fixado em `rust-toolchain.toml` e as dependências de sistema indicadas no [CI](.github/workflows/ci.yml). Execute `cargo test --workspace --locked` e `node --test tests/*.test.cjs`. O CI testa os três sistemas e o laboratório de transportes; verificações adicionais em Debian, Fedora e Arch podem ser ativadas manualmente. O workflow Release reutiliza os artefatos aprovados pelo CI.

Veja [CHANGELOG.md](CHANGELOG.md), [a revisão técnica](docs/AUDIT-2026-10-22.md), [desempenho](docs/PERFORMANCE.md) e [segurança](SECURITY.md). Após este fechamento, a manutenção seguirá os bugs encontrados no uso diário.

[PayPal](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL) · GPL-3.0-or-later.
