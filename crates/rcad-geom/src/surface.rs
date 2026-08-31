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

    /// How far a point can move by going into this surface's two coordinates
    /// and back out again, as a multiple of the largest number involved.
    ///
    /// The size to multiply by is the largest magnitude among the point's three
    /// coordinates, the numbers the surface places itself with — a plane's
    /// offset, a ball's centre, a pipe's axis origin — and the radius. It is
    /// absolute in that size and never relative to the answer, for the reason
    /// [`Plane::SIGNED_DISTANCE_ROUNDOFF`] opens with: a coordinate on its way
    /// through zero still carries the rounding of the numbers it came from.
    ///
    /// Where the number comes from, on a pipe, with `u = f64::EPSILON / 2` and
    /// `L` the size above:
    ///
    /// - `point - axis origin` rounds each component, at most `u` of something
    ///   no bigger than `2L`.
    /// - Each of the three dot products against a unit direction is three
    ///   products and two sums on terms adding to at most `2√3·L`, so it is out
    ///   by about `12u·L`.
    /// - For a point on the skin the pair the angle is read from sits `radius`
    ///   from zero, so moving each of the two by `12u·L` turns the angle by
    ///   about `17u·L / radius`, and `atan2` adds a rounding of its own.
    /// - Turning back into a point multiplies that angle by the radius, which
    ///   cancels it back to `17u·L`. Then `cos` and `sin` round, and seven more
    ///   arithmetic steps each round something no bigger than about `4L`.
    ///
    /// That comes to under `100u·L`, which is `50 · f64::EPSILON · L`. The
    /// constant is five times it, and the room is for the trigonometry: Rust
    /// does not say how accurate `sin`, `cos` and `atan2` are, so this holds for
    /// a libm that is a few ulps out rather than one that is perfect.
    ///
    /// A ball's chain is the same shape with one term fewer — nothing runs
    /// along an axis — and a plane's is shorter still, with no trigonometry in
    /// it at all, so the pipe's number covers all three.
    ///
    /// The worst a search over balls and pipes across forty orders of magnitude
    /// of size can find is `2.4 · f64::EPSILON · L` — the ordinary gap between
    /// a worst case and a drawn one, because roundings mostly cancel.
    /// `a_round_trip_through_a_surfaces_own_two_coordinates.rs` measures it and
    /// prints the figure.
    ///
    /// **Turned the other way** — a coordinate handed to [`Surface::point_at`]
    /// and read back out — the same bound applies once it is converted into
    /// that coordinate, by dividing by how far a step in it actually carries
    /// you *there*. Round a pipe that is the radius. Round a ball it is the
    /// radius of the ring the point sits on, which shrinks to nothing at a
    /// pole — and that is the honest reason a pole's longitude is not
    /// determined, rather than a shortcoming of the arithmetic.
    ///
    /// What is **not** promised: bit equality in either direction, or that
    /// `coordinates_of(point_at(u, v))` is `(u, v)` for a `u` outside
    /// `(-π, π]`, which it is not, or at a pole, where it is not defined.
    pub const ROUND_TRIP_ROUNDOFF: f64 = 256.0 * f64::EPSILON;

    /// Where `point` lands in this surface's own two coordinates —
    /// [`Surface::point_at`] the other way round.
    ///
    /// Bare numbers, no [`Decision`](rcad_predicates::Decision) and no
    /// [`Result`](crate::Result). This decides nothing and cannot fail, and a
    /// `(u, v)` is a working space rather than an answer, so handing back
    /// something a caller could branch on would invite exactly the decision on
    /// projected coordinates the kernel is built not to take
    /// (`docs/decisions.md` §25.2).
    ///
    /// **It answers about the nearest point on the surface**, so a point off
    /// the surface gets its projection's coordinates. The axis of a pipe and
    /// the centre of a ball have no nearest point; the answer there is `(0, 0)`
    /// on a ball and `(0, along the axis)` on a pipe, which is what the
    /// arithmetic gives without a branch. Each surface's own `coordinates_of`
    /// says what its two coordinates mean and where they are not determined.
    #[must_use]
    pub fn coordinates_of(&self, point: Point) -> (f64, f64) {
        match self {
            Surface::Plane(plane) => plane.plane_coordinates_of(point),
            Surface::Sphere(sphere) => sphere.coordinates_of(point),
            Surface::Cylinder(cylinder) => cylinder.coordinates_of(point),
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
