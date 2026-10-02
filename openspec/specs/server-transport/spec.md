# server-transport Specification

## Purpose

The single accept/frame/dispatch loop behind every I++ DME server in this repo (`IppServer` in `ippdme-net`, `server.rs`). The mock server and the imposter are handlers on top of it; no other crate owns a listener or framing code.

## Requirements

### Requirement: One handler per server, one session per connection
`IppServer` SHALL accept TCP connections, frame each with the line codec, and hand every inbound command to a `Handler`. Each connection SHALL get its own `Session` value from `Handler::new_session`, passed mutably to every call on that connection, so handlers can keep per-connection state. Connections SHALL be served concurrently on separate tasks.

#### Scenario: State is per connection
- **WHEN** one client changes session state and a second client connects
- **THEN** the second connection starts from a fresh session

#### Scenario: Concurrent clients
- **WHEN** two clients are connected
- **THEN** each is served independently

### Requirement: Closures are stateless handlers
Any `Fn(Tag, Term) -> Future<Output = Message>` closure SHALL work as a handler with no session state.

#### Scenario: Echo server
- **WHEN** a server is built from a closure that always returns an ack
- **THEN** every command receives an ack

### Requirement: Client-side responses are ignored
Inbound messages that are marked responses rather than commands SHALL be ignored by the server loop.

#### Scenario: Response sent to server
- **WHEN** a client sends `00001 # Ack()`
- **THEN** the handler is not called and the connection stays open

### Requirement: Binding and addresses
`bind` SHALL bind a TCP listener, `local_addr` SHALL report the actual bound address (so port 0 gives an ephemeral port), and `serve` SHALL accept connections until dropped. `serve_connection` SHALL drive one connection over any byte stream until the peer closes it.

#### Scenario: Ephemeral port
- **WHEN** a server binds port 0
- **THEN** `local_addr` returns the port the OS assigned

### Requirement: A failed connection does not stop the server
An error on one connection SHALL be logged and end only that connection; the server SHALL keep accepting others.

#### Scenario: Garbage from one client
- **WHEN** one client sends an unparseable line
- **THEN** that connection ends and other clients are unaffected

### Requirement: A handler chooses how to answer
A handler SHALL answer each command with an action: reply with a message, send a raw text line verbatim (bypassing serialization, for simulating a faulty server), or close the connection without answering.

#### Scenario: Close action
- **WHEN** a handler returns the close action
- **THEN** the client's connection ends without a reply

#### Scenario: Raw line action
- **WHEN** a handler returns a raw line
- **THEN** that text followed by CRLF is written to the client unchanged
