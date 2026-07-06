# Playhouse Star Rating (0–100)

Playhouse Stars are a **Lighthouse-inspired combined audit score** for your project. After `playhouse verify` or `playhouse score`, you get a single **0–100 rating** with category breakdowns and a plain-language **why** list.

## Quick start

```bash
playhouse verify --json          # full suite + stars
playhouse score --json           # same audit, star-focused output
playhouse score --last --json    # read .playhouse/reports/score.json
```

## Scale

| Stars | Grade | Meaning |
|------:|-------|---------|
| 90–100 | Production Ready ★★★★★ | Ship with confidence |
| 75–89 | Good ★★★★☆ | Solid; minor gaps |
| 60–74 | Fair ★★★☆☆ | Address weak categories |
| 40–59 | Needs Work ★★☆☆☆ | Significant QA debt |
| 0–39 | Critical ★☆☆☆☆ | Do not ship |

Default **pass threshold**: **75** (`star_pass_threshold` in settings). Verify fails if stars are below threshold **or** any engine exits non-zero.

## Categories (weighted)

Each engine normalizes to 0–100, then weights combine (skipped engines rebalance):

| Category | Weight | Engine |
|----------|-------:|--------|
| Toolchain | 10% | `playhouse doctor` |
| Security (static) | 25% | Trivy — vulns + secrets |
| Functional | 25% | Playwright — pass rate |
| Security (DAST) | 20% | Arkenar — high/medium/low findings |
| Performance & UX | 20% | Lighthouse — avg of perf, a11y, BP, SEO |

## Why this design?

- **Lighthouse-style**: category scores → weighted overall score, not just pass/fail.
- **Agent-friendly**: one number + JSON at `.playhouse/reports/score.json`.
- **Transparent**: `why[]` explains strengths, weaknesses, and what to fix.

## Package managers

Playwright and Lighthouse tooling install/run via **npm, pnpm, yarn, or bun**:

- Setting: `package_manager` = `auto` | `npm` | `pnpm` | `yarn` | `bun`
- `auto` picks from lockfiles: `bun.lockb` → bun, `pnpm-lock.yaml` → pnpm, `yarn.lock` → yarn, `package-lock.json` → npm

## Report location

```
.playhouse/reports/score.json
```

Contains `playhouseScore`, per-category stars, `why`, engine metrics, and timestamp.
