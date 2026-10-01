//! Validated value types used as command arguments.
//!
//! Constructors reject values the wire format can't carry or the protocol
//! doesn't allow (NaN/infinity, non-unit surface normals, names with quotes
//! or control characters), so an invalid command can't be built in the first
//! place instead of being rejected by the machine at runtime. Fields are
//! private to keep those invariants; read them through the accessors.

use crate::ast::Term;
use crate::error::{IppError, Result};

fn invalid(name: &str, reason: impl Into<String>) -> IppError {
    IppError::InvalidArgument {
        name: name.into(),
        reason: reason.into(),
    }
}

fn finite(name: &str, v: f64) -> Result<f64> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(invalid(name, format!("must be a finite number, got {v}")))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CoordSystem {
    /// Machine Coordinate System.
    Mcs,
    /// Part Coordinate System.
    Pcs,
}

impl CoordSystem {
    pub(crate) fn as_ident(&self) -> &'static str {
        match self {
            CoordSystem::Mcs => "MCS",
            CoordSystem::Pcs => "PCS",
        }
    }

    pub(crate) fn from_ident(s: &str) -> Option<Self> {
        match s {
            "MCS" => Some(CoordSystem::Mcs),
            "PCS" => Some(CoordSystem::Pcs),
            _ => None,
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
    pub(crate) fn as_ident(&self) -> &'static str {
        match self {
            CsyTransformKind::PartCsy => "PartCsy",
            CsyTransformKind::JogDisplayCsy => "JogDisplayCsy",
            CsyTransformKind::JogMoveCsy => "JogMoveCsy",
            CsyTransformKind::SensorCsy => "SensorCsy",
            CsyTransformKind::MoveableMachineCsy => "MoveableMachineCsy",
            CsyTransformKind::MultipleArmCsy => "MultipleArmCsy",
        }
    }

    pub(crate) fn from_ident(s: &str) -> Option<Self> {
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

/// The name of a saved work piece coordinate system. The wire format has no
/// string escaping, so a name must be non-empty printable ASCII without a
/// double quote, at most [`CoordSystemName::MAX_LEN`] characters.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "String", into = "String")
)]
pub struct CoordSystemName(String);

impl CoordSystemName {
    pub const MAX_LEN: usize = 255;

    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(invalid("name", "must not be empty"));
        }
        if name.len() > Self::MAX_LEN {
            return Err(invalid(
                "name",
                format!("must be at most {} characters", Self::MAX_LEN),
            ));
        }
        if let Some(c) = name
            .chars()
            .find(|c| (!c.is_ascii_graphic() && *c != ' ') || *c == '"')
        {
            return Err(invalid(
                "name",
                format!("must be printable ASCII without '\"', found {c:?}"),
            ));
        }
        Ok(CoordSystemName(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for CoordSystemName {
    type Error = IppError;

    fn try_from(name: String) -> Result<Self> {
        Self::new(name)
    }
}

impl TryFrom<&str> for CoordSystemName {
    type Error = IppError;

    fn try_from(name: &str) -> Result<Self> {
        Self::new(name)
    }
}

impl From<CoordSystemName> for String {
    fn from(name: CoordSystemName) -> String {
        name.0
    }
}

impl std::fmt::Display for CoordSystemName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The `(X0, Y0, Z0, Theta, Psi, Phi)` Euler-angle transformation of a
/// coordinate system relative to the machine coordinate system (spec 6.3.3,
/// "Transformation chain"). All six values must be finite.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "CsyTransformRepr", into = "CsyTransformRepr")
)]
pub struct CsyTransform {
    x0: f64,
    y0: f64,
    z0: f64,
    theta: f64,
    psi: f64,
    phi: f64,
}

