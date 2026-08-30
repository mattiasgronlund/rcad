//! The acceptance oracle (`docs/decisions.md` §14.7, §15.8).
//!
//! Every other refusal-shaped test in this tree checks that bad input is
//! refused. None of them checks that a refusal was **deserved** rather than a
//! timid filter, and without that a cascade which refuses far more often than
//! it must passes every oracle on the list.
//!
//! So this file argues both halves, per case, pass or fail, never a rate:
//!
//! - **Nothing on a curved surface is ever certified off it.** The quarter
//!   circle here is the unit circle exactly, so a point on it is on the sphere
//!   whatever parameter it came from. A certified `Pos` or `Neg` is a bug.
//! - **Everything exactly computable is decided.** A point comfortably off a
//!   surface, and a point whose coordinates are `f64` and land exactly on one,
//!   must come back with a certified sign. A refusal there is the timid filter,
//!   and it is a failure.
//!
//! The negative control is the whole argument in one test: the same point, at
//! the same parameter, handed to the same predicate two ways. As an exact `f64`
//! triple it is certified off a circle it is exactly on — an exact answer to
//! the wrong question (§2.5). As an enclosure it is not. Neither assertion
//! needs a tolerance anywhere, which is the point of building the oracle out of
//! curves whose exact answer is known.

#![forbid(unsafe_code)]

mod support;

use rcad_geom::{
    Axis, BSplineCurve, BSplineSurface, Cylinder, KnotVector, Plane, Point, Sphere, Vector,
};
use rcad_predicates::{Decision, Interval, Point as PredicatePoint, Policy, SideOfSurface};

use support::{
    THE_DISTANCE_ONLY_EXACT_ARITHMETIC_CAN_PLACE,
    a_straight_run_through_a_point_only_exact_arithmetic_can_place, the_octant_of_the_unit_sphere,
    the_quarter_circle, the_quarter_circle_extruded_one_unit_in_z,
};

/// The negative control, and the reason the rest of the file is written the way
/// it is.
///
/// `(3/5, 4/5)` is exactly on the unit circle and neither coordinate is an
/// `f64`. Handed over as [`PredicatePoint::Exact`], the cascade answers exactly
/// — about the `f64` point it was given, which is a hair off the circle. It
/// certifies a side. That is not a bug in the cascade; it is the wrong question
/// reaching it, and it is what an `|f(p)| < tol` oracle degrades into.
///
/// The same evaluation as an enclosure cannot be certified off the circle, and
/// no tolerance appears in either half.
#[test]
fn the_same_point_on_the_circle_is_certified_off_it_as_an_f64_and_never_as_an_enclosure() {
    let arc = the_quarter_circle();
    let unit_sphere = Sphere::new(Point::origin(), 1.0).expect("a ball of radius one");

    let as_an_exact_triple = PredicatePoint::from(arc.evaluate(0.5).unwrap().coordinates());
    match unit_sphere.side_of(as_an_exact_triple) {
        Decision::Pos | Decision::Neg => (),
        other => panic!(
            "the f64 nearest (3/5, 4/5) is not on the unit circle, so exact arithmetic about it \
             has to certify a side: {other:?}"
        ),
    }

    let as_an_enclosure = arc.evaluate_with_enclosure(0.5).unwrap();
    let decision = unit_sphere.side_of(as_an_enclosure);
    assert!(
        !is_a_certified_sign(decision),
        "this point is exactly on the unit circle and came back {decision:?}"
    );
}

/// The same claim across the whole arc rather than at the one parameter that
/// makes the prettiest fraction.
#[test]
fn no_point_on_the_quarter_circle_is_ever_certified_off_the_unit_sphere() {
    let arc = the_quarter_circle();
    let unit_sphere = Sphere::new(Point::origin(), 1.0).expect("a ball of radius one");

    for u in parameters_across_the_domain() {
        let decision = unit_sphere.side_of(arc.evaluate_with_enclosure(u).unwrap());
        assert!(
            !is_a_certified_sign(decision),
            "at u = {u} a point exactly on the unit circle came back {decision:?}"
        );
    }
}

/// The surface half of the same thing. The arc pulled along `z` is exactly the
/// unit pipe about `z`, so no point on it may be certified off it either.
#[test]
fn no_point_on_the_extruded_circle_is_ever_certified_off_the_unit_pipe() {
    let wall = the_quarter_circle_extruded_one_unit_in_z();
    let pipe = Cylinder::new(Axis::z(), 1.0).expect("a pipe of radius one");

    for u in parameters_across_the_domain() {
        for v in parameters_across_the_domain() {
            let decision = pipe.side_of(wall.evaluate_with_enclosure(u, v).unwrap());
            assert!(
                !is_a_certified_sign(decision),
                "at ({u}, {v}) a point exactly on the unit pipe came back {decision:?}"
            );
        }
    }
}

