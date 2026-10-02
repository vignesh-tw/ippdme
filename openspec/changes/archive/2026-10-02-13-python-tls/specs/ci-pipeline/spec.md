## MODIFIED Requirements
### Requirement: Python-level tests against the mock
`tests/` SHALL contain pytest integration tests of the Python client, including TLS and mutual TLS, run against `IppMockServer`.

#### Scenario: Strict ordering from Python
- **WHEN** the Python tests run a strict mock and send `go_to` before `home`
- **THEN** they assert an error response with reason `NotHomed`

#### Scenario: Unknown command from Python
- **WHEN** the Python tests send an unknown command
- **THEN** they assert an error response with a reason

