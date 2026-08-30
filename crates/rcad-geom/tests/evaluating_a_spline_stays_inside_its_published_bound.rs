//! What `EVALUATION_ROUNDOFF` claims, checked against arithmetic that does not
//! round.
//!
//! The derivation lives in the constant's doc comment. This is the other half
//! `docs/decisions.md` §15.6 insists on: **a published bound needs a test that
//! could fail, not a paragraph.** The `4·ε` next door became true only when a
//! test evaluated the same expression exactly and subtracted, and the same
//! applies here.
//!
//! What it cannot do is show the bound is *tight*. Rounding errors cancel, so
//! this passes at a smaller constant too. The derivation is what makes it a
//! bound; the test is what stops the expression drifting out from under it.
//!
//! How much smaller, measured rather than guessed: the closest any case here
//! gets is the degree-5 one, whose control points alternate sign so almost the
//! whole sum cancels and what is left is nearly all rounding. Its bound is
//! about 300 times its actual error, and shrinking `EVALUATION_ROUNDOFF` by
//! about 500 is where this file starts failing. Two things eat that room and
//! neither is slack in the derivation: the constant is published at
//! `MAX_DEGREE` stages while these curves run 1 to 5, and errors cancel instead
//! of piling up. The consequence worth knowing is that a change to the bound
//! worth less than a factor of a few hundred passes here unnoticed — the weight
//! term in the perspective divide is worth about a factor of two, so this file
//! does not see it at all.
//!
//! The exact side is a rational number on `i128`, reduced by `gcd` at every
//! step, running the identical de Boor recursion over curves whose knots,
//! control coordinates and weights are small whole numbers, read at parameters
//! that are exact `f64` fractions. Every operation is checked, so a case that
//! outgrew `i128` would fail loudly rather than wrap — the answer to that is a
//! smaller case, never a looser assertion.

use proptest::prelude::*;
use rcad_geom::{BSplineCurve, KnotVector, Point};

#[path = "support/mod.rs"]
mod support;

/// The nets the exact comparison runs over: degrees 1, 2 and 3 with uniform
/// spans and a repeated interior knot, and one degree-5 case built to cancel.
/// No case has all its weights at one, so the perspective divide is always
/// doing something.
fn curves_made_of_small_whole_numbers() -> Vec<BSplineCurve> {
    vec![
        curve(
            1,
            &[0.0, 0.0, 1.0, 2.0, 2.0],
            &[(0, 0, 0), (2, 4, 8), (6, 0, -4)],
            &[1.0, 3.0, 2.0],
        ),
        curve(
            2,
            &[0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0],
            &[(1, 0, 0), (1, 1, 0), (0, 1, 3), (-2, 1, 1)],
            &[1.0, 2.0, 1.0, 3.0],
        ),
        curve(
            2,
            &[0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 2.0],
            &[(1, 0, 0), (1, 1, 0), (0, 1, 0), (-1, 2, 5), (3, -3, 2)],
            &[1.0, 1.0, 2.0, 4.0, 1.0],
        ),
        curve(
            3,
            &[0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0, 3.0],
            &[
                (0, 0, 0),
                (1, 2, 3),
                (4, -1, 2),
                (-3, 5, 0),
                (2, 2, -6),
                (7, 0, 1),
            ],
            &[1.0, 2.0, 1.0, 4.0, 3.0, 1.0],
        ),
        // Alternating signs across a long span, so the sum cancels almost
        // everything and what is left is nearly all rounding. This is the case
        // that gets closest to the published bound.
        curve(
            5,
            &[
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0,
            ],
            &[
                (7, -7, 7),
                (-7, 7, -7),
                (7, -7, 7),
                (-7, 7, -7),
                (7, -7, 7),
                (-7, 7, -7),
                (7, -7, 7),
                (-7, 7, -7),
            ],
            &[1.0, 3.0, 1.0, 5.0, 2.0, 7.0, 1.0, 4.0],
        ),
    ]
}

#[test]
fn evaluating_a_curve_of_whole_numbers_lands_inside_the_published_bound() {
    for curve in curves_made_of_small_whole_numbers() {
        for (u, exactly) in eighths_of(&curve) {
            let evaluated = curve
                .evaluate(u)
                .expect("every eighth is inside the domain");
            let bound = curve
                .evaluation_roundoff_at(u)
                .expect("every eighth is inside the domain")
                .expect("small whole numbers are as normal as numbers get");

            let span = curve.knots().span_containing(u);
            let truth = exact_de_boor(&curve, exactly, span);
            let weight = truth[3];

            for (axis, coordinate) in evaluated.coordinates().iter().enumerate() {
                let missed = Exact::from_f64(*coordinate)
                    .minus(truth[axis].over(weight))
                    .abs()
                    .to_f64();
                assert!(
                    missed <= bound,
                    "degree {} at {u}, axis {axis}: off by {missed}, bound {bound}",
                    curve.degree()
                );
            }
        }
    }
}

