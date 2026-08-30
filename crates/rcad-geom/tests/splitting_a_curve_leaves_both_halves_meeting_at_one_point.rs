//! A split seam is a control point, so the two halves meet exactly.
//!
//! `docs/decisions.md` §14.3 lists "both halves of a split agreeing bit for bit
//! at a seam of multiplicity `p`" among the relations that are exactly true.
//! The reason it is exact and not merely close: at that multiplicity the curve
//! passes *through* a control point, and the two halves are handed the same four
//! stored numbers rather than each computing their own version of them.
//!
//! Three claims, and only the third needs a bound:
//!
//! 1. the last control point of the lower half and the first of the upper half
//!    are the same bits, and each half evaluated at the seam gives that point;
//! 2. that shared point is **on the curve** — within the published refinement
//!    bound of where the original curve truly is at the seam, worked out in
//!    arithmetic that does not round;
//! 3. each half agrees with the whole curve over the stretch it covers. This one
//!    is *not* exact away from the seam, because insertion rounds. It is
//!    checked against [`rcad_geom::REFINEMENT_ROUNDOFF`] and
//!    [`rcad_geom::EVALUATION_ROUNDOFF`] added together — the triangle
//!    inequality through the true point, with no third number anywhere.
//!
//! # There is only one route through the kernel, so the second one is exact
//!
//! Claim 2 used to be a different thing: raise the seam to a multiplicity of
//! `degree` by hand, split the result, and check it against splitting straight
//! away. That is not two routes. `split_at` raises the multiplicity with the
//! same loop, character for character, so after the by-hand raising its own
//! `while` ran zero times and **both sides reached `cut_at` on bit-identical
//! input**. §14.1's second question — do the two runs take different paths
//! through the dangerous code? — was answered "yes" by a test that could not
//! have failed.
//!
//! There is no second route to a split inside the kernel: raising by Boehm and
//! cutting is what a split *is*, and any other spelling of it calls the same
//! insertion. So the independent route is arithmetic instead of code —
//! `support::rational` works out where the curve truly is, on `i128` rationals
//! with nothing rounded anywhere. It shares no arithmetic at all with the code
//! under test, which is the strongest form of question 2's answer.
//!
//! What it cannot do is run on the generated families: exact rationals over a
//! net of arbitrary `f64` outgrow `i128` within a few stages. So the exact
//! route runs on curves of small whole numbers read at eighths, and the
//! families carry the claims that need no exact answer.
//!
//! §14.1's four questions:
//!
//! 1. **Exactly true, or bounded?** Claim 1 exactly; claims 2 and 3 against
//!    published bounds and nothing else.
//! 2. **Do the two runs take different paths?** For claim 2, as different as
//!    they get: `f64` de Boor through repeated insertion against exact
//!    rationals. For claims 1 and 3, the half runs de Boor at a different
//!    degree of freedom count, over a knot vector clamped where the whole curve
//!    had an ordinary interior knot, and at a different span index.
//! 3. **What family of inputs?** All four generated families, plus the
//!    small-whole-number curves the exact route needs. Family 2 is the one
//!    aimed here: it carries interior knots at every multiplicity from one up
//!    to the degree, so the seam is sometimes reached with no insertions at all
//!    and sometimes with `degree` of them.
//! 4. **Which §12 gap?** §12.3's. Every half is run back through the checking
//!    constructors, so a split that produced a curve the kernel would refuse to
//!    build fails here rather than at some caller in M3.

use proptest::prelude::*;
use rcad_geom::{BSplineCurve, EVALUATION_ROUNDOFF, KnotVector, Point, REFINEMENT_ROUNDOFF};

#[path = "support/mod.rs"]
mod support;

/// Everything a split has to satisfy, for one curve cut at one parameter.
fn the_two_halves_meet_at_the_seam(curve: &BSplineCurve, seam: f64) -> Result<(), TestCaseError> {
    let degree = curve.degree();
    let (lo, hi) = curve.domain();
    let (below, above) = curve
        .split_at(seam)
        .map_err(|refusal| TestCaseError::fail(format!("{seam} could not be cut at: {refusal}")))?;

    prop_assert!(support::the_kernel_would_build_this_curve_again(&below));
    prop_assert!(support::the_kernel_would_build_this_curve_again(&above));
    prop_assert_eq!(below.domain(), (lo, seam));
    prop_assert_eq!(above.domain(), (seam, hi));
    prop_assert_eq!(below.degree(), degree);
    prop_assert_eq!(above.degree(), degree);
    prop_assert_eq!(below.knots().multiplicity_of(seam), degree + 1);
    prop_assert_eq!(above.knots().multiplicity_of(seam), degree + 1);

    let last = below.control_point_count() - 1;
    prop_assert_eq!(
        below.homogeneous_control_point(last),
        above.homogeneous_control_point(0),
        "the halves were handed the same stored point, so this cannot merely be close"
    );
    prop_assert_eq!(
        support::coordinate_bits(below.evaluate(seam)?),
        support::coordinate_bits(above.evaluate(seam)?)
    );
    prop_assert_eq!(
        support::coordinate_bits(below.evaluate(seam)?),
        support::coordinate_bits(below.control_point(last))
    );

    for half in [&below, &above] {
        each_half_says_what_the_whole_curve_says(curve, half)?;
    }
    Ok(())
}

