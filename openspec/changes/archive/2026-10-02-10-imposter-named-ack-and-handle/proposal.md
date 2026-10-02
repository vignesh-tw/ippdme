## Why

`StartSession` replies `Ready()` under the ack marker, so a stub needs a named ack. The call log also has to be readable after `serve` consumes the imposter. The README and CLAUDE.md gained imposter documentation (e68aa86).

## What Changes

- `ack: Name` replies `Name()` under the ack marker (`imposter`).
- `Imposter::handle()` returns a handle that outlives `serve` (`imposter`).

## Capabilities

### Modified Capabilities

- `imposter`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `cb60659, e68aa86`. No code changes.
