# Playhouse QA CLI — Release 0.2 Sign-Off Plan

Internal review of issues found during OWASP Juice Shop evaluation and agent workflow testing. Each item is tracked on GitHub and mapped to concrete code changes in this repo.

**Repo:** https://github.com/nicholasxdavis/playhouse-cli  
**Status:** Release 0.2.0 ready to tag (all issues #2-#35 closed)

---

## Summary

Fourteen bugs and gaps were confirmed against the current Rust codebase. They fall into three groups:

| Group | Count | Risk |
|-------|------:|------|
| Scoring and reporting integrity | 6 | High — false pass/fail, inflated stars |
| Agent and CI workflow | 5 | Medium — slow iteration, missing context |
| Platform and configuration | 3 | Medium — Windows friction, bad config accepted |

Nothing in this list blocks building or running Playhouse today. Several items can produce misleading verify results. Those are marked **P0** and should ship before the next public release.

---

## GitHub issue map

| # | GitHub | Title | Priority |
|---|--------|-------|----------|
| 1 | [#2](https://github.com/nicholasxdavis/playhouse-cli/issues/2) | Trivy skips dot-directories | P0 |
| 2 | [#3](https://github.com/nicholasxdavis/playhouse-cli/issues/3) | Skipped audits inflate score | P0 |
| 3 | [#4](https://github.com/nicholasxdavis/playhouse-cli/issues/4) | Trivy cache and secret count mismatch | P0 |
| 4 | [#5](https://github.com/nicholasxdavis/playhouse-cli/issues/5) | JSON exitCode mismatch | P0 |
| 5 | [#6](https://github.com/nicholasxdavis/playhouse-cli/issues/6) | Doctor skips node_modules check | P1 |
| 6 | [#7](https://github.com/nicholasxdavis/playhouse-cli/issues/7) | Test pattern not on functional/verify | P1 |
| 7 | [#8](https://github.com/nicholasxdavis/playhouse-cli/issues/8) | No auth for Arkenar/Lighthouse | P2 |
| 8 | [#9](https://github.com/nicholasxdavis/playhouse-cli/issues/9) | No failure output in JSON | P1 |
| 9 | [#10](https://github.com/nicholasxdavis/playhouse-cli/issues/10) | stdout vs score.json skip mismatch | P0 |
| 10 | [#11](https://github.com/nicholasxdavis/playhouse-cli/issues/11) | Windows EPERM/EBUSY locks | P2 |
| 11 | [#12](https://github.com/nicholasxdavis/playhouse-cli/issues/12) | No server lifecycle in verify | P2 |
| 12 | [#13](https://github.com/nicholasxdavis/playhouse-cli/issues/13) | Config values not validated | P1 |
| 13 | [#14](https://github.com/nicholasxdavis/playhouse-cli/issues/14) | SKILL.md too large | P1 |
| 14 | [#15](https://github.com/nicholasxdavis/playhouse-cli/issues/15) | Scanner crash scores 100/100 | P0 |
| 15 | [#16](https://github.com/nicholasxdavis/playhouse-cli/issues/16) | Github Actions unpinned action references | P0 |
| 16 | [#17](https://github.com/nicholasxdavis/playhouse-cli/issues/17) | Github Actions workflow cache poisoning risk | P0 |
| 17 | [#18](https://github.com/nicholasxdavis/playhouse-cli/issues/18) | Github Actions excessive permissions | P1 |
| 18 | [#19](https://github.com/nicholasxdavis/playhouse-cli/issues/19) | Credential persistence in workflow artifacts | P1 |
| 19 | [#20](https://github.com/nicholasxdavis/playhouse-cli/issues/20) | High cyclomatic complexity in render_score_report | P1 |
| 20 | [#21](https://github.com/nicholasxdavis/playhouse-cli/issues/21) | Deeply nested control flow in ui_blocks or handlers | P1 |
| 21 | [#22](https://github.com/nicholasxdavis/playhouse-cli/issues/22) | High file complexity in ui_blocks | P1 |
| 22 | [#23](https://github.com/nicholasxdavis/playhouse-cli/issues/23) | Similar code duplication in TUI splash and mascot | P1 |
| 23 | [#24](https://github.com/nicholasxdavis/playhouse-cli/issues/24) | High parameter count in render_tool_call | P1 |
| 24 | [#25](https://github.com/nicholasxdavis/playhouse-cli/issues/25) | Vulnerable lru dependency RUSTSEC-2026-0002 | P1 |
| 25 | [#26](https://github.com/nicholasxdavis/playhouse-cli/issues/26) | Complex boolean logic in score or detect | P1 |
| 26 | [#27](https://github.com/nicholasxdavis/playhouse-cli/issues/27) | Identical code duplication in splash and mascot | P1 |
| 27 | [#28](https://github.com/nicholasxdavis/playhouse-cli/issues/28) | High return statement count in detect_at | P1 |
| 28 | [#29](https://github.com/nicholasxdavis/playhouse-cli/issues/29) | Actionlint expression secrets context validation error | P1 |
| 29 | [#30](https://github.com/nicholasxdavis/playhouse-cli/issues/30) | Vulnerable paste dependency RUSTSEC-2024-0436 | P1 |
| 30 | [#31](https://github.com/nicholasxdavis/playhouse-cli/issues/31) | Template injection vulnerability in GitHub workflows | P2 |
| 31 | [#32](https://github.com/nicholasxdavis/playhouse-cli/issues/32) | GitHub Action workflows static token use | P2 |
| 32 | [#33](https://github.com/nicholasxdavis/playhouse-cli/issues/33) | Rubocop Style/StringLiterals violation | P2 |
| 33 | [#34](https://github.com/nicholasxdavis/playhouse-cli/issues/34) | Rubocop Style/FrozenStringLiteralComment violation | P2 |
| 34 | [#35](https://github.com/nicholasxdavis/playhouse-cli/issues/35) | Rubocop Style/Documentation violation | P2 |


---

## Release phases

### Phase A — P0 (must ship)

Scoring and trust fixes. Verify must not pass when it should fail, and JSON must match shell exit codes.

1. **#3** — Stop weight rebalancing from inflating stars when browser audits are skipped without an explicit opt-out.
2. **#5** — Align per-engine JSON `exitCode` with Playhouse exit codes (not raw Trivy exit 0 on findings).
3. **#15** — Treat missing or empty scanner output as failure, not 100/100.
4. **#10** — Single skip/fail decision shared by progress output, `audit_json`, and `score.json`.
5. **#4** — Consistent Trivy counts between stdout, engine metrics, and `score.json`; fresh cache on verify.
6. **#2** — Include dot-directories (e.g. `.well-known/`) in Trivy filesystem scans.

### Phase B — P1 (should ship)

Agent and operator quality of life.

7. **#9** — Add `failureOutput` to functional engine JSON on test failure.
8. **#7** — Add pattern arg to `playhouse functional` and optional `--test` on verify (playwright already supports pattern on `playhouse playwright`).
9. **#13** — Validate `default_url` format and require `scan_root` / `test_root` paths to exist on disk.
10. **#6** — Doctor checks that workspace `node_modules` (or lockfile install state) is present when Node is required.
11. **#14** — Shorter default skill template; document existing `playhouse skill disable` and `playhouse_skill_enabled`.

### Phase C — P2 (follow-up)

Larger features. Can ship in 0.2.x after Phase A/B if time is short.

12. **#8** — Workspace `audit_headers` for Lighthouse and Arkenar.
13. **#12** — Optional `--start-server` / `--server-port` on verify.
14. **#11** — Windows retry/skip logic when `node_modules` is locked.

### Phase D — CI security P0

15. **#16** — Pin third-party GitHub Actions to commit SHAs.
16. **#17** — Harden rust-cache keys and save-if for fork PRs.

### Phase E — Code quality P1

17. **#18** through **#30** — Workflow least privilege, artifact hygiene, TUI refactors, dependency audits, detect_at cleanup.

### Phase F — CI and packaging P2

18. **#31** — Template injection hardening in workflow run steps.
19. **#32** — npm OIDC trusted publishing (no long-lived NPM_TOKEN).
20. **#33** through **#35** — Homebrew formula Rubocop compliance.

---

## Issue detail (confirmed against codebase)

### 1. Trivy skips dot-directories — [#2](https://github.com/nicholasxdavis/playhouse-cli/issues/2)

**Where:** `src/engines/trivy.rs` runs `trivy fs .` under `scan_root` with no override for Trivy's default dot-directory skip.

**Impact:** Files under `.well-known/` (CSAF, `security.txt`, accidental secrets) are not scanned.

**Fix:** Pass Trivy flags to scan hidden directories, or run a second targeted pass on `.well-known`. Add `trivy_skip_dirs` workspace setting defaulting to `node_modules,.git` only (not all dot dirs). Document in `trivy.md`.

**Acceptance:** A secret placed under `.well-known/` is reported by `playhouse trivy --json`.

---

### 2. Skipped audits inflate score — [#3](https://github.com/nicholasxdavis/playhouse-cli/issues/3)

**Where:** `src/score.rs` — `compute()` skips `er.skipped` engines; `weighted_stars()` redistributes weight across remaining categories. `skip_lighthouse_without_server` defaults to `true` in `src/config.rs`.

**Impact:** With no URL, Arkenar and Lighthouse are skipped and their 20% weights move to Trivy, functional, and toolchain. A project can pass at ~88 stars while web audits would score 0 if run.

Example (from Juice Shop eval):
- All engines: `(88×0.10) + (75×0.25) + (100×0.25) + (0×0.20) + (0×0.20) = 52.5` — fail
- Web audits skipped: `(88×0.167) + (75×0.417) + (100×0.417) = 87.6` — pass

**Fix:**
- Split skip reasons: **explicit** (`skip_*_in_verify`, stack N/A) vs **implicit** (no URL, server unreachable).
- Implicit skips on web stacks: score category 0/100 or fail verify unless `skip_lighthouse_without_server` is intentionally set.
- Update `METHODOLOGY` string and `stars.md` to match new rules.
- Add regression test replacing `skipped_engines_rebalance_weights` if behavior changes.

**Acceptance:** `playhouse verify --json` with no URL on a web project does not pass above threshold unless user explicitly opts out.

---

### 3. Trivy cache and count mismatch — [#4](https://github.com/nicholasxdavis/playhouse-cli/issues/4)

**Where:** `src/engines/trivy.rs` — no `--cache-dir` or cache clear; `count_findings()` filters by `--severity` but stdout human text and `score.json` can diverge when cache is stale.

**Impact:** stdout may show 10 secrets while `score.json` summary shows 1.

**Fix:**
- On verify, use `--cache-dir` under `.playhouse/cache/trivy` or pass `--clear-cache` once per verify run.
- Single `count_findings()` result drives stdout, engine metrics, and `score.rs`.
- Add `rawSecretCount` vs `filteredSecretCount` in JSON if severity filter applies.

**Acceptance:** After adding test secrets, `playhouse trivy --json` and `.playhouse/reports/score.json` report the same secret count without a manual cache clear.

---

### 4. JSON exitCode mismatch — [#5](https://github.com/nicholasxdavis/playhouse-cli/issues/5)

**Where:** `src/engines/trivy.rs` sets `"exitCode": exit` to Trivy's process exit (often `0` even when secrets exist). Playhouse returns exit `4` for findings. Engine metrics JSON shows `"exitCode": 0` with `"passed": false`.

**Impact:** Agents parsing engine-level JSON treat failures as success.

**Fix:**
- Set metrics `exitCode` to the Playhouse code returned from `execute()` (0, 1, 2, 3, 4, 5).
- Optionally add `toolExitCode` for the raw subprocess exit.
- Apply same pattern in `playwright.rs`, `arkenar.rs`, `lighthouse.rs`.

**Acceptance:** When `playhouse trivy --json` exits 4, every `exitCode` field in stdout JSON is `4`.

---

### 5. Doctor skips node_modules — [#6](https://github.com/nicholasxdavis/playhouse-cli/issues/6)

**Where:** `src/detect.rs` `run_doctor()` checks global tools and `@playwright/test` binary, not general install state.

**Impact:** Doctor reports ready when `npm install` was never run.

**Fix:** When `profile.needs_node()`, add check for `node_modules` under `scan_root` (or lockfile hash vs install marker). Use detected package manager (`src/pkgmgr.rs`). Warn, do not hard-fail, with fix hint `npm install` / `pnpm install`.

**Acceptance:** Empty `node_modules` on a Node web project yields a doctor warn, not pass.

---

### 6. Test pattern on functional/verify — [#7](https://github.com/nicholasxdavis/playhouse-cli/issues/7)

**Where:** `playhouse playwright <pattern>` works (`src/cli/args.rs`, `src/engines/playwright.rs`). `playhouse functional` has no pattern arg. `audit.rs` calls `functional::execute(..., None)`.

**Impact:** Verify always runs the full suite. Agents cannot target one spec during iteration.

**Fix:**
- Add optional `pattern` to `Functional` command in `args.rs`.
- Add `--test <pattern>` to `Verify` command; pass through `audit::run_audit`.
- Extend `cargo`, `pytest`, `npm_test` runners to accept filter args where supported.

**Acceptance:** `playhouse functional webhook.unit.test.ts --json` and `playhouse verify --test webhook --json` run a subset.

---

### 7. No auth for web audits — [#8](https://github.com/nicholasxdavis/playhouse-cli/issues/8)

**Where:** `src/engines/lighthouse.rs` and `src/engines/arkenar.rs` take URL only. Baseplate `src/assets/baseplates/web-auth.spec.ts` covers Playwright auth only.

**Impact:** DAST and Lighthouse scan the login page only on authenticated apps.

**Fix:**
- Add workspace key `audit_headers` (map of header name to value) in `WorkspaceConfig` and `config_cli.rs`.
- Lighthouse: pass `--extra-headers` JSON.
- Arkenar: pass headers if CLI supports them; document limits.
- Never store secrets in committed config; support env var substitution in a follow-up.

**Acceptance:** With `audit_headers` set, Lighthouse scores a route that returns 401 without headers.

---

### 8. No failure output in JSON — [#9](https://github.com/nicholasxdavis/playhouse-cli/issues/9)

**Where:** `src/engines/playwright.rs` and other functional runners store stats only; stderr is dropped.

**Impact:** JSON shows `"failed": 1` with no reason.

**Fix:** On non-zero exit, attach last 50 lines of stderr (and stdout if parse failed) as `failureOutput` in engine metrics. Cap size at 8 KB.

**Acceptance:** A failing test returns `failureOutput` containing the assertion message in `playhouse playwright --json`.

---

### 9. stdout vs score.json skip mismatch — [#10](https://github.com/nicholasxdavis/playhouse-cli/issues/10)

**Where:** `src/audit.rs` `run_browser_audits()` marks engines `skipped: true` when `target_url` is `None`. When a URL is set but unreachable, engines run, fail, and appear as failures in `score.json`. Progress callbacks and human summary can disagree depending on code path. `score::save_report()` writes the same `engines` vec used for scoring.

**Impact:** Operators see "skipped" in one view and exit 2/5 failures in `score.json`.

**Fix:**
- Add URL reachability probe (`detect::probe_url`) before browser audits.
- If unreachable and `skip_lighthouse_without_server`: mark engine skipped with reason `url-unreachable` in engines, categories, progress, and `score.json`.
- If unreachable and setting is false: fail with clear error, score 0.

**Acceptance:** Same skip/fail status in terminal summary, `audit_json`, and `.playhouse/reports/score.json`.

---

### 10. Windows file locks — [#11](https://github.com/nicholasxdavis/playhouse-cli/issues/11)

**Where:** `src/install.rs`, `src/pkgmgr.rs` — no retry on EPERM/EBUSY during npm installs triggered by verify.

**Impact:** Verify can fail and leave partial `node_modules` on Windows when IDE locks files.

**Fix:** Retry install steps 3 times with backoff. If `node_modules` exists and lock persists, skip reinstall and warn. Document in AGENTS.md Windows section.

**Acceptance:** Verify completes or fails cleanly when `node_modules` is locked but already populated.

---

### 11. No server lifecycle — [#12](https://github.com/nicholasxdavis/playhouse-cli/issues/12)

**Where:** `src/cli/handlers.rs` `run_verify()` resolves URL but never spawns a dev server.

**Impact:** Agents manually start servers and often leave orphan processes.

**Fix:** Add optional flags to `Verify`:
- `--start-server "npm run dev"`
- `--server-port 3000`
- `--server-timeout 120`

Spawn process, poll `detect::probe_ports`, run audits, kill process group on exit. Store command in workspace config `dev_server_command` as a later enhancement.

**Acceptance:** `playhouse verify --start-server "npm start" --server-port 3000 --json` runs browser audits without a pre-running server.

---

### 12. Config validation gaps — [#13](https://github.com/nicholasxdavis/playhouse-cli/issues/13)

**Where:** `src/config_cli.rs` validates `scan_root` / `test_root` path traversal only (`validate_workspace_subpath` does not check directory exists). `default_url` accepts any string.

**Impact:** `playhouse config set default_url abc` saves; verify crashes later.

**Fix:**
- Parse `default_url` with `url` crate; require `http` or `https` scheme.
- After `resolve_subpath`, require `fs::metadata` is a directory for `scan_root` and `test_root`.
- Return clear error from `playhouse config set` on invalid input.

**Acceptance:** Invalid URL and missing directory are rejected at set time with a useful message.

---

### 13. SKILL.md size — [#14](https://github.com/nicholasxdavis/playhouse-cli/issues/14)

**Where:** `src/assets/playhouse_skill.md` (~218 lines). `playhouse init` installs via `workspace::install_playhouse_skill()`. Disable path exists: `playhouse skill disable`, `playhouse_skill_enabled` global setting.

**Impact:** Agents load a large file that overlaps AGENTS.md and `playhouse agent --json`.

**Fix:**
- Cut template to ~80 lines: commands table, JSON rule, verify checklist, pointer to manifest.
- Default `playhouse_skill_enabled` stays true; document disable in skill header.
- Add `playhouse agent --json` as the live source of truth in the trimmed skill.

**Acceptance:** New `playhouse init` writes a skill under 100 lines; `playhouse skill disable` still works.

---

### 14. Scanner crash scores 100/100 — [#15](https://github.com/nicholasxdavis/playhouse-cli/issues/15)

**Where:**
- `src/score.rs` `score_lighthouse()` — empty score values + `exit_code == 0` yields 100 stars.
- `src/engines/arkenar.rs` — empty JSON `{}` parses as zero findings (pass).
- `reportParseError` path already scores 0 (partial fix exists).

**Impact:** Crashed or empty scans look like perfect security or performance.

**Fix:**
- Require non-empty scores for Lighthouse pass; set `scanComplete: false` in metrics on failure.
- Arkenar: fail if report file missing, parse error, or zero bytes stdout and non-zero exit.
- `engine_ok()` in `score.rs` should treat `scanComplete: false` as fail.

**Acceptance:** Deleting `.playhouse/reports/arkenar.json` mid-run or killing Arkenar results in 0 stars and verify fail, not 100/100.

---

### 15. Github Actions unpinned action references - [#16](https://github.com/nicholasxdavis/playhouse-cli/issues/16)

**Where:** Workflow files in `.github/workflows/`

**Impact:** Security vulnerability (supply chain attack) if third-party actions are compromised.

**Fix:** Pin third-party actions to specific commit SHAs rather than tag versions.

**Acceptance:** No unpinned third-party actions exist in workflows.

---

### 16. Github Actions workflow cache poisoning risk - [#17](https://github.com/nicholasxdavis/playhouse-cli/issues/17)

**Where:** Cache actions in `.github/workflows/`

**Impact:** Possible cache poisoning where untrusted code or dependencies are injected into CI.

**Fix:** Audit and restrict cache keys and write configurations.

**Acceptance:** Caching mechanisms are locked down against branch cache poisoning.

---

### 17. Github Actions excessive permissions - [#18](https://github.com/nicholasxdavis/playhouse-cli/issues/18)

**Where:** Workflow configurations in `.github/workflows/`

**Impact:** Unnecessary write or modify access increases token compromise blast radius.

**Fix:** Set explicit minimum permissions for each job.

**Acceptance:** Jobs run with minimum required privilege scopes.

---

### 18. Credential persistence in workflow artifacts - [#19](https://github.com/nicholasxdavis/playhouse-cli/issues/19)

**Where:** Build artifact definitions in `.github/workflows/`

**Impact:** Sensitive credentials or configuration details could persist in uploaded artifacts.

**Fix:** Exclude configuration or credential files from uploaded artifacts.

**Acceptance:** Stored artifacts are audited to be clean of secrets.

---

### 19. High cyclomatic complexity in render_score_report - [#20](https://github.com/nicholasxdavis/playhouse-cli/issues/20)

**Where:** src/tui/ui_blocks.rs

**Impact:** High code maintenance overhead and difficulty in reading/verifying code.

**Fix:** Extract parts of render_score_report into smaller helper functions.

**Acceptance:** Cyclomatic complexity count of render_score_report is reduced.

---

### 20. Deeply nested control flow in ui_blocks or handlers - [#21](https://github.com/nicholasxdavis/playhouse-cli/issues/21)

**Where:** src/tui/ui_blocks.rs, src/cli/handlers.rs

**Impact:** Control flow is overly complex and hard to trace.

**Fix:** Use early returns or match statements to simplify nesting.

**Acceptance:** Code nesting level is reduced to 3 or less.

---

### 21. High file complexity in ui_blocks - [#22](https://github.com/nicholasxdavis/playhouse-cli/issues/22)

**Where:** src/tui/ui_blocks.rs

**Impact:** Monolithic files are hard to navigate and maintain.

**Fix:** Decompose TUI logic into smaller component files.

**Acceptance:** ui_blocks.rs total complexity is under the lint limit.

---

### 22. Similar code duplication in TUI splash and mascot - [#23](https://github.com/nicholasxdavis/playhouse-cli/issues/23)

**Where:** src/tui/splash.rs, src/tui/mascot.rs

**Impact:** Duplicate lines require multiple updates and invite inconsistencies.

**Fix:** Consolidate shared logic or rendering structures.

**Acceptance:** Code duplication reports for splash/mascot are resolved.

---

### 23. High parameter count in render_tool_call - [#24](https://github.com/nicholasxdavis/playhouse-cli/issues/24)

**Where:** src/tui/ui_blocks.rs

**Impact:** Method signatures are unwieldy and hard to maintain.

**Fix:** Group parameters into a single helper struct.

**Acceptance:** render_tool_call takes a config struct instead of 6 raw arguments.

---

### 24. Vulnerable lru dependency RUSTSEC-2026-0002 - [#25](https://github.com/nicholasxdavis/playhouse-cli/issues/25)

**Where:** Cargo.toml, Cargo.lock

**Impact:** Vulnerable lru dependency poses dependency audit risks.

**Fix:** Update lru version to a patched release.

**Acceptance:** osv-scanner or cargo audit reports clean build.

---

### 25. Complex boolean logic in score or detect - [#26](https://github.com/nicholasxdavis/playhouse-cli/issues/26)

**Where:** src/score.rs, src/detect.rs

**Impact:** Complex binary expressions are hard to read and test.

**Fix:** Split complex logical assertions into intermediate variables.

**Acceptance:** Conditional blocks are clean and easy to follow.

---

### 26. Identical code duplication in splash and mascot - [#27](https://github.com/nicholasxdavis/playhouse-cli/issues/27)

**Where:** src/tui/splash.rs, src/tui/mascot.rs

**Impact:** Maintenance overhead due to exact copy-pasted blocks.

**Fix:** Extract identical lines to a single shared function.

**Acceptance:** Mass of duplication is resolved.

---

### 27. High return statement count in detect_at - [#28](https://github.com/nicholasxdavis/playhouse-cli/issues/28)

**Where:** src/detect.rs

**Impact:** Multiple return paths make function logic hard to follow.

**Fix:** Simplify detect_at flow to use fewer returns.

**Acceptance:** Return statement count is reduced.

---

### 28. Actionlint expression secrets context validation error - [#29](https://github.com/nicholasxdavis/playhouse-cli/issues/29)

**Where:** .github/workflows/release.yml

**Impact:** Workflow fails compiling due to referencing secrets context incorrectly.

**Fix:** Pass secret via inputs/env or resolve the validation error.

**Acceptance:** actionlint runs on workflows without errors.

---

### 29. Vulnerable paste dependency RUSTSEC-2024-0436 - [#30](https://github.com/nicholasxdavis/playhouse-cli/issues/30)

**Where:** Cargo.toml, Cargo.lock

**Impact:** Security vulnerability in paste macros.

**Fix:** Update paste version in Cargo.toml.

**Acceptance:** osv-scanner passes on paste.

---

### 30. Template injection vulnerability in GitHub workflows - [#31](https://github.com/nicholasxdavis/playhouse-cli/issues/31)

**Where:** .github/workflows/ci.yml, .github/workflows/release.yml

**Impact:** Untrusted issue or commit strings could trigger arbitrary command execution.

**Fix:** Use env variables instead of direct string template expansions in run scripts.

**Acceptance:** Workflows are secure from template injection.

---

### 31. GitHub Action workflows static token use - [#32](https://github.com/nicholasxdavis/playhouse-cli/issues/32)

**Where:** .github/workflows/release.yml

**Impact:** Risk of static credential exposure.

**Fix:** Transition to Trusted Publishing (OIDC) where supported.

**Acceptance:** Trusted Publishing is active for package publishing.

---

### 32. Rubocop Style/StringLiterals violation - [#33](https://github.com/nicholasxdavis/playhouse-cli/issues/33)

**Where:** packaging/homebrew/playhouse.rb

**Impact:** Code style inconsistency.

**Fix:** Convert double-quoted strings to single-quoted strings where appropriate.

**Acceptance:** Rubocop reports clean formatting for string literals.

---

### 33. Rubocop Style/FrozenStringLiteralComment violation - [#34](https://github.com/nicholasxdavis/playhouse-cli/issues/34)

**Where:** packaging/homebrew/playhouse.rb

**Impact:** Lacks optimization and triggers style warnings.

**Fix:** Add # frozen_string_literal: true magic comment to the first line.

**Acceptance:** Magic comment is present in Homebrew formula.

---

### 34. Rubocop Style/Documentation violation - [#35](https://github.com/nicholasxdavis/playhouse-cli/issues/35)

**Where:** packaging/homebrew/playhouse.rb

**Impact:** Documentation gaps on class definitions.

**Fix:** Add documentation comment for class Playhouse.

**Acceptance:** Top-level class has descriptive comment.

---


## Sign-off checklist (before tagging next release)

- [x] All P0 issues closed (#2, #3, #4, #5, #10, #15)
- [x] All new static analysis P0 issues closed (#16, #17)
- [x] All P1 issues closed (#6, #7, #9, #13, #14)
- [x] All new static analysis P1 issues closed (#18, #19, #20, #21, #22, #23, #24, #25, #26, #27, #28, #29, #30)
- [x] All P2 issues closed (#8, #11, #12)
- [x] All new static analysis P2 issues closed (#31, #32, #33, #34, #35)

- [x] `cargo test` passes on Windows (63/63)
- [x] `playhouse verify --json` on this repo exits 0
- [x] `playhouse agent handoff --json` writes valid `.playhouse/AGENT.json`
- [ ] `cargo test` passes on macOS and Linux (CI on push)
- [ ] `stars.md` and `AGENTS.md` match scoring behavior after P0 fixes
- [ ] Beta issue [#1](https://github.com/nicholasxdavis/playhouse-cli/issues/1) updated with known platform notes

---

## References

| Doc | Purpose |
|-----|---------|
| `AGENTS.md` | Agent operating manual |
| `stars.md` | Star methodology |
| `.dev/runbook.md` | Release process |
| `playhouse agent --json` | Live command manifest |
