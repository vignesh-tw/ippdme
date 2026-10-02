## RENAMED Requirements
- FROM: `### Requirement: One handler per server`
- TO: `### Requirement: One handler per server, one session per connection`

## ADDED Requirements
### Requirement: A handler chooses how to answer
A handler SHALL answer each command with an action: reply with a message, send a raw text line verbatim (bypassing serialization, for simulating a faulty server), or close the connection without answering.

#### Scenario: Close action
- **WHEN** a handler returns the close action
- **THEN** the client's connection ends without a reply

#### Scenario: Raw line action
- **WHEN** a handler returns a raw line
- **THEN** that text followed by CRLF is written to the client unchanged

## MODIFIED Requirements
### Requirement: One handler per server, one session per connection
`IppServer` SHALL accept TCP connections, frame each with the line codec, and hand every inbound command to a `Handler`. Each connection SHALL get its own `Session` value from `Handler::new_session`, passed mutably to every call on that connection, so handlers can keep per-connection state. Connections SHALL be served concurrently on separate tasks.

#### Scenario: State is per connection
- **WHEN** one client changes session state and a second client connects
- **THEN** the second connection starts from a fresh session

#### Scenario: Concurrent clients
- **WHEN** two clients are connected
- **THEN** each is served independently

