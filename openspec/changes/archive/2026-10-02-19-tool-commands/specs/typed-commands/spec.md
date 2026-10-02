## ADDED Requirements
### Requirement: AlignTool takes one or two direction vectors
`AlignTool` SHALL take either a primary unit vector with a maximum error angle `alpha`, or a primary and a secondary unit vector with error angles `alpha` and `beta`. Vectors SHALL be unit vectors within tolerance. Any other argument shape SHALL be rejected.

#### Scenario: One vector
- **WHEN** `AlignTool(0, 0, 1, 0.5)` is parsed and serialized
- **THEN** it round-trips

#### Scenario: Bad shape
- **WHEN** `AlignTool` is given three arguments
- **THEN** parsing fails

## MODIFIED Requirements
### Requirement: Typed coverage of the supported I++ DME methods
`Command` SHALL provide typed variants for the server methods (session, daemons, errors, properties, version), the DME methods (homing, user enable, machine class, error status, `Get`, `GoTo`, `PtMeas`, report subscriptions, tool handling including `AlignTool` and `EnumTools`), and the CartCMM coordinate-system methods (get/set coordinate system, get/set transformations, save/load/delete/enumerate named coordinate systems) as listed in `docs/SUPPORTED_METHODS.md`. Variable-shape methods (`GetProp`, `SetProp`, `Get`, `OnPtMeasReport`, and similar) SHALL carry their arguments as raw terms.

#### Scenario: Property path round trip
- **WHEN** `GetProp(Tool.PtMeasPar.Speed())` is parsed into a command and serialized
- **THEN** the original term is reproduced

#### Scenario: Round trip
- **WHEN** `Command::StartSession` is converted to a term and back
- **THEN** the result is `Command::StartSession`

### Requirement: Names are validated for the wire
Coordinate-system names and tool names SHALL be non-empty, at most 255 characters, printable ASCII (letters, digits, punctuation and space) and SHALL NOT contain a double quote, because the wire format has no string escaping. These checks SHALL apply to names built in code and names parsed from the wire.

#### Scenario: Quote in a name
- **WHEN** a coordinate-system name containing `"` is constructed
- **THEN** it is rejected with an invalid-argument error

#### Scenario: Empty or over-long name
- **WHEN** a name is empty or longer than 255 characters
- **THEN** it is rejected

