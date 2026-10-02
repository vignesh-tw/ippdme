# server-transport Specification

## Purpose

The single accept/frame/dispatch loop behind the I++ DME servers in this repo (`IppServer` in `ippdme-net`, `server.rs`). The mock server and the imposter are handlers on top of it.

## Requirements

### Requirement: One handler per server
`IppServer` SHALL accept TCP connections, frame each with the line codec, and hand every inbound command to a `Handler`, which returns exactly one response message. Connections SHALL be served concurrently on separate tasks.

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
