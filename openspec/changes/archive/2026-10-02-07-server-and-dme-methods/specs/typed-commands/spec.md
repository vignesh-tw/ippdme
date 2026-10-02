## MODIFIED Requirements
### Requirement: Typed coverage of the supported I++ DME methods
`Command` SHALL provide typed variants for `StartSession`, `EndSession`, `GetDMEVersion`, `Home`, `GoTo`, `PtMeas`, `SetCoordSystem` and the bare stubs `OnMoveArc` and `ScanOnCircle`, plus the server methods (`StopDaemon`, `StopAllDaemons`, `AbortE`, `GetErrorInfo`, `ClearAllErrors`, `GetProp`, `GetPropE`, `SetProp`, `EnumProp`, `EnumAllProp`) and the DME status methods (`IsHomed`, `EnableUser`, `DisableUser`, `IsUserEnabled`, `GetMachineClass`, `GetErrStatusE`, `GetXtdErrStatus`, `Get`, `OnPtMeasReport`, `OnMoveReportE`). Variable-shape methods SHALL carry their arguments as raw terms.

#### Scenario: Round trip
- **WHEN** `Command::StartSession` is converted to a term and back
- **THEN** the result is `Command::StartSession`

#### Scenario: Property path round trip
- **WHEN** `GetProp(Tool.PtMeasPar.Speed())` is parsed into a command and serialized
- **THEN** the original term is reproduced

