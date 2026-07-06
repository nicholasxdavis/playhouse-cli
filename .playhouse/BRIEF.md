# Playhouse Workspace Brief

Project: playhouse cli
Workspace: C:\Users\lifa2\OneDrive\Desktop\playhouse cli
Verify URL: none
Stack: rust | Functional runner: cargo-test | Browser audits: false
Tools ready: 4/4
Last score: 100/100 (Production Ready)
Stars pass threshold: 75/100
Package manager: auto
Lighthouse threshold: 50%
Trivy severity: HIGH,CRITICAL
Stay-on-track: enabled
Playhouse skill: enabled (recommended) (`C:\Users\lifa2\OneDrive\Desktop\playhouse cli\.playhouse\SKILL.md`)
Agent notes: none

## Agent workflow

1. `playhouse agent --json` - full manifest (read first)
2. `playhouse agent status --json` - quick health check
3. `playhouse agent plan --json` - phased workflow for this repo
4. `playhouse agent handoff --json` - run verify and export handoff bundle
5. `playhouse config schema --json` - all settable keys

## Headless commands

```bash
playhouse doctor --json
playhouse install
playhouse init [--stay-on-track]
playhouse verify [--url URL] --json
playhouse score [--url URL] [--last] --json
playhouse playwright [pattern] --json
playhouse trivy --json
playhouse arkenar [url] --json
playhouse lighthouse [url] --json
playhouse config get|set <key> [value] --json
playhouse export
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Pass |
| 1 | Test or verify failure |
| 2 | Lighthouse below threshold |
| 3 | Arkenar high/medium findings |
| 4 | Trivy findings |
| 5 | Tool missing - run playhouse install |

## Handoff checklist

1. Run `playhouse verify --json` or `playhouse agent handoff --json`
2. Fix Playwright failures, Arkenar findings, Trivy HIGH/CRITICAL
3. Lighthouse scores above 50%
4. Playhouse Stars at or above 75/100
5. Never commit secrets
6. If stay-on-track enabled, complete `C:\Users\lifa2\OneDrive\Desktop\playhouse cli\.playhouse\stay-on-track\SKILL.md` first

## Config files

- Global: `~/.config/playhouse/settings.json` (or platform equivalent)
- Workspace: `.playhouse/config.json`
- Handoff bundle: `.playhouse/AGENT.json`
- Score report: `.playhouse/reports/score.json`
