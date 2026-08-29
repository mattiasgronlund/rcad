//! Flat surfaces.

use serde::{Deserialize, Serialize};

use crate::isometry::Isometry;
use crate::point::{Point, Vector};
use crate::{Error, Result};

/// A flat surface: every point `p` on it satisfies `normal · p = offset`.
///
/// The normal always has length one, so [`Plane::signed_distance_to`] is a real
/// distance and not a scaled one.
///
/// This is the only surface that takes part in topology in M0
/// (`docs/decisions.md` §3.1): a real cube, with volume, area and centroid
/// summed face by face from the divergence theorem.
///
/// Serializes as its four [`Plane::coefficients`], and is re-checked and scaled
/// back to a unit normal on the way in. Scaling a normal that is already very
/// nearly unit length can move the last bit or two, so a plane that has been
/// written and read back is the same plane to within rounding, not bit for bit.
///
/// # Signs here are arithmetic, not decisions
///
/// [`Plane::signed_distance_to`] is three multiplications and two additions.
/// Near the plane its sign is not trustworthy, and nothing in this crate is
/// allowed to branch on it. The certified "which side is this point on" answer
/// is a predicate returning a `Decision`, and it arrives with the predicate
/// cascade — [`Plane::coefficients`] is the shape it will consume, and it is
/// also what a three-plane intersection wants.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "[f64; 4]", into = "[f64; 4]")]
pub struct Plane {
    normal: Vector,
    offset: f64,
}

impl Plane {
    /// The plane through `point` at right angles to `normal`.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `point` is not made of finite numbers or if the
    /// plane would sit so far from the origin that its offset overflows, and
    /// [`Error::ZeroLength`] if `normal` has no length to point along.
    pub fn from_point_and_normal(point: Point, normal: Vector) -> Result<Self> {
        if !point.is_finite() {
            return Err(Error::NotFinite {
                what: "point on a plane",
            });
        }
        let normal = normal.normalized_as("plane normal")?;
        let offset = normal.dot(point - Point::origin());
        if !offset.is_finite() {
            return Err(Error::NotFinite {
                what: "distance from the origin to a plane",
            });
        }
        Ok(Self { normal, offset })
    }

    /// The plane through three points, with the normal following the right-hand
    /// rule around `first`, `second`, `third`.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if any point is not made of finite numbers, and
    /// [`Error::PointsAreInALine`] if the three points give no plane at all.
    ///
    /// That last check only catches the cross product coming out exactly zero.
    /// Three points that are *nearly* in a line still give a plane here, and a
    /// badly conditioned one. Deciding "nearly" needs the exact orientation
    /// predicate, which is a `Decision` and arrives with the predicate cascade;
    /// picking a threshold here instead would be a second, quieter answer to the
    /// same question.
    pub fn through_three_points(first: Point, second: Point, third: Point) -> Result<Self> {
        for point in [first, second, third] {
            if !point.is_finite() {
                return Err(Error::NotFinite {
                    what: "point on a plane",
                });
            }
        }
        let normal = (second - first).cross(third - first);
        if normal == Vector::zero() {
            return Err(Error::PointsAreInALine);
        }
        Self::from_point_and_normal(first, normal)
    }

    /// The plane `a * x + b * y + c * z + d = 0`, from `[a, b, c, d]`.
    ///
    /// The coefficients are scaled so the normal has length one, which changes
    /// the numbers but not the plane.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if a coefficient is not a finite number, and
    /// [`Error::ZeroLength`] if `a`, `b` and `c` are all zero, which describes
    /// no plane.
    pub fn from_coefficients([a, b, c, d]: [f64; 4]) -> Result<Self> {
        if !d.is_finite() {
            return Err(Error::NotFinite {
                what: "plane coefficient",
            });
        }
        let direction = Vector::new(a, b, c);
        let length = direction.length();
        let normal = direction.normalized_as("plane normal")?;
        let offset = -d / length;
        if !offset.is_finite() {
            return Err(Error::NotFinite {
                what: "distance from the origin to a plane",
            });
        }
        Ok(Self { normal, offset })
    }

    /// Which way the plane faces. Always has length one.
    #[must_use]
    pub fn normal(&self) -> Vector {
        self.normal
    }

    /// How far the plane sits from the origin, measured along the normal.
    ///
    /// Positive when the origin is on the back of the plane.
    #[must_use]
    pub fn offset(&self) -> f64 {
        self.offset
    }

    /// `[a, b, c, d]` for `a * x + b * y + c * z + d = 0`.
    ///
    /// `[a, b, c]` is the unit normal. This is the form a three-plane
    /// intersection and the side-of-plane predicate take.
    #[must_use]
    pub fn coefficients(&self) -> [f64; 4] {
        let [a, b, c] = self.normal.components();
        [a, b, c, -self.offset]
    }

    /// The point on the plane nearest the world origin.
    ///
    /// Also the corner that [`Plane::plane_coordinates_of`] measures from.
    #[must_use]
    pub fn origin(&self) -> Point {
        Point::origin() + self.normal * self.offset
    }

