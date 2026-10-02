## ADDED Requirements
### Requirement: Tags can be allocated before sending
`allocate_tag()` SHALL reserve the next tag without sending anything, and `send_with_tag(tag, term)` SHALL send using a reserved tag. `send` SHALL be exactly an allocation followed by `send_with_tag`. This lets a caller (such as the TUI) log the outbound message with its real tag before awaiting the reply.

#### Scenario: Log before reply
- **WHEN** a caller allocates a tag, records it, then calls `send_with_tag`
- **THEN** the reply carries the recorded tag

