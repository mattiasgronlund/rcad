//! A ball of radius 1 at the origin and the block `[0.9, 10.9] × [−5, 5] ×
//! [−9.5, 0.5]`, which pokes into the ball's side past `x = 0.9`. The block's
//! top plane `z = 0.5` misses the cap the block takes, but the fuse cuts the
//! ball along it outside the block too, into the cap above `z = 0.5` and the
//! rest below it with the `x = 0.9` circle as an island.
//!
//! `docs/decisions.md` §400.5 measured it reaching the mesher's refusal of a
//! wire right round beside one that is not, `AWireWillNotLieFlatOnItsFace`
//! (§212.6): the union both ways round and the ball less the block, the three
//! fuses that keep that face. The rim is walked at four corners at the fuse's
//! deflection, one of them under the island, so the cut up to the pole from it
//! is taken through the island.
//!
//! **The two pieces are one face again since the joiner's ball arm**
//! (§229.2): the `z = 0.5` circle has the ball on both sides and is no edge,
//! so the union both ways round and the ball less the block keep the ball's
//! skin as one face of one wire, the `x = 0.9` circle: the ball less the cap
//! the block takes.
//!
//! The answers are the closed forms: the block shares with the ball the cap
//! past `x = 0.9`, and nothing past `z = 0.5`, since `0.9² + 0.5² > 1`.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

const BALL: f64 = 4.0 * PI / 3.0;
const BLOCK: f64 = 10.0 * 10.0 * 10.0;

#[test]
fn a_block_poking_into_a_ball_measures_under_every_rule() {
    let shared = cap(0.9);
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for ball_first in [false, true] {
            let truth = match (rule, ball_first) {
                (Rule::Union, _) => BALL + BLOCK - shared,
                (Rule::Difference, true) => BALL - shared,
                (Rule::Difference, false) => BLOCK - shared,
                (Rule::Intersection, _) => shared,
                (other, _) => panic!("no closed form written for {other:?}"),
            };
            let what = format!("{rule:?}, ball first {ball_first}");
            let (mut model, ball, block) = a_ball_and_a_block();
            let (arguments, tools) = if ball_first {
                (ball, block)
            } else {
                (block, ball)
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
            if matches!(
                (rule, ball_first),
                (Rule::Union, _) | (Rule::Difference, true)
            ) {
                assert_eq!(
                    skins_by_wires(&model, &answer),
                    [1],
                    "{what}: the ball's skin is one face of one wire"
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

fn a_ball_and_a_block() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(1.0).expect("1 mm is a working size"));
    let ball = model
        .add_ball(1.0, &Isometry::identity())
        .expect("a ball of radius 1");
    let rectangle = ProfileBuilder::new(vec![[0.9, -5.0], [10.9, -5.0], [10.9, 5.0], [0.9, 5.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let block = model
        .add_extrusion(
            &rectangle,
            10.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -9.5)).expect("a finite shift"),
        )
        .expect("a block");
    (model, ball, block)
}

/// The cap of the unit ball beyond a plane `h` from its centre.
fn cap(h: f64) -> f64 {
    PI * (1.0 - h).powi(2) * (2.0 + h) / 3.0
}
