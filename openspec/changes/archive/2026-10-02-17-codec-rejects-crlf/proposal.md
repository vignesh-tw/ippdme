## Why

A string argument containing a newline would end its line early and smuggle a second command onto the wire.

## What Changes

- The encoder refuses outbound messages containing CR or LF (`line-codec`).

## Capabilities

### Modified Capabilities

- `line-codec`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `bbc308a`. No code changes.
