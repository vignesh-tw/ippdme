## Why

The imposter can now serve TLS and mutual TLS, so clients can be tested against encrypted endpoints without a real machine. TLS is documented in the README and CLAUDE.md.

## What Changes

- `tls:` block in the imposter YAML and `.tls(...)` on the builder (`imposter`).
- TLS is enabled by default in Python, the TUI and the imposter (`tls-transport`).

## Capabilities

### Modified Capabilities

- `imposter`
- `tls-transport`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `7c4d3f7`. No code changes.
