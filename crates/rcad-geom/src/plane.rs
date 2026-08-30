//! Flat surfaces.

use rcad_predicates::expansion::{expansion_sign, expansion_sum, product_is_exact, two_product};
use rcad_predicates::{
    Decision, Filtered, Interval, Point as PredicatePoint, Predicate, decide, meet_of_three_planes,
    product_is_normal,
};
use serde::{Deserialize, Serialize};

use crate::isometry::Isometry;
use crate::point::{Point, Vector};
use crate::{Error, Result};

/// The name the plane's sign predicate goes by in provenance records.
pub const SIDE_OF_PLANE: &str = "side of plane";

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
    /// What rounding can cost [`Plane::signed_distance_to`], per unit of the
    /// largest coordinate it is handed.
    ///
    /// The expression is `normal · (point − origin) − offset`. The origin is
    /// exactly zero, so subtracting it rounds nothing; what is left is three
    /// products, two additions and a subtraction.
    ///
    /// Counting halves of an ulp does not give the answer, because a rounding
    /// costs half an ulp of the *running sum*, which is bigger than any one
    /// term. Track how many roundings each term passes through instead. With
    /// `u = f64::EPSILON / 2` and terms `tᵢ = normalᵢ · pointᵢ`: `t₁` and `t₂`
    /// are rounded on the way in and then survive two additions and the
    /// subtraction, so `4u` each; `t₃` joins later and takes `3u`; the offset
    /// only meets the subtraction, so `u`. That gives
    /// `4u·(|t₁| + |t₂| + |t₃|) + u·|offset|`.
    ///
    /// A unit normal has `|n₁| + |n₂| + |n₃| ≤ √3`, so the sum of the terms is
    /// at most `√3` times the largest coordinate. Against `L`, the largest of
    /// the coordinates and the offset, the whole thing comes to
    /// `u·(4√3 + 1)·L ≈ 3.97·u·2·L`, which is under `4 · f64::EPSILON · L`.
    ///
    /// This bounds the *reading* and nothing else. It says nothing about how
    /// well the stored plane matches the one a caller meant — a normal is
    /// scaled to unit length on the way in, and that rounds too. A caller
    /// deciding how far off a plane a point may sit wants this times a slack
    /// factor of its own, and that factor belongs at the caller with its own
    /// reason (`docs/decisions.md` §15.6).
    pub const SIGNED_DISTANCE_ROUNDOFF: f64 = 4.0 * f64::EPSILON;

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

    /// Which side of the plane `point` is on: [`Decision::Pos`] in front of the
    /// normal, [`Decision::Neg`] behind it, [`Decision::Zero`] exactly on it.
    ///
    /// Same sign convention as [`Plane::signed_distance_to`], decided through
    /// the predicate cascade instead of guessed from an `f64`.
    ///
    /// # What this certifies, and what it does not
    ///
    /// This is the exact sign of `normal · point - offset` **for the four
    /// numbers this plane holds**. For a point with exact `f64` coordinates it
    /// always decides; for a constructed point whose box straddles the plane it
    /// returns [`Decision::Undecidable`], which is the honest answer.
    ///
    /// It is *not* a certified answer about the plane the caller meant. Every
    /// constructor here divides by the normal's length, and that division
    /// rounds, so the stored plane sits a rounding away from the one that was
    /// asked for. A vertex used to build a face is genuinely on the face's true
    /// plane and genuinely, provably, a rounding off the stored one — so this
    /// predicate will answer `Pos` or `Neg` with total confidence where the
    /// truthful answer is `Zero`. That is an exact answer to the wrong
    /// question, which is the failure this kernel exists to avoid
    /// (`docs/decisions.md` §2.5).
    ///
    /// **So: a caller asking "is this point on the plane this face was built
    /// from" must not ask here.** It must call
    /// [`orient3d_of_points`](rcad_predicates::orient3d_of_points) on the three
    /// points that defined the face, which never divides and is certified about
    /// the geometry the caller actually named. `Point` converts into the
    /// predicate crate's point type, so that call is a one-liner.
    ///
    /// What this *is* good for is consistency: it agrees with
    /// [`Plane::signed_distance_to`], [`Plane::project`] and every other
    /// accessor on this type, because they all use the same stored numbers. Use
    /// it to classify against a plane, not to rediscover a point that built one.
    #[must_use]
    pub fn side_of_point(&self, point: PredicatePoint) -> Decision {
        decide(&SideOfPlane {
            plane: *self,
            point,
        })
        .0
    }

    /// Where three planes meet.
    ///
    /// The answer is a constructed point: a box the true meeting point is
    /// somewhere inside. Cramer's rule needs a division, and dividing two
    /// ranges almost never lands on an `f64`, so the box has real width even
    /// when the three planes are square to each other.
    ///
    /// That width is the point of it. A predicate over a constructed point
    /// cannot fall through to exact arithmetic, so it can genuinely come back
    /// [`Decision::Undecidable`] — this is the geometry-side entry to the case
    /// `docs/decisions.md` §2.5 asks M0 to reach.
    ///
    /// # Errors
    ///
    /// [`Error::Predicate`] wrapping
    /// [`PlanesDoNotMeetInAPoint`](rcad_predicates::Error::PlanesDoNotMeetInAPoint)
    /// when the three normals lie in one plane — so the three surfaces share a
    /// line, or two of them are parallel, and there is no single meeting point —
    /// or when they are close enough to that for the determinant not to be
    /// bounded away from zero.
    pub fn meet_of_three(first: &Plane, second: &Plane, third: &Plane) -> Result<PredicatePoint> {
        Ok(meet_of_three_planes(
            first.coefficients(),
            second.coefficients(),
            third.coefficients(),
        )?)
    }
}

