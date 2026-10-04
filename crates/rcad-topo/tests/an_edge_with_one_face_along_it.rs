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
//! sentences were false when this file was written, and neither was a near
//! miss: one pair of boxes reached it 104 times in 624 fuses.
//!
//! **That pair reaches it in none of the 624 now** (`docs/decisions.md`
//! §351.6). The cube's top corners lie on the taller box's side edges, the
//! shape §351.6 traced in a comb and an ell, where one solid's pieces named
//! the corner as a vertex and the other's as the meet of three planes, and the
//! two rounded apart. An input corner with exactly three planes round it is
//! named by those planes now, and that took this count from 104 to 0. The
//! turned pairs of `fuses_that_must_agree.rs` that are still not a solid on the
//! shipped draw are all `count: 4`. Whether a turned pair still reaches
//! `count: 1` anywhere was not searched again.
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
//! | the sweep [`how_often_one_pair_of_turned_boxes_leaves_such_an_edge`] re-runs | 624 | 104 |
//!
//! The six families are `a_lattice_pair`, `a_flush_pair`, `a_turned_pair`,
//! `a_skew_pair`, `a_pair_somewhere_else` and a trio of lattice solids fused two
//! against one. **Two of the six reached it** when this was measured: the turned
//! pairs, and the trio. The two-solid lattice, flush, skew and elsewhere
//! families reached it in none of their 3,000 fuses each, which is why the case
//! §48's lane started from looks closed when you only look there.
//!
//! **Only the turned pairs reach it now.** The trio's refusal was a fuse
//! reading a piece flush with two solids by whichever one was named first; once
//! a piece that one solid faces and another backs counted as inside both,
//! [`two_boxes_fused_against_a_third_that_used_to_leave_an_edge_with_one_face_along_it`]
//! answered, and it now pins that answer. The table above is the search as it
//! was run and was not re-run, so its `81` still counts the trio's refusals.
//!
//! # It was a gap and not a limit
//!
//! §16.10's test is whether the answer exists. It did for the turned pair: its
//! answer is the taller of the two boxes, and the same pair unturned hands it
//! back, volume 2, validating. So a pair that was answered standing still was
//! refused turned. `Policy::tolerant` refused it too, so it was not a budget.
//! Both policies answer it now.

#[path = "fuzzing/comparison.rs"]
mod comparison;

use core::f64::consts::TAU;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

use comparison::{Answer, answer_of, same_answer};

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
/// It was refused `ResultIsNotManifold { count: 1 }`, certified and tolerant
/// alike, until a fuse stopped reading a piece flush with two solids by
/// whichever of them was named first. `bar`'s `y = 4` face, where
/// `z ∈ [1, 2]`, is such a piece: it lies on `near` facing the same way and on
/// `far` facing the other, so the union of the two has material on both sides
/// of it.
///
/// The answer is checked against `near ∩ bar`, which reaches the same region
/// without `far`: `far ∩ bar` is **empty**, so `far` contributes nothing, and
/// the two must agree bit for bit.
#[test]
fn two_boxes_fused_against_a_third_that_used_to_leave_an_edge_with_one_face_along_it() {
    let (mut model, [far, near, bar]) = three_boxes();
    let (both, volume) = intersected(&mut model, &[far, near], bar);
    assert_eq!(volume, 2.0, "`(far ∪ near) ∩ bar` is `near`");

    let (mut model, [far, _, bar]) = three_boxes();
    let (_, volume) = intersected(&mut model, &[far], bar);
    assert_eq!(volume, 0.0, "`far` meets `bar` in a face and nothing more");

    let (mut model, [_, near, bar]) = three_boxes();
    let (near_alone, volume) = intersected(&mut model, &[near], bar);
    assert_eq!(volume, 2.0);

    assert_eq!(
        same_answer(
            &near_alone,
            &both,
            Rule::Intersection,
            "adding a box that only touches the tool"
        ),
        Ok(())
    );
}

/// The answer's geometry and its volume, once it has been checked to validate.
fn intersected(model: &mut Model, arguments: &[SolidKey], tool: SolidKey) -> (Answer, f64) {
    let solids = model
        .fuse(
            arguments,
            &[tool],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("boxes on the integer grid leave nothing to round")
        .solids()
        .to_vec();
    validates(model);
    (answer_of(model, &solids), volume_of(model, &solids))
}

/// The same box and a taller copy of it in the same place unite to the taller
/// box turned, as they do unturned.
///
/// A unit cube and a `1 × 1 × 2` box, both with their low corner at the origin,
/// so the cube is exactly the bottom half of the box and their union is the box.
/// Both are then placed by **one** rotation: a sixty-fourth of a turn about
/// `(0, 1, 1)` through the origin. The two carry the same planes in their own
/// coordinates and the same placement on top, so nothing about the pair is
/// nearly-anything — it is coincident exactly, as it is unturned.
///
/// Turned, it was `ResultIsNotManifold { count: 1 }` under both policies. The
/// cube's top corners lie on the taller box's side edges, the shape of
/// `a_turned_tooth_corner_on_the_edge_of_an_ell.rs`, and the union has answered
/// since an input corner with exactly three planes round it is named by those
/// planes (`docs/decisions.md` §351.6).
#[test]
fn a_box_and_a_taller_one_in_the_same_place_unite_to_the_taller_box_turned_too() {
    for policy in [
        Policy::certified(),
        Policy::tolerant(a_model().scale().nominal_size()),
    ] {
        let (mut model, cube, box_) = a_box_and_a_taller_one(Some(a_sixty_fourth_of_a_turn()));
        let answered = model
            .fuse(
                &[cube],
                &[box_],
                Rule::Union,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|refusal| panic!("the turned pair unites: {refusal:?}"));
        assert_eq!(answered.solids().len(), 1);
        let volume = volume_of(&model, answered.solids());
        assert!(
            (volume - 2.0).abs() < 1e-12,
            "the turned union is the taller box, and it takes up {volume}"
        );
        validates(&model);
    }

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
/// [`a_box_and_a_taller_one_in_the_same_place_unite_to_the_taller_box_turned_too`]'s,
/// turned about each of the twenty-six whole-number directions whose parts are
/// `-1`, `0` or `1`, by one to eight sixty-fourths of a turn, under all three
/// rules. **All 624 are answered.** 104 came back `count: 1` and 520 were
/// answered until an input corner with exactly three planes round it was named
/// by those planes (`docs/decisions.md` §351.6).
///
/// The numbers are asserted rather than printed, and §19 is why: a refusal
/// coming back fails here. The wider grid this was cut down from, every
/// direction with parts in `-3..=3` and every angle to a whole turn, was 64,638
/// fuses with 14,230 of them `count: 1`, and was not run again.
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
    assert_eq!(one_face, 0, "an edge with one face along it, §39.4's row");
    assert_eq!(answered, 624);
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
