## ADDED Requirements
### Requirement: Example imposters
The repository SHALL ship `examples/plain/imposter.yaml` (a well-behaved machine on port 1294) and `examples/plain/faults.yaml` (a misbehaving machine on port 1294).

#### Scenario: Run the plain machine
- **WHEN** `cargo run -p ippdme-imposter --bin ippdme-imposter -- examples/plain/imposter.yaml` is run
- **THEN** it prints that it is listening on `127.0.0.1:1294`

### Requirement: Fault scenarios
`faults.yaml` SHALL demonstrate: a first `GoTo` that returns `Error(CollisionDetected)` and later ones that succeed, a `PtMeas` that replies after 8 seconds (longer than the client's 5 s timeout), an `IsHomed` that drops the connection, and a `GetMachineClass` that sends a malformed line.

#### Scenario: Retry test
- **WHEN** a client sends `GoTo` twice to the faults machine
- **THEN** the first is a collision error and the second is acknowledged

#### Scenario: Client timeout
- **WHEN** a client with the default timeout sends `PtMeas` to the faults machine
- **THEN** the client reports a timeout

