//! A unit ball at the origin fused with two blocks that overlap each other:
//! the slab `[0.9, 5] × [−5, 5] × [−5, 5]`, which takes a cap off the ball, and
//! the link `[0, 3] × [2, 3] × [1.5, 2]`, which never reaches the ball
//! (`docs/decisions.md` §400.5).
//!
//! The ball meets the slab and the slab overlaps the link, so all three are cut
//! against each other, and the link's plane `x = 0` cuts the ball where nothing
//! else meets it: a whole circle through both poles, cut there into two arcs
//! that are pieces of no edge of the ball. Both pieces of the ball either side
//! of it are kept by a union, so each pole has those two arcs along it and
//! nothing else. The union also joins pieces of the slab's face `x = 0.9` that
//! the link's planes cut, and joining folds every corner with two edges along
//! it. Both poles were folded, the two arcs became one edge with no ends, and
//! the union panicked in `fuse/assemble.rs`: *a run of edges that became one
//! edge has two ends left*. Now a closed run like that keeps every corner, as
//! the answer does when nothing else in the fuse is joined, and the union is
//! one solid of the volume below. The other rules joined nothing and answered
//! before as well.
//!
//! Every answer has a curved face, so each is measured against its own bound.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

const BALL: f64 = 4.0 * PI / 3.0;
const SLAB: f64 = 4.1 * 10.0 * 10.0;
const LINK: f64 = 3.0 * 1.0 * 0.5;
const THE_BLOCKS_SHARE: f64 = 2.1 * 1.0 * 0.5;

/// What the ball and the slab share: the cap of the unit ball past
/// `x = 0.9`, a tenth deep, `πh²(3r − h)/3`.
fn the_ball_and_the_slab_share() -> f64 {
    let deep = 0.1;
    PI * deep * deep * (3.0 - deep) / 3.0
}

/// The draw that panicked, both ways round.
#[test]
fn the_ball_united_with_both_blocks_is_one_solid_holding_all_three() {
    let truth = BALL + SLAB + LINK - THE_BLOCKS_SHARE - the_ball_and_the_slab_share();
    for ball_first in [true, false] {
        let what = format!("union, ball first {ball_first}");
        let (model, answer) = fused(Rule::Union, ball_first);
        assert_one_solid_of(&model, &answer, truth, &what);
    }
}

/// The intersection and both differences of the same three, both ways round
/// where the rule allows it.
#[test]
fn the_other_rules_answer_the_same_three() {
    let cap = the_ball_and_the_slab_share();
    for (rule, ball_first, truth, what) in [
        (Rule::Intersection, true, cap, "intersection, ball first"),
        (Rule::Intersection, false, cap, "intersection, blocks first"),
        (
            Rule::Difference,
            true,
            BALL - cap,
            "the ball less the blocks",
        ),
        (
            Rule::Difference,
            false,
            SLAB + LINK - THE_BLOCKS_SHARE - cap,
            "the blocks less the ball",
        ),
    ] {
        let (model, answer) = fused(rule, ball_first);
        assert_one_solid_of(&model, &answer, truth, what);
    }
}

/// The union keeps the ball in two faces, either side of the circle `x = 0`,
/// and fused again with a cube far from it is refused on one of them: the
/// arrangement asks where the face meets its neighbour across that circle,
/// and the neighbour is the same ball. That is the refusal of every solid with
/// two faces on one ball, and queue row Q584 holds it. When Q584 answers it,
/// this is two solids, the cube and the union.
#[test]
fn the_union_fused_again_with_a_far_cube_is_refused_on_the_balls_two_faces() {
    let (mut model, first) = fused(Rule::Union, true);
    let far = model
        .add_cube(1.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    model
        .place_solid(far, &shifted([20.0, 20.0, 20.0]))
        .expect("this model made the key");
    let refused = model.fuse(
        &first,
        &[far],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    let Err(Error::AFaceMeetsItsNeighbourOnItsOwnSurface { face }) = refused else {
        panic!("refused on a face of the ball, got {refused:?}");
    };
    let surface = model
        .face(face)
        .expect("the refusal names a face of the model")
        .surface();
    assert!(
        matches!(surface, Surface::Sphere(_)),
        "refused on a face of the ball, got one on {surface:?}"
    );
}

/// The ball, the slab and the link, fused by `rule` with the ball as the
/// argument or as the tool.
fn fused(rule: Rule, ball_first: bool) -> (Model, Vec<SolidKey>) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let ball = model
        .add_ball(1.0, &shifted([0.0, 0.0, 0.0]))
        .expect("a ball of radius 1");
    let slab = block(&mut model, [0.9, -5.0, -5.0], [5.0, 5.0, 5.0]);
    let link = block(&mut model, [0.0, 2.0, 1.5], [3.0, 3.0, 2.0]);
    let (arguments, tools) = if ball_first {
        (vec![ball], vec![slab, link])
    } else {
        (vec![slab, link], vec![ball])
    };
    let answer = model
        .fuse(
            &arguments,
            &tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{rule:?}, ball first {ball_first}: refused {why:?}"))
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{rule:?}, ball first {ball_first}: {why:?}"));
    (model, answer)
}

/// A block from `low` to `high`, a rectangle extruded up `z`.
fn block(model: &mut Model, low: [f64; 3], high: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [low[0], low[1]],
        [high[0], low[1]],
        [high[0], high[1]],
        [low[0], high[1]],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(&rectangle, high[2] - low[2], &shifted([0.0, 0.0, low[2]]))
        .expect("a block")
}

fn shifted([x, y, z]: [f64; 3]) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}

fn assert_one_solid_of(model: &Model, answer: &[SolidKey], truth: f64, what: &str) {
    let [solid] = answer[..] else {
        panic!("{what}: one solid, got {answer:?}");
    };
    measured_against(model, solid, &[truth], what);
}

/// Which of `truths` the solid measures as, inside the bound its measurement
/// publishes for itself. Anything else fails.
fn measured_against(model: &Model, solid: SolidKey, truths: &[f64], what: &str) -> f64 {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the solid measures: {why:?}"));
    *truths
        .iter()
        .find(|&&truth| (measured.volume() - truth).abs() <= measured.volume_error())
        .unwrap_or_else(|| {
            panic!(
                "{what}: {} measured, {} off, against {truths:?}",
                measured.volume(),
                measured.volume_error()
            )
        })
}
