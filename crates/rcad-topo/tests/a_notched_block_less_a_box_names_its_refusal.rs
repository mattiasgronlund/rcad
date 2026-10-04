//! A block notched by a bar, less a box inside it: the same block, bar and box
//! as `a_shell_inside_another_names_its_refusal.rs`, with the box taken out
//! last rather than first.
//!
//! The block runs from the origin to `(10, 10, 4)`. The bar has radius 2 and
//! stands on the block's edge at `x = y = 10`, from `z = −1` to `z = 5`, so it
//! takes a quarter-round notch out of that edge, top to bottom. The box runs
//! from `(2, 2, 1)` to `(4, 4, 3)`, clear of every face and of the notch.
//!
//! **The notch leaves the block's top a face nothing can follow.** Its side
//! along the notch is part of the circle where the bar's pipe crosses `z = 4`,
//! and the top keeps the outside of that circle (`holds.rs`,
//! `is_a_family_this_follows`).
//!
//! Taken out last, the box's floor at `z = 1` is one piece, and it is placed
//! against the notched block by a point inside it. A print in `select.rs`'s
//! `settle_against_each`, taken out, showed all four points tried —
//! `(2.5, 2.5, 1)`, `(2.5, 3.5, 1)`, `(3.5, 3.5, 1)` and `(3.5, 2.5, 1)` —
//! coming back `CannotFollowAFacesTrimming` naming the notched top. The fuse
//! folded that into `Undecidable` under "a point in a piece that every solid
//! could place", the floor's four corners its inputs, at `Level::Interval` with
//! an infinite enclosure. A caller reading it would loosen the policy, and no
//! policy moves it (`docs/decisions.md` §4.2, §35.6). It now names the face,
//! as the box taken out first has since Q504. Q882.
//!
//! **This is a refusal of an answer that exists**, a gap and not a limit
//! (§16.10): the box is inside the notched block, and the top holds what it
//! holds. Following a face that keeps the outside of a circle is what would
//! answer it, and that is Q969's.

use rcad_geom::{Isometry, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

/// The box taken out last: the notched block less the box is refused by the
/// name of the notched block's top.
#[test]
fn a_notched_block_less_a_box_is_refused_for_its_notched_top() {
    let mut model = a_model();
    let notched = a_notched_block(&mut model);
    let boxed = the_box(&mut model);
    let refused = fused(&mut model, &[notched], &[boxed], Rule::Difference);
    assert_names_the_top(&model, notched, &refused, "the notched block less the box");
}

/// The box taken out first: the hollowed block less the bar is refused the
/// same way, naming the hollowed block's top. The same fuse is pinned in
/// `a_shell_inside_another_names_its_refusal.rs`; here it is the other order
/// of the one above, and the two must not name different refusals.
#[test]
fn a_hollowed_block_less_a_bar_is_refused_for_its_notched_top_too() {
    let mut model = a_model();
    let hollowed = a_hollowed_block(&mut model);
    let bar = a_bar_through_the_blocks_edge(&mut model);
    let refused = fused(&mut model, &[hollowed], &[bar], Rule::Difference);
    assert_names_the_top(
        &model,
        hollowed,
        &refused,
        "the hollowed block less the bar",
    );
}

/// The box's floor is placed against the notched block whatever the rule, so
/// united with it and cut down to it the refusal is the same one.
#[test]
fn a_notched_block_united_with_or_cut_down_to_a_box_is_refused_for_its_notched_top() {
    for (rule, asked) in [
        (Rule::Union, "the notched block united with the box"),
        (Rule::Intersection, "the notched block cut down to the box"),
    ] {
        let mut model = a_model();
        let notched = a_notched_block(&mut model);
        let boxed = the_box(&mut model);
        let refused = fused(&mut model, &[notched], &[boxed], rule);
        assert_names_the_top(&model, notched, &refused, asked);
    }
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// The block less the bar, which is answered: one solid.
fn a_notched_block(model: &mut Model) -> SolidKey {
    let block = a_block(model, [0.0, 0.0, 0.0], [10.0, 10.0, 4.0]);
    let bar = a_bar_through_the_blocks_edge(model);
    one_answered(model, block, bar, "the block less the bar")
}

/// The block less the box, which is answered: one solid.
fn a_hollowed_block(model: &mut Model) -> SolidKey {
    let block = a_block(model, [0.0, 0.0, 0.0], [10.0, 10.0, 4.0]);
    let boxed = the_box(model);
    one_answered(model, block, boxed, "the block less the box")
}

fn one_answered(model: &mut Model, one: SolidKey, less: SolidKey, asked: &str) -> SolidKey {
    let answer = model
        .fuse(
            &[one],
            &[less],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{asked} is refused: {why:?}"));
    let [solid] = answer.solids()[..] else {
        panic!("{asked} is one solid, not {:?}", answer.solids());
    };
    solid
}

fn the_box(model: &mut Model) -> SolidKey {
    a_block(model, [2.0, 2.0, 1.0], [2.0, 2.0, 2.0])
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
