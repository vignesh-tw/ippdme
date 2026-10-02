# ci-pipeline Specification

## Purpose

What CI guarantees on every push to `main` and every pull request (`.github/workflows/ci.yml`), and how the workspace builds and ships.

## Requirements

### Requirement: Rust quality gate
CI SHALL run `cargo fmt --all -- --check`, `cargo clippy --workspace --exclude ippdme-py --all-targets -- -D warnings` and `cargo test --workspace --exclude ippdme-py`, and SHALL also clippy and test `ippdme-net` with `--features tls`. The Python extension crate SHALL be excluded from plain cargo workspace commands because it cannot link without `maturin` (or a dynamic-lookup linker flag) on macOS.

#### Scenario: Warnings fail the build
- **WHEN** clippy reports a warning
- **THEN** the Rust job fails

### Requirement: ARM64 cross-compilation
CI SHALL build `ippdme-core` and `ippdme-net` with TLS for `aarch64-unknown-linux-gnu` using the aarch64 GCC as both linker and C compiler (the TLS provider compiles C and assembly), keeping the libraries usable on a Raspberry Pi.

#### Scenario: ARM build
- **WHEN** a change breaks the aarch64 build of the core or net crates
- **THEN** the cross-compile job fails

### Requirement: Python bindings are built and tested
CI SHALL create a virtualenv with `uv venv`, install `maturin` and `pytest`, run `maturin develop -m crates/ippdme-py/Cargo.toml` (which refuses to run without an active virtualenv), run `pytest tests/`, and then run `examples/python/run-all.sh`.

#### Scenario: Python example breaks
- **WHEN** a binding change breaks `examples/python/plain_client.py`
- **THEN** the Python job fails

#### Scenario: Binding change breaks tests
- **WHEN** a binding change breaks a Python test
- **THEN** the Python job fails

### Requirement: Python-level tests against the mock
`tests/` SHALL contain pytest integration tests of the Python client, including TLS and mutual TLS, run against `IppMockServer`.

#### Scenario: Strict ordering from Python
- **WHEN** the Python tests run a strict mock and send `go_to` before `home`
- **THEN** they assert an error response with reason `NotHomed`

#### Scenario: Unknown command from Python
- **WHEN** the Python tests send an unknown command
- **THEN** they assert an error response with a reason

### Requirement: Licensing and distribution
The workspace SHALL be dual-licensed MIT and Apache-2.0 and SHALL ship as Rust libraries, a Python package (`pip install ippdme`), a TUI and an imposter binary.

#### Scenario: Licence files
- **WHEN** the repository root is listed
- **THEN** `LICENSE-MIT` and `LICENSE-APACHE` are present
