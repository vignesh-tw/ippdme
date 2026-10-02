## ADDED Requirements
### Requirement: CR and LF cannot be smuggled into an outbound line
The encoder SHALL refuse any outbound message whose wire text contains a CR or LF inside it, failing with an invalid-argument protocol error and writing nothing, so a string argument cannot inject a second line onto the wire.

#### Scenario: Newline in a string argument
- **WHEN** a command whose string argument contains `\n` is encoded
- **THEN** encoding fails with an invalid-argument error and no bytes are written

