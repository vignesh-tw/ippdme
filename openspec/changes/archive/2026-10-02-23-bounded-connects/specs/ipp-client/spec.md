## RENAMED Requirements
- FROM: `### Requirement: Connecting over TCP`
- TO: `### Requirement: Connecting is bounded in time`

## MODIFIED Requirements
### Requirement: Connecting is bounded in time
`IppClient::connect` SHALL give up after 5 seconds by default, and `connect_timeout` SHALL allow an explicit limit. On expiry it SHALL fail with a connect-timeout error rather than hang. `IppClient::from_stream` SHALL wrap any already-connected `AsyncRead + AsyncWrite` stream (TCP, TLS, in-memory pipe).

#### Scenario: Unreachable address
- **WHEN** connecting to an address that never accepts within the limit
- **THEN** the call fails with a connect-timeout error naming the limit

#### Scenario: Connected
- **WHEN** a mock server is listening
- **THEN** `connect` returns a client ready to send

