//! The five families, and the claim that the kernel takes all of them.
//!
//! Every generator in `support` builds its knot vector and its control net
//! directly and then `expect`s the constructor to take it, so simply drawing a
//! value from one of these strategies is the assertion. What is added here is
//! that the thing built is the thing that was asked for: the right number of
//! control points, a domain with room in it, and weights that came back the way
//! they went in.
//!
//! Without this, the four generated families would have no consumer until the
//! evaluation stage lands, and a generator nobody runs is a generator that does
//! not work.

use proptest::prelude::*;
use rcad_geom::{BSplineCurve, KnotVector, Point};

#[path = "support/mod.rs"]
mod support;

/// What every curve out of a generated family has to look like, whatever the
/// family was aiming at.
fn a_sound_curve(curve: &BSplineCurve) -> Result<(), TestCaseError> {
    prop_assert_eq!(
        curve.control_point_count(),
        curve.knots().control_point_count()
    );

    let (lo, hi) = curve.domain();
    prop_assert!(lo < hi);
    prop_assert_eq!(curve.domain(), curve.knots().domain());
    prop_assert_eq!(
        curve.knots().span_containing(hi),
        curve.knots().knots().len() - curve.degree() - 2
    );

    for i in 0..curve.control_point_count() {
        prop_assert!(curve.weight(i) > 0.0);
        prop_assert!(
            curve
                .homogeneous_control_point(i)
                .iter()
                .all(|number| number.is_finite())
        );
    }

    // The knot vector was accepted once on the way in. Handing the same numbers
    // back proves the accepted vector is one the rules still call sound, rather
    // than something that only got through because of how it was built.
    let rebuilt = KnotVector::new(curve.knots().degree(), curve.knots().knots().to_vec())
        .expect("a knot vector the kernel already holds is one it still accepts");
    prop_assert_eq!(&rebuilt, curve.knots());
    Ok(())
}

proptest! {
    #[test]
    fn a_uniform_polynomial_curve_is_accepted(curve in support::a_uniform_polynomial_curve()) {
        a_sound_curve(&curve)?;
        for i in 0..curve.control_point_count() {
            prop_assert_eq!(curve.weight(i), 1.0);
        }
        for span in curve.degree()..curve.knots().knots().len() - curve.degree() - 1 {
            prop_assert!(curve.knots().knots()[span] < curve.knots().knots()[span + 1]);
        }
    }

    #[test]
    fn a_curve_with_repeated_interior_knots_is_accepted(
        curve in support::a_curve_with_repeated_interior_knots()
    ) {
        a_sound_curve(&curve)?;

        let knots = curve.knots();
        let all = knots.knots();
        prop_assert_eq!(knots.multiplicity_of(all[0]), curve.degree() + 1);
        prop_assert_eq!(knots.multiplicity_of(all[all.len() - 1]), curve.degree() + 1);
        for &interior in &all[curve.degree() + 1..all.len() - curve.degree() - 1] {
            prop_assert!(knots.multiplicity_of(interior) <= curve.degree());
        }
    }

    #[test]
    fn a_curve_with_spans_too_narrow_to_see_is_accepted(
        curve in support::a_curve_with_spans_too_narrow_to_see()
    ) {
        a_sound_curve(&curve)?;

        // The narrow spans are still there: nothing merged them, and the two
        // control points sitting on top of each other are still two points.
        // A hair is one `f64` step measured where the narrow knots sit, which
        // is why the width is read off the first of them rather than written
        // down as a number.
        let knots = curve.knots().knots();
        let narrowest = knots
            .windows(2)
            .map(|pair| pair[1] - pair[0])
            .filter(|width| *width > 0.0)
            .fold(f64::INFINITY, f64::min);
        let first_narrow_knot = knots[curve.degree() + 1];
        prop_assert_eq!(narrowest, first_narrow_knot.next_up() - first_narrow_knot);
        prop_assert_eq!(curve.control_point(0), curve.control_point(1));
    }

    #[test]
    fn a_curve_spanning_a_wide_range_of_sizes_is_accepted(
        curve in support::a_curve_spanning_a_wide_range_of_sizes()
    ) {
        a_sound_curve(&curve)?;
        for i in 0..curve.control_point_count() {
            let weight = curve.weight(i);
            prop_assert!((1e-3..1e3).contains(&weight));
        }
    }
}

/// The load-bearing fixture: every number in it is dyadic, so the stored curve
/// is the unit circle and not a rounding of one.
#[test]
fn the_quarter_circle_is_stored_with_nothing_rounded_away() {
    let arc = support::the_quarter_circle();

    assert_eq!(arc.degree(), 2);
    assert_eq!(arc.domain(), (0.0, 1.0));
    assert_eq!(arc.control_point_count(), 3);

    assert_eq!(arc.control_point(0), Point::new(1.0, 0.0, 0.0));
    assert_eq!(arc.control_point(1), Point::new(1.0, 1.0, 0.0));
    assert_eq!(arc.control_point(2), Point::new(0.0, 1.0, 0.0));
    assert_eq!(
        [arc.weight(0), arc.weight(1), arc.weight(2)],
        [1.0, 1.0, 2.0]
    );
    assert_eq!(arc.homogeneous_control_point(2), [0.0, 2.0, 0.0, 2.0]);
}

#[test]
fn extruding_the_quarter_circle_copies_it_to_both_ends_and_moves_one_of_them() {
    let arc = support::the_quarter_circle();
    let wall = support::the_quarter_circle_extruded_one_unit_in_z();

    assert_eq!(wall.u_knots().degree(), 2);
    assert_eq!(wall.v_knots().degree(), 1);
    assert_eq!(wall.u_knots().knots(), arc.knots().knots());

    for i in 0..arc.control_point_count() {
        let bottom = arc.control_point(i);
        assert_eq!(wall.control_point(i, 0), bottom);
        assert_eq!(
            wall.control_point(i, 1),
            Point::new(bottom.x(), bottom.y(), bottom.z() + 1.0)
        );
        assert_eq!(wall.weight(i, 0), arc.weight(i));
        assert_eq!(wall.weight(i, 1), arc.weight(i));
    }
}
