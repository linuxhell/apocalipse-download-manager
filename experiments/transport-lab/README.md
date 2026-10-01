# Desktop integration in ADM 0.4.79

**pt-BR:** Esta biblioteca agora também é dependência do desktop. O Link negocia recebimentos QUIC por tickets na conexão TLS autenticada, com progresso, cancelamento e fallback HTTPS. Ative nas duas máquinas. Caminhos adicionais são tentados e reportados; falhas mantêm o principal. Windows pode rejeitar o endereço loopback adicional não atribuído. Ferramentas e Gravações expõem download com dicionário e captura de objetos MoQ, respectivamente. MARS permanece pesquisa. Não há medição de agregação real entre interfaces nem exportação de vídeo MoQ.

**English:** The desktop now depends on this library. Link negotiates QUIC receives through authenticated TLS tickets, with progress/cancellation and HTTPS fallback. Enable it on both peers. Additional paths are attempted and reported; failures retain the primary path. Windows may reject an unassigned additional loopback source. Tools and Recordings expose dictionary downloads and MoQ object capture. MARS remains research; actual interface aggregation and playable MoQ export are unvalidated.

**简体中文：** 桌面现已依赖此库。Link 通过认证 TLS 票据协商 QUIC 接收，支持进度、取消和 HTTPS 回退。请在两个客户端启用。额外路径会被尝试并报告；失败时保留主路径。Windows 可能拒绝未分配的额外环回源。工具和录制提供字典下载及 MoQ 对象捕获。MARS 仍为研究；实际接口带宽聚合及可播放的 MoQ 导出尚未验证。

The CLI instructions below remain useful for standalone interoperability tests. Earlier isolation notes describe the initial prototype; the desktop integration scope above supersedes them.

---

# Laboratório de transportes do ADM / ADM transport lab / ADM 传输实验室

## Português (Brasil)

Este programa experimental é separado do desktop. Não adiciona opções inoperantes ao Apocalipse Link, não altera o atualizador e não anuncia aceleração sem medição. Requer Rust 1.91 ou posterior. O aria2-ultra é publicado separadamente; nenhuma dessas tecnologias está implementada no motor C++.

| Tecnologia | Implementação nesta etapa | Limite concreto |
|---|---|---|
| Multipath QUIC | Transferência autenticada de um arquivo selecionado, certificado confiado explicitamente, até três caminhos adicionais na mesma conexão, SHA-256 e saída sem sobrescrita | Teste real local com duas origens loopback; ainda falta medição com interfaces independentes, falha de caminho, Windows/macOS e integração da autorização/progresso do Link |
| RFC 9842 | Codec Zstandard `dcz`, cabeçalho/hash do dicionário, requisição HTTPS de artefato e dicionário da mesma origem, frescor, negociação e hash final obrigatório | Subconjunto: apenas `match` de caminho exato, sem `id` ou `match-dest`, até 16 MiB de dicionário e 256 MiB de artefato. Não anuncia `dcb`. Não há servidor de atualizações compatível implantado |
| MoQT | Cliente HTTPS/WebTransport de relay explícito; captura uma trilha anunciada em arquivo com limites de tempo/tamanho e fronteiras de objeto | Arquivo bruto `ADM-MOQ-OBJECTS-1`, não MP4/MKV. Não há extração de catalog/codecs, mux de áudio/vídeo ou integração na fila Gravações. Teste de objetos local, não interoperabilidade com relay público |
| MARS | Análise da arquitetura e plano de avaliação abaixo | Nenhum transporte MARS implementado ou ativado; requer forwarders cooperantes e controle de congestionamento próprios |

### Executar

```sh
cargo test --locked --manifest-path experiments/transport-lab/Cargo.toml --all-features
cargo build --locked --manifest-path experiments/transport-lab/Cargo.toml --features moqt
```

Use `apocalipse-transport-lab` (no Windows, `.exe`) gerado em `experiments/transport-lab/target/debug`.

- `serve <IP-local:porta> <arquivo-selecionado> <novo-certificado.der>`
- `get <IP-servidor:porta> <certificado-confiavel.der> <novo-arquivo> [IP-local-adicional ...]`
- `dictionary-get <URL-HTTPS-dicionario> <URL-HTTPS-artefato> <SHA256-esperado> <novo-arquivo>`
- `moqt-capture <URL-HTTPS-relay> <broadcast> <track> <novo-arquivo.capture> <segundos>`

