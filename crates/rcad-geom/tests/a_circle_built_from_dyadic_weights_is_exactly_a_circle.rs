//! The closed-form axis `docs/decisions.md` §14.4 asks for, on the whole
//! circle rather than on one quarter of it.
//!
//! `BSplineCurve::circle` writes a circle down with nothing rounded away, and
//! `Sphere` and `Cylinder` answer about it through the predicate cascade. The
//! two share no lineage at all: one is de Boor over a rational net, the other is
//! `|p − c|² − r²` run through the filters. So the agreement between them is
//! worth something, and it is asserted with **no tolerance anywhere**.
//!
//! Both halves of §14.7 are here, per case, pass or fail, never a rate:
//!
//! - a point on the circle is **never certified off** the sphere of the same
//!   radius,
//! - the same point **is certified** outside a sphere half the size and inside
//!   one twice the size. A refusal there is a timid filter and a failure.
//!
//! And the two exactly-true oracles a circle brings with it: the four quarter
//! joins are its own control points bit for bit, and the two ends of the whole
//! circle are the same four numbers.

#![forbid(unsafe_code)]

mod support;

use rcad_geom::{Axis, BSplineCurve, Cylinder, Point, Sphere, Vector};
use rcad_predicates::{Decision, Policy, SideOfSurface};

use support::{extruded_one_unit_in_z, the_quarter_circle};

/// The load-bearing claim. Every point of the circle, at every parameter swept,
/// is on the sphere of the same radius — so a certified `Pos` or `Neg` is a bug
/// in the arc, in the cascade, or in both.
#[test]
fn no_point_on_a_circle_is_ever_certified_off_the_sphere_of_the_same_radius() {
    for radius in the_radii_worth_trying() {
        let circle = BSplineCurve::circle(Axis::z(), radius).unwrap();
        let ball = Sphere::new(Point::origin(), radius).expect("a positive radius");

        for u in parameters_across(circle.domain()) {
            let decision = ball.side_of(circle.evaluate_with_enclosure(u).unwrap());
            assert!(
                !is_a_certified_sign(decision),
                "at u = {u} a point exactly on the circle of radius {radius} came back {decision:?}"
            );
        }
    }
}

/// The acceptance half. Half the radius and twice the radius are nowhere near
/// the circle, and a cascade that will not say which side it is on is refusing
/// what it could have decided.
#[test]
fn every_point_on_a_circle_is_decided_against_spheres_of_clearly_different_size() {
    for radius in the_radii_worth_trying() {
        let circle = BSplineCurve::circle(Axis::z(), radius).unwrap();
        let smaller = Sphere::new(Point::origin(), radius / 2.0).expect("a positive radius");
        let bigger = Sphere::new(Point::origin(), 2.0 * radius).expect("a positive radius");

        for u in parameters_across(circle.domain()) {
            let point = circle.evaluate_with_enclosure(u).unwrap();
            assert_decided(smaller.side_of(point), Decision::Pos, u);
            assert_decided(bigger.side_of(point), Decision::Neg, u);
        }
    }
}

/// At a quarter join the interior knot repeats as often as the degree, so every
/// de Boor coefficient is exactly zero or exactly one and the blends copy rather
/// than compute. The curve is at that control point, not near it.
///
/// The whole circle closes for the same reason: its first and last control
/// points are built from the same expression, so they are the same four stored
/// numbers and the two ends of the domain read the same.
#[test]
fn a_circle_passes_exactly_through_its_control_point_at_every_quarter_join() {
    let circle = BSplineCurve::circle(Axis::z(), 3.0).unwrap();

    for (quarter, corner) in [(0.0, 0), (1.0, 2), (2.0, 4), (3.0, 6), (4.0, 8)] {
        assert_eq!(
            coordinate_bits(circle.evaluate(quarter).unwrap()),
            coordinate_bits(circle.control_point(corner)),
            "at u = {quarter} the curve is not bit for bit on control point {corner}"
        );
    }

    let (start, end) = circle.domain();
    assert_eq!(
        coordinate_bits(circle.evaluate(start).unwrap()),
        coordinate_bits(circle.evaluate(end).unwrap()),
        "the circle is clamped at both ends and the two ends are the same place"
    );
}

