## MODIFIED Requirements
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

