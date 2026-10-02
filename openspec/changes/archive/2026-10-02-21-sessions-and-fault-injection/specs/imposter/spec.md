## MODIFIED Requirements
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

