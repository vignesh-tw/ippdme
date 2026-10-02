## ADDED Requirements
### Requirement: Typed helpers over common commands
The client SHALL provide helpers for `start_session`, `end_session`, `get_dme_version`, `home`, `go_to`, `pt_meas`, `set_coord_system`, `is_homed` and `is_user_enabled`. Helpers returning nothing SHALL require an ack-marker reply. Helpers returning a value SHALL parse the data reply into a typed value (version string, point, flag). A server `Error(...)` reply SHALL surface as a protocol error carrying the server's reason. `send_command` SHALL return the raw reply message for callers who want to inspect errors themselves.

#### Scenario: Server error from a helper
- **WHEN** `go_to` is answered with `Error(NotHomed)`
- **THEN** the helper fails with a server error whose reason is `NotHomed`

#### Scenario: Typed value
- **WHEN** `pt_meas` is answered with `PtMeas(X(1), Y(2), Z(3), ...)`
- **THEN** it returns the point (1, 2, 3)

## REMOVED Requirements
### Requirement: Helpers over common commands
**Reason**: Helpers no longer hand back raw replies for the caller to inspect; they return typed values and fail on `Error(...)`.
**Migration**: Use the typed helpers, or `send_command` to get the raw reply message.