`serve` e `get` exigem `ADM_LINK_LAB_TOKEN` com token aleatório de pelo menos 32 caracteres. Compartilhe certificado e token por canal confiável. O servidor disponibiliza exclusivamente o arquivo escolhido e aceita uma transferência; não recebe caminhos do cliente. Em `get`, os IPs adicionais precisam pertencer ao computador cliente e ter rota para o servidor. Vários IPs do mesmo adaptador não comprovam soma de banda. A distribuição de pacotes usa o agendador do noq; não promete divisão igual ou ganho de velocidade. A porta UDP precisa estar acessível; não há NAT traversal/relay neste experimento.

O formato de captura MoQ começa com `ADM-MOQ-OBJECTS-1\n`. Cada registro contém tamanho do JSON (u32 big-endian), JSON com grupo/objeto/timestamp/tamanho, seguido dos bytes do objeto. Ao terminar o tempo, o arquivo é truncado até o último objeto completo. A captura depende de uma origem que realmente publique MoQ e da versão negociada pela biblioteca, não apenas de um URL HTTP de vídeo.

### MARS: requisitos para uma implementação real

O artigo define descoberta de caminhos em camadas e controle de congestionamento acoplado entre consumidor e forwarders. Um round-robin entre IPs, múltiplas conexões HTTP ou um QUIC comum não implementa MARS.

1. Obter e revisar o protótipo dos autores e sua licença; não foi localizada uma biblioteca oficial pronta nas buscas desta etapa.
2. Criar uma topologia controlada com consumidor, origem e pelo menos dois forwarders cooperantes, incluindo gargalo compartilhado.
3. Implementar a descoberta em camadas, prevenção de ciclos e feedback de filas; fixar o formato do protocolo, autenticação e limites antes da exposição externa.
4. Medir tempo total, p95, bytes extras e justiça contra QUIC de caminho único e MPQUIC sob perdas, congestionamento e falha de intermediário.
5. Só integrar no Link se os resultados e os testes de segurança justificarem o custo. Sem essa infraestrutura não existe suporte MARS funcional para sites comuns.

### Critérios para integração no desktop

O Link precisa reutilizar sua autenticação e confiança existentes, negociar capacidades com ambos os clientes, preservar transferência TCP/TLS para clientes antigos, e expor progresso/pausa/cancelamento sem deixar arquivo parcial como concluído. O atualizador precisa de metadados assinados e origem que anuncie dicionários corretamente. Gravações precisa de catalog/codec e mux final, mantendo a captura bruta identificada como experimento até lá.

## English

An isolated experimental binary, not desktop integration or a speed claim. MPQUIC performs certificate-authenticated, token-protected file transfer with additional validated paths and SHA-256. RFC 9842 implements a bounded `dcz` codec and an exact-path, same-origin HTTPS fetch subset. MoQT captures objects from an explicit relay/track into a raw archive, not a playable movie. MARS remains research: cooperating forwarders, tiered discovery and coupled congestion control have not been implemented. Production Link, updater and recording queues retain their current transports.

Run the locked tests/build above. Supply a securely shared random `ADM_LINK_LAB_TOKEN` (32+ characters) for `serve`/`get`, and securely distribute the generated DER certificate. Additional local IPs must belong to independent usable client paths to investigate aggregate bandwidth. Local loopback validation does not establish WAN performance or Windows/macOS interoperability.

## 简体中文

这是独立实验程序，尚未集成到桌面版，也不保证速度提升。MPQUIC 支持可信证书、令牌认证、额外验证路径和 SHA-256 文件校验。RFC 9842 实现有大小限制的 `dcz` 编解码及同源 HTTPS 精确路径请求子集。MoQT 从明确指定的 relay/track 捕获原始对象，输出并非可播放视频。MARS 尚处研究阶段，需要协作转发节点、分层路径发现和耦合拥塞控制，当前未实现。现有 Link、更新器和录制队列保持原有传输方式。

按照上方命令构建和测试。`serve`/`get` 要求安全共享至少 32 个字符的随机 `ADM_LINK_LAB_TOKEN` 和 DER 证书。额外本地 IP 必须属于有效客户端路径。本地回环测试不能证明公网加速或 Windows/macOS 互操作性。

## Primary sources

- Multipath QUIC: https://datatracker.ietf.org/doc/draft-ietf-quic-multipath/
- noq implementation: https://github.com/n0-computer/noq
- MARS: https://arxiv.org/abs/2608.06101
- Compression Dictionary Transport: https://www.rfc-editor.org/rfc/rfc9842.html
- MoQ libraries: https://github.com/moq-dev/moq
