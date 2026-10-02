## Why

Examples for TLS and mutual TLS need certificates; generating them keeps private keys out of the repository.

## What Changes

- TLS and mTLS imposters on ports 1295 and 1296, and `make-certs.sh` (`examples`).
- `plain_client.py` and `tls_client.py` (`examples`).

## Capabilities

### Modified Capabilities

- `examples`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `37b9f27`. No code changes.
