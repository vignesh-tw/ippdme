## Why

Requests waiting for a reply only failed when their 5 s timeout expired, even if the connection had already gone away.

## What Changes

- Pending and later requests fail immediately with a connection-closed error (`ipp-client`).

## Capabilities

### Modified Capabilities

- `ipp-client`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `e64a63b`. No code changes.
