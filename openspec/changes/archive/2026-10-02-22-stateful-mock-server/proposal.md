## Why

The mock answered from constants. It is now a small machine simulator with per-connection state, configurable latency and optional strict call-order enforcement, and it is the reference fixture for the Rust and Python test suites.

## What Changes

- Per-connection machine state: session, homed, user enabled, position, coordinate system, tool, saved coordinate systems (`mock-server`).
- `MockConfig` latency and `strict` mode; builder and ephemeral helpers (`mock-server`).
- `latency_ms` and `strict` on the Python `IppMockServer` (`python-bindings`).

## Capabilities

### Modified Capabilities

- `mock-server`
- `python-bindings`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `db4a2b3`. No code changes.