/// The stretch goal: a patch that is a piece of a sphere rather than a piece of
/// a pipe.
///
/// The dyadic weights survive the revolution — each row of the generatrix
/// carries the circle's own `1, 1, 2`, so every weight in the grid is a product
/// of two dyadic numbers and the octant is the unit sphere exactly.
#[test]
fn no_point_on_the_octant_is_ever_certified_off_the_unit_sphere() {
    let octant = the_octant_of_the_unit_sphere();
    let unit_sphere = Sphere::new(Point::origin(), 1.0).expect("a ball of radius one");

    for u in parameters_across_the_domain() {
        for v in parameters_across_the_domain() {
            let decision = unit_sphere.side_of(octant.evaluate_with_enclosure(u, v).unwrap());
            assert!(
                !is_a_certified_sign(decision),
                "at ({u}, {v}) a point exactly on the unit sphere came back {decision:?}"
            );
        }
    }
}

/// The acceptance half. A point on the unit circle is a long way outside a ball
/// half the size and a long way inside one twice the size, and a cascade that
/// will not say so is a filter refusing what it could have decided.
#[test]
fn a_point_on_the_circle_is_decided_against_every_sphere_that_is_not_the_unit_one() {
    let arc = the_quarter_circle();
    let smaller = Sphere::new(Point::origin(), 0.5).expect("a ball of half the radius");
    let bigger = Sphere::new(Point::origin(), 2.0).expect("a ball of twice the radius");

    for u in parameters_across_the_domain() {
        let point = arc.evaluate_with_enclosure(u).unwrap();
        assert_decided(smaller.side_of(point), Decision::Pos, u);
        assert_decided(bigger.side_of(point), Decision::Neg, u);
    }
}

/// The same arc scaled up by four is a circle of radius four, every point of
/// which is outside the unit sphere by a margin nothing should struggle with.
#[test]
fn a_point_on_a_circle_four_times_the_size_is_decided_outside_the_unit_sphere() {
    let arc = scaled_by(&the_quarter_circle(), 4.0);
    let unit_sphere = Sphere::new(Point::origin(), 1.0).expect("a ball of radius one");

    for u in parameters_across_the_domain() {
        let point = arc.evaluate_with_enclosure(u).unwrap();
        assert_decided(unit_sphere.side_of(point), Decision::Pos, u);
    }
}

/// Both ends of the quarter circle are its own control points, bit for bit, so
/// the enclosure there has no width and the cascade gets an exact point. Those
/// two land exactly on the unit sphere, and `Zero` is the answer that has to
/// come back — not `Undecidable`, which would mean the exactly computable case
/// was refused.
#[test]
fn the_two_ends_of_the_quarter_circle_are_decided_exactly_on_the_unit_sphere() {
    let arc = the_quarter_circle();
    let unit_sphere = Sphere::new(Point::origin(), 1.0).expect("a ball of radius one");

    for u in [0.0, 1.0] {
        let point = arc.evaluate_with_enclosure(u).unwrap();
        assert!(
            point.exact_coordinates().is_some(),
            "the end of a clamped curve is a control point, so at u = {u} the enclosure has no \
             width: {point:?}"
        );
        assert_decided(unit_sphere.side_of(point), Decision::Zero, u);
    }
}

/// The corners of the extruded arc, the same way, against the pipe.
#[test]
fn the_corners_of_the_extruded_circle_are_decided_exactly_on_the_unit_pipe() {
    let wall = the_quarter_circle_extruded_one_unit_in_z();
    let pipe = Cylinder::new(Axis::z(), 1.0).expect("a pipe of radius one");

    for u in [0.0, 1.0] {
        for v in [0.0, 1.0] {
            let point = wall.evaluate_with_enclosure(u, v).unwrap();
            assert!(point.exact_coordinates().is_some(), "a corner has no width");
            assert_decided(pipe.side_of(point), Decision::Zero, u);
        }
    }
}

