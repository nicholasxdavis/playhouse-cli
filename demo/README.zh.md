<p align="center">
  <img src="icon/playhouse-logo.png" alt="Playhouse" width="360">
</p>
<p align="center">
   <a href="../README.md">English</a> •
   <a href="README.es.md">Español</a> •
   <a href="README.fr.md">Français</a> •
   <a href="README.de.md">Deutsch</a> •
   <a href="README.it.md">Italiano</a> •
   <a href="README.pt.md">Português</a> •
   <a href="README.ru.md">Русский</a> •
   <b>中文</b> •
   <a href="README.ja.md">日本語</a> •
   <a href="README.ko.md">한국어</a>
</p>
<p align="center">
   <img src="https://img.shields.io/github/actions/workflow/status/nicholasxdavis/playhouse-cli/ci.yml?branch=main&style=for-the-badge&label=CI&color=2bba68" alt="CI 状态">
   <img src="https://img.shields.io/npm/v/@nicholasxdavis/playhouse-cli?style=for-the-badge&label=Version&color=14949c" alt="npm 版本">
   <img src="https://img.shields.io/github/v/release/nicholasxdavis/playhouse-cli?style=for-the-badge&label=Release&color=14949c" alt="GitHub 发布版本">
   <img src="https://img.shields.io/badge/License-MIT-14949c.svg?style=for-the-badge" alt="MIT 许可证">
</p>

<p align="center">
  用于安全、功能测试、性能审计和智能体交付（agent handoff）的 QA 命令行工具（CLI）。<br>
  可从终端、CI 或智能体工具链中以无头（headless）模式运行。为人类用户提供可选的 TUI 界面。
</p>

<p align="center">
  <img src="icon/tui-7-6.png?raw=true" alt="Playhouse CLI 演示" width="75%">
</p>

## 安装

**推荐安装方法：**

```bash
npm install -g @nicholasxdavis/playhouse-cli
playhouse install --full   # 安装 Trivy, Arkenar, Playwright, Lighthouse, chromium
playhouse init             # 初始化 .playhouse/ 目录及智能体技能文件
```

开发依赖：

```bash
pnpm add -D @nicholasxdavis/playhouse-cli
npx playhouse doctor
```

| 配置档 | 命令 | 安装内容 |
|--------|---------|----------|
| 完整（默认） | `playhouse install` 或 `--full` | Trivy, Arkenar, Playwright, Lighthouse, chromium |
| 极简 | `playhouse install --minimal` | 仅安装 Trivy + Arkenar |

可选项目钩子：

```json
"scripts": {
  "postinstall": "playhouse-install-tools"
}
```

设置 `PLAYHOUSE_INSTALL_STRICT=1` 可以在工具安装失败时使 `npm install` 失败。

| 方法 | 命令 |
|--------|---------|
| npm（主要） | `npm i -g @nicholasxdavis/playhouse-cli` |
| npx | `npx @nicholasxdavis/playhouse-cli init` |
| GitHub 交付版本 | [最新二进制文件](https://github.com/nicholasxdavis/playhouse-cli/releases/latest) |
| cargo（开发者） | `cargo install --path . --force` |
| 手动二进制 | 设置 `PLAYHOUSE_BIN` 或 `PLAYHOUSE_SKIP_DOWNLOAD=1` |

npm 包在 `postinstall` 阶段从 GitHub Releases 下载对应系统及 CPU 的原生二进制文件（~12 MB）。当前版本：**v0.1.2**。

**源码编译安装：**

```bash
git clone https://github.com/nicholasxdavis/playhouse-cli.git
cd playhouse-cli
cargo build --release
cargo install --path . --force
```

本地 npm 开发循环：

```bash
cargo build --release
cd packages/playhouse && npm run link-local   # bash 终端
# PowerShell 5.x 终端: cd packages/playhouse; npm run link-local
```

**CI：** [GitHub Actions](https://github.com/nicholasxdavis/playhouse-cli/actions) 在 Ubuntu、macOS 和 Windows 上运行 `cargo test`、npm 冒烟测试以及 `playhouse verify`。

**发布版本：** 推送标签 `v0.1.2`（必须与 `Cargo.toml` 和 `packages/playhouse/package.json` 一致）以发布供 npm postinstall 下载的二进制文件。

## 快速开始

```bash
playhouse                    # 打开 TUI 界面（供人类使用）
playhouse doctor             # 检查工具链状态
playhouse verify --json      # 运行完整 QA 套件并生成 0-100 评分
playhouse agent --json       # 读取智能体清单（建议首先运行）
playhouse upgrade --json     # 检查 GitHub 和 npm 上的更新
```

**浏览器审计（Lighthouse, Arkenar）需要指定一个 URL：**

```bash
playhouse config set default_url http://localhost:3000
```

Playhouse 也会从 `package.json`、Vite 和 Wrangler 配置中读取端口提示，并探测常见的开发端口。

**单体多包仓库（Monorepos）：**

```bash
playhouse config set scan_root apps/web
playhouse config set test_root apps/web
```

## 给开发智能体（Agents）

请先阅读 **[AGENTS.md](../AGENTS.md)**。运行 `playhouse init` 会安装 **`.playhouse/SKILL.md`** 和其他工作区配置文件。

```bash
playhouse skill install
playhouse skill status --json
```

## 命令

| 命令 | 描述 |
|---------|-------------|
| `playhouse agent [--json]` | 完整智能体清单 |
| `playhouse agent status` | 快速状态检查 + 后续建议行动 |
| `playhouse agent plan` | 阶段式工作流建议 |
| `playhouse agent handoff` | 运行全面验证并导出交付包 |
| `playhouse verify` | 一键运行 Trivy + 功能测试 + Arkenar + Lighthouse |
| `playhouse test list\|init\|run` | 测试模板基板管理 |
| `playhouse score` | Playhouse 评分 (0-100) |
| `playhouse doctor` | 工具链健康状况检查 |
| `playhouse install` | 自动安装缺失的捆绑工具 |
| `playhouse config` | 配置项读写 (get/set/schema) |
| `playhouse skill` | `.playhouse/SKILL.md` 工具技能管理 |
| `playhouse upgrade` | 检查 GitHub / npm 上的最新发布版本 |

另请参阅：[stars.md](../stars.md)、[playwright.md](../playwright.md)、[lighthouse.md](../lighthouse.md)、[trivy.md](../trivy.md)、[arkenar.md](../arkenar.md)、[THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md)。

## Playhouse 评分系统

验证后的综合 0-100 审计得分。报告路径为 `.playhouse/reports/score.json`。默认通过分值：75。

## 项目结构

```
你的项目/
  .playhouse/
    SKILL.md
    BRIEF.md
    AGENT.json
    config.json
    reports/score.json
```

## 包管理器

Playwright 和 Lighthouse 支持使用 npm、pnpm、yarn 或 bun：

```bash
playhouse config set package_manager pnpm
```

## 退出状态码

| 状态码 | 含义 |
|------|---------|
| 0 | 通过 |
| 1 | 测试或验证失败 |
| 2 | Lighthouse 得分低于指定阈值 |
| 3 | 发现 Arkenar 安全问题 findings |
| 4 | 发现 Trivy 漏洞或泄漏密钥 |
| 5 | 缺少必要工具 |
