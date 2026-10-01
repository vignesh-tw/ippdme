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
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio_util::codec::Framed;
use tracing::{debug, warn};

use crate::codec::MessageCodec;
use crate::error::Result;

/// What a [`Handler`] does in answer to one command.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Send this message back (the normal case).
    Reply(Message),
    /// Send this text as a line verbatim, bypassing serialization. For
    /// simulating a faulty server that emits garbage.
    RawLine(String),
    /// Close the connection without answering.
    Close,
}

impl From<Message> for Action {
    fn from(msg: Message) -> Self {
        Action::Reply(msg)
    }
}

/// Turns inbound commands into [`Action`]s.
///
/// One `Session` is created per connection and passed mutably to every call
/// on that connection, so a handler can keep per-connection state (an I++
/// session is per connection). Stateless handlers use `()`.
///
/// Any `Fn(Tag, Term) -> impl Future<Output = Message>` closure is a
/// stateless handler, so simple servers don't need a named type.
pub trait Handler: Send + Sync + 'static {
    type Session: Send;

    fn new_session(&self) -> Self::Session;

    fn handle(
        &self,
        session: &mut Self::Session,
        tag: Tag,
        term: Term,
    ) -> impl Future<Output = Action> + Send;
}

impl<F, Fut> Handler for F
where
    F: Fn(Tag, Term) -> Fut + Send + Sync + 'static,
    Fut: Future + Send,
    Fut::Output: Into<Action>,
{
    type Session = ();

    fn new_session(&self) {}

    fn handle(
        &self,
        _session: &mut (),
        tag: Tag,
        term: Term,
    ) -> impl Future<Output = Action> + Send {
        let fut = self(tag, term);
        async move { fut.await.into() }
    }
}

/// A TCP server that speaks I++ DME framing and delegates every command to
/// a [`Handler`].
pub struct IppServer<H> {
    listener: TcpListener,
    handler: Arc<H>,
    #[cfg(feature = "tls")]
    tls: Option<crate::tls::TlsServerConfig>,
}

impl<H: Handler> IppServer<H> {
    pub async fn bind(addr: impl ToSocketAddrs, handler: H) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        Ok(IppServer {
            listener,
            handler: Arc::new(handler),
            #[cfg(feature = "tls")]
            tls: None,
        })
    }

    /// Like [`IppServer::bind`], but every accepted connection is upgraded
    /// to TLS 1.3 (mutual TLS if `tls` was built with a client CA) before
    /// any I++ traffic is read.
    #[cfg(feature = "tls")]
    pub async fn bind_tls(
        addr: impl ToSocketAddrs,
        handler: H,
        tls: crate::tls::TlsServerConfig,
    ) -> Result<Self> {
        let mut server = Self::bind(addr, handler).await?;
        server.tls = Some(tls);
        Ok(server)
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
            #[cfg(feature = "tls")]
            let tls = self.tls.clone();
            tokio::spawn(async move {
                #[cfg(feature = "tls")]
                let result = match tls {
                    // A peer that connects but never completes the handshake
                    // must not hold this task forever.
                    Some(tls) => match tokio::time::timeout(
                        crate::tls::DEFAULT_HANDSHAKE_TIMEOUT,
                        tls.acceptor().accept(stream),
                    )
                    .await
                    {
                        Ok(Ok(stream)) => serve_connection(stream, &*handler).await,
                        Ok(Err(e)) => Err(e.into()),
                        Err(_) => Err(crate::error::NetError::ConnectTimeout(
                            crate::tls::DEFAULT_HANDSHAKE_TIMEOUT,
                        )),
                    },
                    None => serve_connection(stream, &*handler).await,
                };
                #[cfg(not(feature = "tls"))]
                let result = serve_connection(stream, &*handler).await;
                if let Err(e) = result {
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
    let mut session = handler.new_session();

    while let Some(result) = framed.next().await {
        let msg = result?;
        let Message::Command { tag, term } = msg else {
            // Servers don't expect to receive marked responses from clients.
            continue;
        };

        match handler.handle(&mut session, tag, term).await {
            Action::Reply(reply) => {
                if framed.send(reply).await.is_err() {
                    break;
                }
            }
            Action::RawLine(line) => {
                let stream = framed.get_mut();
                let written = async {
                    stream.write_all(line.as_bytes()).await?;
                    stream.write_all(b"\r\n").await?;
                    stream.flush().await
                };
                if written.await.is_err() {
                    break;
                }
            }
            Action::Close => break,
        }
    }
    Ok(())
}
