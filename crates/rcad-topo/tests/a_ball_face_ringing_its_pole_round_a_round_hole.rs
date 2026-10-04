//! A cube of side 2 and a ball of radius 1 at `(0.8, 0.7, 0.65)` in the cube's
//! frame. The ball crosses the cube's planes `x = 0`, `y = 0` and `z = 0` and
//! the edge where `y = z = 0`, and not the other two edges at that corner. So
//! the `x = 0` circle meets nothing else, and the ball's skin inside the cube
//! is one face with two wires: a rim of arcs on `y = 0` and `z = 0` that goes
//! right round one pole, and the `x = 0` circle as an island beside it.
//!
//! The mesher refused that face `AWireWillNotLieFlatOnItsFace` in `cut_up`,
//! since a cap was cut up to its pole into four patches with nowhere to put an
//! island (`docs/decisions.md` §212.6). The intersection and the cube less the
//! ball, the two fuses that keep that skin, were refused both ways round,
//! upright and at 17 of the 20 turns below. Now an island no cut meets goes
//! whole into its patch as a hole, and a cut whose meridian meets one goes
//! through it, each patch beside it taking half the island. 16 of the 17 turns
//! answer; the last is refused by name.
//!
//! Every answer is held to its closed form: the ball inside the cube is the
//! ball less the three caps the planes cut off, plus the piece beyond both
//! `y = 0` and `z = 0`, which two caps took off.

use core::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

const BALL: f64 = 4.0 * PI / 3.0;
const CENTRE: [f64; 3] = [0.8, 0.7, 0.65];

/// The row's own draw, with the cube at the origin and not turned.
#[test]
fn the_upright_cube_and_ball_measure_under_every_rule() {
    every_rule_both_ways(None);
}

/// The same pair with the cube turned about five axes. At each of these
/// turns the skin inside the cube is a cap with the island beside it, and the
/// fuses that keep it were refused.
#[test]
fn the_turned_cube_and_ball_measure_under_every_rule() {
    for (axis, angles) in [
        ([1.0, 1.0, 1.0], &[0.5, 1.0, 2.0, 3.0][..]),
        ([1.0, 2.0, 3.0], &[0.5, 1.0, 2.0, 3.0][..]),
        ([2.0, -1.0, 1.0], &[0.5][..]),
        ([1.0, 0.0, 0.0], &[0.5, 1.0, 2.0, 3.0][..]),
        ([0.0, 1.0, 1.0], &[0.5, 1.0, 3.0][..]),
    ] {
        for &angle in angles {
            every_rule_both_ways(Some((axis, angle)));
        }
    }
}

/// The limit of the cut: turned 2 about `(2, −1, 1)`, the island reaches more
/// than half a turn round the ball and its rim is walked at four corners, so
/// two of the four cuts up to the pole meet the island and a cut through it
/// leaves only two pieces of it. The intersection is refused by name.
#[test]
fn an_island_two_cuts_meet_is_refused_by_name() {
    let (mut model, cube, ball) = a_cube_and_a_ball(Some(([2.0, -1.0, 1.0], 2.0)));
    let refused = model.fuse(
        &[cube],
        &[ball],
        Rule::Intersection,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(
            refused,
            Err(Error::CurvedFaceWithAHoleGoesTooFarRound { .. })
        ),
        "{refused:?}"
    );
}

/// The cube and the ball under union, intersection and difference, each with
/// the ball first and with it last, held to their closed forms. The
/// intersection is checked to reach the face this file is about: a face on the
/// ball with two wires.
fn every_rule_both_ways(turn: Option<([f64; 3], f64)>) {
    let shared =
        BALL - cap(CENTRE[0]) - cap(CENTRE[1]) - cap(CENTRE[2]) + beyond_two(CENTRE[1], CENTRE[2]);
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for ball_first in [false, true] {
            let truth = match (rule, ball_first) {
                (Rule::Union, _) => BALL + 8.0 - shared,
                (Rule::Difference, true) => BALL - shared,
                (Rule::Difference, false) => 8.0 - shared,
                (Rule::Intersection, _) => shared,
                (other, _) => panic!("no closed form written for {other:?}"),
            };
            let what = format!("turned {turn:?}: {rule:?}, ball first {ball_first}");
            let (mut model, cube, ball) = a_cube_and_a_ball(turn);
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
            if rule == Rule::Intersection {
                assert_eq!(
                    skins_by_wires(&model, &answer),
                    [2],
                    "{what}: one face on the ball, of two wires"
                );
            }
            assert_measures(&model, &answer, truth, &what);
        }
    }
}

/// How many wires each face of the answer on a ball has.
fn skins_by_wires(model: &Model, answer: &[SolidKey]) -> Vec<usize> {
    answer
        .iter()
        .flat_map(|&solid| model.solid(solid).expect("the fuse's own key").faces())
        .filter(|face| matches!(face.surface(), Surface::Sphere(_)))
        .map(|face| face.wires().count())
        .collect()
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

/// The cube turned by `turn` about the origin, if any, and the ball at
/// [`CENTRE`] in the cube's own frame.
fn a_cube_and_a_ball(turn: Option<([f64; 3], f64)>) -> (Model, SolidKey, SolidKey) {
    let turn = turn.map_or_else(Isometry::identity, |([x, y, z], angle)| {
        Isometry::rotation(
            &Axis::new(Point::origin(), Vector::new(x, y, z)).expect("an axis"),
            angle,
        )
        .expect("a finite turn")
    });
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
    (model, cube, ball)
}

/// The cap of the unit ball beyond a plane `h` from its centre.
fn cap(h: f64) -> f64 {
    PI * (1.0 - h).powi(2) * (2.0 + h) / 3.0
}

/// The piece of the unit ball at the origin with `u > a` and `v > b`, for
/// `a, b > 0` with `a² + b² < 1`. `a_ball_face_holding_both_poles_inside_three_arcs.rs`
/// checks the same expression against its slices summed one by one.
fn beyond_two(a: f64, b: f64) -> f64 {
    assert!(a * a + b * b < 1.0, "the two planes meet inside the ball");
    let s = (1.0 - a * a - b * b).sqrt();
    2.0 / 3.0 * (s / (a * b)).atan() + 2.0 / 3.0 * a * b * s
        - (a - a.powi(3) / 3.0) * (b / (1.0 - a * a).sqrt()).acos()
        - (b - b.powi(3) / 3.0) * (a / (1.0 - b * b).sqrt()).acos()
}
