//! Helpers for building and inspecting server -> client responses.

use crate::ast::{Marker, Message, Tag, Term};

/// Build an `Ack()` response for the given tag.
pub fn ack(tag: Tag) -> Message {
    Message::Response {
        tag,
        marker: Marker::Ack,
        term: Term::unit("Ack"),
    }
}

/// Build a `Ready()` response for the given tag.
pub fn ready(tag: Tag) -> Message {
    Message::Response {
        tag,
        marker: Marker::Ack,
        term: Term::unit("Ready"),
    }
}

/// Build an `Error(reason)` response for the given tag, where `reason` is a
/// bare identifier such as `UnknownCommand`.
pub fn error(tag: Tag, reason: impl Into<String>) -> Message {
    Message::Response {
        tag,
        marker: Marker::Error,
        term: Term::Call("Error".into(), vec![Term::Ident(reason.into())]),
    }
}

/// Build a `% <term>` data/event response for the given tag.
pub fn data(tag: Tag, term: Term) -> Message {
    Message::Response {
        tag,
        marker: Marker::Data,
        term,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_ack() {
        let msg = ack(Tag(1));
        assert_eq!(msg.to_string(), "00001 # Ack()");
    }

    #[test]
    fn builds_error() {
        let msg = error(Tag(1), "UnknownCommand");
        assert_eq!(msg.to_string(), "00001 ! Error(UnknownCommand)");
    }
}
