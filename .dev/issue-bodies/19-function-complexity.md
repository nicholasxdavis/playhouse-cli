## Problem

The render_score_report function has high cyclomatic complexity (count = 31).

## Impact

High complexity makes the function hard to maintain, read, verify, and modify without introducing regressions.

## Suggested fix

1. Refactor render_score_report in src/tui/ui_blocks.rs.
2. Extract smaller helper methods or sub-blocks to handle specific categories and rendering logic.

**Files:** src/tui/ui_blocks.rs

## Acceptance criteria

- [ ] render_score_report complexity count is reduced below the threshold (count < 30)

## Priority

P1 - should ship (code complexity)
