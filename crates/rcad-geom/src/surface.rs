//! The surfaces a face can sit on.

use serde::{Deserialize, Serialize};

use crate::isometry::Isometry;
use crate::point::Point;
use crate::{Cylinder, Plane, Sphere};

/// The surface a face sits on.
///
/// A concrete list, not a trait object and not a type parameter. Making the
/// model generic over a `Geometry` trait would push a type parameter through
/// every signature in topology to buy a milestone boundary, and
/// `docs/decisions.md` §3.1 turned that down.
///
/// M0 built solids out of [`Plane`] only, and [`Sphere`] and [`Cylinder`] were
/// here as types so the enum was shaped right and the sign predicates had
/// somewhere to land. **Topology accepts all three now**: a round bar has a
/// face on a cylinder and a ball has one on a sphere. Cone and torus are still
/// out, because nothing builds one and nothing has a sign predicate for one.
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

    /// The point at `(u, v)` in this surface's own two coordinates.
    ///
    /// What each of the two *means* is the surface's own business — a distance
    /// across a plane, an angle round a cylinder, a latitude on a ball. The one
    /// thing they have in common is
    /// [`Surface::furthest_a_unit_step_goes`], which is what lets a mesher size
    /// a step without knowing which kind of surface it is holding.
    ///
    /// Exact only where the surface's own trigonometry is, which is nowhere
    /// interesting on a curved one. **No corner of a face is ever made this
    /// way**: corners come from exact constructions and are certified onto
    /// their surface by a predicate. This is for filling in between them, where
    /// the deviation is the thing to be bounded rather than avoided.
    #[must_use]
    pub fn point_at(&self, u: f64, v: f64) -> Point {
        match self {
            Surface::Plane(plane) => plane.point_at(u, v),
            Surface::Sphere(sphere) => sphere.point_at(u, v),
            Surface::Cylinder(cylinder) => cylinder.point_at(u, v),
        }
    }

    /// How far in space one unit of `u`, and one of `v`, can carry you.
    ///
    /// An **upper bound**, not a rate, and the difference is why it is named
    /// this way. On a plane a unit is a unit; along a cylinder a unit is a
    /// unit, and round one of radius `r` it is exactly `r`. On a ball a step in
    /// longitude covers `r` at the equator and less and less towards the poles,
    /// reaching nothing at all — so `r` is the most it can be and almost never
    /// what it is.
    ///
    /// That asymmetry is not a defect waiting to be fixed. Triangulating a
    /// cylinder in `(r·u, v)` scales its domain into an *exact* isometry,
    /// because a cylinder unrolls flat. A ball does not unroll at all, so no
    /// scaling makes its domain faithful and the crowding near its poles is
    /// real. It costs triangles rather than correctness, because what governs
    /// correctness is the proven deviation and not the shape of the triangles.
    #[must_use]
    pub fn furthest_a_unit_step_goes(&self) -> (f64, f64) {
        match self {
            Surface::Plane(_) => (1.0, 1.0),
            Surface::Sphere(sphere) => (sphere.radius(), sphere.radius()),
            Surface::Cylinder(cylinder) => (cylinder.radius(), 1.0),
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
