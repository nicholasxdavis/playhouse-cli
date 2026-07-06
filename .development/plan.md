# Playhouse Development Plan

**Principles:** 100% local. No backend. No cloud dependency. No accounts. No telemetry.  
Runs on any PC (Windows, macOS, Linux) on modest hardware. Lightweight by default.  
**Installable everywhere:** `npm i -g playhouse`, `pnpm add -D playhouse`, `cargo install`, or direct binary.

---

## North star

Playhouse is a **local QA CLI** that agents and humans run from the terminal. Everything happens on disk:

| What | Where |
|------|--------|
| Playhouse binary | npm global / `node_modules/.bin` / `~/.playhouse/bin` / PATH |
| Bundled tools | Trivy, Arkenar, Playwright, Lighthouse in `.playhouse/` or `~/.config/playhouse/` |
| Reports | `.playhouse/reports/` |
| Config | `~/.config/playhouse/settings.json` + `.playhouse/config.json` |
| Agent docs | `.playhouse/SKILL.md`, `AGENTS.md` |
| Tests (optional) | Project tree or `.playhouse/tests/` from baseplates |

No Playhouse server. No API keys for Playhouse. Offline after `playhouse install`.

---

## Distribution model (proof-checked)

Playhouse is a **Rust binary** shipped to users through **npm (primary)** and other package paths. The npm package is a thin **installer + launcher**, not a rewrite in JavaScript.

### Why this works

| Approach | Verdict |
|----------|---------|
| Rewrite CLI in Node | Rejected — slow, heavy, duplicates Rust tooling |
| npm package downloads prebuilt binary | **Chosen** — same pattern as `esbuild`, `turbo`, `@anthropic-ai/claude-code`, `trivy` wrappers |
| `npx playhouse` without global install | Supported via `npx playhouse@latest` |
| `cargo install playhouse` | Supported for Rust developers |
| GitHub Releases zip | Fallback for air-gapped manual install |

### npm package layout (target)

```
packages/playhouse/          # published to npm as "playhouse"
  package.json
  bin/
    playhouse.js             # Node launcher → spawns native binary
  scripts/
    postinstall.js           # download binary for platform/arch from GitHub Releases
  optionalDependencies       # platform-specific packages (alternative pattern)
```

**`package.json` (sketch):**

```json
{
  "name": "playhouse",
  "version": "0.1.0",
  "description": "Local QA CLI — security, functional, performance, agent handoff",
  "bin": { "playhouse": "bin/playhouse.js" },
  "scripts": { "postinstall": "node scripts/postinstall.js" },
  "engines": { "node": ">=18" },
  "files": ["bin", "scripts", "README.md", "LICENSE"]
}
```

**Install flows (all local after postinstall):**

```bash
npm install -g playhouse          # global CLI
pnpm add -D playhouse             # devDependency, npx playhouse
yarn add -D playhouse
bun add -D playhouse
npx playhouse init
cargo install playhouse --git https://github.com/nicholasxdavis/playhouse-cli  # dev path
```

### Proof-check: npm install risks

| Risk | Mitigation |
|------|------------|
| postinstall blocked by corporate proxy | Document manual binary + `PLAYHOUSE_BIN` env override |
| Wrong arch binary | Detect `process.platform` + `process.arch`; fail with clear message |
| npm-only user has no Rust | Never require Rust for end users |
| Node not installed | Document `cargo install` or direct zip; doctor warns |
| Duplicate with cargo install | Launcher checks `PLAYHOUSE_BIN` then bundled path then PATH |
| Package size on npm | Ship **no** binary in tarball — download on postinstall (~12 MB) |

**Exit criteria (distribution):** Fresh Windows/Mac/Linux VM with only Node 18+ → `npm i -g playhouse && playhouse doctor` passes.

---

## Architecture constraints

| Rule | Why |
|------|-----|
| Single Rust binary (source of truth) | Fast, small, cross-platform |
| npm = distribution wrapper | Familiar install for JS ecosystem + agents |
| Tools as subprocesses or bundled binaries | No embedded runtimes beyond each tool |
| Workspace data in `.playhouse/` | Portable, gitignorable, agent-readable |
| Headless-first (`--json`) | CI and agents; TUI optional |
| Lazy install | `playhouse install` pulls Trivy/Arkenar/Playwright/Lighthouse |
| Graceful skip | Missing URL or runner → skip with JSON reason, never hang |

### Lightweight targets

| Component | Target |
|-----------|--------|
| `playhouse` binary | < 15 MB release |
| npm package (sans binary) | < 50 KB |
| Cold `playhouse doctor` | < 2s when cached |
| Disk minimal profile | Trivy + Arkenar ~100 MB |
| Disk full web profile | + Playwright chromium ~150 MB |
| RAM | No daemon; subprocesses exit per engine |
| Network | `npm i` + `playhouse install` only; verify offline |

