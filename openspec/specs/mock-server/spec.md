# mock-server Specification

## Purpose

A virtual CMM that speaks I++ DME over TCP, for testing clients without physical hardware (`IppMockServer` in `ippdme-net`, `mock.rs`).

## Requirements

### Requirement: Lenient by default
The mock SHALL accept any command at any time, in any order. A term that cannot be parsed into a typed command, or is `Raw`, SHALL be answered `Error(UnknownCommand)`.

#### Scenario: Unknown command
- **WHEN** `Bogus()` is sent
- **THEN** the reply is `Error(UnknownCommand)`

### Requirement: Simulated movement latency
`Home` and `GoTo` SHALL take a fixed 500 ms before replying, approximating machine movement.

#### Scenario: GoTo takes time
- **WHEN** `GoTo(...)` is sent
- **THEN** the reply arrives no sooner than 500 ms later

### Requirement: Session and status commands
`StartSession` SHALL reply `Ready()` under the ack marker, `EndSession` SHALL reply `Ack()` and `GetDMEVersion` SHALL reply `DMEVersion("1.4")`. `IsHomed` and `IsUserEnabled` SHALL reply with the flag `1`. `EnableUser` and `DisableUser` SHALL be acknowledged. `GetMachineClass` SHALL reply `CartCMM`, `GetErrStatusE` SHALL reply `ErrStatus(0)` and `GetErrorInfo(n)` SHALL reply with the string `Error n`. Daemon, abort, error-clearing and report-subscription commands SHALL be acknowledged.

#### Scenario: StartSession reply
- **WHEN** `StartSession()` is sent
- **THEN** the reply is an ack-marker `Ready()`

#### Scenario: Flags
- **WHEN** `IsHomed()` is sent
- **THEN** the reply is `IsHomed(1)`

### Requirement: Motion and measurement
`GoTo` SHALL be acknowledged. `PtMeas` SHALL always reply with a `PtMeas` data response at (10.002, 20.001, 5.000) with normal (0, 0, 1). `Get` SHALL reply with X, Y, Z at the same fixed position. `SetCoordSystem`, `OnMoveArc` and `ScanOnCircle` SHALL be acknowledged.

#### Scenario: Fixed measurement
- **WHEN** `PtMeas()` is sent
- **THEN** the reply reports X=10.002, Y=20.001, Z=5.0, I=0, J=0, K=1

### Requirement: Construction
`IppMockServer::bind(addr)` SHALL bind a listener, `local_addr` SHALL report the bound address, and `serve` SHALL accept connections until dropped. `spawn_ephemeral` SHALL bind `127.0.0.1:0`, serve in the background and return the address and task handle for tests.

#### Scenario: Ephemeral port
- **WHEN** `spawn_ephemeral` is called
- **THEN** a mock is serving on a free local port

### Requirement: Properties are not modelled
`GetProp`, `GetPropE`, `EnumProp` and `EnumAllProp` SHALL reply with an empty `Prop()` data response and `SetProp` SHALL be acknowledged, without keeping any property tree.

#### Scenario: GetProp
- **WHEN** `GetProp(Tool.PtMeasPar.Speed())` is sent
- **THEN** the reply is a data response `Prop()`

### Requirement: Coordinate systems
The mock SHALL answer the coordinate-system methods with canned replies. `GetCoordSystem` SHALL reply `PartCsy`, `GetCsyTransformation` SHALL reply the identity transformation (six zeros), `EnumCoordSystems` SHALL reply an empty list and `GetNamedCsyTransformation` an empty reply. `SetCsyTransformation`, `SaveActiveCoordSystem`, `LoadCoordSystem`, `DeleteCoordSystem` and `SaveNamedCsyTransformation` SHALL be acknowledged without keeping any state.

#### Scenario: Canned coordinate system
- **WHEN** `GetCoordSystem()` is sent
- **THEN** the reply is `CoordSystem(PartCsy)`
