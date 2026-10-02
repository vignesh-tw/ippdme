## Why

Invalid values (NaN coordinates, non-unit normals, names with quotes) could be built and sent, and unknown commands silently became `Raw`. Constructors now reject invalid values and parsing is strict unless leniency is requested explicitly.

## What Changes

- `Command::try_from` fails on unknown commands; `from_term_lenient` opts in to `Raw` (`typed-commands`).
- Validated points, names, whole-number arguments and transformations (`typed-commands`).
- Invalid arguments surface as `ValueError` in Python (`python-bindings`).
- Client helpers fail before sending on invalid arguments (`ipp-client`).

## Capabilities

### Modified Capabilities

- `typed-commands`
- `ipp-client`
- `python-bindings`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `25f89ed`. No code changes.