### Hard no list

- No Playhouse cloud / backend / telemetry server
- No Docker required for default path
- No mandatory global Node tools (bundle into `.playhouse/`)
- No SaaS account to run verify

---

## Current state (v0.1)

### Works

- Rust CLI: TUI + headless + agent manifest
- Bundled Trivy + Arkenar (platform downloads)
- Playwright → `.playhouse/npm/`
- Verify + live progress + Playhouse Stars
- Agent handoff, skills, `AGENTS.md`
- npm/pnpm/yarn/bun for Playwright/Lighthouse tooling

### Gaps

| Gap | Blocks production? |
|-----|-------------------|
| **Not on npm yet** | **Yes** |
| Functional = Playwright only | Yes (multi-stack) |
| No stack detection | Yes |
| Lighthouse not bundled | Medium |
| False-pass edge cases | **Yes (trust)** |
| No test baseplates / scaffold CLI | Medium (optional feature) |
| Docs oversell unbuilt features | Medium |
| TUI verify URL ignores config | Medium |

### Honest profile today

**JS/TS web + Playwright + dev server** = primary. Others get Trivy + partial verify.

---

## Phased plan (revised)

### Phase 0 — Trust (verify never lies)

**Goal:** Production trust. No false passes. Honest docs.

| # | Task | Proof |
|---|------|-------|
| 0.1 | Trivy: fail on bad exit / unparseable JSON | Unit test with corrupt output |
| 0.2 | 0 Playwright tests → skip or fail, not 100 stars | Score test fixture |
| 0.3 | Arkenar: fail if report missing | Missing file test |
| 0.4 | Fix Lighthouse PM invocation | Manual + CI |
| 0.5 | TUI verify uses `resolve_verify_url()` | Config URL test |
| 0.6 | Align `skip_lighthouse_without_server` with audit | Config test |
| 0.7 | All engines write `.playhouse/reports/*.json` | File exists after each engine |
| 0.8 | Docs: "Implemented" vs "Planned" headers on engine md files | Review |
| 0.9 | TCP port probe timeout | No hang test |

**Exit:** Broken repo never gets exit 0 + high stars.

---

### Phase 1 — npm package + release pipeline

**Goal:** `npm i -g playhouse` works on Win/Mac/Linux. No Rust required for users.

| # | Task |
|---|------|
| 1.1 | Create `packages/playhouse/` npm wrapper |
| 1.2 | `postinstall.js` — download binary from GitHub Releases (version pinned) |
| 1.3 | `bin/playhouse.js` — spawn native binary, forward argv, exit code |
| 1.4 | CI: build matrix (win/mac/linux x64 + linux arm64) → attach to Release |
| 1.5 | `PLAYHOUSE_BIN` / `PLAYHOUSE_SKIP_DOWNLOAD` env for enterprise |
| 1.6 | Publish to npm (`playhouse` or `@playhouse/cli` if name taken) |
| 1.7 | README install section: npm first, cargo second |
| 1.8 | `playhouse doctor` reports install method (npm-bundled / cargo / path) |

**Optional platform packages (if postinstall flaky):**

```
@playhouse/cli-win32-x64
@playhouse/cli-darwin-arm64
...
```

**Exit:** Node-only machine → `npm i -g playhouse && playhouse --version && playhouse doctor`.

---

### Phase 2 — Local tool bundling (offline verify)

**Goal:** After `playhouse install`, verify needs no network.

| # | Task |
|---|------|
| 2.1 | Bundle Lighthouse in `.playhouse/npm/` (remove runtime npx) |
| 2.2 | Prefer project `node_modules/.bin/playwright` when present |
| 2.3 | Linux ARM64 + Windows ARM64 Trivy bundles |
| 2.4 | `playhouse install --minimal` (Trivy + Arkenar) |
| 2.5 | `playhouse install --full` (default web: + Playwright + Lighthouse + chromium) |
| 2.6 | Install failures block verify (no silent continue) |
| 2.7 | `playhouse install` callable from npm `postinstall` optional hook in user projects |

**Exit:** Airplane mode verify after install on 3 OSes.

---

### Phase 3 — Stack detection

**Goal:** Playhouse adapts per repo. Doctor only checks what matters.

New: `src/project.rs`

| Signals | Stack | Functional runner |
|---------|-------|-------------------|
| `playwright.config.*` | Web E2E | playwright |
| `package.json` + vitest/jest | Web unit | npm-test |
| `pyproject.toml` / `pytest.ini` | Python | pytest |
| `Cargo.toml` | Rust | cargo test |
| `go.mod` | Go | go test |
| `pom.xml` / `gradle` | Java | mvn/gradle test |

