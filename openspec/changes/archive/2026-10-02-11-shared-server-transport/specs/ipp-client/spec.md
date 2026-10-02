## MODIFIED Requirements
### Requirement: Connecting over TCP
`IppClient::connect(addr)` SHALL open a TCP connection and spawn background read and write tasks. `IppClient::from_stream` SHALL wrap any already-connected `AsyncRead + AsyncWrite` stream (TCP, TLS, in-memory pipe).

#### Scenario: Connected
- **WHEN** a mock server is listening
- **THEN** `connect` returns a client ready to send

