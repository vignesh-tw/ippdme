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

/// How far a direction vector's length may differ from 1. Machines report
/// directions rounded to a few decimals, so this is deliberately loose.
pub const NORMAL_TOLERANCE: f64 = 1e-3;

fn check_unit(name: &str, i: f64, j: f64, k: f64) -> Result<()> {
    let len = (i * i + j * j + k * k).sqrt();
    if (len - 1.0).abs() > NORMAL_TOLERANCE {
        return Err(invalid(
            name,
            format!("I, J, K must be a unit vector, length is {len}"),
        ));
    }
    Ok(())
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

/// Check a name that goes inside a quoted string. The wire format has no
/// string escaping, so it must be non-empty printable ASCII without a double
/// quote, at most [`MAX_NAME_LEN`] characters.
fn validate_name(what: &str, name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(invalid(what, "must not be empty"));
    }
    if name.len() > MAX_NAME_LEN {
        return Err(invalid(
            what,
            format!("must be at most {MAX_NAME_LEN} characters"),
        ));
    }
    if let Some(c) = name
        .chars()
        .find(|c| (!c.is_ascii_graphic() && *c != ' ') || *c == '"')
    {
        return Err(invalid(
            what,
            format!("must be printable ASCII without '\"', found {c:?}"),
        ));
    }
    Ok(())
}

pub const MAX_NAME_LEN: usize = 255;