/// The same claim about the joins for an axis that is not a coordinate one.
///
/// A slanted circle is a rounding off the circle that was asked for — the module
/// note on `conic.rs` says so — but landing on its own control points at the
/// joins is a fact about de Boor and holds whatever the net is.
#[test]
fn a_circle_about_a_slanted_axis_still_lands_on_its_control_points_at_the_joins() {
    let slanted =
        Axis::new(Point::new(1.0, 2.0, 3.0), Vector::new(1.0, 2.0, 3.0)).expect("a real direction");
    let circle = BSplineCurve::circle(slanted, 2.0).unwrap();

    for (quarter, corner) in [(0.0, 0), (1.0, 2), (2.0, 4), (3.0, 6), (4.0, 8)] {
        assert_eq!(
            coordinate_bits(circle.evaluate(quarter).unwrap()),
            coordinate_bits(circle.control_point(corner))
        );
    }
}

/// The two ends and the four joins are exact `f64` points, so the cascade gets
/// an exact point rather than a box and has to certify `Zero`. `Undecidable`
/// there would be the exactly computable case being refused.
#[test]
fn the_quarter_joins_of_a_circle_are_decided_exactly_on_the_sphere() {
    let radius = 3.0;
    let circle = BSplineCurve::circle(Axis::z(), radius).unwrap();
    let ball = Sphere::new(Point::origin(), radius).expect("a positive radius");

    for quarter in [0.0, 1.0, 2.0, 3.0, 4.0] {
        let point = circle.evaluate_with_enclosure(quarter).unwrap();
        assert!(
            point.exact_coordinates().is_some(),
            "at u = {quarter} the curve is on a control point, so the enclosure has no width: \
             {point:?}"
        );
        assert_decided(ball.side_of(point), Decision::Zero, quarter);
    }
}

/// The surface half. The circle pulled one unit along `z` is exactly the pipe of
/// that radius about `z`, checked both ways: never certified off it, and always
/// certified against pipes of clearly different size.
#[test]
fn no_point_on_an_extruded_circle_is_ever_certified_off_the_pipe_of_the_same_radius() {
    for radius in the_radii_worth_trying() {
        let wall = extruded_one_unit_in_z(&BSplineCurve::circle(Axis::z(), radius).unwrap());
        let pipe = Cylinder::new(Axis::z(), radius).expect("a positive radius");
        let narrower = Cylinder::new(Axis::z(), radius / 2.0).expect("a positive radius");
        let wider = Cylinder::new(Axis::z(), 2.0 * radius).expect("a positive radius");

        for u in parameters_across(wall.u_knots().domain()) {
            for v in parameters_across(wall.v_knots().domain()) {
                let point = wall.evaluate_with_enclosure(u, v).unwrap();
                let decision = pipe.side_of(point);
                assert!(
                    !is_a_certified_sign(decision),
                    "at ({u}, {v}) a point exactly on the pipe of radius {radius} came back \
                     {decision:?}"
                );
                assert_decided(narrower.side_of(point), Decision::Pos, u);
                assert_decided(wider.side_of(point), Decision::Neg, u);
            }
        }
    }
}

