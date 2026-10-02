## Why

I++ sessions are per connection, so handlers need per-connection state. Handlers also need to simulate faulty servers: dropped connections and garbage lines.

## What Changes

- `Handler::Session` created per connection; handlers answer with an action (reply, raw line, close) (`server-transport`).
- `drop: true` and `malformed: "text"` response kinds (`imposter`).

## Capabilities

### Modified Capabilities

- `server-transport`
- `imposter`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `a5620ad`. No code changes.
