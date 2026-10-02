## Context

Tests of real call order (`Home` before `GoTo`, `EnableUser` before motion) could not be written against a mock that answered from constants.

## Decision

Each connection owns a `MockSession`. By default the mock stays lenient about call order but answers from its state; `MockConfig::strict` additionally rejects commands issued out of order with `NoSession`, `UserNotEnabled` and `NotHomed`. Latency is configurable, with `MockConfig::instant()` for tests.

## Consequences

Behavior changes to the mock must be mirrored in the Rust integration tests and the Python `ippdme.testing` fixtures.
