# Performance notes / Notas de desempenho / 性能说明

This page keeps benchmark details out of the project homepage while preserving the measurements and their limitations.

Esta página mantém os detalhes de benchmark fora da página inicial do projeto, preservando as medições e suas limitações.

本页将性能测试细节移出项目首页，同时保留测量结果及其适用范围说明。

## Reported 8.17 GB ISO measurement

| Engine | Configuration | Total time | Average speed |
| --- | --- | ---: | ---: |
| Classic aria2 | Old default (`-x 1`, one connection) | 103 s | ~79.3 MB/s |
| aria2-next | `--stream-max-connections=16 --file-allocation=trunc` | 88.98 s | ~91.8 MB/s |
| **aria2-ultra** | `-s16 -x16 --file-allocation=trunc` | **71 s** | **~115.1 MB/s** |

### English

The same 8.17 GB ISO was used in the reported runs. Compared with classic aria2, the rounded totals correspond to about 31% less elapsed time and about 45% higher average throughput. Compared with aria2-next at 16 connections and `trunc` allocation, the reported run took about 20% less time (about 25% higher average throughput).

These numbers describe one reported test and are not a guaranteed speed on every server, ISP or machine. The timing alone does not establish the architectural cause of the difference. Reproducible comparisons should use identical connection counts, the same source file and server, verified output hashes, and raw timing logs.

Sources: [aria2-ultra benchmark](https://github.com/linuxhell/aria2-ultra#desempenho) and [aria2-next architecture](https://github.com/AnInsomniacy/aria2-next).

### Português do Brasil

A mesma ISO de 8,17 GB foi usada nas execuções relatadas. Em relação ao aria2 clássico, os totais arredondados correspondem a cerca de 31% menos tempo e aproximadamente 45% mais velocidade média. Em relação ao aria2-next com 16 conexões e alocação `trunc`, a execução relatada levou cerca de 20% menos tempo (aproximadamente 25% mais velocidade média).

Esses números descrevem um teste relatado e não garantem a mesma velocidade em todo servidor, provedor ou computador. O tempo medido, sozinho, não prova a causa arquitetural da diferença. Comparações reproduzíveis devem usar a mesma quantidade de conexões, o mesmo arquivo e servidor, hashes de saída verificados e logs brutos de tempo.

Fontes: [medição do aria2-ultra](https://github.com/linuxhell/aria2-ultra#desempenho) e [arquitetura do aria2-next](https://github.com/AnInsomniacy/aria2-next).

### 简体中文

报告中的测试均使用同一个 8.17 GB ISO 文件。与经典 aria2 相比，四舍五入后的结果约为耗时减少 31%、平均吞吐量提高 45%。与使用 16 个连接和 `trunc` 文件分配的 aria2-next 相比，报告中的运行耗时约减少 20%（平均吞吐量约提高 25%）。

这些数字只描述一次报告的测试，不代表所有服务器、网络或电脑都能达到相同速度。单次耗时测试也不能证明性能差异的架构原因。可复现的比较应使用相同连接数、相同文件和服务器、经过验证的输出哈希以及原始计时日志。

来源：[aria2-ultra 测量报告](https://github.com/linuxhell/aria2-ultra#desempenho) 和 [aria2-next 架构](https://github.com/AnInsomniacy/aria2-next)。
