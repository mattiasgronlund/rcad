//! Ball surfaces.

use serde::{Deserialize, Serialize};

use crate::isometry::Isometry;
use crate::point::Point;
use crate::{Error, Result};

/// A ball's skin: every point exactly `radius` away from `centre`.
///
/// Ships as a type only. Nothing in M0 builds a face on one
/// (`docs/decisions.md` §3.1 keeps M0 planar), and the answer to "is this point
/// inside, on, or outside" is a predicate that returns a `Decision`, not a
/// number — it arrives with the predicate cascade and it goes here. There is
/// deliberately no `signed_distance_to` on this type: an `f64` version would be
/// a second, quieter answer to exactly the question the predicate exists to
/// answer, and it would be thrown away when the predicate lands.
///
/// Serializes as `(centre, radius)` and is re-checked on the way in.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "(Point, f64)", into = "(Point, f64)")]
pub struct Sphere {
    centre: Point,
    radius: f64,
}

impl Sphere {
    /// The ball of the given radius around `centre`.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `centre` is not made of finite numbers, and
    /// [`Error::RadiusNotPositive`] if the radius is zero, negative, or not a
    /// finite number.
    pub fn new(centre: Point, radius: f64) -> Result<Self> {
        if !centre.is_finite() {
            return Err(Error::NotFinite {
                what: "sphere centre",
            });
        }
        if !(radius.is_finite() && radius > 0.0) {
            return Err(Error::RadiusNotPositive { radius });
        }
        Ok(Self { centre, radius })
    }

    /// The point in the middle.
    #[must_use]
    pub fn centre(&self) -> Point {
        self.centre
    }

    /// How far the surface is from the centre. Always greater than zero.
    #[must_use]
    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// The same ball after `movement` is applied. A rigid movement cannot
    /// change the radius.
    #[must_use]
    pub fn transformed(&self, movement: &Isometry) -> Self {
        Self {
            centre: movement.apply_to_point(self.centre),
            radius: self.radius,
        }
    }
}

impl TryFrom<(Point, f64)> for Sphere {
    type Error = Error;

    fn try_from((centre, radius): (Point, f64)) -> Result<Self> {
        Self::new(centre, radius)
    }
}

impl From<Sphere> for (Point, f64) {
    fn from(sphere: Sphere) -> Self {
        (sphere.centre, sphere.radius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vector;

    #[test]
    fn a_ball_with_no_radius_is_not_a_ball() {
        assert_eq!(
            Sphere::new(Point::origin(), 0.0),
            Err(Error::RadiusNotPositive { radius: 0.0 })
        );
    }

    #[test]
    fn a_ball_with_a_negative_radius_is_refused() {
        assert_eq!(
            Sphere::new(Point::origin(), -1.0),
            Err(Error::RadiusNotPositive { radius: -1.0 })
        );
    }

    #[test]
    fn a_ball_centred_on_a_coordinate_that_is_not_a_number_is_refused() {
        assert_eq!(
            Sphere::new(Point::new(f64::NAN, 0.0, 0.0), 1.0),
            Err(Error::NotFinite {
                what: "sphere centre"
            })
        );
    }

    #[test]
    fn moving_a_ball_moves_its_centre_and_leaves_its_radius_alone() {
        let ball = Sphere::new(Point::new(1.0, 0.0, 0.0), 3.0).unwrap();
        let shift = Isometry::translation(Vector::new(0.0, 5.0, 0.0)).unwrap();
        let moved = ball.transformed(&shift);
        assert_eq!(moved.centre(), Point::new(1.0, 5.0, 0.0));
        assert_eq!(moved.radius(), 3.0);
    }
}
