# tui Specification

## Purpose

`ippdme-tui`: a Ratatui terminal UI, "Postman for I++ DME", for manual and exploratory use. It can act as a client to a real machine, host its own mock CMM, or tap the traffic on a port. Network work is never awaited inside the event loop (see the architecture notes in `CLAUDE.md`).

## Requirements

### Requirement: Responsive event loop
The UI SHALL run a single loop over keyboard events, a 100 ms tick and a channel of background results. Network operations (connect, send, start mock, start tap) SHALL be spawned as background tasks that report back through that channel, so a slow or hung network call never freezes keystrokes or redraws.

#### Scenario: Slow reply
- **WHEN** a command is sent and the server takes seconds to reply
- **THEN** the UI keeps redrawing and accepting keys until the reply arrives

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

### Requirement: Preset commands
The left sidebar SHALL list preset commands grouped into categories (Session, Motion, Measurement, Tooling, and others), including `StartSession()`, `EndSession()`, `GetDMEVersion()`, `Home()`, `GoTo(10, 10, 10)`, `GoTo(0, 0, 0)`, `PtMeas()`, `SetCoordSystem(MCS|PCS)`, `EnumTools()`, `Tool()`, `ChangeTool("RefTool")`, `GetCoordSystem()`, `GetCsyTransformation(PartCsy)` and `EnumCoordSystems()`. Up and Down SHALL move the selection (clamped at the ends) and Enter SHALL send the selected preset.

#### Scenario: Send a preset
- **WHEN** the `Session → StartSession()` preset is selected and Enter pressed while connected
- **THEN** `StartSession()` is sent and the reply is logged

### Requirement: Typed command input
With the input box focused (Tab cycles Sidebar, Log, Input) and raw line mode off, Enter SHALL parse the text as an I++ term, add the next tag, and send it. A parse error SHALL be logged and shown in the status bar without sending anything.

#### Scenario: Typed term
- **WHEN** `SetCoordSystem(PCS)` is entered and submitted
- **THEN** it is sent with the next tag and the box is cleared

#### Scenario: Parse error
- **WHEN** `GoTo(` is submitted
- **THEN** a `Parse error: ...` entry appears in the log and nothing is sent

### Requirement: Live protocol log
Every outbound command SHALL be logged with its real tag before its reply arrives (using `allocate_tag` and `send_with_tag`). Replies SHALL be logged colour-coded: green for `#` acks, yellow for `%` data, red for `!` errors, blue for outbound, with the round-trip latency. Failures that occur locally (not connected, still connecting, parse error) and network errors SHALL be logged as red entries instead of silently disappearing.

#### Scenario: Not connected
- **WHEN** a preset is sent while disconnected
- **THEN** a red `<Not connected — command not sent>` entry is logged

#### Scenario: Timeout
- **WHEN** a request times out
- **THEN** a red entry with the timeout error and elapsed latency is logged

### Requirement: Session export
`e` SHALL write the session log to `ippdme-session-<unix-ms>.log` (one line per entry with timestamp, direction, tag, marker, text and latency) and `ippdme-session-<unix-ms>.json` (the entries as pretty JSON) in the current directory, and report the file names or the failure in the status bar.

#### Scenario: Export
- **WHEN** `e` is pressed after some traffic
- **THEN** both files are written and the status names them

### Requirement: Quitting
`q` or Esc outside the input box SHALL quit and restore the terminal. In the input box Esc SHALL return focus to the sidebar.

#### Scenario: Quit
- **WHEN** `q` is pressed with the sidebar focused
- **THEN** the UI exits and the terminal is restored

### Requirement: Command-line options
`--ca-cert PEM` SHALL make client mode connect over TLS 1.3, `--server-name` SHALL set the name the server certificate must match (default the connected host), and `--client-cert` with `--client-key` SHALL present a client certificate. `--tap-listen PORT` SHALL set the tap's listen port. `--server-name`, `--client-cert` or `--client-key` without `--ca-cert`, only one of the certificate pair, an unknown flag, a flag without a value or a non-numeric port SHALL be rejected with an error. Mock-server mode SHALL always be plain TCP.

#### Scenario: No flags
- **WHEN** the TUI starts with no arguments
- **THEN** it uses plain TCP and tap port 1297

#### Scenario: Half an identity
- **WHEN** `--client-cert c.pem` is given without `--client-key`
- **THEN** startup fails with an error

### Requirement: Raw line input
`r` SHALL toggle raw line mode. In raw line mode, Enter SHALL send the whole input exactly as typed, tag included and unparsed, so it can be malformed on purpose, like typing into `nc`. If the line starts with a tag the reply SHALL be awaited and logged; without a tag the status SHALL say a line was sent without waiting for a reply.

#### Scenario: Raw tagged line
- **WHEN** raw mode is on and `00042 Home()` is submitted
- **THEN** that exact line is sent and the reply tagged `00042` is logged

#### Scenario: Raw untagged line
- **WHEN** raw mode is on and `garbage` is submitted
- **THEN** it is sent and the status reports that no reply is awaited

### Requirement: Tap mode
In Tap mode, `c` SHALL start a tap on `127.0.0.1` at the tap port (default 1297, set with `--tap-listen`), forwarding to the host and port in the connection bar. The log SHALL show each line exactly as it crossed the port, prefixed with its connection number, outbound for client-to-server and inbound for server-to-client, plus connection opened/closed notes. Sending from the input box or presets in Tap mode SHALL be refused with a message that Tap mode only watches traffic.

#### Scenario: Watching traffic
- **WHEN** a client connects to the tap and sends `00001 Home()`
- **THEN** the log shows `#1 00001 Home()` as outbound and the reply line as inbound

#### Scenario: Sending in Tap mode
- **WHEN** Enter is pressed on a preset in Tap mode
- **THEN** a denied entry says Tap mode only watches traffic
