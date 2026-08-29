//! Round pipe surfaces.

use serde::{Deserialize, Serialize};

use crate::isometry::Isometry;
use crate::point::Axis;
use crate::{Error, Result};

/// An endless round pipe's skin: every point exactly `radius` away from `axis`.
///
/// Ships as a type only, for the same reason as
/// [`Sphere`](crate::Sphere): the inside/on/outside answer is a predicate
/// returning a `Decision` and it arrives with the predicate cascade. No
/// `signed_distance_to` here on purpose.
///
/// Serializes as `(axis, radius)` and is re-checked on the way in.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "(Axis, f64)", into = "(Axis, f64)")]
pub struct Cylinder {
    axis: Axis,
    radius: f64,
}

impl Cylinder {
    /// The pipe of the given radius around `axis`.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `axis` is not made of finite numbers, and
    /// [`Error::RadiusNotPositive`] if the radius is zero, negative, or not a
    /// finite number.
    pub fn new(axis: Axis, radius: f64) -> Result<Self> {
        if !axis.origin().is_finite() || !axis.direction().is_finite() {
            return Err(Error::NotFinite {
                what: "cylinder axis",
            });
        }
        if !(radius.is_finite() && radius > 0.0) {
            return Err(Error::RadiusNotPositive { radius });
        }
        Ok(Self { axis, radius })
    }

    /// The line down the middle of the pipe.
    #[must_use]
    pub fn axis(&self) -> Axis {
        self.axis
    }

    /// How far the surface is from the axis. Always greater than zero.
    #[must_use]
    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// The same pipe after `movement` is applied. A rigid movement cannot
    /// change the radius.
    #[must_use]
    pub fn transformed(&self, movement: &Isometry) -> Self {
        Self {
            axis: self.axis.transformed(movement),
            radius: self.radius,
        }
    }
}

impl TryFrom<(Axis, f64)> for Cylinder {
    type Error = Error;

    fn try_from((axis, radius): (Axis, f64)) -> Result<Self> {
        Self::new(axis, radius)
    }
}

impl From<Cylinder> for (Axis, f64) {
    fn from(cylinder: Cylinder) -> Self {
        (cylinder.axis, cylinder.radius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Point, Vector};

    #[test]
    fn a_pipe_with_no_radius_is_not_a_pipe() {
        assert_eq!(
            Cylinder::new(Axis::z(), 0.0),
            Err(Error::RadiusNotPositive { radius: 0.0 })
        );
    }

    #[test]
    fn moving_a_pipe_moves_its_axis_and_leaves_its_radius_alone() {
        let pipe = Cylinder::new(Axis::z(), 2.0).unwrap();
        let shift = Isometry::translation(Vector::new(4.0, 0.0, 0.0)).unwrap();
        let moved = pipe.transformed(&shift);
        assert_eq!(moved.axis().origin(), Point::new(4.0, 0.0, 0.0));
        assert_eq!(moved.axis().direction(), Vector::new(0.0, 0.0, 1.0));
        assert_eq!(moved.radius(), 2.0);
    }
}
