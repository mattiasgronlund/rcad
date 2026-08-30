//! Boehm's algorithm copies more than it computes, and this says so sharply.
//!
//! `docs/decisions.md` §14.3 lists "knot insertion is bit-exact outside the
//! affected span" among the relations that are exactly true. The blunt form —
//! insert a knot and the curve is unchanged everywhere — is the same idea
//! needing a bound, and §14.5 files that under gate work. What is asserted here
//! needs no bound at all:
//!
//! - a control point outside the affected span is **bit-identical** in all four
//!   homogeneous coordinates, because it was copied and not recomputed;
//! - evaluation in a span the insertion cannot reach agrees **to the last bit**,
//!   because it reads the same knots and the same stored numbers.
//!
//! §14.1's four questions:
//!
//! 1. **Exactly true, or bounded?** Exactly true. Copying is exact.
//! 2. **Do the two runs take different paths?** Yes. One evaluates a curve of
//!    `n` control points, the other a curve of `n + 1` on a longer knot vector,
//!    at a different span index. They agree bit for bit anyway, which is the
//!    claim.
//! 3. **What family of inputs?** All four generated families, and family 3
//!    hardest: a span one `f64` wide is where a Boehm coefficient would go to
//!    `0/0` if the denominator could shut.
//! 4. **Which §12 gap?** §12.3's — a suite that only checks refusals. Every case
//!    here also asserts the refined curve is one the checking constructors
//!    accept, so a refinement that quietly produced something unbuildable
//!    cannot pass.
//!
//! # Every case compares something, and says so
//!
//! The second claim is about a span the insertion cannot reach, and a curve
//! with too few spans has no such span — the comparison loop then runs zero
//! times and the case asserts nothing at all. That is worse than no test: it
//! reports green and buys nothing. Measured over 3000 draws per family on the
//! generators as they were: 1523, 597, 770 and 1890 draws where the whole curve
//! compared nothing, and about half of all 64,000 insertions compared nothing.
//! The same measurement now: **zero** of either, in any family.
//!
//! Two things stop it now. The generators carry
//! `spans_enough_for_an_untouched_one` spans, which is the counting argument
//! written down beside them; and **every insertion in every case** asserts it
//! had a span clear of the insertion to compare in, so a family that stopped
//! producing them fails here rather than going quiet. A curve too small for the
//! claim is not skipped:
//! [`a_curve_too_small_to_have_an_untouched_span_still_copies_both_of_its_ends`]
//! says what is true there instead.

use proptest::prelude::*;
use rcad_geom::{BSplineCurve, KnotVector, Point};

#[path = "support/mod.rs"]
mod support;

/// The whole claim, for one curve and one parameter to insert.
fn nothing_outside_the_affected_span_moves(
    curve: &BSplineCurve,
    u: f64,
) -> Result<(), TestCaseError> {
    let degree = curve.degree();
    let span = curve.knots().span_containing(u);
    let refined = curve
        .with_knot_inserted(u)
        .map_err(|refusal| TestCaseError::fail(format!("{u} could not be inserted: {refusal}")))?;

    prop_assert!(support::the_kernel_would_build_this_curve_again(&refined));
    prop_assert_eq!(
        refined.control_point_count(),
        curve.control_point_count() + 1
    );
    prop_assert_eq!(
        refined.knots().multiplicity_of(u),
        curve.knots().multiplicity_of(u) + 1
    );
    prop_assert_eq!(refined.domain(), curve.domain());

    // Boehm recomputes control points `span - degree + 1` up to `span` and
    // copies everything on either side of them, the ones above shifted along by
    // the point that was added.
    for i in 0..=span - degree {
        prop_assert_eq!(
            refined.homogeneous_control_point(i),
            curve.homogeneous_control_point(i)
        );
    }
    for i in span + 1..refined.control_point_count() {
        prop_assert_eq!(
            refined.homogeneous_control_point(i),
            curve.homogeneous_control_point(i - 1)
        );
    }

    let untouched = parameters_the_insertion_cannot_reach(curve, span);
    prop_assert!(
        !untouched.is_empty(),
        "inserting at {} into a degree {} curve over {} knots left no span clear of the \
         insertion, so the second half of this claim compared nothing",
        u,
        degree,
        curve.knots().knots().len()
    );
    for at in &untouched {
        prop_assert_eq!(
            support::coordinate_bits(curve.evaluate(*at)?),
            support::coordinate_bits(refined.evaluate(*at)?),
            "the two curves disagree at {} in a span the insertion never touched",
            at
        );
    }

    Ok(())
}

/// Parameters whose whole de Boor window sits clear of the inserted knot.
///
/// Inserting into `span` rewrites control points `span - degree + 1 .. span`
/// and puts a knot at `span + 1`. An evaluation at original span `j` reads
/// knots `j + 1 - degree ..= j + degree` and control points `j - degree ..= j`.
/// Those are all untouched, and carry the same index in both curves, when
/// `j <= span - degree`. Above the insertion the same window is untouched but
/// shifted by one, which needs `j >= span + degree`.
fn parameters_the_insertion_cannot_reach(curve: &BSplineCurve, span: usize) -> Vec<f64> {
    let degree = curve.degree();
    let knots = curve.knots().knots();
    let last_span = knots.len() - degree - 2;

    let mut clear = Vec::new();
    for j in degree..=last_span {
        if knots[j] >= knots[j + 1] {
            continue;
        }
        if j <= span - degree || (span + degree <= last_span && j >= span + degree) {
            clear.push(knots[j]);
            clear.push(knots[j] + (knots[j + 1] - knots[j]) / 2.0);
        }
    }
    clear.retain(|at| curve.knots().contains(*at));
    clear
}

