# examples Specification

## Purpose

A hands-on walkthrough under `examples/` that runs on one PC over `127.0.0.1`: virtual CMMs from YAML and clients that talk to them.

## Requirements

### Requirement: Example imposters
The repository SHALL ship `examples/plain/imposter.yaml` (a well-behaved machine on port 1294), `examples/plain/faults.yaml` (a misbehaving machine on port 1294), `examples/tls/imposter-tls.yaml` (TLS on 1295) and `examples/tls/imposter-mtls.yaml` (mutual TLS on 1296).

#### Scenario: Run the plain machine
- **WHEN** `cargo run -p ippdme-imposter --bin ippdme-imposter -- examples/plain/imposter.yaml` is run
- **THEN** it prints that it is listening on `127.0.0.1:1294`

### Requirement: Fault scenarios
`faults.yaml` SHALL demonstrate: a first `GoTo` that returns `Error(CollisionDetected)` and later ones that succeed, a `PtMeas` that replies after 8 seconds (longer than the client's 5 s timeout), an `IsHomed` that drops the connection, and a `GetMachineClass` that sends a malformed line.

#### Scenario: Retry test
- **WHEN** a client sends `GoTo` twice to the faults machine
- **THEN** the first is a collision error and the second is acknowledged

#### Scenario: Client timeout
- **WHEN** a client with the default timeout sends `PtMeas` to the faults machine
- **THEN** the client reports a timeout

### Requirement: TLS certificates are generated, not committed
`examples/tls/make-certs.sh` SHALL create a CA, a server certificate and a client certificate under `examples/tls/certs/`, which SHALL be git-ignored.

#### Scenario: Fresh checkout
- **WHEN** the repository is cloned
- **THEN** no private keys are present until `make-certs.sh` is run

### Requirement: Client examples
`examples/python/plain_client.py` and `examples/python/tls_client.py` (with `--mtls`) SHALL exercise the Python package against the example imposters. `examples/rust-client` (`ippdme-example-client`) SHALL be a Rust application built on `ippdme-core` and `ippdme-net`. `examples/README.md` SHALL walk through starting a machine, talking to it with `nc`, Python, Rust and the TUI, watching traffic with the tap, injecting faults and trying TLS.

#### Scenario: Python client
- **WHEN** `python examples/python/plain_client.py` runs against the plain imposter
- **THEN** it completes without error

### Requirement: Examples are executable tests
Every example YAML SHALL be parsed and started by a Rust test (`examples/rust-client/tests/examples.rs`) on an ephemeral port by rewriting only its `port:` line, and the behavior `examples/README.md` promises SHALL be asserted.

#### Scenario: Stale example
- **WHEN** a code change makes an example YAML invalid or its documented reply wrong
- **THEN** the example test fails
