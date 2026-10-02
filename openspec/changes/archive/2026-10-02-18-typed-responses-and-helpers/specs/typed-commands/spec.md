## MODIFIED Requirements
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

