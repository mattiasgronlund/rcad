//! Two places of one meet that both lie within a corner's room, and why no fuse
//! shows which of them `a_corner_is_this_place` folds (`docs/decisions.md` §288).
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
//!   and that fuse is the control: it refuses the same way.
//!
//! **Neither shows the branch.** Both places are on all three surfaces, so on
//! the ball the other two crossings run through both of them and fence off a
//! piece no wider than the room. A point on a curved face is a box `256 ε`
//! wide, four times the room, so no point fits in that piece, and every fuse
//! refuses on it by name with nothing else asked. It refuses the same with the
//! fold made and without it. §288.4 has the mutations.
//!
//! **The fold itself is pinned in `src/fuse/arrange.rs`'s `tests` module**, by
//! two tests that take these shapes' faces and call `a_corner_is_this_place`
//! directly (§288.5).
//!
//! **When `select` learns to place a point in a piece this small, these pins
//! move**, and these are the shapes that can then show the branch through a
//! fuse.

use rcad_geom::{Isometry, Scale, Vector};
use rcad_predicates::{Decision, Level, Point as PredicatePoint, Policy, ProvenanceLog, Record};
use rcad_topo::{Error, Fused, Model, ProfileBuilder, Rule};

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

#[test]
fn a_block_corner_where_its_edge_grazes_a_ball_refuses_on_the_piece_between_the_two_places() {
    let place = f64::powi(2.0, -51);
    let edge = f64::powi(2.0, -25);
    let the_two_places = [
        PredicatePoint::Exact([-1.0, edge, -place]),
        PredicatePoint::Exact([-1.0, edge, place]),
    ];
    for corner in EVERY_CORNER {
        for policy in [Policy::certified(), Policy::tolerant(25.0)] {
            for rule in EVERY_RULE {
                let records = what_it_refused_on(&policy, |model, log| {
                    fuse_a_block_whose_corner_is_on_a_ball(model, corner, rule, &policy, log)
                });
                assert_the_one_refusal_is_the_piece(&records, &the_two_places, (corner, rule));
            }
        }
    }
}

#[test]
fn a_ball_smaller_than_the_room_on_a_cube_corner_refuses_on_its_own_face() {
    for exponent in [-48, -50, -52] {
        let s = f64::powi(2.0, exponent);
        let the_three_second_places = [
            PredicatePoint::Exact([0.0, 8.0 * s, 0.0]),
            PredicatePoint::Exact([0.0, 0.0, 24.0 * s]),
            PredicatePoint::Exact([6.0 * s, 0.0, 0.0]),
        ];
        for policy in [Policy::certified(), Policy::tolerant(25.0)] {
            for rule in EVERY_RULE {
                let records = what_it_refused_on(&policy, |model, log| {
                    fuse_a_cube_and_a_ball_on_its_corner(model, s, rule, &policy, log)
                });
                assert_the_one_refusal_is_the_piece(
                    &records,
                    &the_three_second_places,
                    (exponent, rule),
                );
            }
        }
    }
}

/// The records a fuse that is expected to refuse hands back.
///
/// # Panics
///
/// If the fuse answers, or fails with anything but a refusal.
fn what_it_refused_on(
    policy: &Policy,
    fuse: impl FnOnce(&mut Model, &mut ProvenanceLog) -> rcad_topo::Result<Fused>,
) -> Vec<Record> {
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let mut log = ProvenanceLog::new();
    match fuse(&mut model, &mut log) {
        Err(Error::Undecidable { records }) => records.records().to_vec(),
        Ok(fused) => panic!(
            "{policy:?}: the fuse answered with {} solids, so `select` placed a point in the \
             piece between the two places. These shapes can now show which place \
             `a_corner_is_this_place` folds (§288.3)",
            fused.solids().len()
        ),
        Err(other) => panic!("{policy:?}: {other:?}"),
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
