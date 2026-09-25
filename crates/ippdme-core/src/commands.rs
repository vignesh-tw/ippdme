//! Strongly-typed wrappers over the generic [`Term`] AST for the initial set
//! of supported I++ DME commands. These are ergonomic sugar: any command not
//! covered here can still be built and parsed via [`Term`] directly, and
//! `Command::Raw` carries through anything unrecognized.

use crate::ast::Term;
use crate::error::IppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CoordSystem {
    /// Machine Coordinate System.
    Mcs,
    /// Part Coordinate System.
    Pcs,
}

impl CoordSystem {
    fn as_ident(&self) -> &'static str {
        match self {
            CoordSystem::Mcs => "MCS",
            CoordSystem::Pcs => "PCS",
        }
    }
}

/// The coordinate systems selectable via `GetCsyTransformation`/
/// `SetCsyTransformation` (spec 6.3.3.3 / 6.3.3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CsyTransformKind {
    PartCsy,
    JogDisplayCsy,
    JogMoveCsy,
    SensorCsy,
    MoveableMachineCsy,
    MultipleArmCsy,
}

impl CsyTransformKind {
    fn as_ident(&self) -> &'static str {
        match self {
            CsyTransformKind::PartCsy => "PartCsy",
            CsyTransformKind::JogDisplayCsy => "JogDisplayCsy",
            CsyTransformKind::JogMoveCsy => "JogMoveCsy",
            CsyTransformKind::SensorCsy => "SensorCsy",
            CsyTransformKind::MoveableMachineCsy => "MoveableMachineCsy",
            CsyTransformKind::MultipleArmCsy => "MultipleArmCsy",
        }
    }

    fn from_ident(s: &str) -> Option<Self> {
        Some(match s {
            "PartCsy" => CsyTransformKind::PartCsy,
            "JogDisplayCsy" => CsyTransformKind::JogDisplayCsy,
            "JogMoveCsy" => CsyTransformKind::JogMoveCsy,
            "SensorCsy" => CsyTransformKind::SensorCsy,
            "MoveableMachineCsy" => CsyTransformKind::MoveableMachineCsy,
            "MultipleArmCsy" => CsyTransformKind::MultipleArmCsy,
            _ => return None,
        })
    }
}

/// The `(X0, Y0, Z0, Theta, Psi, Phi)` Euler-angle transformation of a
/// coordinate system relative to the machine coordinate system (spec 6.3.3,
/// "Transformation chain").
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CsyTransform {
    pub x0: f64,
    pub y0: f64,
    pub z0: f64,
    pub theta: f64,
    pub psi: f64,
    pub phi: f64,
}

impl CsyTransform {
    pub fn new(x0: f64, y0: f64, z0: f64, theta: f64, psi: f64, phi: f64) -> Self {
        CsyTransform {
            x0,
            y0,
            z0,
            theta,
            psi,
            phi,
        }
    }

    fn to_args(self) -> Vec<Term> {
        vec![
            Term::Number(self.x0),
            Term::Number(self.y0),
            Term::Number(self.z0),
            Term::Number(self.theta),
            Term::Number(self.psi),
            Term::Number(self.phi),
        ]
    }

    fn from_terms(terms: &[Term]) -> Option<Self> {
        let n = |t: &Term| match t {
            Term::Number(v) => Some(*v),
            _ => None,
        };
        Some(CsyTransform {
            x0: n(terms.first()?)?,
            y0: n(terms.get(1)?)?,
            z0: n(terms.get(2)?)?,
            theta: n(terms.get(3)?)?,
            psi: n(terms.get(4)?)?,
            phi: n(terms.get(5)?)?,
        })
    }
}

/// A point in 3D space with an optional surface normal vector, used by
/// [`Command::GoTo`] and [`Command::PtMeas`].
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Point {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
    pub i: Option<f64>,
    pub j: Option<f64>,
    pub k: Option<f64>,
}

