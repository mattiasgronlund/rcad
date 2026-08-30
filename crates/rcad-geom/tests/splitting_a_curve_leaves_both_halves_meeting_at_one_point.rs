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
//! 2. inserting the seam `degree` times by hand and then splitting gives the
//!    same two halves as splitting straight away — two routes, one answer;
//! 3. each half agrees with the whole curve over the stretch it covers. This one
//!    is *not* exact away from the seam, because insertion rounds. It is
//!    checked against [`rcad_geom::REFINEMENT_ROUNDOFF`] and
//!    [`rcad_geom::EVALUATION_ROUNDOFF`] added together — the triangle
//!    inequality through the true point, with no third number anywhere.
//!
//! §14.1's four questions:
//!
//! 1. **Exactly true, or bounded?** Claims 1 and 2 exactly; claim 3 against two
//!    published bounds and nothing else.
//! 2. **Do the two runs take different paths?** Yes. The half runs de Boor at a
//!    different degree of freedom count, over a knot vector clamped where the
//!    whole curve had an ordinary interior knot, and at a different span index.
//! 3. **What family of inputs?** All four generated families. Family 2 is the
//!    one aimed here: it already carries interior knots at every multiplicity
//!    from one up to the degree, so the seam is sometimes reached with no
//!    insertions at all and sometimes with `degree` of them.
//! 4. **Which §12 gap?** §12.3's. Every half is run back through the checking
//!    constructors, so a split that produced a curve the kernel would refuse to
//!    build fails here rather than at some caller in M3.

use proptest::prelude::*;
use rcad_geom::{BSplineCurve, EVALUATION_ROUNDOFF, REFINEMENT_ROUNDOFF};

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

    let mut by_hand = curve.clone();
    while by_hand.knots().multiplicity_of(seam) < degree {
        by_hand = by_hand.with_knot_inserted(seam)?;
    }
    let (also_below, also_above) = by_hand.split_at(seam)?;
    prop_assert!(support::the_same_curve_to_the_last_bit(&below, &also_below));
    prop_assert!(support::the_same_curve_to_the_last_bit(&above, &also_above));

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
