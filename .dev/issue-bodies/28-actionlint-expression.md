## Problem

In GitHub Actions workflows, the secrets context is used in an invalid location where it is not allowed (e.g. in a job-level conditional if expression).

## Impact

The workflow parser will reject the file or behave unexpectedly because secrets context is not available there.

## Suggested fix

1. Locate where the secrets context is being used directly in conditional expressions in release.yml or ci.yml.
2. Store the secret in an environment variable at the step/job level, or refactor the conditional to check a non-secret variable or output.

**Files:** .github/workflows/release.yml, .github/workflows/ci.yml

## Acceptance criteria

- [ ] actionlint runs with zero errors on the workflow files

## Priority

P1 - should ship (CI configuration)