impl Point {
    pub fn xyz(x: f64, y: f64, z: f64) -> Self {
        Point {
            x: Some(x),
            y: Some(y),
            z: Some(z),
            ..Default::default()
        }
    }

    pub fn with_normal(mut self, i: f64, j: f64, k: f64) -> Self {
        self.i = Some(i);
        self.j = Some(j);
        self.k = Some(k);
        self
    }

    fn to_args(self) -> Vec<Term> {
        let mut args = Vec::new();
        let mut push = |name: &str, v: Option<f64>| {
            if let Some(v) = v {
                args.push(Term::Call(name.to_string(), vec![Term::Number(v)]));
            }
        };
        push("X", self.x);
        push("Y", self.y);
        push("Z", self.z);
        push("I", self.i);
        push("J", self.j);
        push("K", self.k);
        args
    }

    fn from_term(term: &Term) -> Self {
        Point {
            x: term.get_num_param("X"),
            y: term.get_num_param("Y"),
            z: term.get_num_param("Z"),
            i: term.get_num_param("I"),
            j: term.get_num_param("J"),
            k: term.get_num_param("K"),
        }
    }
}

/// Strongly-typed commands sent from a client to a CMM/gateway.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Command {
    StartSession,
    EndSession,
    GetDmeVersion,
    Home,
    GoTo(Point),
    PtMeas(Point),
    SetCoordSystem(CoordSystem),
    OnMoveArc,
    ScanOnCircle,

    // --- Server methods (spec 6.3.1) ---
    /// Stop a daemon (e.g. one started by `OnMoveReportE`) by its event tag.
    StopDaemon(u32),
    StopAllDaemons,
    /// Abort all pending transactions and, if possible, the current one.
    AbortE,
    /// Retrieve the human-readable text for a given error number.
    GetErrorInfo(u32),
    ClearAllErrors,
    /// Query one or more properties, e.g. `Tool.PtMeasPar.Speed()`.
    GetProp(Vec<Term>),
    /// High-priority (fast queue) variant of [`Command::GetProp`].
    GetPropE(Vec<Term>),
    /// Set one or more properties, e.g. `Tool.PtMeasPar.Speed(100)`.
    SetProp(Vec<Term>),
    /// Query the direct children of a property/object, e.g. `Tool.GoToPar()`.
    EnumProp(Vec<Term>),
    /// Query all (grand-)children of a property/object, e.g. `Tool()`.
    EnumAllProp(Vec<Term>),

    // --- DME methods (spec 6.3.2) ---
    IsHomed,
    EnableUser,
    DisableUser,
    IsUserEnabled,
    GetMachineClass,
    /// Fast-queue error status query.
    GetErrStatusE,
    GetXtdErrStatus,
    /// Query axis positions and/or dynamic properties, e.g. `X(), Y(), Z()`.
    Get(Vec<Term>),
    /// Define which fields the server reports after a completed `PtMeas`.
    OnPtMeasReport(Vec<Term>),
    /// Start a daemon reporting machine position while it moves.
    OnMoveReportE(Vec<Term>),

    // --- CartCMM methods (spec 6.3.3) ---
    GetCoordSystem,
    GetCsyTransformation(CsyTransformKind),
    SetCsyTransformation(CsyTransformKind, CsyTransform),
    /// Save the currently active work piece coordinate system under a name.
    SaveActiveCoordSystem(String),
    /// Load a previously saved work piece coordinate system by name and make
    /// it the active one.
    LoadCoordSystem(String),
    DeleteCoordSystem(String),
    EnumCoordSystems,
    GetNamedCsyTransformation(String),
    SaveNamedCsyTransformation(String, CsyTransform),

    /// Any command not covered by a typed variant above.
    Raw(Term),
}

impl Command {
    pub fn go_to(x: f64, y: f64, z: f64) -> Self {
        Command::GoTo(Point::xyz(x, y, z))
    }

