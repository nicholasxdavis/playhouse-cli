# Playhouse QA CLI - Release 0.2 Sign-Off Plan

Internal review of issues found during OWASP Juice Shop evaluation and agent workflow testing. Each item is tracked on GitHub and mapped to concrete code changes in this repo.

**Repo:** https://github.com/nicholasxdavis/playhouse-cli  
**Status:** Release 0.2.0 ready to tag (all issues #2 to #35 closed)

---

## Summary

Fourteen core bugs and gaps were confirmed against the current Rust codebase alongside twenty static analysis and security findings. They fall into three primary risk categories:

| Category | Count | Risk |
|---|---:|---|
| Scoring and reporting integrity | 6 | High: false pass/fail results, inflated star ratings |
| Agent and CI workflow | 7 | Medium: slow iteration loops, missing context, CI risks |
| Code quality, platform, and packaging | 21 | Medium: Windows friction, maintenance overhead, style violations |

Nothing in this list blocks building or running Playhouse today. However, several items can produce misleading verification results. Those marked P0 must ship before the next public release.

---

## GitHub issue map

| # | GitHub | Title | Priority |
|---|---|---|---|
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

## Executable 5-Phase Release Plan

### Phase 1: Scoring Integrity and Core Trust (P0)
Focus: Ensure verification never passes when it should fail, and align JSON output with shell exit codes.
1. **#3**: Stop weight rebalancing from inflating stars when browser audits are skipped without an explicit opt-out.
2. **#5**: Align per-engine JSON exitCode fields with standard Playhouse exit codes.
3. **#15**: Treat missing, empty, or crashed scanner output as an explicit failure instead of scoring 100/100.
4. **#10**: Use a single skip/fail evaluation struct shared across terminal output, audit JSON, and score reports.
5. **#4**: Maintain consistent Trivy finding counts between stdout, engine metrics, and score reports, and refresh cache on verify.
6. **#2**: Include hidden dot-directories (such as .well-known) in Trivy filesystem vulnerability and secret scans.

### Phase 2: Agent Workflow and CI Security (P0/P1)
Focus: Improve developer quality of life, test iteration speed, and lock down CI workflow security.
7. **#9**: Attach test failure output and assertions directly inside functional engine JSON reports.
8. **#7**: Add test filter pattern arguments to playhouse functional and playhouse verify.
9. **#13**: Validate default_url formats and ensure required workspace directories exist on disk before saving config.
10. **#6**: Add doctor checks to verify workspace node_modules or lockfile install state before running Node tools.
11. **#14**: Streamline default SKILL.md template size and reference the live CLI manifest.
12. **#16**: Pin all third-party GitHub Action references to exact commit SHAs.
13. **#17**: Harden Rust cache keys and restrict cache write access on pull requests from untrusted forks.

### Phase 3: Code Quality and Architecture Refactoring (P1)
Focus: Reduce code complexity, eliminate duplication, and resolve dependency vulnerabilities.
14. **#18**: Set explicit minimum permissions across all GitHub Action workflow jobs.
15. **#19**: Prevent credential and sensitive config persistence in uploaded workflow artifacts.
16. **#20**: Refactor render_score_report into focused helper functions to reduce cyclomatic complexity.
17. **#21**: Flatten deeply nested control flow in ui_blocks and CLI handlers using guard clauses and early returns.
18. **#22**: Decompose monolithic TUI modules in ui_blocks into smaller component files.
19. **#23**: Consolidate duplicate rendering structures shared between TUI splash and mascot screens.
20. **#24**: Group parameter lists in render_tool_call into a clean configuration struct.
21. **#25**: Update lru dependency to resolve advisory RUSTSEC-2026-0002.
22. **#26**: Simplify compound boolean expressions in score and detect modules into well-named variables.
23. **#27**: Extract identical ASCII banner and layout initialization lines from splash and mascot into a shared helper.
24. **#28**: Simplify return paths in detect_at using structured matching.
25. **#29**: Resolve actionlint expression validation errors in release workflows.
26. **#30**: Update paste dependency to resolve advisory RUSTSEC-2024-0436.

### Phase 4: Advanced Features and Platform Hardening (P2)
Focus: Enhance web audit capabilities and improve Windows platform reliability.
27. **#8**: Support workspace audit_headers configuration for Lighthouse and Arkenar authenticated scans.
28. **#11**: Implement exponential backoff retry logic for Windows file locking conflicts during npm installations.
29. **#12**: Add optional dev server lifecycle management flags (--start-server, --server-port) to playhouse verify.

### Phase 5: Packaging, CI Hardening, and Final Release Sign-Off (P2)
Focus: Secure build scripts, modernize packaging style, and complete cross-platform verification.
30. **#31**: Replace direct string template expansions with environment variables in GitHub Action run steps to prevent script injection.
31. **#32**: Configure npm Trusted Publishing (OIDC) to replace long-lived static NPM_TOKEN secrets.
32. **#33**: Convert string literals to single quotes in Homebrew formula where appropriate for RuboCop compliance.
33. **#34**: Add frozen_string_literal magic comment to Homebrew formula.
34. **#35**: Add descriptive class-level documentation comments to Homebrew formula.

---

## Issue detail (confirmed against codebase)

### 1. Trivy skips dot-directories - [#2](https://github.com/nicholasxdavis/playhouse-cli/issues/2)

**Where:** `src/engines/trivy.rs` runs `trivy fs .` under `scan_root` without overriding Trivy default hidden directory exclusion.

**Impact:** Files under `.well-known/` (such as CSAF, `security.txt`, or accidentally leaked tokens) are not scanned.

**Suggested fix:** Configure Trivy command execution in `src/engines/trivy.rs` to pass `--scanners vuln,secret` and `--hidden` so dot-directories are evaluated. Add a workspace configuration setting `trivy_skip_dirs` defaulting to `node_modules,.git` so build artifacts are excluded while hidden config folders remain protected.

**Acceptance criteria:** A secret placed under `.well-known/` is detected and reported by `playhouse trivy --json`.

---

### 2. Skipped audits inflate score - [#3](https://github.com/nicholasxdavis/playhouse-cli/issues/3)

**Where:** `src/score.rs` compute function ignores skipped engines, and `weighted_stars()` redistributes their weight across remaining categories. By default, `skip_lighthouse_without_server` is true in `src/config.rs`.

**Impact:** When no target URL is provided, Arkenar and Lighthouse are skipped, shifting 40% of total score weight onto static checks. A project can pass with an 88 star average even if its web endpoints would fail DAST and performance audits.

**Suggested fix:** Distinguish between explicit user opt-outs (via `skip_*_in_verify` config flags) and implicit skips caused by missing URLs or offline servers. When web audits are implicitly skipped without an explicit configuration override, assign 0/100 to those categories or mark verification as failed. Update scoring documentation in `stars.md` to reflect these strict rules.

**Acceptance criteria:** Running `playhouse verify --json` without a target URL on a web project fails verification instead of passing with rebalanced weights.

---

### 3. Trivy cache and count mismatch - [#4](https://github.com/nicholasxdavis/playhouse-cli/issues/4)

**Where:** `src/engines/trivy.rs` does not specify `--cache-dir` or clear stale caches. The `count_findings()` helper filters findings by `--severity`, which causes discrepancies between terminal summary text and `score.json` when cache state changes.

**Impact:** Terminal output may display 10 secrets while `score.json` reports only 1.

**Suggested fix:** Isolate Trivy caching inside `.playhouse/cache/trivy` during verification runs. Use a single evaluated finding summary struct to populate stdout, engine metrics JSON, and `score.rs`. Include explicit fields for `rawSecretCount` and `filteredSecretCount` in JSON reports.

**Acceptance criteria:** After introducing test secrets, `playhouse trivy --json` and `.playhouse/reports/score.json` report identical finding counts without requiring manual cache resets.

---

### 4. JSON exitCode mismatch - [#5](https://github.com/nicholasxdavis/playhouse-cli/issues/5)

**Where:** `src/engines/trivy.rs` assigns the raw subprocess return code directly to `"exitCode"` in metrics JSON, which is often `0` even when vulnerabilities or secrets exist. Meanwhile, Playhouse CLI returns exit code `4`.

**Impact:** Automated agents parsing engine JSON reports mistakenly treat security failures as successful runs.

**Suggested fix:** Set the `"exitCode"` property in engine JSON reports to match standard Playhouse return codes (0 for pass, 1 for test failure, 2 for Lighthouse threshold failure, 3 for Arkenar findings, 4 for Trivy findings, 5 for missing tools). Add a separate `"rawToolExitCode"` field to record the underlying CLI return value. Apply this pattern across all engine runners (`trivy.rs`, `playwright.rs`, `arkenar.rs`, `lighthouse.rs`).

**Acceptance criteria:** When `playhouse trivy --json` exits with code 4, every `exitCode` property in the JSON payload equals 4.

---

### 5. Doctor skips node_modules - [#6](https://github.com/nicholasxdavis/playhouse-cli/issues/6)

**Where:** `src/detect.rs` in `run_doctor()` verifies global binaries and `@playwright/test` executables, but ignores local dependency installation state.

**Impact:** Doctor reports the workspace is ready even when `npm install` has never been executed.

**Suggested fix:** When project detection indicates a Node environment, check for the presence of `node_modules` inside `scan_root` or validate lockfile integrity against install markers. Use the package manager detection module (`src/pkgmgr.rs`) to emit a clear warning with the exact installation command required (`npm install`, `pnpm install`, `yarn install`, or `bun install`).

**Acceptance criteria:** Running `playhouse doctor --json` in a Node project with missing dependencies returns a warning status with installation guidance instead of passing.

---

### 6. Test pattern on functional/verify - [#7](https://github.com/nicholasxdavis/playhouse-cli/issues/7)

**Where:** `playhouse playwright <pattern>` works via argument parsing in `src/cli/args.rs` and `src/engines/playwright.rs`, but `playhouse functional` lacks pattern support and `audit.rs` executes the test suite without filters.

**Impact:** Verification always runs the entire test suite, preventing agents from isolating specific failing tests during development.

**Suggested fix:** Add an optional positional `pattern` argument to the `Functional` command definition in `src/cli/args.rs`. Add a `--test <pattern>` option to `playhouse verify` and pass it through `audit::run_audit`. Update functional runners (`cargo.rs`, `pytest.rs`, `npm_test.rs`, `go.rs`) to forward filter strings to their respective CLI test engines.

**Acceptance criteria:** Executing `playhouse functional webhook.unit.test.ts --json` or `playhouse verify --test webhook --json` executes only the matching subset of tests.

---

### 7. No auth for web audits - [#8](https://github.com/nicholasxdavis/playhouse-cli/issues/8)

**Where:** `src/engines/lighthouse.rs` and `src/engines/arkenar.rs` accept only target URLs without authentication headers.

**Impact:** DAST and performance scanners evaluate only unauthenticated landing pages or login screens on protected web applications.

**Suggested fix:** Introduce an `audit_headers` key (key-value map of HTTP headers) in `WorkspaceConfig`. Pass these headers as `--extra-headers` JSON string to Lighthouse and inject them into Arkenar HTTP scan requests. Support environment variable expansion (such as `Bearer ${AUTH_TOKEN}`) to ensure sensitive credentials are never stored as plain text in configuration files.

**Acceptance criteria:** With `audit_headers` configured, Lighthouse and Arkenar successfully evaluate protected routes that otherwise return HTTP 401 Unauthorized.

---

### 8. No failure output in JSON - [#9](https://github.com/nicholasxdavis/playhouse-cli/issues/9)

**Where:** `src/engines/playwright.rs` and other functional test runners record numerical pass/fail statistics but discard standard error streams.

**Impact:** JSON reports indicate test failures without providing assertion error messages or stack traces.

**Suggested fix:** Capture standard error and standard output streams from test execution. When a runner exits with a non-zero status, attach the last 50 lines (capped at 8 KB) to a `failureOutput` string field inside the engine metrics JSON payload.

**Acceptance criteria:** When a functional test fails, `playhouse playwright --json` returns a JSON structure containing the exact assertion error message inside `failureOutput`.

---

### 9. stdout vs score.json skip mismatch - [#10](https://github.com/nicholasxdavis/playhouse-cli/issues/10)

**Where:** `src/audit.rs` in `run_browser_audits()` marks engines as skipped when no target URL is configured. If a URL is specified but the server is offline, engines attempt execution, fail, and get recorded as failures in `score.json`. Terminal progress bars and report files can display conflicting status labels.

**Impact:** Developers see "skipped" in terminal output while `score.json` reports audit failures.

**Suggested fix:** Perform an HTTP reachability probe (`detect::probe_url`) before initiating browser audits. If the endpoint is unreachable and `skip_lighthouse_without_server` is enabled, record the engine status as `skipped` with reason `url-unreachable` consistently across progress notifications, audit JSON, and `score.json`. If the setting is disabled, fail immediately with a clear connection error.

**Acceptance criteria:** Terminal summaries, audit JSON payloads, and `.playhouse/reports/score.json` display identical skip or failure states for offline endpoints.

---

### 10. Windows file locks - [#11](https://github.com/nicholasxdavis/playhouse-cli/issues/11)

**Where:** `src/install.rs` and `src/pkgmgr.rs` do not handle Windows `EPERM` or `EBUSY` file locking errors during package installations triggered by verify.

**Impact:** Verification fails intermittently on Windows when background IDE processes or virus scanners lock files in `node_modules`.

**Suggested fix:** Implement exponential backoff retry logic (up to 3 retries) for file system operations during installation. If `node_modules` is already populated and a locking error occurs during verification check, emit a warning instead of aborting the audit pipeline. Document Windows file locking behaviors in `AGENTS.md`.

**Acceptance criteria:** Verification completes reliably or emits a clean warning when `node_modules` is locked by external processes on Windows.

---

### 11. No server lifecycle - [#12](https://github.com/nicholasxdavis/playhouse-cli/issues/12)

**Where:** `src/cli/handlers.rs` in `run_verify()` checks URL availability but cannot spawn or manage local development servers.

**Impact:** Automated agents must manually background dev servers, frequently leaving orphaned processes running across port 3000.

**Suggested fix:** Add optional lifecycle management flags to `playhouse verify`: `--start-server <cmd>`, `--server-port <port>`, and `--server-timeout <seconds>`. When specified, spawn the development server process in a dedicated process group, poll socket readiness using `detect::probe_ports`, execute all verification engines, and terminate the server process group cleanly upon completion or exit.

**Acceptance criteria:** Executing `playhouse verify --start-server "npm run dev" --server-port 3000 --json` automatically starts the server, completes all audits, and terminates the process afterward.

---

### 12. Config validation gaps - [#13](https://github.com/nicholasxdavis/playhouse-cli/issues/13)

**Where:** `src/config_cli.rs` checks path traversal syntax for `scan_root` and `test_root`, but does not verify that directory paths exist on disk. Furthermore, `default_url` accepts arbitrary unvalidated strings.

**Impact:** Invalid settings such as `default_url abc` are saved without warnings, causing subsequent verification runs to crash.

**Suggested fix:** Validate values in `playhouse config set` before writing to `.playhouse/config.json`. Parse URL strings using the `url` crate to guarantee valid HTTP or HTTPS schemes. Use filesystem metadata checks to confirm that `scan_root` and `test_root` paths point to existing directories.

**Acceptance criteria:** Attempting to set an malformed URL or non-existent directory path via `playhouse config set` is rejected immediately with an explanatory error message.

---

### 13. SKILL.md size - [#14](https://github.com/nicholasxdavis/playhouse-cli/issues/14)

**Where:** `src/assets/playhouse_skill.md` spans over 210 lines and is installed into target workspaces by `playhouse init`.

**Impact:** Consumes excessive context window tokens for automated agents, duplicating documentation already present in `AGENTS.md` and the command manifest.

**Suggested fix:** Trim the default skill template to under 90 lines by focusing strictly on essential CLI commands, JSON formatting rules, verification checklists, and pointing to `playhouse agent --json` as the live source of truth. Ensure `playhouse skill disable` is clearly documented in the header.

**Acceptance criteria:** Running `playhouse init` generates a `.playhouse/SKILL.md` file under 100 lines that accurately guides agent behavior.

---

### 14. Scanner crash scores 100/100 - [#15](https://github.com/nicholasxdavis/playhouse-cli/issues/15)

**Where:** In `src/score.rs`, empty Lighthouse metric scores combined with a zero exit code result in a default 100/100 score. In `src/engines/arkenar.rs`, empty JSON objects `{}` parse as zero vulnerabilities.

**Impact:** Crashed, interrupted, or empty scanner runs masquerade as perfect security and performance scores.

**Suggested fix:** In `src/score.rs` and engine parsers, require valid non-empty report payloads before computing scores. If an engine process terminates unexpectedly, produces zero-byte output, or fails to generate its report file, set `scanComplete: false` in metrics and assign 0 stars for that category.

**Acceptance criteria:** Terminating a scanner process mid-execution results in a 0 star score and verification failure instead of a perfect score.

---

### 15. GitHub Actions unpinned action references - [#16](https://github.com/nicholasxdavis/playhouse-cli/issues/16)

**Where:** Workflow definitions in `.github/workflows/ci.yml` and `.github/workflows/release.yml`.

**Impact:** Exposes the repository to supply chain compromise if third-party action tags are modified or underlying repositories are breached.

**Suggested fix:** Update all third-party GitHub Action references to use exact 40-character commit SHAs instead of version tags. Add inline comments indicating the corresponding human-readable version tag for maintainability.

**Acceptance criteria:** All third-party action invocations in GitHub workflows reference immutable commit SHAs.

---

### 16. GitHub Actions workflow cache poisoning risk - [#17](https://github.com/nicholasxdavis/playhouse-cli/issues/17)

**Where:** Caching configurations across GitHub Action workflows in `.github/workflows/`.

**Impact:** Vulnerability to cache poisoning where malicious pull requests from forks could overwrite shared build caches.

**Suggested fix:** Restrict cache key generation strictly to immutable dependency lockfiles (`Cargo.lock`, `package-lock.json`). Configure workflow permissions so pull requests originating from untrusted forks have read-only cache access and cannot poison primary branch caches.

**Acceptance criteria:** CI workflows enforce strict cache key scoping and prevent untrusted pull requests from updating shared caches.

---

### 17. GitHub Actions excessive permissions - [#18](https://github.com/nicholasxdavis/playhouse-cli/issues/18)

**Where:** Workflow job definitions in `.github/workflows/ci.yml` and `.github/workflows/release.yml`.

**Impact:** Overly broad default token permissions increase the blast radius if a workflow step or dependency is compromised.

**Suggested fix:** Define top-level `permissions: {}` (no permissions) across all workflows. Grant minimum required privilege scopes at the individual job level (for example, `contents: read` for tests, and `id-token: write` only for release publishing jobs).

**Acceptance criteria:** Every workflow and job specifies explicit, principle-of-least-privilege permission blocks.

---

### 18. Credential persistence in workflow artifacts - [#19](https://github.com/nicholasxdavis/playhouse-cli/issues/19)

**Where:** Artifact upload step configurations in `.github/workflows/`.

**Impact:** Risk of sensitive environment variables, configuration dumps, or temporary credentials being archived inside downloaded CI artifacts.

**Suggested fix:** Configure `actions/upload-artifact` steps with strict inclusion paths and explicit exclusion patterns (such as excluding `.env`, `*.key`, and temporary diagnostic logs) to ensure no credential files are packaged.

**Acceptance criteria:** CI build artifacts contain only required compiled binaries and public verification reports without sensitive files.

---

### 19. High cyclomatic complexity in render_score_report - [#20](https://github.com/nicholasxdavis/playhouse-cli/issues/20)

**Where:** `src/tui/ui_blocks.rs`.

**Impact:** High complexity makes TUI rendering logic difficult to read, test, and maintain safely.

**Suggested fix:** Decompose `render_score_report` by extracting category score row rendering, progress bar calculation, and grade formatting into self-contained helper functions, reducing the cyclomatic complexity score below 10.

**Acceptance criteria:** Static analysis tools report acceptable cyclomatic complexity for `render_score_report`.

---

### 20. Deeply nested control flow in ui_blocks or handlers - [#21](https://github.com/nicholasxdavis/playhouse-cli/issues/21)

**Where:** `src/tui/ui_blocks.rs` and `src/cli/handlers.rs`.

**Impact:** Multiple levels of indentation make error handling and execution flow hard to follow.

**Suggested fix:** Refactor nested `if let`, `match`, and conditional blocks using guard clauses, early returns, and helper pattern matching to keep maximum indentation depth at 3 levels or fewer.

**Acceptance criteria:** Code structure throughout UI blocks and CLI handlers maintains an indentation depth of 3 or less.

---

### 21. High file complexity in ui_blocks - [#22](https://github.com/nicholasxdavis/playhouse-cli/issues/22)

**Where:** `src/tui/ui_blocks.rs`.

**Impact:** Monolithic UI files concentrate too many responsibilities, slowing down refactoring and feature additions.

**Suggested fix:** Decompose `ui_blocks.rs` into specialized component modules (for example, `src/tui/components/score.rs`, `src/tui/components/tools.rs`, and `src/tui/components/summary.rs`) to keep file lengths under 400 lines and separate presentation concerns.

**Acceptance criteria:** Total lines of code and cognitive complexity metrics for individual UI component files remain well below linting thresholds.

---

### 22. Similar code duplication in TUI splash and mascot - [#23](https://github.com/nicholasxdavis/playhouse-cli/issues/23)

**Where:** `src/tui/splash.rs` and `src/tui/mascot.rs`.

**Impact:** Duplicate layout wrapper and terminal styling code increases maintenance overhead and risks UI inconsistencies.

**Suggested fix:** Extract shared container formatting, alignment logic, and border styling into a common UI layout helper function shared by both splash and mascot screens.

**Acceptance criteria:** Static duplication analyzers report no structural code duplication between `splash.rs` and `mascot.rs`.

---

### 23. High parameter count in render_tool_call - [#24](https://github.com/nicholasxdavis/playhouse-cli/issues/24)

**Where:** `src/tui/ui_blocks.rs`.

**Impact:** Functions accepting 6 or more individual parameters are error-prone and rigid when adding new UI features.

**Suggested fix:** Introduce a clean configuration struct `ToolCallRenderConfig` encapsulating tool name, execution status, elapsed time, and message payload, passing a single reference to `render_tool_call`.

**Acceptance criteria:** The `render_tool_call` function signature takes a single configuration struct parameter instead of multiple primitive arguments.

---

### 24. Vulnerable lru dependency RUSTSEC-2026-0002 - [#25](https://github.com/nicholasxdavis/playhouse-cli/issues/25)

**Where:** `Cargo.toml` and `Cargo.lock`.

**Impact:** Potential security or denial-of-service risks associated with outdated LRU caching crate versions.

**Suggested fix:** Update the `lru` crate dependency in `Cargo.toml` to version `0.12.5` or higher (or the latest patched version) to resolve advisory RUSTSEC-2026-0002.

**Acceptance criteria:** Running `cargo audit` or `osv-scanner` reports zero vulnerability alerts for crate dependencies.

---

### 25. Complex boolean logic in score or detect - [#26](https://github.com/nicholasxdavis/playhouse-cli/issues/26)

**Where:** `src/score.rs` and `src/detect.rs`.

**Impact:** Dense compound logical expressions increase the likelihood of subtle edge-case evaluation bugs.

**Suggested fix:** Refactor compound conditionals by assigning intermediate evaluations to descriptive boolean variables (such as `is_valid_url`, `has_lockfile`, and `is_engine_skipped`) before combining them in control statements.

**Acceptance criteria:** Boolean evaluation logic in scoring and project detection modules is clear, self-documenting, and easily unit-tested.

---

### 26. Identical code duplication in splash and mascot - [#27](https://github.com/nicholasxdavis/playhouse-cli/issues/27)

**Where:** `src/tui/splash.rs` and `src/tui/mascot.rs`.

**Impact:** Exact copy-pasted ASCII art initialization blocks create redundant code.

**Suggested fix:** Move shared ASCII banner strings and layout setup sequences into a dedicated `src/tui/banner.rs` module imported by both splash and mascot renderers.

**Acceptance criteria:** Copy-paste duplication scanners report zero identical code blocks between splash and mascot modules.

---

### 27. High return statement count in detect_at - [#28](https://github.com/nicholasxdavis/playhouse-cli/issues/28)

**Where:** `src/detect.rs`.

**Impact:** Too many exit points in `detect_at` make tracing project recognition logic difficult for maintainers.

**Suggested fix:** Consolidate detection logic using structured matching or a builder pattern so that workspace profile recognition funnels into a single clean exit point.

**Acceptance criteria:** The `detect_at` function contains a minimal, readable set of return statements.

---

### 28. Actionlint expression secrets context validation error - [#29](https://github.com/nicholasxdavis/playhouse-cli/issues/29)

**Where:** `.github/workflows/release.yml`.

**Impact:** CI linter actionlint fails when evaluating improperly formatted secret context interpolations in workflow scripts.

**Suggested fix:** Pass required secrets into step execution environments via explicit `env:` mappings rather than expanding `${{ secrets.NAME }}` directly inside bash command execution strings.

**Acceptance criteria:** Running actionlint against `.github/workflows/` completes without syntax or context validation errors.

---

### 29. Vulnerable paste dependency RUSTSEC-2024-0436 - [#30](https://github.com/nicholasxdavis/playhouse-cli/issues/30)

**Where:** `Cargo.toml` and `Cargo.lock`.

**Impact:** Security vulnerability in macro expansion crate `paste`.

**Suggested fix:** Update `paste` in `Cargo.toml` to version `1.0.15` or newer, or replace macro usages with standard Rust language features where feasible.

**Acceptance criteria:** Dependency security scans report no active advisories for the `paste` crate.

---

### 30. Template injection vulnerability in GitHub workflows - [#31](https://github.com/nicholasxdavis/playhouse-cli/issues/31)

**Where:** `.github/workflows/ci.yml` and `.github/workflows/release.yml`.

**Impact:** Untrusted user input from pull request titles, branch names, or commit messages could allow arbitrary script injection if evaluated directly in workflow run blocks.

**Suggested fix:** Bind dynamic GitHub event payloads to step environment variables (`env: TITLE: ${{ github.event.pull_request.title }}`) and reference the environment variables inside shell scripts (`"$TITLE"`) instead of direct inline interpolation.

**Acceptance criteria:** All GitHub workflow scripts use environment variables for external event data, preventing script injection.

---

### 31. GitHub Action workflows static token use - [#32](https://github.com/nicholasxdavis/playhouse-cli/issues/32)

**Where:** `.github/workflows/release.yml`.

**Impact:** Long-lived static repository secrets (`NPM_TOKEN`) pose a permanent credential theft risk if exposed.

**Suggested fix:** Configure npm OpenID Connect (OIDC) Trusted Publishing in workflow definitions and package registry settings so releases authenticate using ephemeral, short-lived identity tokens.

**Acceptance criteria:** Release workflows successfully publish npm packages using OIDC Trusted Publishing without requiring static secret tokens.

---

### 32. Rubocop Style/StringLiterals violation - [#33](https://github.com/nicholasxdavis/playhouse-cli/issues/33)

**Where:** `packaging/homebrew/playhouse.rb`.

**Impact:** Inconsistent Ruby string literal formatting violates Homebrew repository contribution standards.

**Suggested fix:** Replace double-quoted strings with single-quoted strings throughout `playhouse.rb` wherever string interpolation or special escape sequences are not required.

**Acceptance criteria:** Running `rubocop packaging/homebrew/playhouse.rb` reports zero string literal formatting offenses.

---

### 33. Rubocop Style/FrozenStringLiteralComment violation - [#34](https://github.com/nicholasxdavis/playhouse-cli/issues/34)

**Where:** `packaging/homebrew/playhouse.rb`.

**Impact:** Lacks Ruby string immutability optimization and triggers style warnings during Homebrew formula linting.

**Suggested fix:** Add `# frozen_string_literal: true` as the very first line of the Homebrew formula file.

**Acceptance criteria:** RuboCop inspection confirms the frozen string literal magic comment is present and valid.

---

### 34. Rubocop Style/Documentation violation - [#35](https://github.com/nicholasxdavis/playhouse-cli/issues/35)

**Where:** `packaging/homebrew/playhouse.rb`.

**Impact:** Missing top-level class documentation triggers RuboCop style warnings and reduces code clarity.

**Suggested fix:** Add a concise, professional documentation comment directly above `class Playhouse < Formula` describing the CLI's functionality as a headless QA security and performance audit tool.

**Acceptance criteria:** RuboCop reports clean documentation compliance for the Homebrew formula class.

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
|---|---|
| `AGENTS.md` | Agent operating manual |
| `stars.md` | Star methodology |
| `.dev/runbook.md` | Release process |
| `playhouse agent --json` | Live command manifest |
