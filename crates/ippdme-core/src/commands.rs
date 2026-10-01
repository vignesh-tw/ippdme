//! Strongly-typed wrappers over the generic [`Term`] AST for the supported
//! I++ DME commands. These are ergonomic sugar: any command not covered here
//! can still be built and parsed via [`Term`] directly, and [`Command::raw`]
//! is the explicit escape hatch for sending one. Parsing is strict by
//! default: an unrecognized command is an error unless the caller opts in to
//! [`Command::from_term_lenient`].

use crate::ast::Term;
use crate::error::{IppError, Result};
pub use crate::values::{CoordSystem, CoordSystemName, CsyTransform, CsyTransformKind, Point};

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
    SaveActiveCoordSystem(CoordSystemName),
    /// Load a previously saved work piece coordinate system by name and make
    /// it the active one.
    LoadCoordSystem(CoordSystemName),
    DeleteCoordSystem(CoordSystemName),
    EnumCoordSystems,
    GetNamedCsyTransformation(CoordSystemName),
    SaveNamedCsyTransformation(CoordSystemName, CsyTransform),

    /// Any command not covered by a typed variant above. Build it with
    /// [`Command::raw`]; strict parsing never produces it.
    Raw(Term),
}

impl Command {
    /// `GoTo` to the given coordinates; fails if any is NaN or infinite.
    pub fn go_to(x: f64, y: f64, z: f64) -> Result<Self> {
        Ok(Command::GoTo(Point::xyz(x, y, z)?))
    }

    /// `PtMeas` with no target, measuring at the current position.
    pub fn pt_meas() -> Self {
        Command::PtMeas(Point::default())
    }

    /// Explicitly send a command with no typed variant. The term is not
    /// checked against the protocol spec.
    pub fn raw(term: Term) -> Self {
        Command::Raw(term)
    }

    /// Like [`Command::try_from`], but an unrecognized command becomes
    /// [`Command::Raw`] instead of an error. Commands that are recognized
    /// but have invalid arguments are still errors.
    pub fn from_term_lenient(term: &Term) -> Result<Self> {
        match Command::try_from(term) {
            Err(IppError::UnknownCommand(_)) => Ok(Command::Raw(term.clone())),
            other => other,
        }
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
                Term::call("SaveActiveCoordSystem", vec![Term::Str(name.into())])
            }
            Command::LoadCoordSystem(name) => {
                Term::call("LoadCoordSystem", vec![Term::Str(name.into())])
            }
            Command::DeleteCoordSystem(name) => {
                Term::call("DeleteCoordSystem", vec![Term::Str(name.into())])
            }
            Command::EnumCoordSystems => Term::unit("EnumCoordSystems"),
            Command::GetNamedCsyTransformation(name) => {
                Term::call("GetNamedCsyTransformation", vec![Term::Str(name.into())])
            }
            Command::SaveNamedCsyTransformation(name, xform) => {
                let mut args = vec![Term::Str(name.into())];
                args.extend(xform.to_args());
                Term::call("SaveNamedCsyTransformation", args)
            }

            Command::Raw(t) => t,
        }
    }
}

fn wrong_type(func: &str, name: &str) -> IppError {
    IppError::WrongArgType {
        func: func.into(),
        name: name.into(),
    }
}

/// A non-negative whole number that fits in `u32` (event tags, error numbers).
fn u32_arg(term: &Term, func: &str, name: &str) -> Result<u32> {
    match term.args().first() {
        Some(Term::Number(n)) if n.fract() == 0.0 && (0.0..=f64::from(u32::MAX)).contains(n) => {
            Ok(*n as u32)
        }
        _ => Err(wrong_type(func, name)),
    }
}

fn kind_arg(term: &Term, func: &str) -> Result<CsyTransformKind> {
    match term.args().first() {
        Some(Term::Ident(s)) => {
            CsyTransformKind::from_ident(s).ok_or_else(|| wrong_type(func, "enumerator"))
        }
        _ => Err(wrong_type(func, "enumerator")),
    }
}

fn name_arg(term: &Term, func: &str) -> Result<CoordSystemName> {
    match term.args().first() {
        Some(Term::Str(s)) => CoordSystemName::new(s.as_str()),
        _ => Err(wrong_type(func, "name")),
    }
}

/// The six numeric terms after the first argument, as a transformation.
fn transform_arg(term: &Term, func: &str, name: &str) -> Result<CsyTransform> {
    CsyTransform::from_terms(term.args().get(1..).unwrap_or(&[]))?
        .ok_or_else(|| wrong_type(func, name))
}

impl TryFrom<&Term> for Command {
    type Error = IppError;

