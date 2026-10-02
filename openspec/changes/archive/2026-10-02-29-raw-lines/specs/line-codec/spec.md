## ADDED Requirements
### Requirement: Raw lines can be sent verbatim
The encoder SHALL accept an outgoing item that is either a typed message or a raw line, and SHALL write a raw line exactly as given plus CRLF without parsing or validating it, so hand-typed or deliberately malformed input reaches the wire unchanged.

#### Scenario: Malformed raw line
- **WHEN** the raw line `not a valid line` is encoded
- **THEN** the bytes written are `not a valid line\r\n`

## MODIFIED Requirements
### Requirement: CR and LF cannot be smuggled into an outbound line
The encoder SHALL refuse any outbound line, typed or raw, that contains a CR or LF inside it, failing with an invalid-argument protocol error and writing nothing, so a string argument cannot inject a second line onto the wire.

#### Scenario: Newline in a string argument
- **WHEN** a command whose string argument contains `\n` is encoded
- **THEN** encoding fails with an invalid-argument error and no bytes are written