impl CsyTransform {
    pub fn new(x0: f64, y0: f64, z0: f64, theta: f64, psi: f64, phi: f64) -> Result<Self> {
        Ok(CsyTransform {
            x0: finite("x0", x0)?,
            y0: finite("y0", y0)?,
            z0: finite("z0", z0)?,
            theta: finite("theta", theta)?,
            psi: finite("psi", psi)?,
            phi: finite("phi", phi)?,
        })
    }

    pub fn x0(&self) -> f64 {
        self.x0
    }
    pub fn y0(&self) -> f64 {
        self.y0
    }
    pub fn z0(&self) -> f64 {
        self.z0
    }
    pub fn theta(&self) -> f64 {
        self.theta
    }
    pub fn psi(&self) -> f64 {
        self.psi
    }
    pub fn phi(&self) -> f64 {
        self.phi
    }

    pub(crate) fn to_args(self) -> Vec<Term> {
        vec![
            Term::Number(self.x0),
            Term::Number(self.y0),
            Term::Number(self.z0),
            Term::Number(self.theta),
            Term::Number(self.psi),
            Term::Number(self.phi),
        ]
    }

    /// Read six numeric terms; `Ok(None)` if they aren't six numbers.
    pub(crate) fn from_terms(terms: &[Term]) -> Result<Option<Self>> {
        let n = |i: usize| match terms.get(i) {
            Some(Term::Number(v)) => Some(*v),
            _ => None,
        };
        let (Some(x0), Some(y0), Some(z0), Some(theta), Some(psi), Some(phi)) =
            (n(0), n(1), n(2), n(3), n(4), n(5))
        else {
            return Ok(None);
        };
        Self::new(x0, y0, z0, theta, psi, phi).map(Some)
    }
}

#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize)]
struct CsyTransformRepr {
    x0: f64,
    y0: f64,
    z0: f64,
    theta: f64,
    psi: f64,
    phi: f64,
}

#[cfg(feature = "serde")]
impl TryFrom<CsyTransformRepr> for CsyTransform {
    type Error = IppError;

    fn try_from(r: CsyTransformRepr) -> Result<Self> {
        Self::new(r.x0, r.y0, r.z0, r.theta, r.psi, r.phi)
    }
}

#[cfg(feature = "serde")]
impl From<CsyTransform> for CsyTransformRepr {
    fn from(t: CsyTransform) -> Self {
        CsyTransformRepr {
            x0: t.x0,
            y0: t.y0,
            z0: t.z0,
            theta: t.theta,
            psi: t.psi,
            phi: t.phi,
        }
    }
}

/// A point in 3D space with an optional surface normal vector, used by
/// [`crate::Command::GoTo`] and [`crate::Command::PtMeas`].
///
/// Coordinates must be finite. The normal is all three of `I`, `J`, `K` or
/// none, and must be a unit vector within [`Point::NORMAL_TOLERANCE`].
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "PointRepr", into = "PointRepr")
)]
pub struct Point {
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
    i: Option<f64>,
    j: Option<f64>,
    k: Option<f64>,
}

impl Point {
    /// How far a normal's length may differ from 1. Machines report
    /// directions rounded to a few decimals, so this is deliberately loose.
    pub const NORMAL_TOLERANCE: f64 = 1e-3;

    /// A point with all three coordinates set and no normal.
    pub fn xyz(x: f64, y: f64, z: f64) -> Result<Self> {
        Self::from_parts(Some(x), Some(y), Some(z), None, None, None)
    }

    /// Add a surface normal; it must be a unit vector.
    pub fn with_normal(self, i: f64, j: f64, k: f64) -> Result<Self> {
        Self::from_parts(self.x, self.y, self.z, Some(i), Some(j), Some(k))
    }

