//! A virtual CMM device that speaks I++ DME over TCP, for testing clients
//! without physical hardware.
//!
//! Each connection is its own simulated machine session with a little state:
//! whether the session is started, the machine homed and the user enabled,
//! the current position, the active coordinate system and tool, and any
//! named coordinate systems saved so far. By default the mock is lenient
//! about call order (any command works at any time) but still answers from
//! that state; [`MockConfig::strict`] additionally rejects commands issued
//! out of order, the way a real machine does.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::time::Duration;

use ippdme_core::{
    response, Command, CoordSystem, CoordSystemName, CsyTransform, CsyTransformKind, Message,
    Point, Tag, Term,
};
use tokio::net::ToSocketAddrs;

use crate::error::Result;
use crate::server::{Action, Handler, IppServer};

/// Tools the mock knows about, as returned by `EnumTools()`.
const TOOLS: [&str; 3] = ["RefTool", "NoTool", "NormalTool"];

/// Where the simulated machine starts out, and what `PtMeas()` reports
/// until something moves it.
const START_POSITION: (f64, f64, f64) = (10.002, 20.001, 5.000);

/// Behavior knobs for [`IppMockServer`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MockConfig {
    /// How long `Home` and `GoTo` take, approximating machine movement.
    /// [`MockConfig::instant`] makes them immediate for fast tests.
    pub latency: Duration,
    /// Reject commands issued out of order (see [`IppMockServer`]).
    pub strict: bool,
}

impl Default for MockConfig {
    fn default() -> Self {
        MockConfig {
            latency: Duration::from_millis(500),
            strict: false,
        }
    }
}

impl MockConfig {
    /// No simulated movement time.
    pub fn instant() -> Self {
        MockConfig {
            latency: Duration::ZERO,
            ..Self::default()
        }
    }

    pub fn with_latency(mut self, latency: Duration) -> Self {
        self.latency = latency;
        self
    }

    /// Enforce call order. A command other than `StartSession` and
    /// `GetDMEVersion` before the session starts is `Error(NoSession)`;
    /// `Home`, `GoTo`, `PtMeas`, `ChangeTool` and `AlignTool` need the user
    /// enabled (`Error(UserNotEnabled)`), and `GoTo` and `PtMeas` also need
    /// the machine homed (`Error(NotHomed)`).
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }
}

/// A mock I++ DME server. Bind it to an address, then `serve` it to accept
/// and handle connections until the process exits or the future is dropped.
pub struct IppMockServer {
    server: IppServer<MockHandler>,
}

/// Builder for an [`IppMockServer`] with non-default behavior.
pub struct MockBuilder {
    config: MockConfig,
    #[cfg(feature = "tls")]
    tls: Option<crate::tls::TlsServerConfig>,
}

impl MockBuilder {
    pub fn config(mut self, config: MockConfig) -> Self {
        self.config = config;
        self
    }

    /// Serve TLS 1.3 (or mutual TLS, per `tls`) instead of plain TCP.
    #[cfg(feature = "tls")]
    pub fn tls(mut self, tls: crate::tls::TlsServerConfig) -> Self {
        self.tls = Some(tls);
        self
    }

    pub async fn bind(self, addr: impl ToSocketAddrs) -> Result<IppMockServer> {
        let handler = MockHandler {
            config: self.config,
        };
        #[cfg(feature = "tls")]
        let server = match self.tls {
            Some(tls) => IppServer::bind_tls(addr, handler, tls).await?,
            None => IppServer::bind(addr, handler).await?,
        };
        #[cfg(not(feature = "tls"))]
        let server = IppServer::bind(addr, handler).await?;
        Ok(IppMockServer { server })
    }
}

impl IppMockServer {
    /// A lenient mock with the default 500ms movement latency.
    pub async fn bind(addr: impl ToSocketAddrs) -> Result<Self> {
        Self::builder().bind(addr).await
    }

    /// Like [`IppMockServer::bind`], but serving TLS 1.3 connections.
    #[cfg(feature = "tls")]
    pub async fn bind_tls(
        addr: impl ToSocketAddrs,
        tls: crate::tls::TlsServerConfig,
    ) -> Result<Self> {
        Self::builder().tls(tls).bind(addr).await
    }

