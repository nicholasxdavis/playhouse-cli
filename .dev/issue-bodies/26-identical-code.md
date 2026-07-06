## Problem

Found 17 lines of identical code in 2 separate locations (mass = 112).

## Impact

Increases repository size and maintenance overhead. Bug fixes must be duplicated across identical sections.

## Suggested fix

1. Locate identical code blocks in TUI modules (e.g. splash.rs and mascot.rs).
2. Refactor the identical code into a single utility helper function in src/tui/ui_blocks.rs or a new utility module.

**Files:** src/tui/splash.rs, src/tui/mascot.rs

## Acceptance criteria

- [ ] Identical code blocks are refactored into a single shared helper function

## Priority

P1 - should ship (duplication)
