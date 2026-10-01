# Supported I++ DME Methods

Tracks which methods from `docs/ippdme_standard.pdf` (I++ DME v1.5) have a
typed [`Command`](../crates/ippdme-core/src/commands.rs) variant in
`ippdme-core`, versus which are not yet implemented.

Any method without a typed variant can still be sent or parsed losslessly via
`Command::raw(Term)` — nothing is blocked by omission, this table just tracks
ergonomic/typed coverage. "Implemented" here means: has a `Command` variant,
round-trips through `Term`, is handled by `IppMockServer`, and (where it makes
sense standalone) has a TUI preset.

Legend: ✅ implemented · ⬜ not yet implemented

## 6.3.1 Server methods

| Method | Status | `Command` variant |
|---|---|---|
| `StartSession()` | ✅ | `StartSession` |
| `EndSession()` | ✅ | `EndSession` |
| `StopDaemon(EventTag)` | ✅ | `StopDaemon(u32)` |
| `StopAllDaemons()` | ✅ | `StopAllDaemons` |
| `AbortE()` | ✅ | `AbortE` |
| `GetErrorInfo(..)` | ✅ | `GetErrorInfo(u32)` |
| `ClearAllErrors()` | ✅ | `ClearAllErrors` |
| `GetProp(..)` | ✅ | `GetProp(Vec<Term>)` |
| `GetPropE(..)` | ✅ | `GetPropE(Vec<Term>)` |
| `SetProp(..)` | ✅ | `SetProp(Vec<Term>)` |
| `EnumProp(..)` | ✅ | `EnumProp(Vec<Term>)` |
| `EnumAllProp(..)` | ✅ | `EnumAllProp(Vec<Term>)` |
| `GetDMEVersion()` | ✅ | `GetDmeVersion` |

## 6.3.2 DME methods

| Method | Status | `Command` variant |
|---|---|---|
| `Home()` | ✅ | `Home` |
| `IsHomed()` | ✅ | `IsHomed` |
| `EnableUser()` | ✅ | `EnableUser` |
| `DisableUser()` | ✅ | `DisableUser` |
| `IsUserEnabled()` | ✅ | `IsUserEnabled` |
| `OnPtMeasReport(..)` | ✅ | `OnPtMeasReport(Vec<Term>)` |
| `OnMoveReportE(..)` | ✅ | `OnMoveReportE(Vec<Term>)` |
| `GetMachineClass()` | ✅ | `GetMachineClass` |
| `GetErrStatusE()` | ✅ | `GetErrStatusE` |
| `GetXtdErrStatus()` | ✅ | `GetXtdErrStatus` |
| `Get(..)` | ✅ | `Get(Vec<Term>)` |
| `GoTo(..)` | ✅ | `GoTo(Point)` |
| `PtMeas(..)` | ✅ | `PtMeas(Point)` |
| `Tool()` | ✅ | `Tool` |
| `FindTool(..)` | ✅ | `FindTool(ToolName)` |
| `FoundTool()` | ✅ | `FoundTool` |
| `ChangeTool(..)` | ✅ | `ChangeTool(ToolName)` |
| `SetTool(..)` | ✅ | `SetTool(ToolName)` |
| `AlignTool(..)` | ✅ | `AlignTool(ToolAlignment)` (one or two unit vectors with error angles) |
| `GoToPar()` | ✅ | `GoToPar` (pointer only; parameters are reached via `Get`/`SetProp` on `Tool.GoToPar`) |
| `PtMeasPar()` | ✅ | `PtMeasPar` (pointer only; parameters are reached via `Get`/`SetProp` on `Tool.PtMeasPar`) |
| `EnumTools()` | ✅ | `EnumTools` |
| `Q()` | ⬜ | |
| `ER()` | ⬜ | |
| `GetChangeToolAction(..)` | ⬜ | |
| `EnumToolCollection(..)` / `EnumAllToolCollections(..)` / `OpenToolCollection()` | ⬜ | |
| `IJKAct()` | ⬜ | |
| `PtMeasSelfCenter(..)` / `PtMeasSelfCenterLocked(..)` | ⬜ | |

## 6.3.3 CartCMM methods

