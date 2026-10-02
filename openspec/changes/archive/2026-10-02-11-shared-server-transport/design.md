## Context

The mock server and the imposter both implemented TCP accept, CRLF framing and a per-connection loop.

## Decision

Introduce `IppServer<H: Handler>` in `ippdme-net`. A handler turns one inbound command into one response message. Any `Fn(Tag, Term) -> Future<Output = Message>` is a handler. The mock server and imposter become handlers; neither carries its own `TcpListener` or `Framed` code. `IppClient::from_stream` and `serve_connection` accept any `AsyncRead + AsyncWrite` so other transports (TLS, in-memory pipes) plug in.

## Consequences

Transport features (TLS, timeouts, sessions) are written once, in `ippdme-net`.