/// `normal · point - offset`, evaluated three ways for the cascade.
struct SideOfPlane {
    plane: Plane,
    point: PredicatePoint,
}

impl Predicate for SideOfPlane {
    fn name(&self) -> &'static str {
        SIDE_OF_PLANE
    }

    fn on_floats(&self) -> Option<Filtered> {
        let point = self.point.exact_coordinates()?;
        let normal = self.plane.normal.components();

        // The bound below is a fraction of the permanent, which only bounds
        // anything while every product keeps its relative accuracy. One of them
        // in the subnormals and this level certifies a wrong sign.
        if !(0..3).all(|axis| product_is_normal(normal[axis], point[axis])) {
            return None;
        }

        let terms = [0, 1, 2].map(|axis| normal[axis] * point[axis]);
        let offset = self.plane.offset;

        // The three terms can be of either sign and the offset cancels against
        // their sum, so the permanent adds magnitudes rather than values.
        let permanent = terms[0].abs() + terms[1].abs() + terms[2].abs() + offset.abs();

        Some(Filtered {
            value: terms[0] + terms[1] + terms[2] - offset,
            error_bound: SIDE_OF_PLANE_ERROR_BOUND * permanent,
        })
    }

    fn on_intervals(&self) -> Interval {
        let point = self.point.enclosure();
        let normal = self.plane.normal.components();
        let term = |axis: usize| Interval::exactly(normal[axis]) * point[axis];
        term(0) + term(1) + term(2) - Interval::exactly(self.plane.offset)
    }

    fn exactly(&self) -> Option<Decision> {
        let point = self.point.exact_coordinates()?;
        let normal = self.plane.normal.components();

        let mut total = [0.0f64; LONGEST_PLANE_TOTAL];
        let mut total_len = 0;
        let mut merged = [0.0f64; LONGEST_PLANE_TOTAL];

        for (coordinate, along) in point.into_iter().zip(normal) {
            if !product_is_exact(coordinate, along) {
                return None;
            }
            let (product, roundoff) = two_product(coordinate, along);
            total_len = expansion_sum(&total[..total_len], &[roundoff, product], &mut merged);
            core::mem::swap(&mut total, &mut merged);
        }

        total_len = expansion_sum(&total[..total_len], &[-self.plane.offset], &mut merged);

        match expansion_sign(&merged[..total_len]) {
            Decision::Neg => Some(Decision::Neg),
            Decision::Zero => Some(Decision::Zero),
            Decision::Pos => Some(Decision::Pos),
            // A product overflowed. Exact arithmetic that is not exact has
            // nothing to offer, so this level cannot run.
            Decision::Undecidable { .. } => None,
            other => unreachable!("unhandled decision from exact arithmetic: {other:?}"),
        }
    }
}

/// Level 1's forward error bound, as a multiple of the permanent.
///
/// `epsilon` is the `f64` unit roundoff, 2⁻⁵³.
///
/// Where the 4 comes from: follow the longest chain of roundings from an input
/// to `value` in [`SideOfPlane::on_floats`]. A term `normal · point` is one
/// rounding — both factors are inputs, so nothing is carried in. Folding the
/// three terms together takes two additions, a second and a third, and
/// subtracting the offset is a fourth. `(1 + epsilon)^4 - 1 <=
/// 4*epsilon + 6*epsilon^2 + ...` times the permanent.
///
/// The offset itself travels only the last of those, so the four covers it. The
/// `32*epsilon^2` covers the higher-order terms and the rounding of the
/// permanent; what it has to beat is about `6*epsilon`.
///
/// Every step of that argument is a *relative* one, so the bound holds only
/// while nothing overflows or falls into the subnormals.
/// [`SideOfPlane::on_floats`] enforces the bottom end product by product
/// through [`product_is_normal`], and `decide` the top end by refusing a bound
/// that is not itself a normal number.
const SIDE_OF_PLANE_ERROR_BOUND: f64 = {
    let epsilon = f64::EPSILON / 2.0;
    (4.0 + 32.0 * epsilon) * epsilon
};

