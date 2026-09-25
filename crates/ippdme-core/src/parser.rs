//! `winnow`-based recursive-descent parser for I++ DME protocol lines.

use crate::ast::{Marker, Message, Tag, Term};
use crate::error::IppError;
use winnow::ascii::{digit1, float, multispace0};
use winnow::combinator::{alt, delimited, opt, separated};
use winnow::prelude::*;
use winnow::token::{one_of, take_till, take_while};

fn parse_tag(input: &mut &str) -> ModalResult<Tag> {
    digit1
        .verify(|s: &str| s.len() == 5)
        .try_map(|s: &str| s.parse::<u32>())
        .map(Tag)
        .parse_next(input)
}

fn parse_marker(input: &mut &str) -> ModalResult<Marker> {
    one_of(['#', '!', '%'])
        .map(|c| Marker::from_char(c).expect("one_of guarantees a valid marker char"))
        .parse_next(input)
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_cont(c: char) -> bool {
    // '.' allowed for dotted property paths, e.g. `Tool.PtMeasPar.Speed()`
    // used by GetProp/SetProp/EnumProp and friends.
    c.is_ascii_alphanumeric() || c == '_' || c == '.'
}

fn parse_ident(input: &mut &str) -> ModalResult<String> {
    (one_of(is_ident_start), take_while(0.., is_ident_cont))
        .take()
        .map(|s: &str| s.to_string())
        .parse_next(input)
}

fn parse_string(input: &mut &str) -> ModalResult<Term> {
    delimited('"', take_till(0.., '"'), '"')
        .map(|s: &str| Term::Str(s.to_string()))
        .parse_next(input)
}

fn parse_number(input: &mut &str) -> ModalResult<Term> {
    float.map(Term::Number).parse_next(input)
}

fn parse_call_or_ident(input: &mut &str) -> ModalResult<Term> {
    let name = parse_ident(input)?;
    let args = opt(delimited(
        '(',
        delimited(
            multispace0,
            separated(0.., parse_term, (multispace0, ',', multispace0)),
            multispace0,
        ),
        ')',
    ))
    .parse_next(input)?;
    Ok(match args {
        Some(args) => Term::Call(name, args),
        None => Term::Ident(name),
    })
}

fn parse_term(input: &mut &str) -> ModalResult<Term> {
    alt((parse_call_or_ident, parse_number, parse_string)).parse_next(input)
}

fn parse_message_inner(input: &mut &str) -> ModalResult<Message> {
    let tag = parse_tag(input)?;
    multispace0.parse_next(input)?;
    let marker = opt((parse_marker, multispace0).map(|(m, _)| m)).parse_next(input)?;
    let term = parse_term(input)?;
    multispace0.parse_next(input)?;
    Ok(match marker {
        Some(marker) => Message::Response { tag, marker, term },
        None => Message::Command { tag, term },
    })
}

/// Parse a single I++ DME protocol line (with or without a trailing CRLF)
/// into a [`Message`].
pub fn parse_message(line: &str) -> Result<Message, IppError> {
    let trimmed = line.trim_end_matches(['\r', '\n']);
    let mut input = trimmed;
    parse_message_inner
        .parse_next(&mut input)
        .map_err(|e| IppError::Parse(format!("{e}")))
        .and_then(|msg| {
            if input.is_empty() {
                Ok(msg)
            } else {
                Err(IppError::Parse(format!(
                    "trailing input after message: {input:?}"
                )))
            }
        })
}

/// Parse a bare term (no tag/marker), useful for parsing nested arguments or
/// testing.
pub fn parse_term_str(input: &str) -> Result<Term, IppError> {
    let mut s = input;
    parse_term(&mut s).map_err(|e| IppError::Parse(format!("{e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Marker;

    #[test]
    fn parses_simple_command() {
        let msg = parse_message("00001 StartSession()\r\n").unwrap();
        match msg {
            Message::Command { tag, term } => {
                assert_eq!(tag.0, 1);
                assert_eq!(term, Term::Call("StartSession".into(), vec![]));
            }
            _ => panic!("expected command"),
        }
    }

    #[test]
    fn parses_nested_command() {
        let msg = parse_message("00042 GoTo(X(10.0), Y(20.0), Z(5.0))").unwrap();
        let term = msg.term();
        assert_eq!(term.name(), Some("GoTo"));
        assert_eq!(term.get_num_param("X"), Some(10.0));
        assert_eq!(term.get_num_param("Y"), Some(20.0));
        assert_eq!(term.get_num_param("Z"), Some(5.0));
    }

    #[test]
    fn parses_ack_response() {
        let msg = parse_message("00001 # Ack()").unwrap();
        assert!(msg.is_ack());
        assert_eq!(msg.tag().0, 1);
    }

    #[test]
    fn parses_error_response() {
        let msg = parse_message("00001 ! Error(UnknownCommand)").unwrap();
        assert!(msg.is_error());
        match msg.term() {
            Term::Call(name, args) => {
                assert_eq!(name, "Error");
                assert_eq!(args[0], Term::Ident("UnknownCommand".into()));
            }
            _ => panic!("expected call"),
        }
    }

    #[test]
    fn parses_data_response_with_signed_floats() {
        let msg =
            parse_message("00007 % PtMeas(X(10.002), Y(-20.001), Z(5.000), I(0), J(0), K(1))")
                .unwrap();
        assert!(msg.is_data());
        assert_eq!(msg.term().get_num_param("Y"), Some(-20.001));
    }

    #[test]
    fn parses_ident_argument() {
        let msg = parse_message("00003 SetCoordSystem(MCS)").unwrap();
        assert_eq!(msg.term().args()[0], Term::Ident("MCS".into()));
    }

    #[test]
    fn round_trips_display() {
        let src = "00001 GoTo(X(10.0), Y(20.0), Z(5.0))";
        let msg = parse_message(src).unwrap();
        assert_eq!(msg.to_string(), src);
    }

    #[test]
    fn rejects_malformed_tag() {
        assert!(parse_message("1 GoTo()").is_err());
        assert!(parse_message("999999 GoTo()").is_err());
    }

    #[test]
    fn rejects_trailing_garbage() {
        assert!(parse_message("00001 GoTo() extra").is_err());
    }

    #[test]
    fn parses_dotted_property_path() {
        let msg = parse_message("00001 GetProp(Tool.PtMeasPar.Speed())").unwrap();
        let args = msg.term().args();
        assert_eq!(args[0].name(), Some("Tool.PtMeasPar.Speed"));
    }

    #[test]
    fn marker_round_trip() {
        assert_eq!(Marker::from_char('#'), Some(Marker::Ack));
        assert_eq!(Marker::Data.as_char(), '%');
    }
}
