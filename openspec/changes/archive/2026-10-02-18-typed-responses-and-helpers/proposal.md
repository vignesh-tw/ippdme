## Why

Callers had to pick apart raw reply messages. Response readers and typed `IppClient` helpers now return values and turn server errors into errors.

## What Changes

- `expect_ack`, `expect_data`, `parse_pt_meas`, `parse_dme_version`, `parse_flag` (`typed-commands`).
- Client helpers return `()`, a version string, a point or a flag, and fail on `Error(...)` (`ipp-client`).

## Capabilities

### Modified Capabilities

- `typed-commands`
- `ipp-client`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `1e37e06`. No code changes.
