## Why

The TUI needs to connect to TLS and mutual-TLS endpoints in client mode.

## What Changes

- `--ca-cert`, `--server-name`, `--client-cert`, `--client-key` flags (`tui`).
- Record the TUI surface of TLS (`tls-transport`).

## Capabilities

### Modified Capabilities

- `tui`
- `tls-transport`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `f06a4c2`. No code changes.
