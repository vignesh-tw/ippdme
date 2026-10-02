## ADDED Requirements
### Requirement: Closed connections fail pending requests immediately
When the connection closes or the reader hits a decode error, every request still waiting SHALL fail with a connection-closed error right away instead of running into its timeout. A request started after the connection closed SHALL also fail with connection-closed.

#### Scenario: Server drops mid-command
- **WHEN** the server closes the connection while a command is awaiting its reply
- **THEN** the call fails promptly with a connection-closed error

