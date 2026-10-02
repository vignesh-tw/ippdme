## Why

`ippdme-imposter` is a stub server for headless testing: a predicate selects a stub and a sequence of responses is replayed, configured from YAML or from Rust.

## What Changes

- Specify stub matching, response sequencing, response kinds, the YAML schema, the Rust builder, call verification and the standalone binary (`imposter`).

## Capabilities

### New Capabilities

- `imposter`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `a12b396`. No code changes.
