---
name: playhouse
description: Agent skill for Playhouse QA CLI. Run headless audits with --json. Disable via `playhouse skill disable`.
recommended: true
---

# Playhouse Agent Skill

Headless QA CLI for security, functional tests, performance, and agent handoff. Do not use the TUI (`playhouse` with no args).

**Live manifest:** `playhouse agent --json` (commands, settings, nextActions). Prefer that over this file when they differ.

## Golden rules

1. Shell commands only, not the TUI.
2. `--json` on every command unless the user wants human text.
3. Run `playhouse verify --json` before claiming work is done.
4. Check `exitCode` in JSON (0 pass, 1 fail, 2 lighthouse, 3 arkenar, 4 trivy, 5 missing tool).
5. Windows PowerShell 5.x: use `playhouse -C "C:\project" doctor --json`, not `cd && playhouse`.

## Start of session

```bash
playhouse agent --json
playhouse agent status --json
playhouse doctor --json
```

New workspace: `playhouse init --json` then `playhouse install`

## Commands

| Command | Purpose |
|---------|---------|
| `playhouse doctor --json` | Tool health |
| `playhouse trivy --json` | Static security + secrets |
| `playhouse functional [pattern] --json` | Detected test runner |
| `playhouse playwright [pattern] --json` | Playwright only |
| `playhouse arkenar --json` | DAST web scan |
| `playhouse lighthouse --json` | Perf, a11y, SEO |
| `playhouse verify [--test PATTERN] [--start-server CMD] [--server-port N] --json` | Full audit + stars |
| `playhouse agent handoff --json` | Verify + `.playhouse/AGENT.json` |

## During development

```bash
playhouse functional mytest.spec.ts --json
playhouse trivy --json
playhouse lighthouse --json
```

## Before handoff

```bash
playhouse verify --json
playhouse agent handoff --json
```

Pass gate: exit 0, stars >= 75 (default), no Trivy HIGH/CRITICAL or secrets.

## URL for browser audits

```bash
playhouse config set default_url http://localhost:3000
```

Or pass `--url` on verify. Or start a server inline:

```bash
playhouse verify --start-server "npm run dev" --json
```

### Auth for browser audits

```bash
playhouse config set audit_headers '{"Authorization":"Bearer TOKEN"}'
```

Do not commit tokens. Use workspace config only.

## Key paths

| Path | Purpose |
|------|---------|
| `.playhouse/BRIEF.md` | Workspace QA summary |
| `.playhouse/AGENT.json` | Handoff bundle |
| `.playhouse/reports/score.json` | Last star rating |
| `.playhouse/config.json` | `default_url`, `scan_root`, `test_root`, `audit_headers` |

## Monorepo

```bash
playhouse config set scan_root apps/web
playhouse config set test_root apps/web
```

## Config

```bash
playhouse config schema --json
playhouse config set star_pass_threshold 75
```

Disable this skill: `playhouse skill disable`
