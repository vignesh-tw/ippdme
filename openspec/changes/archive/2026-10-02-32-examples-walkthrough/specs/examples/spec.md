## MODIFIED Requirements
### Requirement: Client examples
`examples/python/plain_client.py` and `examples/python/tls_client.py` (with `--mtls`) SHALL exercise the Python package against the example imposters. `examples/rust-client` (`ippdme-example-client`) SHALL be a Rust application built on `ippdme-core` and `ippdme-net`. `examples/README.md` SHALL walk through starting a machine, talking to it with `nc`, Python, Rust and the TUI, watching traffic with the tap, injecting faults and trying TLS.

#### Scenario: Python client
- **WHEN** `python examples/python/plain_client.py` runs against the plain imposter
- **THEN** it completes without error