/// The bounded claim: a half evaluated over its own stretch agrees with the
/// whole curve there.
///
/// The allowance is the sum of two published bounds and nothing else. The half's
/// answer is within [`rcad_geom::REFINEMENT_ROUNDOFF`] of the true point of the
/// curve it came from; the whole curve's answer is within
/// [`rcad_geom::EVALUATION_ROUNDOFF`] of the same true point; the difference
/// between the two answers is at most their sum. No slack is added on top,
/// because there is no argument that would justify one — if this needed a third
/// number the bound would be the thing under suspicion, not the code.
fn each_half_says_what_the_whole_curve_says(
    curve: &BSplineCurve,
    half: &BSplineCurve,
) -> Result<(), TestCaseError> {
    let mut compared = 0;
    for at in support::parameters_across(half.domain(), 33) {
        let from_the_half = half.evaluate(at)?;
        let from_the_whole = curve.evaluate(at)?;

        // A `None` is the published bound saying its precondition does not hold,
        // which is a real answer and not an input to be filtered. The count
        // below is what stops every case being one.
        let (Some(refining), Some(evaluating)) = (
            half.refinement_roundoff_at(curve, at)?,
            curve.evaluation_roundoff_at(at)?,
        ) else {
            continue;
        };
        compared += 1;
        let allowed = refining + evaluating;

        for (axis, (mine, theirs)) in
            core::iter::zip(from_the_half.coordinates(), from_the_whole.coordinates()).enumerate()
        {
            prop_assert!(
                (mine - theirs).abs() <= allowed,
                "at {at} on axis {axis} the half says {mine} and the whole curve says {theirs}, \
                 further apart than the {allowed} the two published bounds allow"
            );
        }
    }
    prop_assert!(
        compared > 0,
        "every parameter on this half was skipped, so this half was never checked at all"
    );
    Ok(())
}

/// One curve, cut at every parameter it can be cut at.
fn every_cut_gives_two_halves_that_meet(curve: &BSplineCurve) -> Result<(), TestCaseError> {
    let seams = support::parameters_a_curve_can_be_split_at(curve);
    prop_assert!(
        !seams.is_empty(),
        "every curve has room strictly inside its own domain"
    );
    for seam in seams {
        the_two_halves_meet_at_the_seam(curve, seam)?;
    }
    Ok(())
}

proptest! {
    #[test]
    fn a_uniform_polynomial_curve_splits_into_two_halves_that_meet_exactly(
        curve in support::a_uniform_polynomial_curve()
    ) {
        every_cut_gives_two_halves_that_meet(&curve)?;
    }

    /// Interior knots at every multiplicity from one up to the degree, so the
    /// seam is reached after anywhere from zero to `degree` insertions.
    #[test]
    fn a_curve_with_repeated_interior_knots_splits_into_two_halves_that_meet_exactly(
        curve in support::a_curve_with_repeated_interior_knots()
    ) {
        every_cut_gives_two_halves_that_meet(&curve)?;
    }

    /// Cutting where the spans either side are one `f64` wide. Raising the seam
    /// to full multiplicity here is where a coefficient comes out as exactly
    /// zero or exactly one, and where a denominator would shut if it could.
    #[test]
    fn a_curve_with_spans_too_narrow_to_see_splits_into_two_halves_that_meet_exactly(
        curve in support::a_curve_with_spans_too_narrow_to_see()
    ) {
        every_cut_gives_two_halves_that_meet(&curve)?;
    }

    #[test]
    fn a_curve_spanning_a_wide_range_of_sizes_splits_into_two_halves_that_meet_exactly(
        curve in support::a_curve_spanning_a_wide_range_of_sizes()
    ) {
        every_cut_gives_two_halves_that_meet(&curve)?;
    }
}

/// The seam is on the curve, and each half is the curve it was cut from —
/// against arithmetic that does not round.
///
/// This is the claim the halves agreeing with each other cannot make. They
/// share the same four stored numbers at the seam, so they agree there whatever
/// those numbers are; whether the numbers are *right* needs something outside
/// the kernel to say so, and `support::rational` says it with no rounding
/// anywhere.
///
/// Small whole numbers throughout, read at eighths, because that is what exact
/// rationals on `i128` can carry. Degrees 1 to 3 with a repeated interior knot
/// among them, so the seam is sometimes reached with no insertions and
/// sometimes with `degree` of them.
#[test]
fn every_half_of_a_split_curve_of_whole_numbers_is_where_exact_arithmetic_says_it_is() {
    for curve in curves_a_split_can_be_checked_exactly_on() {
        for seam in support::parameters_a_curve_can_be_split_at(&curve) {
            let (below, above) = curve.split_at(seam).expect("a parameter inside the domain");

            let shared = below.evaluate(seam).expect("the top of the lower half");
            assert_eq!(
                support::coordinate_bits(shared),
                support::coordinate_bits(
                    above.evaluate(seam).expect("the bottom of the upper half")
                )
            );
            no_further_off_than_the_bound_allows(&below, &curve, seam);

            for half in [&below, &above] {
                for at in support::parameters_across(half.domain(), 9) {
                    no_further_off_than_the_bound_allows(half, &curve, at);
                }
            }
        }
    }
}