    pub fn builder() -> MockBuilder {
        MockBuilder {
            config: MockConfig::default(),
            #[cfg(feature = "tls")]
            tls: None,
        }
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

/// The simulated machine state of one connection.
struct MockSession {
    started: bool,
    homed: bool,
    user_enabled: bool,
    position: (f64, f64, f64),
    coord_system: CoordSystem,
    part_transform: CsyTransform,
    active_tool: &'static str,
    found_tool: Option<&'static str>,
    saved_coord_systems: BTreeMap<String, CsyTransform>,
}

struct MockHandler {
    config: MockConfig,
}

impl Handler for MockHandler {
    type Session = MockSession;

    fn new_session(&self) -> MockSession {
        MockSession {
            started: false,
            homed: false,
            user_enabled: false,
            position: START_POSITION,
            coord_system: CoordSystem::Pcs,
            part_transform: CsyTransform::default(),
            active_tool: "NoTool",
            found_tool: None,
            saved_coord_systems: BTreeMap::new(),
        }
    }

    async fn handle(&self, session: &mut MockSession, tag: Tag, term: Term) -> Action {
        let Ok(cmd) = Command::try_from(&term) else {
            return Action::Reply(response::error(tag, "UnknownCommand"));
        };
        if self.config.strict {
            if let Some(reason) = out_of_order(session, &cmd) {
                return Action::Reply(response::error(tag, reason));
            }
        }
        Action::Reply(self.execute(session, tag, cmd).await)
    }
}

/// Why `cmd` can't run in the session's current state, if it can't.
fn out_of_order(session: &MockSession, cmd: &Command) -> Option<&'static str> {
    if !session.started && !matches!(cmd, Command::StartSession | Command::GetDmeVersion) {
        return Some("NoSession");
    }
    let needs_user = matches!(
        cmd,
        Command::Home
            | Command::GoTo(_)
            | Command::PtMeas(_)
            | Command::ChangeTool(_)
            | Command::AlignTool(_)
    );
    if needs_user && !session.user_enabled {
        return Some("UserNotEnabled");
    }
    if matches!(cmd, Command::GoTo(_) | Command::PtMeas(_)) && !session.homed {
        return Some("NotHomed");
    }
    None
}

fn number(name: &str, v: f64) -> Term {
    Term::Call(name.into(), vec![Term::Number(v)])
}

fn known_tool(name: &str) -> Option<&'static str> {
    TOOLS.iter().copied().find(|t| *t == name)
}

fn csy_ident(cs: CoordSystem) -> &'static str {
    match cs {
        CoordSystem::Mcs => "MachineCsy",
        CoordSystem::Pcs => "PartCsy",
    }
}

fn transform_term(call: &str, t: &CsyTransform) -> Term {
    Term::call(
        call,
        [t.x0(), t.y0(), t.z0(), t.theta(), t.psi(), t.phi()]
            .map(Term::Number)
            .to_vec(),
    )
}

fn named_list(call: &str, names: impl Iterator<Item = String>) -> Term {
    Term::call(call, names.map(Term::Str).collect())
}

impl MockHandler {
    async fn travel(&self) {
        if !self.config.latency.is_zero() {
            tokio::time::sleep(self.config.latency).await;
        }
    }