proptest! {
    /// Family 4 is the wide-dynamic-range one, aimed straight at the
    /// precondition. The claim is one-sided on purpose: a bound that comes back
    /// has to hold, and there is no obligation for one to come back at all.
    ///
    /// What holds it to account without exact arithmetic is the convex hull.
    /// Every evaluated point is `Σ (Nᵢ wᵢ / W) Pᵢ` with every share
    /// non-negative and the shares summing to one, so the true point is inside
    /// the box around the control points the span reaches, and the computed one
    /// is inside that box widened by the bound and nothing else.
    #[test]
    fn a_curve_of_wildly_different_sizes_either_bounds_its_own_rounding_or_says_it_cannot(
        curve in support::a_curve_spanning_a_wide_range_of_sizes(),
        along in 0.0f64..=1.0,
    ) {
        let u = somewhere_along(&curve, along);
        let evaluated = curve.evaluate(u)?;

        if let Some(bound) = curve.evaluation_roundoff_at(u)? {
            prop_assert!(bound.is_finite() && bound >= 0.0);
            stays_in_the_hull(&curve, u, evaluated, bound)?;
        }
    }

    /// Families 1 to 3 keep every weight at one and every coordinate in the
    /// ordinary range, so a bound is not merely allowed — refusing to publish
    /// one would be the timid filter `docs/decisions.md` §14.7 is about.
    #[test]
    fn a_polynomial_curve_always_gets_a_bound_and_stays_inside_it(
        curve in prop_oneof![
            support::a_uniform_polynomial_curve(),
            support::a_curve_with_repeated_interior_knots(),
            support::a_curve_with_spans_too_narrow_to_see(),
        ],
        along in 0.0f64..=1.0,
    ) {
        let u = somewhere_along(&curve, along);
        let evaluated = curve.evaluate(u)?;
        let bound = curve
            .evaluation_roundoff_at(u)?
            .expect("weights of one and coordinates around the origin are normal numbers");

        prop_assert!(bound.is_finite() && bound >= 0.0);
        stays_in_the_hull(&curve, u, evaluated, bound)?;
    }
}

/// A parameter `along` of the way through the domain, ends included.
fn somewhere_along(curve: &BSplineCurve, along: f64) -> f64 {
    let (lo, hi) = curve.domain();
    (lo + (hi - lo) * along).min(hi)
}

/// The evaluated point sits in the box around the control points the span
/// reaches, widened by `bound` and by the rounding of reading those control
/// points back out.
fn stays_in_the_hull(
    curve: &BSplineCurve,
    u: f64,
    evaluated: Point,
    bound: f64,
) -> Result<(), TestCaseError> {
    let span = curve.knots().span_containing(u);
    let corners: Vec<Point> = (span - curve.degree()..=span)
        .map(|i| curve.control_point(i))
        .collect();

    for axis in 0..3 {
        let lo = corners
            .iter()
            .map(|corner| corner.coordinates()[axis])
            .fold(f64::INFINITY, f64::min);
        let hi = corners
            .iter()
            .map(|corner| corner.coordinates()[axis])
            .fold(f64::NEG_INFINITY, f64::max);

        // `control_point` divides the stored weight back out, which rounds
        // once, so the true corner can be `u` either side of the one read here.
        let slack = bound + f64::EPSILON * lo.abs().max(hi.abs());
        let coordinate = evaluated.coordinates()[axis];
        prop_assert!(
            lo - slack <= coordinate && coordinate <= hi + slack,
            "at {u}, axis {axis}: {coordinate} is outside [{lo}, {hi}] by more than {slack}"
        );
    }
    Ok(())
}

/// Every eighth of the way across the domain, plus every knot inside it, as an
/// `f64` and as the exact number that `f64` is.
fn eighths_of(curve: &BSplineCurve) -> Vec<(f64, Exact)> {
    let (lo, hi) = curve.domain();
    let steps = (8.0 * (hi - lo)) as i64;
    let mut parameters: Vec<f64> = (0..=steps).map(|step| lo + step as f64 / 8.0).collect();
    parameters.extend(curve.knots().knots());
    parameters.retain(|u| curve.knots().contains(*u));
    parameters
        .into_iter()
        .map(|u| (u, Exact::from_f64(u)))
        .collect()
}

