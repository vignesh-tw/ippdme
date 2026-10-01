//! A transparent TCP tap: sits between a client and a server, forwards the
//! bytes unchanged in both directions, and reports every line it sees.
//!
//! This is the "what is actually on the port" view, from outside both
//! applications, the way `nc` or `tcpdump` would show it. Point the client at
//! the tap's port and the tap at the real server. It forwards raw bytes, so it
//! shows the protocol as sent, malformed lines included. It only understands
//! plain TCP: a TLS connection passes through but its content is encrypted,
//! so the reported "lines" are not readable.

use std::net::SocketAddr;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::broadcast;
use tracing::{debug, warn};

use crate::error::Result;

const EVENT_BUFFER: usize = 4096;

/// Which way a line travelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TapDirection {
    ClientToServer,
    ServerToClient,
}

/// Something observed on the tapped port.
#[derive(Debug, Clone, PartialEq)]
pub enum TapEvent {
    /// A client connected (and the tap reached the target).
    Opened { conn: u64, peer: SocketAddr },
    /// One line, without its CR/LF terminator, decoded lossily as UTF-8.
    Line {
        conn: u64,
        direction: TapDirection,
        line: String,
    },
    /// The connection ended (either side closed it, or the target could not
    /// be reached).
    Closed { conn: u64 },
}

/// A listener that forwards every connection to `target` and reports what
/// passes through. Call [`IppTap::subscribe`] before [`IppTap::serve`].
pub struct IppTap {
    listener: TcpListener,
    target: String,
    events: broadcast::Sender<TapEvent>,
}

impl IppTap {
    /// Listen on `listen`, forwarding to `target` (e.g. `"127.0.0.1:1294"`,
    /// resolved on each new connection).
    pub async fn bind(listen: impl ToSocketAddrs, target: impl Into<String>) -> Result<Self> {
        let listener = TcpListener::bind(listen).await?;
        let (events, _) = broadcast::channel(EVENT_BUFFER);
        Ok(IppTap {
            listener,
            target: target.into(),
            events,
        })
    }

    /// The actual bound address — useful when binding to port 0 in tests.
    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.listener.local_addr()?)
    }

    /// Receive every event from now on. Subscribe before `serve` consumes
    /// the tap; a slow subscriber may miss events rather than slow traffic.
    pub fn subscribe(&self) -> broadcast::Receiver<TapEvent> {
        self.events.subscribe()
    }

    /// Accept connections forever, relaying each on its own task.
    pub async fn serve(self) -> Result<()> {
        let mut next_conn = 1u64;
        loop {
            let (client, peer) = self.listener.accept().await?;
            let conn = next_conn;
            next_conn += 1;
            let target = self.target.clone();
            let events = self.events.clone();
            tokio::spawn(async move {
                match TcpStream::connect(&target).await {
                    Ok(server) => {
                        debug!(%peer, conn, "tap: relaying");
                        let _ = events.send(TapEvent::Opened { conn, peer });
                        relay(conn, client, server, &events).await;
                    }
                    Err(e) => warn!(%peer, %target, error = %e, "tap: cannot reach target"),
                }
                let _ = events.send(TapEvent::Closed { conn });
            });
        }
    }
}

async fn relay(
    conn: u64,
    client: TcpStream,
    server: TcpStream,
    events: &broadcast::Sender<TapEvent>,
) {
    let (client_r, client_w) = client.into_split();
    let (server_r, server_w) = server.into_split();
    tokio::join!(
        pump(
            conn,
            TapDirection::ClientToServer,
            client_r,
            server_w,
            events
        ),
        pump(
            conn,
            TapDirection::ServerToClient,
            server_r,
            client_w,
            events
        ),
    );
}

/// Copy `from` to `to`, reporting each line, until `from` closes or either
/// side errors; then close `to` so the peer sees the end of the stream.
async fn pump(
    conn: u64,
    direction: TapDirection,
    mut from: impl AsyncRead + Unpin,
    mut to: impl AsyncWrite + Unpin,
    events: &broadcast::Sender<TapEvent>,
) {
    let mut lines = LineSplitter::default();
    let mut buf = [0u8; 4096];
    loop {
        let n = match from.read(&mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        if to.write_all(&buf[..n]).await.is_err() {
            break;
        }
        for line in lines.push(&buf[..n]) {
            let _ = events.send(TapEvent::Line {
                conn,
                direction,
                line,
            });
        }
    }
    // A final line with no terminator is still worth showing.
    if let Some(line) = lines.finish() {
        let _ = events.send(TapEvent::Line {
            conn,
            direction,
            line,
        });
    }
    let _ = to.shutdown().await;
}

/// Splits a byte stream into lines on `\n`, dropping a preceding `\r`.
#[derive(Default)]
struct LineSplitter {
    pending: Vec<u8>,
}

impl LineSplitter {
    fn push(&mut self, bytes: &[u8]) -> Vec<String> {
        let mut out = Vec::new();
        for &b in bytes {
            if b == b'\n' {
                out.push(self.take_line());
            } else {
                self.pending.push(b);
            }
        }
        out
    }

    fn finish(&mut self) -> Option<String> {
        (!self.pending.is_empty()).then(|| self.take_line())
    }

    fn take_line(&mut self) -> String {
        if self.pending.last() == Some(&b'\r') {
            self.pending.pop();
        }
        let line = String::from_utf8_lossy(&self.pending).into_owned();
        self.pending.clear();
        line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_lines_across_chunks_and_strips_cr() {
        let mut s = LineSplitter::default();
        assert!(s.push(b"00001 Start").is_empty());
        assert_eq!(
            s.push(b"Session()\r\n00002 Ho"),
            vec!["00001 StartSession()"]
        );
        assert_eq!(s.push(b"me()\n"), vec!["00002 Home()"]);
        assert_eq!(s.finish(), None);
    }

    #[test]
    fn keeps_an_unterminated_final_line() {
        let mut s = LineSplitter::default();
        s.push(b"no newline");
        assert_eq!(s.finish().as_deref(), Some("no newline"));
    }
}
