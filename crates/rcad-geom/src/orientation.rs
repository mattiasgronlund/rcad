//! Which way round a piece of geometry is used.

use serde::{Deserialize, Serialize};

/// Whether a face, edge or wire is used the way its geometry points, or the
/// other way round.
///
/// Topology reuses one surface or curve from several places, and each use picks
/// a side. Reversing twice gets you back to forward, which is what
/// [`Orientation::composed_with`] is for.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Orientation {
    /// The geometry is used the way it points.
    #[default]
    Forward,
    /// The geometry is used with its direction flipped.
    Reversed,
}

impl Orientation {
    /// The orientation you end up with after using `self` inside something that
    /// is itself oriented `outer`.
    ///
    /// Reversing twice is forward again.
    #[must_use]
    pub fn composed_with(self, outer: Orientation) -> Orientation {
        if self.is_reversed() == outer.is_reversed() {
            Orientation::Forward
        } else {
            Orientation::Reversed
        }
    }

    /// The other one.
    #[must_use]
    pub fn reversed(self) -> Orientation {
        self.composed_with(Orientation::Reversed)
    }

    /// Whether this flips the geometry.
    #[must_use]
    pub fn is_reversed(self) -> bool {
        matches!(self, Orientation::Reversed)
    }

    /// `1.0` when forward and `-1.0` when reversed.
    ///
    /// Exact in `f64`, so multiplying by it does not lose anything. The
    /// divergence-theorem volume sum uses it to give each face its sign.
    #[must_use]
    pub fn sign(self) -> f64 {
        if self.is_reversed() { -1.0 } else { 1.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reversing_twice_gets_you_back_to_forward() {
        assert_eq!(
            Orientation::Reversed.composed_with(Orientation::Reversed),
            Orientation::Forward
        );
        assert_eq!(Orientation::Reversed.reversed(), Orientation::Forward);
    }

    #[test]
    fn reversing_once_from_either_side_gives_reversed() {
        assert_eq!(
            Orientation::Forward.composed_with(Orientation::Reversed),
            Orientation::Reversed
        );
        assert_eq!(
            Orientation::Reversed.composed_with(Orientation::Forward),
            Orientation::Reversed
        );
    }

    #[test]
    fn composing_two_forwards_stays_forward() {
        assert_eq!(
            Orientation::Forward.composed_with(Orientation::Forward),
            Orientation::Forward
        );
    }

    #[test]
    fn the_sign_is_plus_one_forward_and_minus_one_reversed() {
        assert_eq!(Orientation::Forward.sign(), 1.0);
        assert_eq!(Orientation::Reversed.sign(), -1.0);
    }

    #[test]
    fn an_unstated_orientation_is_forward() {
        assert_eq!(Orientation::default(), Orientation::Forward);
    }
}
