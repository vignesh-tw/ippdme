## Why

`ippdme-tui` is a Postman-style terminal UI for manually commanding a CMM. It needs the client to let a caller log a tag before awaiting the reply.

## What Changes

- Specify the TUI: modes, connect/disconnect, presets, typed input, protocol log, export, quitting (`tui`).
- Add tag pre-allocation to the client (`ipp-client`).

## Capabilities

### New Capabilities

- `tui`

### Modified Capabilities

- `ipp-client`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `21725ff`. No code changes.
