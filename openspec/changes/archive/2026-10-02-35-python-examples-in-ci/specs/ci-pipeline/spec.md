## MODIFIED Requirements
### Requirement: Python bindings are built and tested
CI SHALL create a virtualenv with `uv venv`, install `maturin` and `pytest`, run `maturin develop -m crates/ippdme-py/Cargo.toml` (which refuses to run without an active virtualenv), run `pytest tests/`, and then run `examples/python/run-all.sh`.

#### Scenario: Python example breaks
- **WHEN** a binding change breaks `examples/python/plain_client.py`
- **THEN** the Python job fails

#### Scenario: Binding change breaks tests
- **WHEN** a binding change breaks a Python test
- **THEN** the Python job fails

