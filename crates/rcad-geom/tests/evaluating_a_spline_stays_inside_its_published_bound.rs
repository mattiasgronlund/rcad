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
//! The exact side is `support::rational`: a rational number on `i128`, reduced
//! by `gcd` at every
//! step, running the identical de Boor recursion over curves whose knots,
//! control coordinates and weights are small whole numbers, read at parameters
//! that are exact `f64` fractions. Every operation is checked, so a case that
//! outgrew `i128` would fail loudly rather than wrap — the answer to that is a
//! smaller case, never a looser assertion.

use proptest::prelude::*;
use rcad_geom::{BSplineCurve, KnotVector, Point};

#[path = "support/mod.rs"]
mod support;

use support::rational::{Exact, exact_de_boor};

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
