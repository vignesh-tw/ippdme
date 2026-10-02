## Why

The CartCMM coordinate-system methods (get/set transformations, named and saved coordinate systems) were added as typed commands with canned mock replies and presets. `docs/SUPPORTED_METHODS.md` (920eca1) now tracks which spec methods are typed.

## What Changes

- Typed variants for coordinate-system and transformation methods (`typed-commands`).
- Canned mock replies for them (`mock-server`).
- TUI presets for them (`tui`).

## Capabilities

### Modified Capabilities

- `typed-commands`
- `mock-server`
- `tui`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `80dc16a, 920eca1`. No code changes.
