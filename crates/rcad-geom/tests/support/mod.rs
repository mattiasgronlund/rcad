//! The five families of spline the M1 suites are built on.
//!
//! Four are generated and one is written out by hand. None of them are shared
//! with the extruded polygons M0 generates: those exercise a different kernel
//! and a shared generator would quietly tie the two suites together
//! (`docs/decisions.md` §13 rule 5).
//!
//! Every generated family builds its knot vector and its control net
//! **directly**, so a strategy cannot produce something the constructors
//! refuse, and each one ends in an `expect` rather than a filter. Nothing here
//! uses `prop_filter_map(… .ok())`: a suite that keeps only the inputs that were
//! accepted cannot see a valid input being wrongly refused, which is the half
//! that caught `ProfileBuilder` turning away a right triangle
//! (`docs/decisions.md` §12.3).
//!
//! Below the families are the shapes the enclosure measurement is taken on —
//! sized by working size rather than drawn at random, so a figure written down
//! in one run means the same thing in the next — and the reading itself,
//! [`WidestEnclosure`].

// Every test binary that includes this module compiles all of it and uses a
// subset, so most of what is here is dead code from any one of their points of
// view.
#![allow(dead_code)]

use core::ops::Range;

use proptest::prelude::*;
use rcad_geom::{BSplineCurve, BSplineSurface, KnotVector, Point};
use rcad_predicates::{Interval, Point as PredicatePoint};

/// Family 1: uniform clamped knots, a control net scattered about the origin,
/// every weight exactly one — the polynomial baseline.
pub fn a_uniform_polynomial_curve() -> impl Strategy<Value = BSplineCurve> {
    (1usize..=5, 1usize..=6)
        .prop_flat_map(|(degree, spans)| {
            let knots = uniform_clamped_knots(degree, spans);
            let net = a_control_net(knots.control_point_count(), -100.0..100.0);
            (Just(knots), net)
        })
        .prop_map(|(knots, points)| {
            let weights = vec![1.0; points.len()];
            curve_through(knots, &points, &weights)
        })
}

/// Family 2: interior knots repeated anywhere from once up to `degree` times —
/// the kinks, and the seam a split lands on.
pub fn a_curve_with_repeated_interior_knots() -> impl Strategy<Value = BSplineCurve> {
    (2usize..=5, 2usize..=5)
        .prop_flat_map(|(degree, spans)| {
            let multiplicities = proptest::collection::vec(1usize..=degree, spans - 1);
            (Just(degree), Just(spans), multiplicities)
        })
        .prop_flat_map(|(degree, spans, multiplicities)| {
            let knots = clamped_knots_repeating(degree, spans, &multiplicities);
            let net = a_control_net(knots.control_point_count(), -100.0..100.0);
            (Just(knots), net)
        })
        .prop_map(|(knots, points)| {
            let weights = vec![1.0; points.len()];
            curve_through(knots, &points, &weights)
        })
}

/// Family 3: interior knots one `f64` apart and control points sitting on top
/// of one another — spans too narrow to see, which nothing may snap shut.
pub fn a_curve_with_spans_too_narrow_to_see() -> impl Strategy<Value = BSplineCurve> {
    (1usize..=4, 2usize..=4)
        .prop_flat_map(|(degree, narrow_knots)| {
            let knots = clamped_knots_a_hair_apart(degree, narrow_knots);
            let net = a_control_net(knots.control_point_count(), -10.0..10.0);
            (Just(knots), net)
        })
        .prop_map(|(knots, mut points)| {
            points[1] = points[0];
            let weights = vec![1.0; points.len()];
            curve_through(knots, &points, &weights)
        })
}

/// Family 4: control points from a micrometre to a kilometre and weights across
/// six orders of magnitude, aimed straight at the precondition the published
/// evaluation bound rests on.
pub fn a_curve_spanning_a_wide_range_of_sizes() -> impl Strategy<Value = BSplineCurve> {
    (1usize..=4, 1usize..=4)
        .prop_flat_map(|(degree, spans)| {
            let knots = uniform_clamped_knots(degree, spans);
            let count = knots.control_point_count();
            let coordinates = proptest::collection::vec(a_coordinate_of_any_size(), count * 3);
            let weights = proptest::collection::vec(a_size(-3..3), count);
            (Just(knots), coordinates, weights)
        })
        .prop_map(|(knots, coordinates, weights)| {
            let points: Vec<Point> = coordinates
                .chunks_exact(3)
                .map(|corner| Point::new(corner[0], corner[1], corner[2]))
                .collect();
            curve_through(knots, &points, &weights)
        })
}

