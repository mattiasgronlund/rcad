//! Ball surfaces.

use rcad_predicates::expansion::{expansion_sign, expansion_sum, product_is_exact, two_product};
use rcad_predicates::{
    Decision, Filtered, Interval, NOTHING_IS_KNOWN, Point as PredicatePoint, Predicate,
    SideOfSurface, are_real_numbers, decide, is_a_real_point, product_is_normal,
};
use serde::{Deserialize, Serialize};

use crate::exact::square_of_expansion;
use crate::isometry::Isometry;
use crate::point::Point;
use crate::{Error, Result};

/// The name the sphere's sign predicate goes by in provenance records.
pub const SIDE_OF_SPHERE: &str = "side of sphere";

/// A ball's skin: every point exactly `radius` away from `centre`.
///
/// Ships as a type and a sign predicate. Nothing in M0 builds a face on one —
/// `docs/decisions.md` §3.1 keeps M0 planar — so [`SideOfSurface::side_of`] is
/// the whole of what it does.
///
/// There is deliberately no `signed_distance_to` here. An `f64` distance would
/// be a second, quieter answer to exactly the question the predicate exists to
/// answer, and callers would reach for the cheap one.
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

/// Inside, on, or outside, decided through the cascade.
///
/// The sign of `|point - centre|² - radius²`: [`Decision::Neg`] inside,
/// [`Decision::Pos`] outside, [`Decision::Zero`] exactly on the skin.
///
/// A point with exact `f64` coordinates always gets a certified answer, as long
/// as no product overflows or falls into the subnormals. A constructed point is
/// only known to lie inside a box, so when that box straddles the skin the
/// answer is [`Decision::Undecidable`] and no amount of precision will change
/// it (`docs/decisions.md` §2.5).
///
/// The radius is taken as written. A sphere whose radius came from a
/// calculation carries whatever rounding that calculation left in it, and this
/// predicate answers about the sphere it was handed, not the one that was meant.
///
/// A point or a sphere that is not made of real numbers gets
/// [`NOTHING_IS_KNOWN`] before the cascade runs. [`Sphere::transformed`] cannot
/// fail and can overflow, so a sphere built from finite numbers can be holding
/// an infinite centre by the time it reaches here.
///
/// Alone among the six predicates in this kernel, this one already refused such
/// input on its own, at all three levels. Level 2 works out
/// `Σ (pointᵢ − centreᵢ)² − radius²`, so a coordinate that is not a real number
/// enters only as a square inside a sum, with no zero coefficient anywhere to
/// multiply it away: the sum comes out as an enclosure of everything and the
/// honest `Undecidable` falls out of it. Level 1 hands over because its error
/// bound is then not a normal number, and level 3 because
/// [`expansion_sign`] refuses an expansion that is not finite. So the guard
/// changes no answer here today. It is here so all six refuse the same input
/// the same way, and so a later change to that expression cannot quietly
/// reopen the hole.
impl SideOfSurface for Sphere {
    fn side_of(&self, point: PredicatePoint) -> Decision {
        let sphere_is_real =
            are_real_numbers(self.centre.coordinates()) && are_real_numbers([self.radius]);
        if !(is_a_real_point(point) && sphere_is_real) {
            return NOTHING_IS_KNOWN;
        }
        decide(&HowFarFromTheSkin {
            sphere: *self,
            point,
        })
        .0
    }
}

/// `|point - centre|² - radius²`, evaluated three ways for the cascade.
struct HowFarFromTheSkin {
    sphere: Sphere,
    point: PredicatePoint,
}

