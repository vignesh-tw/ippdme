# ippdme

An open-source, memory-safe implementation of the **I++ DME** protocol — the
line-based ASCII protocol used across industrial manufacturing to command
Coordinate Measuring Machines (CMMs) and other dimensional measurement
equipment, typically over TCP/IP port `1294`.

`ippdme` ships as four things from one Rust workspace:

- **`ippdme-core`** / **`ippdme-net`** — a pure Rust protocol parser/AST and
  an async Tokio TCP client + mock CMM server, usable directly from Rust and
  cross-compilable to ARM (e.g. Raspberry Pi).
- **`ippdme`** (PyPI) — Python bindings via [PyO3](https://pyo3.rs) /
  [Maturin](https://www.maturin.rs), exposing a synchronous, Pythonic API for
  QA/CI pipelines and data scientists.
- **`ippdme-tui`** — an interactive terminal UI ("Postman for I++ DME") to
  inspect, mock, and command CMMs live over the wire, for manual/exploratory
  use.
- **`ippdme-imposter`** — a [Mountebank](https://www.mbtest.org)-style,
  YAML-configurable stub server for headless, automated testing against a
  virtual CMM, without touching Rust or the TUI.

See [`docs/SUPPORTED_METHODS.md`](docs/SUPPORTED_METHODS.md) for which I++
DME protocol methods have typed support today versus which are reachable only
via the explicit `Command::raw` escape hatch (`Command::try_from` is strict;
`Command::from_term_lenient` falls back to it).

## Workspace layout

```
ippdme/
├── Cargo.toml                   # Workspace manifest
├── pyproject.toml                # Maturin build configuration for PyPI
├── crates/
│   ├── ippdme-core/               # Protocol parser, AST, serializer
│   ├── ippdme-net/                # Async Tokio client, server transport, mock server, wire tap
│   ├── ippdme-imposter/           # Mountebank-style YAML stub server
│   ├── ippdme-py/                 # PyO3 bindings for Python
│   └── ippdme-tui/                # Ratatui terminal UI
├── examples/                      # Run-it-yourself walkthrough: imposters, TLS, Python and Rust clients
└── python/ippdme/                 # Python package source (mixed maturin layout)
```

## Try it

[`examples/`](examples/README.md) is a hands-on walkthrough on one PC: start a
virtual CMM from a YAML file, talk to it with `nc`, Python, Rust or the TUI,
watch the traffic on the port, inject faults, and try TLS and mutual TLS.

## Rust

`ippdme-py` is a Python extension module and needs `maturin` (see below), not
a plain `cargo build`, so exclude it from workspace-wide commands:

```bash
cargo build --workspace --exclude ippdme-py
cargo test --workspace --exclude ippdme-py
cargo clippy --workspace --exclude ippdme-py --all-targets -- -D warnings
```

## Python

```bash
python3 -m venv .venv && source .venv/bin/activate
pip install maturin
maturin develop -m crates/ippdme-py/Cargo.toml

python3 -c "
from ippdme import IppClient, IppMockServer

server = IppMockServer(port=1294)
server.start_in_background()

client = IppClient.connect('127.0.0.1:1294')
client.start_session()
response = client.go_to(x=10.0, y=20.0, z=5.0)
print(response.is_ack())
"
```

## TUI ("Postman for I++ DME")

Build and run `ippdme-tui` directly with Cargo — no separate install step:

```bash
cargo run -p ippdme-tui
```

### Connect to the built-in mock server

The TUI can act as its own mock CMM, so you can try the whole flow without any
external hardware or a separate mock-server process:

1. Launch it: `cargo run -p ippdme-tui`.
2. Press `m` to toggle the mode shown in the connection bar from `Client` to
   `Mock Server`.
3. Press `c` to connect. This binds an embedded `IppMockServer` on
   `127.0.0.1:1294` (press `p` beforehand to edit the port, `h` for the host,
   if you want a different one) and automatically connects a client to it, so
   you can immediately exercise it. The bar turns green and shows
   `CONNECTED`.
4. Use `↑`/`↓` to pick a preset in the left sidebar (e.g. `Session →
   StartSession()`, `Motion → GoTo(10, 10, 10)`, `Measurement → PtMeas()`)
   and press `Enter` to send it. The outbound command appears in blue in the
   live protocol stream on the right, followed by the color-coded response
   (green `#` ack, yellow `%` data, red `!` error) with its round-trip
   latency.
5. Press `Tab` to move focus into the raw command box and type any I++ term
   directly, e.g. `SetCoordSystem(PCS)`, then `Enter` to send it.
6. Press `e` at any time to export the session transcript to a timestamped
   `.log` and `.json` file in the current directory.
7. Press `c` again to disconnect, `q` or `Esc` to quit.

### Raw line input and Tap mode

Press `r` to make the input box send whole lines exactly as typed, tag
included and unparsed, like typing into `nc`. Press `m` until the mode reads
**Tap** to watch the raw lines crossing a port between any client and server
(clients connect to `--tap-listen`, default 1297; the connection bar's host and
port are where it forwards to). The same tap is available without the UI as
`cargo run -p ippdme-net --bin ippdme-tap -- --target 127.0.0.1:1294`.
[`examples/`](examples/README.md) walks through both.

### Connect to a real CMM or gateway

Leave the mode as `Client`, press `h`/`p` to set the target host/port (default
`127.0.0.1:1294`), then press `c` to connect — the rest of the workflow
(presets, raw commands, live stream, export) is identical.

## Imposter (headless automated testing)

`ippdme-imposter` is for scripted/CI testing: define how a virtual CMM
should respond to specific calls in a YAML file, no Rust required, then
point any I++ DME client at it.

```bash
cargo run -p ippdme-imposter --bin ippdme-imposter -- examples/plain/imposter.yaml
```

```yaml
port: 1294
stubs:
  - predicate:
      call: PtMeas
    responses:
      - data:
          call: PtMeas
          args:
            - { name: X, value: 10.002 }
            - { name: Y, value: 20.001 }
        after_ms: 500   # simulate probe-touch latency

  - predicate:
      call: GetErrorInfo
      args: [42]
    responses:
      - error: CollisionDetected
```

The same `Predicate`/`Stub`/`ResponseSpec` types are also usable directly
from Rust (e.g. inline in a `#[tokio::test]`) via `Imposter::builder()` —
see `crates/ippdme-imposter/tests/imposter_integration.rs` for examples,
including response sequencing (a stub can fail once, then succeed) and
`Imposter::handle().received_calls()` for verifying what a client under
test actually sent.

## TLS

Standard I++ DME is plaintext TCP. `ippdme-net`'s `tls` feature (rustls, TLS
1.3 only) wraps the same protocol in TLS, with optional mutual TLS. It is
enabled by default in the Python package, the TUI and the imposter.

- **Rust:** `IppClient::connect_tls(addr, &TlsClientConfig::new(name, ca_pem, identity))`;
  server side `IppServer::bind_tls` / `IppMockServer::bind_tls` with a
  `TlsServerConfig` (pass a client CA to require client certificates). Any
  `AsyncRead + AsyncWrite` stream can also go through `IppClient::from_stream`.
- **Python:** `IppClient.connect(addr, ca_cert=..., server_name=..., client_cert=..., client_key=...)`
  and `IppMockServer(port, cert=..., key=..., client_ca=...)`, all PEM file paths.
- **TUI:** `ippdme-tui --ca-cert ca.pem [--server-name NAME] [--client-cert c.pem --client-key c.key]`
  (client mode only).
- **Imposter:** a top-level `tls:` block in the YAML with `cert`, `key` and an
  optional `client_ca`, or `Imposter::builder().tls(...)` in Rust.

Connecting is bounded: the TCP connect plus TLS handshake must finish within 5s
(`TlsClientConfig::with_connect_timeout`, `connect_timeout=` in Python), so
pointing a TLS client at a machine without TLS fails with a timeout or handshake
error instead of hanging. There is no fallback to plaintext.

A real CMM will not speak TLS itself; put a TLS-terminating proxy in front of
it and point the client at the proxy.

## Protocol essentials

- Transport: plain-text TCP, CRLF line endings.
- Every command starts with a 5-digit zero-padded tag: `00001 GoTo(X(10.0), Y(20.0), Z(5.0))`.
- Responses are marked `#` (ack), `!` (error), or `%` (data/event):
  `00001 # Ack()`, `00001 ! Error(UnknownCommand)`, `00001 % PtMeas(X(10.002), ...)`.

## License

MIT OR Apache-2.0
