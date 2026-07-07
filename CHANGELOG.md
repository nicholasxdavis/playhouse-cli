# Changelog

All notable changes to Playhouse CLI are documented here.

## [0.3.3] — 2026-07-06

### CI
- Fix verify-smoke: run against `tests/fixtures/rust-app`, fix Trivy 0.72 flags (`--hidden`, `--log-level`) and JSON capture via `--output`
- Add `validate-release-matrix` CI step; `scripts/` TypeScript typecheck in CI

### TUI
- `/install --minimal` and `/install --full` (headless parity)
- `/skill enable` alias; restore `/version`, `/uninstall`, `/auth login`, `/test` subcommands
- `/help` grouped commands (QA / Agent / Config / Meta) with verify flag reference
- Natural-language shortcuts: version, install, score, agent handoff, skill, and more
- Integration tests for agent subcommands, install profiles, config URL validation

### Tooling
- Shared `scripts/manifest/release-targets.json` for Homebrew, npm platform map, and release matrix validation
- Migrate release scripts to TypeScript (`scripts/*.ts` + `tsx`)
- Trivy engine regression tests for `--output` / `--quiet` flag compatibility

## [0.3.2] — 2026-07-06

### CI / quality
- Fix all clippy `-D warnings` failures (cross-platform dead code, test module order)
- CI clippy green on Linux/macOS/Windows

### TUI
- Workspace config tab + `/config get|set|schema` (default_url, scan_root, etc.)
- Verify dev-server flags: `--test`, `--start-server`, `--port`, `--server-timeout`
- Handoff inherits full verify options

### Packaging
- Homebrew formula sha256 auto-updated on release (no more REPLACE_ON_RELEASE)

## [0.3.1] — 2026-07-06

### TUI

- Fix feed flow: loading spinners transition to `+` / `x` in place (no orphan rows)
- Wire 0.3.0 headless commands: `/functional`, `/status`, `/upgrade`, `/update`, agent subcommands
- Fix `/score`: runs star audit only (not full verify); `/score last` shows saved report
- Add `/doctor resolve` for native binding rebuilds
- Add `/agent handoff [url]` with verify + BRIEF + AGENT.json export
- Add `/verify [url]`, `/init --no-skill`, skill/stay-on-track disable/status
- Call `maybe_auto_init` on TUI launch (respects auto-init setting)
- Doctor/install use stack-aware `ensure_profile` (matches headless CLI)
- Slash menu, todo warn/fail states, and styling fixes

### CLI / core

- Share `detect::resolve_native_bindings` between headless doctor and TUI
- Add `score::load_saved_report` for TUI `/score last`
- Clippy fixes across agent, uninstall, trivy, detect

### Packaging

- Fix Homebrew formula Ruby syntax (`Formula` / `desc` on separate lines)
- Version sync script checks Homebrew `version` field

## [0.3.0] — 2026-07-06

- Agent workflow: `agent rules|paths|next-action`, verify progress, auth login
- Functional test runner detection, upgrade/update/uninstall commands
- Scoring integrity, CI hardening, Trivy/Arkenar improvements
- Five-phase release plan (issues #36–#75)

## [0.2.0] — earlier

- Initial public release with TUI, Playhouse Stars, bundled Trivy/Arkenar
