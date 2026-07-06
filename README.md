<p align="center">
  <img src="https://github.com/nicholasxdavis/playhouse-cli/raw/main/demo/icon/playhouse-logo.png" alt="Playhouse" width="360">
</p>

<h1 align="center">Playhouse</h1>

> **If you are an LLM or coding agent, stop reading this file.**  
> Go to **[AGENTS.md](AGENTS.md)** — it has the full command reference, workflows, exit codes, config keys, and everything you need to operate Playhouse correctly.

<p align="center">QA CLI for security, functional testing, performance audits, and agent handoff.<br>Run headless from shell, CI, or agent tooling. Optional TUI for humans.</p>

## Install

**Recommended (no Rust required):**

```bash
npm install -g playhouse
playhouse install --full   # Trivy, Arkenar, Playwright, Lighthouse, chromium
playhouse init             # set up .playhouse/ and agent skill
```

Also works as a dev dependency: `pnpm add -D playhouse` then `npx playhouse doctor`.

| Profile | Command | Installs |
|---------|---------|----------|
| Full (default) | `playhouse install` or `--full` | Trivy, Arkenar, Playwright, Lighthouse, chromium |
| Minimal | `playhouse install --minimal` | Trivy + Arkenar only |

Optional project hook (after `playhouse` is a devDependency):

```json
"scripts": {
  "postinstall": "playhouse-install-tools"
}
```

Set `PLAYHOUSE_INSTALL_STRICT=1` to fail `npm install` when tool install fails.

| Method | Command |
|--------|---------|
| npm (primary) | `npm i -g playhouse` |
| npx | `npx playhouse@latest init` |
| cargo (developers) | `cargo install --path . --force` |
| manual binary | set `PLAYHOUSE_BIN` / `PLAYHOUSE_SKIP_DOWNLOAD=1` |

Requires **Node.js 18+** for Playwright and Lighthouse. The Playhouse CLI itself is a native binary (~12 MB) downloaded on `npm install`.

**From source (Rust):**

```bash
cargo build --release
# or
cargo install --path . --force
```

Local npm dev loop: `cargo build --release && cd packages/playhouse && npm run link-local`

**CI:** GitHub Actions runs on [ubuntu, macOS, Windows](https://github.com/nicholasxdavis/playhouse-cli/actions) — `cargo test`, npm smoke, and `playhouse verify` on each OS.

**Releases:** tag `v0.1.0` (match `Cargo.toml` / `packages/playhouse/package.json`) to publish GitHub Release assets for npm postinstall.

## Quick start

```bash
playhouse                    # TUI (humans)
playhouse doctor             # check tools
playhouse verify --json      # full QA suite + 0-100 star rating
playhouse agent --json       # agent manifest (read first)
```

**Browser audits (Lighthouse, Arkenar)** need a URL. Set the workspace default once:

```bash
playhouse config set default_url http://localhost:3000
```

Playhouse also probes port hints from `package.json` / Vite / Wrangler config, then common dev ports.

**Monorepos:** set `scan_root` and `test_root` in `.playhouse/config.json` (or via `playhouse config set scan_root apps/web`).

## For agents

Read **[AGENTS.md](AGENTS.md)** first. In consumer projects, `playhouse init` installs **`.playhouse/SKILL.md`** and other workspace files.

```bash
playhouse skill install      # install or refresh .playhouse/SKILL.md
playhouse skill status --json
```

Enabled by default. Disable with `playhouse skill disable` or config.

## Commands

| Command | Description |
|---------|-------------|
| `playhouse agent [--json]` | Full agent manifest |
| `playhouse agent status` | Quick health + next actions |
| `playhouse agent plan` | Phased workflow |
| `playhouse agent handoff` | Verify + export handoff bundle |
| `playhouse verify` | Trivy + Playwright + Arkenar + Lighthouse |
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
    SKILL.md              # agent skill (recommended)
    BRIEF.md              # workspace QA brief
    AGENT.json            # handoff bundle
    config.json           # workspace config
    stay-on-track/        # optional discipline skill
      SKILL.md
      PROJECT.md
    reports/
      score.json
```

## Package managers

Playwright and Lighthouse use npm, pnpm, yarn, or bun. Auto-detected from lockfiles or set via:

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

## License

MIT - see [LICENSE](LICENSE).
