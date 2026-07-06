# @nicholasxdavis/playhouse-cli

npm installer and launcher for the [Playhouse](https://github.com/nicholasxdavis/playhouse-cli) Rust CLI. `postinstall` downloads the native binary from [GitHub Releases](https://github.com/nicholasxdavis/playhouse-cli/releases) for your OS and CPU.

**Current version:** 0.1.0 (release tag `v0.1.0`)

## Install

```bash
npm install -g @nicholasxdavis/playhouse-cli
playhouse install --full
playhouse init
```

Project dev dependency:

```bash
pnpm add -D @nicholasxdavis/playhouse-cli
npx playhouse doctor
```

Requires **Node.js 18+**. Rust is not required for end users.

## Environment

| Variable | Purpose |
|----------|---------|
| `PLAYHOUSE_BIN` | Use a specific binary (skip bundled `vendor/`) |
| `PLAYHOUSE_SKIP_DOWNLOAD` | Skip `postinstall` download (`1` or `true`) |
| `PLAYHOUSE_VERSION` | Pin release version (default: package version) |
| `PLAYHOUSE_GITHUB_REPO` | Override GitHub repo (`owner/repo`) |

## Local development

From repo root:

```bash
cargo build --release
cd packages/playhouse
npm run link-local
node bin/playhouse.js --version
```

## Publish (maintainers)

1. Sync version in `Cargo.toml` and `package.json` (`node scripts/check-version-sync.js`).
2. Tag and push: `git tag v0.1.0 && git push origin v0.1.0` (GitHub Actions uploads binaries).
3. `cd packages/playhouse && npm publish --access public`

The `playhouse` command name is unchanged after install (see `bin` in `package.json`).
