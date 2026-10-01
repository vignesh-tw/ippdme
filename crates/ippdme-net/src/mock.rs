//! A virtual CMM device that speaks I++ DME over TCP, for testing clients
//! without physical hardware.

use std::net::SocketAddr;
use std::time::Duration;

use ippdme_core::{response, Command, CsyTransform, Message, Point, Tag, Term};
use tokio::net::ToSocketAddrs;

use crate::error::Result;
use crate::server::{Action, IppServer};

/// Simulated latency for `GoTo`, approximating real machine movement time.
const GO_TO_LATENCY: Duration = Duration::from_millis(500);

/// A mock I++ DME server. Bind it to an address, then `serve` it to accept
/// and handle connections until the process exits or the future is dropped.
pub struct IppMockServer {
    server: IppServer<MockHandler>,
}

impl IppMockServer {
    pub async fn bind(addr: impl ToSocketAddrs) -> Result<Self> {
        let server = IppServer::bind(addr, MockHandler).await?;
        Ok(IppMockServer { server })
    }

    /// Like [`IppMockServer::bind`], but serving TLS 1.3 connections.
    #[cfg(feature = "tls")]
    pub async fn bind_tls(
        addr: impl ToSocketAddrs,
        tls: crate::tls::TlsServerConfig,
    ) -> Result<Self> {
        let server = IppServer::bind_tls(addr, MockHandler, tls).await?;
        Ok(IppMockServer { server })
    }

    /// The actual bound address — useful when binding to port 0 in tests.
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.server.local_addr()
    }

    /// Accept connections forever, handling each on its own task.
    pub async fn serve(self) -> Result<()> {
        self.server.serve().await
    }
}

struct MockHandler;

impl crate::server::Handler for MockHandler {
    type Session = ();

    fn new_session(&self) {}

    async fn handle(&self, _session: &mut (), tag: Tag, term: Term) -> Action {
        let reply = match Command::try_from(&term) {
            Ok(cmd) => handle_command(tag, cmd).await,
            Err(_) => response::error(tag, "UnknownCommand"),
        };
        Action::Reply(reply)
    }
}

async fn handle_command(tag: Tag, cmd: Command) -> Message {
    match cmd {
        Command::StartSession => response::ready(tag),
        Command::EndSession => response::ack(tag),
        Command::GetDmeVersion => {
            let term = Term::Call("DMEVersion".into(), vec![Term::Str("1.4".into())]);
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
            let measured = Point::xyz(10.002, 20.001, 5.000)
                .and_then(|p| p.with_normal(0.0, 0.0, 1.0))
                .expect("constant point is valid");
            let term: Term = Command::PtMeas(measured).into();
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
            let term = Term::Call("GetErrorInfo".into(), vec![Term::Str(format!("Error {n}"))]);
            response::data(tag, term)
        }
        Command::ClearAllErrors => response::ack(tag),
        Command::GetProp(_)
        | Command::GetPropE(_)
        | Command::EnumProp(_)
        | Command::EnumAllProp(_) => {
            // The mock doesn't model a real property tree; echo an empty result.
            response::data(tag, Term::unit("Prop"))
        }
        Command::SetProp(_) => response::ack(tag),

        Command::IsHomed => {
            let term = Term::Call("IsHomed".into(), vec![Term::Number(1.0)]);
            response::data(tag, term)
        }
        Command::EnableUser | Command::DisableUser => response::ack(tag),
        Command::IsUserEnabled => {
            let term = Term::Call("IsUserEnabled".into(), vec![Term::Number(1.0)]);
            response::data(tag, term)
        }
        Command::GetMachineClass => {
            let term = Term::Call(
                "GetMachineClass".into(),
                vec![Term::Ident("CartCMM".into())],
            );
            response::data(tag, term)
        }
        Command::GetErrStatusE => {
            let term = Term::Call("ErrStatus".into(), vec![Term::Number(0.0)]);
            response::data(tag, term)
        }
        Command::GetXtdErrStatus => response::data(tag, Term::unit("XtdErrStatus")),
        Command::Get(_) => {
            let args = vec![
                Term::Call("X".into(), vec![Term::Number(10.002)]),
                Term::Call("Y".into(), vec![Term::Number(20.001)]),
                Term::Call("Z".into(), vec![Term::Number(5.000)]),
            ];
            response::data(tag, Term::call("Get", args))
        }
        Command::OnPtMeasReport(_) | Command::OnMoveReportE(_) => response::ack(tag),

        Command::GetCoordSystem => {
            let term = Term::Call("CoordSystem".into(), vec![Term::Ident("PartCsy".into())]);
            response::data(tag, term)
        }
        Command::GetCsyTransformation(_) => {
            let identity = CsyTransform::default();
            let term = Term::Call(
                "GetCsyTransformation".into(),
                vec![
                    Term::Number(identity.x0()),
                    Term::Number(identity.y0()),
                    Term::Number(identity.z0()),
                    Term::Number(identity.theta()),
                    Term::Number(identity.psi()),
                    Term::Number(identity.phi()),
                ],
            );
            response::data(tag, term)
        }
        Command::SetCsyTransformation(..) => response::ack(tag),
        Command::SaveActiveCoordSystem(_) | Command::LoadCoordSystem(_) => response::ack(tag),
        Command::DeleteCoordSystem(_) => response::ack(tag),
        Command::EnumCoordSystems => response::data(tag, Term::unit("EnumCoordSystems")),
        Command::GetNamedCsyTransformation(_) => {
            response::data(tag, Term::unit("GetNamedCsyTransformation"))
        }
        Command::SaveNamedCsyTransformation(..) => response::ack(tag),

        Command::Tool
        | Command::FoundTool
        | Command::GoToPar
        | Command::PtMeasPar
        | Command::FindTool(_)
        | Command::ChangeTool(_)
        | Command::SetTool(_) => response::ack(tag),
        // The mock reaches exactly the alignment it was asked for.
        Command::AlignTool(alignment) => {
            let term: Term = Command::AlignTool(alignment).into();
            response::data(tag, term)
        }
        Command::EnumTools => {
            let names = ["RefTool", "NoTool", "NormalTool"];
            let args = names.iter().map(|n| Term::Str((*n).into())).collect();
            response::data(tag, Term::call("EnumTools", args))
        }

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
