# ippdme

An open-source, memory-safe implementation of the **I++ DME** protocol — the
line-based ASCII protocol used across industrial manufacturing to command
Coordinate Measuring Machines (CMMs) and other dimensional measurement
equipment, typically over TCP/IP port `1294`.

`ippdme` ships as three things from one Rust workspace:

- **`ippdme-core`** / **`ippdme-net`** — a pure Rust protocol parser/AST and
  an async Tokio TCP client + mock CMM server, usable directly from Rust and
  cross-compilable to ARM (e.g. Raspberry Pi).
- **`ippdme`** (PyPI) — Python bindings via [PyO3](https://pyo3.rs) /
  [Maturin](https://www.maturin.rs), exposing a synchronous, Pythonic API for
  QA/CI pipelines and data scientists.
- **`ippdme-tui`** — an interactive terminal UI ("Postman for I++ DME") to
  inspect, mock, and command CMMs live over the wire.

See [`docs/SUPPORTED_METHODS.md`](docs/SUPPORTED_METHODS.md) for which I++
DME protocol methods have typed support today versus which are reachable only
via the generic `Command::Raw` fallback.

## Workspace layout

```
ippdme/
├── Cargo.toml                   # Workspace manifest
├── pyproject.toml                # Maturin build configuration for PyPI
├── crates/
│   ├── ippdme-core/              # Protocol parser, AST, serializer
│   ├── ippdme-net/                # Async Tokio TCP client & mock server
│   ├── ippdme-py/                 # PyO3 bindings for Python
│   └── ippdme-tui/                # Ratatui terminal UI
└── python/ippdme/                 # Python package source (mixed maturin layout)
```

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

### Connect to a real CMM or gateway

Leave the mode as `Client`, press `h`/`p` to set the target host/port (default
`127.0.0.1:1294`), then press `c` to connect — the rest of the workflow
(presets, raw commands, live stream, export) is identical.

## Protocol essentials

- Transport: plain-text TCP, CRLF line endings.
- Every command starts with a 5-digit zero-padded tag: `00001 GoTo(X(10.0), Y(20.0), Z(5.0))`.
- Responses are marked `#` (ack), `!` (error), or `%` (data/event):
  `00001 # Ack()`, `00001 ! Error(UnknownCommand)`, `00001 % PtMeas(X(10.002), ...)`.

## License

MIT OR Apache-2.0
