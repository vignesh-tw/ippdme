## Why

Debugging an I++ conversation needs the view from the port itself, malformed lines included, not the view of one application.

## What Changes

- `IppTap` forwards bytes unchanged and reports each line per direction (`wire-tap`).

## Capabilities

### New Capabilities

- `wire-tap`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `fbc9f18`. No code changes.
