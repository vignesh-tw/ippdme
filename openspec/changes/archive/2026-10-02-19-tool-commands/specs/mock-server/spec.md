## ADDED Requirements
### Requirement: Tools
The mock SHALL know the tools `RefTool`, `NoTool` and `NormalTool`, and `EnumTools` SHALL list them. `Tool`, `FoundTool`, `FindTool`, `ChangeTool` and `SetTool` SHALL be acknowledged without keeping state. `AlignTool` SHALL echo the requested alignment back as a data response.

#### Scenario: Enumerate tools
- **WHEN** `EnumTools()` is sent
- **THEN** the reply lists `RefTool`, `NoTool` and `NormalTool`

