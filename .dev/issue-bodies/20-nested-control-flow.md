## Problem

Deeply nested control flow (nesting level = 5) exists within the TUI rendering or CLI handlers.

## Impact

Monolithic conditional logic reduces code readability and increases the chance of logical errors.

## Suggested fix

1. Use early returns (guard clauses) to handle error states.
2. Flatten control structures with pattern matching or helper methods.

**Files:** src/tui/ui_blocks.rs, src/cli/handlers.rs

## Acceptance criteria

- [ ] Max nesting levels in control flow are reduced to 3 or less

## Priority

P1 - should ship (nesting complexity)