macro_rules! name_type {
    ($(#[$doc:meta])* $name:ident, $what:literal) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        #[cfg_attr(
            feature = "serde",
            derive(serde::Serialize, serde::Deserialize),
            serde(try_from = "String", into = "String")
        )]
        pub struct $name(String);

        impl $name {
            pub fn new(name: impl Into<String>) -> Result<Self> {
                let name = name.into();
                validate_name($what, &name)?;
                Ok($name(name))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = IppError;

            fn try_from(name: String) -> Result<Self> {
                Self::new(name)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = IppError;

            fn try_from(name: &str) -> Result<Self> {
                Self::new(name)
            }
        }

        impl From<$name> for String {
            fn from(name: $name) -> String {
                name.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

name_type!(
    /// The name of a saved work piece coordinate system (see
    /// [`validate_name`] for the allowed characters).
    CoordSystemName,
    "name"
);

name_type!(
    /// The name of a tool, as used by `FindTool`, `ChangeTool` and
    /// `SetTool` (see [`validate_name`] for the allowed characters).
    ToolName,
    "tool name"
);

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
    /// How far a normal's length may differ from 1; see [`NORMAL_TOLERANCE`].
    pub const NORMAL_TOLERANCE: f64 = NORMAL_TOLERANCE;

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
            (Some(i), Some(j), Some(k)) => check_unit("normal", i, j, k)?,
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

/// A normalized direction vector `(i, j, k)`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "UnitVectorRepr", into = "UnitVectorRepr")
)]
pub struct UnitVector {
    i: f64,
    j: f64,
    k: f64,
}

impl UnitVector {
    pub fn new(i: f64, j: f64, k: f64) -> Result<Self> {
        finite("I", i)?;
        finite("J", j)?;
        finite("K", k)?;
        check_unit("vector", i, j, k)?;
        Ok(UnitVector { i, j, k })
    }

    pub fn i(&self) -> f64 {
        self.i
    }
    pub fn j(&self) -> f64 {
        self.j
    }
    pub fn k(&self) -> f64 {
        self.k
    }
}

#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize)]
struct UnitVectorRepr {
    i: f64,
    j: f64,
    k: f64,
}

#[cfg(feature = "serde")]
impl TryFrom<UnitVectorRepr> for UnitVector {
    type Error = IppError;

    fn try_from(r: UnitVectorRepr) -> Result<Self> {
        Self::new(r.i, r.j, r.k)
    }
}

#[cfg(feature = "serde")]
impl From<UnitVector> for UnitVectorRepr {
    fn from(v: UnitVector) -> Self {
        UnitVectorRepr {
            i: v.i,
            j: v.j,
            k: v.k,
        }
    }
}

/// Arguments of `AlignTool` (spec 6.3.2.20): the main tool axis direction
/// with its maximum allowed error angle, and optionally the secondary
/// (working plane) direction with its own error angle. An error angle of zero
/// disables the check.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "ToolAlignmentRepr", into = "ToolAlignmentRepr")
)]
pub struct ToolAlignment {
    primary: UnitVector,
    alpha: f64,
    secondary: Option<(UnitVector, f64)>,
}

fn angle(name: &str, v: f64) -> Result<f64> {
    if v.is_finite() && v >= 0.0 {
        Ok(v)
    } else {
        Err(invalid(
            name,
            format!("must be a finite angle of at least 0, got {v}"),
        ))
    }
}

impl ToolAlignment {
    /// Align the main axis only.
    pub fn primary(vector: UnitVector, alpha: f64) -> Result<Self> {
        Ok(ToolAlignment {
            primary: vector,
            alpha: angle("alpha", alpha)?,
            secondary: None,
        })
    }

    /// Also align the secondary direction.
    pub fn with_secondary(self, vector: UnitVector, beta: f64) -> Result<Self> {
        Ok(ToolAlignment {
            secondary: Some((vector, angle("beta", beta)?)),
            ..self
        })
    }

    pub fn primary_vector(&self) -> UnitVector {
        self.primary
    }
    pub fn alpha(&self) -> f64 {
        self.alpha
    }
    pub fn secondary(&self) -> Option<(UnitVector, f64)> {
        self.secondary
    }

    pub(crate) fn to_args(self) -> Vec<Term> {
        let n = Term::Number;
        let mut args = vec![n(self.primary.i), n(self.primary.j), n(self.primary.k)];
        match self.secondary {
            None => args.push(n(self.alpha)),
            Some((v, beta)) => args.extend([n(v.i), n(v.j), n(v.k), n(self.alpha), n(beta)]),
        }
        args
    }

    /// Read `i1,j1,k1,alpha` or `i1,j1,k1,i2,j2,k2,alpha,beta`; `Ok(None)`
    /// if the terms aren't one of those shapes.
    pub(crate) fn from_terms(terms: &[Term]) -> Result<Option<Self>> {
        let nums: Option<Vec<f64>> = terms
            .iter()
            .map(|t| match t {
                Term::Number(v) => Some(*v),
                _ => None,
            })
            .collect();
        match nums.as_deref() {
            Some(&[i, j, k, alpha]) => Self::primary(UnitVector::new(i, j, k)?, alpha).map(Some),
            Some(&[i1, j1, k1, i2, j2, k2, alpha, beta]) => {
                Self::primary(UnitVector::new(i1, j1, k1)?, alpha)?
                    .with_secondary(UnitVector::new(i2, j2, k2)?, beta)
                    .map(Some)
            }
            _ => Ok(None),
        }
    }
}

#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize)]
struct ToolAlignmentRepr {
    primary: UnitVector,
    alpha: f64,
    secondary: Option<(UnitVector, f64)>,
}

#[cfg(feature = "serde")]
impl TryFrom<ToolAlignmentRepr> for ToolAlignment {
    type Error = IppError;

    fn try_from(r: ToolAlignmentRepr) -> Result<Self> {
        let base = Self::primary(r.primary, r.alpha)?;
        match r.secondary {
            Some((v, beta)) => base.with_secondary(v, beta),
            None => Ok(base),
        }
    }
}

#[cfg(feature = "serde")]
impl From<ToolAlignment> for ToolAlignmentRepr {
    fn from(a: ToolAlignment) -> Self {
        ToolAlignmentRepr {
            primary: a.primary,
            alpha: a.alpha,
            secondary: a.secondary,
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

    #[test]
    fn tool_name_validation() {
        assert!(ToolName::new("Conf1Tip1").is_ok());
        assert!(ToolName::new("").is_err());
        assert!(ToolName::new("T\"1").is_err());
    }

    #[test]
    fn tool_alignment_validation_and_shape() {
        let up = UnitVector::new(0.0, 0.0, 1.0).unwrap();
        assert!(UnitVector::new(0.0, 0.0, 2.0).is_err());
        assert!(ToolAlignment::primary(up, -1.0).is_err());
        assert!(ToolAlignment::primary(up, f64::NAN).is_err());
        let one = ToolAlignment::primary(up, 5.0).unwrap();
        assert_eq!(one.to_args().len(), 4);
        let two = one
            .with_secondary(UnitVector::new(1.0, 0.0, 0.0).unwrap(), 2.0)
            .unwrap();
        assert_eq!(two.to_args().len(), 8);
        assert_eq!(
            ToolAlignment::from_terms(&two.to_args()).unwrap(),
            Some(two)
        );
    }
}
