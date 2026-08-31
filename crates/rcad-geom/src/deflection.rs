//! How far a mesh may stray from the surface it stands for.

use serde::{Deserialize, Serialize};

use crate::scale::Scale;
use crate::{Error, Result};

/// How far a mesh is allowed to stray from the surface it stands for.
///
/// A length, in the model's own unit. A triangle standing in for a piece of
/// curved surface is never on it, and this is the most it may be off by.
///
/// # This is not a tolerance, and it is not `Policy`
///
/// Three knobs meet around a mesh and they do different jobs. Keeping them
/// apart is the whole reason this is its own type.
///
/// - **This** decides how *fine* the mesh is. A coarse mesh is a request, not a
///   failure, and nothing judges it.
/// - **`Policy`** decides what happens when the deviation cannot be *proven* to
///   be under this. Certified refuses; tolerant hands back a tagged answer.
/// - **The model's linear tolerance** is about the geometry itself and has
///   nothing to say about triangles.
///
/// Conflating the first two is the tempting mistake: it would mean a caller who
/// asked for a rougher mesh had also, silently, asked the kernel to certify
/// less. `docs/decisions.md` §2.6 forbids exactly that shape for `Policy`, and
/// the reason carries over.
///
/// # It travels as an argument
///
/// Never a field on a model. A number that changes what an operation returns,
/// stored where the caller cannot see it at the call site, is the footgun §2.6
/// exists to keep out — and a mesh cached against one deflection and read back
/// under another is the same bug wearing a hat.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "f64", into = "f64")]
pub struct Deflection(f64);

impl Deflection {
    /// How far the default mesh may stray, as a fraction of the model's
    /// nominal size.
    ///
    /// A thousandth: for a 100 mm part, 0.1 mm.
    ///
    /// Six orders of magnitude coarser than
    /// [`Scale::RELATIVE_LINEAR_TOLERANCE`], and deliberately. They answer
    /// different questions — how well do we know where this surface *is*
    /// against how many triangles do you want for it — and a mesh refined to
    /// the tolerance would have something like a thousand times the triangles
    /// in each direction for a picture nobody could tell apart.
    pub const RELATIVE_DEFAULT: f64 = 1e-3;

    /// A deflection of exactly this many of the model's units.
    ///
    /// # Errors
    ///
    /// [`Error::DeflectionNotPositive`] if it is zero, negative, or not a
    /// finite number. Zero is refused rather than treated as "as fine as
    /// possible", because no finite mesh reaches it and a caller who asks for
    /// it deserves to be told so rather than to wait.
    pub fn of(length: f64) -> Result<Self> {
        if !(length.is_finite() && length > 0.0) {
            return Err(Error::DeflectionNotPositive { length });
        }
        Ok(Self(length))
    }

    /// The default for a model working at this scale.
    ///
    /// [`Scale::nominal_size`] times [`Deflection::RELATIVE_DEFAULT`].
    #[must_use]
    pub fn for_scale(scale: Scale) -> Self {
        Self(scale.nominal_size() * Self::RELATIVE_DEFAULT)
    }

    /// How far, in the model's own unit.
    #[must_use]
    pub fn length(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for Deflection {
    type Error = Error;

    fn try_from(length: f64) -> Result<Self> {
        Self::of(length)
    }
}

impl From<Deflection> for f64 {
    fn from(deflection: Deflection) -> Self {
        deflection.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Unit;

    #[test]
    fn a_deflection_has_to_be_a_positive_length() {
        for length in [0.0, -1.0, -0.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(
                Deflection::of(length).is_err(),
                "{length} is not a distance a mesh can stray"
            );
        }
        assert_eq!(Deflection::of(0.25).map(Deflection::length), Ok(0.25));
    }

    #[test]
    fn the_default_is_a_thousandth_of_the_nominal_size() {
        let scale = Scale::millimetres(100.0).expect("100 mm is a working size");
        assert_eq!(Deflection::for_scale(scale).length(), 0.1);
    }

    /// The relation that says these two are answering different questions.
    ///
    /// If the default deflection ever slid down towards the linear tolerance,
    /// every mesh in the kernel would grow by a factor nobody asked for, and
    /// the first sign of it would be a slow test suite rather than a wrong
    /// answer. Cheaper to state it here.
    #[test]
    fn the_default_mesh_is_far_coarser_than_the_geometry_is_trusted_to() {
        for nominal in [1.0, 100.0, 1e6, 1e-3] {
            let scale = Scale::new(Unit::Millimetre, nominal).expect("a working size");
            let deflection = Deflection::for_scale(scale).length();
            let tolerance = scale.default_linear_tolerance();
            assert!(
                deflection > tolerance * 1e5,
                "at a nominal size of {nominal} the mesh may stray {deflection} \
                 and the geometry is trusted to {tolerance}"
            );
        }
    }

    #[test]
    fn a_deflection_survives_a_round_trip_through_its_length() {
        let deflection = Deflection::of(0.125).expect("an eighth is a length");
        let written = f64::from(deflection);
        assert_eq!(Deflection::try_from(written), Ok(deflection));
    }
}
