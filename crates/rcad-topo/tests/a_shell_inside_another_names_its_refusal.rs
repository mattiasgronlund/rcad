//! Hollows whose points are asked whether they are inside the shell round
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
//!
//! # A ball touching the block's top and bottom with its poles
//!
//! A ball of radius 2 centred at `(5, 5, 2)` touches the top with its north
//! pole and the bottom with its south pole, so every corner of the hollow is on
//! the block's skin and none places it. The ball's one edge, its seam, is half
//! a circle from pole to pole and does not close, so it has no far side to ask
//! either. That was refused `Undecidable`, with one record over the two poles
//! (row Q1126); until row Q883 the record named the block's corner at the
//! origin, which nothing asked about. Since row Q2101 the middle of each open
//! arc is asked as well, and the seam's middle, `(7, 5, 2)` on the ball's
//! equator, is inside the block: the block less the ball is one solid with the
//! ball as a hollow, measuring `400 − 32π/3`. The other five fuses of the pair
//! answer as they did.
//!
//! # A ball as wide as the square bar it is in
//!
//! The same ball in a bar 4 by 4 across, from `x = 3` to `x = 7`, touches the
//! bar's two sides as well, and the seam's middle is where it touches `x = 7`.
//! Every point of the ball's edge that is asked is on the bar's skin, and the
//! bar less the ball was refused `Undecidable`, with one record naming the two
//! poles and the seam's middle (row Q2115). Since row Q2147 a point of the
//! ball's face off its seam is asked after them, and it is inside the bar: one
//! solid with the ball as a hollow, measuring `160 − 32π/3`.

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

/// The block less a ball of radius 2 centred at `(5, 5, 2)`, which touches the
/// block's top with its north pole and its bottom with its south pole: one
/// solid, the block's six faces and the ball's one face as a hollow, holding
/// the block's 400 less the ball's `32π/3`, under a certified policy and a
/// tolerant one.
///
/// Both corners of the hollow are on the block's skin, and its one edge, the
/// seam, does not close, so neither a corner nor the far side of a closed edge
/// says which side of the skin the ball is. It was refused `Undecidable` at
/// *one shell inside another*, naming the two poles (queue row Q1126). The
/// seam's middle, `(7, 5, 2)`, is inside the block, and it is asked since queue
/// row Q2101.
///
/// **The hollow keeps its seam**, one edge walked from pole to pole and back.
/// History names no image of the input ball's seam or poles here, and it names
/// none for a ball wholly inside the block and touching nothing either: that
/// is a gap in history, not in this answer.
#[test]
fn a_block_less_a_ball_touching_its_top_and_bottom_keeps_the_ball_as_a_hollow() {
    the_ball_touching_its_top_and_bottom_is_a_hollow_of([0.0, 0.0, 0.0], [10.0, 10.0, 4.0]);
}

/// The same ball in a square bar as wide as it is, 4 by 4 and 10 long from
/// `x = 3` to `x = 7`: one solid, the bar's six faces and the ball's one face
/// as a hollow, holding the bar's 160 less the ball's `32π/3`, under a
/// certified policy and a tolerant one.
///
/// The ball touches the bar's top and bottom with its poles and its two sides
/// at `(3, 5, 2)` and `(7, 5, 2)`. The second is the middle of the ball's seam,
/// so every point of the ball's edge that is asked is on the bar's skin, and
/// it was refused `Undecidable` at *one shell inside another*, naming the two
/// poles and the seam's middle (queue row Q2115). A gap and not a limit
/// (§16.10): a point of the ball's face off its seam is inside the bar, and it
/// is asked since queue row Q2147.
#[test]
fn a_square_bar_less_a_ball_as_wide_as_it_is_keeps_the_ball_as_a_hollow() {
    the_ball_touching_its_top_and_bottom_is_a_hollow_of([3.0, 0.0, 0.0], [4.0, 10.0, 4.0]);
}

