//! A ball's skin with several holes in it and the material outside every one,
//! none of them going right round the ball.
//!
//! A bar pushed right through a ball and united with it leaves the ball's skin
//! with a hole where the bar goes in and another where it comes out, and so do
//! two blocks poking into a ball at two places. The mesher handed such a face
//! over whole, with the first hole taken for the outer loop, and the
//! triangulator filled the holes and not the skin round them: Q443's turned
//! cube was refused `TwoPatchesOfTheMeshDrewTheSameSide` under intersection
//! and `Invalid(FacesDoNotAgreeWhichWayTheyFace)` as the cube less the ball, in
//! both orders.
//!
//! The ball is now opened along a meridian in every stretch of longitude no
//! hole reaches, into one strip from pole to pole per run of holes, and each
//! strip is cut from its holes up to both poles (`docs/decisions.md` §402).
//! Holes in one strip are cut one above the other where each lies wholly above
//! the next, and side by side where a straight cut up and down from each meets
//! nothing but the poles. A hole in the crook of another is neither, and is
//! refused by name.
//!
//! Every answer here is held to its closed form, under every rule and with the
//! arguments both ways round: the ball and the other solids, less twice what
//! they share for a union, and so on. What a block shares with the ball is a
//! cap, or the piece beyond two planes at once (`beyond_two`).

use core::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

const BALL: f64 = 4.0 * PI / 3.0;

/// Q443's second draw: a cube of side 2 turned one radian about `(2, −1, 1)`,
/// and a ball of radius 1 at `(0.8, 0.7, 0.65)` in the cube's frame. The ball
/// crosses the cube's planes `x = 0`, `y = 0` and `z = 0` and its edge where
/// `y = z = 0`, and not the other two edges, so its skin inside the cube has
/// a round hole where `x = 0` cuts it and a hole of two arcs beside it. At a
/// deflection of `0.01` the two overlap in longitude by a few hundredths of a
/// radian, and are cut side by side.
#[test]
fn a_turned_cube_whose_faces_leave_two_holes_in_a_ball_measures_under_every_rule() {
    let centre = [0.8, 0.7, 0.65];
    let shared = BALL - cap(0.8) - cap(0.7) - cap(0.65) + beyond_two(0.7, 0.65);
    every_rule_both_ways(
        || {
            let (model, cube, ball) = a_turned_cube_and_a_ball(centre);
            (model, vec![ball], vec![cube])
        },
        8.0,
        shared,
        Rule::Intersection,
        "a turned cube",
    );
}

/// A slab `1.2` thick across a ball of radius 1, square to `x`: its two faces
/// cut a round hole each, half a turn apart round the ball's polar axis, and
/// no other face of it reaches the ball. Each hole gets a strip of its own.
#[test]
fn a_slab_across_a_ball_measures_under_every_rule() {
    every_rule_both_ways(
        || {
            let mut model = a_model();
            let ball = a_ball_at_the_origin(&mut model);
            let slab = block(&mut model, [-0.6, -2.0, -2.0], [0.6, 2.0, 2.0]);
            (model, vec![ball], vec![slab])
        },
        1.2 * 4.0 * 4.0,
        BALL - 2.0 * cap(0.6),
        Rule::Intersection,
        "a slab",
    );
}

/// Two balls of radius `0.5`, `1.2` from the middle of a ball of radius 1, one
/// a radian above its equator and one a radian below, at one longitude: two
/// round holes, one wholly above the other, cut as one chain from the top pole
/// to the bottom one.
#[test]
fn two_balls_one_above_the_other_in_a_ball_measure_under_every_rule() {
    every_rule_both_ways(
        || two_small_balls_in_a_ball([[0.0, 1.0], [0.0, -1.0]]),
        2.0 * SMALL_BALL,
        2.0 * lens(1.0, 0.5, 1.2),
        Rule::Union,
        "two balls one above the other",
    );
}

/// The same two balls side by side: one at longitude `0` and latitude `0.6`,
/// the other at `0.9` and `−0.2`. Their holes overlap in longitude by about
/// five hundredths of a radian and in latitude by about four, so no meridian
/// runs between them and neither lies above the other, and each is cut
/// straight up and down to the poles past the other.
#[test]
fn two_balls_side_by_side_in_a_ball_measure_under_every_rule() {
    every_rule_both_ways(
        || two_small_balls_in_a_ball([[0.0, 0.6], [0.9, -0.2]]),
        2.0 * SMALL_BALL,
        2.0 * lens(1.0, 0.5, 1.2),
        Rule::Union,
        "two balls side by side",
    );
}

