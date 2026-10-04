//! A block with a box-shaped hollow in it, and a bar standing through one of
//! the block's upright edges.
//!
//! The block runs from the origin to `(10, 10, 4)` and the hollow from
//! `(2, 2, 1)` to `(4, 4, 3)`, clear of every face. The bar has radius 2 and
//! stands on the block's edge at `x = y = 10`, from `z = −1` to `z = 5`, so it
//! takes a quarter-round notch out of that edge, top to bottom, and touches the
//! hollow nowhere.
//!
//! **The notch leaves the block's top a face nothing can follow.** Its side
//! along the notch is part of the circle where the bar's pipe crosses `z = 4`,
//! and the top keeps the outside of that circle, which is not one convex piece
//! of the plane (`holds.rs`, `is_a_family_this_follows`). Whether the hollow is
//! inside the block's skin is asked by firing a ray from a corner of the hollow
//! at that skin, and the first ray climbs through `z = 4`, so every corner of
//! the hollow comes back `CannotFollowAFacesTrimming` naming the top.
//!
//! The fuse used to report that as `Undecidable`, with one record — the
//! predicate "one shell inside another" at `Level::Interval`, an infinite
//! enclosure, and the input `(0, 0, 0)`: the block's own corner, which nothing
//! asked about. A caller reading it would loosen the policy, and no policy
//! moves it (`docs/decisions.md` §4.2, §35.6). It now names the face. Q504.
//!
//! **This is a refusal of an answer that exists**, a gap and not a limit
//! (§16.10): the hollow is inside the block, and the top face holds what it
//! holds. Following a face that keeps the outside of a circle is what would
//! answer it.

use rcad_geom::{Isometry, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

/// The block with its hollow, less the bar, is refused by the name of the face
/// that stopped it: the block's notched top.
#[test]
fn a_hollowed_block_less_a_bar_through_its_edge_is_refused_for_its_notched_top() {
    let mut model = a_model();
    let hollowed = a_block_with_a_hollow(&mut model);
    let bar = a_bar_through_the_blocks_edge(&mut model);
    let refused = fused(&mut model, &[hollowed], &[bar], Rule::Difference);
    assert_names_the_top(
        &model,
        hollowed,
        &refused,
        "the hollowed block less the bar",
    );
}

/// United with the bar, the hollow is still asked about against the notched
/// skin, and the refusal is the same one.
#[test]
fn a_hollowed_block_united_with_a_bar_through_its_edge_is_refused_for_its_notched_top() {
    let mut model = a_model();
    let hollowed = a_block_with_a_hollow(&mut model);
    let bar = a_bar_through_the_blocks_edge(&mut model);
    let refused = fused(&mut model, &[hollowed], &[bar], Rule::Union);
    assert_names_the_top(
        &model,
        hollowed,
        &refused,
        "the hollowed block united with the bar",
    );
}

/// The hollow and the notch taken out in one fuse: the plain block less the box
/// and the bar together is refused the same way, naming the plain block's top.
#[test]
fn a_block_less_a_box_and_a_bar_at_once_is_refused_for_its_notched_top() {
    let mut model = a_model();
    let block = a_block(&mut model, [0.0, 0.0, 0.0], [10.0, 10.0, 4.0]);
    let hollow = a_block(&mut model, [2.0, 2.0, 1.0], [2.0, 2.0, 2.0]);
    let bar = a_bar_through_the_blocks_edge(&mut model);
    let refused = fused(&mut model, &[block], &[hollow, bar], Rule::Difference);
    assert_names_the_top(
        &model,
        block,
        &refused,
        "the block less the box and the bar",
    );
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// The block less the box, which is answered: one solid of two shells.
fn a_block_with_a_hollow(model: &mut Model) -> SolidKey {
    let block = a_block(model, [0.0, 0.0, 0.0], [10.0, 10.0, 4.0]);
    let hollow = a_block(model, [2.0, 2.0, 1.0], [2.0, 2.0, 2.0]);
    let answer = model
        .fuse(
            &[block],
            &[hollow],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("the block less the box is refused: {why:?}"));
    let [solid] = answer.solids()[..] else {
        panic!(
            "the block less the box is one solid, not {:?}",
            answer.solids()
        );
    };
    assert_eq!(
        model
            .solid(solid)
            .expect("this model made the key")
            .shells()
            .count(),
        2,
        "the block less the box: its skin and the hollow"
    );
    solid
}

fn a_bar_through_the_blocks_edge(model: &mut Model) -> SolidKey {
    model
        .add_round_bar(2.0, 6.0, &at(10.0, 10.0, -1.0))
        .expect("a bar of radius two")
}

fn a_block(model: &mut Model, low: [f64; 3], size: [f64; 3]) -> SolidKey {
    let [x, y, z] = size;
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [x, 0.0], [x, y], [0.0, y]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    model
        .add_extrusion(&rectangle, z, &at(low[0], low[1], low[2]))
        .expect("a positive height")
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}

fn fused(model: &mut Model, one: &[SolidKey], other: &[SolidKey], rule: Rule) -> Error {
    let mut log = ProvenanceLog::new();
    let refused = model
        .fuse(one, other, rule, &Policy::certified(), &mut log)
        .map(|answer| answer.solids().to_vec())
        .expect_err("refused until a face keeping the outside of a circle can be followed");
    assert!(
        log.is_empty(),
        "a refusal leaves the caller's log as it was"
    );
    refused
}

/// The refusal is the face's own, and the face it names is the top of `block`
/// — a key into the caller's model, which the fuse's scratch model is not.
fn assert_names_the_top(model: &Model, block: SolidKey, refused: &Error, asked: &str) {
    let Error::AFaceTrimmedByACurveThisCannotFollow { face } = *refused else {
        panic!("{asked} is refused for the face it cannot follow, not {refused:?}");
    };
    let named = model
        .solid(block)
        .expect("this model made the key")
        .faces()
        .find(|each| each.key() == face)
        .unwrap_or_else(|| panic!("{asked} names {face:?}, which is not a face of the block"));
    let Surface::Plane(top) = named.surface() else {
        panic!("{asked} names a face on {:?}", named.surface());
    };
    assert_eq!(
        (top.normal().components(), top.offset()),
        ([0.0, 0.0, 1.0], 4.0),
        "{asked} names the block's top, at z = 4"
    );
}
