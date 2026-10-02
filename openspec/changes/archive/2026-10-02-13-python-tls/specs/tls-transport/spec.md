## ADDED Requirements
### Requirement: Surfaces expose the same options
Python SHALL expose TLS as `IppClient.connect(addr, ca_cert=, server_name=, client_cert=, client_key=)` and `IppMockServer(port, cert=, key=, client_ca=)`. Details are in the `python-bindings` spec.

#### Scenario: Round trip across surfaces
- **WHEN** a TLS mock is started from Python and a Python client connects with the CA
- **THEN** commands are answered

