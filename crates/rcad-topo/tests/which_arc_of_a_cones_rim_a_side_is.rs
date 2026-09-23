//! Two arcs of a cone's rim circle between one pair of corners are two edges.
//!
//! A ball's circle has had its two arcs told apart since §194: `assemble`'s
//! `which_arc_of_a_circle_this_is` reads the sign of `∇round × ∇plane`, which
//! runs one way round a closed crossing all the way along it, so exactly one of
//! the two arcs leaves the lower-numbered corner along it. **Nothing in that
//! argument is about a ball.** What it needs is a crossing that closes, and a
//! plane square to a cone's axis cuts one — the rim circle of every cap this
//! kernel builds.
//!
//! Until that was read here the two arcs were named by their corners alone and
//! came out as one edge with four faces along it, which is not a solid a `Model`
//! can hold. §206.5 counted what it cost: 24 of the 96 fuses in
//! `how_often_a_fuse_with_a_cone_refuses.rs`'s block family, 8 of
//! `fuses_that_must_agree.rs`'s wider grid, and the four configurations below.
//!
//! # What is checked here, and why answering is half of it
//!
//! The doc comment's own invariant is that **both faces along an arc, and both
//! orders of the arguments, come to the same answer**. A fuse that answers at
//! all is the first half: if the cone's face and the cap's face disagreed about
//! which of the two arcs they were walking, each arc would be handed one face
//! instead of two, and `Model::validate` and the fuse's own manifold check both
//! refuse that. The count is asked directly all the same, so a failure says
//! *which* edge rather than only that something is wrong. The second half is
//! asked by fusing the same pair both ways round and comparing the two answers
//! bit for bit.
//!
//! # The hyperbolas are in every fuse here
//!
//! Each block has four side walls parallel to the cone's axis, and each cuts it
//! in a hyperbola. A hyperbola branch never closes, so two corners on it leave
//! one arc and not two; naming an arc there would put two edges where the answer
//! has one, and the answer would come back **wrong** rather than refused. That
//! is why `a_plane_cuts_a_cone_in_a_closed_curve` asks for a proved sign instead
//! of assuming a plane-and-cone crossing closes, and these answers coming back
//! sound is what says the `None` side of it holds too.

use std::collections::BTreeMap;

use rcad_geom::{Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey, WireStepView};

mod cones;
mod pairs;

use cones::a_pointed_cone;
use pairs::FrustumAndBlock;

/// The four configurations of §49.6's grid that used to end with four faces
/// along one edge.
///
/// Measured on the base rather than chosen: `ordering_points_along_an_endless
/// _bend.rs`'s eighty-four answered 53 there and answers 57 here, and these four
/// are the difference. Every one of them came back
/// `ResultIsNotManifold { count: 4 }` on the base.
///
/// All four have the same frustum — slope a quarter, caps of radius one and two
/// — standing in a block whose walls cut the cone in hyperbolas. What differs is
/// how wide the block is and where the axis stands in it.
const THE_FOUR_THAT_USED_TO_HAVE_FOUR_FACES_ALONG_ONE_EDGE: [FrustumAndBlock; 4] = [
    a_frustum_in_a_block(7, 3),
    a_frustum_in_a_block(7, 4),
    a_frustum_in_a_block(11, 4),
    a_frustum_in_a_block(11, 7),
];

/// One of those configurations: a block `side` across with the frustum's axis at
/// `at` in both directions.
const fn a_frustum_in_a_block(side: u32, at: u32) -> FrustumAndBlock {
    FrustumAndBlock {
        side,
        narrow: 1,
        wide: 2,
        up_per_unit_wider: 4,
        at: [at, at],
    }
}

/// All four answer now, and the model they leave behind is sound.
#[test]
fn the_four_that_used_to_have_four_faces_along_one_edge_answer() {
    for pair in THE_FOUR_THAT_USED_TO_HAVE_FOUR_FACES_ALONG_ONE_EDGE {
        let (mut model, keys) = pair.build();
        let made = fused(&mut model, &[keys[0]], &[keys[1]], Rule::Union, pair);
        assert!(
            !made.is_empty(),
            "{pair:?}: a block and a frustum that overlap unite into something"
        );
    }
}

/// Both faces along each arc name it the same edge.
///
/// This is the invariant `which_arc_of_a_circle_this_is` is written to keep,
/// asked of the answer rather than of the function: an edge with one face along
/// it is two faces that disagreed about which arc they were walking, and an edge
/// with four is two arcs that were named as one.
#[test]
fn every_edge_of_the_answer_has_exactly_two_faces_along_it() {
    for pair in THE_FOUR_THAT_USED_TO_HAVE_FOUR_FACES_ALONG_ONE_EDGE {
        let (mut model, keys) = pair.build();
        for made in fused(&mut model, &[keys[0]], &[keys[1]], Rule::Union, pair) {
            let along = how_many_faces_along_each_edge(&model, made);
            let wrong: BTreeMap<_, _> = along
                .iter()
                .filter(|&(_, &how_many)| how_many != 2)
                .collect();
            assert!(
                wrong.is_empty(),
                "{pair:?}: every edge of a solid has two faces along it, and these do not: \
                 {wrong:?}"
            );
        }
    }
}

