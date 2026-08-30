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
//!
//! It is read two ways, because one reading stopped covering everything. Steps
//! of the model's working size is the figure a person asks for, and it is held
//! to a tight ceiling on the shapes where it means something. On a curve whose
//! weights span six orders of magnitude over knots spaced a million to one it
//! measures the curve's own conditioning rather than the arithmetic, so that
//! family is held to the share of the published bound the box uses up instead.
//! Both constants below say what was measured and over how many draws.

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

/// The same for a generated curve whose weights are all one — families 1 to 3.
///
/// **Measured over a hundred thousand draws of each**: the worst was 32 steps
/// for family 1, 55 for family 2 and 46 for family 3, with 99th percentiles of
/// 26, 50 and 34. The tail is thin, so this is a little over twice the worst,
/// rounded to a power of two — the same shape of safety factor as the constant
/// above and tight enough that a doubling of the width shows up here at once.
///
/// If it fails, the answer is not a bigger number. It is a real widening to
/// find, or it is the news that M3's certified marching needs centred forms —
/// which is the thing these figures were measured to decide.
const WIDEST_ENCLOSURE_FROM_A_CURVE_OF_PLAIN_WEIGHTS: f64 = 128.0;

/// What a curve whose weights span six orders of magnitude is held to instead:
/// how much of the published bound the box uses up.
///
/// # Why family 4 gets a different reading
///
/// A width in steps of the working size stopped meaning what it meant. The old
/// figure — worst 157 over a hundred thousand draws — was measured on knots
/// that were always `0, 1, 2, 3`, where an evenly spaced sweep lands at tidy
/// fractions of every span. The generators now put spans of a million steps
/// next to spans of one, so a parameter can land a hair below a knot in a span
/// a million times wider than its neighbour. There the de Boor coefficient
/// comes out a hair under one, `1 − share` is a tiny number carrying the
/// coefficient's own half-ulp, and against control weights in a ratio of tens
/// of thousands the point is genuinely that uncertain. **Measured over a
/// million draws of family 4**: median 15 steps, 99.9th 986, worst 66,120.
///
/// That is the curve being ill-conditioned, not the arithmetic being sloppy,
/// and the box is honest about it. A ceiling in steps of the size would have to
/// be near a million to stop flickering, which would assert nothing. It is
/// worth knowing on its own account: certified marching along a rational curve
/// of six decades of weight, on knots spaced like that, is the case M3 will
/// need centred forms for.
///
/// So the ceiling for that family is scale-free instead. The box is compared
/// with [`rcad_geom::EVALUATION_ROUNDOFF`]'s published bound at the same
/// parameter, which carries the weights and the knots with it. **Measured over
/// two hundred thousand draws of each family**: the box's half-width never
/// passed 0.082 of the bound in family 1, 0.145 in family 2, 0.121 in family 3
/// or 0.107 in family 4. This is a little over three times the worst of those.
///
/// The bound is a yardstick here and nowhere else — nothing intersects it with
/// the enclosure at runtime, for the reason the test below its name gives.
const WIDEST_SHARE_OF_THE_PUBLISHED_BOUND: f64 = 0.5;

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

    /// The ceiling in steps of the working size, over the families whose
    /// weights are all one.
    #[test]
    fn no_generated_curve_of_plain_weights_has_an_enclosure_wider_than_the_ceiling(
        curve in prop_oneof![
            a_uniform_polynomial_curve(),
            a_curve_with_repeated_interior_knots(),
            a_curve_with_spans_too_narrow_to_see(),
        ]
    ) {
        let widest = widest_enclosure_of_a_curve(&curve).in_ulps_of_the_size;
        prop_assert!(
            widest <= WIDEST_ENCLOSURE_FROM_A_CURVE_OF_PLAIN_WEIGHTS,
            "a degree {} curve encloses its point in {widest} f64 steps of its own size",
            curve.degree()
        );
    }

    /// The scale-free ceiling, over all four families: the box never uses up
    /// more than a fraction of what the published bound allows.
    ///
    /// This is the one family 4 is judged by, and the constant's note says why
    /// steps of the working size stopped being the right reading for it.
    #[test]
    fn no_generated_curve_encloses_its_point_more_loosely_than_the_published_bound(
        curve in prop_oneof![
            a_uniform_polynomial_curve(),
            a_curve_with_repeated_interior_knots(),
            a_curve_with_spans_too_narrow_to_see(),
            a_curve_spanning_a_wide_range_of_sizes(),
        ]
    ) {
        for u in parameters_across(curve.domain(), 65) {
            // A `None` is the bound's precondition failing, which is a real
            // answer about the input and not something to assert against.
            let Some(allowed) = curve.evaluation_roundoff_at(u).unwrap() else {
                continue;
            };
            for side in curve.evaluate_with_enclosure(u).unwrap().enclosure() {
                // Half the box either side of the point is what the bound —
                // a distance from the true point — is comparable with.
                let share = side.width() / 2.0 / allowed;
                prop_assert!(
                    share <= WIDEST_SHARE_OF_THE_PUBLISHED_BOUND,
                    "at {u} a degree {} curve encloses its point in {share} of the {allowed} \
                     the published bound allows",
                    curve.degree()
                );
            }
        }
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