/// Family 5: the quarter of the unit circle from `(1, 0)` to `(0, 1)`, exactly.
///
/// Control points `(1,0)`, `(1,1)`, `(0,1)` with weights `1, 1, 2` on the knots
/// `[0,0,0,1,1,1]` at degree 2. Every number in it is dyadic, so the curve is
/// the unit circle with nothing rounded away: at `u = 1/4` it is at
/// `(15/17, 8/17)` and at `u = 1/2` at `(3/5, 4/5)`, both exactly on the circle
/// and neither of them an `f64`. That is what makes it the case
/// `docs/decisions.md` §14.7 asks for — a point provably on a surface that no
/// `f64` can name.
pub fn the_quarter_circle() -> BSplineCurve {
    let knots = KnotVector::new(2, vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0])
        .expect("a clamped single-span quadratic");
    let corners = [
        Point::new(1.0, 0.0, 0.0),
        Point::new(1.0, 1.0, 0.0),
        Point::new(0.0, 1.0, 0.0),
    ];
    curve_through(knots, &corners, &[1.0, 1.0, 2.0])
}

/// Family 5: [`the_quarter_circle`] pulled one unit along `z`.
///
/// Degree 2 across the arc and degree 1 up the extrusion. Every point on it is
/// exactly one unit from the `z` axis, which is what `Cylinder::side_of` is
/// asked about.
pub fn the_quarter_circle_extruded_one_unit_in_z() -> BSplineSurface {
    let arc = the_quarter_circle();
    let up = KnotVector::new(1, vec![0.0, 0.0, 1.0, 1.0]).expect("one straight span");

    let mut corners = Vec::new();
    let mut weights = Vec::new();
    for i in 0..arc.control_point_count() {
        let bottom = arc.control_point(i);
        corners.push(bottom);
        corners.push(Point::new(bottom.x(), bottom.y(), bottom.z() + 1.0));
        weights.push(arc.weight(i));
        weights.push(arc.weight(i));
    }

    BSplineSurface::new(arc.knots().clone(), up, &corners, &weights)
        .expect("a valid curve swept along a straight span is a valid surface")
}

/// Family 5: the octant of the unit sphere between `(1,0,0)`, `(0,1,0)` and
/// `(0,0,1)`, exactly.
///
/// The quarter arc from `(1,0,0)` to `(0,0,1)` turned a quarter way round the
/// `z` axis. Piegl and Tiller's revolution puts the circle's own weights
/// `1, 1, 2` on each row of the generatrix, so every weight in the grid is a
/// product of two of them — `1, 1, 2, 1, 1, 2, 2, 2, 4` — and the whole net
/// stays dyadic. The top row is the pole three times over, which is a
/// degenerate edge and not a problem for evaluation: the weights are still
/// above zero.
///
/// At `u = v = 1/2` it is at `(9/25, 12/25, 4/5)`, whose squares add up to
/// exactly one and none of which is an `f64`.
pub fn the_octant_of_the_unit_sphere() -> BSplineSurface {
    let across = KnotVector::new(2, vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0])
        .expect("a clamped single-span quadratic");

    let corners = [
        Point::new(1.0, 0.0, 0.0),
        Point::new(1.0, 1.0, 0.0),
        Point::new(0.0, 1.0, 0.0),
        Point::new(1.0, 0.0, 1.0),
        Point::new(1.0, 1.0, 1.0),
        Point::new(0.0, 1.0, 1.0),
        Point::new(0.0, 0.0, 1.0),
        Point::new(0.0, 0.0, 1.0),
        Point::new(0.0, 0.0, 1.0),
    ];
    let weights = [1.0, 1.0, 2.0, 1.0, 1.0, 2.0, 2.0, 2.0, 4.0];

    BSplineSurface::new(across.clone(), across, &corners, &weights)
        .expect("nine points, nine positive weights")
}

