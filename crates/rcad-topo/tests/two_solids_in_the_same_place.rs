//! Two solids that fill some of the same space are refused; two that only touch
//! are not.
//!
//! `docs/roadmap.md` §7.4 names the two degenerate cases this is about — a
//! solid **fully contained** in another, and two solids **touching but not
//! overlapping** — and `docs/decisions.md` §43.2 criterion 8 is the work.
//!
//! **Why this is not a bounding-box test wearing a hat.** Five of the six pairs
//! below have overlapping boxes, and the pass has to say different things about
//! them: two copies of one cube share space, a cube in the middle of a bigger
//! one shares space, and two cubes stacked face to face do not. Only one pair is
//! decided by the boxes. Drop the stacked cubes and a box test would pass this
//! file, so their two boxes are asserted to meet rather than assumed to.
//!
//! The last two are the gap, pinned rather than described: two bars crossing in
//! a plus, and two balls at one place. Both share space and the pass walks past
//! both. They are here so that closing the gap moves a test rather than a
//! sentence.
//!
//! **The oracle is an exactly true answer** (testing skill's first row). Every
//! cube here has whole-number corners a `f64` holds exactly, every placement is
//! a whole-number shift, and `Policy::certified()` is what asks: no answer below
//! is within anything of anything, and nothing is approximated, which the empty
//! provenance log asserts.

use rcad_geom::{Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, SharedSpace, SolidKey};

/// Two descriptions of the very same cube fill the very same space.
///
/// No corner of either is *strictly* inside the other — every one of them is a
/// corner of the other cube too — so the corner search comes up empty and what
/// answers is the pair of faces sitting on one plane with both cubes' material
/// on the same side of it.
#[test]
fn two_cubes_at_the_same_place_share_space() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let one = a_cube_of_side_two(&mut model, Vector::new(0.0, 0.0, 0.0));
    let other = a_cube_of_side_two(&mut model, Vector::new(0.0, 0.0, 0.0));

    assert!(the_boxes_meet(&model, one, other));
    let answer = model
        .solid(one)
        .shares_space_with(model.solid(other), &policy, &mut log);
    assert_eq!(answer, SharedSpace::Shared);

    let refused = model.check_no_two_solids_share_space(&policy, &mut log);
    assert!(
        matches!(refused, Err(Error::TwoSolidsShareSpace { .. })),
        "expected the pass to refuse two cubes at the same place, got {refused:?}"
    );
    assert!(
        log.is_empty(),
        "whole-number cubes need nothing approximated"
    );
}

/// A cube in the middle of a bigger one is inside it, and that is roadmap
/// §7.4's *fully contained*.
///
/// The small cube runs from `0.5` to `1.5` on every axis and the big one from
/// `0` to `2`, so no face of either is on a plane of the other and the corner
/// search is the only thing that can answer.
#[test]
fn a_cube_wholly_inside_a_bigger_one_shares_its_space() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let big = a_cube_of_side_two(&mut model, Vector::new(0.0, 0.0, 0.0));
    let small = a_cube_of_side(&mut model, 1.0, Vector::new(0.5, 0.5, 0.5));

    let answer = model
        .solid(big)
        .shares_space_with(model.solid(small), &policy, &mut log);
    assert_eq!(answer, SharedSpace::Shared);

    let refused = model.check_no_two_solids_share_space(&policy, &mut log);
    assert!(
        matches!(refused, Err(Error::TwoSolidsShareSpace { .. })),
        "expected the pass to refuse a cube inside a cube, got {refused:?}"
    );
    assert!(
        log.is_empty(),
        "whole-number cubes need nothing approximated"
    );
}

/// Two cubes stacked face to face touch and share no space, and the pass lets
/// them through.
///
/// This is the case that keeps the pass from being the box test: the two boxes
/// meet along the whole of a face, exactly as the two coincident cubes' boxes
/// do. What separates them is which way the shared faces point out of their own
/// material — away from each other here, and the same way there.
#[test]
fn two_cubes_stacked_face_to_face_share_no_space() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let below = a_cube_of_side_two(&mut model, Vector::new(0.0, 0.0, 0.0));
    let above = a_cube_of_side_two(&mut model, Vector::new(0.0, 0.0, 2.0));

    assert!(
        the_boxes_meet(&model, below, above),
        "the point of this test is that the boxes meet exactly as two cubes at \
         one place do, so a box test would refuse these too"
    );
    let answer = model
        .solid(below)
        .shares_space_with(model.solid(above), &policy, &mut log);
    assert_eq!(answer, SharedSpace::NothingFound);

    model
        .check_no_two_solids_share_space(&policy, &mut log)
        .expect("two cubes stacked face to face share no space");
    assert!(
        log.is_empty(),
        "whole-number cubes need nothing approximated"
    );
}

