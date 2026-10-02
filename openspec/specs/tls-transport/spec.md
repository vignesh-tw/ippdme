# tls-transport Specification

## Purpose

Optional TLS 1.3 (and mutual TLS) around the I++ DME protocol, implemented in `ippdme-net` behind the `tls` feature (`tls.rs`, rustls + ring).

## Requirements

### Requirement: TLS 1.3 only, behind a feature flag
TLS support SHALL be gated by the `tls` Cargo feature of `ippdme-net` and SHALL offer only TLS 1.3. There SHALL be no fallback to plaintext.

#### Scenario: Plaintext peer
- **WHEN** a TLS client connects to a server that does not speak TLS
- **THEN** the connect fails and no I++ traffic is sent in the clear

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
`IppServer::bind_tls` and `IppMockServer::bind_tls` (and the mock builder's `tls`) SHALL upgrade every accepted connection to TLS before reading any I++ traffic.

#### Scenario: Mock over TLS
- **WHEN** a mock server is bound with a TLS config and a TLS client connects with the matching CA
- **THEN** `StartSession` is answered normally

### Requirement: Surfaces expose the same options
Python SHALL expose TLS as `IppClient.connect(addr, ca_cert=, server_name=, client_cert=, client_key=)` and `IppMockServer(port, cert=, key=, client_ca=)`. Details are in the `python-bindings` spec.

#### Scenario: Round trip across surfaces
- **WHEN** a TLS mock is started from Python and a Python client connects with the CA
- **THEN** commands are answered
