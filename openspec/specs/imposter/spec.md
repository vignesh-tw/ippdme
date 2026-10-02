# imposter Specification

## Purpose

A Mountebank-style stub server for headless, automated testing (`ippdme-imposter`). A stub pairs a predicate with a sequence of responses; the imposter replays them to any I++ DME client. It can be configured from YAML (no Rust needed) or from a Rust builder, and both build the same runtime `Stub`/`Predicate`/`ResponseSpec` types so there is one schema. It runs as the `ippdme-imposter` binary or as a library.

## Requirements

### Requirement: Calls are matched against stubs in order
For each inbound command the imposter SHALL log the call term, then use the first stub whose predicate matches. A predicate SHALL match on the call name and, if `args` is given, only when the call's arguments equal them exactly. Without `args` it SHALL match any arguments for that name. If no stub matches, the imposter SHALL reply `Error(UnknownStub)`.

#### Scenario: Name-only predicate
- **WHEN** a stub matches call `PtMeas` with no args and the client sends `PtMeas(X(1.0))`
- **THEN** the stub's response is sent

#### Scenario: Exact-args predicate
- **WHEN** a stub matches `GetErrorInfo` with args `[42]` and the client sends `GetErrorInfo(7)`
- **THEN** the stub does not match

#### Scenario: No stub
- **WHEN** the client sends `Bogus()` and no stub matches
- **THEN** the reply is `Error(UnknownStub)`

### Requirement: Response sequences cycle and stick on the last entry
A stub's `responses` list SHALL be played in order, one entry per match. Once exhausted, the last response SHALL repeat for every later match. A stub SHALL have at least one response. This supports "fails once, then succeeds" scenarios.

#### Scenario: Fail once then succeed
- **WHEN** a stub lists `error: Jammed` followed by `ack: true` and the client sends the call three times
- **THEN** the replies are the error, then an ack, then an ack again

### Requirement: Response kinds
Each response entry SHALL be exactly one of: `ack` (`true` for `Ack()`, or a name such as `Ready` for `Ready()` under the ack marker), `error: Reason` (an `Error(Reason)` reply), `data` (a data response built from a call shape), `drop: true` (close the connection without replying), or `malformed: "text"` (send that text as the reply line verbatim). An entry that sets none or more than one SHALL be rejected when the config is loaded. An optional `after_ms` SHALL delay the reply to simulate machine latency.

#### Scenario: Named ack
- **WHEN** a response is `ack: Ready`
- **THEN** the reply is `00001 # Ready()`

#### Scenario: Drop
- **WHEN** a response is `drop: true`
- **THEN** the connection closes with no reply

#### Scenario: Malformed
- **WHEN** a response is `malformed: "garbage"`
- **THEN** the client receives the line `garbage`

#### Scenario: Delay
- **WHEN** a response has `after_ms: 500`
- **THEN** the reply arrives no sooner than 500 ms after the call

#### Scenario: Ambiguous entry
- **WHEN** a response sets both `ack` and `error`
- **THEN** loading the config fails with an invalid-response error

### Requirement: YAML schema for hand-editing
The YAML file SHALL have a top-level `port`, an optional `tls` block, and a list of `stubs`, each with a `predicate` (`call`, optional `args`) and `responses`. Arguments and data SHALL be writable as a bare number, a bare identifier, `{str: "text"}` for a quoted string, `{name: X, value: 10.002}` for a named parameter such as `X(10.002)`, or a general nested `{call: Name, args: [...]}`. The YAML-facing types SHALL be separate from the core `Term` so the AST's internal tagging does not leak into hand-edited files, and `ippdme-core` SHALL NOT depend on serde.

#### Scenario: Point data
- **WHEN** a data response lists args `{name: X, value: 10.002}` for call `PtMeas`
- **THEN** the reply term is `PtMeas(X(10.002), ...)`

#### Scenario: Invalid YAML
- **WHEN** a config file is not valid YAML or has the wrong shape
- **THEN** loading fails with a YAML error naming the problem

### Requirement: Rust builder API
The library SHALL allow building an imposter in code: `Imposter::builder().stub(Stub::when(predicate).responds_with(response)...).bind(addr)`, optionally with `.tls(...)`. It SHALL also load from `from_yaml_file` and `from_yaml_str`. YAML-loaded imposters SHALL bind `127.0.0.1` on the configured port.

#### Scenario: Inline test imposter
- **WHEN** a test builds an imposter on `("127.0.0.1", 0)` with one stub
- **THEN** `local_addr` gives the port a client can connect to

### Requirement: Received calls can be verified
`Imposter::handle()` SHALL return a cheap, cloneable handle whose `received_calls()` returns every call term received so far in arrival order. Because `serve` consumes the imposter, the handle SHALL be obtained before serving.

#### Scenario: Verify what the client sent
- **WHEN** a client under test sends `Home()` then `PtMeas()`
- **THEN** `received_calls()` returns `Home()` then `PtMeas()`

### Requirement: Standalone binary
`ippdme-imposter <path-to-imposter.yaml>` SHALL load the file, print `ippdme-imposter listening on ADDR (PATH)`, and serve until killed. With no argument it SHALL print usage and exit 2. If the file fails to load it SHALL print the error and exit 1; if serving stops with an error it SHALL print it and exit 1.

#### Scenario: No argument
- **WHEN** the binary is run with no arguments
- **THEN** it prints `usage: ippdme-imposter <path-to-imposter.yaml>` and exits with status 2

### Requirement: TLS from YAML or builder
A `tls:` block with `cert`, `key` and optional `client_ca` (PEM file paths) SHALL make the imposter serve TLS 1.3, and setting `client_ca` SHALL require client certificates. Missing or invalid files SHALL fail at load time.

#### Scenario: mTLS imposter
- **WHEN** the YAML has `tls.client_ca` and a client connects without a certificate
- **THEN** the connection is rejected
