## Why

The mock server and the imposter each had their own listener and framing code. `IppServer` now owns the accept/frame/dispatch loop and both are handlers on top of it, and `IppClient` accepts any byte stream.

## What Changes

- Specify `IppServer` and its `Handler` trait (`server-transport`).
- The client can wrap any `AsyncRead + AsyncWrite` stream (`ipp-client`).

## Capabilities

### New Capabilities

- `server-transport`

### Modified Capabilities

- `ipp-client`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `18b8da7`. No code changes.
