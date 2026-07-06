 
<p align="center" style="margin-bottom:0;">
  <img src="https://github.com/nicholasxdavis/playhouse-cli/raw/main/demo/icon/playhouse-logo.png" alt="Playhouse" width="550">
</p>
<p align="center" style="margin-top:0;">
  QA CLI for security, functional testing, performance audits, and agent handoff.<br>
  Run headless from shell, CI, or agent tooling. Optional TUI for humans.
</p>
## Install

```bash
cargo install --path . --force
# or
cargo build --release
```

Requires [Rust](https://rust-lang.org). Node.js is needed for Playwright and Lighthouse (installed automatically).

```bash
playhouse install   # bundled Trivy, Arkenar, Playwright
playhouse init      # set up .playhouse/ and agent skill
```

## Quick start

```bash
playhouse                    # TUI (humans)
playhouse doctor             # check tools
playhouse verify --json      # full QA suite + 0-100 star rating
playhouse agent --json       # agent manifest (read first)
```

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

See also: [stars.md](stars.md), [playwright.md](playwright.md), [lighthouse.md](lighthouse.md), [trivy.md](trivy.md), [arkenar.md](arkenar.md).

## Playhouse Stars

Combined 0-100 audit score after verify. Report at `.playhouse/reports/score.json`. Default pass threshold: 75.


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
