//! Async TCP client for the I++ DME protocol: automatic tag generation,
//! request/response correlation, timeouts, and a broadcast stream of every
//! inbound message (for live inspection, e.g. the TUI).

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use ippdme_core::{response, Command, CoordSystem, Message, Point, Tag, Term};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::time::timeout;
use tokio_util::codec::Framed;

use crate::codec::MessageCodec;
use crate::error::{NetError, Result};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
const EVENT_BUFFER: usize = 1024;

type PendingMap = Arc<Mutex<HashMap<Tag, oneshot::Sender<Message>>>>;

/// An async client connection to an I++ DME server (a real CMM/gateway, or
/// [`crate::mock::IppMockServer`]).
pub struct IppClient {
    write_tx: mpsc::UnboundedSender<Message>,
    pending: PendingMap,
    next_tag: Arc<AtomicU32>,
    closed: Arc<AtomicBool>,
    events: broadcast::Sender<Message>,
    default_timeout: Duration,
}

impl IppClient {
    /// Connect to `addr` (e.g. `"127.0.0.1:1294"`) and spawn the background
    /// read/write tasks.
    pub async fn connect(addr: impl ToSocketAddrs) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self::from_stream(stream))
    }

    /// Connect to `addr` over TLS 1.3, verifying the server against `tls`.
    #[cfg(feature = "tls")]
    pub async fn connect_tls(
        addr: impl ToSocketAddrs,
        tls: &crate::tls::TlsClientConfig,
    ) -> Result<Self> {
        let tcp = TcpStream::connect(addr).await?;
        let stream = tls.connector().connect(tls.server_name(), tcp).await?;
        Ok(Self::from_stream(stream))
    }

    /// Wrap an already-connected byte stream (a [`TcpStream`], a TLS
    /// stream, an in-memory duplex pipe in tests, ...).
    pub fn from_stream<S>(stream: S) -> Self
    where
        S: AsyncRead + AsyncWrite + Send + 'static,
    {
        let framed = Framed::new(stream, MessageCodec);
        let (mut sink, mut stream) = framed.split();

        let (write_tx, mut write_rx) = mpsc::unbounded_channel::<Message>();
        let pending: PendingMap = Arc::new(Mutex::new(HashMap::new()));
        let (events, _) = broadcast::channel(EVENT_BUFFER);

        tokio::spawn(async move {
            while let Some(msg) = write_rx.recv().await {
                if sink.send(msg).await.is_err() {
                    break;
                }
            }
        });

        let closed = Arc::new(AtomicBool::new(false));
        let pending_reader = pending.clone();
        let events_reader = events.clone();
        let closed_reader = closed.clone();
        tokio::spawn(async move {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(msg) => {
                        let _ = events_reader.send(msg.clone());
                        if let Message::Response { tag, .. } = &msg {
                            if let Some(tx) = pending_reader.lock().unwrap().remove(tag) {
                                let _ = tx.send(msg);
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
            // The connection is gone: fail everything still waiting now
            // rather than letting each request run into its timeout.
            closed_reader.store(true, Ordering::SeqCst);
            pending_reader.lock().unwrap().clear();
        });

        IppClient {
            write_tx,
            pending,
            next_tag: Arc::new(AtomicU32::new(1)),
            closed,
            events,
            default_timeout: DEFAULT_TIMEOUT,
        }
    }

    pub fn set_default_timeout(&mut self, d: Duration) {
        self.default_timeout = d;
    }

    /// Subscribe to every inbound message (responses and events) as they
    /// arrive, for live inspection independent of request/response pairing.
    pub fn subscribe(&self) -> broadcast::Receiver<Message> {
        self.events.subscribe()
    }

    fn next_tag(&self) -> Tag {
        Tag::new(self.next_tag.fetch_add(1, Ordering::Relaxed))
    }

    /// Allocate the next outbound tag without sending anything, e.g. so a
    /// caller can log/display the tag before awaiting the response — see
    /// [`IppClient::send_with_tag`].
    pub fn allocate_tag(&self) -> Tag {
        self.next_tag()
    }

    /// Send a raw [`Term`] as a command and await its correlated response.
    pub async fn send(&self, term: Term) -> Result<Message> {
        let tag = self.next_tag();
        self.send_with_tag(tag, term).await
    }

    /// Send a raw [`Term`] using a tag allocated ahead of time via
    /// [`IppClient::allocate_tag`], and await its correlated response.
    pub async fn send_with_tag(&self, tag: Tag, term: Term) -> Result<Message> {
        let (tx, rx) = oneshot::channel();
        self.pending.lock().unwrap().insert(tag, tx);
        // Checked after inserting: if the reader closed in between, its
        // final clear may have missed this entry.
        if self.closed.load(Ordering::SeqCst) {
            self.pending.lock().unwrap().remove(&tag);
            return Err(NetError::ConnectionClosed);
        }

        let message = Message::Command { tag, term };
        if self.write_tx.send(message).is_err() {
            self.pending.lock().unwrap().remove(&tag);
            return Err(NetError::ConnectionClosed);
        }

        match timeout(self.default_timeout, rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => Err(NetError::ConnectionClosed),
            Err(_) => {
                self.pending.lock().unwrap().remove(&tag);
                Err(NetError::Timeout(tag))
            }
        }
    }

    pub async fn send_command(&self, cmd: Command) -> Result<Message> {
        self.send(cmd.into()).await
    }

    /// Send `cmd` and require an ack; a server `Error(...)` response
    /// becomes `Err(NetError::Protocol(IppError::ServerError { .. }))`.
    async fn send_expecting_ack(&self, cmd: Command) -> Result<()> {
        Ok(response::expect_ack(&self.send_command(cmd).await?)?)
    }

    pub async fn start_session(&self) -> Result<()> {
        self.send_expecting_ack(Command::StartSession).await
    }

    pub async fn end_session(&self) -> Result<()> {
        self.send_expecting_ack(Command::EndSession).await
    }

    pub async fn get_dme_version(&self) -> Result<String> {
        let reply = self.send_command(Command::GetDmeVersion).await?;
        Ok(response::parse_dme_version(&reply)?)
    }

    pub async fn home(&self) -> Result<()> {
        self.send_expecting_ack(Command::Home).await
    }

    pub async fn go_to(&self, x: f64, y: f64, z: f64) -> Result<()> {
        self.send_expecting_ack(Command::go_to(x, y, z)?).await
    }

    /// Measure a point at the current position and return what the machine
    /// reports.
    pub async fn pt_meas(&self) -> Result<Point> {
        let reply = self.send_command(Command::pt_meas()).await?;
        Ok(response::parse_pt_meas(&reply)?)
    }

    pub async fn set_coord_system(&self, cs: CoordSystem) -> Result<()> {
        self.send_expecting_ack(Command::SetCoordSystem(cs)).await
    }

    pub async fn is_homed(&self) -> Result<bool> {
        let reply = self.send_command(Command::IsHomed).await?;
        Ok(response::parse_flag(&reply, "IsHomed")?)
    }

    pub async fn is_user_enabled(&self) -> Result<bool> {
        let reply = self.send_command(Command::IsUserEnabled).await?;
        Ok(response::parse_flag(&reply, "IsUserEnabled")?)
    }
}
