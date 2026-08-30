//! Bezier extraction hands the same stored point to both sides of every join.
//!
//! Roadmap §6.2's Bezier clipping is what `bezier_segments` is for, and clipping
//! walks a chain of segments. A chain whose links only nearly meet grows a
//! hairline gap at every join, and the gaps are where a clipping algorithm loses
//! a root. So the claim is not that neighbouring segments are close at their
//! shared end — it is that they carry the same four stored numbers there.
//!
//! §14.1's four questions:
//!
//! 1. **Exactly true, or bounded?** The joins, the shapes and the single-Bezier
//!    case are exact. The one bounded claim — a segment agreeing with the curve
//!    it came from — is checked only where
//!    [`rcad_geom::REFINEMENT_ROUNDOFF`]'s stage count actually covers the
//!    refinement, which is a curve with one interior knot value.
//! 2. **Do the two runs take different paths?** Yes. A segment evaluates over a
//!    knot vector of `2 * (degree + 1)` knots where the curve it came from had
//!    interior knots, so every de Boor coefficient in it is a different number.
//! 3. **What family of inputs?** All four generated families. Family 2 decides
//!    how many insertions each knot needs and family 3 makes the spans between
//!    them one `f64` wide.
//! 4. **Which §12 gap?** §12.3's, again as the acceptance half: every segment
//!    goes back through the checking constructors.

use proptest::prelude::*;
use rcad_geom::{BSplineCurve, KnotVector, Point};

#[path = "support/mod.rs"]
mod support;

/// Everything the list of segments has to satisfy.
fn the_segments_tile_the_curve(curve: &BSplineCurve) -> Result<(), TestCaseError> {
    let degree = curve.degree();
    let segments = curve.bezier_segments();
    let (lo, hi) = curve.domain();

    // Every knot lies between the two clamps, so a window with width is a span
    // inside the domain and there is nothing else to exclude.
    let spans_with_width = curve
        .knots()
        .knots()
        .windows(2)
        .filter(|pair| pair[0] < pair[1])
        .count();
    prop_assert_eq!(segments.len(), spans_with_width);

    for (which, segment) in segments.iter().enumerate() {
        prop_assert!(support::the_kernel_would_build_this_curve_again(segment));
        prop_assert_eq!(segment.degree(), degree);
        prop_assert_eq!(segment.control_point_count(), degree + 1);

        let (start, end) = segment.domain();
        let knots = segment.knots().knots();
        prop_assert_eq!(knots.len(), 2 * (degree + 1));
        prop_assert!(knots[..=degree].iter().all(|knot| *knot == start));
        prop_assert!(knots[degree + 1..].iter().all(|knot| *knot == end));
        prop_assert!(start < end);

        if which > 0 {
            let last = &segments[which - 1];
            prop_assert_eq!(last.domain().1, start);
            prop_assert_eq!(
                last.homogeneous_control_point(degree),
                segment.homogeneous_control_point(0),
                "segment {} and segment {} do not share the point they join at",
                which - 1,
                which
            );
        }
    }

    prop_assert_eq!(segments[0].domain().0, lo);
    prop_assert_eq!(segments[segments.len() - 1].domain().1, hi);
    Ok(())
}

proptest! {
    #[test]
    fn a_uniform_polynomial_curve_comes_apart_into_beziers_that_join_exactly(
        curve in support::a_uniform_polynomial_curve()
    ) {
        the_segments_tile_the_curve(&curve)?;
    }

    #[test]
    fn a_curve_with_repeated_interior_knots_comes_apart_into_beziers_that_join_exactly(
        curve in support::a_curve_with_repeated_interior_knots()
    ) {
        the_segments_tile_the_curve(&curve)?;
    }

    /// Spans one `f64` wide still count as spans, so each of them gets a
    /// segment of its own rather than being quietly merged away.
    #[test]
    fn a_curve_with_spans_too_narrow_to_see_comes_apart_into_beziers_that_join_exactly(
        curve in support::a_curve_with_spans_too_narrow_to_see()
    ) {
        the_segments_tile_the_curve(&curve)?;
    }

    #[test]
    fn a_curve_spanning_a_wide_range_of_sizes_comes_apart_into_beziers_that_join_exactly(
        curve in support::a_curve_spanning_a_wide_range_of_sizes()
    ) {
        the_segments_tile_the_curve(&curve)?;
    }

    /// One interior knot value, so the whole extraction is at most `degree`
    /// insertions deep and [`rcad_geom::REFINEMENT_ROUNDOFF`] covers it. A curve
    /// with more interior knots blends deeper than the cap the bound was derived
    /// under, and asserting against it there would be asserting against a number
    /// that is not a bound.
    #[test]
    fn a_bezier_segment_says_what_the_curve_it_came_from_says(
        curve in a_curve_with_one_interior_knot()
    ) {
        the_segments_tile_the_curve(&curve)?;

        for segment in curve.bezier_segments() {
            for at in support::parameters_across(segment.domain(), 33) {
                let (Some(refining), Some(evaluating)) = (
                    segment.refinement_roundoff_at(&curve, at)?,
                    curve.evaluation_roundoff_at(at)?,
                ) else {
                    continue;
                };
                let allowed = refining + evaluating;

                let mine = segment.evaluate(at)?;
                let theirs = curve.evaluate(at)?;
                for (axis, (a, b)) in
                    core::iter::zip(mine.coordinates(), theirs.coordinates()).enumerate()
                {
                    prop_assert!(
                        (a - b).abs() <= allowed,
                        "at {at} on axis {axis} the segment says {a} and the curve says {b}, \
                         further apart than the {allowed} the two published bounds allow"
                    );
                }
            }
        }
    }
}

/// Two spans meeting at one interior knot, whose multiplicity runs from one up
/// to the degree.
///
/// Built directly rather than filtered: a strategy that kept only the curves a
/// constructor accepted could not see a good one being turned away
/// (`docs/decisions.md` §12.3).
fn a_curve_with_one_interior_knot() -> impl Strategy<Value = BSplineCurve> {
    (2usize..=5)
        .prop_flat_map(|degree| (Just(degree), 1usize..=degree))
        .prop_map(|(degree, multiplicity)| {
            let mut knots = vec![0.0; degree + 1];
            knots.extend(core::iter::repeat_n(1.0, multiplicity));
            knots.extend(core::iter::repeat_n(2.0, degree + 1));
            let knots = KnotVector::new(degree, knots)
                .expect("clamped, rising, and one interior knot at most degree times");

            let count = knots.control_point_count();
            let points: Vec<Point> = (0..count)
                .map(|i| Point::new(i as f64 * 0.25, 3.0 - i as f64, 1.0 / (i as f64 + 2.0)))
                .collect();
            let weights: Vec<f64> = (0..count).map(|i| 1.0 + (i % 3) as f64).collect();
            BSplineCurve::new(knots, &points, &weights).expect("finite points, positive weights")
        })
}

/// The exact conic is already one Bezier, so extraction has nothing to do and
/// must hand it back untouched rather than round-trip it through an insertion.
#[test]
fn the_quarter_circle_is_already_one_bezier_and_comes_back_unchanged() {
    let arc = support::the_quarter_circle();
    let segments = arc.bezier_segments();

    assert_eq!(segments.len(), 1);
    assert!(support::the_same_curve_to_the_last_bit(&segments[0], &arc));
}
