# AGENTS.md

**You are an LLM or coding agent.** This file is your operating manual for Playhouse. Read it before touching code, running audits, or claiming work is done.

Humans can use the README. You should not rely on it.

---

## What Playhouse is

Playhouse is a **QA CLI** for security, functional testing, performance audits, and agent handoff. It bundles:

| Engine | Tool | What it checks |
|--------|------|----------------|
| Doctor | built-in | Node, package manager, Playwright, Trivy, Arkenar health |
| Trivy | bundled binary | Filesystem vulns + leaked secrets |
| Playwright | npm/pnpm/yarn/bun | Functional / E2E tests |
| Arkenar | bundled binary | DAST web security scan (replaces OWASP ZAP) |
| Lighthouse | via Node | Performance, accessibility, best practices, SEO |

**You run headless shell commands.** Do not open or depend on the TUI (`playhouse` with no args). That is for humans.

---

## Read order (every session)

1. **This file** (`AGENTS.md`) if you are in the Playhouse CLI repo.
2. **`.playhouse/SKILL.md`** if it exists in the workspace (installed by `playhouse init`; recommended in every consumer project).
3. **`playhouse agent --json`** — live manifest with `readOrder`, `nextActions`, paths, and settings.
4. **`.playhouse/BRIEF.md`** — workspace QA summary.
5. **`.playhouse/AGENT.json`** — last handoff bundle (if present).

Always prefer `--json` on every command. Parse structured output; do not guess from exit codes alone.

---

## Golden rules

1. **Headless only** — shell commands, not the TUI.
2. **JSON first** — use `--json` unless the user explicitly wants human text.
3. **Never skip verify** before calling work production-ready or complete.
4. **Fix and re-run** until exit code is `0` and Playhouse Stars meet the pass threshold.
5. **Do not commit secrets** flagged by Trivy.
6. **Follow `nextActions`** from `playhouse agent status --json` when unsure what to do next.
7. **Do not invent commands** — only use what is documented here or in `playhouse agent --json`.

---

## Bootstrap (new workspace)

```bash
playhouse agent --json
playhouse init --json
playhouse install
playhouse doctor --json
playhouse skill install    # .playhouse/SKILL.md (on by default)
```

If doctor reports missing tools: `playhouse install` then re-run doctor.

---

## Agent commands (start here)

| Command | Purpose |
|---------|---------|
| `playhouse agent --json` | Full manifest: commands, settings, readOrder, nextActions, handoff checklist |
| `playhouse agent status --json` | Quick health, last score, recommended next actions |
| `playhouse agent plan --json` | Phased workflow (start → during → handoff) |
| `playhouse agent handoff [--url URL] --json` | Run full verify + write `.playhouse/AGENT.json` |

The manifest is the source of truth when this file and the repo drift. Run it first in any unfamiliar workspace.

---

## All commands

Global flag: **`--json`** on any subcommand for machine-readable output.

### Workspace and tools

| Command | Description |
|---------|-------------|
| `playhouse init [--stay-on-track] [--json]` | Create `.playhouse/`, install tools, write BRIEF, install agent skill |
| `playhouse install [--json]` | Install bundled Trivy, Arkenar, Playwright |
| `playhouse doctor [--json]` | Tool health check |
| `playhouse export [--json]` | Write or refresh `.playhouse/BRIEF.md` |

### QA engines (run individually)

| Command | Description |
|---------|-------------|
| `playhouse trivy [--json]` | Static security + secret scan |
| `playhouse playwright [pattern] [--json]` | Run Playwright tests; optional file or grep pattern |
| `playhouse arkenar [url] [--json]` | DAST web scan; URL auto-detected if omitted |
| `playhouse lighthouse [url] [--json]` | Performance / a11y / SEO audit; URL auto-detected if omitted |

### Full audits

| Command | Description |
|---------|-------------|
| `playhouse verify [--url URL] [--json]` | All engines + Playhouse Stars (0–100) |
| `playhouse score [--url URL] [--json]` | Star rating audit (same engines as verify) |
| `playhouse score --last [--json]` | Read last score from `.playhouse/reports/score.json` |

### Configuration

| Command | Description |
|---------|-------------|
| `playhouse config [--json]` | Dump global + workspace settings and paths |
| `playhouse config schema [--json]` | List all settable keys |
| `playhouse config get <key> [--json]` | Read one setting |
| `playhouse config set <key> <value> [--json]` | Update one setting |