/// One curve, every parameter it will take, so a family is not judged on a
/// single lucky insertion point.
///
/// Every insertion has to leave a span clear on one side or the other, and the
/// claim above asserts that rather than hoping for it: a curve out of any
/// family here carries enough spans for one to exist, so an insertion with
/// nowhere clear to look is a broken generator and not a curve to skip.
fn every_insertion_leaves_the_far_end_alone(curve: &BSplineCurve) -> Result<(), TestCaseError> {
    let candidates = support::parameters_a_knot_can_be_inserted_at(curve);
    prop_assert!(
        !candidates.is_empty(),
        "every curve has at least one span with room in the middle of it"
    );
    for u in candidates {
        nothing_outside_the_affected_span_moves(curve, u)?;
    }
    Ok(())
}

proptest! {
    #[test]
    fn a_uniform_polynomial_curve_keeps_every_control_point_boehm_did_not_touch(
        curve in support::a_uniform_polynomial_curve()
    ) {
        every_insertion_leaves_the_far_end_alone(&curve)?;
    }

    #[test]
    fn a_curve_with_repeated_interior_knots_keeps_every_control_point_boehm_did_not_touch(
        curve in support::a_curve_with_repeated_interior_knots()
    ) {
        every_insertion_leaves_the_far_end_alone(&curve)?;
    }

    /// Family 3 is the one aimed at this code. A span one `f64` wide makes the
    /// coefficient `(u - knots[i]) / (knots[i + degree] - knots[i])` as small as
    /// it can be without the denominator shutting, and the two control points
    /// sitting on top of one another mean a blend of them cannot be told apart
    /// from a copy unless the copying really is exact.
    #[test]
    fn a_curve_with_spans_too_narrow_to_see_keeps_every_control_point_boehm_did_not_touch(
        curve in support::a_curve_with_spans_too_narrow_to_see()
    ) {
        every_insertion_leaves_the_far_end_alone(&curve)?;
    }

    #[test]
    fn a_curve_spanning_a_wide_range_of_sizes_keeps_every_control_point_boehm_did_not_touch(
        curve in support::a_curve_spanning_a_wide_range_of_sizes()
    ) {
        every_insertion_leaves_the_far_end_alone(&curve)?;
    }
}

/// The small curves the property above cannot be asked about, and what is true
/// of them instead.
///
/// One Bezier span at each degree from 1 to 5. The insertion reaches every
/// control point but the first and the last, so no span is clear of it and
/// [`parameters_the_insertion_cannot_reach`] comes back empty — which this
/// asserts, rather than leaving it to be discovered by a property test quietly
/// comparing nothing. What survives with no clear span anywhere is the part
/// that never needed one: the two end control points are copied bit for bit,
/// and a clamped curve is exactly its end control point at either end of the
/// domain, so it reads back the same bits before and after.
#[test]
fn a_curve_too_small_to_have_an_untouched_span_still_copies_both_of_its_ends() {
    for degree in 1..=5 {
        let curve = one_bezier_span(degree);
        let middle = 0.5;
        let span = curve.knots().span_containing(middle);
        assert!(
            parameters_the_insertion_cannot_reach(&curve, span).is_empty(),
            "a degree {degree} curve of one span has no room for an untouched one"
        );

        let refined = curve
            .with_knot_inserted(middle)
            .expect("the middle of the one span");
        assert!(support::the_kernel_would_build_this_curve_again(&refined));
        assert_eq!(
            refined.homogeneous_control_point(0),
            curve.homogeneous_control_point(0)
        );
        assert_eq!(
            refined.homogeneous_control_point(refined.control_point_count() - 1),
            curve.homogeneous_control_point(curve.control_point_count() - 1)
        );
        for at in [0.0, 1.0] {
            assert_eq!(
                support::coordinate_bits(curve.evaluate(at).unwrap()),
                support::coordinate_bits(refined.evaluate(at).unwrap()),
                "a degree {degree} curve moved at {at}, where it is its own end control point"
            );
        }
    }
}

/// One clamped span over `[0, 1]`, with weights that are not all one so the
/// blend has something to do.
fn one_bezier_span(degree: usize) -> BSplineCurve {
    let mut knots = vec![0.0; degree + 1];
    knots.extend(core::iter::repeat_n(1.0, degree + 1));
    let knots = KnotVector::new(degree, knots).expect("a clamped single-span curve");

    let count = knots.control_point_count();
    let points: Vec<Point> = (0..count)
        .map(|i| Point::new(i as f64 * 0.25, 3.0 - i as f64, 1.0 / (i as f64 + 2.0)))
        .collect();
    let weights: Vec<f64> = (0..count).map(|i| 1.0 + (i % 3) as f64).collect();
    BSplineCurve::new(knots, &points, &weights).expect("finite points and positive weights")
}

/// The exact conic, refined at the middle: an insertion that has to keep both
/// ends of the arc where they were, weights and all.
#[test]
fn inserting_a_knot_into_the_quarter_circle_leaves_both_of_its_ends_alone() {
    let arc = support::the_quarter_circle();
    let refined = arc
        .with_knot_inserted(0.5)
        .expect("the middle of the one span");

    assert_eq!(refined.control_point_count(), 4);
    assert_eq!(
        refined.homogeneous_control_point(0),
        arc.homogeneous_control_point(0)
    );
    assert_eq!(
        refined.homogeneous_control_point(3),
        arc.homogeneous_control_point(2)
    );
    for u in [0.0, 1.0] {
        assert_eq!(
            support::coordinate_bits(arc.evaluate(u).unwrap()),
            support::coordinate_bits(refined.evaluate(u).unwrap())
        );
    }
}
