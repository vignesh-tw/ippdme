## Why

The Python examples need a Python environment and running imposters, so they are exercised by the Python CI job via `examples/python/run-all.sh`.

## What Changes

- `run-all.sh` starts the plain, TLS and mTLS imposters, runs the Python clients and stops them (`examples`).
- The Python CI job runs it (`ci-pipeline`).

## Capabilities

### Modified Capabilities

- `examples`
- `ci-pipeline`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `50cd9d0`. No code changes.
