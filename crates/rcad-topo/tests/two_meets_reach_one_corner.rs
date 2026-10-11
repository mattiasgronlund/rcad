//! Two places of one meet that both lie within a corner's room, and why no fuse
//! shows which of them `a_corner_is_this_place` folds (`docs/decisions.md` §288,
//! §346).
//!
//! `place_the_meets` folds a meet of three surfaces into a corner of the face
//! only when no *other* place of the same meet reaches the corner's box grown by
//! what writing the corner down cost (§253). That room is `16 × 4 × ε` times the
//! corner's size, floored at one, so `2⁻⁴⁶` at a corner no bigger than one.
//! Both shapes here put two places of one meet inside that room:
//!
//! - **An edge grazing a ball, with the block's corner on the ball.** §244.2's
//!   numbers: a ball of radius `1 + 2⁻⁵¹` at the origin and the edge
//!   `x = −1, y = 2⁻²⁵` meet at `z = ±2⁻⁵¹`, `2⁻⁵⁰` apart, and a block whose
//!   corner is one of those two is exactly on the ball.
//! - **A ball smaller than the room on a cube's corner.** A ball of radius `13s`
//!   at `(3s, 4s, 12s)` goes through the origin, and each edge of the cube
//!   there meets it a second time at `6s`, `8s` or `24s`. At `s = 2⁻⁵⁰` two of
//!   those are within `2⁻⁴⁶`, and at `2⁻⁵²` all three. At `2⁻⁴⁸` none is,
//!   and that fuse is the control.
//!
//! **Neither shows the branch yet, for two different reasons.**
//!
//! - The grazing block's two places fence off three pieces — the ball's skin
//!   inside the block and two slivers of the block's faces inside the ball —
//!   and **no point with `f64` coordinates is inside any of them**
//!   (`a_block_edge_grazing_a_ball.rs` shows it). Where the block reaches both
//!   places, `select` places the three by the sides they share (§365.3). With
//!   the block above the bottom place it stops on a lens of the ball beyond
//!   the block's floor instead. With the block below the top place the same
//!   lens beyond its roof is placed too (§35.6), and what stops the fuse is
//!   the answer: too wide a range under a certified policy, a curved face
//!   with no seam under a tolerant one. Where the block reaches one place, the
//!   ball's sliver shares no side with the block and is still refused. The
//!   test below says where each form stops.
//! - The small ball's piece inside the cube used to be refused the same way,
//!   because `select` gave a point on a curved face a box floored at one unit
//!   wide. That box is sized by the ball now, and the fuse gets past `select`.
//!   What stops it is the check of the answer, which cannot mesh the ball's
//!   faces, just as it cannot at `s = 2⁻⁴⁰` where `select` never refused.
//!
//! **The fold itself is pinned in `src/fuse/arrange.rs`'s `tests` module**, by
//! two tests that take these shapes' faces and call `a_corner_is_this_place`
//! directly (§288.5).

use rcad_geom::{Isometry, Scale, Vector};
use rcad_predicates::{Decision, Level, Point as PredicatePoint, Policy, ProvenanceLog, Record};
use rcad_topo::{Error, Fused, Invalid, Model, ProfileBuilder, Rule};

const INSIDE_A_PIECE: &str = "a point inside a piece of a face";

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// Where the corner sits, and which way from it the block runs.
#[derive(Clone, Copy, Debug)]
enum Corner {
    /// At `z = 2⁻⁵¹`, the block above it: the other place is past the corner.
    TopPlaceBlockAbove,
    /// At `z = 2⁻⁵¹`, the block below it: the other place is on the edge.
    TopPlaceBlockBelow,
    /// At `z = −2⁻⁵¹`, the block above it: the other place is on the edge.
    BottomPlaceBlockAbove,
}

const EVERY_CORNER: [Corner; 3] = [
    Corner::TopPlaceBlockAbove,
    Corner::TopPlaceBlockBelow,
    Corner::BottomPlaceBlockAbove,
];

