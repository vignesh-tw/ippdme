//! A predicate plus the sequence of responses it plays back.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::predicate::Predicate;
use crate::response::TimedResponse;

/// A single stub: match `predicate`, then reply with the next entry in
/// `responses`. Once the sequence is exhausted, the last response repeats
/// for every subsequent match (Mountebank's "sticky last response" semantics)
/// — useful for a one-shot fault followed by steady-state behavior.
#[derive(Debug)]
pub struct Stub {
    predicate: Predicate,
    responses: Vec<TimedResponse>,
    cursor: AtomicUsize,
}

impl Stub {
    pub fn new(predicate: Predicate, responses: Vec<TimedResponse>) -> Self {
        assert!(
            !responses.is_empty(),
            "a stub must have at least one response"
        );
        Stub {
            predicate,
            responses,
            cursor: AtomicUsize::new(0),
        }
    }

    /// Start building a stub matching `predicate`.
    pub fn when(predicate: Predicate) -> StubBuilder {
        StubBuilder {
            predicate,
            responses: Vec::new(),
        }
    }

    pub fn matches(&self, term: &ippdme_core::Term) -> bool {
        self.predicate.matches(term)
    }

    /// Advance and return the next response in the sequence, sticking on the
    /// last one once exhausted.
    pub fn next_response(&self) -> &TimedResponse {
        let i = self.cursor.fetch_add(1, Ordering::Relaxed);
        let idx = i.min(self.responses.len() - 1);
        &self.responses[idx]
    }
}

/// Builder for [`Stub`], for constructing imposters programmatically in
/// Rust rather than from a YAML file.
pub struct StubBuilder {
    predicate: Predicate,
    responses: Vec<TimedResponse>,
}

impl StubBuilder {
    pub fn responds_with(mut self, response: impl Into<TimedResponse>) -> Self {
        self.responses.push(response.into());
        self
    }

    pub fn build(self) -> Stub {
        Stub::new(self.predicate, self.responses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::response::ResponseSpec;
    use ippdme_core::Term;

    #[test]
    fn sticks_on_last_response_once_exhausted() {
        let stub = Stub::when(Predicate::call("PtMeas"))
            .responds_with(ResponseSpec::Ack)
            .responds_with(ResponseSpec::Error("Jammed".into()))
            .build();

        assert_eq!(stub.next_response().spec, ResponseSpec::Ack);
        assert_eq!(
            stub.next_response().spec,
            ResponseSpec::Error("Jammed".into())
        );
        // Exhausted: keeps returning the last one.
        assert_eq!(
            stub.next_response().spec,
            ResponseSpec::Error("Jammed".into())
        );
    }

    #[test]
    fn matches_delegates_to_predicate() {
        let stub = Stub::when(Predicate::call("Home"))
            .responds_with(ResponseSpec::Ack)
            .build();
        assert!(stub.matches(&Term::unit("Home")));
        assert!(!stub.matches(&Term::unit("GoTo")));
    }
}