/// The block from `low`, `size` across, less a ball of radius 2 centred at
/// `(5, 5, 2)`, under a certified policy and a tolerant one: one solid, the
/// block's six faces and the ball's one face as a hollow walked down its seam
/// from pole to pole and back, holding the block's volume less the ball's
/// `32π/3`.
fn the_ball_touching_its_top_and_bottom_is_a_hollow_of(low: [f64; 3], size: [f64; 3]) {
    let poles = [[5.0, 5.0, 4.0], [5.0, 5.0, 0.0]];
    for (name, policy) in [
        ("certified", Policy::certified()),
        ("tolerant", Policy::tolerant(100.0)),
    ] {
        let mut model = a_model();
        let block = a_block(&mut model, low, size);
        let ball = a_ball(&mut model, 2.0, at(5.0, 5.0, 2.0));
        let answer = model
            .fuse(
                &[block],
                &[ball],
                Rule::Difference,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{name}: refused {why:?}"));
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{name}: the model does not validate: {why:?}"));
        let [solid] = answer.solids()[..] else {
            panic!("{name}: one solid, not {:?}", answer.solids());
        };
        let view = model.solid(solid).expect("this model made the key");
        let mut faces: Vec<usize> = view.shells().map(|shell| shell.face_count()).collect();
        faces.sort_unstable();
        assert_eq!(
            faces,
            [1, 6],
            "{name}: the ball's one face and the block's six"
        );
        let ball_face = view
            .faces()
            .find(|face| face.surface().as_plane().is_none())
            .expect("the hollow is the ball's face");
        let seam: Vec<_> = ball_face
            .wires()
            .flat_map(|wire| wire.coedges())
            .map(|coedge| {
                (
                    coedge.edge().key(),
                    coedge.start().point().coordinates(),
                    coedge.end().point().coordinates(),
                )
            })
            .collect();
        let [(down, top, bottom), (up, from, to)] = seam[..] else {
            panic!("{name}: the ball's face is walked down its seam and back, not {seam:?}");
        };
        assert!(
            down == up && top == to && bottom == from && [top, bottom] == poles,
            "{name}: the ball's face is walked down its seam and back, not {seam:?}"
        );
        let measured = view
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .expect("a solid measures");
        let by_hand = size.iter().product::<f64>() - 32.0 * PI / 3.0;
        assert!(
            (measured.volume() - by_hand).abs() <= measured.volume_error(),
            "{name}: measured {} ± {} against {by_hand} by hand",
            measured.volume(),
            measured.volume_error()
        );
    }
}

/// The other five fuses of that block and ball, which leave no hollow and so
/// ask nothing of the poles, answer: the block less the ball's union is the
/// block, their intersection is the ball, and the ball less the block is
/// nothing.
#[test]
fn the_other_fuses_of_a_block_and_a_ball_touching_its_top_and_bottom_answer() {
    let ball = 32.0 * PI / 3.0;
    for (rule, block_first, by_hand) in [
        (Rule::Union, true, Some(400.0)),
        (Rule::Union, false, Some(400.0)),
        (Rule::Intersection, true, Some(ball)),
        (Rule::Intersection, false, Some(ball)),
        (Rule::Difference, false, None),
    ] {
        let mut model = a_model();
        let (block, round) = a_block_and_a_ball_touching_its_top_and_bottom(&mut model);
        let (one, other) = if block_first {
            (block, round)
        } else {
            (round, block)
        };
        let policy = Policy::certified();
        let answer = model
            .fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{rule:?}, block first {block_first}: {why:?}"));
        let Some(by_hand) = by_hand else {
            assert!(
                answer.solids().is_empty(),
                "the ball less the block is nothing, not {:?}",
                answer.solids()
            );
            continue;
        };
        let [solid] = answer.solids()[..] else {
            panic!("{rule:?} is one solid, not {:?}", answer.solids());
        };
        let measured = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .expect("a solid measures");
        assert!(
            (measured.volume() - by_hand).abs() <= measured.volume_error(),
            "{rule:?}, block first {block_first}: measured {} ± {} against {by_hand}",
            measured.volume(),
            measured.volume_error()
        );
    }
}

fn a_block_and_a_ball_touching_its_top_and_bottom(model: &mut Model) -> (SolidKey, SolidKey) {
    let block = a_block(model, [0.0, 0.0, 0.0], [10.0, 10.0, 4.0]);
    let ball = a_ball(model, 2.0, at(5.0, 5.0, 2.0));
    (block, ball)
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