/// Two cubes a long way apart share no space, and the boxes alone prove it.
#[test]
fn two_cubes_plainly_apart_share_no_space() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let here = a_cube_of_side_two(&mut model, Vector::new(0.0, 0.0, 0.0));
    let far = a_cube_of_side_two(&mut model, Vector::new(10.0, 0.0, 0.0));

    let answer = model
        .solid(here)
        .shares_space_with(model.solid(far), &policy, &mut log);
    assert_eq!(answer, SharedSpace::Apart);

    model
        .check_no_two_solids_share_space(&policy, &mut log)
        .expect("two cubes ten apart share no space");
    assert!(
        log.is_empty(),
        "whole-number cubes need nothing approximated"
    );
}

/// Two bars crossing in a plus really do share space, and this pass walks past
/// them.
///
/// The module doc of `rcad_topo`'s overlap pass says so; this is that sentence
/// run rather than said (`docs/decisions.md` §16.10). The long bar fills
/// `0..6` by `0..2` by `0..2` and the crossing one `2..4` by `-2..4` by
/// `0.5..1.5`, so the block `2..4` by `0..2` by `0.5..1.5` is in both. No corner
/// of either is inside the other — every corner of the crossing bar is past the
/// long one in `y`, and every corner of the long one is past the crossing one in
/// `x` — and no face of either is on a plane of the other.
///
/// **A gap and not a limit.** The answer exists and an edge of one bar tested
/// against a face of the other would find it. When that is built this test
/// changes to `SharedSpace::Shared`, which is the point of pinning it.
#[test]
fn two_bars_crossing_in_a_plus_share_space_and_this_pass_does_not_see_it() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let long = a_bar(
        &mut model,
        [[0.0, 0.0], [6.0, 0.0], [6.0, 2.0], [0.0, 2.0]],
        2.0,
        0.0,
    );
    let across = a_bar(
        &mut model,
        [[2.0, -2.0], [4.0, -2.0], [4.0, 4.0], [2.0, 4.0]],
        1.0,
        0.5,
    );

    assert!(the_boxes_meet(&model, long, across));
    let answer = model
        .solid(long)
        .shares_space_with(model.solid(across), &policy, &mut log);
    assert_eq!(
        answer,
        SharedSpace::NothingFound,
        "the two bars share a whole block and nothing here finds it"
    );
}

/// Two balls at the same place share all their space, and this pass walks past
/// them too.
///
/// The same gap from the other side: what refuses two cubes at one place is a
/// pair of faces sitting on one *plane*, and a ball's skin is not one. The
/// corner search is empty for the same reason it is with the cubes — the seam
/// corners of each ball are on the other's skin.
///
/// **A gap and not a limit.** Two faces on one sphere, facing their material the
/// same way, is the same argument the plane rule makes; nothing here asks it.
#[test]
fn two_balls_at_the_same_place_share_space_and_this_pass_does_not_see_it() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let one = a_ball(&mut model);
    let other = a_ball(&mut model);

    assert!(the_boxes_meet(&model, one, other));
    let answer = model
        .solid(one)
        .shares_space_with(model.solid(other), &policy, &mut log);
    assert_eq!(
        answer,
        SharedSpace::NothingFound,
        "the two balls are the same ball and nothing here finds it"
    );
}

fn a_ball(model: &mut Model) -> SolidKey {
    model
        .add_ball(4.0, &Isometry::identity())
        .expect("a ball of radius four")
}

/// A bar: a rectangle in `x` and `y`, pulled `height` up from `bottom`.
fn a_bar(model: &mut Model, corners: [[f64; 2]; 4], height: f64, bottom: f64) -> SolidKey {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let outline = ProfileBuilder::new(corners.to_vec())
        .build(&policy, &mut log)
        .expect("four whole-number corners make a rectangle");
    let bar = model
        .add_extrusion(
            &outline,
            height,
            &Isometry::translation(Vector::new(0.0, 0.0, bottom)).expect("a finite placement"),
        )
        .expect("a rectangle pulled up is a bar");
    assert!(log.is_empty(), "a whole-number bar approximates nothing");
    bar
}

/// Whether the boxes round the two solids have any point in common.
///
/// Two cubes at one place and two cubes stacked face to face both say yes, and
/// the pass says different things about them. That is what makes the two tests
/// a pair rather than a repetition.
fn the_boxes_meet(model: &Model, one: SolidKey, other: SolidKey) -> bool {
    let mine = model.solid(one).box_around_it().expect("a cube has a box");
    let theirs = model
        .solid(other)
        .box_around_it()
        .expect("a cube has a box");
    (0..3).all(|axis| {
        mine.0.coordinates()[axis] <= theirs.1.coordinates()[axis]
            && theirs.0.coordinates()[axis] <= mine.1.coordinates()[axis]
    })
}

fn a_cube_of_side_two(model: &mut Model, at: Vector) -> SolidKey {
    a_cube_of_side(model, 2.0, at)
}

fn a_cube_of_side(model: &mut Model, side: f64, at: Vector) -> SolidKey {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let cube = model
        .add_cube(side, &policy, &mut log)
        .expect("a cube with a positive side");
    assert!(log.is_empty(), "a whole-number cube approximates nothing");
    model.place_solid(
        cube,
        &Isometry::translation(at).expect("a finite placement"),
    );
    cube
}

fn a_working_size() -> Scale {
    Scale::millimetres(100.0).expect("a hundred millimetres is a working size")
}