/// Family 5: a straight run whose middle is a point on a sphere too big for
/// `f64` to place without exact arithmetic.
///
/// The two ends are `(0,0,0)` and twice the legs of the Pythagorean triple
/// built from `m = 12345, n = 6789`, so at `u = 1/2` the curve is exactly at
/// `(106308504, 167620410, 0)` — `198489546` from the origin, with squares
/// that run past 53 bits. Every number on the way there is a whole one, so the
/// enclosure at the middle has no width at all, and that is what lets level 3
/// of the cascade see it.
pub fn a_straight_run_through_a_point_only_exact_arithmetic_can_place() -> BSplineCurve {
    let knots = KnotVector::new(1, vec![0.0, 0.0, 1.0, 1.0]).expect("one straight span");
    let ends = [
        Point::origin(),
        Point::new(2.0 * 106_308_504.0, 2.0 * 167_620_410.0, 0.0),
    ];
    curve_through(knots, &ends, &[1.0, 1.0])
}

/// How far the middle of that run is from the origin.
pub const THE_DISTANCE_ONLY_EXACT_ARITHMETIC_CAN_PLACE: f64 = 198_489_546.0;

/// A curve of `degree` whose control points fill a box `working_size` across,
/// with every weight one — the well-behaved shape, for measuring against.
pub fn a_well_conditioned_curve(degree: usize, working_size: f64) -> BSplineCurve {
    let knots = uniform_clamped_knots(degree, 3);
    let count = knots.control_point_count();
    let points: Vec<Point> = (0..count)
        .map(|i| {
            Point::new(
                scattered(3 * i) * working_size,
                scattered(3 * i + 1) * working_size,
                scattered(3 * i + 2) * working_size,
            )
        })
        .collect();
    curve_through(knots, &points, &vec![1.0; count])
}

/// A curve of `degree` whose control points run from `working_size` down to a
/// millionth of it and whose weights cover six orders of magnitude.
pub fn a_curve_with_a_wide_dynamic_range(degree: usize, working_size: f64) -> BSplineCurve {
    let knots = uniform_clamped_knots(degree, 3);
    let count = knots.control_point_count();
    let points: Vec<Point> = (0..count)
        .map(|i| {
            Point::new(
                spread_over_six_decades(3 * i, working_size),
                spread_over_six_decades(3 * i + 1, working_size),
                spread_over_six_decades(3 * i + 2, working_size),
            )
        })
        .collect();
    let weights: Vec<f64> = (0..count).map(|i| 10f64.powi(i as i32 % 7 - 3)).collect();
    curve_through(knots, &points, &weights)
}

/// The tensor-product patch of two [`a_well_conditioned_curve`] nets.
pub fn a_well_conditioned_surface(degree: usize, working_size: f64) -> BSplineSurface {
    let knots = uniform_clamped_knots(degree, 3);
    let count = knots.control_point_count();
    let points: Vec<Point> = (0..count * count)
        .map(|i| {
            Point::new(
                scattered(3 * i) * working_size,
                scattered(3 * i + 1) * working_size,
                scattered(3 * i + 2) * working_size,
            )
        })
        .collect();
    BSplineSurface::new(knots.clone(), knots, &points, &vec![1.0; count * count])
        .expect("a square grid of finite points and weights of one")
}

/// The patch version of [`a_curve_with_a_wide_dynamic_range`].
pub fn a_surface_with_a_wide_dynamic_range(degree: usize, working_size: f64) -> BSplineSurface {
    let knots = uniform_clamped_knots(degree, 3);
    let count = knots.control_point_count();
    let points: Vec<Point> = (0..count * count)
        .map(|i| {
            Point::new(
                spread_over_six_decades(3 * i, working_size),
                spread_over_six_decades(3 * i + 1, working_size),
                spread_over_six_decades(3 * i + 2, working_size),
            )
        })
        .collect();
    let weights: Vec<f64> = (0..count * count)
        .map(|i| 10f64.powi(i as i32 % 7 - 3))
        .collect();
    BSplineSurface::new(knots.clone(), knots, &points, &weights)
        .expect("a square grid of finite points and positive weights")
}

