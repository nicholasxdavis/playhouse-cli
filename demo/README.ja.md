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
   <a href="README.zh.md">中文</a> •
   <b>日本語</b> •
   <a href="README.ko.md">한국어</a>
</p>
<p align="center">
   <img src="https://img.shields.io/github/actions/workflow/status/nicholasxdavis/playhouse-cli/ci.yml?branch=main&style=for-the-badge&label=CI&color=2bba68" alt="CIステータス">
   <img src="https://img.shields.io/npm/v/@nicholasxdavis/playhouse-cli?style=for-the-badge&label=Version&color=14949c" alt="npmバージョン">
   <img src="https://img.shields.io/github/v/release/nicholasxdavis/playhouse-cli?style=for-the-badge&label=Release&color=14949c" alt="GitHubリリース">
   <img src="https://img.shields.io/badge/License-MIT-14949c.svg?style=for-the-badge" alt="MITライセンス">
</p>

<p align="center">
  セキュリティ、機能テスト、パフォーマンス監査、およびエージェントへの引き継ぎのための品質保証（QA）CLIツール。<br>
  シェル、CI、またはエージェントのツールからヘッドレス（headless）で実行されます。人間用のオプションのTUI。
</p>

<p align="center">
  <img src="icon/tui-7-6.png?raw=true" alt="Playhouse CLIデモ" width="75%">
</p>

## インストール

**推奨手順:**

```bash
npm install -g @nicholasxdavis/playhouse-cli
playhouse install --full   # Trivy, Arkenar, Playwright, Lighthouse, chromium
playhouse init             # .playhouse/ とエージェントスキルをセットアップ
```

開発依存関係:

```bash
pnpm add -D @nicholasxdavis/playhouse-cli
npx playhouse doctor
```

| プロファイル | コマンド | インストールされるツール |
|--------|---------|-----------------------|
| フル（デフォルト） | `playhouse install` または `--full` | Trivy, Arkenar, Playwright, Lighthouse, chromium |
| 最小限 | `playhouse install --minimal` | Trivy + Arkenar のみ |

プロジェクトのオプションのフック:

```json
"scripts": {
  "postinstall": "playhouse-install-tools"
}
```

ツールのインストールに失敗した際に `npm install` 自体を失敗させるには、`PLAYHOUSE_INSTALL_STRICT=1` を設定してください。

| 方法 | コマンド |
|--------|---------|
| npm（主要） | `npm i -g @nicholasxdavis/playhouse-cli` |
| npx | `npx @nicholasxdavis/playhouse-cli init` |
| GitHub リリース | [最新バイナリ](https://github.com/nicholasxdavis/playhouse-cli/releases/latest) |
| cargo（開発者用） | `cargo install --path . --force` |
| 手動バイナリ | `PLAYHOUSE_BIN` または `PLAYHOUSE_SKIP_DOWNLOAD=1` を設定 |

npmパッケージは `postinstall` 実行時に、OSおよびCPUに応じたネイティブバイナリ（~12 MB）をGitHub Releasesからダウンロードします。現在のリリース: **v0.1.2**。

**ソースからインストール:**

```bash
git clone https://github.com/nicholasxdavis/playhouse-cli.git
cd playhouse-cli
cargo build --release
cargo install --path . --force
```

ローカルのnpm開発ループ:

```bash
cargo build --release
cd packages/playhouse && npm run link-local   # bash
# PowerShell 5.x の場合: cd packages/playhouse; npm run link-local
```

**CI:** [GitHub Actions](https://github.com/nicholasxdavis/playhouse-cli/actions) は Ubuntu、macOS、Windows上で `cargo test`、npmのスモークテスト、および `playhouse verify` を実行します。

**リリース:** タグ `v0.1.2`（`Cargo.toml` および `packages/playhouse/package.json` と一致させる必要があります）をプッシュし、npm postinstall用のバイナリをリリースします。

## クイックスタート

```bash
playhouse                    # TUI（人間用）
playhouse doctor             # ツールのチェック
playhouse verify --json      # 完全なQAスイートの実行 + 0〜100スターの評価
playhouse agent --json       # エージェント・マニフェスト（最初に読む）
playhouse upgrade --json     # GitHubとnpmでの更新チェック
```

**ブラウザ監査（Lighthouse、Arkenar）にはURLが必要です:**

```bash
playhouse config set default_url http://localhost:3000
```

Playhouseは、`package.json`、Vite、Wranglerの設定からポートのヒントも読み取り、一般的な開発用ポートをスキャンします。

**モノレポ:**

```bash
playhouse config set scan_root apps/web
playhouse config set test_root apps/web
```

## エージェント用情報

最初に **[AGENTS.md](../AGENTS.md)** をお読みください。`playhouse init` を実行すると、**`.playhouse/SKILL.md`** やその他のワークスペースファイルがインストールされます。

```bash
playhouse skill install
playhouse skill status --json
```

## コマンド

| コマンド | 説明 |
|---------|-------------|
| `playhouse agent [--json]` | 完全なエージェント・マニフェスト |
| `playhouse agent status` | クイックヘルスチェック + 次のアクション |
| `playhouse agent plan` | フェーズ別のワークフロー |
| `playhouse agent handoff` | 検証と引き継ぎバンドルの書き出し |
| `playhouse verify` | Trivy + 機能テスト + Arkenar + Lighthouse |
| `playhouse test list\|init\|run` | テスト用テンプレート（ベースプレート） |
| `playhouse score` | Playhouse評価（0〜100スター） |
| `playhouse doctor` | ツールのヘルスチェック |
| `playhouse install` | バンドルされたツールの自動インストール |
| `playhouse config` | 設定（取得/設定/スキーマ） |
| `playhouse skill` | `.playhouse/SKILL.md`の管理 |
| `playhouse upgrade` | GitHubやnpmで最新リリースを確認 |

参照情報: [stars.md](../stars.md), [playwright.md](../playwright.md), [lighthouse.md](../lighthouse.md), [trivy.md](../trivy.md), [arkenar.md](../arkenar.md), [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md).

## Playhouse評価（スター）

検証後の0〜100の複合監査スコア。レポートは `.playhouse/reports/score.json` に出力されます。デフォルトの合格しきい値: 75。

## プロジェクト構成

```
プロジェクトルート/
  .playhouse/
    SKILL.md
    BRIEF.md
    AGENT.json
    config.json
    reports/score.json
```

## パッケージマネージャー

PlaywrightおよびLighthouseはnpm、pnpm、yarn、またはbunを使用します:

```bash
playhouse config set package_manager pnpm
```

## 終了コード

| コード | 意味 |
|------|---------|
| 0 | 合格（成功） |
| 1 | 失敗 |
| 2 | Lighthouseスコアがしきい値未満 |
| 3 | Arkenarの指摘事項あり |
| 4 | Trivyの指摘事項あり |
| 5 | 必要なツールがありません |
