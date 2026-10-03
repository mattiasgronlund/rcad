//! A block turned off the axes and a ball poking into it at one corner,
//! intersected with the block first and then with the ball first.
//!
//! The answer is a piece of the ball cut by three of the block's faces, and
//! every edge where a face crosses the ball is an arc of a circle. Which of the
//! two arcs between its ends an edge is, is written down as a half-space
//! through the chord. That half-space used to be worked out from whichever end
//! the first face to reach the edge walked away from — the block's face in one
//! order and the ball's in the other — and the two ends give the same plane
//! rounded two different ways. So the two orders handed back the same solid
//! with pickers that differed in their last bits, and the fuse fuzzer's
//! both-ways check caught it on 84 files of one night (`docs/decisions.md`
//! §312).

#![forbid(unsafe_code)]

#[path = "fuzzing/comparison.rs"]
mod comparison;

use rcad_geom::{Axis, EdgeRun, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

use comparison::{answer_of, same_answer};

#[test]
fn a_turned_block_and_a_ball_intersect_to_the_same_numbers_either_way_round() {
    let (model, block, ball) = a_turned_block_and_a_ball_at_its_corner();

    let block_first = intersected(&model, block, ball);
    let ball_first = intersected(&model, ball, block);

    assert_eq!(
        same_answer(
            &answer_of(&block_first.0, &block_first.1),
            &answer_of(&ball_first.0, &ball_first.1),
            Rule::Intersection,
            "putting the ball first"
        ),
        Ok(())
    );
}

/// What makes the test above able to fail: the answer has arcs, each picked
/// out by a half-space, and it is a sound solid.
#[test]
fn the_intersection_is_a_piece_of_the_ball_cut_by_three_faces_along_arcs() {
    let (model, block, ball) = a_turned_block_and_a_ball_at_its_corner();
    let (model, solids) = intersected(&model, block, ball);

    let [solid] = solids.as_slice() else {
        panic!("one piece of the ball, and not {solids:?}");
    };
    let faces: Vec<_> = model
        .solid(*solid)
        .expect("the fuse's own key")
        .faces()
        .collect();
    let flat = faces
        .iter()
        .filter(|face| face.surface().as_plane().is_some())
        .count();
    assert!(
        flat == 3 && faces.len() > flat,
        "three of the block's faces and the ball's skin: {flat} flat of {}",
        faces.len()
    );
    let arcs = faces
        .iter()
        .flat_map(|face| face.wires())
        .flat_map(|wire| wire.coedges())
        .filter(|coedge| {
            matches!(
                coedge.edge().run(),
                EdgeRun::PartOfWhereItCrosses { picked_by, .. } if picked_by.len() == 1
            )
        })
        .count();
    assert!(
        arcs >= 6,
        "each of the three faces crosses the ball in arcs, each walked by two faces: {arcs}"
    );
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the intersection is a sound solid");
}

/// A cube of side 2 turned one radian about `(1, 1, 1)`, and a ball of radius 1
/// whose centre is `(0.7, 0.65, 0.6)` in the cube's own frame: close enough to
/// three of its faces to cross each of them in a circle, and more than 1 from
/// the corner those three share, so the corner is not inside it.
///
/// Turned about `(1, 2, 3)` instead, the same pair used to be refused
/// `CannotMeshAFace`: a cut up the ball's cap to its pole met the cap's rim a
/// second time. It meshes now, and
/// `a_cap_whose_rim_bends_back_past_its_own_meridian.rs` checks it against the
/// pair upright.
fn a_turned_block_and_a_ball_at_its_corner() -> (Model, SolidKey, SolidKey) {
    let turn = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(1.0, 1.0, 1.0)).expect("an axis"),
        1.0,
    )
    .expect("a finite turn");
    let mut model = Model::new(Scale::millimetres(1.0).expect("1 mm is a working size"));
    let block = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    model
        .place_solid(block, &turn)
        .expect("this model made the key");
    let centre = turn.apply_to_point(Point::new(0.7, 0.65, 0.6));
    let ball = model
        .add_ball(
            1.0,
            &Isometry::translation(centre - Point::origin()).expect("a finite placement"),
        )
        .expect("a ball");
    (model, block, ball)
}

fn intersected(model: &Model, argument: SolidKey, tool: SolidKey) -> (Model, Vec<SolidKey>) {
    let mut model = model.clone();
    let fused = model
        .fuse(
            &[argument],
            &[tool],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a certified intersection of a turned block and a ball");
    let solids = fused.solids().to_vec();
    (model, solids)
}
