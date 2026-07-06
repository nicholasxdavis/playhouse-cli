# Arkenar DAST — Web Security Engine (ZAP Replacement)

## Overview

Playhouse ships **[Arkenar](https://github.com/realozk/ARKENAR)** as its dynamic application security testing (DAST) engine — a **pure Rust, MIT-licensed** scanner that replaces OWASP ZAP without Docker, Java, or heavyweight orchestration.

| | OWASP ZAP (old plan) | Arkenar (Playhouse) |
|---|---|---|
| License | Apache 2.0 | **MIT** |
| Runtime | Docker / JVM | **Single static binary** |
| Install | `docker pull` + volumes | **`playhouse install`** |
| Agent output | Custom ZAP JSON | **`.playhouse/reports/arkenar.json`** + `--json` |

Arkenar covers what Playhouse needs for dev/CI security:

- Native Rust crawler + forced browse
- Security header / misconfiguration checks
- Parameter fuzzing (`--enable-param-fuzz`)
- JS secret analysis (`--enable-js-analysis`)
- Scoped scans (`--scope`) against local dev servers
- JSON reports for agents

## Auto-install

Playhouse bundles Arkenar into `~/.config/playhouse/bin/arkenar` (or `arkenar.exe` on Windows) alongside Trivy and Playwright:

```bash
playhouse install          # installs all bundled tools
playhouse doctor --json    # confirms Arkenar is ready
```

Auto-install is **on by default** (`auto_install_tools` in `/config` → Tools).

## Headless CLI (agents)

```bash
# DAST scan against running dev server (auto-detects localhost ports)
playhouse arkenar --json

# Explicit URL
playhouse arkenar http://localhost:3000 --json

# Full verify suite (Trivy + Playwright + Arkenar + Lighthouse when URL available)
playhouse verify --json
```

### Exit codes

| Code | Meaning |
|------|---------|
| `0` | Clean — no high/medium findings |
| `3` | **Arkenar** high or medium severity findings |
| `5` | Arkenar not installed — run `playhouse install` |

### JSON envelope

```json
{
  "engine": "arkenar",
  "passed": false,
  "exitCode": 3,
  "target": "http://localhost:3000",
  "reportPath": ".playhouse/reports/arkenar.json",
  "summary": { "high": 0, "medium": 2, "low": 1, "total": 3 },
  "findings": { }
}
```

Agents should read `summary.high` and `summary.medium`. Any non-zero high/medium fails verify.

## TUI

```text
/arkenar     Run DAST against local dev server
/install     Install Arkenar + Trivy + Playwright
/verify      Full suite including Arkenar
/config      Tools tab → skip Arkenar in verify
             Stay-on-track tab → Arkenar scan profile
```

## Configuration (`/config`)

| Setting | Default | Description |
|---------|---------|-------------|
| Skip Arkenar in verify | off | Skip DAST during `/verify` |
| Arkenar advanced mode | off | `simple` vs `advanced` scan profile |
| Arkenar param fuzz | on | Parameter fuzzing |
| Arkenar JS analysis | on | Scan linked scripts for secrets |

Headless: `playhouse config --json`

## Reports

- **JSON report:** `.playhouse/reports/arkenar.json`
- **Agent manifest:** `playhouse agent --json` → `tools.arkenarPath`, `commands`

## Workflow for agents

1. Start dev server (`npm run dev`)
2. `playhouse install` (once)
3. `playhouse arkenar --json` or `playhouse verify --json`
4. Fix high/medium findings
5. Re-run until `exitCode: 0`

## Why Arkenar over ZAP

- **No Docker** — critical for Windows dev machines and fast CI
- **MIT** — ships cleanly with Playhouse
- **Rust** — aligns with Playhouse's stack; one binary, no JVM
- **JSON-native** — built for automation (`-o report.json`, `--json`)
- **Lightweight** — scoped local scans with rate limits (default 50 req/s)

## Legal / scope

Arkenar is for **authorized testing only**. Playhouse defaults to `--scope` (same-origin) and local dev URLs. Only scan systems you own or have permission to test.

## Links

- Arkenar repo: https://github.com/realozk/ARKENAR
- License: MIT
- Playhouse install: `playhouse install`
