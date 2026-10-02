## Context

Plaintext I++ DME carries machine commands unprotected. Real CMMs will not speak TLS, so TLS only helps with a terminating proxy or with software endpoints such as the imposter.

## Decisions

- rustls with the ring provider, TLS 1.3 only, behind the `tls` Cargo feature of `ippdme-net`.
- PEM bytes in, with `*_files` helpers, so callers decide where credentials come from.
- Mutual TLS is enabled on the server by supplying a client CA.
- All TLS and socket logic stays in `ippdme-net`; other crates pass options through.
