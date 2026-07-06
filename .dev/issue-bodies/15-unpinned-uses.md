## Problem

GitHub Actions workflows use unpinned/tag-based action references instead of full commit SHAs.

## Impact

Vulnerable to supply chain attacks if the third-party action tags are moved or the action repository is compromised.

## Suggested fix

1. Update all third-party GitHub Action references in the workflow files to use the full 40-character commit SHA.
2. Add inline comments indicating the tag version for readability.

**Files:** .github/workflows/ci.yml, .github/workflows/release.yml

## Acceptance criteria

- [ ] All third-party actions in .github/workflows/ pin to a specific 40-character commit SHA

## Priority

P0 - release blocker (security gap)
