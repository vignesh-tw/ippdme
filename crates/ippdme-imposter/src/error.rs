use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImposterError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid imposter config: {0}")]
    Config(#[from] serde_yaml_ng::Error),
    #[error("stub response must set exactly one of ack/error/data: {0}")]
    InvalidResponse(String),
    #[error(transparent)]
    Net(#[from] ippdme_net::NetError),
}

pub type Result<T> = std::result::Result<T, ImposterError>;
