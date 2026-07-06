## Problem

GitHub Actions workflows expand untrusted inputs directly inside template expressions, which can lead to template/code injection.

## Impact

An attacker with access to input values (e.g. pull request titles, issue comments, or ref names) can run arbitrary shell commands inside the runner container.

## Suggested fix

1. Replace direct string template expansion (e.g. ${{ github.event.head_commit.message }}) in run steps with environment variables.
2. Refer to the environment variable inside the script instead of using GitHub expression syntax.

**Files:** .github/workflows/ci.yml, .github/workflows/release.yml

## Acceptance criteria

- [ ] Workflows do not expand untrusted input fields in run scripts directly via template expansion

## Priority

P2 - follow-up (template security)
