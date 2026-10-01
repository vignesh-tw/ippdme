//! Server-side transport: accept connections, frame them with
//! [`MessageCodec`], and dispatch every inbound command to a [`Handler`].
//!
//! This is the one place that owns the server accept/read/write loop; the
//! mock CMM ([`crate::mock::IppMockServer`]) and the Mountebank-style
//! imposter are just handlers on top of it.

use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use ippdme_core::{Message, Tag, Term};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio_util::codec::Framed;
use tracing::{debug, warn};

use crate::codec::MessageCodec;
use crate::error::Result;

/// Turns one inbound command into its single response message.
///
/// Implemented for any `Fn(Tag, Term) -> impl Future<Output = Message>`
/// closure, so simple servers don't need a named type.
pub trait Handler: Send + Sync + 'static {
    fn handle(&self, tag: Tag, term: Term) -> impl Future<Output = Message> + Send;
}

impl<F, Fut> Handler for F
where
    F: Fn(Tag, Term) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Message> + Send,
{
    fn handle(&self, tag: Tag, term: Term) -> impl Future<Output = Message> + Send {
        self(tag, term)
    }
}

/// A TCP server that speaks I++ DME framing and delegates every command to
/// a [`Handler`].
pub struct IppServer<H> {
    listener: TcpListener,
    handler: Arc<H>,
}

impl<H: Handler> IppServer<H> {
    pub async fn bind(addr: impl ToSocketAddrs, handler: H) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        Ok(IppServer {
            listener,
            handler: Arc::new(handler),
        })
    }

    /// The actual bound address — useful when binding to port 0 in tests.
    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.listener.local_addr()?)
    }

    /// Accept connections forever, handling each on its own task.
    pub async fn serve(self) -> Result<()> {
        loop {
            let (stream, peer) = self.listener.accept().await?;
            debug!(%peer, "server: accepted connection");
            let handler = self.handler.clone();
            tokio::spawn(async move {
                if let Err(e) = serve_connection(stream, &*handler).await {
                    warn!(%peer, error = %e, "server: connection ended with error");
                }
            });
        }
    }
}

/// Drive one connection over any byte transport until the peer closes it.
/// Public so tests (and future transports such as TLS) can serve a stream
/// that didn't come from a [`TcpListener`].
pub async fn serve_connection<S, H>(stream: S, handler: &H) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
    H: Handler,
{
    let mut framed = Framed::new(stream, MessageCodec);

    while let Some(result) = framed.next().await {
        let msg = result?;
        let Message::Command { tag, term } = msg else {
            // Servers don't expect to receive marked responses from clients.
            continue;
        };

        let reply = handler.handle(tag, term).await;
        if framed.send(reply).await.is_err() {
            break;
        }
    }
    Ok(())
}
