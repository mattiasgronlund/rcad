//! `ResultIsNotManifold { count: 1 }` — an edge of the answer with **one** face
//! along it — is **live**, and `docs/decisions.md` §39.4 says it is withdrawn.
//!
//! # The record, and what this file measures against it
//!
//! §38.6 withdrew the row and §39.4 carries the withdrawal:
//!
//! > `ResultIsNotManifold { count: 1 }` is **withdrawn** (§38.6).
//!
//! `.claude/skills/fuse-debugging/SKILL.md` repeats it — *"`count: 1` is
//! withdrawn (§38.6); seeing it again is a regression, not a refusal"*. Both
//! sentences are false on this tree, and neither is a near miss: one pair of
//! boxes reaches it 107 times in 624 fuses.
//!
//! Both fixes that landed the day this was written are in the tree and both
//! helped. A solid's own planes now cut its own faces, and a plane and its
//! reverse name one corner — and between them they closed the case §48's lane
//! reported, a `2 × 1 × 2` block against a one-thick `Frame{3, 3}`, which
//! [`a_block_and_a_frame_that_used_to_leave_an_edge_with_one_face_along_it`]
//! pins as answered. They did not close the refusal. The lane that reported the
//! block and the frame reported the one case it had.
//!
//! # How wide the search was
//!
//! 83,268 fuses, and 14,418 of them came back `count: 1`:
//!
//! | Where the configurations came from | Fuses | `count: 1` |
//! |---|---|---|
//! | six families of `tests/pairs`, 500 draws each, all three rules both ways round | 18,000 | 81 |
//! | every turn of one pair of boxes: 342 whole-number axes × 63 angles × 3 rules | 64,638 | 14,230 |
//! | the block and the frame §48's lane reported, three rules both ways round | 6 | 0 |
//! | the sweep [`how_often_one_pair_of_turned_boxes_leaves_such_an_edge`] re-runs | 624 | 107 |
//!
//! The six families are `a_lattice_pair`, `a_flush_pair`, `a_turned_pair`,
//! `a_skew_pair`, `a_pair_somewhere_else` and a trio of lattice solids fused two
//! against one. **Two of the six reach it**: the turned pairs, and the trio. The
//! two-solid lattice, flush, skew and elsewhere families reach it in none of
//! their 3,000 fuses each, which is why the case §48's lane started from looks
//! closed when you only look there.
//!
//! # It is a gap and not a limit
//!
//! §16.10's test is whether the answer exists. It does, in both cases below, and
//! each test asserts the answer beside the refusal:
//!
//! - the trio's answer is a `1 × 2 × 1` box, and the **two halves of the same
//!   question hand it back**: one of the two arguments meets the tool in nothing
//!   at all, and the other meets it in that box, volume 2, validating;
//! - the turned pair's answer is the taller of the two boxes, and the same pair
//!   unturned hands it back, volume 2, validating.
//!
//! So a question whose two halves are both answered is refused when it is asked
//! at once, and a pair that is answered standing still is refused turned.
//! `Policy::tolerant` refuses both too, so it is not a budget.

use core::f64::consts::TAU;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

