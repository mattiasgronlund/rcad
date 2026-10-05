//! A block notched by a bar, less a box inside it: the same block, bar and box
//! as `a_shell_inside_another_names_its_refusal.rs`, with the box taken out
//! last rather than first.
//!
//! The block runs from the origin to `(10, 10, 4)`. The bar has radius 2 and
//! stands on the block's edge at `x = y = 10`, from `z = −1` to `z = 5`, so it
//! takes a quarter-round notch out of that edge, top to bottom. The box runs
//! from `(2, 2, 1)` to `(4, 4, 3)`, clear of every face and of the notch.
//!
//! **The notch leaves the block's top keeping the outside of a circle.** Its
//! side along the notch is part of the circle where the bar's pipe crosses
//! `z = 4`. Taken out last, the box's floor at `z = 1` is one piece, placed
//! against the notched block by a point inside it, and every ray from such a
//! point climbs through that top.
//!
//! Row Q882 found those rays coming back `CannotFollowAFacesTrimming` naming
//! the top, and the fuse folding that into `Undecidable` under "a point in a
//! piece that every solid could place", which a caller would read as "loosen
//! the policy" and no policy moves (`docs/decisions.md` §4.2, §35.6). It made
//! the fuse name the face instead, and pinned that here. **It was a refusal of
//! an answer that exists**, a gap and not a limit (§16.10), and the file's name
//! still says refusal. Since row Q1002 a flat face the cuts cannot follow winds
//! its own sides, so the top says what it holds and every fuse here answers.
//! The refusal Q882 named is pinned on shapes that still reach it, in
//! `a_narrow_lune_from_a_pole_corner.rs`.
//!
//! Each answer is checked against the volume worked out by hand: the block's
//! 400, less the quarter of the bar inside it, `4π`, less the box's 8 where the
//! box is taken out.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// The box taken out last: one solid, its notched skin and the box's hollow.
#[test]
fn a_notched_block_less_a_box_keeps_the_box_as_a_hollow() {
    let mut model = a_model();
    let notched = a_notched_block(&mut model);
    let boxed = the_box(&mut model);
    one_solid_measuring(
        &mut model,
        &[notched],
        &[boxed],
        Rule::Difference,
        Answer {
            shells: 2,
            by_hand: 392.0 - 4.0 * PI,
        },
    );
}

/// The box taken out first, then the bar: the same solid as the box taken out
/// last, so the two orders measure the same and keep the same hollow.
#[test]
fn a_hollowed_block_less_a_bar_is_the_notched_block_less_the_box() {
    let mut model = a_model();
    let hollowed = a_hollowed_block(&mut model);
    let bar = a_bar_through_the_blocks_edge(&mut model);
    one_solid_measuring(
        &mut model,
        &[hollowed],
        &[bar],
        Rule::Difference,
        Answer {
            shells: 2,
            by_hand: 392.0 - 4.0 * PI,
        },
    );
}

/// United with the box inside it, the notched block is the notched block; cut
/// down to the box, it is the box.
#[test]
fn a_notched_block_united_with_or_cut_down_to_a_box_inside_it_is_one_or_the_other() {
    for (rule, by_hand) in [(Rule::Union, 400.0 - 4.0 * PI), (Rule::Intersection, 8.0)] {
        let mut model = a_model();
        let notched = a_notched_block(&mut model);
        let boxed = the_box(&mut model);
        one_solid_measuring(
            &mut model,
            &[notched],
            &[boxed],
            rule,
            Answer { shells: 1, by_hand },
        );
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

/// What one fuse here answers: one solid, of this many shells, whose volume is
/// `by_hand`.
struct Answer {
    shells: usize,
    by_hand: f64,
}

/// The fuse answers one solid of `want.shells` shells, with nothing in the log,
/// whose volume is `want.by_hand` to within what the mesh could be hiding.
fn one_solid_measuring(
    model: &mut Model,
    one: &[SolidKey],
    other: &[SolidKey],
    rule: Rule,
    want: Answer,
) {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let answer = model
        .fuse(one, other, rule, &policy, &mut log)
        .unwrap_or_else(|why| panic!("{rule:?} is refused: {why:?}"));
    assert!(log.is_empty(), "a certified fuse logs nothing: {log:?}");
    let [solid] = answer.solids()[..] else {
        panic!("{rule:?} is one solid, not {:?}", answer.solids());
    };
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{rule:?}: the model is unsound: {why:?}"));
    let view = model.solid(solid).expect("this model made the key");
    assert_eq!(view.shells().count(), want.shells, "{rule:?}: its shells");
    let measured = view
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a solid measures");
    assert!(
        (measured.volume() - want.by_hand).abs() <= measured.volume_error(),
        "{rule:?}: measured {} ± {} against {} by hand",
        measured.volume(),
        measured.volume_error(),
        want.by_hand
    );
}
