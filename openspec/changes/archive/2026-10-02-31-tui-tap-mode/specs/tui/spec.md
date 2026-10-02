## ADDED Requirements
### Requirement: Tap mode
In Tap mode, `c` SHALL start a tap on `127.0.0.1` at the tap port (default 1297, set with `--tap-listen`), forwarding to the host and port in the connection bar. The log SHALL show each line exactly as it crossed the port, prefixed with its connection number, outbound for client-to-server and inbound for server-to-client, plus connection opened/closed notes. Sending from the input box or presets in Tap mode SHALL be refused with a message that Tap mode only watches traffic.

#### Scenario: Watching traffic
- **WHEN** a client connects to the tap and sends `00001 Home()`
- **THEN** the log shows `#1 00001 Home()` as outbound and the reply line as inbound

#### Scenario: Sending in Tap mode
- **WHEN** Enter is pressed on a preset in Tap mode
- **THEN** a denied entry says Tap mode only watches traffic

## MODIFIED Requirements
### Requirement: Three modes
The UI SHALL have modes `Client` (connect to a host and port, default `127.0.0.1:1294`), `Mock Server` (bind an embedded `IppMockServer` on `127.0.0.1` at the chosen port and connect a client to it) and `Tap` (run an `IppTap`). `m` SHALL cycle Client, Mock Server, Tap, back to Client. Switching mode while connected or connecting SHALL be refused with the status "Disconnect before switching mode".

#### Scenario: Cycling
- **WHEN** `m` is pressed three times while disconnected
- **THEN** the mode returns to `Client`

#### Scenario: Locked while connected
- **WHEN** `m` is pressed while connected
- **THEN** the mode does not change and the status explains why

#### Scenario: Toggling
- **WHEN** `m` is pressed while disconnected in Client mode
- **THEN** the mode switches to Mock Server

### Requirement: Connecting and disconnecting
`c` SHALL start connecting in the current mode, and pressing it again while connected SHALL disconnect (dropping the client and stopping any embedded mock or tap). The connection bar SHALL show `CONNECTED` in green, `CONNECTING` in yellow or `DISCONNECTED` in red, plus the mode and target. `h` and `p` SHALL edit the host and port while disconnected. Connect failures SHALL be reported in the status bar.

#### Scenario: Mock server mode
- **WHEN** the mode is Mock Server and `c` is pressed
- **THEN** a mock listens on the chosen port and a client connects to it, and the bar shows `CONNECTED`

#### Scenario: Connect failure
- **WHEN** the target refuses the connection
- **THEN** the status shows `Connect failed: ...` and the state stays disconnected

### Requirement: Command-line options
`--ca-cert PEM` SHALL make client mode connect over TLS 1.3, `--server-name` SHALL set the name the server certificate must match (default the connected host), and `--client-cert` with `--client-key` SHALL present a client certificate. `--tap-listen PORT` SHALL set the tap's listen port. `--server-name`, `--client-cert` or `--client-key` without `--ca-cert`, only one of the certificate pair, an unknown flag, a flag without a value or a non-numeric port SHALL be rejected with an error. Mock-server mode SHALL always be plain TCP.

#### Scenario: No flags
- **WHEN** the TUI starts with no arguments
- **THEN** it uses plain TCP and tap port 1297

#### Scenario: Half an identity
- **WHEN** `--client-cert c.pem` is given without `--client-key`
- **THEN** startup fails with an error

