## Context

Typed commands were thin sugar over the AST with public fields, so an invalid command could be built and only rejected, if at all, by the machine.

## Decision

Validated argument types live in `values.rs`: constructors reject invalid values and fields are private. Parsing from the wire goes through the same constructors. `Command::try_from` is strict (`UnknownCommand`); `Command::from_term_lenient` is the explicit opt-in to the `Raw` fallback, and still rejects bad arguments of known commands.

## Consequences

`Command::go_to` and `Point::xyz` return `Result`. The Python layer maps `InvalidArgument` to `ValueError`.
