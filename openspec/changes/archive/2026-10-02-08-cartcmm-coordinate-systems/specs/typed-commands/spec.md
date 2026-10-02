## MODIFIED Requirements
### Requirement: Typed coverage of the supported I++ DME methods
`Command` SHALL provide typed variants for `StartSession`, `EndSession`, `GetDMEVersion`, `Home`, `GoTo`, `PtMeas`, `SetCoordSystem` and the bare stubs `OnMoveArc` and `ScanOnCircle`, plus the server methods (`StopDaemon`, `StopAllDaemons`, `AbortE`, `GetErrorInfo`, `ClearAllErrors`, `GetProp`, `GetPropE`, `SetProp`, `EnumProp`, `EnumAllProp`), the DME status methods (`IsHomed`, `EnableUser`, `DisableUser`, `IsUserEnabled`, `GetMachineClass`, `GetErrStatusE`, `GetXtdErrStatus`, `Get`, `OnPtMeasReport`, `OnMoveReportE`) and the CartCMM coordinate-system methods (`GetCoordSystem`, `GetCsyTransformation`, `SetCsyTransformation`, `SaveActiveCoordSystem`, `LoadCoordSystem`, `DeleteCoordSystem`, `EnumCoordSystems`, `GetNamedCsyTransformation`, `SaveNamedCsyTransformation`). Variable-shape methods SHALL carry their arguments as raw terms. `docs/SUPPORTED_METHODS.md` SHALL track which spec methods have typed variants.

#### Scenario: Round trip
- **WHEN** `Command::StartSession` is converted to a term and back
- **THEN** the result is `Command::StartSession`

#### Scenario: Property path round trip
- **WHEN** `GetProp(Tool.PtMeasPar.Speed())` is parsed into a command and serialized
- **THEN** the original term is reproduced

### Requirement: Coordinate-system and transformation arguments
`SetCoordSystem` SHALL accept the identifiers `MCS` and `PCS`. `GetCsyTransformation` and `SetCsyTransformation` SHALL accept the kinds `PartCsy`, `JogDisplayCsy`, `JogMoveCsy`, `SensorCsy`, `MoveableMachineCsy` and `MultipleArmCsy`. A transformation SHALL be the six numbers `X0, Y0, Z0, Theta, Psi, Phi`. Named coordinate systems SHALL be addressed by a quoted string name.

#### Scenario: Unknown coordinate system
- **WHEN** `SetCoordSystem(XYZ)` is parsed
- **THEN** it fails with a wrong-argument-type error

