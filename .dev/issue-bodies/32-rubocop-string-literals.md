## Problem

Ruby code uses double-quoted string literals where single-quoted string literals would suffice.

## Impact

Violates standard Ruby style guidelines and results in static analysis failures.

## Suggested fix

1. Update packaging/homebrew/playhouse.rb to use single-quoted strings when string interpolation or special symbols are not required.
2. Alternatively, configure rubocop to auto-correct the style issues.

**Files:** packaging/homebrew/playhouse.rb

## Acceptance criteria

- [ ] Rubocop style check for string literals passes cleanly

## Priority

P2 - follow-up (code style)