/// Both orders of the arguments come to the same answer.
///
/// The corner numbers are the answer's and both faces share them, so swapping
/// the two sides may not change which arc is which. What is compared is the
/// answer's own shape — every face's surface and every corner it is walked
/// through, bit for bit — and never a mesh: two runs mesh a curved face in the
/// order they meet it, so a volume read off one differs from the other in the
/// last few bits without anything having changed.
#[test]
fn the_same_pair_fused_both_ways_round_comes_to_the_same_answer() {
    for pair in THE_FOUR_THAT_USED_TO_HAVE_FOUR_FACES_ALONG_ONE_EDGE {
        let (mut model, keys) = pair.build();
        let one_way = fused(&mut model, &[keys[0]], &[keys[1]], Rule::Union, pair);
        let one_way = shape_of(&model, &one_way);

        let (mut model, keys) = pair.build();
        let other_way = fused(&mut model, &[keys[1]], &[keys[0]], Rule::Union, pair);
        let other_way = shape_of(&model, &other_way);

        assert_eq!(
            one_way, other_way,
            "{pair:?}: a union is the same either way round, and here it is not"
        );
    }
}

/// A cone that ends in a point is not the ball's seam case, and does not need
/// the ball's seam guard.
///
/// `which_arc_of_a_circle_this_is` turns a whole ball's seam down because a
/// seam's material side names a half of the meridian rather than a side, and it
/// spots one by [`rcad_topo`]'s `MeetPoint::APoleOfAClosedSurface`. The only
/// thing in the crate that makes one is `Arrangement::the_whole_surface`, and
/// its first statement refuses every surface that is not a ball — so no cone
/// side can carry one.
///
/// A cone's own singular point is its apex, and it is a different thing: it is
/// where the surface's second coordinate runs out rather than a hole that has to
/// be cut open, which is why a pointed cone is built with no seam at all
/// (`tests/cones/mod.rs`). This asks the shape that has one for an answer, and
/// asks the same manifold question of it.
///
/// On the base this pair came back `ResultIsNotManifold { count: 4 }` like the
/// four above, which is the half of the claim a test can reach: the rim circle
/// of a cone that ends in a point arrives at the edge naming and is answered
/// there, with the ball's seam guard doing nothing on the way.
#[test]
fn a_cone_that_ends_in_a_point_is_answered_the_same_way() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let block = model
        .add_cube(7.0, &policy, &mut ProvenanceLog::new())
        .expect("a whole-number cube");
    let cone = a_pointed_cone(&mut model, 0.25, 8.0);
    model
        .place_solid(
            cone,
            &Isometry::translation(Vector::new(3.0, 3.0, -5.0)).expect("a whole-number shift"),
        )
        .expect("this model made the key");
    let made = model
        .fuse(
            &[block],
            &[cone],
            Rule::Union,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a pointed cone standing up through a cube's floor")
        .solids()
        .to_vec();
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("a fuse hands back a sound model or refuses");
    for solid in made {
        for (edge, how_many) in how_many_faces_along_each_edge(&model, solid) {
            assert_eq!(
                how_many, 2,
                "a pointed cone's answer has {how_many} faces along {edge}"
            );
        }
    }
}

/// Fuse, insisting on an answer and on a sound model.
///
/// # Panics
///
/// If the fuse refuses. On the base every call here refuses with
/// `Error::ResultIsNotManifold { count: 4 }`, which is this file failing without
/// the change it is for.
fn fused(
    model: &mut Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
    pair: FrustumAndBlock,
) -> Vec<SolidKey> {
    let policy = Policy::certified();
    let made = model
        .fuse(arguments, tools, rule, &policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{pair:?} under {rule:?}: {why}"))
        .solids()
        .to_vec();
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("a fuse hands back a sound model or refuses");
    made
}

/// What an answer is, written so that two answers can be compared without
/// depending on the order the faces came out in.
///
/// One entry per face: the surface it sits on and, per wire, the corners that
/// wire is walked through. Coordinates go in as bits, so this is exact.
fn shape_of(model: &Model, made: &[SolidKey]) -> Vec<Vec<String>> {
    let mut answer: Vec<Vec<String>> = made
        .iter()
        .map(|&solid| {
            let mut faces: Vec<String> = model
                .solid(solid)
                .expect("a solid the fuse made")
                .faces()
                .map(|face| {
                    let mut wires: Vec<String> = face
                        .wires()
                        .map(|wire| {
                            let mut at: Vec<[u64; 3]> = wire
                                .steps()
                                .map(|step| {
                                    let WireStepView::Along(coedge) = step else {
                                        panic!("these answers are made of coedges: {step:?}")
                                    };
                                    coedge.start().point().coordinates().map(f64::to_bits)
                                })
                                .collect();
                            at.sort_unstable();
                            format!("{at:?}")
                        })
                        .collect();
                    wires.sort();
                    format!("{:?} {:?} {wires:?}", face.surface(), face.orientation())
                })
                .collect();
            faces.sort();
            faces
        })
        .collect();
    answer.sort();
    answer
}

/// How many coedges — which is how many faces — walk each edge of a solid.
///
/// An edge is named by the `Debug` spelling of its key, because a key carries no
/// order of its own and a `BTreeMap` needs one. It is exact all the same: two
/// different keys never print alike.
fn how_many_faces_along_each_edge(model: &Model, solid: SolidKey) -> BTreeMap<String, usize> {
    let view = model.solid(solid).expect("a solid the fuse made");
    let mut along: BTreeMap<String, usize> = BTreeMap::new();
    for face in view.faces() {
        for wire in face.wires() {
            for step in wire.steps() {
                let WireStepView::Along(coedge) = step else {
                    panic!("these answers are made of coedges and nothing else: {step:?}")
                };
                *along
                    .entry(format!("{:?}", coedge.edge().key()))
                    .or_default() += 1;
            }
        }
    }
    along
}