    /// Strict parse: unknown command names are
    /// [`IppError::UnknownCommand`]; use [`Command::from_term_lenient`] to
    /// get [`Command::Raw`] for them instead.
    fn try_from(term: &Term) -> Result<Self> {
        let name = term
            .name()
            .ok_or_else(|| IppError::Parse("term has no callable name".into()))?;
        Ok(match name {
            "StartSession" => Command::StartSession,
            "EndSession" => Command::EndSession,
            "GetDMEVersion" => Command::GetDmeVersion,
            "Home" => Command::Home,
            "GoTo" => Command::GoTo(Point::from_term(term)?),
            "PtMeas" => Command::PtMeas(Point::from_term(term)?),
            "SetCoordSystem" => {
                let cs = match term.args().first() {
                    Some(Term::Ident(s)) => CoordSystem::from_ident(s),
                    _ => None,
                };
                Command::SetCoordSystem(cs.ok_or_else(|| wrong_type(name, "coord_system"))?)
            }
            "OnMoveArc" => Command::OnMoveArc,
            "ScanOnCircle" => Command::ScanOnCircle,

            "StopDaemon" => Command::StopDaemon(u32_arg(term, name, "event_tag")?),
            "StopAllDaemons" => Command::StopAllDaemons,
            "AbortE" => Command::AbortE,
            "GetErrorInfo" => Command::GetErrorInfo(u32_arg(term, name, "error_number")?),
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
            "GetCsyTransformation" => Command::GetCsyTransformation(kind_arg(term, name)?),
            "SetCsyTransformation" => Command::SetCsyTransformation(
                kind_arg(term, name)?,
                transform_arg(term, name, "transform")?,
            ),
            "SaveActiveCoordSystem" => Command::SaveActiveCoordSystem(name_arg(term, name)?),
            "LoadCoordSystem" => Command::LoadCoordSystem(name_arg(term, name)?),
            "DeleteCoordSystem" => Command::DeleteCoordSystem(name_arg(term, name)?),
            "EnumCoordSystems" => Command::EnumCoordSystems,
            "GetNamedCsyTransformation" => {
                Command::GetNamedCsyTransformation(name_arg(term, name)?)
            }
            "SaveNamedCsyTransformation" => Command::SaveNamedCsyTransformation(
                name_arg(term, name)?,
                transform_arg(term, name, "transform")?,
            ),

            other => return Err(IppError::UnknownCommand(other.to_string())),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_message;

    #[test]
    fn go_to_round_trips() {
        let cmd = Command::go_to(10.0, 20.0, 5.0).unwrap();
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
    fn unknown_command_is_an_error_when_strict() {
        let msg = parse_message("00001 SomeFutureCommand(Foo(1.0))").unwrap();
        assert_eq!(
            Command::try_from(msg.term()),
            Err(IppError::UnknownCommand("SomeFutureCommand".into()))
        );
    }

    #[test]
    fn unknown_command_becomes_raw_only_when_lenient() {
        let msg = parse_message("00001 SomeFutureCommand(Foo(1.0))").unwrap();
        let cmd = Command::from_term_lenient(msg.term()).unwrap();
        assert_eq!(cmd, Command::raw(msg.term().clone()));
    }

    #[test]
    fn lenient_parse_still_rejects_bad_args_of_known_commands() {
        let term = crate::parse_term_str("GetErrorInfo(\"x\")").unwrap();
        assert!(Command::from_term_lenient(&term).is_err());
    }

    #[test]
    fn event_numbers_must_be_whole_and_in_range() {
        for bad in [
            "StopDaemon(-1)",
            "StopDaemon(1.5)",
            "StopDaemon(5000000000)",
        ] {
            let term = crate::parse_term_str(bad).unwrap();
            assert!(Command::try_from(&term).is_err(), "{bad}");
        }
    }

    #[test]
    fn non_finite_coordinates_from_the_wire_are_rejected() {
        let term = crate::parse_term_str("GoTo(X(-inf), Y(0), Z(0))").unwrap();
        assert!(Command::try_from(&term).is_err());
    }

    #[test]
    fn non_unit_normal_from_the_wire_is_rejected() {
        let term = crate::parse_term_str("PtMeas(X(1), Y(2), Z(3), I(0), J(0), K(5))").unwrap();
        assert!(Command::try_from(&term).is_err());
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
        let xform = CsyTransform::new(10.0, 20.0, 5.0, 0.0, 90.0, 0.0).unwrap();
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
        let name = CoordSystemName::new("Fixture1").unwrap();
        let term: Term = Command::SaveActiveCoordSystem(name.clone()).into();
        assert_eq!(term.to_string(), "SaveActiveCoordSystem(\"Fixture1\")");
        let back = Command::try_from(&term).unwrap();
        assert_eq!(back, Command::SaveActiveCoordSystem(name.clone()));

        let term: Term = Command::LoadCoordSystem(name.clone()).into();
        let back = Command::try_from(&term).unwrap();
        assert_eq!(back, Command::LoadCoordSystem(name));
    }

    #[test]
    fn invalid_coord_system_name_from_the_wire_is_rejected() {
        let term = Term::call("LoadCoordSystem", vec![Term::Str("caf\u{e9}".into())]);
        assert!(Command::try_from(&term).is_err());
    }

    #[test]
    fn pt_meas_parses_partial_response() {
        let msg = parse_message("00007 % PtMeas(X(10.002), Y(20.001), Z(5.000), I(0), J(0), K(1))")
            .unwrap();
        let cmd = Command::try_from(msg.term()).unwrap();
        match cmd {
            Command::PtMeas(p) => {
                assert_eq!(p.x(), Some(10.002));
                assert_eq!(p.k(), Some(1.0));
            }
            _ => panic!("expected PtMeas"),
        }
    }
}
