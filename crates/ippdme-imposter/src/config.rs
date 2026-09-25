//! The YAML-facing shape of an imposter file.
//!
//! These structs are deserialized by serde and then converted into the
//! runtime types in [`crate::predicate`] / [`crate::response`] / [`crate::stub`].
//! Deliberately kept separate from `ippdme_core::Term`: `Term` is not
//! serde-derived (the core crate stays pure/no-I/O, see its `serde` feature
//! gate, which we don't enable here), and a Term-shaped YAML file would leak
//! the AST's internal enum tagging (`Call: [name, [...]]`) into a file meant
//! to be hand-edited by people who don't know Rust.

use std::path::Path;

use ippdme_core::Term;
use serde::Deserialize;

use crate::error::{ImposterError, Result};
use crate::predicate::Predicate;
use crate::response::{ResponseSpec, TimedResponse};
use crate::stub::Stub;

/// Top-level shape of an imposter YAML file.
#[derive(Debug, Clone, Deserialize)]
pub struct ImposterConfig {
    pub port: u16,
    #[serde(default)]
    pub stubs: Vec<StubConfig>,
}

impl ImposterConfig {
    pub fn from_yaml_str(yaml: &str) -> Result<Self> {
        Ok(serde_yaml_ng::from_str(yaml)?)
    }

    pub fn from_yaml_file(path: impl AsRef<Path>) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Self::from_yaml_str(&text)
    }

    pub fn into_stubs(self) -> Result<Vec<Stub>> {
        self.stubs.into_iter().map(StubConfig::into_stub).collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct StubConfig {
    pub predicate: PredicateConfig,
    pub responses: Vec<ResponseConfig>,
}

impl StubConfig {
    fn into_stub(self) -> Result<Stub> {
        let predicate = self.predicate.into_predicate();
        let responses = self
            .responses
            .into_iter()
            .map(ResponseConfig::into_timed_response)
            .collect::<Result<Vec<_>>>()?;
        Ok(Stub::new(predicate, responses))
    }
}

/// Matches an incoming term by call name and, optionally, exact args
/// (a Mountebank-style `equals` predicate).
#[derive(Debug, Clone, Deserialize)]
pub struct PredicateConfig {
    pub call: String,
    #[serde(default)]
    pub args: Option<Vec<ArgValue>>,
}

impl PredicateConfig {
    fn into_predicate(self) -> Predicate {
        Predicate::new(
            self.call,
            self.args
                .map(|args| args.into_iter().map(Term::from).collect()),
        )
    }
}

/// One entry in a stub's `responses` list. Exactly one of `ack`/`error`/`data`
/// must be set; `after_ms` optionally delays the reply to simulate machine
/// latency (e.g. `GoTo`/`Home` movement time).
#[derive(Debug, Clone, Deserialize)]
pub struct ResponseConfig {
    #[serde(default)]
    pub ack: Option<AckValue>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub data: Option<CallShape>,
    #[serde(default)]
    pub after_ms: u64,
}

impl ResponseConfig {
    fn into_timed_response(self) -> Result<TimedResponse> {
        let spec = match (self.ack, self.error, self.data) {
            (Some(AckValue::Bool(true)), None, None) => ResponseSpec::ack(),
            (Some(AckValue::Named(name)), None, None) => ResponseSpec::ack_named(name),
            (None, Some(reason), None) => ResponseSpec::Error(reason),
            (None, None, Some(call)) => ResponseSpec::Data(call.into()),
            other => {
                return Err(ImposterError::InvalidResponse(format!("{other:?}")));
            }
        };
        Ok(TimedResponse {
            spec,
            after_ms: self.after_ms,
        })
    }
}

/// `ack: true` sends the plain `Ack()` term; `ack: Ready` sends `Ready()`
/// under the same Ack marker, for replies like `StartSession`'s that ack
/// with a named term rather than the literal `Ack()`.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum AckValue {
    Bool(bool),
    Named(String),
}

/// `Name(arg1, arg2, ...)`, the YAML-friendly counterpart to
/// [`ippdme_core::Term::Call`].
#[derive(Debug, Clone, Deserialize)]
pub struct CallShape {
    pub call: String,
    #[serde(default)]
    pub args: Vec<ArgValue>,
}

impl From<CallShape> for Term {
    fn from(c: CallShape) -> Term {
        Term::Call(c.call, c.args.into_iter().map(Term::from).collect())
    }
}

/// One argument to a [`CallShape`] or [`PredicateConfig`], covering every
/// shape `Term` can take without exposing `Term`'s own enum tagging.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ArgValue {
    /// A bare number, e.g. `42`, `10.002`.
    Number(f64),
    /// A bare word with no quotes, e.g. `MCS` — becomes `Term::Ident`.
    Ident(String),
    /// An explicit quoted-string literal, e.g. `{str: "Fixture1"}`.
    Str { str: String },
    /// A named single-value parameter, e.g. `{name: X, value: 10.002}`,
    /// the common `X(10.002)`-style wrapper I++ DME uses for point fields.
    Param { name: String, value: Box<ArgValue> },
    /// A fully general nested call, for anything the shorthands above can't
    /// express.
    Call(CallShape),
}

impl From<ArgValue> for Term {
    fn from(v: ArgValue) -> Term {
        match v {
            ArgValue::Number(n) => Term::Number(n),
            ArgValue::Ident(s) => Term::Ident(s),
            ArgValue::Str { str } => Term::Str(str),
            ArgValue::Param { name, value } => Term::Call(name, vec![Term::from(*value)]),
            ArgValue::Call(c) => c.into(),
        }
    }
}
