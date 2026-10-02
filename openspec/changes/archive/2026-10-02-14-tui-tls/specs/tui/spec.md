## ADDED Requirements
### Requirement: Command-line options
`--ca-cert PEM` SHALL make client mode connect over TLS 1.3, `--server-name` SHALL set the name the server certificate must match (default the connected host), and `--client-cert` with `--client-key` SHALL present a client certificate. `--server-name`, `--client-cert` or `--client-key` without `--ca-cert`, only one of the certificate pair, an unknown flag or a flag without a value SHALL be rejected with an error. Mock-server mode SHALL always be plain TCP.

#### Scenario: No flags
- **WHEN** the TUI starts with no arguments
- **THEN** it uses plain TCP

#### Scenario: Half an identity
- **WHEN** `--client-cert c.pem` is given without `--client-key`
- **THEN** startup fails with an error

