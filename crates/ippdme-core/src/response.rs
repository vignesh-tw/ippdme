//! Helpers for building server -> client responses, and for reading them
//! back into typed values (`expect_*` / `parse_*`).

use crate::ast::{Marker, Message, Tag, Term};
use crate::error::{IppError, Result};
use crate::values::Point;

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

fn describe(msg: &Message) -> String {
    msg.to_string()
}

/// The reason carried by an `Error(...)` response, e.g. `UnknownCommand`.
fn server_error(term: &Term) -> IppError {
    let reason = match term.args().first() {
        Some(Term::Ident(s)) | Some(Term::Str(s)) => s.clone(),
        _ => term.to_string(),
    };
    IppError::ServerError { reason }
}

/// Succeeds for an `#` (ack) response, whatever its term is (`Ack()`,
/// `Ready()`, ...). An `!` response becomes [`IppError::ServerError`].
pub fn expect_ack(msg: &Message) -> Result<()> {
    match msg {
        Message::Response {
            marker: Marker::Ack,
            ..
        } => Ok(()),
        Message::Response {
            marker: Marker::Error,
            term,
            ..
        } => Err(server_error(term)),
        other => Err(IppError::UnexpectedKind {
            expected: "ack response".into(),
            got: describe(other),
        }),
    }
}

/// The term of a `%` (data) response named `name`. An `!` response becomes
/// [`IppError::ServerError`].
pub fn expect_data<'a>(msg: &'a Message, name: &str) -> Result<&'a Term> {
    match msg {
        Message::Response {
            marker: Marker::Data,
            term,
            ..
        } if term.name() == Some(name) => Ok(term),
        Message::Response {
            marker: Marker::Error,
            term,
            ..
        } => Err(server_error(term)),
        other => Err(IppError::UnexpectedKind {
            expected: format!("{name} data response"),
            got: describe(other),
        }),
    }
}

/// The measured point from a `PtMeas(...)` data response.
pub fn parse_pt_meas(msg: &Message) -> Result<Point> {
    Point::from_term(expect_data(msg, "PtMeas")?)
}

/// The version string from a `DMEVersion("...")` data response.
pub fn parse_dme_version(msg: &Message) -> Result<String> {
    let term = expect_data(msg, "DMEVersion")?;
    match term.args().first() {
        Some(Term::Str(s)) => Ok(s.clone()),
        _ => Err(IppError::WrongArgType {
            func: "DMEVersion".into(),
            name: "version".into(),
        }),
    }
}

/// A `0`/`1` flag from a data response named `name`, e.g. `IsHomed(1)`.
pub fn parse_flag(msg: &Message, name: &str) -> Result<bool> {
    match expect_data(msg, name)?.args().first() {
        Some(Term::Number(n)) => Ok(*n != 0.0),
        _ => Err(IppError::WrongArgType {
            func: name.into(),
            name: "flag".into(),
        }),
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

    #[test]
    fn expect_ack_accepts_any_ack_and_surfaces_server_errors() {
        assert!(expect_ack(&ack(Tag(1))).is_ok());
        assert!(expect_ack(&ready(Tag(1))).is_ok());
        assert_eq!(
            expect_ack(&error(Tag(1), "CollisionDetected")),
            Err(IppError::ServerError {
                reason: "CollisionDetected".into()
            })
        );
        assert!(expect_ack(&data(Tag(1), Term::unit("X"))).is_err());
    }

    #[test]
    fn parses_pt_meas_version_and_flags() {
        let point = Point::xyz(1.0, 2.0, 3.0).unwrap();
        let msg = data(Tag(1), crate::Command::PtMeas(point).into());
        assert_eq!(parse_pt_meas(&msg).unwrap(), point);

        let msg = data(
            Tag(2),
            Term::call("DMEVersion", vec![Term::Str("1.4".into())]),
        );
        assert_eq!(parse_dme_version(&msg).unwrap(), "1.4");

        let msg = data(Tag(3), Term::call("IsHomed", vec![Term::Number(1.0)]));
        assert!(parse_flag(&msg, "IsHomed").unwrap());
        let msg = data(Tag(3), Term::call("IsHomed", vec![Term::Number(0.0)]));
        assert!(!parse_flag(&msg, "IsHomed").unwrap());
    }

    #[test]
    fn data_parsers_report_server_errors_and_wrong_replies() {
        assert!(matches!(
            parse_pt_meas(&error(Tag(1), "NotHomed")),
            Err(IppError::ServerError { .. })
        ));
        assert!(parse_pt_meas(&ack(Tag(1))).is_err());
        assert!(parse_dme_version(&data(Tag(1), Term::unit("Other"))).is_err());
    }
}