/// `half` evaluated at `at` is within its published refinement bound of where
/// `original` truly is there.
///
/// # Panics
///
/// If the bound is not published. These curves are made of small whole numbers,
/// so its precondition holds and a `None` would be the timid filter
/// `docs/decisions.md` §14.7 is about rather than an input to skip.
fn no_further_off_than_the_bound_allows(half: &BSplineCurve, original: &BSplineCurve, at: f64) {
    let allowed = half
        .refinement_roundoff_at(original, at)
        .expect("a parameter from the half's own domain")
        .expect("small whole numbers are as normal as numbers get");
    let truth = support::rational::the_true_point_at(original, at);
    let evaluated = half.evaluate(at).expect("a parameter from its own domain");

    for (axis, coordinate) in evaluated.coordinates().into_iter().enumerate() {
        let missed = support::rational::Exact::from_f64(coordinate)
            .minus(truth[axis])
            .abs()
            .to_f64();
        assert!(
            missed <= allowed,
            "at {at} on axis {axis} the half says {coordinate}, off the true point by {missed}, \
             where the published bound allows {allowed}"
        );
    }
}

/// Curves the exact route can carry: whole knots, whole control coordinates,
/// whole weights, and no case bigger than `i128` can hold.
///
/// Its own list rather than a shared fixture. The evaluation bound's suite has
/// one of these too, aimed at cancellation; this one is aimed at the seams a
/// split lands on (`docs/decisions.md` §13 rule 5).
fn curves_a_split_can_be_checked_exactly_on() -> Vec<BSplineCurve> {
    vec![
        whole_numbered_curve(
            1,
            &[0.0, 0.0, 1.0, 2.0, 2.0],
            &[(0, 0, 0), (2, 4, 8), (6, 0, -4)],
            &[1.0, 3.0, 2.0],
        ),
        whole_numbered_curve(
            2,
            &[0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0],
            &[(1, 0, 0), (1, 1, 0), (0, 1, 3), (-2, 1, 1)],
            &[1.0, 2.0, 1.0, 3.0],
        ),
        // The interior knot already repeats as often as the degree, so a split
        // there inserts nothing at all and cuts the stored net where it stands.
        whole_numbered_curve(
            2,
            &[0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 2.0],
            &[(1, 0, 0), (1, 1, 0), (0, 1, 0), (-1, 2, 5), (3, -3, 2)],
            &[1.0, 1.0, 2.0, 4.0, 1.0],
        ),
        whole_numbered_curve(
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
    ]
}

/// A curve from whole numbers, so nothing rounds on the way in.
fn whole_numbered_curve(
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

/// The published refinement bound is the two-pass composition of the evaluation
/// bound, so it has to be bigger than one pass and smaller than two and a bit.
///
/// A typo in the composition is the cheapest way for it to go wrong, and it is
/// the one thing about it a test can see without evaluating anything.
#[test]
fn the_refinement_bound_costs_a_little_more_than_two_evaluations() {
    const { assert!(REFINEMENT_ROUNDOFF > 2.0 * EVALUATION_ROUNDOFF) };
    const { assert!(REFINEMENT_ROUNDOFF < 2.1 * EVALUATION_ROUNDOFF) };
    assert!(REFINEMENT_ROUNDOFF.is_normal());
}

/// The exact conic cut in half: both halves still pass through the point on the
/// unit circle that no `f64` can name.
#[test]
fn splitting_the_quarter_circle_leaves_the_seam_exactly_on_the_circle() {
    let arc = support::the_quarter_circle();
    let (below, above) = arc.split_at(0.5).expect("the middle of the one span");

    let seam = below.evaluate(0.5).unwrap();
    assert_eq!(
        support::coordinate_bits(seam),
        support::coordinate_bits(above.evaluate(0.5).unwrap())
    );

    // Three fifths and four fifths are not f64, so this is against the whole
    // curve's own published bound rather than against an equality.
    let bound = arc
        .evaluation_roundoff_at(0.5)
        .unwrap()
        .expect("every number in this arc is a small whole one")
        + below
            .refinement_roundoff_at(&arc, 0.5)
            .unwrap()
            .expect("and so is every number in either half");
    assert!((seam.x() - 0.6).abs() <= bound);
    assert!((seam.y() - 0.8).abs() <= bound);
    assert_eq!(seam.z(), 0.0, "a flat arc has an exact z");
}
