## ADDED Requirements
### Requirement: TLS certificates are generated, not committed
`examples/tls/make-certs.sh` SHALL create a CA, a server certificate and a client certificate under `examples/tls/certs/`, which SHALL be git-ignored.

#### Scenario: Fresh checkout
- **WHEN** the repository is cloned
- **THEN** no private keys are present until `make-certs.sh` is run

### Requirement: Client examples
`examples/python/plain_client.py` and `examples/python/tls_client.py` (with `--mtls`) SHALL exercise the Python package against the example imposters.

#### Scenario: Python client
- **WHEN** `python examples/python/plain_client.py` runs against the plain imposter
- **THEN** it completes without error

## MODIFIED Requirements
### Requirement: Example imposters
The repository SHALL ship `examples/plain/imposter.yaml` (a well-behaved machine on port 1294), `examples/plain/faults.yaml` (a misbehaving machine on port 1294), `examples/tls/imposter-tls.yaml` (TLS on 1295) and `examples/tls/imposter-mtls.yaml` (mutual TLS on 1296).

#### Scenario: Run the plain machine
- **WHEN** `cargo run -p ippdme-imposter --bin ippdme-imposter -- examples/plain/imposter.yaml` is run
- **THEN** it prints that it is listening on `127.0.0.1:1294`

