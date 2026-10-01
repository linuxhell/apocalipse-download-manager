# ADM 0.4.79

> Firefox 0.3.189 in this release uses the maintainer-supplied Mozilla-signed XPI. SHA-256: `06c39c075b04a81fb2db53338a5a074d424c60e74beff92497b6d6d69e58c78e`.

## Português (Brasil)

- **Áudio direto:** a análise de MP3, M4A, AAC, OGG, WAV, FLAC, OPUS e áudio identificado pela extensão agora oferece conversão por FFmpeg para MP3, M4A, OPUS, FLAC ou WAV. O download original termina antes da conversão. O nome acompanha o formato escolhido; desativar a conversão restaura o nome original. Arquivos MP4 não recebem esse controle extra; as opções existentes do motor continuam disponíveis.
- **Apocalipse Link / QUIC (experimental):** nas opções QUIC da janela Link, ative nos dois computadores. Recebimento de arquivos compartilhados usa um certificado e token temporários negociados pela conexão TLS autenticada. É possível informar até três IPs locais adicionais; caminhos indisponíveis mantêm o caminho principal. QUIC indisponível recorre ao HTTPS. Envios continuam por HTTPS. Certificados, senha da conta e autorização dos compartilhamentos não são substituídos. Transferências verificam SHA-256 e respeitam pausa/cancelamento. Windows pode rejeitar origens loopback não atribuídas; isso não impede o recebimento pelo caminho principal.
- **Ferramentas / dicionário:** download explícito com dicionário RFC 9842 (`dcz`), mesma origem HTTPS, caminho exato e SHA-256 esperado. Limite de 256 MiB. Requer servidor compatível; não instala atualizações automaticamente e preserva arquivos existentes.
- **Gravações / MoQ (experimental):** capture objetos de relay, transmissão e faixa compatíveis, com duração de até uma hora e limite de 256 MiB. Parar salva somente objetos completos. Saída `.admmoq`, ainda sem exportação MP4/MKV e sem detecção automática de sites.
- Novos controles em português, inglês e chinês simplificado.
- **Diagnóstico e persistência:** falhas de escrita/rotação do log e de gravação da fila/configurações ficam visíveis no painel e no ZIP. Erros capturados da interface chegam aos dois logs com correlação. A extensão informa fila, descartes, erros de armazenamento/transporte e horários de envio/sincronização. Sanitização reforçada para JSON e campos sensíveis entre aspas.
- Extensão atualizada para **0.3.189**. Motor aria2-ultra ultra.2 incluído pelo empacotamento existente.

MARS permanece pesquisa: não há integração executável. Não há promessa de aceleração universal ou medição WAN dos novos transportes.

## English

Direct-audio analysis now offers post-download FFmpeg conversion to MP3/M4A/OPUS/FLAC/WAV. MP4 does not show the extra conversion control. Link has optional authenticated QUIC receives, additional local source IPs and HTTPS fallback; uploads retain HTTPS. Tools exposes bounded, SHA-256 verified RFC 9842 dictionary downloads for compatible same-origin HTTPS servers, without automatic update installation. Recordings exposes bounded MoQ object capture with stop support; `.admmoq` is an object archive, not playable MP4/MKV. MARS remains research. Logging and persistence health are exposed in the panel and ZIP; caught UI errors are correlated across logs. Extension telemetry exposes delivery/storage errors, queue counters and sync timestamps. JSON/quoted secret sanitization is hardened. Extension updated to 0.3.189. No universal speedup claim.

## 简体中文

直接音频分析新增下载完成后通过 FFmpeg 转换为 MP3/M4A/OPUS/FLAC/WAV。MP4 不显示额外转换控件。Link 支持可选的认证 QUIC 接收、额外本地源 IP 和 HTTPS 回退；上传继续使用 HTTPS。工具提供有大小限制及 SHA-256 验证的 RFC 9842 字典下载，需要兼容的同源 HTTPS 服务器，不自动安装更新。录制提供有时长/大小限制及停止功能的 MoQ 对象捕获；`.admmoq` 为对象存档，不是可播放的 MP4/MKV。MARS 仍为研究。面板和 ZIP 现显示日志及持久化健康状态，界面捕获的错误具有跨日志关联。扩展显示传输/存储错误、队列计数和同步时间。强化 JSON 及带引号敏感字段的脱敏。扩展更新至 0.3.189。不承诺普遍提速。
