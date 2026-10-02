## ADDED Requirements
### Requirement: Dropping the client closes the connection
Dropping an `IppClient` SHALL stop its reader task and close its half of the socket, so the server sees the connection end instead of it leaking until the server hangs up.

#### Scenario: Client dropped
- **WHEN** the client value goes out of scope
- **THEN** the server observes end-of-stream on that connection