impl Predicate for HowFarFromTheSkin {
    fn name(&self) -> &'static str {
        SIDE_OF_SPHERE
    }

    fn on_floats(&self) -> Option<Filtered> {
        let point = self.point.exact_coordinates()?;
        let centre = self.sphere.centre.coordinates();

        let gaps = [0, 1, 2].map(|axis| point[axis] - centre[axis]);
        let radius = self.sphere.radius;

        // The bound below is a fraction of the permanent, which only bounds
        // anything while every product keeps its relative accuracy. One of them
        // in the subnormals and this level certifies a wrong sign.
        if !(gaps.iter().all(|&gap| product_is_normal(gap, gap))
            && product_is_normal(radius, radius))
        {
            return None;
        }

        let squares = gaps.map(|gap| gap * gap);
        let radius_squared = radius * radius;

        // Every square is positive, so the only cancellation is the final
        // subtraction. The permanent is therefore the plain sum of the four
        // magnitudes.
        let permanent = squares[0] + squares[1] + squares[2] + radius_squared;

        Some(Filtered {
            value: squares[0] + squares[1] + squares[2] - radius_squared,
            error_bound: SIDE_OF_SPHERE_ERROR_BOUND * permanent,
        })
    }

    fn on_intervals(&self) -> Interval {
        let point = self.point.enclosure();
        let centre = self.sphere.centre.coordinates();
        let squared_gap = |axis: usize| {
            let gap = point[axis] - Interval::exactly(centre[axis]);
            gap * gap
        };
        let radius = Interval::exactly(self.sphere.radius);
        squared_gap(0) + squared_gap(1) + squared_gap(2) - radius * radius
    }

    fn exactly(&self) -> Option<Decision> {
        let point = self.point.exact_coordinates()?;
        let centre = self.sphere.centre.coordinates();

        let mut total = [0.0f64; LONGEST_SPHERE_TOTAL];
        let mut total_len = 0;
        let mut merged = [0.0f64; LONGEST_SPHERE_TOTAL];
        let mut squared_gap = [0.0f64; LONGEST_SQUARED_GAP];

        for (coordinate, middle) in point.into_iter().zip(centre) {
            let gap = crate::exact::difference(coordinate, middle);
            let gap_len = square_of_expansion(&gap, &mut squared_gap)?;
            total_len = expansion_sum(&total[..total_len], &squared_gap[..gap_len], &mut merged);
            core::mem::swap(&mut total, &mut merged);
        }

        let radius = self.sphere.radius;
        if !product_is_exact(radius, radius) {
            return None;
        }
        let (radius_squared, roundoff) = two_product(radius, radius);
        total_len = expansion_sum(
            &total[..total_len],
            &[-roundoff, -radius_squared],
            &mut merged,
        );

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
/// `epsilon` is the `f64` unit roundoff, 2⁻⁵³ — half an ulp of 1, and so the
/// most a single rounding can cost in relative terms.
///
/// Where the 6 comes from: follow the longest chain of roundings from an input
/// coordinate to `value` in [`HowFarFromTheSkin::on_floats`]. A gap
/// `point - centre` is one rounding. Squaring it is a second, and it multiplies
/// two copies of an already-rounded gap, so it carries a third. Folding the
/// three squares together takes two additions, a fourth and a fifth, and the
/// subtraction of the radius term is a sixth. Six roundings, each costing at
/// most a factor of `(1 + epsilon)`, come to
/// `(1 + epsilon)^6 - 1 <= 6*epsilon + 15*epsilon^2 + ...` times the permanent.
///
/// The radius term travels a shorter chain — one rounding to square it and one
/// to subtract it — so the six covers it too. The `48*epsilon^2` covers the
/// higher-order terms and the rounding of the permanent itself, with room to
/// spare: what it has to beat is about `15*epsilon`.
///
/// Every step of that argument is a *relative* one, so like every bound of this
/// shape it holds only while nothing overflows or falls into the subnormals.
/// [`HowFarFromTheSkin::on_floats`] enforces the bottom end product by product
/// through [`product_is_normal`], and `decide` the top end by refusing a bound
/// that is not itself a normal number.
const SIDE_OF_SPHERE_ERROR_BOUND: f64 = {
    let epsilon = f64::EPSILON / 2.0;
    (6.0 + 48.0 * epsilon) * epsilon
};

/// A gap between two coordinates is two entries, so its square is at most four
/// scaled pairs.
const LONGEST_SQUARED_GAP: usize = 8;

/// Three squared gaps and a squared radius, exactly.
const LONGEST_SPHERE_TOTAL: usize = 3 * LONGEST_SQUARED_GAP + 2;

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
    use rcad_predicates::{Level, decide, meet_of_three_planes};

    use super::*;
    use crate::Vector;

    fn unit_sphere() -> Sphere {
        Sphere::new(Point::origin(), 1.0).unwrap()
    }

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

    #[test]
    fn a_point_well_outside_is_positive_and_one_well_inside_is_negative() {
        assert_eq!(
            unit_sphere().side_of(PredicatePoint::from([2.0, 0.0, 0.0])),
            Decision::Pos
        );
        assert_eq!(
            unit_sphere().side_of(PredicatePoint::from([0.25, 0.25, 0.25])),
            Decision::Neg
        );
    }

    #[test]
    fn a_point_clear_of_the_skin_is_settled_by_the_cheapest_level() {
        let (decision, level) = decide(&HowFarFromTheSkin {
            sphere: unit_sphere(),
            point: PredicatePoint::from([2.0, 0.0, 0.0]),
        });
        assert_eq!(decision, Decision::Pos);
        assert_eq!(level, Level::Float);
    }

    #[test]
    fn a_point_on_a_ball_small_enough_to_see_in_f64_is_exactly_on_it() {
        assert_eq!(
            unit_sphere().side_of(PredicatePoint::from([1.0, 0.0, 0.0])),
            Decision::Zero
        );
    }

    /// Legs and hypotenuse of a Pythagorean triple big enough that the squares
    /// run past 53 bits. Every coordinate is a whole number, so the point sits
    /// exactly on the skin — but only exact arithmetic can see it.
    #[test]
    fn a_point_on_a_ball_too_big_to_see_in_f64_still_comes_out_exactly_on_it() {
        let ball = Sphere::new(Point::origin(), 198_489_546.0).unwrap();
        let on_the_skin = PredicatePoint::from([106_308_504.0, 167_620_410.0, 0.0]);

        let (decision, level) = decide(&HowFarFromTheSkin {
            sphere: ball,
            point: on_the_skin,
        });
        assert_eq!(decision, Decision::Zero);
        assert_eq!(level, Level::Exact);
    }

    /// The three planes meet at (1/3, 1/3, 1/3), which is not an `f64`, and the
    /// ball through it has radius sqrt(1/3), which is not one either. Nothing
    /// about this can be decided, and the honest answer says so.
    #[test]
    fn a_constructed_point_sitting_on_the_skin_cannot_be_placed() {
        let meet = meet_of_three_planes(
            [1.0, 1.0, 1.0, -1.0],
            [1.0, -1.0, 0.0, 0.0],
            [0.0, 1.0, -1.0, 0.0],
        )
        .unwrap();
        let ball = Sphere::new(Point::origin(), (1.0f64 / 3.0).sqrt()).unwrap();

        match ball.side_of(meet) {
            Decision::Neg | Decision::Zero | Decision::Pos => {
                panic!("neither the box nor the radius is exact enough to place this")
            }
            Decision::Undecidable { enclosure } => assert!(enclosure > 0.0),
            other => panic!("unhandled decision: {other:?}"),
        }
    }

    /// A point just outside a tiny ball, and level 1 used to place it inside.
    ///
    /// With `g = 2^-538` and a radius of `3 * 2^-539 = 1.5 * g`, the point
    /// `(g, g, g)` is `sqrt(3) * g` from the centre against a radius of
    /// `1.5 * g`, and `sqrt(3) > 1.5`, so it is outside. Exactly:
    ///
    /// ```text
    /// 3*g² - (1.5*g)² = (3 - 2.25) * 2^-1076 = 0.75 * 2^-1076 = 3/16 of a u,
    /// ```
    ///
    /// where `u = 2^-1074` is the smallest number `f64` has. The truthful
    /// answer is [`Decision::Pos`].
    ///
    /// Every one of those products is a fraction of one `u`, so `f64` rounds
    /// each to a whole number of them: the three squares `0.25u` each go to
    /// `0`, and the squared radius `0.5625u` goes to `1u`. That leaves
    /// `0 - 1u`, negative where the truth is positive. The permanent is `1u`,
    /// and the bound `6.7e-16 * 1u` rounds to zero, so `1u > 0` certifies.
    #[test]
    fn a_point_whose_squares_fall_off_the_bottom_of_the_range_is_not_certified() {
        let gap = 2f64.powi(-538);
        let ball = Sphere::new(Point::origin(), 3.0 * 2f64.powi(-539)).unwrap();

        let decision = ball.side_of(PredicatePoint::from([gap, gap, gap]));
        match decision {
            Decision::Neg => panic!("this point is outside the ball, not inside it"),
            Decision::Pos | Decision::Zero => panic!("nothing here can prove a sign: {decision:?}"),
            Decision::Undecidable { enclosure } => assert!(enclosure > 0.0),
            other => panic!("unhandled decision: {other:?}"),
        }
    }

    /// The same ball and point scaled up by `2^600`, as an oracle for the test
    /// above.
    ///
    /// The centre is the origin, so scaling the radius and the point together
    /// by a positive number multiplies `|point|² - radius²` by the square of
    /// it — positive, so the sign is untouched. At `2^600` the squares are
    /// ordinary normal numbers and the answer is the [`Decision::Pos`] the
    /// test above cannot prove.
    #[test]
    fn the_same_ball_and_point_scaled_up_to_fit_put_the_point_outside() {
        let gap = 2f64.powi(-538 + 600);
        let ball = Sphere::new(Point::origin(), 3.0 * 2f64.powi(-539 + 600)).unwrap();
        assert_eq!(
            ball.side_of(PredicatePoint::from([gap, gap, gap])),
            Decision::Pos
        );
    }

    #[test]
    fn a_constructed_point_well_inside_is_still_decided() {
        let meet = meet_of_three_planes(
            [1.0, 1.0, 1.0, -1.0],
            [1.0, -1.0, 0.0, 0.0],
            [0.0, 1.0, -1.0, 0.0],
        )
        .unwrap();

        match unit_sphere().side_of(meet) {
            Decision::Neg => (),
            other => panic!("a box around (1/3, 1/3, 1/3) is inside the unit ball: {other:?}"),
        }
    }

    /// A point that is not a point is neither in the ball nor out of it.
    ///
    /// This one predicate reached the same refusal on its own before the guard
    /// existed — see the note on the `impl` — so the test pins behaviour rather
    /// than catching a regression. What it does catch is the guard answering
    /// something *different* from what the cascade used to work out.
    #[test]
    fn a_point_that_is_not_a_point_is_neither_in_nor_out() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for axis in 0..3 {
                let mut broken = [0.0, 0.0, 5.0];
                broken[axis] = bad;
                assert_eq!(
                    unit_sphere().side_of(PredicatePoint::from(broken)),
                    Decision::Undecidable {
                        enclosure: f64::INFINITY
                    },
                    "{bad} on axis {axis}"
                );
            }
        }
    }

    /// [`Sphere::transformed`] cannot fail and can overflow, so a ball built
    /// from finite numbers can be holding an infinite centre. It is then no
    /// longer a ball, and has no inside.
    #[test]
    fn a_ball_shoved_past_the_end_of_the_number_line_has_no_inside() {
        let far = 1e308;
        let shove = Isometry::translation(Vector::new(far, 0.0, 0.0)).unwrap();
        let shoved = Sphere::new(Point::new(far, 0.0, 0.0), 1.0)
            .unwrap()
            .transformed(&shove);

        assert!(!shoved.centre().is_finite(), "the centre overflowed");
        assert_eq!(
            shoved.side_of(PredicatePoint::from([0.0, 5.0, 0.0])),
            Decision::Undecidable {
                enclosure: f64::INFINITY
            }
        );
    }
}
