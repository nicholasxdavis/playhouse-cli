## Problem

GitHub Actions workflows write/read runtime cache artifacts in a way that is potentially vulnerable to cache poisoning.

## Impact

An attacker could potentially poison caches with malicious dependencies or built binaries across branches or custom builds, leading to execution of untrusted code in CI/CD.

## Suggested fix

1. Audit Swatinem/rust-cache and node cache usages in workflows.
2. Restrict write/read settings or isolate cache keys by branch/PR context to prevent poison across scopes.

**Files:** .github/workflows/ci.yml, .github/workflows/release.yml

## Acceptance criteria

- [ ] Cache configurations are audited and updated to prevent cross-branch cache poisoning

## Priority

P0 - release blocker (cache poisoning risk)
