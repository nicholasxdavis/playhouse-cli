## Problem

Workflows use static long-lived tokens for authentication when publishing releases or packages instead of Trusted Publishing / OIDC.

## Impact

Static authentication credentials stored as secrets are vulnerable to compromise and leakage.

## Suggested fix

1. Configure GitHub OIDC integration for the npm and GitHub release steps.
2. Replace static secret tokens with Trusted Publishing permissions in the workflow file.

**Files:** .github/workflows/release.yml

## Acceptance criteria

- [ ] Release workflow uses Trusted Publishing/OIDC for authentication instead of long-lived secrets where supported

## Priority

P2 - follow-up (auth best practices)