### Skills

| Command | Description |
|---------|-------------|
| `playhouse skill install` | Install or refresh `.playhouse/SKILL.md` (recommended) |
| `playhouse skill enable` | Same as install |
| `playhouse skill disable` | Disable workspace playhouse skill flag |
| `playhouse skill status [--json]` | Skill path, enabled state, exists on disk |
| `playhouse stay-on-track enable` | Optional: install `.playhouse/stay-on-track/SKILL.md` + `PROJECT.md` |
| `playhouse stay-on-track disable` | Disable stay-on-track for workspace |
| `playhouse stay-on-track status [--json]` | Stay-on-track status |

### Human-only

| Command | Description |
|---------|-------------|
| `playhouse` (no args) | Opens TUI — **do not use this** |

---

## Exit codes

| Code | Meaning | Action |
|------|---------|--------|
| `0` | Pass | OK to proceed |
| `1` | Test or verify failure | Read JSON output, fix, re-run |
| `2` | Lighthouse below threshold | Improve perf/a11y/SEO or adjust `lighthouse_threshold` |
| `3` | Arkenar high/medium findings | Fix DAST findings |
| `4` | Trivy vulns or secrets | Fix or remove secrets; never commit them |
| `5` | Required tool missing | Run `playhouse install` |

Verify fails if **any** engine fails **or** Playhouse Stars are below `star_pass_threshold` (default **75**).

---

## Playhouse Stars (0–100)

Combined audit score after verify or score. Report: `.playhouse/reports/score.json`.

| Stars | Grade | Meaning |
|------:|-------|---------|
| 90–100 | Production Ready | Ship with confidence |
| 75–89 | Good | Solid; minor gaps |
| 60–74 | Fair | Address weak categories |
| 40–59 | Needs Work | Significant QA debt |
| 0–39 | Critical | Do not ship |

**Default pass threshold: 75** (`star_pass_threshold`).

### Category weights

| Category | Weight | Engine |
|----------|-------:|--------|
| Toolchain | 10% | doctor |
| Security (static) | 25% | Trivy |
| Functional | 25% | Playwright |
| Security (DAST) | 20% | Arkenar |
| Performance & UX | 20% | Lighthouse |

Skipped engines are excluded; remaining weights rebalance.

```bash
playhouse verify --json
playhouse score --last --json
```

See [stars.md](stars.md) for full methodology.

---

## URL resolution (Lighthouse + Arkenar + verify)

Browser audits need a URL. Resolution order:

1. `--url` flag on the command
2. Workspace `default_url` in `.playhouse/config.json`
3. Global `default_lighthouse_url` in Playhouse settings
4. Auto-detected local dev server

Set workspace URL:

```bash
playhouse config set default_url http://localhost:3000
```

If no URL is available and `skip_lighthouse_without_server` is true, Lighthouse may be skipped (stars rebalance).

---

## Configuration reference

Run `playhouse config schema --json` for the live list.

### Global settings (`~/.config/playhouse/settings.json` or platform equivalent)

| Key | Type | Description |
|-----|------|-------------|
| `package_manager` | string | `auto`, `npm`, `pnpm`, `yarn`, `bun` |
| `star_pass_threshold` | u8 | Min stars (0–100) to pass verify (default 75) |
| `lighthouse_threshold` | f64 | Min Lighthouse score 0.0–1.0 per category |
| `default_lighthouse_url` | string\|null | Default URL for browser audits |
| `trivy_severity` | string | e.g. `HIGH,CRITICAL` |
| `json_output_default` | bool | Commands default to JSON output |
| `auto_install_tools` | bool | Auto-install when tools missing |
| `auto_export_agent_brief` | bool | Write BRIEF.md after verify |
| `auto_export_handoff_json` | bool | Write AGENT.json after verify/handoff |
| `agent_mode` | bool | Agent-friendly defaults |
| `skip_playwright_in_verify` | bool | Skip Playwright in verify |
| `skip_trivy_in_verify` | bool | Skip Trivy in verify |
| `skip_arkenar_in_verify` | bool | Skip Arkenar in verify |
| `skip_lighthouse_without_server` | bool | Skip browser audits when no URL |
| `stay_on_track_enabled` | bool | Enable stay-on-track skill by default (`.playhouse/stay-on-track/`) |
| `playhouse_skill_enabled` | bool | Install `.playhouse/SKILL.md` (default **true**) |
| `arkenar_advanced_mode` | bool | Arkenar DAST advanced profile |

