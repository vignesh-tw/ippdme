## MODIFIED Requirements
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

