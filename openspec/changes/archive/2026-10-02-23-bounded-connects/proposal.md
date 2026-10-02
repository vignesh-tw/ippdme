## Why

Pointing a TLS client at a machine without TLS, or a plain client at an unreachable host, hung. Connecting and handshaking are now bounded by a 5 s timeout with no plaintext fallback.

## What Changes

- Connect and connect-plus-handshake timeouts (`ipp-client`, `tls-transport`).
- `connect_timeout` in Python and `TimeoutError` mapping (`python-bindings`).

## Capabilities

### Modified Capabilities

- `ipp-client`
- `tls-transport`
- `python-bindings`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `415c3d7`. No code changes.
