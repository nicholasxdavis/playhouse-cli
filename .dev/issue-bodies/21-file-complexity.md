## Problem

Large source files contain very high total complexity counts (e.g. count = 71 in ui_blocks.rs).

## Impact

Monolithic source files become difficult to maintain and navigate as the project grows.

## Suggested fix

1. Decompose monolithic files into smaller modules.
2. For example, move TUI components, models, and utility handlers in src/tui/ui_blocks.rs into separate files under src/tui/components/.

**Files:** src/tui/ui_blocks.rs, src/cli/handlers.rs

## Acceptance criteria

- [ ] Total complexity count of individual files is reduced below the lint threshold

## Priority

P1 - should ship (file structure)