/// The hand-written net the quarter arc used to be, kept as an assertion now
/// that `support::the_quarter_circle` is the constructor.
///
/// This is the one place the numbers are still written out by a person, so a
/// change to the constructor that quietly moved a control point or a weight has
/// something to fail against.
#[test]
fn the_quarter_arc_is_the_net_that_used_to_be_written_out_by_hand() {
    let arc = the_quarter_circle();

    assert_eq!(arc.degree(), 2);
    assert_eq!(arc.knots().knots(), &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    assert_eq!(
        [
            arc.homogeneous_control_point(0),
            arc.homogeneous_control_point(1),
            arc.homogeneous_control_point(2),
        ],
        [
            [1.0, 0.0, 0.0, 1.0],
            [1.0, 1.0, 0.0, 1.0],
            [0.0, 2.0, 0.0, 2.0]
        ],
        "control points (1,0), (1,1), (0,1) with weights 1, 1, 2"
    );
}

/// `(3/5, 4/5)` scaled by the radius, with no tolerance in the assertion.
///
/// The middle of the first quarter is exactly `(3r/5, 4r/5)`. For a radius of
/// one neither coordinate is an `f64`, so the enclosure must have width and must
/// hold the nearest `f64` strictly inside it — that settles the real claim,
/// because a box holding `0.6` strictly holds everything within half a step
/// of it.
#[test]
fn the_middle_of_the_first_quarter_encloses_three_fifths_and_four_fifths() {
    let circle = BSplineCurve::circle(Axis::z(), 1.0).unwrap();
    let [x, y, z] = circle.evaluate_with_enclosure(0.5).unwrap().enclosure();

    assert!(x.lo() < 0.6 && 0.6 < x.hi(), "{x:?} does not hold 3/5");
    assert!(y.lo() < 0.8 && 0.8 < y.hi(), "{y:?} does not hold 4/5");
    assert_eq!(z.width(), 0.0, "a flat circle has an exact z");
    assert_eq!(z.lo(), 0.0);
}

/// Radii that are exact `f64` and cover a wide range of sizes. The control
/// points are `0`, `r` and `−r`, so every one of them is exact whatever `r` is —
/// including the radii below that are not dyadic themselves.
fn the_radii_worth_trying() -> [f64; 6] {
    [1.0, 3.0, 0.1, 1e-6, 1e6, 12_345.678]
}

/// A point's three coordinates as the bits they are stored in, so `+0.0` and
/// `−0.0` are not called the same number.
fn coordinate_bits(point: Point) -> [u64; 3] {
    point.coordinates().map(f64::to_bits)
}

/// A certified sign, which is the answer a point exactly on a surface must never
/// get.
fn is_a_certified_sign(decision: Decision) -> bool {
    match decision {
        Decision::Pos | Decision::Neg => true,
        Decision::Zero | Decision::Undecidable { .. } => false,
        other => panic!("unhandled decision: {other:?}"),
    }
}

/// That `decision` is the expected answer and that `Policy::certified()` would
/// take it. The second half is what makes this the acceptance oracle rather
/// than an ordinary equality.
fn assert_decided(decision: Decision, expected: Decision, at: f64) {
    assert!(
        a_certified_policy_accepts(decision),
        "at {at} the answer is exactly computable and came back {decision:?}, which a certified \
         policy refuses"
    );
    assert_eq!(decision, expected, "at {at}");
}

/// Whether [`Policy::certified()`] would take this answer as an answer.
fn a_certified_policy_accepts(decision: Decision) -> bool {
    match decision {
        Decision::Pos | Decision::Neg | Decision::Zero => true,
        Decision::Undecidable { enclosure } => Policy::certified().wobble_is_acceptable(enclosure),
        other => panic!("unhandled decision: {other:?}"),
    }
}

/// Sixty-five evenly spaced parameters across the domain and three that are
/// nothing like round numbers, so the sweep is not all dyadic fractions where
/// the arithmetic happens to be kind. Both ends are included, because that is
/// where the enclosure has no width at all.
fn parameters_across(domain: (f64, f64)) -> Vec<f64> {
    let (lo, hi) = domain;
    let mut all: Vec<f64> = (0..=64)
        .map(|step| lo + (hi - lo) * f64::from(step) / 64.0)
        .collect();
    all.extend([1.0 / 3.0, 0.1, 0.7].map(|share: f64| lo + (hi - lo) * share));
    all
}