/// Three exact products of two entries each, and the offset.
const LONGEST_PLANE_TOTAL: usize = 7;

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

    use rcad_predicates::{Level, orient3d};

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
    fn a_point_in_front_of_the_plane_is_positive_and_one_behind_it_is_negative() {
        let plane = plane_at_height_two();
        assert_eq!(
            plane.side_of_point(PredicatePoint::from([9.0, -9.0, 5.0])),
            Decision::Pos
        );
        assert_eq!(
            plane.side_of_point(PredicatePoint::from([0.0, 0.0, 0.0])),
            Decision::Neg
        );
    }

    #[test]
    fn a_point_clear_of_the_plane_is_settled_by_the_cheapest_level() {
        let (decision, level) = decide(&SideOfPlane {
            plane: plane_at_height_two(),
            point: PredicatePoint::from([0.0, 0.0, 100.0]),
        });
        assert_eq!(decision, Decision::Pos);
        assert_eq!(level, Level::Float);
    }

    #[test]
    fn a_point_on_a_plane_whose_numbers_are_exact_comes_out_exactly_on_it() {
        assert_eq!(
            plane_at_height_two().side_of_point(PredicatePoint::from([3.0, 4.0, 2.0])),
            Decision::Zero
        );
    }

    /// The reason [`Plane::side_of_point`]'s doc tells callers who need a
    /// certified "is this point on the face" to use `orient3d` instead.
    ///
    /// All three points define the plane, so the truthful answer for each is
    /// `Zero`, and `orient3d` — which never divides — says so. Normalising the
    /// normal moved the stored plane off them, and the predicate is certain
    /// about the plane it was given. The first point even has an `f64` distance
    /// of exactly zero while the certified sign against the stored coefficients
    /// is negative.
    #[test]
    fn the_points_that_defined_a_plane_are_not_on_the_plane_that_gets_stored() {
        let corners = [[1.0, 2.0, 3.0], [4.0, 5.0, 7.0], [2.0, 9.0, 4.0]];
        let [first, second, third] = corners.map(|[x, y, z]| Point::new(x, y, z));
        let plane = Plane::through_three_points(first, second, third).unwrap();

        assert_eq!(plane.signed_distance_to(first), 0.0);
        for corner in corners {
            assert_eq!(
                plane.side_of_point(PredicatePoint::from(corner)),
                Decision::Neg,
                "the stored plane is certain, and wrong, about {corner:?}"
            );
            assert_eq!(
                orient3d(corners[0], corners[1], corners[2], corner),
                Decision::Zero,
                "the three defining points say what is true about {corner:?}"
            );
        }
    }

    /// A point that is exactly on the plane, and level 1 used to place it in
    /// front.
    ///
    /// The stored normal is `(1, 2, 7)` divided by `sqrt(54)`, and that
    /// division happens to leave `n1` exactly `2 * n0` and `n2` exactly
    /// `7 * n0` in `f64` — the test asserts it, because the rest only follows
    /// if it holds. The offset is zero, so for the point `(-3u, -9u, 3u)` the
    /// value is
    ///
    /// ```text
    /// n0*(-3u) + n1*(-9u) + n2*(3u) = n0 * u * (-3 - 18 + 21) = 0
    /// ```
    ///
    /// exactly, whatever `n0` and `u` are. The truthful answer is
    /// [`Decision::Zero`].
    ///
    /// `u` is the smallest number `f64` has, so each of the three products is
    /// a fraction of one `u` and gets rounded to a whole number of them:
    /// `-0.409u` to `-0`, `-1.23u` to `-1u`, and `2.86u` to `3u`. Those sum to
    /// `+2u` where the truth is zero. The permanent is built from the same
    /// three rounded products, so the bound is `4.4e-16 * 4u`, which is far
    /// below one `u` and rounds to zero — and `2u > 0` certifies.
    #[test]
    fn a_point_whose_products_fall_off_the_bottom_of_the_range_is_not_certified() {
        let plane = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 2.0, 7.0))
            .expect("a normal of length sqrt(54) is a fine normal");
        let [n0, n1, n2] = plane.normal().components();
        assert_eq!(n1, 2.0 * n0, "the rest of this test rests on this");
        assert_eq!(n2, 7.0 * n0, "the rest of this test rests on this");

        let u = f64::from_bits(1);
        let decision = plane.side_of_point(PredicatePoint::from([-3.0 * u, -9.0 * u, 3.0 * u]));
        match decision {
            Decision::Pos => panic!("this point is exactly on the plane, not in front of it"),
            Decision::Neg | Decision::Zero => panic!("nothing here can prove a sign: {decision:?}"),
            Decision::Undecidable { enclosure } => assert!(enclosure > 0.0),
            other => panic!("unhandled decision: {other:?}"),
        }
    }

    /// The same point scaled up by `2^600`, as an oracle for the test above.
    ///
    /// The plane passes through the origin, so `normal · point - 0` is linear
    /// in the point: multiplying the point by a positive number multiplies the
    /// value by it and leaves the sign alone. At `2^600` the three products
    /// are ordinary normal numbers, the exact cancellation is visible, and the
    /// answer is the [`Decision::Zero`] the test above cannot prove.
    #[test]
    fn the_same_point_scaled_up_to_fit_comes_out_exactly_on_the_plane() {
        let plane = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 2.0, 7.0))
            .expect("a normal of length sqrt(54) is a fine normal");
        let big = f64::from_bits(1) * 2f64.powi(600);
        assert_eq!(
            plane.side_of_point(PredicatePoint::from([-3.0 * big, -9.0 * big, 3.0 * big])),
            Decision::Zero
        );
    }

    #[test]
    fn three_planes_that_cross_give_a_box_around_where_they_meet() {
        let meet = Plane::meet_of_three(
            &Plane::from_coefficients([1.0, 1.0, 1.0, -1.0]).unwrap(),
            &Plane::from_coefficients([1.0, -1.0, 0.0, 0.0]).unwrap(),
            &Plane::from_coefficients([0.0, 1.0, -1.0, 0.0]).unwrap(),
        )
        .unwrap();

        assert_eq!(meet.exact_coordinates(), None, "a third is not an f64");
        for side in meet.enclosure() {
            assert!(side.lo() <= 1.0 / 3.0 && 1.0 / 3.0 <= side.hi());
        }
        assert!(meet.uncertainty() > 0.0);
    }

    #[test]
    fn two_parallel_planes_have_nowhere_to_meet() {
        let outcome = Plane::meet_of_three(
            &plane_at_height_two(),
            &Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0)).unwrap(),
            &Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, 0.0)).unwrap(),
        );
        assert!(matches!(
            outcome,
            Err(Error::Predicate(
                rcad_predicates::Error::PlanesDoNotMeetInAPoint { .. }
            ))
        ));
    }

    #[test]
    fn three_planes_sharing_one_line_have_no_single_meeting_point() {
        let outcome = Plane::meet_of_three(
            &Plane::from_coefficients([1.0, 0.0, 0.0, 0.0]).unwrap(),
            &Plane::from_coefficients([0.0, 1.0, 0.0, 0.0]).unwrap(),
            &Plane::from_coefficients([1.0, 1.0, 0.0, 0.0]).unwrap(),
        );
        assert!(matches!(
            outcome,
            Err(Error::Predicate(
                rcad_predicates::Error::PlanesDoNotMeetInAPoint { .. }
            ))
        ));
    }

    /// The meeting point of the three planes lies on the first of them, and it
    /// is only known as a box. Exact arithmetic cannot help — the input itself
    /// is not exact — so the honest answer is that we cannot tell.
    #[test]
    fn a_constructed_point_on_a_plane_cannot_be_placed_on_it() {
        let plane = Plane::from_coefficients([1.0, 1.0, 1.0, -1.0]).unwrap();
        let meet = Plane::meet_of_three(
            &plane,
            &Plane::from_coefficients([1.0, -1.0, 0.0, 0.0]).unwrap(),
            &Plane::from_coefficients([0.0, 1.0, -1.0, 0.0]).unwrap(),
        )
        .unwrap();

        match plane.side_of_point(meet) {
            Decision::Neg | Decision::Zero | Decision::Pos => {
                panic!("the box is not tight enough to place this")
            }
            Decision::Undecidable { enclosure } => assert!(enclosure > 0.0),
            other => panic!("unhandled decision: {other:?}"),
        }
    }

    #[test]
    fn a_constructed_point_well_clear_of_a_plane_is_still_decided() {
        let meet = Plane::meet_of_three(
            &Plane::from_coefficients([1.0, 1.0, 1.0, -1.0]).unwrap(),
            &Plane::from_coefficients([1.0, -1.0, 0.0, 0.0]).unwrap(),
            &Plane::from_coefficients([0.0, 1.0, -1.0, 0.0]).unwrap(),
        )
        .unwrap();

        match plane_at_height_two().side_of_point(meet) {
            Decision::Neg => (),
            other => panic!("a box around (1/3, 1/3, 1/3) is well below z = 2: {other:?}"),
        }
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
