## Problem

The detect_at function in src/detect.rs has too many return statements (count = 9).

## Impact

Too many return paths make tracing the execution flow of the function difficult.

## Suggested fix

1. Refactor detect_at in src/detect.rs.
2. Group or consolidate match arms or simplify logic flow to reduce the number of early return statements.

**Files:** src/detect.rs

## Acceptance criteria

- [ ] Number of return statements in detect_at is reduced to 6 or fewer

## Priority

P1 - should ship (flow control)
