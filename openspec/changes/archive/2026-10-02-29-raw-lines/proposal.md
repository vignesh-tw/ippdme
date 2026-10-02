## Why

Testing a server's reaction to malformed input needs a way to send exactly what was typed, tag included, like `nc`.

## What Changes

- `leading_tag` reads a tag from a raw line (`wire-format`).
- The encoder can write a raw line verbatim and still refuses CR/LF (`line-codec`).
- `IppClient::send_line` (`ipp-client`).

## Capabilities

### Modified Capabilities

- `wire-format`
- `line-codec`
- `ipp-client`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `f52187c`. No code changes.