| Method | Status | `Command` variant |
|---|---|---|
| `SetCoordSystem(..)` | ✅ | `SetCoordSystem(CoordSystem)` — ⚠️ see note below |
| `GetCoordSystem()` | ✅ | `GetCoordSystem` |
| `GetCsyTransformation(..)` | ✅ | `GetCsyTransformation(CsyTransformKind)` |
| `SetCsyTransformation(..)` | ✅ | `SetCsyTransformation(CsyTransformKind, CsyTransform)` |
| `X()` / `Y()` / `Z()` / `IJK()` (query form) | ✅ (generic) | reachable as bare `Term::unit(..)` args of `Get`/`OnPtMeasReport`/`OnMoveReportE` |
| `X(..)` / `Y(..)` / `Z(..)` / `IJK(..)` (move form) | ✅ | via `Point` args of `GoTo`/`PtMeas` |
| `R()` | ⬜ | rotary-table position query; reachable as a raw arg of `Get`, no typed helper yet |
| `SaveActiveCoordSystem(..)` | ✅ | `SaveActiveCoordSystem(CoordSystemName)` |
| `LoadCoordSystem(..)` | ✅ | `LoadCoordSystem(CoordSystemName)` |
| `DeleteCoordSystem(..)` | ✅ | `DeleteCoordSystem(CoordSystemName)` |
| `EnumCoordSystems(..)` | ✅ | `EnumCoordSystems` |
| `GetNamedCsyTransformation(..)` | ✅ | `GetNamedCsyTransformation(CoordSystemName)` |
| `SaveNamedCsyTransformation(..)` | ✅ | `SaveNamedCsyTransformation(CoordSystemName, CsyTransform)` |

> ⚠️ **Known discrepancy:** the existing `SetCoordSystem`/`CoordSystem` enum
> uses the shorthand identifiers `MCS`/`PCS` (matching the dialog examples in
> spec section 7), while section 6.3.3.1 itself lists the parameter as one of
> `MachineCsy`/`MoveableMachineCsy`/`MultipleArmCsy`/`PartCsy`. This predates
> the phased work in this document; flagging it here rather than changing
> established behavior out of scope.

## 6.3.4 / 6.3.5 ToolChanger and Tool methods

⬜ Not yet implemented (`ReQualify()`, `ScanPar()`, tool block properties like
`Name()`, `Id()`, `CollisionVolume()`, `Alignment()`, `AvrRadius()`,
`AlignmentVolume()`, `Collection()`).

## 6.4 Part methods

⬜ Not yet implemented (`Temperature()`, `Temperature(..)`,
`XpanCoefficient()`, `XpanCoefficient(..)`) — reachable today as raw property
paths via `Get`/`SetProp`/`GetProp`.

## Section 11 — Scanning

⬜ Not yet implemented (`OnScanReport(..)`, `ScanOnCircleHint(..)`,
`ScanOnCircle(..)`* , `ScanOnLineHint(..)`, `ScanOnLine(..)`,
`ScanOnCurveHint(..)`, `ScanOnCurveDensity(..)`, `ScanOnCurve(..)`,
`ScanOnHelix(..)`, `ScanUnknownHint(..)`, `ScanUnknownDensity(..)`,
`ScanInPlaneEndIsSphere(..)`, `ScanInPlaneEndIsPlane(..)`,
`ScanInPlaneEndIsCyl(..)`, `ScanInCylEndIsSphere(..)`,
`ScanInCylEndIsPlane(..)`).

\* `Command::ScanOnCircle` and `Command::OnMoveArc` exist today as bare unit
stubs (no arguments) from before this document existed; they don't yet cover
the parameterized methods above.

## Section 12 — Rotary Table

⬜ Not yet implemented (`AlignPart(..)`).

## Section 13 — Formtesters

⬜ Not yet implemented (`CenterPart(..)`, `TiltPart(..)`,
`TiltCenterPart(..)`, `LockAxis(..)`, `LockPosition(..)`).

## Strict parsing

`Command::try_from(&Term)` returns `IppError::UnknownCommand` for methods not
listed above; use `Command::from_term_lenient` to get `Command::Raw` instead.

## Updating this document

When adding a new `Command` variant in `crates/ippdme-core/src/commands.rs`,
update the corresponding row/entry here in the same change.
