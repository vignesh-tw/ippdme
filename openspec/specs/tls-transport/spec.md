# tls-transport Specification

## Purpose

Optional TLS 1.3 (and mutual TLS) around the I++ DME protocol. Standard I++ DME is plaintext TCP; TLS is an add-on for protecting traffic through a TLS-terminating proxy or an imposter. All TLS and socket logic lives in `ippdme-net` behind the `tls` feature (`tls.rs`, rustls + ring). The Python package, the TUI and the imposter only pass options through to it.

## Requirements

### Requirement: TLS 1.3 only, behind a feature flag
TLS support SHALL be gated by the `tls` Cargo feature of `ippdme-net` and SHALL offer only TLS 1.3. It SHALL be enabled by default in the Python package, the TUI and the imposter. There SHALL be no fallback to plaintext.

#### Scenario: Plaintext peer
- **WHEN** a TLS client connects to a server that does not speak TLS
- **THEN** the connect fails with a timeout or handshake error and never sends I++ traffic in the clear

### Requirement: Credentials are supplied as PEM
Certificates and keys SHALL be passed as PEM bytes, with `from_files` helpers that read them from disk. A `TlsIdentity` SHALL hold a certificate chain and private key. A client config SHALL hold the trusted CA, the server name to verify against, and an optional client identity. A server config SHALL hold the server identity and an optional client CA.

#### Scenario: Invalid PEM
- **WHEN** a config is built from bytes that are not valid PEM
- **THEN** construction fails with a TLS error

### Requirement: Clients verify the server
`IppClient::connect_tls` SHALL verify the server certificate against the configured CA and the configured server name, and SHALL fail the connection if verification fails.

#### Scenario: Wrong server name
- **WHEN** the configured server name does not match the certificate
- **THEN** the handshake fails

### Requirement: Mutual TLS is opt-in on the server
When a server config is built with a client CA, the server SHALL require every client to present a certificate chaining to that CA and SHALL reject clients that do not. Without a client CA the server SHALL NOT request a client certificate.

#### Scenario: Missing client certificate
- **WHEN** a client with no identity connects to an mTLS server
- **THEN** the connection is rejected

#### Scenario: Valid client certificate
- **WHEN** a client presents a certificate signed by the server's client CA
- **THEN** the session proceeds and I++ commands are answered

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

### Requirement: Handshakes are bounded
The client's TCP connect plus TLS handshake SHALL together finish within 5 seconds by default, configurable with `with_connect_timeout`, failing with a connect-timeout error. The server SHALL drop a connection whose handshake does not complete within 5 seconds so a silent peer cannot hold a task forever.

#### Scenario: Silent peer on the server
- **WHEN** a peer connects to a TLS server and sends nothing
- **THEN** the server ends that connection after the handshake timeout
