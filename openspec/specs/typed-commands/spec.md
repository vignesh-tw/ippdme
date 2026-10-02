# typed-commands Specification

## Purpose

Ergonomic, validated Rust types on top of the generic wire model: a `Command` enum, validated argument types, and the response helpers used to build and read replies. Implemented in `ippdme-core` (`commands.rs`, `values.rs`, `response.rs`). The AST stays the source of truth; typed commands are sugar over it. Which I++ DME methods have a typed variant is tracked in `docs/SUPPORTED_METHODS.md`.

## Requirements

### Requirement: Commands convert to and from terms
Every `Command` variant SHALL convert into a `Term`, and `Command::try_from(&Term)` SHALL convert a term back into the same variant. Method names SHALL match the I++ DME spelling, for example `GetDmeVersion` serializes as `GetDMEVersion`.

#### Scenario: Round trip
- **WHEN** `Command::go_to(10.0, 20.0, 5.0)` is converted to a term and back
- **THEN** the result equals the original command

### Requirement: Typed coverage of the supported I++ DME methods
`Command` SHALL provide typed variants for the server methods (session, daemons, errors, properties, version), the DME methods (homing, user enable, machine class, error status, `Get`, `GoTo`, `PtMeas`, report subscriptions, tool handling including `AlignTool` and `EnumTools`), and the CartCMM coordinate-system methods (get/set coordinate system, get/set transformations, save/load/delete/enumerate named coordinate systems) as listed in `docs/SUPPORTED_METHODS.md`. Variable-shape methods (`GetProp`, `SetProp`, `Get`, `OnPtMeasReport`, and similar) SHALL carry their arguments as raw terms.

#### Scenario: Property path round trip
- **WHEN** `GetProp(Tool.PtMeasPar.Speed())` is parsed into a command and serialized
- **THEN** the original term is reproduced

#### Scenario: Round trip
- **WHEN** `Command::StartSession` is converted to a term and back
- **THEN** the result is `Command::StartSession`

### Requirement: Coordinate-system and transformation arguments
`SetCoordSystem` SHALL accept the identifiers `MCS` and `PCS`. `GetCsyTransformation` and `SetCsyTransformation` SHALL accept the kinds `PartCsy`, `JogDisplayCsy`, `JogMoveCsy`, `SensorCsy`, `MoveableMachineCsy` and `MultipleArmCsy`. A transformation SHALL be the six finite numbers `X0, Y0, Z0, Theta, Psi, Phi`.

#### Scenario: Unknown coordinate system
- **WHEN** `SetCoordSystem(XYZ)` is parsed
- **THEN** it fails with a wrong-argument-type error

#### Scenario: Non-finite transformation value
- **WHEN** a transformation is built with an infinite component
- **THEN** it is rejected

### Requirement: Response builders and readers
The library SHALL build `Ack()` and `Ready()` acknowledgements, `Error(reason)` errors with a bare-identifier reason, and `%` data responses for a given tag. `expect_ack` SHALL succeed on any ack-marker response (whatever its term) and turn an error response into a `ServerError` carrying the reason. `expect_data(name)` SHALL return the term of a data response with that name, turn an error response into `ServerError`, and fail with an unexpected-kind error for anything else. Typed readers SHALL be provided for `PtMeas` (a point), `DMEVersion` (a string) and `0`/`1` flags such as `IsHomed`.

#### Scenario: Ready counts as an ack
- **WHEN** `expect_ack` is given `00001 # Ready()`
- **THEN** it succeeds

#### Scenario: Server error surfaced
- **WHEN** `expect_ack` is given `00001 ! Error(CollisionDetected)`
- **THEN** it fails with a `ServerError` whose reason is `CollisionDetected`

#### Scenario: Wrong reply kind
- **WHEN** `parse_pt_meas` is given an ack response
- **THEN** it fails with an unexpected-kind error

