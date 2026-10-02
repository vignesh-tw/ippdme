## ADDED Requirements
### Requirement: Client TLS options
Passing `ca_cert` (PEM path) SHALL switch the connection to TLS 1.3. `server_name` SHALL default to the host part of the address, with IPv6 brackets stripped. `client_cert` and `client_key` SHALL be given together to present a client certificate. Supplying `server_name`, `client_cert` or `client_key` without `ca_cert`, or only one of the certificate pair, SHALL raise `ValueError`.

#### Scenario: Half an identity
- **WHEN** `client_cert` is given without `client_key`
- **THEN** `ValueError` is raised

#### Scenario: Options without TLS
- **WHEN** `server_name` is given without `ca_cert`
- **THEN** `ValueError` is raised

## MODIFIED Requirements
### Requirement: Python exceptions map from network errors
A request timeout SHALL raise `TimeoutError`. A closed connection, I/O error, shutdown, TLS error or protocol error SHALL raise `ConnectionError`.

#### Scenario: Connection refused
- **WHEN** `IppClient.connect` targets a closed port
- **THEN** `ConnectionError` is raised

### Requirement: Embedded mock server
`IppMockServer(port=1294, *, cert=None, key=None, client_ca=None)` SHALL wrap the mock server. `start_in_background()` SHALL bind `127.0.0.1` and return immediately, doing nothing if already started. `port` SHALL report the real bound port, resolving port 0 to the ephemeral port once started. `stop()` SHALL abort the server, and dropping the object SHALL stop it. `cert` and `key` SHALL be given together and enable TLS, `client_ca` requires them and enables mutual TLS, and any other combination SHALL raise `ValueError`.

#### Scenario: Ephemeral port
- **WHEN** `IppMockServer(port=0)` is started
- **THEN** `server.port` is the non-zero port actually bound

#### Scenario: client_ca alone
- **WHEN** `IppMockServer(client_ca="ca.pem")` is created
- **THEN** `ValueError` is raised

