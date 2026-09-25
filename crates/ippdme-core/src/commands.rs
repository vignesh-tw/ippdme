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
            Command::SetCoordSystem(cs) => {
                Term::call("SetCoordSystem", vec![Term::Ident(cs.as_ident().to_string())])
            }
            Command::OnMoveArc => Term::unit("OnMoveArc"),
            Command::ScanOnCircle => Term::unit("ScanOnCircle"),
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
    fn pt_meas_parses_partial_response() {
        let msg =
            parse_message("00007 % PtMeas(X(10.002), Y(20.001), Z(5.000), I(0), J(0), K(1))")
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
