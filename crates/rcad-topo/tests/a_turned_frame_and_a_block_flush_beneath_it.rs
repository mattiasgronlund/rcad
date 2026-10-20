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
//!
//! **The same union with a round bar far off, and a block across its rim**
//! (queue row Q2378). The union joins what is one face a second time once the
//! strip between the two fronts is left out, and that second pass runs over
//! everything the fuse made, so the bar is joined twice too. The bar is one
//! round and two high at `(20, 0, 0)`, and the block across its rim is
//! `x ∈ [20.5, 22]`, `y ∈ [-0.25, 0.375]`, `z ∈ [1.5, 2.5]`.

use std::collections::HashSet;

use core::f64::consts::PI;

// `pairs` describes its frustum through `cones::a_frustum`.
#[path = "fuzzing/comparison.rs"]
mod comparison;
mod cones;
mod pairs;

use rcad_geom::{Deflection, EdgeRun, Isometry, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

use comparison::{Answer, Face, Step, WhatItSitsOn, answer_of, same_answer};
use pairs::{Configuration, Elsewhere, Placed, Shape, Skew, Turn};

const FRAME: usize = 0;
const BLOCK: usize = 1;

#[test]
fn a_turned_frame_and_the_block_flush_beneath_it_unite_to_one_solid_of_fourteen() {
    both_ways_round_give(Rule::Union, 1, 14.0);
}

/// **Taking the block back out of the union gives the frame back.** Over the
/// turned planes the block's front and the frame's front have the same normal
/// bits and offsets `2.7755575615628914e-17` and `0.0`, so the union, with the
/// strip between them left out, had one edge with a front on either side and
/// no third face at its end at the turned `(5, 0, 0)`. No three planes placed
/// that corner, and the second fuse was refused at "where three surfaces meet"
/// with no bound at all (queue row Q1446). The union now makes the two fronts
/// one face (§426), and the answer is checked against the frame as it stood
/// before either fuse.
#[test]
fn the_block_taken_back_out_of_the_union_is_the_frame() {
    let (mut model, keys) = the_frame_and_the_block().build();
    let union = fuse_and_validate(&mut model, &[keys[FRAME]], &[keys[BLOCK]], Rule::Union);
    let less_the_block = fuse_and_validate(&mut model, &union, &[keys[BLOCK]], Rule::Difference);
    assert_eq!(
        less_the_block.len(),
        1,
        "the union less the block is one solid"
    );
    let measured = volume_of(&model, &less_the_block);
    assert!(
        (measured - 12.0).abs() < 1e-9,
        "the union less the block takes up {measured}, not 12"
    );
    if let Err(why) = the_same_flat_solid_to_within(
        &answer_of(&model, &[keys[FRAME]]),
        &answer_of(&model, &less_the_block),
        1e-12,
    ) {
        panic!("the union less the block is not the frame: {why}");
    }
}

/// Each face of `expected` matches exactly one face of `found`, and they have
/// as many faces. Two faces match when their planes have the same normal bits
/// and offsets at most `apart`, they are used the same way round, and each
/// ring of one is the same ring of the other walked the same way from some
/// corner, with each corner at most `apart` from the other's.
///
/// **Not bit for bit**, because a fuse writes each corner as the meet of three
/// of its planes where the frame has its built corners turned: the corners
/// round the frame's hole, which the block never comes near, come back an ulp
/// or two from where the frame has them, and the joined front sits on the
/// block's front, `2.8e-17` from the frame's.
fn the_same_flat_solid_to_within(
    expected: &Answer,
    found: &Answer,
    apart: f64,
) -> Result<(), String> {
    let (expected, found): (Vec<&Face>, Vec<&Face>) = (
        expected.solids.iter().flatten().collect(),
        found.solids.iter().flatten().collect(),
    );
    if expected.len() != found.len() {
        return Err(format!(
            "{} faces where {} were expected",
            found.len(),
            expected.len()
        ));
    }
    for face in &expected {
        let matching = found
            .iter()
            .filter(|other| faces_match(face, other, apart))
            .count();
        if matching != 1 {
            return Err(format!("{matching} faces match the one on {:?}", face.sits));
        }
    }
    Ok(())
}

fn faces_match(face: &Face, other: &Face, apart: f64) -> bool {
    let (WhatItSitsOn::Flat(plane), WhatItSitsOn::Flat(other_plane)) = (&face.sits, &other.sits)
    else {
        panic!("the frame and the block are flat all over");
    };
    face.reversed == other.reversed
        && plane[..3]
            .iter()
            .zip(&other_plane[..3])
            .all(|(one, other)| one.to_bits() == other.to_bits())
        && (plane[3] - other_plane[3]).abs() <= apart
        && face.wires.len() == other.wires.len()
        && face
            .wires
            .iter()
            .zip(&other.wires)
            .all(|(ring, other_ring)| rings_match(ring, other_ring, apart))
}

fn rings_match(ring: &[Step], other: &[Step], apart: f64) -> bool {
    ring.len() == other.len()
        && (0..other.len()).any(|start| {
            ring.iter().enumerate().all(|(position, step)| {
                let there = other[(start + position) % other.len()].at;
                (0..3).all(|axis| (step.at[axis] - there[axis]).abs() <= apart)
            })
        })
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
    let solids = fuse_and_validate(&mut model, &[keys[arguments]], &[keys[tools]], rule);
    (model, solids)
}

fn fuse_and_validate(
    model: &mut Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
) -> Vec<SolidKey> {
    let solids = match model.fuse(
        arguments,
        tools,
        rule,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) {
        Ok(fused) => fused.solids().to_vec(),
        Err(why) => panic!("the {rule:?} of {arguments:?} against {tools:?} refused: {why:?}"),
    };
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a fuse's answer validates");
    solids
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

/// **The bar keeps every arc of its rims shorter than half its circle.** The
/// block's planes `x = 20.5`, `y = -0.25` and `y = 0.375` cut the bar's wall
/// right down its far side too, and where those cuts meet the rims the first
/// pass keeps a corner, since pieces of the wall met there (§224.1). The
/// second pass, with no cut left there to say why, folded them, and the top
/// rim came back as one arc `5.646108277541867` long from one side of the
/// block round to the other, past half its circle.
#[test]
fn a_bar_with_a_block_across_its_rim_beside_them_keeps_every_arc_under_half_its_circle() {
    for (asked, model, bar) in the_bar_united_beside_the_frame_and_the_block() {
        let mut seen = HashSet::new();
        let past_half: Vec<f64> = model
            .solid(bar)
            .expect("this model made the key")
            .faces()
            .flat_map(|face| face.wires().flat_map(|wire| wire.coedges()))
            .map(|side| side.edge())
            .filter(|edge| seen.insert(edge.key()))
            .filter(|edge| matches!(edge.run(), EdgeRun::PartOfWhereItCrosses { .. }))
            .map(|edge| edge.length().expect("an arc of a circle measures"))
            .filter(|length| length.length() + length.error() >= PI)
            .map(|length| length.length())
            .collect();
        assert!(
            past_half.is_empty(),
            "{asked}: the bar has arcs past half its circle, {past_half:?} long"
        );
    }
}

/// **And it comes back as the bar and the block across its rim united on
/// their own**, bit for bit, whichever group each is handed over in. The
/// second pass folded the corners on the bottom rim as well, into one closed
/// circle, and the bar answered 20 edges and 13 corners where on its own it
/// has 29 and 22.
#[test]
fn a_bar_with_a_block_across_its_rim_beside_them_unites_as_it_does_on_its_own() {
    let on_its_own = {
        let mut model =
            Model::new(Scale::millimetres(pairs::WORKING_SIZE).expect("a working size"));
        let (bar, across) = a_bar_and_a_block_across_its_rim(&mut model);
        let united = fuse_and_validate(&mut model, &[bar], &[across], Rule::Union);
        answer_of(&model, &united)
    };
    for (asked, model, bar) in the_bar_united_beside_the_frame_and_the_block() {
        if let Err(why) = same_answer(&on_its_own, &answer_of(&model, &[bar]), Rule::Union, asked) {
            panic!("{why}");
        }
    }
}

/// The frame, the block beneath it, the bar and the block across its rim united
/// in one call, in each of three groupings, with the solid of the answer that
/// holds the bar.
///
/// **Not with the frame and the bar in one group**: that union is refused
/// "a point inside a piece of a face" before anything is joined, a fault of
/// its own.
fn the_bar_united_beside_the_frame_and_the_block() -> Vec<(&'static str, Model, SolidKey)> {
    let groupings: [(&str, Grouping); 3] = [
        (
            "the frame and the block across against the block and the bar",
            |[frame, block, bar, across]| (vec![frame, across], vec![block, bar]),
        ),
        (
            "the block and the bar against the frame and the block across",
            |[frame, block, bar, across]| (vec![block, bar], vec![frame, across]),
        ),
        (
            "the frame against the block, the bar and the block across",
            |[frame, block, bar, across]| (vec![frame], vec![block, bar, across]),
        ),
    ];
    groupings
        .into_iter()
        .map(|(asked, grouped)| {
            let (mut model, keys) = the_frame_and_the_block().build();
            let (bar, across) = a_bar_and_a_block_across_its_rim(&mut model);
            let (arguments, tools) = grouped([keys[FRAME], keys[BLOCK], bar, across]);
            let united = fuse_and_validate(&mut model, &arguments, &tools, Rule::Union);
            assert_eq!(united.len(), 2, "{asked}: the union is two solids");
            let holding_the_bar = *united
                .iter()
                .find(|&&solid| {
                    model
                        .solid(solid)
                        .expect("this model made the key")
                        .faces()
                        .any(|face| matches!(face.surface(), Surface::Cylinder(_)))
                })
                .unwrap_or_else(|| panic!("{asked}: no solid of the union holds the bar"));
            (asked, model, holding_the_bar)
        })
        .collect()
}

/// The arguments and the tools of one union, out of the frame, the block
/// beneath it, the bar and the block across its rim, in that order.
type Grouping = fn([SolidKey; 4]) -> (Vec<SolidKey>, Vec<SolidKey>);

fn a_bar_and_a_block_across_its_rim(model: &mut Model) -> (SolidKey, SolidKey) {
    let bar = model
        .add_round_bar(
            1.0,
            2.0,
            &Isometry::translation(Vector::new(20.0, 0.0, 0.0)).expect("a finite shift"),
        )
        .expect("a bar");
    let rectangle = ProfileBuilder::new(vec![
        [20.5, -0.25],
        [22.0, -0.25],
        [22.0, 0.375],
        [20.5, 0.375],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    let across = model
        .add_extrusion(
            &rectangle,
            1.0,
            &Isometry::translation(Vector::new(0.0, 0.0, 1.5)).expect("a finite shift"),
        )
        .expect("a positive height");
    (bar, across)
}