/// Two boxes fused as one argument against a third, on the integer grid, with
/// no turn anywhere — and an edge of the answer gets one face along it.
///
/// The three boxes, all `1` wide along `x`:
///
/// - `far`, `y ∈ [4, 5]`, `z ∈ [1, 2]`;
/// - `near`, `y ∈ [2, 4]`, `z ∈ [1, 2]`;
/// - `bar`, `y ∈ [0, 4]`, `z ∈ [0, 2]`.
///
/// `far` and `near` meet face to face on `y = 4`, which is also a face of `bar`,
/// and `near` shares `bar`'s `x = 0`, `x = 1` and `z = 2` planes as well. Every
/// coordinate is a whole number and every plane is axis-aligned, so nothing here
/// rounds: `(far ∪ near) ∩ bar` is exactly `near`, a `1 × 2 × 1` box of volume
/// two.
///
/// The two runs after it are the demonstration that the answer exists rather
/// than an argument that it ought to, and neither needs a word of reasoning to
/// read: `far ∩ bar` is **empty**, and `near ∩ bar` is `near`, volume two. So
/// `far` contributes nothing to `(far ∪ near) ∩ bar`, both halves of it are
/// answered on their own, and adding the solid that contributes nothing is what
/// turns the answer into a refusal.
#[test]
fn two_boxes_fused_against_a_third_leave_an_edge_with_one_face_along_it() {
    let (mut model, [far, near, bar]) = three_boxes();

    let refused = model.fuse(
        &[far, near],
        &[bar],
        Rule::Intersection,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(refused, Err(Error::ResultIsNotManifold { count: 1 })),
        "an edge of the answer got one face along it, and §39.4 calls that withdrawn: {refused:?}"
    );

    let tolerant = model.fuse(
        &[far, near],
        &[bar],
        Rule::Intersection,
        &Policy::tolerant(model.scale().nominal_size()),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(tolerant, Err(Error::ResultIsNotManifold { count: 1 })),
        "not a budget: tolerant refuses it the same way, {tolerant:?}"
    );

    let nothing = model
        .fuse(
            &[far],
            &[bar],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("`far` against `bar` is answered");
    assert_eq!(volume_of(&model, nothing.solids()), 0.0);

    let answered = model
        .fuse(
            &[near],
            &[bar],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("`near` against `bar` is answered");
    assert_eq!(volume_of(&model, answered.solids()), 2.0);
    validates(&model);
}

/// The same box and a taller copy of it in the same place, refused once the two
/// are turned — and answered when they are not.
///
/// A unit cube and a `1 × 1 × 2` box, both with their low corner at the origin,
/// so the cube is exactly the bottom half of the box and their union is the box.
/// Both are then placed by **one** rotation: a sixty-fourth of a turn about
/// `(0, 1, 1)` through the origin. The two carry the same planes in their own
/// coordinates and the same placement on top, so nothing about the pair is
/// nearly-anything — it is coincident exactly, as it is unturned.
///
/// Unturned the union is the taller box, volume two. Turned, it is
/// `ResultIsNotManifold { count: 1 }`.
#[test]
fn a_box_and_a_taller_one_in_the_same_place_are_refused_once_they_are_turned() {
    let (mut model, cube, box_) = a_box_and_a_taller_one(Some(a_sixty_fourth_of_a_turn()));
    let refused = model.fuse(
        &[cube],
        &[box_],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(refused, Err(Error::ResultIsNotManifold { count: 1 })),
        "an edge of the answer got one face along it, and §39.4 calls that withdrawn: {refused:?}"
    );

    let tolerant = model.fuse(
        &[cube],
        &[box_],
        Rule::Union,
        &Policy::tolerant(model.scale().nominal_size()),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(tolerant, Err(Error::ResultIsNotManifold { count: 1 })),
        "not a budget: tolerant refuses it the same way, {tolerant:?}"
    );

    let (mut still, cube, box_) = a_box_and_a_taller_one(None);
    let answered = still
        .fuse(
            &[cube],
            &[box_],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the same pair unturned is answered");
    assert_eq!(volume_of(&still, answered.solids()), 2.0);
    validates(&still);
}

/// How often one pair of boxes reaches it, over every turn of a small grid.
///
/// The pair is
/// [`a_box_and_a_taller_one_in_the_same_place_are_refused_once_they_are_turned`]'s,
/// turned about each of the twenty-six whole-number directions whose parts are
/// `-1`, `0` or `1`, by one to eight sixty-fourths of a turn, under all three
/// rules. **107 of the 624 come back `count: 1` and the other 517 are answered**,
/// so it is neither a corner of the grid nor most of it.
///
/// The numbers are asserted rather than printed, and §19 is why: the day the
/// refusal is closed this fails, which is the reminder that §39.4's row and
/// `fuse-debugging`'s table both need editing back. The wider grid this was cut
/// down from — every direction with parts in `-3..=3`, every angle to a whole
/// turn — is 64,638 fuses with 14,230 of them `count: 1`, and takes about a
/// minute and a half.
#[test]
fn how_often_one_pair_of_turned_boxes_leaves_such_an_edge() {
    let mut fuses = 0;
    let mut answered = 0;
    let mut one_face = 0;
    let mut something_else = 0;
    for direction in whole_number_directions() {
        for steps in 1..=8 {
            let turn = Isometry::rotation(
                &Axis::new(Point::origin(), direction).expect("a direction with length"),
                TAU * f64::from(steps) / 64.0,
            )
            .expect("a finite rotation");
            for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                let (mut model, cube, box_) = a_box_and_a_taller_one(Some(turn));
                let answer = model.fuse(
                    &[cube],
                    &[box_],
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                fuses += 1;
                match answer {
                    Ok(_) => answered += 1,
                    Err(Error::ResultIsNotManifold { count: 1 }) => one_face += 1,
                    Err(_) => something_else += 1,
                }
            }
        }
    }
    assert_eq!(fuses, 624);
    assert_eq!(one_face, 107, "an edge with one face along it, §39.4's row");
    assert_eq!(answered, 517);
    assert_eq!(something_else, 0);
}

/// The case §48's lane reported is answered now, and this is what it answers.
///
/// A `2 × 1` block extruded two at the origin, against a one-thick
/// `Frame{3, 3}` — a three-by-three square with a one-by-one hole in the middle
/// of it — put down at `[0, -2, 0]`. One argument order gave
/// `ResultIsNotManifold { count: 1 }` before the two fixes landed; all six fuses
/// answer now.
///
/// The volumes are the check and not `is_ok`. The block encloses 4 and the frame
/// 8; they overlap in `x ∈ [0, 2]`, `y ∈ [0, 1]`, `z ∈ [0, 1]`, which is 2 —
/// the frame's hole is at `y ∈ [-1, 0]` and only touches the block's `y = 0`
/// face. So union is 10, intersection 2, the block less the frame 2, and the
/// frame less the block 6. Every corner is a whole number, so these are exact.
#[test]
fn a_block_and_a_frame_that_used_to_leave_an_edge_with_one_face_along_it() {
    for (rule, block_first, expected) in [
        (Rule::Union, true, 10.0),
        (Rule::Union, false, 10.0),
        (Rule::Intersection, true, 2.0),
        (Rule::Intersection, false, 2.0),
        (Rule::Difference, true, 2.0),
        (Rule::Difference, false, 6.0),
    ] {
        let (mut model, block, frame) = a_block_and_a_frame();
        let (argument, tool) = if block_first {
            (block, frame)
        } else {
            (frame, block)
        };
        let answered = model
            .fuse(
                &[argument],
                &[tool],
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|refusal| panic!("{rule:?}, block first {block_first}: {refusal:?}"));
        assert_eq!(
            volume_of(&model, answered.solids()),
            expected,
            "{rule:?}, block first {block_first}"
        );
        validates(&model);
    }
}

fn three_boxes() -> (Model, [SolidKey; 3]) {
    let mut model = a_model();
    let far = box_from([0.0, 4.0, 1.0], [1.0, 1.0, 1.0], &mut model);
    let near = box_from([0.0, 2.0, 1.0], [1.0, 2.0, 1.0], &mut model);
    let bar = box_from([0.0, 0.0, 0.0], [1.0, 4.0, 2.0], &mut model);
    (model, [far, near, bar])
}

fn a_box_and_a_taller_one(turn: Option<Isometry>) -> (Model, SolidKey, SolidKey) {
    let mut model = a_model();
    let cube = box_from([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], &mut model);
    let taller = box_from([0.0, 0.0, 0.0], [1.0, 1.0, 2.0], &mut model);
    if let Some(turn) = turn {
        model
            .place_solid(cube, &turn)
            .expect("this model made the key");
        model
            .place_solid(taller, &turn)
            .expect("this model made the key");
    }
    (model, cube, taller)
}

fn a_sixty_fourth_of_a_turn() -> Isometry {
    Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(0.0, 1.0, 1.0)).expect("a direction with length"),
        TAU / 64.0,
    )
    .expect("a finite rotation")
}

fn whole_number_directions() -> Vec<Vector> {
    let mut out = Vec::new();
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                if [x, y, z] != [0, 0, 0] {
                    out.push(Vector::new(f64::from(x), f64::from(y), f64::from(z)));
                }
            }
        }
    }
    out
}

fn a_block_and_a_frame() -> (Model, SolidKey, SolidKey) {
    let mut model = a_model();
    let block = box_from([0.0, 0.0, 0.0], [2.0, 1.0, 2.0], &mut model);
    let outline = ProfileBuilder::new(vec![[0.0, -2.0], [3.0, -2.0], [3.0, 1.0], [0.0, 1.0]])
        .with_hole(vec![[1.0, -1.0], [2.0, -1.0], [2.0, 0.0], [1.0, 0.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a whole-number frame outline");
    let frame = model
        .add_extrusion(&outline, 1.0, &Isometry::identity())
        .expect("building it needs no decision");
    (model, block, frame)
}

fn box_from([x, y, z]: [f64; 3], [width, depth, height]: [f64; 3], model: &mut Model) -> SolidKey {
    let outline = ProfileBuilder::new(vec![
        [x, y],
        [x + width, y],
        [x + width, y + depth],
        [x, y + depth],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a whole-number rectangle");
    model
        .add_extrusion(
            &outline,
            height,
            &Isometry::translation(Vector::new(0.0, 0.0, z)).expect("a finite placement"),
        )
        .expect("building it needs no decision")
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
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

fn validates(model: &Model) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the answer is a model this kernel calls valid");
}
