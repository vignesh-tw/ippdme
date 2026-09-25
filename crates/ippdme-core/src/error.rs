use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum IppError {
    #[error("failed to parse I++ DME line: {0}")]
    Parse(String),
    #[error("invalid tag: expected 5 ASCII digits, got {0:?}")]
    InvalidTag(String),
    #[error("unexpected message kind: expected {expected}, got {got}")]
    UnexpectedKind { expected: String, got: String },
    #[error("missing required argument {name:?} for {func:?}")]
    MissingArg { func: String, name: String },
    #[error("argument {name:?} for {func:?} has the wrong type")]
    WrongArgType { func: String, name: String },
    #[error("unknown command: {0}")]
    UnknownCommand(String),
}

pub type Result<T> = std::result::Result<T, IppError>;
