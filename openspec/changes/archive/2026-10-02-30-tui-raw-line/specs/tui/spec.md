## ADDED Requirements
### Requirement: Raw line input
`r` SHALL toggle raw line mode. In raw line mode, Enter SHALL send the whole input exactly as typed, tag included and unparsed, so it can be malformed on purpose, like typing into `nc`. If the line starts with a tag the reply SHALL be awaited and logged; without a tag the status SHALL say a line was sent without waiting for a reply.

#### Scenario: Raw tagged line
- **WHEN** raw mode is on and `00042 Home()` is submitted
- **THEN** that exact line is sent and the reply tagged `00042` is logged

#### Scenario: Raw untagged line
- **WHEN** raw mode is on and `garbage` is submitted
- **THEN** it is sent and the status reports that no reply is awaited

## MODIFIED Requirements
### Requirement: Typed command input
With the input box focused (Tab cycles Sidebar, Log, Input) and raw line mode off, Enter SHALL parse the text as an I++ term, add the next tag, and send it. A parse error SHALL be logged and shown in the status bar without sending anything.

#### Scenario: Typed term
- **WHEN** `SetCoordSystem(PCS)` is entered and submitted
- **THEN** it is sent with the next tag and the box is cleared

#### Scenario: Parse error
- **WHEN** `GoTo(` is submitted
- **THEN** a `Parse error: ...` entry appears in the log and nothing is sent

