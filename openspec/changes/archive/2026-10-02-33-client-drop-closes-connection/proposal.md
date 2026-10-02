## Why

A dropped `IppClient` left its reader task holding the socket open until the server hung up.

## What Changes

- Dropping the client stops the reader task and closes the connection (`ipp-client`).

## Capabilities

### Modified Capabilities

- `ipp-client`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `bbfc3e0`. No code changes.
