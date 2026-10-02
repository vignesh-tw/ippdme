## Why

The Python client and mock server needed to reach TLS and mutual-TLS endpoints.

## What Changes

- TLS options on `IppClient.connect` and `IppMockServer` (`python-bindings`).
- Python TLS tests (`ci-pipeline`).
- Record the Python surface of TLS (`tls-transport`).

## Capabilities

### Modified Capabilities

- `python-bindings`
- `ci-pipeline`
- `tls-transport`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `f99b6bd`. No code changes.
