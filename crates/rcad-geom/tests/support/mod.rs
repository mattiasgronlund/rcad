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

// Families 1 to 4 are consumed in full by the evaluation, knot-insertion and
// enclosure stages; this stage only asserts that what they build is accepted.
#![allow(dead_code)]

use core::ops::Range;

use proptest::prelude::*;
use rcad_geom::{BSplineCurve, BSplineSurface, KnotVector, Point};

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