/// The case that says an exact enclosure is worth having.
///
/// A degree-1 curve through whole-number control points, read at a half, is at
/// a whole-number point — so the enclosure has no width, the cascade gets an
/// exact point, and **level 3 can run**. The point is on a sphere whose squares
/// run past 53 bits, so nothing below level 3 can place it: had the enclosure
/// come back as a zero-width box instead of an exact point, this would be
/// `Undecidable` and the acceptance oracle would fail.
#[test]
fn a_degree_one_curve_at_an_exact_parameter_reaches_the_exact_level_of_the_cascade() {
    let run = a_straight_run_through_a_point_only_exact_arithmetic_can_place();
    let big_sphere = Sphere::new(
        Point::origin(),
        THE_DISTANCE_ONLY_EXACT_ARITHMETIC_CAN_PLACE,
    )
    .expect("a positive radius");

    let point = run.evaluate_with_enclosure(0.5).unwrap();
    let coordinates = point
        .exact_coordinates()
        .expect("every step from a whole-number net at a half is exact");
    assert_eq!(coordinates, [106_308_504.0, 167_620_410.0, 0.0]);
    assert_decided(big_sphere.side_of(point), Decision::Zero, 0.5);

    let as_a_box_of_no_width = PredicatePoint::Constructed(coordinates.map(Interval::exactly));
    assert!(
        !a_certified_policy_accepts(big_sphere.side_of(as_a_box_of_no_width)),
        "the very same numbers in a box of no width are refused, which is what makes the exact \
         point above the thing that decides this case"
    );
}

/// A flat patch is exactly a plane, and a point on it is decided on it however
/// awkward the numbers along the plane are.
///
/// The corners sit in `z = 0` with weights that make `x` and `y` come out as
/// numbers no `f64` can name — but the `z` numerator is exactly zero all the
/// way through, so `z` divides exactly and the point is certified on the plane.
/// The two planes a unit either side of it are certified too.
#[test]
fn a_flat_patch_is_decided_on_the_plane_it_lies_in() {
    let patch = a_flat_patch_with_lopsided_weights();
    let flat = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
        .expect("a normal of length one");
    let above = Plane::from_point_and_normal(Point::new(0.0, 0.0, 1.0), Vector::new(0.0, 0.0, 1.0))
        .expect("a normal of length one");

    for u in parameters_across_the_domain() {
        for v in parameters_across_the_domain() {
            let point = patch.evaluate_with_enclosure(u, v).unwrap();
            assert_decided(flat.side_of_point(point), Decision::Zero, u);
            assert_decided(above.side_of_point(point), Decision::Neg, u);
        }
    }
}

/// A certified sign, which is the answer a point exactly on a surface must
/// never get.
fn is_a_certified_sign(decision: Decision) -> bool {
    match decision {
        Decision::Pos | Decision::Neg => true,
        Decision::Zero | Decision::Undecidable { .. } => false,
        other => panic!("unhandled decision: {other:?}"),
    }
}

/// That `decision` is the expected answer and that `Policy::certified()` would
/// take it.
///
/// The second half is what makes this the acceptance oracle rather than an
/// ordinary equality: a refusal here is a filter turning away a case whose
/// answer was there to be had.
fn assert_decided(decision: Decision, expected: Decision, at: f64) {
    assert!(
        a_certified_policy_accepts(decision),
        "at {at} the answer is exactly computable and came back {decision:?}, which a certified \
         policy refuses"
    );
    assert_eq!(decision, expected, "at {at}");
}

/// Whether [`Policy::certified()`] would take this answer as an answer.
///
/// It allows no wobble at all, so the units an enclosure is carrying never come
/// into it (`docs/decisions.md` §15.2): a refusal of any width is a refusal,
/// and anything the cascade proved stands.
fn a_certified_policy_accepts(decision: Decision) -> bool {
    match decision {
        Decision::Pos | Decision::Neg | Decision::Zero => true,
        Decision::Undecidable { enclosure } => Policy::certified().wobble_is_acceptable(enclosure),
        other => panic!("unhandled decision: {other:?}"),
    }
}

/// Thirty-three evenly spaced parameters and three that are nothing like a
/// round number, so the sweep is not all dyadic fractions where the arithmetic
/// happens to be kind.
fn parameters_across_the_domain() -> Vec<f64> {
    let mut all: Vec<f64> = (0..=32).map(|step| f64::from(step) / 32.0).collect();
    all.extend([1.0 / 3.0, 0.1, 0.7]);
    all
}

/// The same curve with every control point multiplied by `factor`.
///
/// A power of two, so the net is scaled with nothing rounded and the result is
/// exactly the circle of that radius.
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

/// Four corners in the plane `z = 0`, with weights that make the point at a
/// general parameter awkward in `x` and `y` and exactly zero in `z`.
fn a_flat_patch_with_lopsided_weights() -> BSplineSurface {
    let straight = KnotVector::new(1, vec![0.0, 0.0, 1.0, 1.0]).expect("one straight span");
    let corners = [
        Point::new(0.0, 0.0, 0.0),
        Point::new(3.0, 0.0, 0.0),
        Point::new(0.0, 5.0, 0.0),
        Point::new(7.0, 11.0, 0.0),
    ];
    BSplineSurface::new(straight.clone(), straight, &corners, &[1.0, 3.0, 7.0, 11.0])
        .expect("four points, four positive weights")
}