#### Scenario: Flag parsing
- **WHEN** `parse_flag(msg, "IsHomed")` is given `IsHomed(1)` and then `IsHomed(0)`
- **THEN** it returns true and then false

#### Scenario: Error builder
- **WHEN** `error(tag 1, "UnknownCommand")` is built
- **THEN** it displays as `00001 ! Error(UnknownCommand)`

### Requirement: Strict parsing by default with an explicit lenient escape hatch
`Command::try_from` SHALL fail with `UnknownCommand` for a method that has no typed variant. `Command::from_term_lenient` SHALL instead return `Command::Raw(term)` for unknown methods, but SHALL still fail when a recognized method has invalid arguments. `Command::raw(term)` SHALL be the explicit way to send an untyped term, and the term SHALL NOT be checked against the protocol spec.

#### Scenario: Unknown command, strict
- **WHEN** `Command::try_from` is given `Bogus()`
- **THEN** it fails with `UnknownCommand`

#### Scenario: Unknown command, lenient
- **WHEN** `Command::from_term_lenient` is given `Bogus()`
- **THEN** it returns `Command::Raw` wrapping that term

#### Scenario: Lenient still rejects bad arguments
- **WHEN** `Command::from_term_lenient` is given `GetErrorInfo(1.5)`
- **THEN** it fails rather than falling back to `Raw`

### Requirement: Event and error numbers are whole numbers in range
`StopDaemon` and `GetErrorInfo` arguments SHALL be non-negative whole numbers that fit in an unsigned 32-bit integer. Any other value coming off the wire SHALL be rejected with a wrong-argument-type error.

#### Scenario: Fractional or negative number
- **WHEN** `GetErrorInfo(-1)` or `StopDaemon(2.5)` is parsed into a command
- **THEN** it fails with a wrong-argument-type error

### Requirement: Points are validated
A `Point` SHALL hold optional `X`, `Y`, `Z` and an optional surface normal `I`, `J`, `K`. All coordinates SHALL be finite. The normal SHALL be given as all three of `I`, `J`, `K` or none, and SHALL be a unit vector within a tolerance of 1e-3 on its length. Fields SHALL be private and read through accessors. Invalid values SHALL be rejected both when built in code and when parsed from the wire.

#### Scenario: Non-finite coordinate
- **WHEN** `Command::go_to(f64::NAN, 0.0, 0.0)` is called
- **THEN** it fails with an invalid-argument error

#### Scenario: Non-unit normal
- **WHEN** a point is given the normal (1, 1, 1)
- **THEN** construction fails because the length is not 1

#### Scenario: Partial normal
- **WHEN** a `PtMeas` term supplies `I` and `J` but not `K`
- **THEN** parsing fails because the three components must be given together

#### Scenario: Measure in place
- **WHEN** `Command::pt_meas()` is serialized
- **THEN** it is `PtMeas()` with no arguments

### Requirement: Names are validated for the wire
Coordinate-system names and tool names SHALL be non-empty, at most 255 characters, printable ASCII (letters, digits, punctuation and space) and SHALL NOT contain a double quote, because the wire format has no string escaping. These checks SHALL apply to names built in code and names parsed from the wire.

#### Scenario: Quote in a name
- **WHEN** a coordinate-system name containing `"` is constructed
- **THEN** it is rejected with an invalid-argument error

#### Scenario: Empty or over-long name
- **WHEN** a name is empty or longer than 255 characters
- **THEN** it is rejected

### Requirement: AlignTool takes one or two direction vectors
`AlignTool` SHALL take either a primary unit vector with a maximum error angle `alpha`, or a primary and a secondary unit vector with error angles `alpha` and `beta`. Vectors SHALL be unit vectors within tolerance. Any other argument shape SHALL be rejected.

#### Scenario: One vector
- **WHEN** `AlignTool(0, 0, 1, 0.5)` is parsed and serialized
- **THEN** it round-trips

#### Scenario: Bad shape
- **WHEN** `AlignTool` is given three arguments
- **THEN** parsing fails
