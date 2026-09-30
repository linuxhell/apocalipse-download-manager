# Apocalipse Download Manager 0.4.77 — extensão / extension / 扩展 0.3.187

## Português do Brasil

> [!IMPORTANT]
> **Baixe o novo Apocalipse Download Manager 0.4.77, instale a extensão 0.3.187 e atualize o aria2 em Ferramentas para usufruir dos benefícios.**

Players do Reddit em Shadow DOM aberto são encontrados; os nomes usam o título do post e downloads de posts são encaminhados ao yt-dlp. As instruções de prévia do TikTok permanecem visíveis após fechar o painel. Corrigidas a detecção de arquivos compactados e a limpeza de partes temporárias. Cliques e fases dos metadados magnet têm diagnóstico compacto. Pacotes Windows, Linux tar.gz/AppImage e macOS são x64; Firefox inclui o XPI assinado pela Mozilla fornecido pelo mantenedor.

### Desempenho

Medição real relatada, usando o mesmo arquivo ISO de 8,17 GB em todos os testes:

| Engine | Configuração | Tempo total | Velocidade média |
| --- | --- | --- | --- |
| aria2 clássico | Padrão antigo (`-x 1`, uma conexão) | 103 s | ~79,3 MB/s |
| aria2-next | `--stream-max-connections=16 --file-allocation=trunc` | 88,98 s | ~91,8 MB/s |
| **aria2-ultra** | `-s16 -x16 --file-allocation=trunc` (**padrão atual, nada a configurar**) | **71 s** | **~115,1 MB/s** |

**Comparado ao aria2 clássico:** uma única conexão fica à mercê da variação de throughput. Neste teste, houve uma queda de ~100 MB/s para ~62 MB/s entre 80–88% do download, enquanto o aria2-ultra manteve, segundo o relato, uma taxa próxima de ~117,7 MB/s do início ao fim. Os tempos totais correspondem a **~31% menos tempo e ~45% mais velocidade média**. Os defaults do fork mudaram para que ele já comece com a configuração que teve melhor desempenho nessa medição, sem exigir que cada usuário descubra e passe as flags manualmente.

**Comparado ao aria2-next:** com 16 conexões e alocação `trunc` nos dois lados, o aria2-ultra levou **~20% menos tempo** (~25% mais velocidade média). O aria2-next usa libcurl para HTTP e libtorrent-rasterbar para BitTorrent; o aria2-ultra mantém o motor do aria2 clássico e adiciona suporte nativo a BitTorrent v2/híbrido. Evitar a substituição das duas stacks é a justificativa arquitetural do fork, mas este teste de tempo, sozinho, não comprova que a sobrecarga das bibliotecas causou a diferença nem que v2/híbrido seja a única vantagem do aria2-next.

