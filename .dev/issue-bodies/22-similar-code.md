## Problem

Similar code structures exist in multiple locations in the repository (e.g. 16 lines of similar code in 2 locations with mass = 78).

## Impact

Duplicate code forces duplicate maintenance and can lead to inconsistent behavior if a fix is applied to only one location.

## Suggested fix

1. Identify duplicated logic blocks (e.g. TUI banner/mascot printing or command executions).
2. Refactor them into a common utility function or shared component.

**Files:** src/tui/splash.rs, src/tui/mascot.rs

## Acceptance criteria

- [ ] Identified similar code blocks are consolidated into shared methods or helper traits

## Priority

P1 - should ship (duplication)
