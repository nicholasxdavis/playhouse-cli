## Problem

GitHub Actions workflows define overly broad or excessive top-level or job-level permissions rather than the minimum necessary.

## Impact

Increases the attack surface and blast radius in the event a workflow token is compromised.

## Suggested fix

1. Define explicit, read-only permissions for each job.
2. Grant write access only where strictly required (e.g. for release notes or package publication).

**Files:** .github/workflows/ci.yml, .github/workflows/release.yml

## Acceptance criteria

- [ ] Workflows have explicit minimum permission scopes defined for each job

## Priority

P1 - should ship (least privilege)
