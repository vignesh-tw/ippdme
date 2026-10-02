## MODIFIED Requirements
### Requirement: Terms model every argument shape
A term SHALL be one of: a call `Name(arg, ...)` (including the parameterless `Name()`), a bare identifier (`MCS`), a number (`10.002`, `-5`, `1e-3`), or a double-quoted string. Identifiers SHALL start with an ASCII letter or underscore and MAY continue with ASCII letters, digits, underscores and dots, so dotted property paths such as `Tool.PtMeasPar.Speed` parse as one name.

#### Scenario: Nested command parses
- **WHEN** `00042 GoTo(X(10.0), Y(20.0), Z(5.0))` is parsed
- **THEN** the result is a command with tag 42 whose term is named `GoTo` and whose numeric parameters `X`, `Y`, `Z` are 10.0, 20.0, 5.0

#### Scenario: Dotted property path parses
- **WHEN** `00001 GetProp(Tool.PtMeasPar.Speed())` is parsed
- **THEN** the first argument's name is `Tool.PtMeasPar.Speed`

#### Scenario: Signed floats parse
- **WHEN** a data response contains `Y(-20.001)`
- **THEN** the numeric parameter `Y` reads as -20.001

