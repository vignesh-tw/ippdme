## ADDED Requirements
### Requirement: Tags are five-digit zero-padded numbers
A tag SHALL be serialized as exactly five ASCII digits, zero-padded. The maximum tag value SHALL be 99999, and constructing a tag with `Tag::new` SHALL wrap values above the maximum back into the 0..=99999 range.

#### Scenario: Tag is padded on output
- **WHEN** the tag `1` is displayed
- **THEN** it is rendered as `00001`

#### Scenario: Tag wraps past the maximum
- **WHEN** `Tag::new(100000)` is called
- **THEN** the resulting tag is `00000`

### Requirement: Terms model every argument shape
A term SHALL be one of: a call `Name(arg, ...)` (including the parameterless `Name()`), a bare identifier (`MCS`), a number (`10.002`, `-5`, `1e-3`), or a double-quoted string. Identifiers SHALL start with an ASCII letter or underscore and MAY continue with ASCII letters, digits and underscores.

#### Scenario: Nested command parses
- **WHEN** `00042 GoTo(X(10.0), Y(20.0), Z(5.0))` is parsed
- **THEN** the result is a command with tag 42 whose term is named `GoTo` and whose numeric parameters `X`, `Y`, `Z` are 10.0, 20.0, 5.0

#### Scenario: Signed floats parse
- **WHEN** a data response contains `Y(-20.001)`
- **THEN** the numeric parameter `Y` reads as -20.001

### Requirement: Commands and responses are distinguished by marker
A line with a tag and no marker SHALL parse as a command. A line with a tag followed by one of `#` (acknowledgement), `!` (error) or `%` (data/event) SHALL parse as a response carrying that marker. A message SHALL expose `is_ack`, `is_error` and `is_data` predicates matching its marker.

#### Scenario: Ack response
- **WHEN** `00001 # Ack()` is parsed
- **THEN** the message is a response with an ack marker and tag 1

#### Scenario: Error response
- **WHEN** `00001 ! Error(UnknownCommand)` is parsed
- **THEN** the message is an error response whose term is `Error` with the identifier argument `UnknownCommand`

#### Scenario: Data response
- **WHEN** `00007 % PtMeas(X(10.002), ...)` is parsed
- **THEN** `is_data()` is true

### Requirement: Parsing is strict about framing
Parsing a line SHALL fail when the tag is not exactly five digits, and SHALL fail when any input remains after the term. A trailing CR and/or LF SHALL be accepted and ignored.

#### Scenario: Malformed tag
- **WHEN** `1 GoTo()` or `999999 GoTo()` is parsed
- **THEN** parsing fails with a parse error

#### Scenario: Trailing garbage
- **WHEN** `00001 GoTo() extra` is parsed
- **THEN** parsing fails with a parse error naming the trailing input

#### Scenario: CRLF is tolerated
- **WHEN** `00001 StartSession()\r\n` is parsed
- **THEN** it parses as a `StartSession` command with tag 1

### Requirement: Display round-trips the wire form
Displaying a parsed message SHALL reproduce the canonical wire text, with arguments separated by `, `. Whole-valued numbers SHALL be rendered with one decimal place (`10.0`).

#### Scenario: Round trip
- **WHEN** `00001 GoTo(X(10.0), Y(20.0), Z(5.0))` is parsed and displayed
- **THEN** the output equals the input

### Requirement: Helpers read term parameters
A term SHALL provide lookups for the first numeric parameter and first identifier parameter by name, and the library SHALL provide `parse_term_str` for parsing a bare term without a tag or marker.

#### Scenario: Numeric parameter
- **WHEN** `GoTo(X(10.0), Y(20.0))` is queried for `X`
- **THEN** 10.0 is returned

