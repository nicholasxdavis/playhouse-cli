## Problem

GitHub Actions uploads build artifacts that could potentially persist sensitive credentials or secrets.

## Impact

Stored artifacts containing credentials or sensitive environment configuration can be downloaded by unauthorized users, leading to credential leaks.

## Suggested fix

1. Audit actions/upload-artifact actions in release and CI workflows.
2. Ensure no sensitive env files, package configurations, or raw config files are packaged inside target artifact directories.

**Files:** .github/workflows/ci.yml, .github/workflows/release.yml

## Acceptance criteria

- [ ] Uploaded workflow artifacts are verified to contain no credentials or sensitive environment configuration files

## Priority

P1 - should ship (artifact security)
