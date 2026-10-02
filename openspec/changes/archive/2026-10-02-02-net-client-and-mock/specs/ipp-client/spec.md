## ADDED Requirements
### Requirement: Connecting over TCP
`IppClient::connect(addr)` SHALL open a TCP connection and spawn background read and write tasks. `IppClient::from_stream` SHALL wrap an already-connected `TcpStream`.

#### Scenario: Connected
- **WHEN** a mock server is listening
- **THEN** `connect` returns a client ready to send

### Requirement: Tags are allocated automatically and correlate replies
The client SHALL allocate outbound tags from a counter starting at 1, wrapping at the protocol maximum. Each sent command SHALL be matched with the response carrying the same tag, so concurrent requests may be in flight on one connection and each caller receives its own reply.

#### Scenario: Concurrent requests
- **WHEN** two commands are sent without waiting for the first reply
- **THEN** each call returns the response whose tag matches its own command

### Requirement: Requests time out
Waiting for a reply SHALL time out after 5 seconds by default, configurable with `set_default_timeout`, and SHALL fail with a timeout error naming the tag. A timed-out request SHALL be removed from the pending set.

#### Scenario: Silent server
- **WHEN** the server never answers a command
- **THEN** the call fails with a timeout error for that tag after the timeout

### Requirement: Every inbound message is observable
`subscribe()` SHALL return a broadcast receiver of every inbound message, responses and unsolicited events alike, independent of request/response pairing.

#### Scenario: Subscriber sees replies
- **WHEN** a subscriber exists and a command is answered
- **THEN** the subscriber receives the reply message

### Requirement: Helpers over common commands
The client SHALL provide helpers for `start_session`, `end_session`, `get_dme_version`, `home`, `go_to`, `pt_meas` and `set_coord_system`, and `send_command` for any typed command. Each SHALL return the raw reply message, whatever its marker.

#### Scenario: Error reply is a value
- **WHEN** a helper is answered with `Error(UnknownCommand)`
- **THEN** the helper returns that reply message rather than failing

