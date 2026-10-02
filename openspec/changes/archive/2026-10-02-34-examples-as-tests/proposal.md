## Why

Documentation that promises behavior goes stale. Every example YAML is now started and the README's promises are asserted by a test.

## What Changes

- `examples/rust-client/tests/examples.rs` starts each example on an ephemeral port and asserts the documented behavior (`examples`).

## Capabilities

### Modified Capabilities

- `examples`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `fa664f1`. No code changes.
