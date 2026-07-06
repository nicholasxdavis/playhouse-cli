# Third-Party Notices

Playhouse CLI (the Rust orchestrator, TUI, and scripts in this repository) is
licensed under the [MIT License](LICENSE).

Playhouse does **not** include the engines below in this source tree. They are
downloaded or installed at runtime (for example via `playhouse install`) or
invoked from the user's machine via npm/npx. Each engine remains under its own
license. This file summarizes those engines and your attribution obligations when
you distribute or use Playhouse.

## Bundled engines (installed by Playhouse)

### Arkenar

- **Purpose:** Dynamic application security testing (DAST)
- **License:** MIT
- **Project:** https://github.com/realozk/ARKENAR
- **How Playhouse uses it:** Downloads release binaries to `~/.config/playhouse/bin/`
  (or the platform equivalent) and runs them as a subprocess.

### Trivy

- **Purpose:** Static security, dependency CVEs, secrets, IaC, and SBOM scanning
- **License:** Apache License 2.0
- **Project:** https://github.com/aquasecurity/trivy
- **How Playhouse uses it:** Downloads release binaries to `~/.config/playhouse/bin/`
  and runs them as a subprocess.

### Playwright

- **Purpose:** Browser automation and functional verification
- **License:** Apache License 2.0
- **Project:** https://github.com/microsoft/playwright
- **How Playhouse uses it:** Installs `@playwright/test` into `.playhouse/npm/` via
  the user's package manager and runs Playwright CLI commands. Browser binaries
  (Chromium, etc.) are downloaded by Playwright under **their** licenses; see
  Playwright's `ThirdPartyNotices.txt` in the installed package.

### Lighthouse

- **Purpose:** Performance, accessibility, SEO, and best-practices auditing
- **License:** Apache License 2.0
- **Project:** https://github.com/GoogleChrome/lighthouse
- **How Playhouse uses it:** Invokes the `lighthouse` CLI via global install,
  package-manager exec, or `npx` (not vendored in this repository).

## Rust dependencies

Playhouse's Rust crates (clap, tokio, ratatui, serde, etc.) are resolved at
build time via Cargo. Most use MIT or Apache-2.0 (often dual-licensed). Run
`cargo license` (with the [cargo-license](https://crates.io/crates/cargo-license)
tool) for a full dependency license report before release.

## What you need to do

1. **Ship this file** alongside Playhouse when you distribute binaries or
   packages.
2. **Keep the MIT `LICENSE`** for Playhouse's own code.
3. **Do not relicense** Arkenar, Trivy, Playwright, or Lighthouse; include
   their copyright and license notices when you redistribute their binaries or
   npm packages (Playwright and Lighthouse publish license files in their
   distributions).
4. **Arkenar DAST** is for authorized testing only. Only scan systems you own or
   have permission to test.

## Trademark note

"Trivy", "Playwright", "Lighthouse", and "Arkenar" are names of their respective
projects. This project is not affiliated with Aqua Security, Microsoft, Google,
or the Arkenar authors unless stated otherwise.
