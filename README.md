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

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
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

## Protocol essentials

- Transport: plain-text TCP, CRLF line endings.
- Every command starts with a 5-digit zero-padded tag: `00001 GoTo(X(10.0), Y(20.0), Z(5.0))`.
- Responses are marked `#` (ack), `!` (error), or `%` (data/event):
  `00001 # Ack()`, `00001 ! Error(UnknownCommand)`, `00001 % PtMeas(X(10.002), ...)`.

## License

MIT OR Apache-2.0
