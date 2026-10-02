## MODIFIED Requirements
### Requirement: Embedded mock server
`IppMockServer(port=1294, *, cert=None, key=None, client_ca=None, latency_ms=500, strict=False)` SHALL wrap the mock server. `start_in_background()` SHALL bind `127.0.0.1` and return immediately, doing nothing if already started. `port` SHALL report the real bound port, resolving port 0 to the ephemeral port once started. `stop()` SHALL abort the server, and dropping the object SHALL stop it. `cert` and `key` SHALL be given together and enable TLS, `client_ca` requires them and enables mutual TLS, and any other combination SHALL raise `ValueError`.

#### Scenario: Ephemeral port
- **WHEN** `IppMockServer(port=0)` is started
- **THEN** `server.port` is the non-zero port actually bound

#### Scenario: client_ca alone
- **WHEN** `IppMockServer(client_ca="ca.pem")` is created
- **THEN** `ValueError` is raised

