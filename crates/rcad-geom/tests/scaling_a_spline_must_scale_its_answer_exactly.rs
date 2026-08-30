//! Scale a control net by a power of two and every answer scales by exactly
//! that, with no bound anywhere.
//!
//! `docs/decisions.md` §14.1 asks four questions of an oracle.
//!
//! 1. **Exactly true, or bounded?** Exactly true, and bit for bit. Every
//!    operation in de Boor is homogeneous of degree one in the control points —
//!    the coefficients depend only on the knots and the parameter, and the
//!    perspective divide is `(s·X)/W = s·(X/W)` — and multiplying by a power of
//!    two changes nothing but the exponent. So there is no tolerance here to
//!    tune, which is the point.
//! 2. **Different paths through the dangerous code?** The two runs meet
//!    different exponents at every rounding: the same mantissas, but products
//!    and sums landing in different binades. That is the shape of the check
//!    that caught `orient3d` certifying a wrong sign (§12.2), aimed here at the
//!    new arithmetic.
//! 3. **Which generator?** Families 1 to 4 from `support`, which are M1's own
//!    and are shared with no other milestone (§13 rule 5).
//! 4. **Which gap in §12?** §12.2's. A bound that stops holding at the edges of
//!    the `f64` range is exactly what a scaling test walks a curve into.
//!
//! Derivatives are covered as well as evaluation, because §14.3 says so and
//! because the derived net brings a division by a knot difference that
//! evaluation does not have.
//!
//! The scaled curve is built from the stored homogeneous net rather than from
//! `control_point`, which divides the weight back out and rounds. Scaling the
//! stored numbers is exact; scaling what comes back out of a divide is not.

use proptest::prelude::*;
use rcad_geom::{BSplineCurve, BSplineSurface};

#[path = "support/mod.rs"]
mod support;

proptest! {
    #[test]
    fn a_uniform_polynomial_curve_scales_exactly(
        curve in support::a_uniform_polynomial_curve(),
        exponent in -8i32..=8,
        along in 0.0f64..=1.0,
    ) {
        every_answer_scales(&curve, exponent, along)?;
    }

    #[test]
    fn a_curve_with_kinks_scales_exactly(
        curve in support::a_curve_with_repeated_interior_knots(),
        exponent in -8i32..=8,
        along in 0.0f64..=1.0,
    ) {
        every_answer_scales(&curve, exponent, along)?;
    }

    /// Spans one `f64` wide make the derived net enormous. Scaling it must
    /// still be exact, which is the case where a stray addition would show.
    #[test]
    fn a_curve_whose_spans_are_too_narrow_to_see_scales_exactly(
        curve in support::a_curve_with_spans_too_narrow_to_see(),
        exponent in -8i32..=8,
        along in 0.0f64..=1.0,
    ) {
        every_answer_scales(&curve, exponent, along)?;
    }

    /// The family that reaches furthest towards the ends of the `f64` range,
    /// where a relative argument stops holding.
    #[test]
    fn a_curve_spanning_a_wide_range_of_sizes_scales_exactly(
        curve in support::a_curve_spanning_a_wide_range_of_sizes(),
        exponent in -8i32..=8,
        along in 0.0f64..=1.0,
    ) {
        every_answer_scales(&curve, exponent, along)?;
    }
}

/// A surface is two de Boor passes and two derived nets, so it gets the same
/// treatment by hand — the generated families are curves.
#[test]
fn an_extruded_arc_scales_exactly() {
    let surface = support::the_quarter_circle_extruded_one_unit_in_z();
    for exponent in -8i32..=8 {
        let factor = 2f64.powi(exponent);
        let bigger = surface_scaled_by(&surface, factor);

        for step in 0..=8 {
            let u = f64::from(step) / 8.0;
            for other in 0..=8 {
                let v = f64::from(other) / 8.0;
                same_bits(
                    surface.evaluate(u, v).unwrap().coordinates(),
                    bigger.evaluate(u, v).unwrap().coordinates(),
                    factor,
                    "the point",
                );
                same_bits(
                    surface.partial_u(u, v).unwrap().components(),
                    bigger.partial_u(u, v).unwrap().components(),
                    factor,
                    "the u derivative",
                );
                same_bits(
                    surface.partial_v(u, v).unwrap().components(),
                    bigger.partial_v(u, v).unwrap().components(),
                    factor,
                    "the v derivative",
                );
            }
        }
    }
}

/// The point and the tangent at one parameter, on the curve and on the curve
/// scaled by `2^exponent`.
fn every_answer_scales(
    curve: &BSplineCurve,
    exponent: i32,
    along: f64,
) -> Result<(), TestCaseError> {
    let factor = 2f64.powi(exponent);
    let bigger = curve_scaled_by(curve, factor);

    let (lo, hi) = curve.domain();
    let u = (lo + (hi - lo) * along).min(hi);

    let here = curve.evaluate(u)?.coordinates();
    let there = bigger.evaluate(u)?.coordinates();
    for axis in 0..3 {
        prop_assert_eq!(
            there[axis].to_bits(),
            (here[axis] * factor).to_bits(),
            "the point at {} scaled by {}, axis {}",
            u,
            factor,
            axis
        );
    }

    let slope = curve.tangent(u)?.components();
    let steeper = bigger.tangent(u)?.components();
    for axis in 0..3 {
        prop_assert_eq!(
            steeper[axis].to_bits(),
            (slope[axis] * factor).to_bits(),
            "the tangent at {} scaled by {}, axis {}",
            u,
            factor,
            axis
        );
    }
    Ok(())
}

fn same_bits(here: [f64; 3], there: [f64; 3], factor: f64, what: &str) {
    for axis in 0..3 {
        assert_eq!(
            there[axis].to_bits(),
            (here[axis] * factor).to_bits(),
            "{what} scaled by {factor}, axis {axis}: {} against {}",
            there[axis],
            here[axis] * factor
        );
    }
}

/// The same curve with every control point moved `factor` further out, and
/// every weight left alone.
fn curve_scaled_by(curve: &BSplineCurve, factor: f64) -> BSplineCurve {
    let net = (0..curve.control_point_count())
        .map(|i| scaled(curve.homogeneous_control_point(i), factor))
        .collect();
    BSplineCurve::try_from((curve.knots().clone(), net))
        .expect("a power of two changes an exponent and nothing else")
}

fn surface_scaled_by(surface: &BSplineSurface, factor: f64) -> BSplineSurface {
    let mut net = Vec::new();
    for i in 0..surface.u_knots().control_point_count() {
        for j in 0..surface.v_knots().control_point_count() {
            net.push(scaled(surface.homogeneous_control_point(i, j), factor));
        }
    }
    BSplineSurface::try_from((surface.u_knots().clone(), surface.v_knots().clone(), net))
        .expect("a power of two changes an exponent and nothing else")
}

fn scaled([x, y, z, weight]: [f64; 4], factor: f64) -> [f64; 4] {
    [x * factor, y * factor, z * factor, weight]
}
