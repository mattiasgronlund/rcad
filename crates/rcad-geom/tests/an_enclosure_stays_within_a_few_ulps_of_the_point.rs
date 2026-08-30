//! What an enclosure promises, and how wide it is allowed to get.
//!
//! The promises are properties over the four generated families: the ordinary
//! `f64` point is always inside the box, an evaluation with nothing to round
//! comes back with no width at all, and scaling a curve by a power of two
//! scales its enclosure by exactly the same factor.
//!
//! The width is M1's exit criterion 13, and it is asserted here rather than
//! only reported by `benches/how_wide_the_enclosure_gets.rs`. A `criterion` run
//! is something a human reads, and `docs/decisions.md` §12.5 was honest that
//! nobody notices one of those regressing. The bench is where the figure comes
//! from; this is what fails CI when it drifts.

#![forbid(unsafe_code)]

mod support;

use proptest::prelude::*;
use rcad_geom::{BSplineCurve, KnotVector, Point};

use support::{
    a_curve_spanning_a_wide_range_of_sizes, a_curve_with_a_wide_dynamic_range,
    a_curve_with_repeated_interior_knots, a_curve_with_spans_too_narrow_to_see,
    a_surface_with_a_wide_dynamic_range, a_uniform_polynomial_curve, a_well_conditioned_curve,
    a_well_conditioned_surface, parameters_across, widest_enclosure_of_a_curve,
    widest_enclosure_of_a_surface,
};

/// The widest enclosure a curve at an ordinary working size may produce, in
/// `f64` steps of the model's own size.
///
/// **Measured first, then set with room on top.** The bench sweeps degrees 1 to
/// 5 and working sizes from 1 to a million, over the well-conditioned and the
/// wide-dynamic-range shapes, for curves and for patches. The widest anything
/// there reached was 20 steps, and the figure barely moves with the working
/// size — which is what something scale-free should look like. This ceiling is
/// a little over three times that, rounded to a power of two.
///
/// A safety factor, not a bound, so per `docs/decisions.md` §15.6 rule 2 it
/// lives here at its consumer with its own reason rather than being smuggled
/// into a derived constant. Three times leaves room for a net unluckier than
/// the ones measured and is far too little to hide a change that made the
/// arithmetic meaningfully worse: two more roundings per de Boor stage at
/// degree 5 would show up here at once.
const WIDEST_ENCLOSURE_AT_AN_ORDINARY_WORKING_SIZE: f64 = 64.0;

/// The same for a randomly generated curve, where the weights alone can span
/// six orders of magnitude.
///
/// Wider on purpose, and the reason is family 4: a curve whose weights run from
/// a thousandth to a thousand has homogeneous control points five hundred times
/// larger than its affine ones, and the box is measured against the affine size
/// a caller thinks in. Over a hundred thousand draws of that family the worst
/// was 157 steps, three in a hundred thousand went past 128, and none reached
/// 256. Families 1 to 3 never passed 41.
///
/// So: three times the worst of a hundred thousand draws, rounded to a power of
/// two. Loose enough that a normal run cannot trip it by luck, tight enough
/// that a tenfold widening cannot hide behind it.
///
/// If either ceiling fails, the answer is not a bigger number. It is a real
/// widening to find, or it is the news that M3's certified marching needs
/// centred forms — which is the thing these figures were measured to decide.
const WIDEST_ENCLOSURE_FROM_A_GENERATED_CURVE: f64 = 512.0;

