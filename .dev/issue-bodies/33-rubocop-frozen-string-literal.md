## Problem

Missing the frozen string literal magic comment at the top of the Homebrew formula file.

## Impact

Ruby string allocations are not optimized and the code violates Rubocop's default styling rules.

## Suggested fix

1. Add `# frozen_string_literal: true` at the very top of packaging/homebrew/playhouse.rb.

**Files:** packaging/homebrew/playhouse.rb

## Acceptance criteria

- [ ] The file packaging/homebrew/playhouse.rb contains the magic frozen string literal comment as its first line

## Priority

P2 - follow-up (code style)
