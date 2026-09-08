//! [`where_a_closed_crossing_starts`] hands back a place or it hands back a
//! name, and never an `Ok` carrying a point that is not one.
//!
//! `docs/decisions.md` §74. The conic arm of that function used to end in
//! `Ok(conic.at(0.0))` with nothing between it and the caller, and the
//! degenerate-conic check one level down is skipped on purpose for a conic whose
//! size is not finite — so the two together left a door open, and the pipe
//! below walks through it: `where_a_closed_crossing_starts` answered
//! `Ok(Point([NaN, NaN, NaN]))` for the very pair [`walk`] refuses by name.
//!
//! Both halves of that matter. A `NaN` triple is what `rcad-topo`'s
//! `where_the_crossing_starts` asserts it will never be handed, on the stated
//! grounds that "a seed that is not a place is a bug in the walk" — so the
//! answer was not merely useless, it was the thing the layer above had written
//! down as impossible.

#![forbid(unsafe_code)]

use rcad_geom::{
    Axis, Cone, Cylinder, Deflection, EdgeRun, Error, NamedRun, Plane, Point, Sphere, Surface,
    Vector, walk, where_a_closed_crossing_starts,
};
use rcad_predicates::{Policy, ProvenanceLog};

/// A pipe fat enough that the ellipse a nearly-parallel plane cuts out of it
/// runs off the top of `f64` is refused by name, and the same pair written
/// smaller starts somewhere.
///
/// **A gap and not a limit** (§16.10): the crossing exists and the answer is
/// representable — the same two surfaces two hundred orders of magnitude down
/// start at a whole number — so what ran out is `f64`'s exponent, and
/// [`Error::CannotWorkOutHowBigACrossingIs`] is the name this repo already gives
/// that. It is the name [`walk`] gives the identical pair, which is the second
/// half of the check: the two public doors onto one crossing must not disagree
/// about whether it can be answered.
///
/// The ellipse is stretched down the plane's slope by `‖n‖ / |n·w|`, so a radius
/// of `1e300` against a plane leaning `1e-20` off the axis puts its long
/// semi-axis at infinity. `Conic::at` then evaluates `centre + across·cos 0 +
/// and·sin 0`, and `∞ · 0` is `NaN` in every coordinate — which is why the
/// broken answer was `NaN` rather than an honest infinity.
#[test]
fn a_conic_too_big_to_hold_a_point_is_refused_and_the_same_pair_smaller_starts_somewhere() {
    let shallow = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, 1e-20))
            .expect("a direction with length"),
    );
    let started_at = |radius: f64| {
        let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), radius).expect("a radius"));
        where_a_closed_crossing_starts(&right_round(), [&pipe, &shallow], &mut ProvenanceLog::new())
    };

    assert_eq!(
        started_at(1e300),
        Err(Error::CannotWorkOutHowBigACrossingIs),
        "a start that is not a place is not a start"
    );

    let smaller = started_at(1e200).expect("the same pair written smaller is answered");
    assert!(
        smaller.is_finite(),
        "the smaller pair started at {smaller:?}"
    );
    assert!(
        smaller.coordinates().iter().any(|it| *it != 0.0),
        "the smaller pair started at the origin, which is not on this pipe"
    );

    // The deflection scales with the shape, and the shape here is the ellipse
    // and not the pipe: the plane stretches it by `1 / |n·w|`, so what a walk of
    // it has to be allowed to stray is that much bigger than the radius too.
    let walked = |radius: f64, deflection: f64| {
        let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), radius).expect("a radius"));
        walk(
            &right_round(),
            [&pipe, &shallow],
            Deflection::of(deflection).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|along| along.points().len())
    };
    assert_eq!(
        walked(1e300, 1e298),
        Err(Error::CannotWorkOutHowBigACrossingIs),
        "the other door onto this crossing gives it a different name"
    );
    assert!(
        walked(1e200, 1e218).is_ok(),
        "the other door refuses the pair this one answers: {:?}",
        walked(1e200, 1e218)
    );
}