/// Two corner forms refuse `a point inside a piece of a face`, on a piece of
/// the ball, and which piece depends on where the block is. The third gets
/// past `select` and its answer is refused.
///
/// - **With the block above the top place**, on the sliver between the two
///   places. It holds no point with `f64` coordinates (§346.3), and it is
///   outside the block, below its floor, touching the block only at the
///   corner. So no side of it is a side of a piece of the block's faces, and
///   the rule that places a piece by a side it shares (§365.3) cannot reach
///   it.
/// - **With the block above the bottom place**, reaching the top place too,
///   the three slivers are placed by that rule, and the refusal moves to the
///   ball's lens beyond `x = −1` below the block's floor. That lens runs
///   between `(−1, ±2⁻²⁵, −2⁻⁵¹)`, `2⁻²⁴` across and `2⁻²⁵` high, with both
///   its corners on the plane of the floor: the candidates blended from two
///   corners lie on that plane, not in the lens. It is outside the block and
///   touches it at one corner, so the rule does not reach it either. Until the
///   rule it refused on the sliver, as the first one does.
/// - **With the block below the top place**, reaching the bottom place too,
///   the slivers are placed the same way, and so is the same lens above the
///   roof, between `(−1, ±2⁻²⁵, 2⁻⁵¹)`: outside the block, by the piece of the
///   ball across a side that starts clear of every face of the block
///   (§35.6, Q1537). The lens is the ball beyond `x = −1` above `z = 2⁻⁵¹`, and
///   the block's roof is `z = 2⁻⁵¹`, so every point of the lens but its two
///   corners is above the block. What stops the fuse then is the answer: under
///   `Policy::certified()` its shortest edge is too short beside its
///   coordinates, `DynamicRangeTooWide` at a ratio of `2⁵²`, which is asked
///   before anything is meshed, and under `Policy::tolerant(25.0)` a curved
///   face of it goes right round with no seam. Why the same rule leaves the
///   lens below the floor of the form above unplaced was not traced.
#[test]
fn a_block_corner_where_its_edge_grazes_a_ball_refuses_on_the_balls_piece_or_its_own_answer() {
    let place = f64::powi(2.0, -51);
    let edge = f64::powi(2.0, -25);
    for corner in EVERY_CORNER {
        let refused_on = match corner {
            Corner::TopPlaceBlockAbove => Some([[-1.0, edge, -place], [-1.0, edge, place]]),
            Corner::TopPlaceBlockBelow => None,
            Corner::BottomPlaceBlockAbove => Some([[-1.0, -edge, -place], [-1.0, edge, -place]]),
        }
        .map(|corners| corners.map(PredicatePoint::Exact));
        for (policy, certified) in [(Policy::certified(), true), (Policy::tolerant(25.0), false)] {
            for rule in EVERY_RULE {
                let case = (corner, rule, certified);
                let mut model =
                    Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
                let mut log = ProvenanceLog::new();
                match (
                    refused_on,
                    fuse_a_block_whose_corner_is_on_a_ball(
                        &mut model, corner, rule, &policy, &mut log,
                    ),
                ) {
                    (_, Ok(fused)) => panic!(
                        "{case:?}: the fuse answered with {} solids, so every piece of the \
                         ball was placed. These shapes can now show which place \
                         `a_corner_is_this_place` folds (§288.3)",
                        fused.solids().len()
                    ),
                    (Some(corners), Err(Error::Undecidable { records })) => {
                        assert_the_one_refusal_is_the_piece(records.records(), &corners, case);
                    }
                    (None, Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })))
                        if certified =>
                    {
                        assert_eq!(ratio, f64::powi(2.0, 52), "{case:?}");
                    }
                    (None, Err(Error::ACurvedFaceGoesRightRoundWithNoSeam { .. }))
                        if !certified => {}
                    (_, Err(other)) => panic!("{case:?}: {other:?}"),
                }
            }
        }
    }
}

