## ADDED Requirements
### Requirement: Bytes are forwarded unchanged
A tap SHALL listen on one address, open a connection to the target for each accepted client, and copy bytes unchanged in both directions, so malformed lines and unusual framing reach their destination exactly as sent. When either side closes, the tap SHALL close the other side so the peer sees the end of the stream.

#### Scenario: Round trip through the tap
- **WHEN** a client connects to the tap and sends `00001 StartSession()` while the target is a mock server
- **THEN** the client receives the mock's reply unchanged

#### Scenario: Malformed line passes through
- **WHEN** a client sends a line that does not parse
- **THEN** the target receives exactly those bytes

### Requirement: Every line is reported per direction
The tap SHALL broadcast events: `Opened` (connection id and peer) once the target is reached, `Line` (connection id, direction `ClientToServer` or `ServerToClient`, and the text without its CR/LF, decoded lossily as UTF-8), and `Closed`. Connections SHALL be numbered from 1. A final line with no terminator SHALL still be reported when the stream ends.

#### Scenario: Both directions
- **WHEN** a client sends `00001 Home()` and receives `00001 # Ack()`
- **THEN** a client-to-server line `00001 Home()` and a server-to-client line `00001 # Ack()` are reported for the same connection id

#### Scenario: Unterminated last line
- **WHEN** a client sends `00002 Hom` and disconnects
- **THEN** `00002 Hom` is reported as a line before `Closed`

### Requirement: An unreachable target is reported, not fatal
If the target cannot be reached, the tap SHALL log the failure, emit `Closed` for that connection, and keep accepting others.

#### Scenario: Target down
- **WHEN** a client connects while the target is not listening
- **THEN** the client connection is closed and the tap continues to accept new clients

### Requirement: Subscribers do not slow traffic
`subscribe()` SHALL be called before `serve` consumes the tap. A slow subscriber MAY miss events but SHALL NOT delay forwarding.

#### Scenario: Lagging subscriber
- **WHEN** a subscriber stops reading
- **THEN** forwarded traffic is unaffected

### Requirement: Plain TCP only
The tap SHALL NOT decrypt TLS. A TLS connection MAY pass through but the reported lines are encrypted bytes and not readable.

#### Scenario: TLS through the tap
- **WHEN** a TLS client connects through the tap
- **THEN** the handshake still succeeds end to end and the reported lines are not readable I++ text

