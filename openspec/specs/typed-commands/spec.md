# typed-commands Specification

## Purpose

Ergonomic Rust wrappers over the generic wire model: a `Command` enum with `From<Command> for Term` / `TryFrom<&Term>`, and builders for server responses. Implemented in `ippdme-core` (`commands.rs`, `response.rs`). The AST stays the source of truth; typed commands are sugar over it.

## Requirements

### Requirement: Commands convert to and from terms
Every `Command` variant SHALL convert into a `Term`, and `Command::try_from(&Term)` SHALL convert a term back into the same variant. Method names SHALL match the I++ DME spelling, for example `GetDmeVersion` serializes as `GetDMEVersion`.

#### Scenario: Round trip
- **WHEN** `Command::go_to(10.0, 20.0, 5.0)` is converted to a term and back
- **THEN** the result equals the original command

### Requirement: Unknown commands parse as Raw
`Command::try_from` SHALL convert any unrecognized term into `Command::Raw(term)` instead of failing, so every command can be carried through even without a typed variant.

#### Scenario: Unknown command
- **WHEN** `Bogus()` is converted with `try_from`
- **THEN** the result is `Command::Raw` wrapping that term

### Requirement: Typed coverage of the supported I++ DME methods
`Command` SHALL provide typed variants for `StartSession`, `EndSession`, `GetDMEVersion`, `Home`, `GoTo`, `PtMeas`, `SetCoordSystem` and the bare stubs `OnMoveArc` and `ScanOnCircle`, plus the server methods (`StopDaemon`, `StopAllDaemons`, `AbortE`, `GetErrorInfo`, `ClearAllErrors`, `GetProp`, `GetPropE`, `SetProp`, `EnumProp`, `EnumAllProp`) and the DME status methods (`IsHomed`, `EnableUser`, `DisableUser`, `IsUserEnabled`, `GetMachineClass`, `GetErrStatusE`, `GetXtdErrStatus`, `Get`, `OnPtMeasReport`, `OnMoveReportE`). Variable-shape methods SHALL carry their arguments as raw terms.

#### Scenario: Round trip
- **WHEN** `Command::StartSession` is converted to a term and back
- **THEN** the result is `Command::StartSession`

#### Scenario: Property path round trip
- **WHEN** `GetProp(Tool.PtMeasPar.Speed())` is parsed into a command and serialized
- **THEN** the original term is reproduced

### Requirement: Points carry optional coordinates and a normal
A `Point` SHALL hold optional `X`, `Y`, `Z` and optional surface normal components `I`, `J`, `K`, serialized only for the components that are set. `Command::pt_meas()` SHALL build a `PtMeas` with no arguments.

#### Scenario: Measure in place
- **WHEN** `Command::pt_meas()` is serialized
- **THEN** it is `PtMeas()` with no arguments

### Requirement: Coordinate-system and transformation arguments
`SetCoordSystem` SHALL accept the identifiers `MCS` and `PCS` and reject any other argument.

#### Scenario: Unknown coordinate system
- **WHEN** `SetCoordSystem(XYZ)` is converted
- **THEN** it fails with a wrong-argument-type error

### Requirement: Response builders and readers
The library SHALL build `Ack()` and `Ready()` acknowledgements, `Error(reason)` errors with a bare-identifier reason, and `%` data responses for a given tag.

#### Scenario: Error builder
- **WHEN** `error(tag 1, "UnknownCommand")` is built
- **THEN** it displays as `00001 ! Error(UnknownCommand)`
