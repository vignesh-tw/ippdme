## MODIFIED Requirements
### Requirement: TLS 1.3 only, behind a feature flag
TLS support SHALL be gated by the `tls` Cargo feature of `ippdme-net` and SHALL offer only TLS 1.3. It SHALL be enabled by default in the Python package, the TUI and the imposter. There SHALL be no fallback to plaintext.

#### Scenario: Plaintext peer
- **WHEN** a TLS client connects to a server that does not speak TLS
- **THEN** the connect fails with a timeout or handshake error and never sends I++ traffic in the clear

### Requirement: Any server can serve TLS
`IppServer::bind_tls`, `IppMockServer::bind_tls` (and the mock builder's `tls`) and the imposter's TLS option SHALL upgrade every accepted connection to TLS before reading any I++ traffic.

#### Scenario: Mock over TLS
- **WHEN** a mock server is bound with a TLS config and a TLS client connects with the matching CA
- **THEN** `StartSession` is answered normally

### Requirement: Surfaces expose the same options
Python SHALL expose TLS as `IppClient.connect(addr, ca_cert=, server_name=, client_cert=, client_key=, connect_timeout=)` and `IppMockServer(port, cert=, key=, client_ca=)`. The TUI SHALL expose `--ca-cert`, `--server-name`, `--client-cert` and `--client-key` for client mode. The imposter YAML SHALL expose a `tls:` block with `cert`, `key` and optional `client_ca`. Details of each surface are in the `python-bindings`, `tui` and `imposter` specs.

#### Scenario: Round trip across surfaces
- **WHEN** an imposter serves TLS from YAML and a Python client connects with the CA
- **THEN** commands are answered