    /// How far `point` sits in front of the plane. Negative behind it.
    ///
    /// The size of this is a real distance. Its *sign*, close to zero, is not
    /// something to trust — see the note on [`Plane`].
    #[must_use]
    pub fn signed_distance_to(&self, point: Point) -> f64 {
        self.normal.dot(point - Point::origin()) - self.offset
    }

    /// The point on the plane nearest to `point`.
    #[must_use]
    pub fn project(&self, point: Point) -> Point {
        point - self.normal * self.signed_distance_to(point)
    }

    /// Two directions in the plane, at right angles to each other and to the
    /// normal, with `u` crossed into `v` giving the normal back.
    ///
    /// The choice is arbitrary but fixed: the same plane always gives the same
    /// pair, so a face parameterised today matches one parameterised tomorrow.
    ///
    /// # Panics
    ///
    /// If the stored normal has no length. The constructors do not allow that.
    #[must_use]
    pub fn basis(&self) -> (Vector, Vector) {
        let [x, y, z] = self.normal.components().map(f64::abs);
        let least_aligned_axis = if x <= y && x <= z {
            Vector::new(1.0, 0.0, 0.0)
        } else if y <= z {
            Vector::new(0.0, 1.0, 0.0)
        } else {
            Vector::new(0.0, 0.0, 1.0)
        };
        let u = self
            .normal
            .cross(least_aligned_axis)
            .normalized_as("direction in a plane")
            .expect("a unit normal is never parallel to its least aligned coordinate axis");
        (u, self.normal.cross(u))
    }

    /// Where `point` lands in the plane's own two coordinates.
    ///
    /// Points off the plane are projected onto it first, so this throws away
    /// however far off the plane the point was.
    #[must_use]
    pub fn plane_coordinates_of(&self, point: Point) -> (f64, f64) {
        let (u, v) = self.basis();
        let across = point - self.origin();
        (across.dot(u), across.dot(v))
    }

    /// The point at the given two coordinates in the plane.
    ///
    /// Undoes [`Plane::plane_coordinates_of`] for points that were on the plane.
    #[must_use]
    pub fn point_at(&self, along_u: f64, along_v: f64) -> Point {
        let (u, v) = self.basis();
        self.origin() + u * along_u + v * along_v
    }

    /// The same flat surface facing the other way.
    ///
    /// Same set of points, opposite normal, so every signed distance flips sign.
    #[must_use]
    pub fn reversed(&self) -> Self {
        Self {
            normal: -self.normal,
            offset: -self.offset,
        }
    }

    /// The same plane after `movement` is applied.
    ///
    /// # Panics
    ///
    /// If the moved normal no longer has any length. A rotation cannot do that,
    /// so reaching this means the plane or the movement was already broken.
    #[must_use]
    pub fn transformed(&self, movement: &Isometry) -> Self {
        let normal = movement
            .apply_to_vector(self.normal)
            .normalized_as("moved plane normal")
            .expect("a rotation cannot turn a normal into a zero-length vector");
        let on_the_plane = movement.apply_to_point(self.origin());
        Self {
            normal,
            offset: normal.dot(on_the_plane - Point::origin()),
        }
    }
}

impl TryFrom<[f64; 4]> for Plane {
    type Error = Error;

    fn try_from(coefficients: [f64; 4]) -> Result<Self> {
        Self::from_coefficients(coefficients)
    }
}

impl From<Plane> for [f64; 4] {
    fn from(plane: Plane) -> Self {
        plane.coefficients()
    }
}

#[cfg(test)]
mod tests {
    use core::f64::consts::FRAC_PI_2;

    use super::*;
    use crate::Axis;

