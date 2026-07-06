---
name: playhouse
description: Read this skill when working in a project that uses Playhouse QA CLI. Tells agents exactly how to run headless audits, interpret scores, and hand off work. Recommended for every Playhouse-enabled workspace.
recommended: true
---

# Playhouse Agent Skill

You are in a project that uses **Playhouse**, a QA CLI for security, functional, performance, and agent handoff workflows. This file is for **agents**. Humans may use the TUI (`playhouse` with no args); you should use **headless commands** only.

## Read this first

1. Run `playhouse agent --json` for the live manifest (paths, settings, next actions).
2. Read `.playhouse/BRIEF.md` for this workspace summary.
3. Follow `readOrder` and `nextActions` from the agent manifest.
4. Use `--json` on every command unless the user explicitly wants human text.

## What Playhouse does

| Engine | Command | Purpose |
|--------|---------|---------|
| Doctor | `playhouse doctor --json` | Tool health (Node, npm, Playwright, Trivy, Arkenar) |
| Trivy | `playhouse trivy --json` | Static security + secret scan |
| Functional | `playhouse functional --json` | Run detected test runner (cargo, playwright, pytest, …) |
| Test scaffold | `playhouse test list\|init\|add\|run --json` | Starter test baseplates per stack |
| Arkenar | `playhouse arkenar --json` | DAST web security scan |
| Lighthouse | `playhouse lighthouse --json` | Performance, a11y, SEO |
| Verify | `playhouse verify --json` | Full suite + Playhouse Stars (0-100) |
| Score | `playhouse score --json` | Star rating audit |
| Handoff | `playhouse agent handoff --json` | Verify + export handoff bundle |

Bundled tools install via `playhouse install` (no manual Trivy/Playwright setup).

## Agent workflow

### Start of session

```bash
playhouse agent --json
playhouse agent status --json
playhouse agent plan --json
playhouse doctor --json
```

If tools are missing: `playhouse install`

If workspace is new: `playhouse init --json`

If no tests exist: `playhouse test list --json` then `playhouse test init --json` (uses stack default plate)

### During development

Run targeted checks as you change code:

```bash
playhouse test run --json
playhouse functional --json
playhouse playwright --json
playhouse trivy --json
playhouse lighthouse --json
playhouse arkenar --json
```

### Test baseplates

When `tests.detected` is false in the agent manifest, scaffold starters:

```bash
playhouse test list --json
playhouse test init --json
playhouse test init --plate web-smoke --json
playhouse test add --plate web-a11y --json
playhouse test run --json
```

Plates: `web-smoke`, `web-auth`, `web-a11y`, `api-health`, `rust-lib`, `python-pytest`, `go-http`. Manifest: `.playhouse/tests/manifest.json`.

### Before handoff (required)

```bash
playhouse verify --json
# or the full bundle:
playhouse agent handoff --json
```

Do not mark work complete until verify exits **0** and Playhouse Stars meet the pass threshold.

## Playhouse Stars (0-100)

Combined audit score (Lighthouse-inspired). After verify:

- Report: `.playhouse/reports/score.json`
- Read last score: `playhouse score --last --json`
- Default pass threshold: **75/100** (configurable)

Grades: 90+ Production Ready, 75+ Good, 60+ Fair, below 60 needs work.

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Pass |
| 1 | Test or verify failure |
| 2 | Lighthouse below threshold |
| 3 | Arkenar high/medium findings |
| 4 | Trivy vulnerabilities or secrets |
| 5 | Tool missing - run `playhouse install` |

Always check `exitCode` in JSON output.

## Shell commands (agents on Windows)

PowerShell 5.x does not support `&&`. Never run `cd path && playhouse ...`.

Preferred:

```text
playhouse -C "/path/to/project" agent status --json
```

Or set your shell `working_directory` to the project root. See the `shell` block in `playhouse agent --json`.

## URL for browser audits

Lighthouse and Arkenar need a URL. Resolution order:

1. `--url` flag
2. **Workspace `default_url`** — primary, set once per project:
   ```bash
   playhouse config set default_url http://localhost:3000
   ```
3. Global `default_lighthouse_url` in `~/.config/playhouse/settings.json`
4. Live local dev server (port hints from `package.json` scripts, Vite/Astro/Wrangler config, then common ports)

Port hints appear in `playhouse agent --json` under `urls.portHints` and `urls.suggestedUrl`.

## Configuration

```bash
playhouse config --json
playhouse config schema --json
playhouse config get package_manager
playhouse config set star_pass_threshold 75
```

Useful keys: `package_manager`, `star_pass_threshold`, `lighthouse_threshold`, `default_url` (workspace, primary), `scan_root`, `test_root`, `functional_runner`, `skip_lighthouse_in_verify`, `skip_arkenar_in_verify`, `agent_notes`.

## Monorepo workspaces

Point Playhouse at a sub-package from the repo root via `.playhouse/config.json`:

```bash
playhouse config set scan_root apps/web
playhouse config set test_root apps/web
playhouse config set default_url http://localhost:3000
playhouse config set functional_runner playwright
```

- `scan_root` — stack detection, Trivy scans, port hints, Node tool resolution
- `test_root` — where `cargo test` / `playwright` / `pytest` run (defaults to `scan_root`)
- `functional_runner` — override auto-detection when needed

## File map

| Path | Purpose |
|------|---------|
| `.playhouse/SKILL.md` | This file - agent instructions (recommended) |
| `.playhouse/BRIEF.md` | Workspace QA brief |
| `.playhouse/AGENT.json` | Handoff bundle after verify/handoff |
| `.playhouse/reports/score.json` | Last Playhouse Star rating |
| `.playhouse/tests/manifest.json` | Applied test baseplates |
| `.playhouse/config.json` | Workspace settings (`default_url`, `scan_root`, `test_root`, `functional_runner`) |
| `.playhouse/stay-on-track/SKILL.md` | Optional discipline skill (if enabled) |
| `.playhouse/stay-on-track/PROJECT.md` | Project info for stay-on-track (if enabled) |

## Handoff checklist

- [ ] `playhouse doctor --json` - tools healthy
- [ ] `playhouse verify --json` - exit 0
- [ ] Playhouse Stars at or above pass threshold
- [ ] No Trivy HIGH/CRITICAL or secrets
- [ ] Playwright tests pass
- [ ] Lighthouse scores above threshold
- [ ] `.playhouse/AGENT.json` written (auto or via `playhouse agent handoff --json`)

## Rules for agents

1. **Headless only** - run shell commands, do not assume a TUI is open.
2. **JSON first** - parse structured output for decisions.
3. **Never skip verify** before claiming production-ready.
4. **Fix and re-run** until exit 0.
5. **Do not commit secrets** flagged by Trivy.
6. **Read nextActions** from `playhouse agent status --json` when unsure what to do next.

## Quick recipes

**Bootstrap**

```bash
playhouse agent --json
playhouse init --json
playhouse install
playhouse doctor --json
```

**Full audit**

```bash
playhouse verify --url http://localhost:3000 --json
playhouse score --last --json
```

**Handoff**

```bash
playhouse agent handoff --json
playhouse export
```
