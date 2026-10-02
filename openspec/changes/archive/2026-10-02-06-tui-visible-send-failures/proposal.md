## Why

The TUI silently dropped commands sent before the connection completed. Rejected sends are now logged as red entries so they are not lost behind a status message.

## What Changes

- A send rejected locally (not connected, still connecting, parse error) is logged and shown in the status bar (`tui`).

## Capabilities

### Modified Capabilities

- `tui`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `ccc8688`. No code changes.
