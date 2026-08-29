//! The surfaces a face can sit on.

use serde::{Deserialize, Serialize};

use crate::isometry::Isometry;
use crate::{Cylinder, Plane, Sphere};

/// The surface a face sits on.
///
/// A concrete list, not a trait object and not a type parameter. Making the
/// model generic over a `Geometry` trait would push a type parameter through
/// every signature in topology to buy a milestone boundary, and
/// `docs/decisions.md` §3.1 turned that down.
///
/// M0 builds solids out of [`Plane`] only. [`Sphere`] and [`Cylinder`] are here
/// as types so the enum is shaped right and so the sign predicates have
/// somewhere to land; nothing in topology accepts them yet.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Surface {
    /// A flat surface.
    Plane(Plane),
    /// A ball's skin.
    Sphere(Sphere),
    /// An endless round pipe's skin.
    Cylinder(Cylinder),
}

impl Surface {
    /// The plane, if this is one. `None` for everything else.
    ///
    /// M0's face construction and its volume, area and centroid sums are planar
    /// only, and this is how they say so.
    #[must_use]
    pub fn as_plane(&self) -> Option<&Plane> {
        match self {
            Surface::Plane(plane) => Some(plane),
            Surface::Sphere(_) | Surface::Cylinder(_) => None,
        }
    }

    /// The same surface after `movement` is applied.
    #[must_use]
    pub fn transformed(&self, movement: &Isometry) -> Surface {
        match self {
            Surface::Plane(plane) => Surface::Plane(plane.transformed(movement)),
            Surface::Sphere(sphere) => Surface::Sphere(sphere.transformed(movement)),
            Surface::Cylinder(cylinder) => Surface::Cylinder(cylinder.transformed(movement)),
        }
    }
}

impl From<Plane> for Surface {
    fn from(plane: Plane) -> Self {
        Surface::Plane(plane)
    }
}

impl From<Sphere> for Surface {
    fn from(sphere: Sphere) -> Self {
        Surface::Sphere(sphere)
    }
}

impl From<Cylinder> for Surface {
    fn from(cylinder: Cylinder) -> Self {
        Surface::Cylinder(cylinder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Axis, Point, Vector};

    #[test]
    fn only_a_plane_answers_to_being_asked_for_a_plane() {
        let plane =
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0)).unwrap();
        assert_eq!(Surface::from(plane).as_plane(), Some(&plane));

        let ball = Sphere::new(Point::origin(), 1.0).unwrap();
        assert_eq!(Surface::from(ball).as_plane(), None);

        let pipe = Cylinder::new(Axis::z(), 1.0).unwrap();
        assert_eq!(Surface::from(pipe).as_plane(), None);
    }

    #[test]
    fn moving_a_surface_moves_whichever_surface_it_is() {
        let shift = Isometry::translation(Vector::new(0.0, 0.0, 4.0)).unwrap();

        let ball = Sphere::new(Point::origin(), 1.0).unwrap();
        let moved = Surface::from(ball).transformed(&shift);
        assert_eq!(
            moved,
            Surface::Sphere(Sphere::new(Point::new(0.0, 0.0, 4.0), 1.0).unwrap())
        );
    }
}
