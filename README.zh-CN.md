<p align="center"><img src="assets/branding/apocalipse-a-blue.svg" width="180" alt="Apocalipse Download Manager"></p>

# Apocalipse Download Manager

[Português do Brasil](README.md) · [English](README.en.md) · [简体中文](README.zh-CN.md)

便携式下载管理器，采用 Rust/Tauri 界面，提供浏览器扩展、媒体捕获、种子下载、计算机间传输及 Nibiru 引擎原生集成。

**当前源码状态：ADM `0.6.1`、扩展 `0.3.206`；截至 2026-10-20，Nibiru/ED2K 集成仍在开发。** 应用版本号不代表本分支的所有改动都已发布。本文描述当前源码；已发布版本另有[更新日志](CHANGELOG.md)和 [Releases](https://github.com/linuxhell/apocalipse-download-manager/releases) 说明。

## 当前功能

| 范围 | 本源码树已实现的功能 |
| --- | --- |
| HTTP/HTTPS | 校验文件身份的断点续传、分段下载、经验证的镜像源、SHA-256、队列及自适应并发 |
| 种子/磁力链接 | Nibiru Xtreme、文件选择、节点/会话信息及媒体预览 |
| 浏览器 | 捕获视频、音频和图片并发送至 ADM；支持 `pt-BR`、`en`、`zh-CN` |
| 网页提取 | Nibiru Power 处理已支持的来源，其余现有流程使用 yt-dlp 路径 |
| HLS/DASH | 根据能力集成 Nibiru M3U8；适用流程保留原有路径 |
| 本地媒体 | Colapse 类型化操作：检查、WAV、FLAC、QOA、实验性 QOIR 与 PPM/PAM、Y4M、MP4 faststart |
| 录制 | 渐进式录制，并通过配置的媒体引擎导出 |
| Apocalipse Link | 通过 TLS 进行经过身份验证的传输；QUIC 是独立选项，默认关闭 |
| ED2K/Kad | 独立窗口、服务器搜索、来源查询、分块校验、主动共享和有界 Kad 会话 |
| 界面 | 980×620 主窗口、明暗模式、系统托盘及运行时语言切换 |

<p align="center"><img src="assets/screenshots/downloads.png" width="800" alt="显示示例数据的下载界面"></p>

截图用于展示界面，不证明公网连通性，也不证明最新检查点已在每个平台运行。

## 开始使用

1. 从 [Releases](https://github.com/linuxhell/apocalipse-download-manager/releases) 获取软件包，或编译本分支。将整个包解压到可写目录。
2. 启动 ADM。Windows 下请将随附 DLL 保留在程序旁，界面需要 WebView2；Linux 桌面使用 GTK/WebKit。
3. 在**设置 → 工具**中检查路径及能力。Nibiru 程序默认位于 `data/tools/` 的子目录，也支持自定义路径。
4. 安装对应的 Chrome、Edge 或 Firefox 扩展。本源码树的扩展清单为 `0.3.206`；本地生成 XPI 不会自动获得 Mozilla 签名。
5. 选择 `Português do Brasil`、`English` 或 `简体中文`。扩展与 ED2K 窗口会采用设置的语言。

可执行文件旁的 `data/` 保存设置、队列、日志及工具。密码和令牌使用操作系统凭据库；复制便携目录不保证这些秘密也能迁移。请仅下载您有权保存的内容。

## 引擎与集成

| 引擎 | 在 ADM 中的用途 | 当前契约 |
| --- | --- | --- |
| [Nibiru Xtreme](https://github.com/linuxhell/nibiru-xtreme) | 传输/种子及内嵌 ED2K 库 | 原生 `nibiru.*` RPC，验证身份和能力 |
| [Nibiru Power](https://github.com/linuxhell/nibiru-power) | 在支持的网页中发现媒体 | `extract URL --json --timeout 30`；ADM 监督器期限为 40 秒 |
| [Nibiru M3U8](https://github.com/linuxhell/nibiru-m3u8) | 支持的播放列表及分段 | 原生 CLI、能力声明及 JSONL 事件 |
| [Nibiru Colapse](https://github.com/linuxhell/nibiru-colapse) | 本地媒体文件 | 类型化操作；JSON 包含 `schema_version=1`、`engine`、`operation`、`result` |

四个程序均支持 `capabilities --json`。ADM 拒绝不兼容的身份/schema。Power 不接收 yt-dlp 命令，Colapse 不接收 FFmpeg 命令。FFmpeg、FFprobe、yt-dlp 和 QuickJS 仍用于原生引擎未覆盖的流程；请检查软件包实际包含的程序。

旧的 `aria2_rpc_*` 设置键用于迁移；当前集成启动 Xtreme 并使用 `nibiru.*` RPC。程序修订固定在 [scripts/nibiru-engines.json](scripts/nibiru-engines.json)。ED2K 库在[桌面 Cargo.toml](apps/desktop/src-tauri/Cargo.toml) 和 `Cargo.lock` 中另行固定；之后仅修改文档的提交不会改变已固定代码。

## 当前参数

### 应用与 CLI

| 命令 | 作用 |
| --- | --- |
| `apocalipse-desktop` | 打开应用 |
| `apocalipse-desktop --hidden` | 启动时隐藏主窗口 |
| `apocalipse-desktop --open URL` | 打开已识别的来源，也用于操作系统关联 |
| `apocalipse-desktop --verify-media-routing REPORT.json` | CI 诊断；写入报告后退出，不打开界面 |
| `apocalipse-cli URL DESTINATION` | 通过 Rust core 下载，请求 8 个连接并保留现有目标 |
| `apocalipse-cli benchmark URL DESTINATION [CONNECTIONS] [MIRROR ...]` | 默认 8 个连接，范围 1–32，启用自适应调度，输出 JSON 汇总 |

以上是 Cargo 构建目标名，软件包可能重命名桌面程序。Windows 下添加 `.exe`。CLI 是独立的 core 工具，不控制所有界面功能，也不提供请求头、校验和或覆盖目标选项。

### 界面设置

| `settings.json` 字段 | 默认值与含义 |
| --- | --- |
| `max_active_downloads` | `3`；可调整为 1–20 个任务 |
| `connections_per_download` | `16`；请求上限为 1–32；多路复用流不一定对应同等数量的 TCP 连接 |
| `adaptive_efficiency` | `true`；根据引擎/传输调整并发 |
| `global_bandwidth_limit` | `0` 表示不限速；字节/秒，上限 10 GiB/s |
| `download_directory` | 初始 `null`；在界面中选择 |
| `xtreme_path`、`power_path`、`m3u8_path`、`colapse_path` | 可选；默认 `data/tools/NAME/NAME[.exe]` |
| `xtreme_rpc_enabled`、`xtreme_rpc_auto_start` | `true`；`xtreme_rpc_port` 为 `null` 时自动选端口 |
| `ffmpeg_path`、`yt_dlp_path`、`qjs_path` | 对应流程的可选工具路径 |
| `proxy_enabled`、`proxy_url`、`dns_enabled`、`dns_servers` | 可配置代理/DNS，启用开关初始为 `false` |
| `language`、`dark_mode` | 语言 `en`、`pt-BR`、`zh-CN`；默认 `en` 和 `false` |
| `link_quic_enabled` | `false`；独立于 Link TLS 的实验选项 |

修改设置时优先使用界面。bridge 监听 `127.0.0.1:17654`；Link 基础端口为 `17655`。这些数字不是 CLI 参数。不要在配置示例中填写秘密。

### 本地媒体

Nibiru 适配器接受 13 个类型化操作，其中 12 个显示在面板中。拼接通过契约提供，当前界面尚无多输入选择器。路径必须为绝对路径；输入必须存在，输出必须是新文件。未知字段以及属于其他操作的字段会被拒绝。

| 操作 | 请求字段 |
| --- | --- |
| `inspect` | `input` |
| `wav-cut`、`y4m-cut` | `input`、`output`、`start`、`frames` |
| `wav-gain` | `input`、`output`、`db` |
| `wav-normalize` | `input`、`output`、`peak_db` |
| `wav-concat` | `inputs`、`output` |
| `flac-encode`、`flac-decode`、`qoa-encode`、`qoa-decode` | `input`、`output` |
| `qoir-encode` | `input`、`output`；可选 `input_color`：`bt709` 或 `srgb` |
| `qoir-decode` | `input`、`output` |
| `mp4-faststart` | `input`、`output` |

WAV 的 `start`/`frames` 按采样帧计数，Y4M 按视频帧计数。增益单位为 dB；归一化目标是峰值 dBFS，不测量 LUFS。FLAC 对支持的整数配置提供无损转换。QOA 使用有损 PCM16 且不包含校验和，不能精确保留原始音频。MP4 检查不解码所识别的编解码器。请参阅 Colapse 能力声明及 README 限制。

QOIR 是与 QOI 不同的实验性图像格式。编码接受单幅 P6 PPM 或 P7 PAM 图像，支持每通道 8 位的 RGB/RGBA，并生成无损 QOIR；解码导出 PAM，也接受该格式的有损级别。上限为 67,108,864 像素。输入颜色默认为 BT.709，除非提供已识别的颜色注释或显式 `input_color`；选择 sRGB 不会转换像素。PAM 导出会拒绝预乘 alpha 和无法表示的元数据，包括 ICC 配置文件。QOIR 没有强制校验和。面板标明格式处于实验阶段，且仅在编码时显示颜色选项。

ADM 契约要求裁剪 `frames >= 1`，且 `start + frames` 不溢出；增益范围为 −120 到 +120 dB，通过 ADM 归一化时接受 −120 到 0 dBFS，这是适配器边界与 Colapse 上限共同作用的范围。拼接接受 1–256 个输入。媒体监督器的期限为六小时。这些适配器边界叠加在 Colapse core 限制之上。

## ED2K/Kad：实际状态

已实现服务器登录/搜索、来源、MD4/分块校验的直接下载、续传、用户主动选择的共享、上传队列和向服务器发布共享列表。Kad 使用 HELLO、端点挑战、迭代查询和直接来源。[ED2K/Kad 文档](docs/ed2k-kad-native.md) 详述数据包、联系人、队列和取消操作的边界。

两个 ED2K 下载可并行运行；每项传输一次使用一个 peer，每次尝试最多使用 16 个直接来源。上传允许 1–64 个槽位及总 KiB/s 限速。TCP 监听优先使用 4662，被占用时另选端口。以服务器分配的 ID 为准：本地端口打开不证明 NAT/防火墙外部可达性。

Kad 最多有 512 个联系人、32 个待处理查询，每个查询期限三秒，端点验证有效期 20 分钟。直接来源可在没有 ED2K 服务器时进入队列。Kad 关键词搜索/发布、积分、AICH、混淆及与 aMule 完全等价的功能尚未完成。自有组件之间的本地测试不证明与 eMule/aMule 的公网互操作性。

## 编译、测试与打包

使用稳定版 Rust，在根目录执行以加载 `.cargo/config.toml`，其中启用了 reqwest 的 HTTP/3 API。JavaScript CI 使用 Node 22；构建脚本使用 Python 3.11 或更新版本。

```sh
cargo test --locked -p apocalipse-core
cargo build --locked --release -p apocalipse-cli
cargo build --locked --release -p apocalipse-desktop
node --test tests/*.test.cjs
python3 scripts/build_nibiru_engines.py --output dist/nibiru-tools --require-all
```

Debian/Ubuntu 桌面构建使用 `build-essential`、`pkg-config`、`libwebkit2gtk-4.1-dev`、`libappindicator3-dev`、`librsvg2-dev`、`libpam0g-dev`、`libssl-dev`。[CI 矩阵](.github/workflows/validate-portable.yml) 包含 Fedora/Arch 命令及 Windows/macOS 任务。只编译 core/CLI 不需要 GTK/WebKit。

| `build_nibiru_engines.py` 选项 | 行为 |
| --- | --- |
| `--output DIRECTORY` | 必填；引擎与 `build.json` 输出目录 |
| `--require-all` | 任一所需修订缺失时失败 |
| `--source-root DIRECTORY` | 使用 `DIRECTORY/nibiru-*` 本地 Git 克隆，始终检出固定修订；支持尚未发布的提交 |

脚本为其运行系统构建程序，检查版本/能力，复制许可证声明并记录 SHA-256；它不是交叉编译器。未发布的 pin 需要本地克隆或先发布相应提交。

### 验证与限制

当前检查点通过了直接用 Node 执行并按 TAP 统计的 44 个文件、341 个 JavaScript 测试，以及全部 33 个常规原生 harness 测试。另一个测试使用真实 Colapse 程序执行了全部 13 个类型化操作。JavaScript 套件中的四个 Node/VM 面板测试检查实时语言切换及三种语言下的 36 个模拟请求，包括颜色选项。三个 workflow 均通过 YAML、Bash 块语法及集成测试过滤器检查。此前的审计检查点还执行了 52 个 core 测试和四个真实引擎，并将桌面程序交叉编译为 Windows GNU x64。各项测试按范围分别报告，不累加重复运行；语法检查不等于执行远程任务。

包含 QOIR 和并行 FLAC 解码器的 Colapse 第 4 轮检查点通过了 172 个 Rust 测试（本次调用忽略四个依赖外部参考/语料的测试）、53 个 Python 测试、格式检查、Clippy，以及 13 个操作/21 个编解码方向的冒烟检查。ADM 的真实适配器测试使用了同一个冻结的 Linux 程序。[基于 FFmpeg 源码阅读的后续修复](https://github.com/linuxhell/nibiru-colapse/blob/main/docs/SOURCE_GUIDED_DEVELOPMENT_2026-10-20.md)、独立音频/像素比较及性能矩阵在引擎仓库中保留各自的记录。日志、命令、哈希和限制将每次执行绑定到对应程序；数量不累加重复运行，也不代表完整编解码器覆盖。VM 界面测试不能证明 Windows WebView 的实际运行行为。

Windows 编译成功不代表界面已运行。原生 Windows 执行、ED2K/Kad 公网连通性、AppImage 打包及完整语义审计仍需专门证据。[限定范围审计](docs/AUDITORIA-NIBIRU-ED2K-2026-10-20.md) 记录实际覆盖范围。安装器支持签名清单，但可信密钥列表为空，尚未强制要求该签名。详见[工具更新](docs/tool-update-signing.md)。

## 演进与贡献

新技术按可测量的速度、完整性、质量、兼容性或维护收益评估。HTTP/3、原生编解码器及并行处理必须保留可复现结果、适用后备路径和同步文档。实验不被宣传为已证明的改进。参阅[性能](docs/PERFORMANCE.md)、[路线图](ROADMAP.md)、[安全](SECURITY.md)和 [issues](https://github.com/linuxhell/apocalipse-download-manager/issues)；历史文档注明各自对应的快照。

您可以[通过 PayPal 支持开发](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL)。

## 许可证

ADM 使用 [GPL-3.0-or-later](LICENSE)。引擎及工具保留各自的许可证和声明；ADM 的许可证不替代这些条款。