/// The small ball's piece inside the cube is placed now, and the answer's own
/// check is what refuses the fuse.
///
/// Before §346 every one of these refused in `select`, on
/// `a point inside a piece of a face` with an infinite enclosure, because the
/// box a point on the ball was given reached `5.7e-14` from its middle, and the
/// ball is `2.9e-15` to `4.6e-14` in radius.
#[test]
fn a_ball_smaller_than_the_room_on_a_cube_corner_is_placed_and_its_answer_cannot_be_meshed() {
    for exponent in [-48, -50, -52] {
        let s = f64::powi(2.0, exponent);
        for policy in [Policy::certified(), Policy::tolerant(25.0)] {
            for rule in EVERY_RULE {
                let case = (exponent, policy.strictness(), rule);
                let mut model =
                    Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
                let mut log = ProvenanceLog::new();
                match fuse_a_cube_and_a_ball_on_its_corner(&mut model, s, rule, &policy, &mut log) {
                    Err(
                        Error::CannotMeshAFace { .. }
                        | Error::AWireWillNotLieFlatOnItsFace { .. }
                        | Error::ACurvedFaceGoesRightRoundWithNoSeam { .. },
                    ) => (),
                    Err(Error::Undecidable { records }) => {
                        panic!("{case:?}: `select` refused the ball again: {records:#?}")
                    }
                    Ok(fused) => panic!(
                        "{case:?}: the fuse answered with {} solids. These shapes can now show \
                         which place `a_corner_is_this_place` folds (§288.3, §346)",
                        fused.solids().len()
                    ),
                    Err(other) => panic!("{case:?}: {other:?}"),
                }
                assert!(
                    log.records().is_empty(),
                    "{case:?}: nothing was asked of the policy: {:#?}",
                    log.records()
                );
            }
        }
    }
}

/// The fuse asked one thing and refused on it: a point inside the piece whose
/// corners are `corners`.
fn assert_the_one_refusal_is_the_piece(
    records: &[Record],
    corners: &[PredicatePoint],
    case: impl core::fmt::Debug,
) {
    assert_eq!(
        records.len(),
        1,
        "{case:?}: one record, the refusal, under either policy: {records:#?}"
    );
    let refused = &records[0];
    assert_eq!(
        refused.predicate(),
        INSIDE_A_PIECE,
        "{case:?}: {refused:#?}"
    );
    assert_eq!(refused.level(), Level::Interval, "{case:?}: {refused:#?}");
    assert!(
        matches!(refused.outcome(), Decision::Undecidable { enclosure } if enclosure == f64::INFINITY),
        "{case:?}: no point was found at all, so nothing could be measured: {refused:#?}"
    );
    assert_eq!(refused.inputs(), corners, "{case:?}");
}

fn fuse_a_block_whose_corner_is_on_a_ball(
    model: &mut Model,
    corner: Corner,
    rule: Rule,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> rcad_topo::Result<Fused> {
    let place = f64::powi(2.0, -51);
    let height = 4.0;
    let bottom = match corner {
        Corner::TopPlaceBlockAbove => place,
        Corner::TopPlaceBlockBelow => place - height,
        Corner::BottomPlaceBlockAbove => -place,
    };
    let ball = model
        .add_ball(1.0 + place, &Isometry::identity())
        .expect("a positive radius");
    let edge = f64::powi(2.0, -25);
    let square = ProfileBuilder::new(vec![
        [-4.0, edge],
        [-1.0, edge],
        [-1.0, edge + 3.0],
        [-4.0, edge + 3.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square");
    let block = model
        .add_extrusion(
            &square,
            height,
            &Isometry::translation(Vector::new(0.0, 0.0, bottom)).expect("a finite shift"),
        )
        .expect("a positive height");
    model.fuse(&[block], &[ball], rule, policy, log)
}

fn fuse_a_cube_and_a_ball_on_its_corner(
    model: &mut Model,
    s: f64,
    rule: Rule,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> rcad_topo::Result<Fused> {
    let cube = model
        .add_cube(1.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a unit cube");
    let ball = model
        .add_ball(
            13.0 * s,
            &Isometry::translation(Vector::new(3.0 * s, 4.0 * s, 12.0 * s))
                .expect("a finite shift"),
        )
        .expect("a positive radius");
    model.fuse(&[cube], &[ball], rule, policy, log)
}