    async fn execute(&self, s: &mut MockSession, tag: Tag, cmd: Command) -> Message {
        match cmd {
            Command::StartSession => {
                s.started = true;
                response::ready(tag)
            }
            Command::EndSession => {
                s.started = false;
                response::ack(tag)
            }
            Command::GetDmeVersion => {
                let term = Term::Call("DMEVersion".into(), vec![Term::Str("1.4".into())]);
                response::data(tag, term)
            }
            Command::Home => {
                self.travel().await;
                s.homed = true;
                response::ack(tag)
            }
            Command::GoTo(target) => {
                self.travel().await;
                s.position = moved(s.position, &target);
                response::ack(tag)
            }
            Command::PtMeas(target) => {
                // Measure where asked, or here if no target was given.
                s.position = moved(s.position, &target);
                let (x, y, z) = s.position;
                let normal = target
                    .i()
                    .zip(target.j())
                    .zip(target.k())
                    .map_or((0.0, 0.0, 1.0), |((i, j), k)| (i, j, k));
                let measured = Point::xyz(x, y, z)
                    .and_then(|p| p.with_normal(normal.0, normal.1, normal.2))
                    .expect("simulated position is finite and the normal is unit length");
                let term: Term = Command::PtMeas(measured).into();
                response::data(tag, term)
            }
            Command::SetCoordSystem(cs) => {
                s.coord_system = cs;
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
                let flag = f64::from(u8::from(s.homed));
                response::data(tag, Term::Call("IsHomed".into(), vec![Term::Number(flag)]))
            }
            Command::EnableUser => {
                s.user_enabled = true;
                response::ack(tag)
            }
            Command::DisableUser => {
                s.user_enabled = false;
                response::ack(tag)
            }
            Command::IsUserEnabled => {
                let flag = f64::from(u8::from(s.user_enabled));
                let term = Term::Call("IsUserEnabled".into(), vec![Term::Number(flag)]);
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
                let (x, y, z) = s.position;
                let args = vec![number("X", x), number("Y", y), number("Z", z)];
                response::data(tag, Term::call("Get", args))
            }
            Command::OnPtMeasReport(_) | Command::OnMoveReportE(_) => response::ack(tag),

            Command::Tool | Command::GoToPar | Command::PtMeasPar => response::ack(tag),
            Command::FindTool(name) => match known_tool(name.as_str()) {
                Some(tool) => {
                    s.found_tool = Some(tool);
                    response::ack(tag)
                }
                None => response::error(tag, "ToolNotFound"),
            },
            Command::FoundTool => match s.found_tool {
                Some(_) => response::ack(tag),
                None => response::error(tag, "ToolNotDefined"),
            },
            Command::ChangeTool(name) | Command::SetTool(name) => match known_tool(name.as_str()) {
                Some(tool) => {
                    s.active_tool = tool;
                    response::ack(tag)
                }
                None => response::error(tag, "ToolNotFound"),
            },
            // The mock reaches exactly the alignment it was asked for.
            Command::AlignTool(alignment) => {
                let term: Term = Command::AlignTool(alignment).into();
                response::data(tag, term)
            }
            Command::EnumTools => response::data(
                tag,
                named_list("EnumTools", TOOLS.iter().map(|t| t.to_string())),
            ),

            Command::GetCoordSystem => {
                let term = Term::Call(
                    "CoordSystem".into(),
                    vec![Term::Ident(csy_ident(s.coord_system).into())],
                );
                response::data(tag, term)
            }
            Command::GetCsyTransformation(kind) => {
                let t = if kind == CsyTransformKind::PartCsy {
                    s.part_transform
                } else {
                    CsyTransform::default()
                };
                response::data(tag, transform_term("GetCsyTransformation", &t))
            }
            Command::SetCsyTransformation(kind, t) => {
                if kind == CsyTransformKind::PartCsy {
                    s.part_transform = t;
                }
                response::ack(tag)
            }
            Command::SaveActiveCoordSystem(name) => {
                s.saved_coord_systems
                    .insert(name.to_string(), s.part_transform);
                response::ack(tag)
            }
            Command::LoadCoordSystem(name) => match saved(s, &name) {
                Some(t) => {
                    s.part_transform = t;
                    s.coord_system = CoordSystem::Pcs;
                    response::ack(tag)
                }
                None => response::error(tag, "CoordSystemNotFound"),
            },
            Command::DeleteCoordSystem(name) => match s.saved_coord_systems.remove(name.as_str()) {
                Some(_) => response::ack(tag),
                None => response::error(tag, "CoordSystemNotFound"),
            },
            Command::EnumCoordSystems => response::data(
                tag,
                named_list("EnumCoordSystems", s.saved_coord_systems.keys().cloned()),
            ),
            Command::GetNamedCsyTransformation(name) => match saved(s, &name) {
                Some(t) => response::data(tag, transform_term("GetNamedCsyTransformation", &t)),
                None => response::error(tag, "CoordSystemNotFound"),
            },
            Command::SaveNamedCsyTransformation(name, t) => {
                s.saved_coord_systems.insert(name.to_string(), t);
                response::ack(tag)
            }

            Command::Raw(_) => response::error(tag, "UnknownCommand"),
        }
    }
}

fn saved(s: &MockSession, name: &CoordSystemName) -> Option<CsyTransform> {
    s.saved_coord_systems.get(name.as_str()).copied()
}

/// `from`, with each axis `target` specifies replaced.
fn moved(from: (f64, f64, f64), target: &Point) -> (f64, f64, f64) {
    (
        target.x().unwrap_or(from.0),
        target.y().unwrap_or(from.1),
        target.z().unwrap_or(from.2),
    )
}

/// Convenience for tests/examples: pick an ephemeral port, bind, and return
/// the [`IppMockServer`] along with the address it's listening on.
pub async fn spawn_ephemeral() -> Result<(SocketAddr, tokio::task::JoinHandle<Result<()>>)> {
    spawn_ephemeral_with(MockConfig::default()).await
}

/// Like [`spawn_ephemeral`], with explicit [`MockConfig`] behavior.
pub async fn spawn_ephemeral_with(
    config: MockConfig,
) -> Result<(SocketAddr, tokio::task::JoinHandle<Result<()>>)> {
    let server = IppMockServer::builder()
        .config(config)
        .bind(("127.0.0.1", 0))
        .await?;
    let addr = server.local_addr()?;
    let handle = tokio::spawn(server.serve());
    Ok((addr, handle))
}
