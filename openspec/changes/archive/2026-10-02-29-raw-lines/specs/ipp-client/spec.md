## ADDED Requirements
### Requirement: Raw lines can be sent verbatim
`send_line(line)` SHALL send the text exactly as given, unparsed and unvalidated, and SHALL reject a line containing CR or LF. If the line starts with a five-digit tag it SHALL wait for the reply carrying that tag and return it. If it has no leading tag it SHALL return `None` immediately after sending, and any reply SHALL still be delivered through `subscribe()`.

#### Scenario: Tagged raw line
- **WHEN** `send_line("00009 Home()")` is called
- **THEN** it returns the reply tagged `00009`

#### Scenario: Untagged raw line
- **WHEN** `send_line("garbage")` is called
- **THEN** it returns `None` without waiting

#### Scenario: Embedded newline
- **WHEN** `send_line("00001 Home()\r\n00002 Home()")` is called
- **THEN** it fails with an invalid-argument error and nothing is sent