/// The limit of the cut: the same two balls, one at longitude `0` and latitude
/// `−0.4`, the other up and over at `0.42` and `0.4`. Their holes reach the
/// same latitudes by a few hundredths, so neither lies above the other, and
/// each one's cut straight up or down runs into the other, so they are not
/// side by side either. The union is refused by name, where the ball's skin
/// is meshed.
#[test]
fn two_holes_each_in_the_way_of_the_others_cut_are_refused_by_name() {
    let (mut model, ball, small) = two_small_balls_in_a_ball([[0.0, -0.4], [0.42, 0.4]]);
    let refused = model
        .fuse(
            &ball,
            &small,
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .and_then(|fused| {
            let answer = fused.solids().to_vec();
            let [solid] = answer[..] else {
                panic!("one solid, got {answer:?}");
            };
            assert!(
                skins_by_wires(&model, &answer).contains(&2),
                "a face on the ball with two holes, got {:?}",
                skins_by_wires(&model, &answer)
            );
            model
                .solid(solid)
                .expect("the fuse's own key")
                .measure(
                    Deflection::of(0.01).expect("a positive length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .map(|_| ())
        });
    assert!(
        matches!(
            refused,
            Err(Error::CurvedFaceWithAHoleGoesTooFarRound { .. })
        ),
        "{refused:?}"
    );
}

/// The ball and the other solids under union, intersection and difference,
/// each with the ball first and with it last, held to their closed forms. The
/// answer under `two_holes_under` is checked to reach the shape these tests
/// are about: one face on the ball with two holes in it.
fn every_rule_both_ways(
    build: impl Fn() -> (Model, Vec<SolidKey>, Vec<SolidKey>),
    others: f64,
    shared: f64,
    two_holes_under: Rule,
    what: &str,
) {
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for ball_first in [false, true] {
            let truth = match (rule, ball_first) {
                (Rule::Union, _) => BALL + others - shared,
                (Rule::Difference, true) => BALL - shared,
                (Rule::Difference, false) => others - shared,
                (Rule::Intersection, _) => shared,
                (other, _) => panic!("no closed form written for {other:?}"),
            };
            let what = format!("{what}: {rule:?}, ball first {ball_first}");
            let (mut model, ball, rest) = build();
            let (arguments, tools) = if ball_first {
                (ball, rest)
            } else {
                (rest, ball)
            };
            let answer = model
                .fuse(
                    &arguments,
                    &tools,
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
            if rule == two_holes_under {
                assert!(
                    skins_by_wires(&model, &answer).contains(&2),
                    "{what}: a face on the ball with two holes, got {:?}",
                    skins_by_wires(&model, &answer)
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

fn a_turned_cube_and_a_ball(centre: [f64; 3]) -> (Model, SolidKey, SolidKey) {
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
    let at = turn.apply_to_point(Point::new(centre[0], centre[1], centre[2]));
    let ball = model
        .add_ball(
            1.0,
            &Isometry::translation(at - Point::origin()).expect("a finite placement"),
        )
        .expect("a ball");
    (model, cube, ball)
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(1.0).expect("1 mm is a working size"))
}

fn a_ball_at_the_origin(model: &mut Model) -> SolidKey {
    model
        .add_ball(1.0, &Isometry::identity())
        .expect("a ball of radius 1")
}

/// A ball of radius 1 at the origin and two of radius `0.5`, each `1.2` out
/// from its middle at the longitude and latitude given.
fn two_small_balls_in_a_ball(at: [[f64; 2]; 2]) -> (Model, Vec<SolidKey>, Vec<SolidKey>) {
    let mut model = a_model();
    let ball = a_ball_at_the_origin(&mut model);
    let small = at.map(|at| a_ball_out_along(&mut model, at, 1.2, 0.5));
    (model, vec![ball], small.to_vec())
}

/// A ball of `radius`, `out` from the origin at `[longitude, latitude]` round
/// the `z` axis, where the ball at the origin has its poles.
fn a_ball_out_along(
    model: &mut Model,
    [longitude, latitude]: [f64; 2],
    out: f64,
    radius: f64,
) -> SolidKey {
    let way = Vector::new(
        latitude.cos() * longitude.cos(),
        latitude.cos() * longitude.sin(),
        latitude.sin(),
    );
    model
        .add_ball(
            radius,
            &Isometry::translation(way * out).expect("a finite placement"),
        )
        .expect("a small ball")
}

const SMALL_BALL: f64 = 4.0 * PI * 0.125 / 3.0;

/// What two balls of radius `one` and `other` share, their middles `apart`.
fn lens(one: f64, other: f64, apart: f64) -> f64 {
    PI * (one + other - apart).powi(2)
        * (apart * apart + 2.0 * apart * (one + other) - 3.0 * (one - other).powi(2))
        / (12.0 * apart)
}

/// A block from `low` to `high`, a rectangle extruded up `z`.
fn block(model: &mut Model, low: [f64; 3], high: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [low[0], low[1]],
        [high[0], low[1]],
        [high[0], high[1]],
        [low[0], high[1]],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(
            &rectangle,
            high[2] - low[2],
            &Isometry::translation(Vector::new(0.0, 0.0, low[2])).expect("a finite shift"),
        )
        .expect("a block")
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
