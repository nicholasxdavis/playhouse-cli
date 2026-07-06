# Third-party notices

Playhouse bundles or invokes the following third-party tools. Versions match `src/tools.rs` and `packages/playhouse` at release time.

| Component | License | Source | Bundled version |
|-----------|---------|--------|-----------------|
| **Trivy** | Apache-2.0 | [aquasecurity/trivy](https://github.com/aquasecurity/trivy) | 0.72.0 |
| **Arkenar** | MIT | [realozk/ARKENAR](https://github.com/realozk/ARKENAR) | 1.3.2 |
| **Playwright** | Apache-2.0 | [microsoft/playwright](https://github.com/microsoft/playwright) | ^1.49.0 (npm) |
| **Lighthouse** | Apache-2.0 | [GoogleChrome/lighthouse](https://github.com/GoogleChrome/lighthouse) | ^12.4.0 (npm) |

## Rust dependencies

Playhouse is built with Rust crates listed in `Cargo.lock`. Run `cargo license` or inspect `Cargo.lock` for the full dependency graph and licenses.

## npm wrapper

The `packages/playhouse` npm package downloads the native Playhouse binary from [GitHub Releases](https://github.com/nicholasxdavis/playhouse-cli/releases) at install time. Postinstall may also pull Playwright, Lighthouse, and Chromium into `.playhouse/npm` when you run `playhouse install --full`.

## Attribution

- Trivy — Aqua Security
- Arkenar — MIT Rust DAST scanner (OWASP ZAP alternative)
- Playwright — Microsoft
- Lighthouse — Google Chrome team

Playhouse itself is MIT licensed — see [LICENSE](LICENSE).
