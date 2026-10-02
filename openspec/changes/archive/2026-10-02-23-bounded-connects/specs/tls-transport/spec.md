## ADDED Requirements
### Requirement: Handshakes are bounded
The client's TCP connect plus TLS handshake SHALL together finish within 5 seconds by default, configurable with `with_connect_timeout`, failing with a connect-timeout error. The server SHALL drop a connection whose handshake does not complete within 5 seconds so a silent peer cannot hold a task forever.

#### Scenario: Silent peer on the server
- **WHEN** a peer connects to a TLS server and sends nothing
- **THEN** the server ends that connection after the handshake timeout

