# mock-server Specification

## Purpose

A small stateful simulator of a CMM (`IppMockServer` in `ippdme-net`, `mock.rs`), used as the reference fixture for Rust integration tests, the Python `ippdme.testing` fixtures and the TUI's built-in mock mode. Changes to its behavior must stay in sync with both the Rust and the Python test suites.

## Requirements

### Requirement: Lenient by default
By default the mock SHALL accept any command at any time, still answering from its state. Commands that are not typed variants, or that fail strict parsing, SHALL be answered `Error(UnknownCommand)`.

#### Scenario: Move without a session
- **WHEN** `GoTo(...)` is sent before `StartSession()` with default configuration
- **THEN** it is acknowledged

#### Scenario: Unknown command
- **WHEN** `Bogus()` is sent
- **THEN** the reply is `Error(UnknownCommand)`

### Requirement: Simulated movement latency
`Home` and `GoTo` SHALL take the configured latency before replying, 500 ms by default. `MockConfig::instant()` SHALL make them immediate, and `with_latency` SHALL set an explicit value.

#### Scenario: Instant config
- **WHEN** the mock is configured with `instant()`
- **THEN** `Home` replies without added delay

#### Scenario: GoTo takes time
- **WHEN** `GoTo(...)` is sent to a mock with the default configuration
- **THEN** the reply arrives no sooner than 500 ms later

### Requirement: Session and status commands
`StartSession` SHALL mark the session started and reply `Ready()` under the ack marker. `EndSession` SHALL mark it not started and reply `Ack()`. `GetDMEVersion` SHALL reply `DMEVersion("1.4")`. `Home` SHALL set homed. `EnableUser` and `DisableUser` SHALL set and clear user-enabled. `IsHomed` and `IsUserEnabled` SHALL reply with a `0`/`1` flag. `GetMachineClass` SHALL reply `CartCMM`. `GetErrStatusE` SHALL reply `ErrStatus(0)`. `GetErrorInfo(n)` SHALL reply with the string `Error n`. Daemon, abort, error-clearing and report-subscription commands SHALL be acknowledged.

#### Scenario: StartSession reply
- **WHEN** `StartSession()` is sent
- **THEN** the reply is an ack-marker `Ready()`

#### Scenario: Flags
- **WHEN** `Home()` then `IsHomed()` are sent
- **THEN** the second reply is `IsHomed(1)`

### Requirement: Motion and measurement
`GoTo` SHALL set each axis it specifies and keep the others, then acknowledge. `PtMeas` SHALL move to any axes it specifies and reply with a `PtMeas` data response containing the resulting X, Y, Z and a normal: the normal supplied, or (0, 0, 1) if none. With no target `PtMeas` SHALL report the current position. `Get` SHALL reply with the current X, Y, Z.

#### Scenario: Partial GoTo
- **WHEN** the machine is at (10.002, 20.001, 5.000) and `GoTo(X(1.0))` is sent
- **THEN** a following `PtMeas()` reports X=1.0, Y=20.001, Z=5.0

#### Scenario: Default normal
- **WHEN** `PtMeas()` is sent with no normal
- **THEN** the reply has I=0, J=0, K=1

#### Scenario: Fixed measurement
- **WHEN** `PtMeas()` is sent on a fresh session
- **THEN** the reply reports X=10.002, Y=20.001, Z=5.0, I=0, J=0, K=1

### Requirement: Construction
`IppMockServer::bind(addr)` SHALL create a lenient mock with default latency. `IppMockServer::builder()` SHALL allow setting `MockConfig` and a TLS configuration before binding. `local_addr` SHALL report the bound address. `spawn_ephemeral` and `spawn_ephemeral_with(config)` SHALL bind `127.0.0.1:0`, start serving in the background and return the address and task handle for tests.

#### Scenario: Ephemeral helper
- **WHEN** `spawn_ephemeral_with(MockConfig::instant().strict())` is called
- **THEN** a strict, instant mock is serving on a free local port

#### Scenario: Ephemeral port
- **WHEN** `spawn_ephemeral` is called
- **THEN** a mock is serving on a free local port

### Requirement: Properties are not modelled
`GetProp`, `GetPropE`, `EnumProp` and `EnumAllProp` SHALL reply with an empty `Prop()` data response and `SetProp` SHALL be acknowledged, without keeping any property tree.

