## ADDED Requirements
### Requirement: TLS from YAML or builder
A `tls:` block with `cert`, `key` and optional `client_ca` (PEM file paths) SHALL make the imposter serve TLS 1.3, and setting `client_ca` SHALL require client certificates. Missing or invalid files SHALL fail at load time.

#### Scenario: mTLS imposter
- **WHEN** the YAML has `tls.client_ca` and a client connects without a certificate
- **THEN** the connection is rejected

## MODIFIED Requirements
### Requirement: YAML schema for hand-editing
The YAML file SHALL have a top-level `port`, an optional `tls` block, and a list of `stubs`, each with a `predicate` (`call`, optional `args`) and `responses`. Arguments and data SHALL be writable as a bare number, a bare identifier, `{str: "text"}` for a quoted string, `{name: X, value: 10.002}` for a named parameter such as `X(10.002)`, or a general nested `{call: Name, args: [...]}`. The YAML-facing types SHALL be separate from the core `Term` so the AST's internal tagging does not leak into hand-edited files, and `ippdme-core` SHALL NOT depend on serde.

#### Scenario: Point data
- **WHEN** a data response lists args `{name: X, value: 10.002}` for call `PtMeas`
- **THEN** the reply term is `PtMeas(X(10.002), ...)`

#### Scenario: Invalid YAML
- **WHEN** a config file is not valid YAML or has the wrong shape
- **THEN** loading fails with a YAML error naming the problem

### Requirement: Rust builder API
The library SHALL allow building an imposter in code: `Imposter::builder().stub(Stub::when(predicate).responds_with(response)...).bind(addr)`, optionally with `.tls(...)`. It SHALL also load from `from_yaml_file` and `from_yaml_str`. YAML-loaded imposters SHALL bind `127.0.0.1` on the configured port.

#### Scenario: Inline test imposter
- **WHEN** a test builds an imposter on `("127.0.0.1", 0)` with one stub
- **THEN** `local_addr` gives the port a client can connect to

