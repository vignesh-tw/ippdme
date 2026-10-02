## MODIFIED Requirements
### Requirement: Helpers over common commands
The client SHALL provide helpers for `start_session`, `end_session`, `get_dme_version`, `home`, `go_to`, `pt_meas` and `set_coord_system`, and `send_command` for any typed command. Each SHALL return the raw reply message, whatever its marker. A helper given invalid arguments (such as a non-finite coordinate) SHALL fail with an invalid-argument error without sending anything.

#### Scenario: Error reply is a value
- **WHEN** a helper is answered with `Error(UnknownCommand)`
- **THEN** the helper returns that reply message rather than failing

#### Scenario: Invalid argument
- **WHEN** `go_to` is called with a NaN coordinate
- **THEN** it fails with an invalid-argument error and nothing is sent

