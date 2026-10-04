//! A ball standing clear of a block comes back from a fuse as the ball it was:
//! one face, not cut anywhere (`docs/decisions.md` §400).
//!
//! The fuse cut two solids against each other whenever their whole-solid boxes
//! touched, and every plane of the one cut every face of the other. A block's
//! plane that crosses a ball clear of the block cut the ball in two, and
//! nothing folds two pieces of a ball back together, so the answer was a ball
//! of two faces with a rim between them that marks nothing. One ulp further
//! out the boxes stopped touching and the same ball came back whole. Four balls
//! by the corner and edges of a bigger block were refused outright, in all
//! twelve of the fuses below, and the cut answer could not be fused again.
//!
//! Two solids are now cut against each other unless it is proved that no face
//! of the one shares a point with any face of the other. Each draw here is
//! clear of its block by at least six hundredths, so the proof goes through,
//! and each answer is the block and the ball apart: the block's volume plus
//! `4π/3`, with the ball in one face.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

const BALL: f64 = 4.0 * PI / 3.0;

/// The draw the row was found on: a block from `(-2, -2, 0)` to `(0, 0, 2)`
/// and a ball of radius 1 at `(1, 0.5, 1)`. The ball rests on the plane
/// `x = 0` at `(0, 0.5, 1)`, half a unit beside the block's face there, and
/// the block's plane `y = 0` crosses it in a circle the block never reaches.
#[test]
fn a_ball_resting_on_a_blocks_plane_beside_the_block_unites_whole() {
    for ball_first in [false, true] {
        let (mut model, block, ball) = a_block_and_a_ball(2.0, [-2.0, -2.0, 0.0], [1.0, 0.5, 1.0]);
        let answer = united(&mut model, block, ball, ball_first);
        assert_the_block_and_the_ball_apart(
            &model,
            &answer,
            8.0,
            &format!("ball first {ball_first}"),
        );
    }
}

/// The same draw one ulp further out, where the two whole-solid boxes stop
/// touching. It came back whole before the proof as well, and the two have to
/// agree.
#[test]
fn the_same_ball_one_ulp_further_out_unites_whole_too() {
    let out = f64::from_bits(1.0_f64.to_bits() + 1);
    assert_eq!(out, 1.000_000_000_000_000_2);
    let (mut model, block, ball) = a_block_and_a_ball(2.0, [-2.0, -2.0, 0.0], [out, 0.5, 1.0]);
    let answer = united(&mut model, block, ball, false);
    assert_the_block_and_the_ball_apart(&model, &answer, 8.0, "one ulp out");
}

/// A block and a ball whose boxes do not touch, with a big ball whose box
/// reaches both and which touches neither. A chain of boxes used to put all
/// three in one group, and the block's plane `y = 0` cut the small ball.
#[test]
fn a_ball_linked_to_a_block_only_through_a_third_balls_box_unites_whole() {
    let (mut model, block, ball) = a_block_and_a_ball(2.0, [-4.0, -2.0, 0.0], [0.0, 0.5, 10.0]);
    let big = model
        .add_ball(4.0, &shifted([2.0, 4.0, 6.0]))
        .expect("a ball of radius 4");
    let fused = model
        .fuse(
            &[block],
            &[ball, big],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("three solids that touch nowhere unite: {why:?}"));
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the answer is a sound model");
    let mut found: Vec<f64> = fused
        .solids()
        .iter()
        .map(|&solid| what_this_solid_is(&model, solid, "linked"))
        .collect();
    found.sort_by(f64::total_cmp);
    assert_eq!(
        found,
        [BALL, 8.0, BALL * 64.0],
        "the block, the small ball and the big ball apart"
    );
}

