//! A ball touching the upright edge between two tilted walls of a prism.
//!
//! The prism is the triangle `(0, 0)`, `(4, 3)`, `(−3, 3)` stood 32 tall, so
//! one of its upright edges is the `z` axis, with wall 1 (`0.6x − 0.8y = 0`)
//! leaning away from it towards `(4, 3)` and wall 2 (`x + y = 0`) towards
//! `(−3, 3)`. Two balls, centred at height 16:
//!
//! - **radius 1 at `(1, 0)`**, touching the edge exactly at `v = (0, 0, 16)`,
//!   which is a shape a designer makes on purpose;
//! - **radius `1 + 2⁻⁵¹` at `(1, 2⁻²⁵)`**, queue row Q74's ball, which crosses
//!   the edge by far less than a rounding.
//!
//! Wall 1 cuts a cap `0.4` deep off either ball, and its circle lies in the
//! wall and touches the edge at `v`. Wall 2's plane cuts the ball in a circle
//! that lies outside the wall and touches it only at `v`. Both places the edge
//! meets the ball are the one vertex `v`, both circles start and end there, so
//! the ball's face between them is bounded by a figure eight through `v`, and
//! in the union wall 1's wire runs down the edge, round its circle and on down
//! the edge. Each named `v` twice, and union and difference were refused
//! `CannotMeshAFace { CornersCoincide }` until the mesher cut such a wire into
//! the loops that touch there (`docs/decisions.md` §364).
//!
//! The fuse was right all along: the intersection measured within its error.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Fused, Model, ProfileBuilder, Rule, SolidKey};

/// A ball: its radius and where its centre is across the floor.
type Ball = (f64, [f64; 2]);

/// The ball that touches the edge exactly.
const TOUCHING: Ball = (1.0, [1.0, 0.0]);

/// Queue row Q74's ball, a rounding past the edge.
fn q74s_ball() -> Ball {
    (1.0 + f64::powi(2.0, -51), [1.0, f64::powi(2.0, -25)])
}

/// The prism's volume: a triangle of area 10.5, 32 tall.
const THE_PRISM: f64 = 336.0;

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// The regression: under a tolerant policy both balls fuse with the prism by
/// every rule, either way round, into one solid whose measured volume is within
/// its own published error of the closed form.
#[test]
fn under_a_tolerant_policy_every_fuse_of_either_ball_measures_what_it_should() {
    for ball in [TOUCHING, q74s_ball()] {
        for rule in EVERY_RULE {
            for ball_first in [true, false] {
                assert_measures_what_it_should(ball, rule, ball_first, &Policy::tolerant(25.0));
            }
        }
    }
}

/// Under a certified policy the ball that touches exactly is answered the same
/// way: its touching point is a double and the fuse proves every turn there.
#[test]
fn under_a_certified_policy_every_fuse_of_the_touching_ball_measures_what_it_should() {
    for rule in EVERY_RULE {
        for ball_first in [true, false] {
            assert_measures_what_it_should(TOUCHING, rule, ball_first, &Policy::certified());
        }
    }
}

/// Pinned as it answers: under a certified policy Q74's ball is refused by
/// every rule, either way round, on the order of two points along a crossing
/// that are less than a rounding apart. That is the fuse, not the mesher.
#[test]
fn under_a_certified_policy_q74s_ball_is_refused_on_the_order_along_a_crossing() {
    for rule in EVERY_RULE {
        for ball_first in [true, false] {
            let (_, fused) = fuse(q74s_ball(), rule, ball_first, &Policy::certified());
            let Err(Error::Undecidable { records }) = fused else {
                panic!("{rule:?}, ball first {ball_first}: expected Undecidable, got {fused:?}");
            };
            assert_eq!(
                records.records().last().map(|record| record.predicate()),
                Some("order along a crossing"),
                "{rule:?}, ball first {ball_first}"
            );
        }
    }
}

fn assert_measures_what_it_should(ball: Ball, rule: Rule, ball_first: bool, policy: &Policy) {
    let (model, fused) = fuse(ball, rule, ball_first, policy);
    let fused =
        fused.unwrap_or_else(|why| panic!("{ball:?}, {rule:?}, ball first {ball_first}: {why:?}"));
    let [solid] = fused.solids()[..] else {
        panic!(
            "{ball:?}, {rule:?}, ball first {ball_first}: one solid, not {:?}",
            fused.solids()
        );
    };
    let measured = model
        .solid(solid)
        .expect("the fuse made it")
        .measure(
            Deflection::of(0.001).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{ball:?}, {rule:?}, ball first {ball_first}: {why:?}"));
    let should = what_it_should_measure(ball, rule, ball_first);
    assert!(
        (measured.volume() - should).abs() <= measured.volume_error(),
        "{ball:?}, {rule:?}, ball first {ball_first}: {} ± {} is not {should}",
        measured.volume(),
        measured.volume_error()
    );
}

/// The closed form: wall 1 cuts a cap off the ball, and nothing else of the
/// prism reaches it. The cap is `r − d` deep, `d` the centre's distance from
/// the wall.
fn what_it_should_measure((radius, [x, y]): Ball, rule: Rule, ball_first: bool) -> f64 {
    let depth = radius - (0.6 * x - 0.8 * y);
    let cap = core::f64::consts::PI * depth * depth * (3.0 * radius - depth) / 3.0;
    let ball = 4.0 / 3.0 * core::f64::consts::PI * radius * radius * radius;
    match (rule, ball_first) {
        (Rule::Union, _) => THE_PRISM + ball - cap,
        (Rule::Intersection, _) => cap,
        (Rule::Difference, true) => ball - cap,
        (Rule::Difference, false) => THE_PRISM - cap,
        (other, _) => panic!("no closed form written for {other:?}"),
    }
}

/// The model the fuse ran in, and what the fuse said.
fn fuse(
    (radius, [x, y]): Ball,
    rule: Rule,
    ball_first: bool,
    policy: &Policy,
) -> (Model, rcad_topo::Result<Fused>) {
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let prism = the_prism(&mut model);
    let ball = model
        .add_ball(
            radius,
            &Isometry::translation(Vector::new(x, y, 16.0)).expect("a finite shift"),
        )
        .expect("a positive radius");
    let (one, other) = if ball_first {
        (ball, prism)
    } else {
        (prism, ball)
    };
    let fused = model.fuse(&[one], &[other], rule, policy, &mut ProvenanceLog::new());
    (model, fused)
}

fn the_prism(model: &mut Model) -> SolidKey {
    let triangle = ProfileBuilder::new(vec![[0.0, 0.0], [4.0, 3.0], [-3.0, 3.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a triangle");
    model
        .add_extrusion(&triangle, 32.0, &Isometry::identity())
        .expect("a positive height")
}