/// Every parameter this curve will take one more copy of.
///
/// The interior knots that have not yet reached the degree, and the middle of
/// every span with room in it. A curve with no interior knots still has the
/// middle of its one span, so this is never empty.
pub fn parameters_a_knot_can_be_inserted_at(curve: &BSplineCurve) -> Vec<f64> {
    let degree = curve.degree();
    let knots = curve.knots();
    let all = knots.knots();

    let mut candidates: Vec<f64> = all[degree + 1..all.len() - degree - 1].to_vec();
    for span in degree..all.len() - degree - 1 {
        if all[span] < all[span + 1] {
            candidates.push(all[span] + (all[span + 1] - all[span]) / 2.0);
        }
    }
    candidates.retain(|u| knots.contains(*u) && knots.multiplicity_of(*u) < degree);
    candidates
}

/// Every parameter this curve can be cut at: anywhere strictly inside its
/// domain, which is where both halves have a curve in them.
pub fn parameters_a_curve_can_be_split_at(curve: &BSplineCurve) -> Vec<f64> {
    let degree = curve.degree();
    let all = curve.knots().knots();
    let (lo, hi) = curve.domain();

    let mut candidates: Vec<f64> = all[degree + 1..all.len() - degree - 1].to_vec();
    for span in degree..all.len() - degree - 1 {
        if all[span] < all[span + 1] {
            candidates.push(all[span] + (all[span + 1] - all[span]) / 2.0);
        }
    }
    candidates.retain(|u| lo < *u && *u < hi);
    candidates
}

/// A point's three coordinates as the bits they are stored in.
///
/// Comparing points with `==` would call `+0.0` and `-0.0` the same number.
/// The refinement oracles claim more than that — that untouched numbers are
/// *copied* — so they compare what is actually stored.
pub fn coordinate_bits(point: Point) -> [u64; 3] {
    point.coordinates().map(f64::to_bits)
}

/// Whether two curves are the same curve down to the last bit: the same
/// degree, the same knots, and the same weighted control net.
pub fn the_same_curve_to_the_last_bit(one: &BSplineCurve, other: &BSplineCurve) -> bool {
    let same_knots = one.degree() == other.degree()
        && one.knots().knots().len() == other.knots().knots().len()
        && core::iter::zip(one.knots().knots(), other.knots().knots())
            .all(|(a, b)| a.to_bits() == b.to_bits());

    same_knots
        && one.control_point_count() == other.control_point_count()
        && (0..one.control_point_count()).all(|i| {
            core::iter::zip(
                one.homogeneous_control_point(i),
                other.homogeneous_control_point(i),
            )
            .all(|(a, b)| a.to_bits() == b.to_bits())
        })
}

/// Whether the checking constructors would build this curve from scratch.
///
/// The knot vector goes back through [`KnotVector::new`] and the net through
/// [`BSplineCurve::new`], which takes affine points and weights and is
/// therefore a genuinely different way in than the one refinement used. A
/// refinement that produces a curve the kernel would refuse to build is a bug,
/// and this is the half of the suite that can see it
/// (`docs/decisions.md` §14.7).
pub fn the_kernel_would_build_this_curve_again(curve: &BSplineCurve) -> bool {
    let Ok(knots) = KnotVector::new(curve.knots().degree(), curve.knots().knots().to_vec()) else {
        return false;
    };
    let points: Vec<Point> = (0..curve.control_point_count())
        .map(|i| curve.control_point(i))
        .collect();
    let weights: Vec<f64> = (0..curve.control_point_count())
        .map(|i| curve.weight(i))
        .collect();
    BSplineCurve::new(knots, &points, &weights).is_ok()
}

/// The widest enclosure something hands back anywhere across its domain, read
/// two ways.
///
/// Both readings are needed, and the second is the one to judge by.
///
/// A width in ulps of the coordinate is the natural thing to ask for and it
/// falls apart exactly where a control net has signs in it: a coordinate on its
/// way through zero divides an ordinary absolute width by a number heading for
/// nothing, and the answer runs to millions of ulps while the box itself is a
/// few `f64` steps across. That is the same fact `EVALUATION_ROUNDOFF`'s
/// derivation opens with — a bound here is absolute in the size of the input,
/// never relative to the answer, because `(+1) + (−1)` comes out as zero with a
/// rounding still attached to it.
#[derive(Clone, Copy, Debug)]
pub struct WidestEnclosure {
    /// In `f64` steps of the coordinate it encloses.
    pub in_ulps_of_the_coordinate: f64,
    /// In `f64` steps of the largest control coordinate the thing is built
    /// from — the model's own working size, and what a bound on de Boor is
    /// stated in.
    pub in_ulps_of_the_size: f64,
}