/// Q316's four balls by the corner and edges of a block of side 4 from
/// `(-2, -4, -1)`, each clear of it, under union both ways round and as the
/// ball less the block: twelve fuses, every one of which was refused.
#[test]
fn four_balls_by_a_bigger_blocks_corner_and_edges_unite_and_lose_nothing() {
    for at in [
        [2.75, 0.75, 1.0],
        [2.9, 0.6, 1.0],
        [2.75, -2.0, 3.75],
        [2.75, 0.75, 3.75],
    ] {
        for ball_first in [false, true] {
            let (mut model, block, ball) = a_block_and_a_ball(4.0, [-2.0, -4.0, -1.0], at);
            let answer = united(&mut model, block, ball, ball_first);
            assert_the_block_and_the_ball_apart(
                &model,
                &answer,
                64.0,
                &format!("ball at {at:?}, ball first {ball_first}"),
            );
        }
        let (mut model, block, ball) = a_block_and_a_ball(4.0, [-2.0, -4.0, -1.0], at);
        let what = format!("ball at {at:?} less the block");
        let fused = model
            .fuse(
                &[ball],
                &[block],
                Rule::Difference,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what}: refused {why:?}"));
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        let [left] = fused.solids()[..] else {
            panic!(
                "{what}: the ball and nothing else, got {:?}",
                fused.solids()
            );
        };
        assert_eq!(what_this_solid_is(&model, left, &what), BALL, "{what}");
    }
}

/// The answer to the first draw, fused again with a cube far from both, is
/// three solids. With the ball cut in two the second fuse was refused
/// `AFaceMeetsItsNeighbourOnItsOwnSurface`.
#[test]
fn the_answer_fused_again_with_a_far_cube_answers() {
    let (mut model, block, ball) = a_block_and_a_ball(2.0, [-2.0, -2.0, 0.0], [1.0, 0.5, 1.0]);
    let first = united(&mut model, block, ball, false);
    let far = model
        .add_cube(1.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    model
        .place_solid(far, &shifted([10.0, 10.0, 10.0]))
        .expect("this model made the key");
    let fused = model
        .fuse(
            &first,
            &[far],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("the answer and a far cube unite: {why:?}"));
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the answer is a sound model");
    let mut found: Vec<f64> = fused
        .solids()
        .iter()
        .map(|&solid| what_this_solid_is(&model, solid, "fused again"))
        .collect();
    found.sort_by(f64::total_cmp);
    assert_eq!(found, [1.0, BALL, 8.0], "the cube, the ball and the block");
}

/// A cube of `side` with its low corner at `corner`, and a ball of radius 1
/// centred at `at`.
fn a_block_and_a_ball(side: f64, corner: [f64; 3], at: [f64; 3]) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let block = model
        .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    model
        .place_solid(block, &shifted(corner))
        .expect("this model made the key");
    let ball = model
        .add_ball(1.0, &shifted(at))
        .expect("a ball of radius 1");
    (model, block, ball)
}

fn shifted([x, y, z]: [f64; 3]) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}

fn united(model: &mut Model, block: SolidKey, ball: SolidKey, ball_first: bool) -> Vec<SolidKey> {
    let (arguments, tools) = if ball_first {
        (ball, block)
    } else {
        (block, ball)
    };
    let fused = model
        .fuse(
            &[arguments],
            &[tools],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("a block and a ball clear of it unite: {why:?}"));
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the answer is a sound model");
    fused.solids().to_vec()
}

fn assert_the_block_and_the_ball_apart(model: &Model, answer: &[SolidKey], block: f64, what: &str) {
    let mut found: Vec<f64> = answer
        .iter()
        .map(|&solid| what_this_solid_is(model, solid, what))
        .collect();
    found.sort_by(f64::total_cmp);
    assert_eq!(found, [BALL, block], "{what}: the block and the ball apart");
}

/// The volume of a solid that is a block on whole numbers, measured exactly,
/// or a whole ball in one face, `4πr³/3` once its measured volume is inside the
/// bound it publishes for itself. Anything else fails.
fn what_this_solid_is(model: &Model, solid: SolidKey, what: &str) -> f64 {
    let view = model.solid(solid).expect("this model made the key");
    let surfaces: Vec<Surface> = view.faces().map(|face| face.surface()).collect();
    let measured = view
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the solid measures: {why:?}"));
    if surfaces.iter().all(|surface| surface.as_plane().is_some()) {
        assert_eq!(
            measured.volume_error(),
            0.0,
            "{what}: a block on whole numbers measures exactly"
        );
        return measured.volume();
    }
    let [Surface::Sphere(ball)] = surfaces[..] else {
        panic!(
            "{what}: a solid that is neither a whole ball in one face nor flat: {} faces, \
             {surfaces:?}",
            surfaces.len()
        );
    };
    let truth = 4.0 * PI * ball.radius().powi(3) / 3.0;
    assert!(
        (measured.volume() - truth).abs() <= measured.volume_error(),
        "{what}: {} measured, {} off, against {truth}",
        measured.volume(),
        measured.volume_error()
    );
    truth
}
