## MODIFIED Requirements
### Requirement: Response kinds
Each response entry SHALL be exactly one of: `ack` (`true` for `Ack()`, or a name such as `Ready` for `Ready()` under the ack marker), `error: Reason` (an `Error(Reason)` reply) or `data` (a data response built from a call shape). An optional `after_ms` SHALL delay the reply to simulate machine latency. An entry that sets none or more than one kind SHALL be rejected when the config is loaded.

#### Scenario: Named ack
- **WHEN** a response is `ack: Ready`
- **THEN** the reply is `00001 # Ready()`

#### Scenario: Delay
- **WHEN** a response has `after_ms: 500`
- **THEN** the reply arrives no sooner than 500 ms after the call

#### Scenario: Ambiguous entry
- **WHEN** a response sets both `ack` and `error`
- **THEN** loading the config fails with an invalid-response error

### Requirement: Received calls can be verified
`Imposter::handle()` SHALL return a cheap, cloneable handle whose `received_calls()` returns every call term received so far in arrival order. Because `serve` consumes the imposter, the handle SHALL be obtained before serving.

#### Scenario: Verify what the client sent
- **WHEN** a client under test sends `Home()` then `PtMeas()`
- **THEN** `received_calls()` returns `Home()` then `PtMeas()`