    pub fn pt_meas() -> Self {
        Command::PtMeas(Point::default())
    }
}

impl From<Command> for Term {
    fn from(cmd: Command) -> Term {
        match cmd {
            Command::StartSession => Term::unit("StartSession"),
            Command::EndSession => Term::unit("EndSession"),
            Command::GetDmeVersion => Term::unit("GetDMEVersion"),
            Command::Home => Term::unit("Home"),
            Command::GoTo(p) => Term::call("GoTo", p.to_args()),
            Command::PtMeas(p) => Term::call("PtMeas", p.to_args()),
            Command::SetCoordSystem(cs) => Term::call(
                "SetCoordSystem",
                vec![Term::Ident(cs.as_ident().to_string())],
            ),
            Command::OnMoveArc => Term::unit("OnMoveArc"),
            Command::ScanOnCircle => Term::unit("ScanOnCircle"),

            Command::StopDaemon(tag) => Term::call("StopDaemon", vec![Term::Number(tag as f64)]),
            Command::StopAllDaemons => Term::unit("StopAllDaemons"),
            Command::AbortE => Term::unit("AbortE"),
            Command::GetErrorInfo(n) => Term::call("GetErrorInfo", vec![Term::Number(n as f64)]),
            Command::ClearAllErrors => Term::unit("ClearAllErrors"),
            Command::GetProp(args) => Term::call("GetProp", args),
            Command::GetPropE(args) => Term::call("GetPropE", args),
            Command::SetProp(args) => Term::call("SetProp", args),
            Command::EnumProp(args) => Term::call("EnumProp", args),
            Command::EnumAllProp(args) => Term::call("EnumAllProp", args),

            Command::IsHomed => Term::unit("IsHomed"),
            Command::EnableUser => Term::unit("EnableUser"),
            Command::DisableUser => Term::unit("DisableUser"),
            Command::IsUserEnabled => Term::unit("IsUserEnabled"),
            Command::GetMachineClass => Term::unit("GetMachineClass"),
            Command::GetErrStatusE => Term::unit("GetErrStatusE"),
            Command::GetXtdErrStatus => Term::unit("GetXtdErrStatus"),
            Command::Get(args) => Term::call("Get", args),
            Command::OnPtMeasReport(args) => Term::call("OnPtMeasReport", args),
            Command::OnMoveReportE(args) => Term::call("OnMoveReportE", args),

            Command::GetCoordSystem => Term::unit("GetCoordSystem"),
            Command::GetCsyTransformation(kind) => Term::call(
                "GetCsyTransformation",
                vec![Term::Ident(kind.as_ident().to_string())],
            ),
            Command::SetCsyTransformation(kind, xform) => {
                let mut args = vec![Term::Ident(kind.as_ident().to_string())];
                args.extend(xform.to_args());
                Term::call("SetCsyTransformation", args)
            }
            Command::SaveActiveCoordSystem(name) => {
                Term::call("SaveActiveCoordSystem", vec![Term::Str(name)])
            }
            Command::LoadCoordSystem(name) => Term::call("LoadCoordSystem", vec![Term::Str(name)]),
            Command::DeleteCoordSystem(name) => {
                Term::call("DeleteCoordSystem", vec![Term::Str(name)])
            }
            Command::EnumCoordSystems => Term::unit("EnumCoordSystems"),
            Command::GetNamedCsyTransformation(name) => {
                Term::call("GetNamedCsyTransformation", vec![Term::Str(name)])
            }
            Command::SaveNamedCsyTransformation(name, xform) => {
                let mut args = vec![Term::Str(name)];
                args.extend(xform.to_args());
                Term::call("SaveNamedCsyTransformation", args)
            }

            Command::Raw(t) => t,
        }
    }
}

impl TryFrom<&Term> for Command {
    type Error = IppError;

