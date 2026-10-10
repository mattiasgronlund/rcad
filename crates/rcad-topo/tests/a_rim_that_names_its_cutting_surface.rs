//! A ball cut in two by the plane of a block it touches at one point, and the
//! both-orders comparison writing the rim between the two halves down.
//!
//! # What the fuzzer found
//!
//! 24 crash files over the nights of 2026-09-26 and 2026-09-27, 13 of them
//! different, all panicked in `tests/fuzzing/comparison.rs`'s `Run::of`:
//! *"the fuse does not write a rim that names its cutting surface"*. Every one
//! is the same shape: a block and a ball of radius 1 standing clear of each
//! other, fused under `Union`. The fuse put every face plane of the block to
//! the ball's face whenever the two solids' boxes touched, whether or not the
//! solids met (until §400), one of those planes crossed the ball, and the
//! answer carried the ball's face cut in two with a closed rim between the
//! halves. Both halves sit on the one
//! ball, so the pair either side of the rim says nothing about where it runs,
//! and the fuse names **the block's plane that crossed the ball** instead:
//! `EdgeRun::RightRoundWhereItCrosses { and }` (§217.1).
//!
//! That is the right answer, and the fault was the comparison's. Its arm was
//! written at `a0e2bd19` saying the fuse never writes this run, and the fuse
//! started writing it at `f7bce221` 35 minutes later, the same evening. So the
//! fuzz target's harness panicked on a sound answer before it could compare it
//! with anything.
//!
//! # The draw
//!
//! The fuzzer's shape, with the ball's centre moved from about `3.05e-5` off
//! the block's `y = 0` plane to half a unit off it, **on the block's side**.
//! That plane crosses the ball in a circle of radius `√3/2`, **parallel to the
//! ball's seam**, which lies in the plane through its centre square to `y`.
//! The ball touches the block's face `x = 0` at the one point `(0, −0.5, 1)`,
//! and its planes `z = 0` and `z = 2` at one point each outside the block.
//!
//! **A ball clear of the block is no longer cut at all** (`docs/decisions.md`
//! §400), pinned by
//! `a_ball_resting_on_a_blocks_plane_beside_the_block_unites_whole` in
//! `a_ball_clear_of_a_block_comes_back_whole.rs`. The fuzzer's balls stood
//! clear of their blocks, and so did this draw until §400, half a unit off
//! `y = 0` on the other side: the fuse used to cut two solids against each
//! other whenever their whole-solid boxes touched, and now cuts them only
//! where it cannot prove they do not meet. A ball that touches the block
//! meets it, so the block's planes still cut it, and the rim is still
//! written. That is the one kind of draw found where two solids meet and a
//! rim between two faces of one ball survives into an answer; §400 says
//! which others were tried and where each stops.

#[path = "fuzzing/comparison.rs"]
mod comparison;

use rcad_geom::{EdgeRun, Isometry, Plane, Point, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

use comparison::{answer_of, same_answer};

/// The rim between the two halves of the ball names the block's plane `y = 0`,
/// which is the only surface of the draw it lies on besides the ball.
#[test]
fn a_ball_touching_a_block_cut_in_two_by_the_blocks_plane_names_that_plane() {
    let (mut model, block, ball) = a_block_and_a_ball_touching_it();
    let fused = model
        .fuse(
            &[block],
            &[ball],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a block and a ball that touch at one point unite");

    let rims: Vec<Surface> = every_run_of(&model, fused.solids())
        .into_iter()
        .filter_map(|run| {
            let EdgeRun::RightRoundWhereItCrosses { and } = run else {
                return None;
            };
            Some(and)
        })
        .collect();
    let the_blocks_plane = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
            .expect("the y axis is a unit normal"),
    );
    assert_eq!(
        rims,
        [the_blocks_plane, the_blocks_plane],
        "the ball comes back in two halves with one rim between them, walked once by each half, \
         and the rim has to say it runs where the block's plane y = 0 crosses the ball"
    );
}

/// The regression: the union asked both ways round is written down and found
/// to be the same answer, rim and all.
///
/// Before the comparison had a shape for the rim, writing either answer down
/// panicked, so neither order was held to the other.
#[test]
fn the_union_is_the_same_answer_both_ways_round_with_the_rim_written_down() {
    let (model, block, ball) = a_block_and_a_ball_touching_it();
    let answered = |arguments: SolidKey, tools: SolidKey| {
        let mut model = model.clone();
        let fused = model
            .fuse(
                &[arguments],
                &[tools],
                Rule::Union,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a block and a ball that touch at one point unite");
        answer_of(&model, fused.solids())
    };

    assert_eq!(
        same_answer(
            &answered(block, ball),
            &answered(ball, block),
            Rule::Union,
            "swapping the block and the ball"
        ),
        Ok(())
    );
}

/// A block from `(-2, -2, 0)` to `(0, 0, 2)`, and a ball of radius 1 centred
/// at `(1, -0.5, 1)`, touching the block's face `x = 0` at `(0, -0.5, 1)`. The
/// block's plane `y = 0` crosses the ball outside the block.
fn a_block_and_a_ball_touching_it() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(rcad_geom::Scale::millimetres(100.0).expect("a working size"));
    let block = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side 2");
    model
        .place_solid(
            block,
            &Isometry::translation(Vector::new(-2.0, -2.0, 0.0)).expect("a finite placement"),
        )
        .expect("this model made the key");
    let ball = model
        .add_ball(
            1.0,
            &Isometry::translation(Vector::new(1.0, -0.5, 1.0)).expect("a finite placement"),
        )
        .expect("a ball of radius 1");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a block and a ball touching it are a sound model");
    (model, block, ball)
}

/// Every edge run of every face of the answer, once per coedge.
fn every_run_of(model: &Model, solids: &[SolidKey]) -> Vec<EdgeRun> {
    solids
        .iter()
        .flat_map(|&solid| model.solid(solid).expect("this model made the key").faces())
        .flat_map(|face| face.wires().flat_map(|wire| wire.coedges()))
        .map(|coedge| coedge.edge().run())
        .collect()
}
