//! A frame with a block flush beneath one corner of its front wall, turned,
//! fuses under every rule (queue row Q1426).
//!
//! The frame is five by three and one high, with the three-by-one hole a wall
//! of one leaves, at the origin. The block is two by one and one high at
//! `(3, 0, -1)`, so its top is the frame's floor from `x = 3` to `x = 5` under
//! the front wall, and its walls at `x = 5` and `y = 0` are the frame's walls
//! carried on down. Both are turned together by 27/64 of a turn about
//! `(-2, -3, -2)`. This is what `how_often_a_certified_fuse_of_a_turned_pair_refuses`
//! in `fuses_that_must_agree.rs` shrank to at `RCAD_SOAK_FACTOR=66
//! RCAD_SOAK_SEED=2`, and it panicked in `fuse/assemble.rs` on a face of two
//! straight sides.
//!
//! The frame takes up `15 - 3 = 12` and the block 2, and the two share only
//! the face between them, so their union is one solid of 14, neither takes
//! anything from the other, and what they share is nothing.

// `pairs` describes its frustum through `cones::a_frustum`.
#[path = "fuzzing/comparison.rs"]
mod comparison;
mod cones;
mod pairs;

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

use comparison::{answer_of, same_answer};
use pairs::{Configuration, Elsewhere, Placed, Shape, Skew, Turn};

const FRAME: usize = 0;
const BLOCK: usize = 1;

#[test]
fn a_turned_frame_and_the_block_flush_beneath_it_unite_to_one_solid_of_fourteen() {
    both_ways_round_give(Rule::Union, 1, 14.0);
}

#[test]
fn a_turned_frame_and_the_block_flush_beneath_it_share_nothing() {
    both_ways_round_give(Rule::Intersection, 0, 0.0);
}

#[test]
fn a_turned_frame_less_the_block_flush_beneath_it_is_the_whole_frame() {
    let (model, solids) = fused(FRAME, BLOCK, Rule::Difference);
    assert_eq!(solids.len(), 1, "the frame less the block is one solid");
    let measured = volume_of(&model, &solids);
    assert!(
        (measured - 12.0).abs() < 1e-9,
        "the frame less the block takes up {measured}, not 12"
    );
}

#[test]
fn a_turned_block_less_the_frame_flush_above_it_is_the_whole_block() {
    let (model, solids) = fused(BLOCK, FRAME, Rule::Difference);
    assert_eq!(solids.len(), 1, "the block less the frame is one solid");
    let measured = volume_of(&model, &solids);
    assert!(
        (measured - 2.0).abs() < 1e-9,
        "the block less the frame takes up {measured}, not 2"
    );
}

/// Both orders give `count` solids taking up `volume` between them, and the two
/// answers are the same bit for bit.
fn both_ways_round_give(rule: Rule, count: usize, volume: f64) {
    let forwards = fused(FRAME, BLOCK, rule);
    let backwards = fused(BLOCK, FRAME, rule);
    for ((model, solids), order) in [(&forwards, "frame first"), (&backwards, "block first")] {
        assert_eq!(
            solids.len(),
            count,
            "the {rule:?} with the {order} is {count} solids"
        );
        let measured = volume_of(model, solids);
        assert!(
            (measured - volume).abs() < 1e-9,
            "the {rule:?} with the {order} takes up {measured}, not {volume}"
        );
    }
    if let Err(why) = same_answer(
        &answer_of(&forwards.0, &forwards.1),
        &answer_of(&backwards.0, &backwards.1),
        rule,
        "swapping the frame and the block",
    ) {
        panic!("{why}");
    }
}

fn the_frame_and_the_block() -> Configuration {
    Configuration {
        solids: vec![
            Placed {
                shape: Shape::Frame { width: 5, depth: 3 },
                skew: Skew::none(),
                height: 1,
                at: [0, 0, 0],
            },
            Placed {
                shape: Shape::Block { width: 2, depth: 1 },
                skew: Skew::none(),
                height: 1,
                at: [3, 0, -1],
            },
        ],
        turn: Some(Turn {
            direction: [-2, -3, -2],
            steps: 27,
        }),
        elsewhere: Elsewhere::here(),
    }
}

/// Fuse solid `arguments` against solid `tools` under a certified policy, and
/// check that what came back validates.
fn fused(arguments: usize, tools: usize, rule: Rule) -> (Model, Vec<SolidKey>) {
    let (mut model, keys) = the_frame_and_the_block().build();
    let mut log = ProvenanceLog::new();
    let solids = match model.fuse(
        &[keys[arguments]],
        &[keys[tools]],
        rule,
        &Policy::certified(),
        &mut log,
    ) {
        Ok(fused) => fused.solids().to_vec(),
        Err(why) => panic!("the {rule:?} of solid {arguments} against {tools} refused: {why:?}"),
    };
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a fuse's answer validates");
    (model, solids)
}

fn volume_of(model: &Model, solids: &[SolidKey]) -> f64 {
    solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.1).expect("a tenth of a millimetre is a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a solid this size measures")
                .volume()
        })
        .sum()
}