proptest! {
    /// The cross-check that `EVALUATION_ROUNDOFF` is deliberately not used for.
    ///
    /// Interval de Boor is the only enclosure, and the published `f64` bound is
    /// not a second source and is never intersected with it — an empty
    /// intersection would be a free check at the price of making a published
    /// bound load-bearing at runtime, which is where `docs/decisions.md` §12.2's
    /// mistake lives. So the check happens here, as a test, where a failure is
    /// a failure and not a silently wrong answer in a customer's model.
    #[test]
    fn the_ordinary_point_is_always_inside_the_enclosure_from_the_same_parameter(
        curve in prop_oneof![
            a_uniform_polynomial_curve(),
            a_curve_with_repeated_interior_knots(),
            a_curve_with_spans_too_narrow_to_see(),
            a_curve_spanning_a_wide_range_of_sizes(),
        ]
    ) {
        for u in parameters_across(curve.domain(), 33) {
            let point = curve.evaluate(u).unwrap();
            let enclosure = curve.evaluate_with_enclosure(u).unwrap().enclosure();
            for (coordinate, side) in point.coordinates().into_iter().zip(enclosure) {
                prop_assert!(
                    side.lo() <= coordinate && coordinate <= side.hi(),
                    "at {u} the f64 answer {coordinate} is outside the box {side:?}"
                );
            }
        }
    }

    /// A degree-1 curve through whole-number control points, read at the middle
    /// of a span, has nothing to round anywhere: the coefficient is exactly a
    /// half, the blend is a sum of two whole numbers halved, and the weight is
    /// one. So the enclosure has no width and the cascade gets an exact point.
    ///
    /// Getting this wrong in the safe direction — always handing back a box —
    /// costs nothing that a test of soundness would notice, and quietly fails
    /// the acceptance oracle, because only an exact point reaches level 3.
    #[test]
    fn a_straight_run_between_whole_numbers_read_at_a_half_has_no_width_at_all(
        curve in a_straight_run_between_whole_numbers()
    ) {
        let (lo, hi) = curve.domain();
        let mut halves: Vec<f64> = curve.knots().knots().to_vec();
        halves.extend((0..halves.len() - 1).map(|i| lo + i as f64 + 0.5));
        halves.retain(|u| (lo..=hi).contains(u));

        for u in halves {
            let point = curve.evaluate_with_enclosure(u).unwrap();
            prop_assert_eq!(
                point.exact_coordinates(),
                Some(curve.evaluate(u).unwrap().coordinates()),
                "nothing at {} rounds, so the enclosure has to be a point", u
            );
        }
    }

    /// Scaling a control net by a power of two scales every intermediate by the
    /// same factor with nothing rounded, and rounding itself is blind to a
    /// power of two — so the enclosure has to come back scaled bit for bit,
    /// both ends. A width that grew or shrank means something in the recursion
    /// is looking at the size of its input.
    ///
    /// The exponents stay inside ±50, which keeps a net that started between a
    /// micrometre and a kilometre well clear of both ends of the `f64` range.
    #[test]
    fn scaling_a_curve_by_a_power_of_two_scales_both_ends_of_its_enclosure_exactly(
        curve in a_uniform_polynomial_curve(),
        exponent in -50i32..50,
    ) {
        let factor = 2f64.powi(exponent);
        let scaled = scaled_by(&curve, factor);

        for u in parameters_across(curve.domain(), 17) {
            let here = curve.evaluate_with_enclosure(u).unwrap().enclosure();
            let there = scaled.evaluate_with_enclosure(u).unwrap().enclosure();
            for (side, scaled_side) in here.into_iter().zip(there) {
                prop_assert_eq!(scaled_side.lo(), factor * side.lo(), "at {} the lower end", u);
                prop_assert_eq!(scaled_side.hi(), factor * side.hi(), "at {} the upper end", u);
            }
        }
    }

    /// The ceiling, over everything the generated families can build.
    #[test]
    fn no_generated_curve_has_an_enclosure_wider_than_the_ceiling(
        curve in prop_oneof![
            a_uniform_polynomial_curve(),
            a_curve_with_repeated_interior_knots(),
            a_curve_with_spans_too_narrow_to_see(),
            a_curve_spanning_a_wide_range_of_sizes(),
        ]
    ) {
        let widest = widest_enclosure_of_a_curve(&curve).in_ulps_of_the_size;
        prop_assert!(
            widest <= WIDEST_ENCLOSURE_FROM_A_GENERATED_CURVE,
            "a degree {} curve encloses its point in {widest} f64 steps of its own size",
            curve.degree()
        );
    }
}

/// The other ceiling, over the sized shapes the bench measures, so the figure
/// quoted above and the figure CI checks come off the same sweep.
#[test]
fn nothing_the_bench_measures_has_an_enclosure_wider_than_the_ceiling() {
    for degree in 1..=5 {
        for size in [1.0, 100.0, 1e3, 1e4, 1e6] {
            let widest = [
                widest_enclosure_of_a_curve(&a_well_conditioned_curve(degree, size)),
                widest_enclosure_of_a_curve(&a_curve_with_a_wide_dynamic_range(degree, size)),
                widest_enclosure_of_a_surface(&a_well_conditioned_surface(degree, size)),
                widest_enclosure_of_a_surface(&a_surface_with_a_wide_dynamic_range(degree, size)),
            ]
            .map(|reading| reading.in_ulps_of_the_size)
            .into_iter()
            .fold(0.0, f64::max);

            assert!(
                widest <= WIDEST_ENCLOSURE_AT_AN_ORDINARY_WORKING_SIZE,
                "at degree {degree} and a working size of {size} the widest enclosure is \
                 {widest} f64 steps of that size"
            );
        }
    }
}

/// A degree-1 curve whose control points are whole numbers small enough that
/// halving a sum of two of them is exact, on knots at whole numbers.
fn a_straight_run_between_whole_numbers() -> impl Strategy<Value = BSplineCurve> {
    (2usize..=6)
        .prop_flat_map(|points| {
            let coordinates = proptest::collection::vec(-1_000_000i32..1_000_000, points * 3);
            (Just(points), coordinates)
        })
        .prop_map(|(points, coordinates)| {
            let mut knots = vec![0.0];
            knots.extend((0..points).map(|i| i as f64));
            knots.push((points - 1) as f64);

            let net: Vec<Point> = coordinates
                .chunks_exact(3)
                .map(|corner| {
                    Point::new(
                        f64::from(corner[0]),
                        f64::from(corner[1]),
                        f64::from(corner[2]),
                    )
                })
                .collect();
            let knots = KnotVector::new(1, knots).expect("clamped at both ends and rising");
            BSplineCurve::new(knots, &net, &vec![1.0; points])
                .expect("whole-number control points and weights of one")
        })
}

/// The same curve with every control point multiplied by `factor`.
fn scaled_by(curve: &BSplineCurve, factor: f64) -> BSplineCurve {
    let net: Vec<[f64; 4]> = (0..curve.control_point_count())
        .map(|i| {
            let [x, y, z, weight] = curve.homogeneous_control_point(i);
            [factor * x, factor * y, factor * z, weight]
        })
        .collect();
    BSplineCurve::try_from((curve.knots().clone(), net))
        .expect("scaling a net by a power of two keeps every weight where it was")
}
