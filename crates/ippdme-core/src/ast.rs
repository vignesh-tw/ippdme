//! Generic Abstract Syntax Tree for the I++ DME ASCII protocol.
//!
//! Every line on the wire is `<tag> <term>` for a client->server command, or
//! `<tag> <marker> <term>` for a server->client response, where `<term>` is a
//! recursively-nested function call such as `GoTo(X(10.0), Y(20.0), Z(5.0))`.

use std::fmt;

/// A 5-digit zero-padded message tag, e.g. `00001`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tag(pub u32);

impl Tag {
    pub const MAX: u32 = 99_999;

    pub fn new(value: u32) -> Self {
        Tag(value % (Self::MAX + 1))
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:05}", self.0)
    }
}

/// A single node in a term tree: either a leaf value or a named function call
/// with zero or more nested argument terms.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Term {
    /// `Name(arg1, arg2, ...)` — also used for bare `Name()` and, with an
    /// empty ident-only shorthand, for parameterless functions like `Ack()`.
    Call(String, Vec<Term>),
    /// A bare identifier with no parens, e.g. `MCS`, `PCS`, `UnknownCommand`.
    Ident(String),
    /// A numeric literal, e.g. `10.002`, `-5`, `1e-3`.
    Number(f64),
    /// A double-quoted string literal.
    Str(String),
}

impl Term {
    pub fn call(name: impl Into<String>, args: Vec<Term>) -> Self {
        Term::Call(name.into(), args)
    }

    pub fn unit(name: impl Into<String>) -> Self {
        Term::Call(name.into(), Vec::new())
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Term::Call(n, _) => Some(n),
            Term::Ident(n) => Some(n),
            _ => None,
        }
    }

    pub fn args(&self) -> &[Term] {
        match self {
            Term::Call(_, args) => args,
            _ => &[],
        }
    }

    /// Find the first argument that is a `Call(name, [Number(v)])` shaped
    /// parameter, e.g. locating `X` in `GoTo(X(10.0), Y(20.0))`.
    pub fn get_num_param(&self, param_name: &str) -> Option<f64> {
        self.args().iter().find_map(|t| match t {
            Term::Call(n, args) if n == param_name => match args.first() {
                Some(Term::Number(v)) => Some(*v),
                _ => None,
            },
            _ => None,
        })
    }

    pub fn get_ident_param(&self, param_name: &str) -> Option<&str> {
        self.args().iter().find_map(|t| match t {
            Term::Call(n, args) if n == param_name => match args.first() {
                Some(Term::Ident(v)) => Some(v.as_str()),
                _ => None,
            },
            _ => None,
        })
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Call(name, args) => {
                write!(f, "{name}(")?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{a}")?;
                }
                write!(f, ")")
            }
            Term::Ident(s) => write!(f, "{s}"),
            Term::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    write!(f, "{:.1}", n)
                } else {
                    write!(f, "{n}")
                }
            }
            Term::Str(s) => write!(f, "\"{s}\""),
        }
    }
}

/// The response-marker character distinguishing acks, errors, and data/event
/// messages from outbound commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Marker {
    /// `#` — acknowledgement / success.
    Ack,
    /// `!` — error.
    Error,
    /// `%` — data or event.
    Data,
}

impl Marker {
    pub fn as_char(&self) -> char {
        match self {
            Marker::Ack => '#',
            Marker::Error => '!',
            Marker::Data => '%',
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '#' => Some(Marker::Ack),
            '!' => Some(Marker::Error),
            '%' => Some(Marker::Data),
            _ => None,
        }
    }
}

impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

/// A full line of the protocol: a tagged command sent by a client, or a
/// tagged, marked response sent by a server.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Message {
    Command { tag: Tag, term: Term },
    Response { tag: Tag, marker: Marker, term: Term },
}

impl Message {
    pub fn tag(&self) -> Tag {
        match self {
            Message::Command { tag, .. } => *tag,
            Message::Response { tag, .. } => *tag,
        }
    }

    pub fn term(&self) -> &Term {
        match self {
            Message::Command { term, .. } => term,
            Message::Response { term, .. } => term,
        }
    }

    pub fn is_ack(&self) -> bool {
        matches!(
            self,
            Message::Response {
                marker: Marker::Ack,
                ..
            }
        )
    }

    pub fn is_error(&self) -> bool {
        matches!(
            self,
            Message::Response {
                marker: Marker::Error,
                ..
            }
        )
    }

    pub fn is_data(&self) -> bool {
        matches!(
            self,
            Message::Response {
                marker: Marker::Data,
                ..
            }
        )
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::Command { tag, term } => write!(f, "{tag} {term}"),
            Message::Response { tag, marker, term } => write!(f, "{tag} {marker} {term}"),
        }
    }
}
