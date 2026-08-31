//! The patch of surface to be triangulated, and the loops that trim it.

use rcad_geom::{Deflection, Surface};

/// One closed loop of a patch boundary, in the surface's own two coordinates.
///
/// The loop closes on its own: the last point does not repeat the first.
#[derive(Clone, Debug, PartialEq)]
pub struct Loop {
    corners: Vec<[f64; 2]>,
}

impl Loop {
    /// A loop through these corners, in order.
    ///
    /// The corners are *already walked*. Whatever the caller had to do to get
    /// here — follow coedges, stand still at a pole, read the same shared
    /// points a neighbouring face will read — is done, and none of it is this
    /// crate's business.
    ///
    /// # Errors
    ///
    /// [`NotALoop::TooFewCorners`] for fewer than three, and
    /// [`NotALoop::CornerIsNotAPoint`] for a corner that is not made of finite
    /// numbers.
    pub fn through(corners: Vec<[f64; 2]>) -> Result<Self, NotALoop> {
        if corners.len() < 3 {
            return Err(NotALoop::TooFewCorners {
                count: corners.len(),
            });
        }
        for (position, corner) in corners.iter().enumerate() {
            if !corner.iter().all(|value| value.is_finite()) {
                return Err(NotALoop::CornerIsNotAPoint { position });
            }
        }
        Ok(Self { corners })
    }

    /// The corners, in order.
    #[must_use]
    pub fn corners(&self) -> &[[f64; 2]] {
        &self.corners
    }
}

/// A patch of one surface, trimmed by its loops, waiting to be triangulated.
///
/// The first loop is the outer one and the rest are holes, which is the same
/// order a face stores its wires in.
#[derive(Clone, Debug, PartialEq)]
pub struct TrimmedDomain {
    surface: Surface,
    outer: Loop,
    holes: Vec<Loop>,
    deflection: Deflection,
}

impl TrimmedDomain {
    /// A patch of `surface` inside `outer`, with `holes` taken out of it, to be
    /// triangulated no further than `deflection` from the true surface.
    #[must_use]
    pub fn new(surface: Surface, outer: Loop, holes: Vec<Loop>, deflection: Deflection) -> Self {
        Self {
            surface,
            outer,
            holes,
            deflection,
        }
    }

    /// The surface the patch lies on.
    #[must_use]
    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    /// The outer boundary.
    #[must_use]
    pub fn outer(&self) -> &Loop {
        &self.outer
    }

    /// The holes.
    #[must_use]
    pub fn holes(&self) -> &[Loop] {
        &self.holes
    }

    /// How far the triangles may stray from the surface.
    ///
    /// An argument the caller passed, never a setting read from somewhere
    /// else — see [`Deflection`] for why that matters.
    #[must_use]
    pub fn deflection(&self) -> Deflection {
        self.deflection
    }
}

/// Why a list of corners is not a loop.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum NotALoop {
    /// Fewer than three corners, which encloses nothing.
    #[error("a loop needs at least three corners, and this has {count}")]
    TooFewCorners {
        /// How many were given.
        count: usize,
    },
    /// A corner is not made of finite numbers.
    #[error("the corner at {position} is not a point")]
    CornerIsNotAPoint {
        /// Which corner.
        position: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcad_geom::{Axis, Cylinder, Scale};

    fn a_square() -> Vec<[f64; 2]> {
        vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
    }

    #[test]
    fn a_loop_needs_three_corners_to_enclose_anything() {
        for corners in [vec![], vec![[0.0, 0.0]], vec![[0.0, 0.0], [1.0, 0.0]]] {
            let count = corners.len();
            assert_eq!(
                Loop::through(corners),
                Err(NotALoop::TooFewCorners { count })
            );
        }
        assert!(Loop::through(a_square()).is_ok());
    }

    #[test]
    fn a_corner_that_is_not_a_point_is_refused() {
        for (position, bad) in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY]
            .into_iter()
            .enumerate()
        {
            let mut corners = a_square();
            corners[position][1] = bad;
            assert_eq!(
                Loop::through(corners),
                Err(NotALoop::CornerIsNotAPoint { position })
            );
        }
    }

    #[test]
    fn a_domain_hands_back_what_it_was_built_from() {
        let pipe = Cylinder::new(Axis::z(), 2.0).expect("a pipe of radius two");
        let deflection = Deflection::for_scale(Scale::millimetres(100.0).expect("a working size"));
        let domain = TrimmedDomain::new(
            Surface::from(pipe),
            Loop::through(a_square()).expect("a square"),
            Vec::new(),
            deflection,
        );

        assert_eq!(domain.surface(), &Surface::from(pipe));
        assert_eq!(domain.outer().corners().len(), 4);
        assert!(domain.holes().is_empty());
        assert_eq!(domain.deflection(), deflection);
    }
}