    fn try_from(term: &Term) -> Result<Self, Self::Error> {
        let name = term
            .name()
            .ok_or_else(|| IppError::Parse("term has no callable name".into()))?;
        Ok(match name {
            "StartSession" => Command::StartSession,
            "EndSession" => Command::EndSession,
            "GetDMEVersion" => Command::GetDmeVersion,
            "Home" => Command::Home,
            "GoTo" => Command::GoTo(Point::from_term(term)),
            "PtMeas" => Command::PtMeas(Point::from_term(term)),
            "SetCoordSystem" => {
                let ident = term.args().first().and_then(|t| match t {
                    Term::Ident(s) => Some(s.as_str()),
                    _ => None,
                });
                let cs = match ident {
                    Some("MCS") => CoordSystem::Mcs,
                    Some("PCS") => CoordSystem::Pcs,
                    _ => {
                        return Err(IppError::WrongArgType {
                            func: "SetCoordSystem".into(),
                            name: "coord_system".into(),
                        })
                    }
                };
                Command::SetCoordSystem(cs)
            }
            "OnMoveArc" => Command::OnMoveArc,
            "ScanOnCircle" => Command::ScanOnCircle,

            "StopDaemon" => {
                let tag = term.args().first().and_then(|t| match t {
                    Term::Number(n) => Some(*n as u32),
                    _ => None,
                });
                match tag {
                    Some(tag) => Command::StopDaemon(tag),
                    None => {
                        return Err(IppError::WrongArgType {
                            func: "StopDaemon".into(),
                            name: "event_tag".into(),
                        })
                    }
                }
            }
            "StopAllDaemons" => Command::StopAllDaemons,
            "AbortE" => Command::AbortE,
            "GetErrorInfo" => {
                let n = term.args().first().and_then(|t| match t {
                    Term::Number(n) => Some(*n as u32),
                    _ => None,
                });
                match n {
                    Some(n) => Command::GetErrorInfo(n),
                    None => {
                        return Err(IppError::WrongArgType {
                            func: "GetErrorInfo".into(),
                            name: "error_number".into(),
                        })
                    }
                }
            }
            "ClearAllErrors" => Command::ClearAllErrors,
            "GetProp" => Command::GetProp(term.args().to_vec()),
            "GetPropE" => Command::GetPropE(term.args().to_vec()),
            "SetProp" => Command::SetProp(term.args().to_vec()),
            "EnumProp" => Command::EnumProp(term.args().to_vec()),
            "EnumAllProp" => Command::EnumAllProp(term.args().to_vec()),

            "IsHomed" => Command::IsHomed,
            "EnableUser" => Command::EnableUser,
            "DisableUser" => Command::DisableUser,
            "IsUserEnabled" => Command::IsUserEnabled,
            "GetMachineClass" => Command::GetMachineClass,
            "GetErrStatusE" => Command::GetErrStatusE,
            "GetXtdErrStatus" => Command::GetXtdErrStatus,
            "Get" => Command::Get(term.args().to_vec()),
            "OnPtMeasReport" => Command::OnPtMeasReport(term.args().to_vec()),
            "OnMoveReportE" => Command::OnMoveReportE(term.args().to_vec()),

            "GetCoordSystem" => Command::GetCoordSystem,
            "GetCsyTransformation" => {
                let kind = term.args().first().and_then(|t| match t {
                    Term::Ident(s) => CsyTransformKind::from_ident(s),
                    _ => None,
                });
                match kind {
                    Some(kind) => Command::GetCsyTransformation(kind),
                    None => {
                        return Err(IppError::WrongArgType {
                            func: "GetCsyTransformation".into(),
                            name: "enumerator".into(),
                        })
                    }
                }
            }
            "SetCsyTransformation" => {
                let args = term.args();
                let kind = args.first().and_then(|t| match t {
                    Term::Ident(s) => CsyTransformKind::from_ident(s),
                    _ => None,
                });
                match (kind, CsyTransform::from_terms(args.get(1..).unwrap_or(&[]))) {
                    (Some(kind), Some(xform)) => Command::SetCsyTransformation(kind, xform),
                    _ => {
                        return Err(IppError::WrongArgType {
                            func: "SetCsyTransformation".into(),
                            name: "enumerator/transform".into(),
                        })
                    }
                }
            }
            "SaveActiveCoordSystem" => {
                let name = term.args().first().and_then(|t| match t {
                    Term::Str(s) => Some(s.clone()),
                    _ => None,
                });
                match name {
                    Some(name) => Command::SaveActiveCoordSystem(name),
                    None => {
                        return Err(IppError::WrongArgType {
                            func: "SaveActiveCoordSystem".into(),
                            name: "name".into(),
                        })
                    }
                }
            }
            "LoadCoordSystem" => {
                let name = term.args().first().and_then(|t| match t {
                    Term::Str(s) => Some(s.clone()),
                    _ => None,
                });
                match name {
                    Some(name) => Command::LoadCoordSystem(name),
                    None => {
                        return Err(IppError::WrongArgType {
                            func: "LoadCoordSystem".into(),
                            name: "name".into(),
                        })
                    }
                }
            }
            "DeleteCoordSystem" => {
                let name = term.args().first().and_then(|t| match t {
                    Term::Str(s) => Some(s.clone()),
                    _ => None,
                });
                match name {
                    Some(name) => Command::DeleteCoordSystem(name),
                    None => {
                        return Err(IppError::WrongArgType {
                            func: "DeleteCoordSystem".into(),
                            name: "name".into(),
                        })
                    }
                }
            }
            "EnumCoordSystems" => Command::EnumCoordSystems,
            "GetNamedCsyTransformation" => {
                let name = term.args().first().and_then(|t| match t {
                    Term::Str(s) => Some(s.clone()),
                    _ => None,
                });
                match name {
                    Some(name) => Command::GetNamedCsyTransformation(name),
                    None => {
                        return Err(IppError::WrongArgType {
                            func: "GetNamedCsyTransformation".into(),
                            name: "name".into(),
                        })
                    }
                }
            }
            "SaveNamedCsyTransformation" => {
                let args = term.args();
                let name = args.first().and_then(|t| match t {
                    Term::Str(s) => Some(s.clone()),
                    _ => None,
                });
                match (name, CsyTransform::from_terms(args.get(1..).unwrap_or(&[]))) {
                    (Some(name), Some(xform)) => Command::SaveNamedCsyTransformation(name, xform),
                    _ => {
                        return Err(IppError::WrongArgType {
                            func: "SaveNamedCsyTransformation".into(),
                            name: "name/transform".into(),
                        })
                    }
                }
            }

            _ => Command::Raw(term.clone()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_message;

    #[test]
    fn go_to_round_trips() {
        let cmd = Command::go_to(10.0, 20.0, 5.0);
        let term: Term = cmd.clone().into();
        assert_eq!(term.to_string(), "GoTo(X(10.0), Y(20.0), Z(5.0))");
        let back = Command::try_from(&term).unwrap();
        assert_eq!(back, cmd);
    }

    #[test]
    fn set_coord_system_round_trips() {
        let term: Term = Command::SetCoordSystem(CoordSystem::Pcs).into();
        assert_eq!(term.to_string(), "SetCoordSystem(PCS)");
    }

    #[test]
    fn unknown_command_becomes_raw() {
        let msg = parse_message("00001 SomeFutureCommand(Foo(1.0))").unwrap();
        let cmd = Command::try_from(msg.term()).unwrap();
        assert!(matches!(cmd, Command::Raw(_)));
    }

    #[test]
    fn get_prop_round_trips_dotted_path() {
        let term: Term = Command::GetProp(vec![Term::unit("Tool.PtMeasPar.Speed")]).into();
        assert_eq!(term.to_string(), "GetProp(Tool.PtMeasPar.Speed())");
        let back = Command::try_from(&term).unwrap();
        assert_eq!(
            back,
            Command::GetProp(vec![Term::unit("Tool.PtMeasPar.Speed")])
        );
    }

    #[test]
    fn set_prop_round_trips_with_value() {
        let term: Term = Command::SetProp(vec![Term::call(
            "Tool.PtMeasPar.Speed",
            vec![Term::Number(100.0)],
        )])
        .into();
        assert_eq!(term.to_string(), "SetProp(Tool.PtMeasPar.Speed(100.0))");
    }

    #[test]
    fn stop_daemon_round_trips() {
        let term: Term = Command::StopDaemon(51).into();
        assert_eq!(term.to_string(), "StopDaemon(51.0)");
        let back = Command::try_from(&term).unwrap();
        assert_eq!(back, Command::StopDaemon(51));
    }

    #[test]
    fn get_error_info_round_trips() {
        let term: Term = Command::GetErrorInfo(511).into();
        let back = Command::try_from(&term).unwrap();
        assert_eq!(back, Command::GetErrorInfo(511));
    }

    #[test]
    fn is_homed_round_trips() {
        let term: Term = Command::IsHomed.into();
        assert_eq!(term.to_string(), "IsHomed()");
        assert_eq!(Command::try_from(&term).unwrap(), Command::IsHomed);
    }

    #[test]
    fn get_round_trips_axis_query() {
        let term: Term =
            Command::Get(vec![Term::unit("X"), Term::unit("Y"), Term::unit("Z")]).into();
        assert_eq!(term.to_string(), "Get(X(), Y(), Z())");
        let back = Command::try_from(&term).unwrap();
        assert_eq!(
            back,
            Command::Get(vec![Term::unit("X"), Term::unit("Y"), Term::unit("Z")])
        );
    }

    #[test]
    fn get_csy_transformation_round_trips() {
        let term: Term = Command::GetCsyTransformation(CsyTransformKind::PartCsy).into();
        assert_eq!(term.to_string(), "GetCsyTransformation(PartCsy)");
        let back = Command::try_from(&term).unwrap();
        assert_eq!(
            back,
            Command::GetCsyTransformation(CsyTransformKind::PartCsy)
        );
    }

    #[test]
    fn set_csy_transformation_round_trips() {
        let xform = CsyTransform::new(10.0, 20.0, 5.0, 0.0, 90.0, 0.0);
        let term: Term =
            Command::SetCsyTransformation(CsyTransformKind::MultipleArmCsy, xform).into();
        assert_eq!(
            term.to_string(),
            "SetCsyTransformation(MultipleArmCsy, 10.0, 20.0, 5.0, 0.0, 90.0, 0.0)"
        );
        let back = Command::try_from(&term).unwrap();
        assert_eq!(
            back,
            Command::SetCsyTransformation(CsyTransformKind::MultipleArmCsy, xform)
        );
    }

    #[test]
    fn save_and_load_coord_system_round_trip() {
        let term: Term = Command::SaveActiveCoordSystem("Fixture1".into()).into();
        assert_eq!(term.to_string(), "SaveActiveCoordSystem(\"Fixture1\")");
        let back = Command::try_from(&term).unwrap();
        assert_eq!(back, Command::SaveActiveCoordSystem("Fixture1".into()));

        let term: Term = Command::LoadCoordSystem("Fixture1".into()).into();
        let back = Command::try_from(&term).unwrap();
        assert_eq!(back, Command::LoadCoordSystem("Fixture1".into()));
    }

    #[test]
    fn pt_meas_parses_partial_response() {
        let msg = parse_message("00007 % PtMeas(X(10.002), Y(20.001), Z(5.000), I(0), J(0), K(1))")
            .unwrap();
        let cmd = Command::try_from(msg.term()).unwrap();
        match cmd {
            Command::PtMeas(p) => {
                assert_eq!(p.x, Some(10.002));
                assert_eq!(p.k, Some(1.0));
            }
            _ => panic!("expected PtMeas"),
        }
    }
}
