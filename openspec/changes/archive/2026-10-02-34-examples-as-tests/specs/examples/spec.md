## ADDED Requirements
### Requirement: Examples are executable tests
Every example YAML SHALL be parsed and started by a Rust test (`examples/rust-client/tests/examples.rs`) on an ephemeral port by rewriting only its `port:` line, and the behavior `examples/README.md` promises SHALL be asserted.

#### Scenario: Stale example
- **WHEN** a code change makes an example YAML invalid or its documented reply wrong
- **THEN** the example test fails