Os números descrevem este teste, sem garantir a mesma velocidade em qualquer servidor ou rede. O relato original do fork informa 47%; os tempos e velocidades arredondados da tabela resultam em aproximadamente 45%. Fontes: [medição do aria2-ultra](https://github.com/linuxhell/aria2-ultra#desempenho) e [arquitetura do aria2-next](https://github.com/AnInsomniacy/aria2-next).

## English

> [!IMPORTANT]
> **Download the new Apocalipse Download Manager 0.4.77, install browser extension 0.3.187, and update aria2 in Tools to benefit from these improvements.**

Reddit players inside open Shadow DOM are discovered; media names use the post title, and Reddit post downloads use yt-dlp. TikTok preview instructions remain visible after closing the popup. Archive detection and temporary download-part cleanup were corrected. Click handoffs and magnet metadata phases have compact diagnostics. Windows, Linux tar.gz/AppImage and macOS packages are x64; Firefox includes the supplied Mozilla-signed XPI.

### Performance

Reported real measurement, using the same 8.17 GB ISO in every test:

| Engine | Configuration | Total time | Average speed |
| --- | --- | --- | --- |
| Classic aria2 | Old default (`-x 1`, one connection) | 103 s | ~79.3 MB/s |
| aria2-next | `--stream-max-connections=16 --file-allocation=trunc` | 88.98 s | ~91.8 MB/s |
| **aria2-ultra** | `-s16 -x16 --file-allocation=trunc` (**current default; no configuration needed**) | **71 s** | **~115.1 MB/s** |

**Compared with classic aria2:** a single connection is exposed to throughput fluctuations. In this test it dropped from ~100 MB/s to ~62 MB/s at 80–88% completion, while aria2-ultra reportedly stayed near ~117.7 MB/s from start to finish. The total times correspond to **~31% less time and ~45% higher average throughput**. The fork changes its defaults so users start with the configuration that performed better in this measurement, instead of discovering and supplying the flags manually.

**Compared with aria2-next:** with 16 connections and `trunc` allocation on both sides, aria2-ultra took **~20% less time** (~25% higher average throughput). aria2-next uses libcurl for HTTP and libtorrent-rasterbar for BitTorrent; aria2-ultra retains classic aria2's engine and adds native BitTorrent v2/hybrid support. Avoiding the replacement of both stacks is the fork's architectural rationale, but this timing test alone does not establish library overhead as the cause or prove that v2/hybrid is aria2-next's only advantage.

These figures describe this particular test, not a guaranteed speed on every server or network. The original fork report states 47%; the rounded times and average speeds above yield approximately 45%. Sources: [aria2-ultra benchmark](https://github.com/linuxhell/aria2-ultra#desempenho) and [aria2-next architecture](https://github.com/AnInsomniacy/aria2-next).

## 简体中文

> [!IMPORTANT]
> **请下载新版 Apocalipse Download Manager 0.4.77，安装浏览器扩展 0.3.187，并在“工具”中更新 aria2，以享受这些改进。**

可发现开放 Shadow DOM 内的 Reddit 播放器；媒体名称使用帖子标题，帖子下载交由 yt-dlp。TikTok 预览提示在弹窗关闭后仍可见。修复压缩文件识别及临时下载分块清理。点击交接和磁力链接元数据阶段提供精简诊断。Windows、Linux tar.gz/AppImage 和 macOS 软件包均为 x64；Firefox 包含维护者提供的 Mozilla 签名 XPI。

### 性能

以下为项目报告的实际测量；所有测试均使用同一个 8.17 GB ISO 文件：

| 引擎 | 配置 | 总耗时 | 平均速度 |
| --- | --- | --- | --- |
| 经典 aria2 | 旧默认值（`-x 1`，单连接） | 103 秒 | ~79.3 MB/s |
| aria2-next | `--stream-max-connections=16 --file-allocation=trunc` | 88.98 秒 | ~91.8 MB/s |
| **aria2-ultra** | `-s16 -x16 --file-allocation=trunc`（**当前默认值，无需额外配置**） | **71 秒** | **~115.1 MB/s** |

**与经典 aria2 相比：**单连接更容易受到吞吐量波动影响。本次测试在下载进度 80–88% 时，从约 100 MB/s 降至约 62 MB/s；据报告，aria2-ultra 从开始到结束保持在约 117.7 MB/s。总耗时对应**约 31% 的时间缩短和约 45% 的平均速度提升**。此分支调整了默认值，让用户直接使用本次测量中更快的配置，无需自行寻找并手动输入参数。

**与 aria2-next 相比：**双方均使用 16 个连接及 `trunc` 文件分配时，aria2-ultra **耗时减少约 20%**（平均速度提高约 25%）。aria2-next 的 HTTP 使用 libcurl，BitTorrent 使用 libtorrent-rasterbar；aria2-ultra 保留经典 aria2 引擎，并加入原生 BitTorrent v2/混合种子支持。避免替换两套协议栈是此分支的架构思路，但单次耗时测试不能证明差异由库的开销造成，也不能证明 v2/混合种子是 aria2-next 的唯一优势。

这些数字仅描述本次测试，不保证所有服务器或网络都有相同速度。分支原报告写为 47%；上述四舍五入的耗时和平均速度计算得出约 45%。来源：[aria2-ultra 测量报告](https://github.com/linuxhell/aria2-ultra#desempenho)及 [aria2-next 架构](https://github.com/AnInsomniacy/aria2-next)。