Expose in `playhouse agent --json` → `workspace.stack`, `functionalRunner`, `browserAudits`.

**Exit:** Rust repo doctor does not warn about missing pnpm.

---

### Phase 4 — Multi-language functional runners

**Goal:** Playhouse Stars "Functional" works beyond Playwright.

`src/engines/functional/` — unified metrics (passed, failed, skipped).

| Runner | Command |
|--------|---------|
| playwright | `playwright test --reporter=json` |
| pytest | `pytest` + junit/json report |
| cargo | `cargo test --message-format=json` |
| go | `go test -json ./...` |
| npm-test | `npm test` |

`playhouse functional --json` — run detected runner only.

**Exit:** `playhouse verify` on Rust lib, no Node installed.

---

### Phase 5 — Test baseplates (optional, agent-friendly)

**Goal:** Agents can scaffold tests from templates or write their own. All local files. No backend.

**Optional** — projects may bring existing tests; baseplates help greenfield.

#### Commands (target)

```bash
playhouse test list --json              # available baseplates
playhouse test init --json              # detect stack, scaffold default suite
playhouse test init --plate web-smoke --json
playhouse test add --plate api-health --json
playhouse test run --json               # run .playhouse/tests or project tests
```

#### Baseplate library (bundled in binary or `src/assets/baseplates/`)

| Plate ID | Stack | Scaffolds |
|----------|-------|-----------|
| `web-smoke` | Playwright | Homepage loads, title present |
| `web-auth` | Playwright | Login flow placeholder |
| `web-a11y` | Playwright | axe-style checks hook |
| `api-health` | Playwright/curl | GET /health 200 |
| `rust-lib` | cargo test | `#[test] fn it_works` |
| `python-pytest` | pytest | `test_import_app` |
| `go-http` | go test | handler test stub |

#### Layout options

```
# Option A: project-native (preferred when stack known)
tests/e2e/smoke.spec.ts          # Playwright
tests/test_app.py                # pytest

# Option B: playhouse-owned (greenfield / agents)
.playhouse/tests/
  smoke.spec.ts
  manifest.json                  # plate metadata, agent notes
```

#### Agent workflow

1. `playhouse agent --json` → `testBaseplates`, `testsDetected`, `testsPath`
2. Agent writes custom tests **or** `playhouse test init --plate web-smoke`
3. Agent edits generated files (placeholders marked `# PLAYHOUSE: customize`)
4. `playhouse playwright` / `playhouse functional` / `playhouse verify`

**Rules:**

- Baseplates are **templates**, not generated servers
- Never overwrite existing tests without `--force`
- `manifest.json` lists plates applied (agent audit trail)
- Custom agent-written tests are first-class — no plate required

| # | Task |
|---|------|
| 5.1 | `src/baseplates/` templates + manifest schema |
| 5.2 | `playhouse test list|init|add|run` subcommands |
| 5.3 | Wire into agent manifest + SKILL.md |
| 5.4 | Detect existing tests → skip init unless asked |
| 5.5 | Baseplate vars: `{{url}}`, `{{project_name}}` from workspace config |

**Exit:** Empty Next.js repo → `playhouse test init --plate web-smoke` → `playhouse verify` runs 1 test.

---

### Phase 6 — URL and browser audits

| # | Task |
|---|------|
| 6.1 | Port hints from `package.json` scripts, vite/next/wrangler configs |
| 6.2 | `skip_lighthouse_in_verify` separate from Arkenar |
| 6.3 | Non-web stacks: browser audits N/A with explicit JSON skip |
| 6.4 | `playhouse config set default_url` documented as primary |

---

### Phase 7 — Monorepo + config

`.playhouse/config.json`: `scan_root`, `test_root`, `default_url`, `functional_runner`.

---

### Phase 8 — Production complete

| # | Task |
|---|------|
| 8.1 | Integration tests (temp dirs, fixtures) |
| 8.2 | CI: npm install smoke + verify on 3 OS |
| 8.3 | `THIRD_PARTY_NOTICES.md` current |
| 8.4 | Version sync: npm package version = Rust binary version |
| 8.5 | `playhouse upgrade` — check GitHub Releases / npm latest (local pull only) |
| 8.6 | Homebrew formula (optional, community or official) |

---

## Tooling matrix (production target)

Every engine must: run locally, write report JSON, appear in verify progress, contribute to stars, have doctor check.

