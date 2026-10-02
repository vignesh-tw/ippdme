## Why

Typed commands, mock replies and TUI presets were extended to the I++ DME server methods (daemons, errors, properties) and DME status methods (homed, user enable, machine class, `Get`, report subscriptions).

## What Changes

- Allow dots in identifiers so property paths like `Tool.PtMeasPar.Speed` parse (`wire-format`).
- Add typed variants for the server and DME status methods (`typed-commands`).
- Mock replies for those methods and for properties (`mock-server`).
- More TUI presets (`tui`).

## Capabilities

### Modified Capabilities

- `wire-format`
- `typed-commands`
- `mock-server`
- `tui`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `edb3872`. No code changes.
