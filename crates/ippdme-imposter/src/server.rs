//! The imposter server: binds a port, matches incoming calls against a
//! stub list, and replays their configured responses.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};

use ippdme_core::{response, Message, Tag, Term};
use ippdme_net::{Handler, IppServer};
use tokio::net::ToSocketAddrs;

use crate::config::ImposterConfig;
use crate::error::Result;
use crate::stub::{Stub, StubBuilder};

/// A Mountebank-style imposter for the I++ DME protocol: bind it to a port,
/// register stubs (predicate -> response sequence), and `serve` it to
/// accept and handle connections. Every call it receives is logged and
/// retrievable via [`Imposter::received_calls`] for test verification.
pub struct Imposter {
    server: IppServer<StubHandler>,
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
        let log = Arc::new(Mutex::new(Vec::new()));
        let handler = StubHandler {
            stubs,
            log: log.clone(),
        };
        let server = IppServer::bind(addr, handler).await?;
        Ok(Imposter { server, log })
    }

    /// The actual bound address — useful when binding to port 0 in tests.
    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.server.local_addr()?)
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
        Ok(self.server.serve().await?)
    }
}

/// Matches each inbound call against the stub list and replays the next
/// configured response.
struct StubHandler {
    stubs: Vec<Stub>,
    log: Arc<Mutex<Vec<Term>>>,
}

impl Handler for StubHandler {
    async fn handle(&self, tag: Tag, term: Term) -> Message {
        self.log.lock().unwrap().push(term.clone());

        match self.stubs.iter().find(|stub| stub.matches(&term)) {
            Some(stub) => {
                let timed = stub.next_response();
                if timed.after_ms > 0 {
                    tokio::time::sleep(std::time::Duration::from_millis(timed.after_ms)).await;
                }
                timed.spec.to_message(tag)
            }
            None => response::error(tag, "UnknownStub"),
        }
    }
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
