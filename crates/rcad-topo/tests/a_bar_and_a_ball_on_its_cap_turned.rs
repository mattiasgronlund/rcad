//! A round bar with a ball on the middle of its top cap, turned.
//!
//! The bar is `add_round_bar` of radius 5 and height 10, the ball
//! `add_ball` of radius 6 moved to `(0, 0, 10)`. Unplaced, the ball's centre
//! is on the bar's axis and the two cross in a circle at `z = 10 − √11`.
//!
//! Both are placed by the turn of `1.1` about the line through `(1, 0, −1)`
//! along `(1, 2, 3)`, the ball by that turn or by two halves of it. In the
//! stored numbers the ball's centre then sits about `1.1e−15` off the pipe's
//! axis, so the two cross in two loops a rounding off circles, and every rule
//! was refused `ResultIsNotManifold { count: 1 }` (row Q2129).

use rcad_geom::{Axis, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule};

#[test]
fn a_bar_and_a_ball_on_its_cap_unturned_answer_every_rule() {
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        assert_eq!(fused(&[], &[], rule), 1, "the {rule:?} is one solid");
    }
}

#[test]
fn a_bar_and_a_ball_on_its_cap_turned_answer_every_rule() {
    let axis =
        Axis::new(Point::new(1.0, 0.0, -1.0), Vector::new(1.0, 2.0, 3.0)).expect("a direction");
    let turn = Isometry::rotation(&axis, 1.1).expect("a finite angle");
    let half = Isometry::rotation(&axis, 0.55).expect("a finite angle");
    for ball_turns in [vec![turn], vec![half, half]] {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            assert_eq!(
                fused(&[turn], &ball_turns, rule),
                1,
                "the {rule:?} is one solid"
            );
        }
    }
}

/// How many solids the bar and the ball fused under `rule` come to, under
/// `Policy::certified()`, once the answer validates.
fn fused(bar_turns: &[Isometry], ball_turns: &[Isometry], rule: Rule) -> usize {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let bar = model
        .add_round_bar(5.0, 10.0, &Isometry::identity())
        .expect("a bar");
    let ball = model
        .add_ball(
            6.0,
            &Isometry::translation(Vector::new(0.0, 0.0, 10.0)).expect("a move"),
        )
        .expect("a ball");
    for placed in bar_turns {
        model
            .place_solid(bar, placed)
            .expect("this model made the key");
    }
    for placed in ball_turns {
        model
            .place_solid(ball, placed)
            .expect("this model made the key");
    }
    let solids = model
        .fuse(
            &[bar],
            &[ball],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("the {rule:?} was refused: {why:?}"))
        .solids()
        .len();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("the {rule:?} does not validate: {why}"));
    solids
}
