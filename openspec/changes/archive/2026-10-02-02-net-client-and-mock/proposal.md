## Why

`ippdme-net` adds the Tokio TCP client, the CRLF line codec and a first mock CMM, which are the foundation every later transport feature builds on.

## What Changes

- Specify line framing (`line-codec`).
- Specify the async client with tag correlation, timeouts and an inbound broadcast (`ipp-client`).
- Specify the first, stateless mock server (`mock-server`).

## Capabilities

### New Capabilities

- `line-codec`
- `ipp-client`
- `mock-server`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `d32cdf3`. No code changes.
