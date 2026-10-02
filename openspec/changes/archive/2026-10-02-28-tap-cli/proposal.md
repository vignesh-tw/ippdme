## Why

The tap should be usable without writing code: point a client at it and watch the lines scroll by.

## What Changes

- `ippdme-tap --target HOST:PORT [--listen ADDR:PORT]` (`wire-tap`).

## Capabilities

### Modified Capabilities

- `wire-tap`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `4f6d61a`. No code changes.
