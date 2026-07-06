# Changelog

All notable changes to Playhouse CLI are documented here.

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
