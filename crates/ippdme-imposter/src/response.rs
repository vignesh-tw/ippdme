//! What a matched stub sends back.

use ippdme_core::{response, Message, Tag, Term};

/// The kind of response a stub sends for a matched call.
#[derive(Debug, Clone, PartialEq)]
pub enum ResponseSpec {
    Ack,
    Error(String),
    Data(Term),
}

impl ResponseSpec {
    pub fn to_message(&self, tag: Tag) -> Message {
        match self {
            ResponseSpec::Ack => response::ack(tag),
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
