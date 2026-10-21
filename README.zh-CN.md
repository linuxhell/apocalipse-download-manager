<p align="center"><img src="assets/branding/apocalipse-a-blue.svg" width="160" alt="Apocalipse"></p>

[Português](README.md) · [English](README.en.md) · [简体中文](README.zh-CN.md)

# Apocalipse Download Manager

使用 Rust/Tauri 开发的便携式下载管理器，提供浏览器扩展。

## 项目状态

本源码基线版本为 **0.6.1**，扩展版本为 **0.3.206**。代码审计仍在进行；源码版本不表示本分支的修改已经发布。

ADM 继续使用外部 **aria2**、**yt-dlp nightly**、**N_m3u8DL-RE**、**FFmpeg**、**QuickJS**、解压工具和播放器。Nibiru Xtreme 独立开发，只有在引擎完成并验证集成后才会替换 aria2。本分支不集成 Nibiru M3U8、Power 或 Colapse。

## 安装

请访问[发布页面](https://github.com/linuxhell/apocalipse-download-manager/releases/latest)，根据实际发布版本选择浏览器扩展。

| 平台 | 安装包 |
| --- | --- |
| Windows x64 | 便携 ZIP |
| Linux x64 | AppImage 或便携 tar.gz |
| macOS x64 | 便携 ZIP |
| Chrome / Edge | 扩展 ZIP |
| Firefox | 发布中提供的 Mozilla 签名 XPI |

解压后启动应用，在**设置 → 工具**中配置工具。安装对应扩展即可从浏览器发送下载。未签名的测试 XPI 不等同于适用于正式版 Firefox 的签名安装包。

## 功能与限制

- HTTP/HTTPS 断点续传、队列、带宽限制、镜像源和 SHA-256 校验。
- 通过 aria2 下载种子和磁力链接，选择文件并调用配置的播放器预览。
- 扩展检测媒体，yt-dlp 负责提取；N_m3u8DL-RE 处理 HLS，FFmpeg 处理音频及转换流程。
- QuickJS 为 yt-dlp 提供 JavaScript 运行时；访问需要 JavaScript 挑战的网站时应配置其路径。
- 渐进式录制与媒体导出。
- Apocalipse Link：选定共享、TLS 和远程系统账户认证。SMB 共享发现取决于操作系统。
- 应用和扩展支持巴西葡萄牙语、英语及简体中文。

兼容性取决于网站、已安装工具和操作系统。传输原型不表示全面兼容或已在公网验证。工具签名清单校验需要配置可信密钥；参见[更新签名](docs/tool-update-signing.md)。

## 结构与开发

| 路径 | 职责 |
| --- | --- |
| `crates/apocalipse-core` | 模型、分类、传输和策略 |
| `crates/apocalipse-cli` | 命令行界面 |
| `apps/desktop/src-tauri` | 桌面后端、工具、持久化和浏览器桥接 |
| `apps/desktop/ui` | 应用及 Link 界面 |
| `browser-extension` | 浏览器捕获与下载发送 |
| `experiments/transport-lab` | 桌面程序使用的实验性传输 |
| `tests` | JavaScript 回归测试和契约 |

需要稳定版 Rust 和 Node.js。Linux 桌面构建还需要 WebKitGTK 4.1、GTK/AppIndicator、librsvg、PAM 及编译工具；工作流列出了各发行版依赖。

```sh
cargo fmt --all -- --check
cargo test --locked -p apocalipse-core -p apocalipse-cli
cargo test --workspace --locked
cargo build --release --locked -p apocalipse-desktop
node --test tests/*.test.cjs
```

完整工作区测试需要桌面原生依赖。编译和测试通过不等于已在 Windows、Linux 和 macOS 的真实使用环境中验证。

## CI 与文档

- `validate-portable.yml`：三个平台的测试、Linux 兼容性检查及候选安装包，不发布 release。
- `transport-lab.yml`：桌面所用传输的回归测试。
- `release.yml`：构建及发布版本；不能把审计分支当作最终版本发布。

参见[更新记录](CHANGELOG.md)、[路线图](ROADMAP.md)、[安全说明](SECURITY.md)和[性能说明](docs/PERFORMANCE.md)。下载文件、未完成文件及检查点不是仓库垃圾。

## 支持与许可

[通过 PayPal 捐赠](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL)。

GPL-3.0-or-later；参见 [LICENSE](LICENSE)。
