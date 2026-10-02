## ADDED Requirements
### Requirement: Properties are not modelled
`GetProp`, `GetPropE`, `EnumProp` and `EnumAllProp` SHALL reply with an empty `Prop()` data response and `SetProp` SHALL be acknowledged, without keeping any property tree.

#### Scenario: GetProp
- **WHEN** `GetProp(Tool.PtMeasPar.Speed())` is sent
- **THEN** the reply is a data response `Prop()`

## MODIFIED Requirements
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