    pub(crate) fn from_parts(
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
        i: Option<f64>,
        j: Option<f64>,
        k: Option<f64>,
    ) -> Result<Self> {
        let check = |name: &str, v: Option<f64>| v.map(|v| finite(name, v)).transpose();
        let point = Point {
            x: check("X", x)?,
            y: check("Y", y)?,
            z: check("Z", z)?,
            i: check("I", i)?,
            j: check("J", j)?,
            k: check("K", k)?,
        };
        match (point.i, point.j, point.k) {
            (None, None, None) => {}
            (Some(i), Some(j), Some(k)) => {
                let len = (i * i + j * j + k * k).sqrt();
                if (len - 1.0).abs() > Self::NORMAL_TOLERANCE {
                    return Err(invalid(
                        "normal",
                        format!("I, J, K must be a unit vector, length is {len}"),
                    ));
                }
            }
            _ => return Err(invalid("normal", "I, J and K must be given together")),
        }
        Ok(point)
    }

    pub fn x(&self) -> Option<f64> {
        self.x
    }
    pub fn y(&self) -> Option<f64> {
        self.y
    }
    pub fn z(&self) -> Option<f64> {
        self.z
    }
    pub fn i(&self) -> Option<f64> {
        self.i
    }
    pub fn j(&self) -> Option<f64> {
        self.j
    }
    pub fn k(&self) -> Option<f64> {
        self.k
    }

    pub(crate) fn to_args(self) -> Vec<Term> {
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

    pub(crate) fn from_term(term: &Term) -> Result<Self> {
        Self::from_parts(
            term.get_num_param("X"),
            term.get_num_param("Y"),
            term.get_num_param("Z"),
            term.get_num_param("I"),
            term.get_num_param("J"),
            term.get_num_param("K"),
        )
    }
}

#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize)]
struct PointRepr {
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
    i: Option<f64>,
    j: Option<f64>,
    k: Option<f64>,
}

#[cfg(feature = "serde")]
impl TryFrom<PointRepr> for Point {
    type Error = IppError;

    fn try_from(r: PointRepr) -> Result<Self> {
        Self::from_parts(r.x, r.y, r.z, r.i, r.j, r.k)
    }
}

#[cfg(feature = "serde")]
impl From<Point> for PointRepr {
    fn from(p: Point) -> Self {
        PointRepr {
            x: p.x,
            y: p.y,
            z: p.z,
            i: p.i,
            j: p.j,
            k: p.k,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_rejects_non_finite_coordinates() {
        assert!(Point::xyz(f64::NAN, 0.0, 0.0).is_err());
        assert!(Point::xyz(0.0, f64::INFINITY, 0.0).is_err());
        assert!(Point::xyz(1.0, 2.0, 3.0).is_ok());
    }

    #[test]
    fn point_normal_must_be_unit_length() {
        let p = Point::xyz(1.0, 2.0, 3.0).unwrap();
        assert!(p.with_normal(0.0, 0.0, 1.0).is_ok());
        assert!(p.with_normal(0.707, 0.707, 0.0).is_ok());
        assert!(p.with_normal(0.0, 0.0, 2.0).is_err());
        assert!(p.with_normal(0.0, 0.0, 0.0).is_err());
    }

    #[test]
    fn point_normal_must_be_complete() {
        assert!(Point::from_parts(None, None, None, Some(0.0), None, Some(1.0)).is_err());
    }

    #[test]
    fn csy_transform_rejects_non_finite() {
        assert!(CsyTransform::new(0.0, 0.0, 0.0, f64::NAN, 0.0, 0.0).is_err());
        assert!(CsyTransform::new(1.0, 2.0, 3.0, 0.0, 90.0, 0.0).is_ok());
    }

    #[test]
    fn coord_system_name_validation() {
        assert!(CoordSystemName::new("Fixture 1").is_ok());
        assert!(CoordSystemName::new("").is_err());
        assert!(CoordSystemName::new("a\"b").is_err());
        assert!(CoordSystemName::new("a\r\nGoTo(X(1))").is_err());
        assert!(CoordSystemName::new("caf\u{e9}").is_err());
        assert!(CoordSystemName::new("x".repeat(256)).is_err());
    }
}
