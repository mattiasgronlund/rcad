//! A round bar of radius 1, four tall, united with one block `x ∈ [0, 2]`,
//! `y ∈ [-1, 1]` from `z = 0` to `2` or from `2` to `4`, and that answer united
//! with the other block (row Q2406).
//!
//! The blocks' sides `y = ±1` rest along the bar's wall on the lines `x = 0,
//! y = ±1`, and their faces `x = 0` cut the bar through its axis on the same
//! lines. So the union is the half of the bar over `x < 0` and one block four
//! tall, the end of a slot: what the bar united with the one block `z ∈ [0, 4]`
//! answers, six faces, twelve edges and eight corners.

use std::collections::{BTreeSet, HashSet};

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// **The bar united with one block, and that answer with the other, is the bar
/// and one block four tall**, whichever block goes first and whichever side of
/// the second fuse each solid is on: six faces, twelve edges and eight corners,
/// at the one-block union's corners, and holding `16 + 2π` within the error its
/// measure bounds.
#[test]
fn a_slot_end_built_from_two_stacked_blocks_is_the_bar_and_one_tall_block() {
    let policy = Policy::tolerant(10.0);
    let by_hand = 2.0 * 2.0 * 4.0 + PI * 4.0 / 2.0;
    let tall = {
        let mut model = a_model();
        let bar = a_bar(&mut model);
        let block = a_block(&mut model, [0.0, 4.0]);
        let united = united(
            &mut model,
            &[bar],
            &[block],
            &policy,
            "the bar and one tall block",
        );
        what_it_is(&mut model, united, &policy, "the bar and one tall block")
    };
    assert_eq!(
        (tall.faces, tall.edges, tall.corners.len()),
        (6, 12, 8),
        "the bar and one tall block: {tall:?}"
    );
    assert!(
        (tall.volume - by_hand).abs() <= tall.volume_error,
        "the bar and one tall block: {tall:?} against {by_hand}"
    );

    for the_lower_block_first in [true, false] {
        for the_half_built_slot_is_the_argument in [true, false] {
            let asked = format!(
                "the lower block first {the_lower_block_first}, \
                 the half-built slot is the argument {the_half_built_slot_is_the_argument}"
            );
            let mut model = a_model();
            let bar = a_bar(&mut model);
            let lower = a_block(&mut model, [0.0, 2.0]);
            let upper = a_block(&mut model, [2.0, 4.0]);
            let (first, second) = if the_lower_block_first {
                (lower, upper)
            } else {
                (upper, lower)
            };
            let half = united(&mut model, &[bar], &[first], &policy, &asked);
            let whole = if the_half_built_slot_is_the_argument {
                united(&mut model, &[half], &[second], &policy, &asked)
            } else {
                united(&mut model, &[second], &[half], &policy, &asked)
            };
            let slot = what_it_is(&mut model, whole, &policy, &asked);
            assert_eq!(
                (slot.faces, slot.edges, &slot.corners),
                (tall.faces, tall.edges, &tall.corners),
                "{asked}: {slot:?}"
            );
            assert!(
                (slot.volume - by_hand).abs() <= slot.volume_error,
                "{asked}: {slot:?} against {by_hand}"
            );
        }
    }
}

/// How many faces and edges a solid has, where its corners are, and what it
/// holds.
#[derive(Debug)]
struct WhatItIs {
    faces: usize,
    edges: usize,
    corners: Vec<[f64; 3]>,
    volume: f64,
    volume_error: f64,
}

/// The one solid the union of `arguments` and `tools` answers, after the whole
/// model is checked.
fn united(
    model: &mut Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    policy: &Policy,
    asked: &str,
) -> SolidKey {
    let united = model
        .fuse(
            arguments,
            tools,
            Rule::Union,
            policy,
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
    model
        .validate(policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|broken| panic!("{asked}: {broken:?}"));
    let [solid] = united.solids() else {
        panic!("{asked}: one solid, not {:?}", united.solids());
    };
    *solid
}

fn what_it_is(model: &mut Model, solid: SolidKey, policy: &Policy, asked: &str) -> WhatItIs {
    let solid = model.solid(solid).expect("this model made the key");
    let mut faces = 0;
    let mut edges = HashSet::new();
    let mut corners = BTreeSet::new();
    for face in solid.faces() {
        faces += 1;
        for side in face.wires().flat_map(|wire| wire.coedges()) {
            edges.insert(side.edge().key());
            for end in [side.start(), side.end()] {
                corners.insert(end.point().coordinates().map(f64::to_bits));
            }
        }
    }
    let measured = solid
        .measure(
            Deflection::of(0.001).expect("a positive length"),
            policy,
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
    WhatItIs {
        faces,
        edges: edges.len(),
        corners: corners
            .into_iter()
            .map(|bits| bits.map(f64::from_bits))
            .collect(),
        volume: measured.volume(),
        volume_error: measured.volume_error(),
    }
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(10.0).expect("a working size"))
}

fn a_bar(model: &mut Model) -> SolidKey {
    model
        .add_round_bar(1.0, 4.0, &Isometry::identity())
        .expect("a bar")
}

/// The block `x ∈ [0, 2]`, `y ∈ [-1, 1]` from `z[0]` up to `z[1]`.
fn a_block(model: &mut Model, z: [f64; 2]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![[0.0, -1.0], [2.0, -1.0], [2.0, 1.0], [0.0, 1.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    model
        .add_extrusion(
            &rectangle,
            z[1] - z[0],
            &Isometry::translation(Vector::new(0.0, 0.0, z[0])).expect("a finite shift"),
        )
        .expect("a positive height")
}
