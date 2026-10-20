//! A round bar of radius 1, four tall, united in one call with two blocks
//! stacked on each other, `x ∈ [0, 2]`, `y ∈ [-1, 1]`, one from `z = 0` to `2`
//! and one from `2` to `4` (row Q2473).
//!
//! The blocks' sides `y = ±1` rest along the bar's wall on the lines `x = 0,
//! y = ±1`, and their faces `x = 0` cut the bar through its axis on the same
//! lines. So the union is the half of the bar over `x < 0` and one block four
//! tall: what the bar united with the one block `z ∈ [0, 4]` answers, six
//! faces, twelve edges and eight corners.

use std::collections::{BTreeSet, HashSet};

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// **The bar and the two stacked blocks unite in one call to the bar and one
/// block four tall**, whichever group the bar is in and in either order of the
/// blocks: six faces, twelve edges and eight corners, at the one-block union's
/// corners, and holding `16 + 2π` within the error its measure bounds.
///
/// The volume is held to `16 + 2π` and not to the one-block union's bit for
/// bit. With the blocks as the argument the two differ in their last bits,
/// `22.28164832516586` against `22.281648325165822` for the one-block union
/// asked the same way round.
///
/// It answered fourteen edges and ten corners until Q2473, keeping `(0, ±1,
/// 2)`, where the arc the blocks' meeting `z = 2` cuts round the wall was
/// dissolved and the line the side `y = ±1` rests along the wall on was two
/// edges, one from each block.
#[test]
fn a_bar_and_two_stacked_blocks_united_in_one_call_is_the_bar_and_one_tall_block() {
    let policy = Policy::tolerant(10.0);
    let by_hand = 2.0 * 2.0 * 4.0 + PI * 4.0 / 2.0;
    let tall = {
        let mut model = a_model();
        let bar = a_bar(&mut model);
        let block = a_block(&mut model, [0.0, 4.0]);
        what_it_is(
            &mut model,
            &[bar],
            &[block],
            &policy,
            "the bar and one tall block",
        )
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

    for the_bar_is_the_argument in [true, false] {
        for the_lower_block_first in [true, false] {
            let asked = format!(
                "the bar is the argument {the_bar_is_the_argument}, \
                 the lower block first {the_lower_block_first}"
            );
            let mut model = a_model();
            let bar = a_bar(&mut model);
            let lower = a_block(&mut model, [0.0, 2.0]);
            let upper = a_block(&mut model, [2.0, 4.0]);
            let blocks = if the_lower_block_first {
                [lower, upper]
            } else {
                [upper, lower]
            };
            let stacked = if the_bar_is_the_argument {
                what_it_is(&mut model, &[bar], &blocks, &policy, &asked)
            } else {
                what_it_is(&mut model, &blocks, &[bar], &policy, &asked)
            };
            assert_eq!(
                (stacked.faces, stacked.edges, &stacked.corners),
                (tall.faces, tall.edges, &tall.corners),
                "{asked}: {stacked:?}"
            );
            assert!(
                (stacked.volume - by_hand).abs() <= stacked.volume_error,
                "{asked}: {stacked:?} against {by_hand}"
            );
        }
    }
}

/// How many faces and edges the one solid a union answers has, where its
/// corners are, and what it holds.
#[derive(Debug)]
struct WhatItIs {
    faces: usize,
    edges: usize,
    corners: Vec<[f64; 3]>,
    volume: f64,
    volume_error: f64,
}

fn what_it_is(
    model: &mut Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    policy: &Policy,
    asked: &str,
) -> WhatItIs {
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
    let solid = model.solid(*solid).expect("this model made the key");
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
