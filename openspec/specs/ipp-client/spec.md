# ipp-client Specification

## Purpose

The async Tokio client for talking to an I++ DME server: tag allocation, request/response correlation, timeouts, a stream of inbound messages, and typed helpers. Implemented as `IppClient` in `ippdme-net` (`client.rs`).

## Requirements

### Requirement: Connecting over TCP
`IppClient::connect(addr)` SHALL open a TCP connection and spawn background read and write tasks. `IppClient::from_stream` SHALL wrap any already-connected `AsyncRead + AsyncWrite` stream (TCP, TLS, in-memory pipe).

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
The client SHALL provide helpers for `start_session`, `end_session`, `get_dme_version`, `home`, `go_to`, `pt_meas` and `set_coord_system`, and `send_command` for any typed command. Each SHALL return the raw reply message, whatever its marker. A helper given invalid arguments (such as a non-finite coordinate) SHALL fail with an invalid-argument error without sending anything.

#### Scenario: Error reply is a value
- **WHEN** a helper is answered with `Error(UnknownCommand)`
- **THEN** the helper returns that reply message rather than failing

#### Scenario: Invalid argument
- **WHEN** `go_to` is called with a NaN coordinate
- **THEN** it fails with an invalid-argument error and nothing is sent

### Requirement: Tags can be allocated before sending
`allocate_tag()` SHALL reserve the next tag without sending anything, and `send_with_tag(tag, term)` SHALL send using a reserved tag. `send` SHALL be exactly an allocation followed by `send_with_tag`. This lets a caller (such as the TUI) log the outbound message with its real tag before awaiting the reply.

#### Scenario: Log before reply
- **WHEN** a caller allocates a tag, records it, then calls `send_with_tag`
- **THEN** the reply carries the recorded tag
