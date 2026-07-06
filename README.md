<p align="center">
  <img src="https://github.com/nicholasxdavis/playhouse-cli/raw/main/demo/icon/playhouse-logo.png" alt="Playhouse" width="360">
</p>

<p align="center">
  <a href="https://github.com/nicholasxdavis/playhouse-cli/actions/workflows/ci.yml?branch=main">
    <img src="https://img.shields.io/github/actions/workflow/status/nicholasxdavis/playhouse-cli/ci.yml?branch=main&style=for-the-badge&label=CI&color=2bba68" alt="CI Status">
  </a>
  <a href="https://www.npmjs.com/package/@nicholasxdavis/playhouse-cli">
    <img src="https://img.shields.io/npm/v/@nicholasxdavis/playhouse-cli?style=for-the-badge&label=Version&color=14949c" alt="npm version">
  </a>
  <a href="https://github.com/nicholasxdavis/playhouse-cli/releases/tag/v0.1.0">
    <img src="https://img.shields.io/github/v/release/nicholasxdavis/playhouse-cli?style=for-the-badge&label=Release&color=14949c" alt="GitHub Release">
  </a>
  <a href="https://github.com/nicholasxdavis/playhouse-cli">
    <img src="https://img.shields.io/github/stars/nicholasxdavis/playhouse-cli?style=for-the-badge&label=Stars&color=2bba68" alt="GitHub Stars">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-2bba68.svg?style=for-the-badge" alt="MIT License">
  </a>
</p>

<p align="center">
  QA CLI for security, functional testing, performance audits, and agent handoff.<br>
  Run headless from shell, CI, or agent tooling. Optional TUI for humans.
</p>

> **If you are an LLM or coding agent, stop reading this file.**  
> Go to **[AGENTS.md](AGENTS.md)** for the full command reference, workflows, exit codes, and config keys.

## Install

**Recommended (Node 18+, no Rust required):**

```bash
npm install -g @nicholasxdavis/playhouse-cli
playhouse install --full   # Trivy, Arkenar, Playwright, Lighthouse, chromium
playhouse init             # set up .playhouse/ and agent skill
```

Dev dependency:

```bash
pnpm add -D @nicholasxdavis/playhouse-cli
npx playhouse doctor
```

| Profile | Command | Installs |
|---------|---------|----------|
| Full (default) | `playhouse install` or `--full` | Trivy, Arkenar, Playwright, Lighthouse, chromium |
| Minimal | `playhouse install --minimal` | Trivy + Arkenar only |

Optional project hook:

```json
"scripts": {
  "postinstall": "playhouse-install-tools"
}
```

Set `PLAYHOUSE_INSTALL_STRICT=1` to fail `npm install` when tool install fails.

| Method | Command |
|--------|---------|
| npm (primary) | `npm i -g @nicholasxdavis/playhouse-cli` |
| npx | `npx @nicholasxdavis/playhouse-cli init` |
| GitHub Release | [v0.1.0 binaries](https://github.com/nicholasxdavis/playhouse-cli/releases/tag/v0.1.0) |
| cargo (developers) | `cargo install --path . --force` |
| manual binary | set `PLAYHOUSE_BIN` or `PLAYHOUSE_SKIP_DOWNLOAD=1` |

The npm package downloads the native binary (~12 MB) from GitHub Releases on `postinstall`. Current release: **v0.1.0**.

**From source:**

```bash
git clone https://github.com/nicholasxdavis/playhouse-cli.git
cd playhouse-cli
cargo build --release
cargo install --path . --force
```

Local npm dev loop:

```bash
cargo build --release && cd packages/playhouse && npm run link-local
```

**CI:** [GitHub Actions](https://github.com/nicholasxdavis/playhouse-cli/actions) runs `cargo test`, npm smoke, and `playhouse verify` on Ubuntu, macOS, and Windows.

**Releases:** push tag `v0.1.0` (must match `Cargo.toml` and `packages/playhouse/package.json`) to publish binaries for npm postinstall.

## Quick start

```bash
playhouse                    # TUI (humans)
playhouse doctor             # check tools
playhouse verify --json      # full QA suite + 0-100 star rating
playhouse agent --json       # agent manifest (read first)
playhouse upgrade --json     # check GitHub + npm for updates
```

**Browser audits (Lighthouse, Arkenar)** need a URL:

```bash
playhouse config set default_url http://localhost:3000
```

Playhouse also reads port hints from `package.json`, Vite, and Wrangler config, then probes common dev ports.

**Monorepos:**

```bash
playhouse config set scan_root apps/web
playhouse config set test_root apps/web
```

## For agents

Read **[AGENTS.md](AGENTS.md)** first. `playhouse init` installs **`.playhouse/SKILL.md`** and other workspace files.

```bash
playhouse skill install
playhouse skill status --json
```

## Commands

| Command | Description |
|---------|-------------|
| `playhouse agent [--json]` | Full agent manifest |
| `playhouse agent status` | Quick health + next actions |
| `playhouse agent plan` | Phased workflow |
| `playhouse agent handoff` | Verify + export handoff bundle |
| `playhouse verify` | Trivy + functional + Arkenar + Lighthouse |
| `playhouse test list\|init\|run` | Test baseplates |
| `playhouse score` | Playhouse Stars (0-100) |
| `playhouse doctor` | Tool health |
| `playhouse install` | Auto-install bundled tools |
| `playhouse config` | Settings (get/set/schema) |
| `playhouse skill` | `.playhouse/SKILL.md` management |
| `playhouse upgrade` | Check GitHub / npm for newer releases |

See also: [stars.md](stars.md), [playwright.md](playwright.md), [lighthouse.md](lighthouse.md), [trivy.md](trivy.md), [arkenar.md](arkenar.md), [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Playhouse Stars

Combined 0-100 audit score after verify. Report at `.playhouse/reports/score.json`. Default pass threshold: 75.

## Project layout

```
your-project/
  .playhouse/
    SKILL.md
    BRIEF.md
    AGENT.json
    config.json
    reports/score.json
```

## Package managers

Playwright and Lighthouse use npm, pnpm, yarn, or bun:

```bash
playhouse config set package_manager pnpm
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Pass |
| 1 | Failure |
| 2 | Lighthouse below threshold |
| 3 | Arkenar findings |
| 4 | Trivy findings |
| 5 | Tool missing |

## Links

| Resource | URL |
|----------|-----|
| Repository | https://github.com/nicholasxdavis/playhouse-cli |
| Release v0.1.0 | https://github.com/nicholasxdavis/playhouse-cli/releases/tag/v0.1.0 |
| npm package | https://www.npmjs.com/package/@nicholasxdavis/playhouse-cli |
| Issues | https://github.com/nicholasxdavis/playhouse-cli/issues |

## License

MIT. See [LICENSE](LICENSE).
