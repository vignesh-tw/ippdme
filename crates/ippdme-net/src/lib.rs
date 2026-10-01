//! `ippdme-net`: async Tokio TCP client and mock CMM server for the I++ DME
//! protocol, built on top of `ippdme-core`.

pub mod client;
pub mod codec;
pub mod error;
pub mod mock;
pub mod server;
pub mod tap;
#[cfg(feature = "tls")]
pub mod tls;

pub use client::{IppClient, DEFAULT_CONNECT_TIMEOUT};
pub use codec::{MessageCodec, Outgoing};
pub use error::{NetError, Result};
pub use mock::{IppMockServer, MockConfig};
pub use server::{serve_connection, Action, Handler, IppServer};
pub use tap::{IppTap, TapDirection, TapEvent};
#[cfg(feature = "tls")]
pub use tls::{TlsClientConfig, TlsIdentity, TlsServerConfig};
