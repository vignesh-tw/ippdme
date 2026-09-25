# CLAUDE.md

Guidance for Claude Code (or any future agent) working in this repository.

## What this is

`ippdme` is an open-source, memory-safe implementation of the **I++ DME**
protocol — a line-based ASCII protocol (CRLF-terminated, default TCP port
`1294`) used to command Coordinate Measuring Machines (CMMs) in industrial
metrology. See `docs/ippdme_standard.pdf` for the full protocol spec.

It's a 4-crate Cargo workspace that ships as three things:

- Pure Rust libraries (`ippdme-core`, `ippdme-net`), cross-compilable to
  `aarch64-unknown-linux-gnu` (Raspberry Pi).
- A Python package (`pip install ippdme`) via PyO3/Maturin.
- A terminal UI (`ippdme-tui`), "Postman for I++ DME".

## Workspace layout

```
crates/ippdme-core/   Protocol AST (Term/Message), winnow parser, Display serializer, typed Command wrappers
crates/ippdme-net/    Tokio async IppClient + IppMockServer, built on ippdme-core
crates/ippdme-py/     PyO3 bindings (crate name `_ippdme`, imported as ippdme._ippdme)
crates/ippdme-tui/    Ratatui terminal UI
python/ippdme/        Python package source (mixed maturin layout: __init__.py, testing.py, py.typed)
tests/                Python-level integration tests (pytest) against the mock server
```

## Architecture notes

- **`ippdme-core` is pure, no I/O.** The AST (`Term`, `Message`, `Tag`,
  `Marker`) is generic — any I++ command can round-trip through it even if no
  typed `Command` variant exists yet (`Command::Raw` is the fallback). Typed
  commands in `commands.rs` are ergonomic sugar on top, not the source of
  truth. When adding a new command, add a `Command` variant + `From`/`TryFrom`
  impls, not a new AST node type.
- **`ippdme-net`'s `IppClient`** allocates tags via an `AtomicU32` and
  correlates responses through a `HashMap<Tag, oneshot::Sender<Message>>`
  guarded by a `Mutex`, fed by a reader task that also fans out every inbound
  message on a `broadcast` channel (`subscribe()`) for live-inspection use
  cases (the TUI does *not* currently use this — see below). `allocate_tag()`
  + `send_with_tag()` exist specifically so a caller (the TUI) can log the
  outbound message with its real tag *before* awaiting the response.
- **`ippdme-net`'s `IppMockServer`** simulates `GoTo`/`Home` latency (500ms
  sleep) and returns synthetic `PtMeas` coordinates. It's the reference
  fixture for both Rust integration tests and the Python `ippdme.testing`
  pytest fixtures — keep its behavior in sync with both test suites if it
  changes.
- **`ippdme-py`** wraps everything in a single shared `tokio::runtime::Runtime`
  (`OnceLock`, see `runtime.rs`) and exposes a *synchronous* Python API: each
  method does `py.allow_threads(|| runtime().block_on(...))`. This means
  every async closure passed to `IppClient`'s methods needs `+ Send` bounds
  (see `client.rs::block_on`) — this bit us once, watch for it if refactoring.
- **`ippdme-tui`** drives a single `tokio::select!` loop (`main.rs::run`)
  between a `crossterm::EventStream`, a 100ms tick, and an
  `mpsc::UnboundedReceiver<AppEvent>` drained in `App::tick()`. Network calls
  are never awaited inline in the event loop — they're `tokio::spawn`ed and
  report back via `AppEvent` (see `app.rs`). Follow this pattern for any new
  async action; blocking the select loop on a network call will freeze
  keystrokes and redraws.

## Build, test, lint

Run these before considering any change done:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --exclude ippdme-py --all-targets -- -D warnings
cargo test --workspace --exclude ippdme-py
```

`ippdme-py` is excluded from plain `cargo` workspace commands — see the macOS
gotcha below. To check it:

```bash
export RUSTFLAGS="-C link-arg=-undefined -C link-arg=dynamic_lookup"  # macOS only, see below
cargo clippy -p ippdme-py -- -D warnings
```

To actually build and test the Python bindings, use `maturin` (it sets the
right link flags itself, no `RUSTFLAGS` needed):

```bash
python3 -m venv .venv && source .venv/bin/activate
pip install maturin pytest
maturin develop -m crates/ippdme-py/Cargo.toml
pytest tests/
```

Run the TUI directly (see README for the interactive walkthrough):

```bash
cargo run -p ippdme-tui
```

## Gotchas / hard-won context

- **`ippdme-py` cannot be built with plain `cargo build -p ippdme-py` on
  macOS.** It's a `cdylib` with the `extension-module` PyO3 feature, so it
  intentionally doesn't link against `libpython`; macOS needs
  `-undefined dynamic_lookup` to allow that, which `maturin` supplies
  automatically but bare `cargo` does not. Either use `maturin develop`, or
  set `RUSTFLAGS="-C link-arg=-undefined -C link-arg=dynamic_lookup"` first.
  This is why every `cargo build/test/clippy --workspace` command in this repo
  and in CI passes `--exclude ippdme-py`.
- **`clippy::useless_conversion` is a false positive on `#[pymethods]`.**
  PyO3's macro-expanded wrapper code triggers it spuriously
  (PyO3#4568-style issue); it's suppressed crate-wide in
  `crates/ippdme-py/src/lib.rs` via `#![allow(clippy::useless_conversion)]`.
  Don't try to "fix" this by rewriting the methods — the lint is wrong, not
  the code.
- **CI's Python job needs an active virtualenv before `maturin develop`** —
  `maturin develop` (as opposed to `maturin build`) refuses to run without one.
  The workflow creates `.venv` with `uv venv` and installs deps with
  `uv pip install` before running it. If you change the Python CI job, keep
  the venv-creation step.
- **`IppClient::send` internally calls `allocate_tag()` then
  `send_with_tag()`.** If you add a new "fire and log immediately" pattern
  (like the TUI does), reuse `send_with_tag`/`allocate_tag` rather than
  re-implementing tag correlation.
- **`docs/ippdme_standard.pdf`** is the reference protocol spec, checked into
  the repo for convenience. It's a large binary file — don't try to read it
  wholesale; consult it only for specific protocol questions.

## Conventions

- Workspace-level dependency versions live in the root `Cargo.toml`
  `[workspace.dependencies]` — add new shared deps there, not per-crate,
  unless the dependency is genuinely crate-specific (e.g. `ratatui`,
  `pyo3`).
- No `unsafe`, no unnecessary abstractions — this codebase favors small,
  direct modules (one concern per file: `codec.rs`, `client.rs`, `mock.rs`,
  etc.) over layered trait hierarchies.
- Commit messages and PRs should *not* carry any AI-attribution footer for
  this project — the user has explicitly opted out of that (see repo commit
  history for the established message style instead: short, imperative,
  one-line-plus-body when needed).
