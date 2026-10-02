## MODIFIED Requirements
### Requirement: Preset commands
The left sidebar SHALL list preset commands grouped into categories (Session, Motion, Measurement, Tooling, and others), including `StartSession()`, `EndSession()`, `GetDMEVersion()`, `Home()`, `GoTo(10, 10, 10)`, `GoTo(0, 0, 0)`, `PtMeas()`, `SetCoordSystem(MCS|PCS)`, `EnumTools()`, `Tool()`, `ChangeTool("RefTool")`, `GetCoordSystem()`, `GetCsyTransformation(PartCsy)` and `EnumCoordSystems()`. Up and Down SHALL move the selection (clamped at the ends) and Enter SHALL send the selected preset.

#### Scenario: Send a preset
- **WHEN** the `Session → StartSession()` preset is selected and Enter pressed while connected
- **THEN** `StartSession()` is sent and the reply is logged