/// The de Boor recursion again, in arithmetic that does not round.
fn exact_de_boor(curve: &BSplineCurve, at: Exact, span: usize) -> [Exact; 4] {
    let degree = curve.degree();
    let knots: Vec<Exact> = curve
        .knots()
        .knots()
        .iter()
        .map(|knot| Exact::from_f64(*knot))
        .collect();
    let mut points: Vec<[Exact; 4]> = (0..=degree)
        .map(|j| {
            curve
                .homogeneous_control_point(j + span - degree)
                .map(Exact::from_f64)
        })
        .collect();

    let one = Exact::whole(1);
    for round in 1..=degree {
        for j in (round..=degree).rev() {
            let below = knots[j + span - degree];
            let above = knots[j + 1 + span - round];
            let share = at.minus(below).over(above.minus(below));
            let rest = one.minus(share);
            let blended: [Exact; 4] = core::array::from_fn(|axis| {
                rest.times(points[j - 1][axis])
                    .plus(share.times(points[j][axis]))
            });
            points[j] = blended;
        }
    }
    points[degree]
}

/// A rational number on `i128`, always reduced and always with a positive
/// bottom.
///
/// Every operation is checked. A case that outgrows `i128` is a case to make
/// smaller, not an assertion to loosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Exact {
    top: i128,
    bottom: i128,
}

impl Exact {
    fn whole(value: i128) -> Self {
        Self {
            top: value,
            bottom: 1,
        }
    }

    fn new(top: i128, bottom: i128) -> Self {
        assert!(bottom != 0, "a rational cannot be over zero");
        let shared = gcd(top.unsigned_abs(), bottom.unsigned_abs()).max(1) as i128;
        let sign = if bottom < 0 { -1 } else { 1 };
        Self {
            top: sign * (top / shared),
            bottom: sign * (bottom / shared),
        }
    }

    /// The exact value of an `f64`: mantissa over a power of two.
    fn from_f64(value: f64) -> Self {
        assert!(
            value.is_finite(),
            "{value} is not a number to be exact about"
        );
        if value == 0.0 {
            return Self::whole(0);
        }
        let bits = value.to_bits();
        let sign = if bits >> 63 == 1 { -1i128 } else { 1 };
        let raw_exponent = ((bits >> 52) & 0x7ff) as i32;
        let raw_mantissa = i128::from(bits & ((1u64 << 52) - 1));
        let (mantissa, exponent) = if raw_exponent == 0 {
            (raw_mantissa, -1074)
        } else {
            (raw_mantissa | (1i128 << 52), raw_exponent - 1075)
        };

        if exponent >= 0 {
            Self::new(sign * mantissa * two_to_the(exponent), 1)
        } else {
            Self::new(sign * mantissa, two_to_the(-exponent))
        }
    }

    fn plus(self, other: Self) -> Self {
        let left = checked(self.top, other.bottom, "adding");
        let right = checked(other.top, self.bottom, "adding");
        Self::new(
            left.checked_add(right).expect("a sum too big for i128"),
            checked(self.bottom, other.bottom, "adding"),
        )
    }

    fn minus(self, other: Self) -> Self {
        self.plus(Self {
            top: -other.top,
            bottom: other.bottom,
        })
    }

    fn times(self, other: Self) -> Self {
        let left = Self::new(self.top, other.bottom);
        let right = Self::new(other.top, self.bottom);
        Self::new(
            checked(left.top, right.top, "multiplying"),
            checked(left.bottom, right.bottom, "multiplying"),
        )
    }

    fn over(self, other: Self) -> Self {
        assert!(other.top != 0, "an exact division by zero");
        self.times(Self::new(other.bottom, other.top))
    }

    fn abs(self) -> Self {
        Self {
            top: self.top.abs(),
            bottom: self.bottom,
        }
    }

    /// The nearest `f64`, which is only ever used to compare a difference
    /// against a bound many orders of magnitude larger than the conversion.
    fn to_f64(self) -> f64 {
        self.top as f64 / self.bottom as f64
    }
}

fn checked(left: i128, right: i128, what: &str) -> i128 {
    left.checked_mul(right)
        .unwrap_or_else(|| panic!("{what} {left} and {right} outgrew i128; make the case smaller"))
}

fn two_to_the(exponent: i32) -> i128 {
    assert!(
        (0..127).contains(&exponent),
        "2^{exponent} does not fit in an i128"
    );
    1i128 << exponent
}

fn gcd(left: u128, right: u128) -> u128 {
    if right == 0 {
        left
    } else {
        gcd(right, left % right)
    }
}

/// A curve from whole numbers, so nothing rounds on the way in.
fn curve(
    degree: usize,
    knots: &[f64],
    corners: &[(i32, i32, i32)],
    weights: &[f64],
) -> BSplineCurve {
    let points: Vec<Point> = corners
        .iter()
        .map(|(x, y, z)| Point::new(f64::from(*x), f64::from(*y), f64::from(*z)))
        .collect();
    let knots = KnotVector::new(degree, knots.to_vec()).expect("a clamped, rising knot vector");
    BSplineCurve::new(knots, &points, weights).expect("one point and one weight per control point")
}
