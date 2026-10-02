## MODIFIED Requirements
### Requirement: Client methods return raw responses
`IppClient.connect(addr, ...)` SHALL return a connected client. Its methods `start_session`, `end_session`, `get_dme_version`, `home`, `go_to(x, y, z)`, `pt_meas`, `set_coord_system("MCS"|"PCS")` and `send_raw(term)` SHALL return a `Response` for whatever the server answered (ack, data or error), so tests can assert on error replies without exceptions. `set_coord_system` SHALL accept the name case-insensitively and raise `ValueError` for anything else. `send_raw` SHALL parse a bare term string and raise `ValueError` if it does not parse. `go_to` SHALL raise `ValueError` for non-finite values.

#### Scenario: Error reply is a value
- **WHEN** a strict mock server is asked to `go_to` before homing
- **THEN** the call returns a `Response` with `is_error()` true and `reason == "NotHomed"`

#### Scenario: Bad coordinate system
- **WHEN** `set_coord_system("XYZ")` is called
- **THEN** `ValueError` is raised

#### Scenario: Raw term
- **WHEN** `send_raw("GetMachineClass()")` is called
- **THEN** the reply is returned as a `Response`

### Requirement: Python exceptions map from network errors
A request timeout SHALL raise `TimeoutError`. Invalid arguments SHALL raise `ValueError`. A closed connection, I/O error, shutdown, TLS error or other protocol error SHALL raise `ConnectionError`.

#### Scenario: Connection refused
- **WHEN** `IppClient.connect` targets a closed port
- **THEN** `ConnectionError` is raised