### Workspace settings (`.playhouse/config.json`)

| Key | Type | Description |
|-----|------|-------------|
| `default_url` | string\|null | Workspace verify URL |
| `agent_notes` | string\|null | Notes for agents in this repo |
| `project_name` | string\|null | Display name |

```bash
playhouse config set package_manager pnpm
playhouse config set star_pass_threshold 75
playhouse config set default_url http://localhost:3000
```

---

## File map

### In any Playhouse-enabled project

```
project-root/
  AGENTS.md                 # Agent doc (Playhouse CLI repo) or project-specific
  .playhouse/
    SKILL.md                # Agent skill (recommended; installed by playhouse init)
    BRIEF.md                # Workspace QA brief
    AGENT.json              # Handoff bundle (verify/handoff output)
    config.json             # Workspace config
    advisories.log          # Human notes from TUI
    stay-on-track/          # Optional discipline skill
      SKILL.md
      PROJECT.md
    reports/
      score.json            # Last Playhouse Star rating
```

### Playhouse CLI repo layout (this repository)

```
src/
  main.rs           # CLI entry, subcommands
  agent.rs          # Manifest, status, plan, handoff, BRIEF
  audit.rs          # Full verify suite + scoring
  score.rs          # Playhouse Stars (0–100)
  workspace.rs      # Init, skills, workspace config
  config.rs         # Global settings
  config_cli.rs     # config get/set/schema
  install.rs        # Bundled tool install
  pkgmgr.rs         # npm/pnpm/yarn/bun abstraction
  engines/          # lighthouse, playwright, trivy, arkenar
  assets/
    playhouse_skill.md      # Template for .playhouse/SKILL.md
    stay_on_track_skill.md  # Template for .playhouse/stay-on-track/SKILL.md
```

---

## Workflows

### Start of session

```bash
playhouse agent --json
playhouse agent status --json
playhouse agent plan --json
playhouse doctor --json
```

### During development (targeted checks)

```bash
playhouse playwright --json
playhouse trivy --json
playhouse lighthouse --json
playhouse arkenar --json
```

### Before handoff (required)

```bash
playhouse verify --json
playhouse agent handoff --json
playhouse export
```

Do not mark work complete until:

- [ ] `playhouse doctor --json` — tools healthy
- [ ] `playhouse verify --json` — exit `0`
- [ ] Playhouse Stars ≥ pass threshold (default 75)
- [ ] No Trivy HIGH/CRITICAL or secrets
- [ ] Playwright tests pass
- [ ] Lighthouse scores above threshold
- [ ] `.playhouse/AGENT.json` written

---

## Package managers

Playwright and Lighthouse run via **npm, pnpm, yarn, or bun**.

- Setting: `package_manager` = `auto` | `npm` | `pnpm` | `yarn` | `bun`
- `auto` detects from lockfiles: `bun.lockb` → bun, `pnpm-lock.yaml` → pnpm, `yarn.lock` → yarn, `package-lock.json` → npm

---

## Engine deep dives

| Doc | Engine |
|-----|--------|
| [playwright.md](playwright.md) | Functional tests |
| [trivy.md](trivy.md) | Security + secrets |
| [lighthouse.md](lighthouse.md) | Performance / a11y / SEO |
| [arkenar.md](arkenar.md) | DAST web scan |
| [stars.md](stars.md) | Playhouse Star Rating |

---

## Quick recipes

**Full audit**

```bash
playhouse verify --url http://localhost:3000 --json
playhouse score --last --json
```

**Handoff bundle**

```bash
playhouse agent handoff --url http://localhost:3000 --json
```

**Refresh agent skill in workspace**

```bash
playhouse skill install
```

**Enable optional stay-on-track discipline**

```bash
playhouse init --stay-on-track
# or
playhouse stay-on-track enable
```

---

## What not to do

- Do not use the TUI (`playhouse` with no args).
- Do not skip `playhouse verify` before claiming done.
- Do not ignore exit codes or Trivy secret findings.
- Do not manually install Trivy/Arkenar/Playwright when `playhouse install` handles it.
- Do not read README.md for operational detail — use this file and `playhouse agent --json`.
