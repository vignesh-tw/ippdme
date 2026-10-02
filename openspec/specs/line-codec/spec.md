# line-codec Specification

## Purpose

Framing of the I++ DME byte stream into parsed messages and back, shared by every client and server in `ippdme-net` (`codec.rs`).

## Requirements

### Requirement: Inbound bytes are framed into one message per line
The decoder SHALL split the stream on LF, accept either bare LF or CRLF, parse each non-empty line into a `Message`, and return nothing until a full line has arrived. Empty lines SHALL be skipped. Several lines arriving in one read SHALL be decoded one at a time, in order.

#### Scenario: Two lines in one buffer
- **WHEN** the buffer holds `00001 # Ack()\r\n00002 ! Error(UnknownCommand)\r\n`
- **THEN** successive decodes return the messages with tags 1 and 2, then nothing

#### Scenario: Partial line
- **WHEN** the buffer holds `00001 # Ack(` with no terminator
- **THEN** the decoder returns nothing and keeps the bytes for later

#### Scenario: Bare LF
- **WHEN** a line ends in LF without CR
- **THEN** it decodes the same as a CRLF-terminated line

### Requirement: Undecodable lines are errors
A line that is not valid UTF-8, or that does not parse as a protocol line, SHALL produce a protocol error from the decoder.

#### Scenario: Garbage line
- **WHEN** the decoder receives `hello world\r\n`
- **THEN** it returns a protocol (parse) error

### Requirement: Outbound lines end in CRLF
The encoder SHALL write each message's wire text followed by CRLF.

#### Scenario: Encoded message
- **WHEN** the command `00001 StartSession()` is encoded
- **THEN** the bytes written are `00001 StartSession()\r\n`

### Requirement: CR and LF cannot be smuggled into an outbound line
The encoder SHALL refuse any outbound line, typed or raw, that contains a CR or LF inside it, failing with an invalid-argument protocol error and writing nothing, so a string argument cannot inject a second line onto the wire.

#### Scenario: Newline in a string argument
- **WHEN** a command whose string argument contains `\n` is encoded
- **THEN** encoding fails with an invalid-argument error and no bytes are written

### Requirement: Raw lines can be sent verbatim
The encoder SHALL accept an outgoing item that is either a typed message or a raw line, and SHALL write a raw line exactly as given plus CRLF without parsing or validating it, so hand-typed or deliberately malformed input reaches the wire unchanged.

#### Scenario: Malformed raw line
- **WHEN** the raw line `not a valid line` is encoded
- **THEN** the bytes written are `not a valid line\r\n`