| Tool | Role | Install | Offline | Status |
|------|------|---------|---------|--------|
| **playhouse** | Orchestrator | npm / cargo | Yes | Implemented |
| **Trivy** | Static security + secrets | bundled | Yes | Implemented |
| **Arkenar** | DAST | bundled | Yes | Implemented |
| **Playwright** | Functional E2E | `.playhouse/npm` | Yes* | Implemented |
| **Lighthouse** | Perf/a11y/SEO | bundle (Phase 2) | Yes | Partial (npx) |
| **pytest** | Python tests | system pip/uv | Yes | Planned |
| **cargo test** | Rust tests | system rust | Yes | Planned |
| **go test** | Go tests | system go | Yes | Planned |

\* After `playwright install` / `playhouse install --full`

---

## Playhouse Stars (unchanged weights, stricter rules)

| Category | Weight | Source |
|----------|-------:|--------|
| Toolchain | 10% | doctor |
| Security static | 25% | Trivy |
| Functional | 25% | playwright / pytest / cargo / go / npm-test |
| Security DAST | 20% | Arkenar |
| Performance & UX | 20% | Lighthouse |

- Skipped N/A ≠ skipped failed
- 0 tests run ≠ pass
- `passed` in JSON ↔ exit 0

---

## Directory layout (target)

```
playhouse-cli/
  packages/
    playhouse/                 # npm distribution (Phase 1)
      package.json
      bin/playhouse.js
      scripts/postinstall.js
  src/
    main.rs
    agent.rs
    audit.rs
    project.rs                 # Phase 3
    baseplates/                # Phase 5 (optional)
      mod.rs
      assets/
        web-smoke/
        rust-lib/
    engines/
      functional/              # Phase 4
    assets/
      playhouse_skill.md
  .development/
    plan.md
  AGENTS.md
  README.md
```

---

## Agent workflow (full)

```
1. npm i -g playhouse          # or devDependency
2. playhouse init --json
3. playhouse install
4. playhouse agent --json      # stack, plates, nextActions
5. [optional] playhouse test init --plate web-smoke
   OR agent writes tests in tests/
6. playhouse verify --json
7. playhouse agent handoff --json
```

All local. Agent reads `AGENTS.md`, `.playhouse/SKILL.md`, manifest JSON.

---

## Milestones (revised)

| Milestone | Phases | Done when |
|-----------|--------|-----------|
| **M0: Installable** | 1 | `npm i -g playhouse` works 3 OS |
| **M1: Trustworthy** | 0 | No false passes |
| **M2: Offline** | 2 | Verify air-gapped |
| **M3: Multi-stack** | 3 + 4 | Rust/Python/Go without Playwright |
| **M4: Agent tests** | 5 | Baseplates + custom tests coexist |
| **M5: Production** | 6 + 7 + 8 | CI green, docs honest, all tools bundled |

---

## Proof-check summary

| Claim | Valid? | Notes |
|-------|--------|-------|
| npm install without Rust | Yes | Binary wrapper pattern proven industry-wide |
| 100% local verify | Yes | After install; no backend |
| Lightweight on weak hardware | Yes | No daemon; minimal profile ~100 MB |
| Multi-language functional | Yes | Subprocess runners, Phase 4 |
| Agent custom tests | Yes | Standard test files in repo |
| Agent baseplates optional | Yes | Template files only, Phase 5 |
| All tools run flawlessly | **Requires Phase 0 + 2** | False passes and npx must be fixed first |
| Works on any PC | Mostly | ARM gaps closing in Phase 2; doctor explains unsupported |

---

## Immediate next steps (priority order)

1. **Phase 0.1–0.3** — false-pass fixes (trust)
2. **Phase 1.1–1.4** — npm package + release CI (installable)
3. **Phase 2.1** — bundle Lighthouse (offline)
4. **Phase 0.5** — TUI URL fix
5. **Phase 3.1** — `project.rs` skeleton
6. **Phase 5.1** — baseplate asset layout (can parallel after Phase 1)
7. **Phase 0.8** — honest doc headers

---

## Out of scope

- Playhouse cloud / hosted verify
- Remote test generation API
- Replacing Trivy/Playwright/Arkenar/Lighthouse
- Mobile native runners (XCTest/Espresso) — future
- AI writes tests without user/agent review (agents use CLI; human/agent owns files)

---

## Success metrics

| Metric | Target |
|--------|--------|
| `npm i -g playhouse` success rate | 99%+ on Node 18+ |
| `playhouse verify` false pass | 0% |
| Offline verify (post-install) | Win/Mac/Linux |
| Install to first verify | < 5 min |
| Stacks with functional runner | 5+ |
| Baseplates available | 6+ optional templates |
| Binary size | < 15 MB |

---

*Last updated: 2026-07-05. Revise as phases ship.*