/// The widest enclosure a curve hands back anywhere across its domain.
///
/// Both ends included: a clamped curve is exact at the ends, so a sweep that
/// stopped short of them would miss the one place there is no width at all.
pub fn widest_enclosure_of_a_curve(curve: &BSplineCurve) -> WidestEnclosure {
    let size = (0..curve.control_point_count())
        .map(|i| curve.control_point(i).largest_coordinate_magnitude())
        .fold(0.0, f64::max);

    let mut widest = WidestEnclosure {
        in_ulps_of_the_coordinate: 0.0,
        in_ulps_of_the_size: 0.0,
    };
    for u in parameters_across(curve.domain(), PARAMETERS_PER_SWEEP) {
        let point = curve
            .evaluate_with_enclosure(u)
            .expect("a parameter from the curve's own domain");
        widen(&mut widest, point, size);
    }
    widest
}

/// The same across a patch, in both directions at once.
pub fn widest_enclosure_of_a_surface(surface: &BSplineSurface) -> WidestEnclosure {
    let rows = surface.u_knots().control_point_count();
    let columns = surface.v_knots().control_point_count();
    let size = (0..rows)
        .flat_map(|i| (0..columns).map(move |j| (i, j)))
        .map(|(i, j)| surface.control_point(i, j).largest_coordinate_magnitude())
        .fold(0.0, f64::max);

    let mut widest = WidestEnclosure {
        in_ulps_of_the_coordinate: 0.0,
        in_ulps_of_the_size: 0.0,
    };
    for u in parameters_across(surface.u_knots().domain(), PARAMETERS_PER_GRID_SIDE) {
        for v in parameters_across(surface.v_knots().domain(), PARAMETERS_PER_GRID_SIDE) {
            let point = surface
                .evaluate_with_enclosure(u, v)
                .expect("a parameter from the surface's own domain");
            widen(&mut widest, point, size);
        }
    }
    widest
}

/// `widest` grown to cover `point`, whose model is `size` across.
fn widen(widest: &mut WidestEnclosure, point: PredicatePoint, size: f64) {
    let step_at_the_size = size.next_up() - size;
    for side in point.enclosure() {
        widest.in_ulps_of_the_coordinate =
            widest.in_ulps_of_the_coordinate.max(width_in_ulps(side));
        widest.in_ulps_of_the_size = widest
            .in_ulps_of_the_size
            .max(side.width() / step_at_the_size);
    }
}

/// Enough parameters to catch a bad span without turning a measurement into a
/// wait.
const PARAMETERS_PER_SWEEP: usize = 65;

/// The same for a patch, where the count is squared.
const PARAMETERS_PER_GRID_SIDE: usize = 17;

/// `steps` parameters evenly spaced across `domain`, both ends included.
pub fn parameters_across(domain: (f64, f64), steps: usize) -> impl Iterator<Item = f64> {
    let (lo, hi) = domain;
    (0..steps).map(move |step| lo + (hi - lo) * step as f64 / (steps - 1) as f64)
}

/// How many `f64` steps wide an enclosure is, measured at the size of the
/// number it encloses.
///
/// The step is read off the larger end of the box, so what comes back is
/// roughly how many of that coordinate's last bits the enclosure is unsure
/// about. It is the reading a person asks for first, and [`WidestEnclosure`]
/// says why it is not the one to judge by. A box of no width comes back as
/// zero however small the coordinate is, because the division is `0 / step`.
pub fn width_in_ulps(side: Interval) -> f64 {
    let magnitude = side.lo().abs().max(side.hi().abs());
    side.width() / (magnitude.next_up() - magnitude)
}