/// Every pair whose crossing is a conic, written at every power of two from
/// `2⁻⁵⁰⁰` to `2⁵⁰⁰`, either starts at a place or says why not.
///
/// **This is the boundary that was actually explored, and it is a boundary and
/// not a proof.** What it sweeps is the *scale* of six configurations through
/// all four conic builders — a plane against a ball, a plane leaning across a
/// pipe, a plane nearly along one, two balls, and a plane cutting a cone in a
/// circle and in a leaning ellipse. Of the 1,212 asks — 101 powers, six pairs,
/// each pair both ways round — 1,096 start at a place and 116 refuse, so the run
/// cannot pass by refusing everything; both counts are held to a floor so that a
/// change which quietly turns one into the other fails here.
///
/// What it does **not** sweep: placements far from the origin whose differences
/// are small, degree-four pairs (they leave through the marcher's own seed and
/// not through `Conic::at`), and the two circle-picking builders — a plane
/// against a torus and a coaxial cone against a pipe — which refuse a run with
/// no picker on it as a tie before any of this is reached. Those last two are
/// guarded at their own builder already, in `which_of_these_circles_this_run_is`.
#[test]
fn every_conic_crossing_this_answers_starts_at_a_place() {
    let mut answered = 0;
    let mut refused = 0;
    for power in (-500..=500).step_by(10) {
        let scale = f64::powi(2.0, power);
        for [one, other] in pairs_that_cross_in_a_conic(scale) {
            for along in [[&one, &other], [&other, &one]] {
                match where_a_closed_crossing_starts(
                    &right_round(),
                    along,
                    &mut ProvenanceLog::new(),
                ) {
                    Ok(start) => {
                        assert!(
                            start.is_finite(),
                            "{one:?} and {other:?} at 2^{power} start at {start:?}"
                        );
                        answered += 1;
                    }
                    Err(_) => refused += 1,
                }
            }
        }
    }
    println!("{answered} of these started at a place and {refused} refused");
    assert!(
        answered > 400,
        "only {answered} of these were answered, so this swept nothing"
    );
    assert!(
        refused > 0,
        "nothing in this range refused, so the refusing half is untested"
    );
}

/// An axis direction is a unit vector however short the one handed in was, and
/// that is what the alternative fix rested on.
///
/// **This is here because the argument in §74.3 was wrong the first time.** Two
/// of the six conic builders scale a semi-axis by `up = along/‖along‖²` rather
/// than by a unit vector, and a direction written `(1e-200, 1e-200, 1e-200)`
/// squares to `3e-400`, which is zero in `f64` — so `up` would be infinite in
/// every coordinate while the radii that go into the conic's size stayed exactly
/// what the caller wrote. A conic with an ordinary size and an infinite
/// semi-axis is precisely the case a *size* check cannot see, and it looked like
/// the reason to check the point instead.
///
/// It is not, because `Axis::new` scales every direction to length one, and so
/// does the `TryFrom` a deserialized axis goes through. That fact lives two
/// files away from the builder that depends on it, which is the real argument
/// for a guard that needs no enumeration — and it is pinned here so that an axis
/// which stops being unit fails a test in the file that reasons about it.
#[test]
fn every_axis_a_caller_can_build_has_a_direction_of_length_one() {
    for direction in [
        Vector::new(1e-200, 1e-200, 1e-200),
        Vector::new(1e200, 0.0, 0.0),
        Vector::new(3.0, 4.0, 0.0),
    ] {
        let axis = Axis::new(Point::origin(), direction).expect("a direction with length");
        assert!(
            (axis.direction().length() - 1.0).abs() < 4.0 * f64::EPSILON,
            "an axis built from {direction:?} points along {:?}",
            axis.direction()
        );
    }
}

/// One configuration of each pair a conic builder answers, written at `scale`.
fn pairs_that_cross_in_a_conic(scale: f64) -> Vec<[Surface; 2]> {
    let plane = |through: [f64; 3], normal: [f64; 3]| {
        Surface::Plane(
            Plane::from_point_and_normal(Point::from(through.map(|it| it * scale)), normal.into())
                .expect("a direction with length"),
        )
    };
    let ball = |centre: [f64; 3], radius: f64| {
        Surface::Sphere(
            Sphere::new(Point::from(centre.map(|it| it * scale)), radius * scale).expect("a radius"),
        )
    };
    let pipe = |radius: f64| {
        Surface::Cylinder(Cylinder::new(Axis::z(), radius * scale).expect("a radius"))
    };
    let cone = Surface::Cone(Cone::new(Axis::z(), 1.0).expect("a slope"));

    vec![
        [plane([0.0, 0.0, 1.0], [1.0, 2.0, 3.0]), ball([0.5, 0.0, 0.0], 6.0)],
        [ball([0.5, 0.0, 0.0], 6.0), ball([3.0, 1.0, 0.5], 7.0)],
        [plane([0.0, 0.0, 1.0], [1.0, 2.0, 3.0]), pipe(2.0)],
        // The pair that overflows: a plane leaning a hair off the pipe's own
        // axis stretches the ellipse by `1 / |n·w|`.
        [plane([0.0, 0.0, 0.0], [1.0, 0.0, 1e-20]), pipe(2.0)],
        [plane([0.0, 0.0, 5.0], [0.0, 0.0, 1.0]), cone.clone()],
        [plane([0.0, 0.0, 5.0], [0.2, 0.0, 1.0]), cone],
    ]
}

/// A run stated as going right round the whole of a crossing, which is the one
/// shape that needs a vertex put on it out of nowhere.
fn right_round() -> NamedRun {
    NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
        .expect("a closed curve is never a piece with nothing picking it")
}
