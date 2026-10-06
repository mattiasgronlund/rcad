//! Hollows whose corners are asked whether they are inside the shell round
//! them: a box in a notched block, and a ball touching a block's skin with its
//! poles.
//!
//! # A box in a block with a bar through its edge
//!
//! The block runs from the origin to `(10, 10, 4)` and the hollow from
//! `(2, 2, 1)` to `(4, 4, 3)`, clear of every face. The bar has radius 2 and
//! stands on the block's edge at `x = y = 10`, from `z = −1` to `z = 5`, so it
//! takes a quarter-round notch out of that edge, top to bottom, and touches the
//! hollow nowhere.
//!
//! **The notch leaves the block's top keeping the outside of a circle.** Its
//! side along the notch is part of the circle where the bar's pipe crosses
//! `z = 4`, which is not one convex piece of the plane (`holds.rs`,
//! `is_a_family_this_follows`). Whether the hollow is inside the block's skin
//! is asked by firing a ray from a corner of the hollow at that skin, and the
//! first ray climbs through `z = 4`, so the top has to say what it holds.
//!
//! The fuse first reported that it could not as `Undecidable`, with one record
//! naming the block's own corner, which nothing asked about (`docs/decisions.md`
//! §4.2, §35.6), and then, from Q504, as a refusal naming the top. **It was a
//! refusal of an answer that exists**, a gap and not a limit (§16.10), and the
//! file's name still says refusal. Since row Q1002 a flat face the cuts cannot
//! follow winds its own sides, each arc read as its chord with the piece
//! between the two added or taken away, so the top answers and so does every
//! fuse here. Each answer is checked against the volume worked out by hand: the
//! block's 400, less the hollow's 8, with the quarter of the bar inside the
//! block, `4π`, taken away or the rest of the bar, `20π`, added.
//!
//! # A ball touching the block's top with its north pole
//!
//! The same block less a ball of radius 1 centred at `(5, 5, 3)`. The ball's
//! north pole, `(5, 5, 4)`, is on the block's top and its south pole,
//! `(5, 5, 2)`, is inside the block. A ball's one face is walked down its seam
//! from the north pole and back up it from the south pole, and the hollow's
//! corners were asked as the **first end of each edge**, so the north pole was
//! asked twice, came back on the skin twice, and the south pole was never asked
//! at all: `Undecidable`, for a hollow whose other corner answers. Each side is
//! now asked from the corner it starts at (row Q883).

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// The block with its hollow, less the bar: the hollow stays a shell of its
/// own, and the notch takes a quarter of the bar out of the block.
#[test]
fn a_hollowed_block_less_a_bar_through_its_edge_keeps_its_hollow() {
    let mut model = a_model();
    let hollowed = a_block_with_a_hollow(&mut model);
    let bar = a_bar_through_the_blocks_edge(&mut model);
    the_one_solid_with_its_hollow(
        &mut model,
        &[hollowed],
        &[bar],
        Rule::Difference,
        392.0 - 4.0 * PI,
    );
}

/// United with the bar, the hollow is still asked about against the notched
/// skin, and stays a shell of its own inside the block and the bar together.
#[test]
fn a_hollowed_block_united_with_a_bar_through_its_edge_keeps_its_hollow() {
    let mut model = a_model();
    let hollowed = a_block_with_a_hollow(&mut model);
    let bar = a_bar_through_the_blocks_edge(&mut model);
    the_one_solid_with_its_hollow(
        &mut model,
        &[hollowed],
        &[bar],
        Rule::Union,
        392.0 + 20.0 * PI,
    );
}

/// The hollow and the notch taken out in one fuse: the plain block less the box
/// and the bar together is the same solid as the hollowed block less the bar.
#[test]
fn a_block_less_a_box_and_a_bar_at_once_keeps_its_hollow() {
    let mut model = a_model();
    let block = a_block(&mut model, [0.0, 0.0, 0.0], [10.0, 10.0, 4.0]);
    let hollow = a_block(&mut model, [2.0, 2.0, 1.0], [2.0, 2.0, 2.0]);
    let bar = a_bar_through_the_blocks_edge(&mut model);
    the_one_solid_with_its_hollow(
        &mut model,
        &[block],
        &[hollow, bar],
        Rule::Difference,
        392.0 - 4.0 * PI,
    );
}

/// The block less a ball whose north pole is on the block's top: one solid,
/// the block's skin and the ball's, holding the block's 400 less the ball's
/// `4π/3`.
#[test]
fn a_block_less_a_ball_touching_its_top_with_one_pole_keeps_the_ball_as_a_hollow() {
    let mut model = a_model();
    let block = a_block(&mut model, [0.0, 0.0, 0.0], [10.0, 10.0, 4.0]);
    let ball = a_ball(&mut model, 1.0, at(5.0, 5.0, 3.0));
    the_one_solid_with_its_hollow(
        &mut model,
        &[block],
        &[ball],
        Rule::Difference,
        400.0 - 4.0 * PI / 3.0,
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

fn a_ball(model: &mut Model, radius: f64, centre: Isometry) -> SolidKey {
    model
        .add_ball(radius, &centre)
        .expect("a ball of positive radius")
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}

/// The fuse answers one solid with two shells, its skin and the hollow, whose
/// volume is `by_hand` to within what the mesh could be hiding.
fn the_one_solid_with_its_hollow(
    model: &mut Model,
    one: &[SolidKey],
    other: &[SolidKey],
    rule: Rule,
    by_hand: f64,
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
    let view = model.solid(solid).expect("this model made the key");
    assert_eq!(
        view.shells().count(),
        2,
        "{rule:?}: the skin and the hollow"
    );
    let measured = view
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a solid measures");
    assert!(
        (measured.volume() - by_hand).abs() <= measured.volume_error(),
        "{rule:?}: measured {} ± {} against {by_hand} by hand",
        measured.volume(),
        measured.volume_error()
    );
}
