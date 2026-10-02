# python-bindings Specification

## Purpose

The `ippdme` Python package (PyO3 crate `ippdme-py`, imported as `ippdme._ippdme`) for QA/CI pipelines and data scientists: a synchronous client, an embeddable mock server, a response wrapper and pytest fixtures. Built with Maturin.

## Requirements

### Requirement: Synchronous API on a shared runtime
All Python methods SHALL be synchronous. They SHALL run async work on a single shared Tokio runtime and SHALL release the GIL while blocking, so other Python threads keep running.

#### Scenario: Blocking call
- **WHEN** `client.go_to(...)` is called
- **THEN** it returns only after the reply arrives or the request times out

### Requirement: Public surface
The package SHALL export `IppClient`, `IppMockServer` and `Response` from `ippdme`, and SHALL ship a `py.typed` marker.

#### Scenario: Import
- **WHEN** `from ippdme import IppClient, IppMockServer, Response` is run
- **THEN** all three names import

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

### Requirement: Response accessors
A `Response` SHALL provide `is_ack()`, `is_error()`, `is_data()`, a `tag`, the top-level term `name`, `get(param)` for a numeric parameter (such as `X` in `PtMeas`), `get_ident(param)` for a nested identifier parameter, and `reason` for the first bare identifier argument (such as `UnknownCommand` in `Error(UnknownCommand)`). Missing values SHALL be `None`. `str()` SHALL give the wire text and `repr()` SHALL be `Response(<wire text>)`.

#### Scenario: Reading a measurement
- **WHEN** `pt_meas()` returns `PtMeas(X(10.002), ...)`
- **THEN** `response.get("X")` is 10.002 and `response.name` is `"PtMeas"`

### Requirement: Python exceptions map from network errors
Timeouts (request and connect) SHALL raise `TimeoutError`. A closed connection, I/O error or shutdown SHALL raise `ConnectionError`. Invalid arguments SHALL raise `ValueError`. TLS and other protocol errors SHALL raise `ConnectionError`.

#### Scenario: Connection refused
- **WHEN** `IppClient.connect` targets a closed port
- **THEN** `ConnectionError` is raised

### Requirement: Embedded mock server
`IppMockServer(port=1294, *, cert=None, key=None, client_ca=None, latency_ms=500, strict=False)` SHALL wrap the mock server. `start_in_background()` SHALL bind `127.0.0.1` and return immediately, doing nothing if already started. `port` SHALL report the real bound port, resolving port 0 to the ephemeral port once started. `stop()` SHALL abort the server, and dropping the object SHALL stop it. `cert` and `key` SHALL be given together and enable TLS, `client_ca` requires them and enables mutual TLS, and any other combination SHALL raise `ValueError`.

#### Scenario: Ephemeral port
- **WHEN** `IppMockServer(port=0)` is started
- **THEN** `server.port` is the non-zero port actually bound

#### Scenario: client_ca alone
- **WHEN** `IppMockServer(client_ca="ca.pem")` is created
- **THEN** `ValueError` is raised

### Requirement: Pytest fixtures
`ippdme.testing` SHALL provide a `mock_server` fixture (an embedded mock on an ephemeral port, stopped at teardown) and an `ipp_client` fixture (connected to `mock_server` with a session already started and ended at teardown), enabled via `pytest_plugins = ["ippdme.testing"]`.

#### Scenario: Using the fixture
- **WHEN** a test takes `ipp_client` and calls `go_to(x=10.0, y=20.0, z=5.0)`
- **THEN** the response `is_ack()`

### Requirement: Client TLS options
Passing `ca_cert` (PEM path) SHALL switch the connection to TLS 1.3. `server_name` SHALL default to the host part of the address, with IPv6 brackets stripped. `client_cert` and `client_key` SHALL be given together to present a client certificate. Supplying `server_name`, `client_cert` or `client_key` without `ca_cert`, or only one of the certificate pair, SHALL raise `ValueError`. `connect_timeout` (seconds, default 5) SHALL bound the TCP connect plus handshake and SHALL raise `ValueError` if negative or not a number.

#### Scenario: Half an identity
- **WHEN** `client_cert` is given without `client_key`
- **THEN** `ValueError` is raised

#### Scenario: Options without TLS
- **WHEN** `server_name` is given without `ca_cert`
- **THEN** `ValueError` is raised
