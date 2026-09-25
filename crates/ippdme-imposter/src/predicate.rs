//! Matching an incoming [`Term`] against a stub's configured predicate.

use ippdme_core::Term;

/// Matches a call by name and, optionally, exact args — a Mountebank-style
/// `equals` predicate. `args: None` matches any args for that call name.
#[derive(Debug, Clone, PartialEq)]
pub struct Predicate {
    call: String,
    args: Option<Vec<Term>>,
}

impl Predicate {
    pub fn new(call: impl Into<String>, args: Option<Vec<Term>>) -> Self {
        Predicate {
            call: call.into(),
            args,
        }
    }

    /// Match any call with this name, regardless of args.
    pub fn call(name: impl Into<String>) -> Self {
        Predicate::new(name, None)
    }

    /// Match this call name only when its args equal `args` exactly.
    pub fn call_with_args(name: impl Into<String>, args: Vec<Term>) -> Self {
        Predicate::new(name, Some(args))
    }

    pub fn matches(&self, term: &Term) -> bool {
        if term.name() != Some(self.call.as_str()) {
            return false;
        }
        match &self.args {
            None => true,
            Some(expected) => term.args() == expected.as_slice(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_by_name_only() {
        let pred = Predicate::call("PtMeas");
        assert!(pred.matches(&Term::call("PtMeas", vec![Term::Number(1.0)])));
        assert!(!pred.matches(&Term::call("GoTo", vec![])));
    }

    #[test]
    fn matches_exact_args() {
        let pred = Predicate::call_with_args("GetErrorInfo", vec![Term::Number(42.0)]);
        assert!(pred.matches(&Term::call("GetErrorInfo", vec![Term::Number(42.0)])));
        assert!(!pred.matches(&Term::call("GetErrorInfo", vec![Term::Number(1.0)])));
    }
}
