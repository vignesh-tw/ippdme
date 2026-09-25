//! The imposter server: binds a port, matches incoming calls against a
//! stub list, and replays their configured responses.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};

use futures::{SinkExt, StreamExt};
use ippdme_core::{response, Message, Term};
use ippdme_net::MessageCodec;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_util::codec::Framed;
use tracing::{debug, warn};

use crate::config::ImposterConfig;
use crate::error::Result;
use crate::stub::{Stub, StubBuilder};

/// A Mountebank-style imposter for the I++ DME protocol: bind it to a port,
/// register stubs (predicate -> response sequence), and `serve` it to
/// accept and handle connections. Every call it receives is logged and
/// retrievable via [`Imposter::received_calls`] for test verification.
pub struct Imposter {
    listener: TcpListener,
    stubs: Arc<Vec<Stub>>,
    log: Arc<Mutex<Vec<Term>>>,
}

impl Imposter {
    /// Start building an imposter with no stubs yet.
    pub fn builder() -> ImposterBuilder {
        ImposterBuilder { stubs: Vec::new() }
    }

    /// Load an imposter's stubs and port from a YAML file and bind it.
    pub async fn from_yaml_file(path: impl AsRef<Path>) -> Result<Self> {
        let config = ImposterConfig::from_yaml_file(path)?;
        Self::from_config(config).await
    }

    /// Load an imposter's stubs and port from a YAML string and bind it.
    pub async fn from_yaml_str(yaml: &str) -> Result<Self> {
        let config = ImposterConfig::from_yaml_str(yaml)?;
        Self::from_config(config).await
    }

    async fn from_config(config: ImposterConfig) -> Result<Self> {
        let port = config.port;
        let stubs = config.into_stubs()?;
        Self::bind(("127.0.0.1", port), stubs).await
    }

    async fn bind(addr: impl ToSocketAddrs, stubs: Vec<Stub>) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Imposter {
            listener,
            stubs: Arc::new(stubs),
            log: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// The actual bound address — useful when binding to port 0 in tests.
    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.listener.local_addr()?)
    }

    /// A cheap, cloneable handle onto this imposter's received-call log.
    /// `serve` consumes `self`, so grab this *before* spawning it if the
    /// caller needs to verify what was received afterward.
    pub fn handle(&self) -> ImposterHandle {
        ImposterHandle {
            log: self.log.clone(),
        }
    }

    /// Accept connections forever, handling each on its own task.
    pub async fn serve(self) -> Result<()> {
        loop {
            let (stream, peer) = self.listener.accept().await?;
            debug!(%peer, "imposter: accepted connection");
            let stubs = self.stubs.clone();
            let log = self.log.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, stubs, log).await {
                    warn!(%peer, error = %e, "imposter: connection ended with error");
                }
            });
        }
    }
}

async fn handle_connection(
    stream: TcpStream,
    stubs: Arc<Vec<Stub>>,
    log: Arc<Mutex<Vec<Term>>>,
) -> Result<()> {
    let mut framed = Framed::new(stream, MessageCodec);

    while let Some(result) = framed.next().await {
        let msg = result?;
        let Message::Command { tag, term } = msg else {
            // Imposters don't expect to receive marked responses from clients.
            continue;
        };

        log.lock().unwrap().push(term.clone());

        let matched = stubs.iter().find(|stub| stub.matches(&term));
        let reply = match matched {
            Some(stub) => {
                let timed = stub.next_response();
                if timed.after_ms > 0 {
                    tokio::time::sleep(std::time::Duration::from_millis(timed.after_ms)).await;
                }
                timed.spec.to_message(tag)
            }
            None => response::error(tag, "UnknownStub"),
        };

        if framed.send(reply).await.is_err() {
            break;
        }
    }
    Ok(())
}

/// A cheap, cloneable handle onto a running [`Imposter`]'s received-call
/// log, obtained via [`Imposter::handle`] before `serve` consumes it.
#[derive(Clone)]
pub struct ImposterHandle {
    log: Arc<Mutex<Vec<Term>>>,
}

impl ImposterHandle {
    /// Every call term received so far, in arrival order, for verifying what
    /// a client under test actually sent.
    pub fn received_calls(&self) -> Vec<Term> {
        self.log.lock().unwrap().clone()
    }
}

/// Builder for programmatically constructing an [`Imposter`] in Rust (rather
/// than loading it from YAML), e.g. inline in a `#[tokio::test]`.
pub struct ImposterBuilder {
    stubs: Vec<Stub>,
}

impl ImposterBuilder {
    pub fn stub(mut self, stub: impl Into<Stub>) -> Self {
        self.stubs.push(stub.into());
        self
    }

    /// Bind to an explicit address (use `("127.0.0.1", 0)` for an ephemeral
    /// port in tests).
    pub async fn bind(self, addr: impl ToSocketAddrs) -> Result<Imposter> {
        Imposter::bind(addr, self.stubs).await
    }
}

impl From<StubBuilder> for Stub {
    fn from(b: StubBuilder) -> Stub {
        b.build()
    }
}

/// Convenience for tests/examples: pick an ephemeral port, bind, and return
/// the [`Imposter`] along with the address it's listening on.
pub async fn spawn_ephemeral(
    stubs: Vec<Stub>,
) -> Result<(SocketAddr, tokio::task::JoinHandle<Result<()>>)> {
    let imposter = Imposter::bind(("127.0.0.1", 0), stubs).await?;
    let addr = imposter.local_addr()?;
    let handle = tokio::spawn(imposter.serve());
    Ok((addr, handle))
}
