//! A virtual CMM device that speaks I++ DME over TCP, for testing clients
//! without physical hardware.

use std::net::SocketAddr;
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use ippdme_core::{response, Command, Message, Point, Tag};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_util::codec::Framed;
use tracing::{debug, warn};

use crate::codec::MessageCodec;
use crate::error::Result;

/// Simulated latency for `GoTo`, approximating real machine movement time.
const GO_TO_LATENCY: Duration = Duration::from_millis(500);

/// A mock I++ DME server. Bind it to an address, then `serve` it to accept
/// and handle connections until the process exits or the future is dropped.
pub struct IppMockServer {
    listener: TcpListener,
}

impl IppMockServer {
    pub async fn bind(addr: impl ToSocketAddrs) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        Ok(IppMockServer { listener })
    }

    /// The actual bound address — useful when binding to port 0 in tests.
    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.listener.local_addr()?)
    }

    /// Accept connections forever, handling each on its own task.
    pub async fn serve(self) -> Result<()> {
        loop {
            let (stream, peer) = self.listener.accept().await?;
            debug!(%peer, "mock server: accepted connection");
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream).await {
                    warn!(%peer, error = %e, "mock server: connection ended with error");
                }
            });
        }
    }
}

async fn handle_connection(stream: TcpStream) -> Result<()> {
    let mut framed = Framed::new(stream, MessageCodec);

    while let Some(result) = framed.next().await {
        let msg = result?;
        let Message::Command { tag, term } = msg else {
            // Servers don't expect to receive marked responses from clients.
            continue;
        };

        let response = match Command::try_from(&term) {
            Ok(cmd) => handle_command(tag, cmd).await,
            Err(_) => response::error(tag, "UnknownCommand"),
        };

        if framed.send(response).await.is_err() {
            break;
        }
    }
    Ok(())
}

async fn handle_command(tag: Tag, cmd: Command) -> Message {
    match cmd {
        Command::StartSession => response::ready(tag),
        Command::EndSession => response::ack(tag),
        Command::GetDmeVersion => {
            let term = ippdme_core::Term::Call(
                "DMEVersion".into(),
                vec![ippdme_core::Term::Str("1.4".into())],
            );
            response::data(tag, term)
        }
        Command::Home => {
            tokio::time::sleep(GO_TO_LATENCY).await;
            response::ack(tag)
        }
        Command::GoTo(_) => {
            tokio::time::sleep(GO_TO_LATENCY).await;
            response::ack(tag)
        }
        Command::PtMeas(_) => {
            // A real CMM would report the probed coordinates; simulate one.
            let measured = Point::xyz(10.002, 20.001, 5.000).with_normal(0.0, 0.0, 1.0);
            let term: ippdme_core::Term = Command::PtMeas(measured).into();
            response::data(tag, term)
        }
        Command::SetCoordSystem(cs) => {
            let _ = cs; // MCS/PCS accepted unconditionally by the mock
            response::ack(tag)
        }
        Command::OnMoveArc | Command::ScanOnCircle => response::ack(tag),

        Command::StopDaemon(_) | Command::StopAllDaemons => response::ack(tag),
        Command::AbortE => response::ack(tag),
        Command::GetErrorInfo(n) => {
            let term = ippdme_core::Term::Call(
                "GetErrorInfo".into(),
                vec![ippdme_core::Term::Str(format!("Error {n}"))],
            );
            response::data(tag, term)
        }
        Command::ClearAllErrors => response::ack(tag),
        Command::GetProp(_)
        | Command::GetPropE(_)
        | Command::EnumProp(_)
        | Command::EnumAllProp(_) => {
            // The mock doesn't model a real property tree; echo an empty result.
            response::data(tag, ippdme_core::Term::unit("Prop"))
        }
        Command::SetProp(_) => response::ack(tag),

        Command::IsHomed => {
            let term =
                ippdme_core::Term::Call("IsHomed".into(), vec![ippdme_core::Term::Number(1.0)]);
            response::data(tag, term)
        }
        Command::EnableUser | Command::DisableUser => response::ack(tag),
        Command::IsUserEnabled => {
            let term = ippdme_core::Term::Call(
                "IsUserEnabled".into(),
                vec![ippdme_core::Term::Number(1.0)],
            );
            response::data(tag, term)
        }
        Command::GetMachineClass => {
            let term = ippdme_core::Term::Call(
                "GetMachineClass".into(),
                vec![ippdme_core::Term::Ident("CartCMM".into())],
            );
            response::data(tag, term)
        }
        Command::GetErrStatusE => {
            let term =
                ippdme_core::Term::Call("ErrStatus".into(), vec![ippdme_core::Term::Number(0.0)]);
            response::data(tag, term)
        }
        Command::GetXtdErrStatus => response::data(tag, ippdme_core::Term::unit("XtdErrStatus")),
        Command::Get(_) => {
            let args = vec![
                ippdme_core::Term::Call("X".into(), vec![ippdme_core::Term::Number(10.002)]),
                ippdme_core::Term::Call("Y".into(), vec![ippdme_core::Term::Number(20.001)]),
                ippdme_core::Term::Call("Z".into(), vec![ippdme_core::Term::Number(5.000)]),
            ];
            response::data(tag, ippdme_core::Term::call("Get", args))
        }
        Command::OnPtMeasReport(_) | Command::OnMoveReportE(_) => response::ack(tag),

        Command::Raw(_) => response::error(tag, "UnknownCommand"),
    }
}

/// Convenience for tests/examples: pick an ephemeral port, bind, and return
/// the [`IppMockServer`] along with the address it's listening on.
pub async fn spawn_ephemeral() -> Result<(SocketAddr, tokio::task::JoinHandle<Result<()>>)> {
    let server = IppMockServer::bind(("127.0.0.1", 0)).await?;
    let addr = server.local_addr()?;
    let handle = tokio::spawn(server.serve());
    Ok((addr, handle))
}