#### Scenario: GetProp
- **WHEN** `GetProp(Tool.PtMeasPar.Speed())` is sent
- **THEN** the reply is a data response `Prop()`

### Requirement: Coordinate systems
`SetCoordSystem` SHALL change the active coordinate system and `GetCoordSystem` SHALL report it as `MachineCsy` (MCS) or `PartCsy` (PCS). `SetCsyTransformation` for `PartCsy` SHALL store the transformation and `GetCsyTransformation` for `PartCsy` SHALL return it; other kinds SHALL return the identity transformation. `SaveActiveCoordSystem(name)` SHALL save the current part transformation under that name. `LoadCoordSystem(name)` SHALL restore a saved transformation and select `PCS`. `DeleteCoordSystem` SHALL remove a saved system. `EnumCoordSystems` SHALL list saved names. `SaveNamedCsyTransformation` and `GetNamedCsyTransformation` SHALL store and read a named transformation. Loading, deleting or reading a name that is not saved SHALL reply `Error(CoordSystemNotFound)`.

#### Scenario: Save and load
- **WHEN** a part transformation is set, saved as "Fixture1", changed, then "Fixture1" is loaded
- **THEN** the original transformation is active and the coordinate system is PCS

#### Scenario: Missing name
- **WHEN** `LoadCoordSystem("Nope")` is sent
- **THEN** the reply is `Error(CoordSystemNotFound)`

#### Scenario: Canned coordinate system
- **WHEN** `GetCoordSystem()` is sent on a fresh session
- **THEN** the reply is `CoordSystem(PartCsy)`

### Requirement: Tools
The mock SHALL know the tools `RefTool`, `NoTool` and `NormalTool`. `EnumTools` SHALL list them. `FindTool(name)` SHALL record a known tool and reply `Error(ToolNotFound)` for an unknown one. `FoundTool` SHALL acknowledge once a tool was found and otherwise reply `Error(ToolNotDefined)`. `ChangeTool` and `SetTool` SHALL make a known tool active and reply `Error(ToolNotFound)` for an unknown one. `AlignTool` SHALL echo the requested alignment back as a data response. `Tool`, `GoToPar` and `PtMeasPar` SHALL be acknowledged.

#### Scenario: Unknown tool
- **WHEN** `ChangeTool("Bogus")` is sent
- **THEN** the reply is `Error(ToolNotFound)`

#### Scenario: FoundTool before FindTool
- **WHEN** `FoundTool()` is sent on a fresh session
- **THEN** the reply is `Error(ToolNotDefined)`

#### Scenario: Enumerate tools
- **WHEN** `EnumTools()` is sent
- **THEN** the reply lists `RefTool`, `NoTool` and `NormalTool`

### Requirement: Each connection is its own machine
Each connection SHALL have independent simulated state: session started, machine homed, user enabled, current position, active coordinate system, part transformation, active tool, last found tool, and saved named coordinate systems. A new session SHALL start not started, not homed, user not enabled, at position (10.002, 20.001, 5.000), with coordinate system `PCS`, an identity part transformation and tool `NoTool`.

#### Scenario: Isolation
- **WHEN** one client homes the machine and a second client connects and asks `IsHomed()`
- **THEN** the second client is told it is not homed

### Requirement: Strict mode enforces call order
With `strict` enabled, a command other than `StartSession` or `GetDMEVersion` before the session starts SHALL be answered `Error(NoSession)`. `Home`, `GoTo`, `PtMeas`, `ChangeTool` and `AlignTool` SHALL require the user to be enabled, answering `Error(UserNotEnabled)` otherwise. `GoTo` and `PtMeas` SHALL additionally require the machine to be homed, answering `Error(NotHomed)` otherwise.

#### Scenario: No session
- **WHEN** a strict mock receives `Home()` first
- **THEN** the reply is `Error(NoSession)`

#### Scenario: User not enabled
- **WHEN** a strict mock has a started session and receives `Home()` before `EnableUser()`
- **THEN** the reply is `Error(UserNotEnabled)`

#### Scenario: Not homed
- **WHEN** a strict mock has a started session and an enabled user and receives `GoTo(...)` before `Home()`
- **THEN** the reply is `Error(NotHomed)`

#### Scenario: Correct order
- **WHEN** the client sends `StartSession`, `EnableUser`, `Home`, `GoTo`
- **THEN** every command is acknowledged
