//! Q444's draw: a cube of side 2 turned one radian about `(2, −1, 1)`, and a
//! ball of radius 1 at `(0.62, 0.71, 0.66)` in the cube's frame — the same
//! corner-into-ball pair as
//! `a_ball_face_holding_both_poles_inside_three_arcs.rs`, turned about a
//! different axis. Turning the whole pair together does not change their
//! relative position, so the closed form below is that file's: the ball
//! inside the cube is the ball less the three caps the cube's faces cut off,
//! plus the three pieces beyond two faces at once, which the caps took off
//! twice.
//!
//! Before Q364's fix to the mesher, this turn hit a different crash than
//! that file's. The skin inside the cube came out, as there, as two faces of
//! three arcs each, one round each pole. But `cut_up` read a cap's pole off
//! the plane across its first coedge, and here that plane — one of the
//! cube's three cutting faces — had both poles on the material's side: north
//! and south both came out `Neg`. The small face round the south pole was
//! cut up to the north pole by mistake, and the sweep that checks the
//! patches it left refused `Invalid(FacesDoNotAgreeWhichWayTheyFace)`, in
//! both orders. The mesher now reads a cap's pole off the turn its own rim
//! walk gained, not off a neighbour's plane, and gets this right.

use core::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

const BALL: f64 = 4.0 * PI / 3.0;
const CENTRE: [f64; 3] = [0.62, 0.71, 0.66];

/// The cube and the ball under union, intersection and difference, each with
/// the ball first and with it last, held to their closed forms. Every one of
/// these six fuses refused `Invalid(FacesDoNotAgreeWhichWayTheyFace)` before
/// Q364's fix; all six now answer.
#[test]
fn a_turned_cubes_corner_in_a_ball_measures_its_closed_form_under_every_rule() {
    let shared = what_the_cube_and_the_ball_share(CENTRE);
    assert!(shared > BALL / 2.0, "{shared} is more than half the ball");
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for ball_first in [false, true] {
            let truth = match (rule, ball_first) {
                (Rule::Union, _) => 8.0 + BALL - shared,
                (Rule::Difference, false) => 8.0 - shared,
                (Rule::Difference, true) => BALL - shared,
                (Rule::Intersection, _) => shared,
                (other, _) => panic!("no closed form written for {other:?}"),
            };
            let what = format!("{rule:?}, ball first {ball_first}");
            let (model, answer) = fused(rule, ball_first, &what);
            assert_measures(&model, &answer, truth, &what);
        }
    }
}

/// What makes the test above reach the crash it is about: the intersection's
/// skin is two faces on the ball, each bounded by one wire of three arcs —
/// one round each pole, the two caps the row's name is about.
#[test]
fn the_intersections_skin_is_two_caps_each_bounded_by_three_arcs() {
    let (model, answer) = fused(Rule::Intersection, false, "the intersection");
    let [solid] = answer[..] else {
        panic!("one piece of the ball, got {answer:?}");
    };
    let wires: Vec<Vec<usize>> = model
        .solid(solid)
        .expect("the fuse's own key")
        .faces()
        .filter(|face| matches!(face.surface(), Surface::Sphere(_)))
        .map(|skin| skin.wires().map(|wire| wire.coedges().count()).collect())
        .collect();
    assert_eq!(wires, [[3], [3]], "two faces of one wire of three arcs");
}

fn fused(rule: Rule, ball_first: bool, what: &str) -> (Model, Vec<SolidKey>) {
    let turn = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(2.0, -1.0, 1.0)).expect("an axis"),
        1.0,
    )
    .expect("a finite turn");
    let mut model = Model::new(Scale::millimetres(1.0).expect("1 mm is a working size"));
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    model
        .place_solid(cube, &turn)
        .expect("this model made the key");
    let at = turn.apply_to_point(Point::new(CENTRE[0], CENTRE[1], CENTRE[2]));
    let ball = model
        .add_ball(
            1.0,
            &Isometry::translation(at - Point::origin()).expect("a finite placement"),
        )
        .expect("a ball");
    let (arguments, tools) = if ball_first {
        (ball, cube)
    } else {
        (cube, ball)
    };
    let answer = model
        .fuse(
            &[arguments],
            &[tools],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: refused {why:?}"))
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: {why:?}"));
    (model, answer)
}

/// The answer's volume, summed over its solids, lands within the bound it
/// publishes for itself of `truth`.
fn assert_measures(model: &Model, answer: &[SolidKey], truth: f64, what: &str) {
    let (mut volume, mut error) = (0.0, 0.0);
    for &solid in answer {
        let measured = model
            .solid(solid)
            .expect("the fuse's own key")
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what}: the answer measures: {why:?}"));
        volume += measured.volume();
        error += measured.volume_error();
    }
    assert!(
        (volume - truth).abs() <= error,
        "{what}: {volume} measured, {error} off at most, against {truth}"
    );
}

/// The ball of radius 1 at `centre` inside the cube `[0, 2]³`, for a centre
/// whose ball reaches none of the planes `x = 2`, `y = 2`, `z = 2` and leaves
/// the corner at the origin outside.
fn what_the_cube_and_the_ball_share([x, y, z]: [f64; 3]) -> f64 {
    BALL - cap(x) - cap(y) - cap(z) + beyond_two(x, y) + beyond_two(x, z) + beyond_two(y, z)
}

/// The cap of the unit ball beyond a plane `h` from its centre.
fn cap(h: f64) -> f64 {
    PI * (1.0 - h).powi(2) * (2.0 + h) / 3.0
}

/// The piece of the unit ball at the origin with `u > a` and `v > b`, for
/// `a, b > 0` with `a² + b² < 1`: the slice at `u` is the segment of a disc of
/// radius `√(1 − u²)` beyond `b`, summed from `a` out.
/// `a_ball_face_holding_both_poles_inside_three_arcs.rs` checks the same
/// expression against its slices summed one by one.
fn beyond_two(a: f64, b: f64) -> f64 {
    assert!(a * a + b * b < 1.0, "the two planes meet inside the ball");
    let s = (1.0 - a * a - b * b).sqrt();
    2.0 / 3.0 * (s / (a * b)).atan() + 2.0 / 3.0 * a * b * s
        - (a - a.powi(3) / 3.0) * (b / (1.0 - a * a).sqrt()).acos()
        - (b - b.powi(3) / 3.0) * (a / (1.0 - b * b).sqrt()).acos()
}
