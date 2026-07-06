## Problem

Complex binary/boolean expressions are used in control statements, making them hard to read.

## Impact

Logical conditions that are too complex increase the chance of logic errors during code modifications.

## Suggested fix

1. Audit boolean statements in score.rs and detect.rs.
2. Break down complex conditions by extracting them to descriptively named local variables.

**Files:** src/score.rs, src/detect.rs

## Acceptance criteria

- [ ] Complex boolean logic conditions are simplified and extracted into local variables

## Priority

P1 - should ship (readability)
