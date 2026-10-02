## Why

Standard I++ DME is plaintext TCP. `ippdme-net` gained an optional TLS 1.3 transport (rustls + ring) with optional mutual TLS, for use through a TLS-terminating proxy or against an imposter.

## What Changes

- Specify TLS config, verification, mutual TLS and TLS-capable servers (`tls-transport`).
- CI also builds and tests the `tls` feature and cross-compiles it (`ci-pipeline`).

## Capabilities

### New Capabilities

- `tls-transport`

### Modified Capabilities

- `ci-pipeline`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `c2592c7`. No code changes.