/// Clamped knots with `spans` interior spans, all of the same width.
///
/// `[0]*(p+1) ++ [1, 2, …, spans-1] ++ [spans]*(p+1)`: every knot is finite, the
/// sequence rises, both clamps are exactly `p + 1` long, every interior knot
/// appears once, and every span has width one.
fn uniform_clamped_knots(degree: usize, spans: usize) -> KnotVector {
    clamped_knots_repeating(degree, spans, &vec![1; spans - 1])
}

/// Clamped knots where the `k`th interior knot is repeated
/// `multiplicities[k]` times.
///
/// Every multiplicity has to be between 1 and `degree`, which is the most an
/// interior knot may repeat.
fn clamped_knots_repeating(degree: usize, spans: usize, multiplicities: &[usize]) -> KnotVector {
    assert_eq!(multiplicities.len(), spans - 1);
    let mut knots = vec![0.0; degree + 1];
    for (interior, &times) in multiplicities.iter().enumerate() {
        assert!(
            (1..=degree).contains(&times),
            "an interior knot may repeat up to the degree, not {times} times"
        );
        knots.extend(core::iter::repeat_n(interior as f64 + 1.0, times));
    }
    knots.extend(core::iter::repeat_n(spans as f64, degree + 1));

    KnotVector::new(degree, knots).expect("clamped at both ends, rising, and no repeat past degree")
}

/// Clamped knots over `[0, 1]` whose interior knots are consecutive `f64`,
/// starting at a half.
///
/// The span from 0 to a half carries all the width there is; the ones after it
/// are as narrow as `f64` can make them without shutting.
fn clamped_knots_a_hair_apart(degree: usize, narrow_knots: usize) -> KnotVector {
    let mut knots = vec![0.0; degree + 1];
    let mut interior = 0.5f64;
    for _ in 0..narrow_knots {
        knots.push(interior);
        interior = interior.next_up();
    }
    knots.extend(core::iter::repeat_n(1.0, degree + 1));

    KnotVector::new(degree, knots)
        .expect("consecutive f64 are distinct, so every interior knot appears once")
}

/// A repeatable spread of numbers between −1 and 1.
///
/// A measured width is only worth writing down if the next run measures the
/// same thing, so the sized families above are laid out by an arithmetic that
/// has no seed to lose. This is the multiplier from Knuth's table; nothing
/// here needs it to be a good generator, only a fixed one.
fn scattered(step: usize) -> f64 {
    let mixed = (step as u64)
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    2.0 * ((mixed >> 11) as f64 / (1u64 << 53) as f64) - 1.0
}

/// A coordinate somewhere between a millionth of `working_size` and
/// `working_size` itself, either side of zero.
fn spread_over_six_decades(step: usize, working_size: f64) -> f64 {
    scattered(step) * working_size * 10f64.powi(-((step % 7) as i32))
}

/// A control net of `count` points with every coordinate drawn from `span`.
fn a_control_net(count: usize, span: Range<f64>) -> impl Strategy<Value = Vec<Point>> {
    proptest::collection::vec((span.clone(), span.clone(), span), count).prop_map(|corners| {
        corners
            .into_iter()
            .map(|(x, y, z)| Point::new(x, y, z))
            .collect()
    })
}

/// A coordinate anywhere from a micrometre to a kilometre, either side of zero.
fn a_coordinate_of_any_size() -> impl Strategy<Value = f64> {
    (a_size(-6..6), any::<bool>()).prop_map(|(size, negative)| if negative { -size } else { size })
}

/// A number between `10^exponents.start` and `10^exponents.end`, with every
/// order of magnitude in between getting equal billing.
fn a_size(exponents: Range<i32>) -> impl Strategy<Value = f64> {
    (exponents, 1.0f64..10.0).prop_map(|(exponent, mantissa)| mantissa * 10f64.powi(exponent))
}

/// The curve through a net that every family here builds to be acceptable.
///
/// # Panics
///
/// If the kernel refuses it. That is the assertion, not an input to be skipped:
/// each family's own note says why what it builds is sound, and a refusal here
/// is the kernel turning away geometry it should have taken.
fn curve_through(knots: KnotVector, control_points: &[Point], weights: &[f64]) -> BSplineCurve {
    BSplineCurve::new(knots, control_points, weights)
        .expect("every family here builds a net the kernel accepts")
}
