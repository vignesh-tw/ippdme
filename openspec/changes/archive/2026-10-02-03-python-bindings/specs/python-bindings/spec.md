## ADDED Requirements
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
`IppClient.connect(addr)` SHALL return a connected client. Its methods `start_session`, `end_session`, `get_dme_version`, `home`, `go_to(x, y, z)`, `pt_meas`, `set_coord_system("MCS"|"PCS")` and `send_raw(term)` SHALL return a `Response` for whatever the server answered (ack, data or error), so tests can assert on error replies without exceptions. `set_coord_system` SHALL accept the name case-insensitively and raise `ValueError` for anything else. `send_raw` SHALL parse a bare term string and raise `ValueError` if it does not parse.

#### Scenario: Error reply is a value
- **WHEN** an unknown command is sent with `send_raw`
- **THEN** the call returns a `Response` with `is_error()` true and a `reason`

#### Scenario: Bad coordinate system
- **WHEN** `set_coord_system("XYZ")` is called
- **THEN** `ValueError` is raised

### Requirement: Response accessors
A `Response` SHALL provide `is_ack()`, `is_error()`, `is_data()`, a `tag`, the top-level term `name`, `get(param)` for a numeric parameter (such as `X` in `PtMeas`), `get_ident(param)` for a nested identifier parameter, and `reason` for the first bare identifier argument (such as `UnknownCommand` in `Error(UnknownCommand)`). Missing values SHALL be `None`. `str()` SHALL give the wire text and `repr()` SHALL be `Response(<wire text>)`.

#### Scenario: Reading a measurement
- **WHEN** `pt_meas()` returns `PtMeas(X(10.002), ...)`
- **THEN** `response.get("X")` is 10.002 and `response.name` is `"PtMeas"`

### Requirement: Python exceptions map from network errors
A request timeout SHALL raise `TimeoutError`. A closed connection, I/O error, shutdown or protocol error SHALL raise `ConnectionError`.

#### Scenario: Connection refused
- **WHEN** `IppClient.connect` targets a closed port
- **THEN** `ConnectionError` is raised

### Requirement: Embedded mock server
`IppMockServer(port=1294)` SHALL wrap the mock server. `start_in_background()` SHALL bind `127.0.0.1` and return immediately, doing nothing if already started. `port` SHALL report the real bound port, resolving port 0 to the ephemeral port once started. `stop()` SHALL abort the server, and dropping the object SHALL stop it.

#### Scenario: Ephemeral port
- **WHEN** `IppMockServer(port=0)` is started
- **THEN** `server.port` is the non-zero port actually bound

### Requirement: Pytest fixtures
`ippdme.testing` SHALL provide a `mock_server` fixture (an embedded mock on an ephemeral port, stopped at teardown) and an `ipp_client` fixture (connected to `mock_server` with a session already started and ended at teardown), enabled via `pytest_plugins = ["ippdme.testing"]`.

#### Scenario: Using the fixture
- **WHEN** a test takes `ipp_client` and calls `go_to(x=10.0, y=20.0, z=5.0)`
- **THEN** the response `is_ack()`

