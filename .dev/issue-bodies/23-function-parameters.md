## Problem

The render_tool_call function has too many parameters (count = 6).

## Impact

Functions with many parameters are hard to test, document, and invoke correctly.

## Suggested fix

1. Group parameters of render_tool_call into a single configuration struct.
2. Pass the struct by reference to keep the function signature clean.

**Files:** src/tui/ui_blocks.rs

## Acceptance criteria

- [ ] Parameter count for render_tool_call is reduced by introducing a parameter struct

## Priority

P1 - should ship (function signature)
