## MODIFIED Requirements
### Requirement: Preset commands
The left sidebar SHALL list preset commands grouped into categories: Session (`StartSession()`, `EndSession()`, `GetDMEVersion()`), Motion (`Home()`, `GoTo(10, 10, 10)`, `GoTo(0, 0, 0)`), Measurement (`PtMeas()`), Tooling (`SetCoordSystem(MCS|PCS)`, `GetCoordSystem()`, `GetCsyTransformation(PartCsy)`, `EnumCoordSystems()`) and further presets for the status methods (`IsHomed()`, `IsUserEnabled()`, `EnableUser()`, `DisableUser()`, `GetMachineClass()`, `GetErrStatusE()`, `GetXtdErrStatus()`, `Get(X, Y, Z)`, `ClearAllErrors()`, `AbortE()`, `StopAllDaemons()`). Up and Down SHALL move the selection and Enter SHALL send the selected preset.

#### Scenario: Send a preset
- **WHEN** the `StartSession()` preset is selected and Enter is pressed while connected
- **THEN** `StartSession()` is sent and the reply is logged

