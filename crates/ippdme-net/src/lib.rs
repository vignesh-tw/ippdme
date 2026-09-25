//! `ippdme-net`: async Tokio TCP client and mock CMM server for the I++ DME
//! protocol, built on top of `ippdme-core`.

pub mod client;
pub mod codec;
pub mod error;
pub mod mock;

pub use client::IppClient;
pub use codec::MessageCodec;
pub use error::{NetError, Result};
pub use mock::IppMockServer;
