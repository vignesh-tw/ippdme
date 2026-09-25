use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("protocol error: {0}")]
    Protocol(#[from] ippdme_core::IppError),
    #[error("request timed out waiting for a response to tag {0}")]
    Timeout(ippdme_core::Tag),
    #[error("connection closed")]
    ConnectionClosed,
    #[error("client is shutting down")]
    Shutdown,
}

pub type Result<T> = std::result::Result<T, NetError>;