    fn plane_at_height_two() -> Plane {
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 2.0), Vector::new(0.0, 0.0, 1.0)).unwrap()
    }

    #[test]
    fn a_point_in_front_of_the_plane_is_a_positive_distance_away() {
        assert_eq!(
            plane_at_height_two().signed_distance_to(Point::new(9.0, -9.0, 5.0)),
            3.0
        );
    }

    #[test]
    fn a_point_behind_the_plane_is_a_negative_distance_away() {
        assert_eq!(
            plane_at_height_two().signed_distance_to(Point::origin()),
            -2.0
        );
    }

    #[test]
    fn the_normal_is_scaled_to_length_one_whatever_the_caller_hands_over() {
        let plane =
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 7.0)).unwrap();
        assert_eq!(plane.normal(), Vector::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn a_plane_needs_a_normal_that_has_some_length() {
        assert_eq!(
            Plane::from_point_and_normal(Point::origin(), Vector::zero()),
            Err(Error::ZeroLength {
                what: "plane normal"
            })
        );
    }

    #[test]
    fn projecting_a_point_puts_it_on_the_plane() {
        let plane = plane_at_height_two();
        let landed = plane.project(Point::new(3.0, 4.0, 100.0));
        assert_eq!(landed, Point::new(3.0, 4.0, 2.0));
        assert_eq!(plane.signed_distance_to(landed), 0.0);
    }

    #[test]
    fn three_points_in_a_line_do_not_make_a_plane() {
        assert_eq!(
            Plane::through_three_points(
                Point::origin(),
                Point::new(1.0, 1.0, 1.0),
                Point::new(3.0, 3.0, 3.0)
            ),
            Err(Error::PointsAreInALine)
        );
    }

    #[test]
    fn three_points_wound_anticlockwise_give_a_normal_pointing_at_you() {
        let plane = Plane::through_three_points(
            Point::origin(),
            Point::new(1.0, 0.0, 0.0),
            Point::new(0.0, 1.0, 0.0),
        )
        .unwrap();
        assert_eq!(plane.normal(), Vector::new(0.0, 0.0, 1.0));
        assert_eq!(plane.offset(), 0.0);
    }

    #[test]
    fn coefficients_are_scaled_so_the_first_three_have_length_one() {
        let plane = Plane::from_coefficients([0.0, 0.0, 2.0, -4.0]).unwrap();
        assert_eq!(plane.coefficients(), [0.0, 0.0, 1.0, -2.0]);
        assert_eq!(plane.offset(), 2.0);
    }

    #[test]
    fn coefficients_that_describe_no_plane_are_refused() {
        assert_eq!(
            Plane::from_coefficients([0.0, 0.0, 0.0, 1.0]),
            Err(Error::ZeroLength {
                what: "plane normal"
            })
        );
    }

    #[test]
    fn a_plane_survives_a_round_trip_through_its_coefficients() {
        let plane = Plane::through_three_points(
            Point::new(1.0, 0.0, 0.0),
            Point::new(0.0, 2.0, 0.0),
            Point::new(0.0, 0.0, 3.0),
        )
        .unwrap();
        let recovered = Plane::try_from(plane.coefficients()).unwrap();
        assert!((recovered.normal() - plane.normal()).length() < 1e-15);
        assert!((recovered.offset() - plane.offset()).abs() < 1e-15);
    }

    #[test]
    fn a_plane_facing_along_an_axis_comes_back_from_its_coefficients_bit_for_bit() {
        let plane = plane_at_height_two();
        assert_eq!(Plane::try_from(plane.coefficients()).unwrap(), plane);
    }

    #[test]
    fn the_two_directions_in_the_plane_are_at_right_angles_and_cross_to_the_normal() {
        for normal in [
            Vector::new(0.0, 0.0, 1.0),
            Vector::new(1.0, 0.0, 0.0),
            Vector::new(1.0, 2.0, 3.0),
            Vector::new(-5.0, 0.5, 0.25),
        ] {
            let plane = Plane::from_point_and_normal(Point::new(1.0, 2.0, 3.0), normal).unwrap();
            let (u, v) = plane.basis();
            assert!((u.length() - 1.0).abs() < 1e-15);
            assert!((v.length() - 1.0).abs() < 1e-15);
            assert!(u.dot(v).abs() < 1e-15);
            assert!((u.cross(v) - plane.normal()).length() < 1e-15);
        }
    }

    #[test]
    fn a_point_on_the_plane_comes_back_from_its_two_plane_coordinates() {
        let plane =
            Plane::from_point_and_normal(Point::new(1.0, 2.0, 3.0), Vector::new(1.0, 2.0, 3.0))
                .unwrap();
        let on_the_plane = plane.point_at(4.0, -7.0);
        let (along_u, along_v) = plane.plane_coordinates_of(on_the_plane);
        assert!((along_u - 4.0).abs() < 1e-12);
        assert!((along_v + 7.0).abs() < 1e-12);
    }

    #[test]
    fn plane_coordinates_ignore_how_far_off_the_plane_a_point_is() {
        let plane = plane_at_height_two();
        let (on, off) = (Point::new(3.0, 4.0, 2.0), Point::new(3.0, 4.0, 90.0));
        assert_eq!(
            plane.plane_coordinates_of(on),
            plane.plane_coordinates_of(off)
        );
    }

    #[test]
    fn reversing_a_plane_flips_every_signed_distance() {
        let plane = plane_at_height_two();
        let point = Point::new(1.0, 1.0, 7.0);
        assert_eq!(
            plane.reversed().signed_distance_to(point),
            -plane.signed_distance_to(point)
        );
    }

    #[test]
    fn moving_a_plane_moves_the_points_on_it_the_same_way() {
        let plane = plane_at_height_two();
        let turn = Isometry::rotation(&Axis::x(), FRAC_PI_2).unwrap();
        let moved = plane.transformed(&turn);

        let on_the_plane = plane.point_at(5.0, -3.0);
        assert!(
            moved
                .signed_distance_to(turn.apply_to_point(on_the_plane))
                .abs()
                < 1e-14
        );
        assert!((moved.normal() - Vector::new(0.0, -1.0, 0.0)).length() < 1e-15);
    }
}
