//! What a matched stub sends back.

use ippdme_core::{response, Message, Tag, Term};

/// The kind of response a stub sends for a matched call.
#[derive(Debug, Clone, PartialEq)]
pub enum ResponseSpec {
    /// An acknowledgement. `None` sends the plain `Ack()` term; `Some(name)`
    /// sends `<name>()` under the same Ack marker — e.g. `StartSession`
    /// replies `Ready()` as an ack, not a literal `Ack()`.
    Ack(Option<String>),
    Error(String),
    Data(Term),
}

impl ResponseSpec {
    pub fn ack() -> Self {
        ResponseSpec::Ack(None)
    }

    pub fn ack_named(name: impl Into<String>) -> Self {
        ResponseSpec::Ack(Some(name.into()))
    }

    pub fn to_message(&self, tag: Tag) -> Message {
        match self {
            ResponseSpec::Ack(None) => response::ack(tag),
            ResponseSpec::Ack(Some(name)) => Message::Response {
                tag,
                marker: ippdme_core::Marker::Ack,
                term: Term::unit(name),
            },
            ResponseSpec::Error(reason) => response::error(tag, reason.clone()),
            ResponseSpec::Data(term) => response::data(tag, term.clone()),
        }
    }
}

/// A [`ResponseSpec`] plus a delay before it's sent, simulating machine
/// latency (e.g. `GoTo`/`Home` movement time).
#[derive(Debug, Clone, PartialEq)]
pub struct TimedResponse {
    pub spec: ResponseSpec,
    pub after_ms: u64,
}

impl TimedResponse {
    pub fn new(spec: ResponseSpec) -> Self {
        TimedResponse { spec, after_ms: 0 }
    }

    pub fn after_ms(mut self, ms: u64) -> Self {
        self.after_ms = ms;
        self
    }
}

impl From<ResponseSpec> for TimedResponse {
    fn from(spec: ResponseSpec) -> Self {
        TimedResponse::new(spec)
    }
}
