## Context

Handlers returned a single `Message`, which could neither keep per-connection state nor misbehave on purpose.

## Decision

`Handler` gains an associated `Session` type, created by `new_session` per connection and passed mutably to every call. `handle` returns an `Action`: `Reply(Message)`, `RawLine(String)` or `Close`. Closures stay usable as stateless handlers. The imposter exposes the two faulty actions as `malformed` and `drop` response kinds.
