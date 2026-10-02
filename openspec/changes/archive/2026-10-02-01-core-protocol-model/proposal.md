## Why

The workspace starts with `ippdme-core`: a generic, I/O-free model of an I++ DME line plus a small set of typed commands. Nothing else can be specified until this exists.

## What Changes

- Specify tags, terms, markers and messages, their parser and serializer (`wire-format`).
- Specify the first `Command` enum (session, home, `GoTo`, `PtMeas`, `SetCoordSystem`, two scan stubs, `Raw`) and the response builders (`typed-commands`).

## Capabilities

### New Capabilities

- `wire-format`
- `typed-commands`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `f9db887`. No code changes.
