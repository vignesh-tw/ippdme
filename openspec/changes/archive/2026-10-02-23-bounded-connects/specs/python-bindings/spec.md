## MODIFIED Requirements
### Requirement: Client TLS options
Passing `ca_cert` (PEM path) SHALL switch the connection to TLS 1.3. `server_name` SHALL default to the host part of the address, with IPv6 brackets stripped. `client_cert` and `client_key` SHALL be given together to present a client certificate. Supplying `server_name`, `client_cert` or `client_key` without `ca_cert`, or only one of the certificate pair, SHALL raise `ValueError`. `connect_timeout` (seconds, default 5) SHALL bound the TCP connect plus handshake and SHALL raise `ValueError` if negative or not a number.

#### Scenario: Half an identity
- **WHEN** `client_cert` is given without `client_key`
- **THEN** `ValueError` is raised

#### Scenario: Options without TLS
- **WHEN** `server_name` is given without `ca_cert`
- **THEN** `ValueError` is raised

### Requirement: Python exceptions map from network errors
Timeouts (request and connect) SHALL raise `TimeoutError`. A closed connection, I/O error or shutdown SHALL raise `ConnectionError`. Invalid arguments SHALL raise `ValueError`. TLS and other protocol errors SHALL raise `ConnectionError`.

#### Scenario: Connection refused
- **WHEN** `IppClient.connect` targets a closed port
- **THEN** `ConnectionError` is raised

