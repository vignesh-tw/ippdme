# tui Specification

## Purpose

`ippdme-tui`: a Ratatui terminal UI, "Postman for I++ DME", for manual and exploratory use against a real machine or a built-in mock CMM.

## Requirements

### Requirement: Responsive event loop
The UI SHALL run a single loop over keyboard events, a 100 ms tick and a channel of background results. Network operations (connect, send, start mock, start tap) SHALL be spawned as background tasks that report back through that channel, so a slow or hung network call never freezes keystrokes or redraws.

#### Scenario: Slow reply
- **WHEN** a command is sent and the server takes seconds to reply
- **THEN** the UI keeps redrawing and accepting keys until the reply arrives

### Requirement: Three modes
The UI SHALL have modes `Client` (connect to a host and port, default `127.0.0.1:1294`) and `Mock Server` (bind an embedded `IppMockServer` on `127.0.0.1` at the chosen port and connect a client to it). `m` SHALL toggle between them, and switching while connected or connecting SHALL be refused with the status "Disconnect before switching mode".

#### Scenario: Toggling
- **WHEN** `m` is pressed while disconnected
- **THEN** the mode switches between Client and Mock Server

#### Scenario: Locked while connected
- **WHEN** `m` is pressed while connected
- **THEN** the mode does not change and the status explains why

### Requirement: Connecting and disconnecting
`c` SHALL start connecting in the current mode, and pressing it again while connected SHALL disconnect (dropping the client and stopping any embedded mock). The connection bar SHALL show `CONNECTED` in green, `CONNECTING` in yellow or `DISCONNECTED` in red, plus the mode and target. `h` and `p` SHALL edit the host and port while disconnected. Connect failures SHALL be reported in the status bar.

#### Scenario: Mock server mode
- **WHEN** the mode is Mock Server and `c` is pressed
- **THEN** a mock listens on the chosen port, a client connects to it, and the bar shows `CONNECTED`

#### Scenario: Connect failure
- **WHEN** the target refuses the connection
- **THEN** the status shows `Connect failed: ...` and the state stays disconnected

### Requirement: Preset commands
The left sidebar SHALL list preset commands grouped into categories: Session (`StartSession()`, `EndSession()`, `GetDMEVersion()`), Motion (`Home()`, `GoTo(10, 10, 10)`, `GoTo(0, 0, 0)`), Measurement (`PtMeas()`), Tooling (`SetCoordSystem(MCS|PCS)`) and further presets for the status methods (`IsHomed()`, `IsUserEnabled()`, `EnableUser()`, `DisableUser()`, `GetMachineClass()`, `GetErrStatusE()`, `GetXtdErrStatus()`, `Get(X, Y, Z)`, `ClearAllErrors()`, `AbortE()`, `StopAllDaemons()`). Up and Down SHALL move the selection and Enter SHALL send the selected preset.

#### Scenario: Send a preset
- **WHEN** the `StartSession()` preset is selected and Enter is pressed while connected
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
