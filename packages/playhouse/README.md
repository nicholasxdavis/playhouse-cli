# playhouse (npm)

Thin **installer + launcher** for the [Playhouse](https://github.com/nicholasxdavis/playhouse-cli) Rust CLI. No JavaScript rewrite — `postinstall` downloads the native binary for your OS/arch.

## Install

```bash
npm install -g playhouse
# or project devDependency
pnpm add -D playhouse
npx playhouse doctor
```

Requires **Node.js 18+**. Rust is **not** required for end users.

## Environment

| Variable | Purpose |
|----------|---------|
| `PLAYHOUSE_BIN` | Use a specific binary (skip bundled `vendor/`) |
| `PLAYHOUSE_SKIP_DOWNLOAD` | Skip `postinstall` download (`1` or `true`) |
| `PLAYHOUSE_VERSION` | Pin release version to download (default: package version) |
| `PLAYHOUSE_GITHUB_REPO` | Override GitHub repo (`owner/repo`) |

## Local development

From repo root:

```bash
cargo build --release
cd packages/playhouse
npm run link-local
node bin/playhouse.js --version
node bin/playhouse.js doctor --json
```

### Optional project postinstall

Add to your app `package.json` after `playhouse` is a devDependency:

```json
"scripts": {
  "postinstall": "playhouse-install-tools"
}
```

Runs `playhouse install --full --json`. Use `--minimal` via `node node_modules/playhouse/scripts/project-postinstall.js --minimal`.

## Publish (maintainers)

1. Tag `v0.1.0` and push — GitHub Release workflow uploads platform binaries.
2. Sync `package.json` version with `Cargo.toml`.
3. `cd packages/playhouse && npm publish --access public`

If the npm name `playhouse` is taken, publish as `@playhouse/cli` and update the `bin` field.
