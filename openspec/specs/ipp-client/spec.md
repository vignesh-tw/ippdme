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

### Requirement: Tags can be allocated before sending
`allocate_tag()` SHALL reserve the next tag without sending anything, and `send_with_tag(tag, term)` SHALL send using a reserved tag. `send` SHALL be exactly an allocation followed by `send_with_tag`. This lets a caller (such as the TUI) log the outbound message with its real tag before awaiting the reply.

#### Scenario: Log before reply
- **WHEN** a caller allocates a tag, records it, then calls `send_with_tag`
- **THEN** the reply carries the recorded tag

### Requirement: Typed helpers over common commands
The client SHALL provide helpers for `start_session`, `end_session`, `get_dme_version`, `home`, `go_to`, `pt_meas`, `set_coord_system`, `is_homed` and `is_user_enabled`. Helpers returning nothing SHALL require an ack-marker reply. Helpers returning a value SHALL parse the data reply into a typed value (version string, point, flag). A server `Error(...)` reply SHALL surface as a protocol error carrying the server's reason. `send_command` SHALL return the raw reply message for callers who want to inspect errors themselves.

#### Scenario: Server error from a helper
- **WHEN** `go_to` is answered with `Error(NotHomed)`
- **THEN** the helper fails with a server error whose reason is `NotHomed`

#### Scenario: Typed value
- **WHEN** `pt_meas` is answered with `PtMeas(X(1), Y(2), Z(3), ...)`
- **THEN** it returns the point (1, 2, 3)
