# Playhouse release scripts (TypeScript)

Node tooling for version sync, Homebrew formula updates, and shared release manifests. Run from the repo root.

## Setup

```bash
npm ci --prefix scripts
```

## Commands

| Script | Purpose |
|--------|---------|
| `npm run check-version-sync --prefix scripts` | Assert `Cargo.toml`, npm, Homebrew, and release-targets copies match |
| `npm run validate-release-matrix --prefix scripts` | Assert release.yml build matrix matches manifest |
| `npm run update-homebrew-formula --prefix scripts [artifactsDir]` | Regenerate `packaging/homebrew/playhouse.rb` from CI artifacts |
| `npm run typecheck --prefix scripts` | `tsc --noEmit` on all scripts |

## Release manifest

Platform triples and asset names live in one place:

```
scripts/manifest/release-targets.json   ← edit here only
```

Consumers:

- `scripts/update-homebrew-formula.ts` — Homebrew URLs and sha256 targets
- `packages/playhouse/scripts/lib/release-targets.json` — npm copy (must stay in sync; validated by `check-version-sync`)
- `packages/playhouse/scripts/lib/platform.js` — Node platform → triple mapping at install time

When adding or changing a release target, update `scripts/manifest/release-targets.json` and copy the same file to `packages/playhouse/scripts/lib/release-targets.json`, then run `check-version-sync`.

## CI

The `version-sync` job in `.github/workflows/ci.yml` runs `check-version-sync`, `validate-release-matrix`, and `typecheck` on every push/PR.

The release workflow uses `update-homebrew-formula.ts` after building platform artifacts.

## Agents

Use `--json` on Playhouse itself for QA; these scripts are **release/packaging only**. Do not invent commands beyond what is listed here and in `AGENTS.md`.
