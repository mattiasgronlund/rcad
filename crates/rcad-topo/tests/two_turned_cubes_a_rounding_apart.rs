//! Two cubes of side two that a turn leaves a rounding apart (queue rows Q397
//! and Q676).
//!
//! The second cube is moved by `(2, 0.5, 0.25)`, so upright the two touch face
//! to face, and both are turned about the axis through `(1, 0, 0)` along
//! `(1, 2, 3)`. The first cube's face `x = 2` and the second's face `x = 0`
//! then face exactly opposite ways, and their offsets no longer quite cancel:
//! at 1.1 and 2.5 they leave a gap of `2^-51` and `2^-52`, and at 0.3 the two
//! faces overlap by `2^-52`.
//!
//! **Where it stops now.** Every fuse used to refuse at the first question:
//! which side of the second cube's face `x = 0` a corner of the first cube lies
//! on, asked of the corner's box, which the face runs straight through.
//! `Arrangement::passes_through` asks the three planes the corner is the meet
//! of instead, and they decide it (Q397). The fuse then refused one step on, in
//! `Arrangement::order_along`, ordering that corner and the point a rounding
//! past it where the other face crosses the corner's edge. Both are meets of
//! three planes that share two, and the planes now put them in order too
//! (Q676, `which_meet_comes_first_along_a_line`).
//!
//! Every fuse then stops in selection. The first cube's face `x = 2` comes out
//! of the arrangement whole, and so does the second's face `x = 0`; each has to
//! be placed against the other cube. The point picked inside it lies on its
//! own plane and so a rounding off the other cube's face, and the ray cast from
//! that point's box into the other cube cannot say which side of that face it
//! starts on: every candidate came back undecided, which a probe of
//! `settle_against_each` showed at 1.1. The answer is the inputs as they stand
//! — the union two solids of eight faces, `A \ B` is `A`, `B \ A` is `B`, and
//! the intersection nothing — so this is still a gap, and
//! [`every_corner_of_each_cube_is_in_front_of_the_others_touching_face_only_when_apart`]
//! is the demonstration that the answer exists: the cubes' own planes, asked
//! exactly, put them apart.
//!
//! **0.3 is a limit.** The faces overlap by `2^-52`, so the union and the
//! intersection keep a sliver whose edges are about `2.2e-16` long at a
//! coordinate of about `2.18`, a spread of `1e16` against the `1e11` a model
//! supports. It is pinned by name here and is to be refused whatever the
//! later stages learn.

use rcad_geom::{Axis, Isometry, Plane, Point, Scale, Vector};
use rcad_predicates::{Decision, Policy, ProvenanceLog, side_of_a_plane_where_three_planes_meet};
use rcad_topo::{Error, Model, Rule, SolidKey};

/// The turns at which the two touching faces leave a gap, and what their
/// offsets add up to there: `-2^-51` and `-2^-52`.
const A_ROUNDING_APART: [(f64, f64); 2] = [(1.1, -2.0 * f64::EPSILON), (2.5, -f64::EPSILON)];

/// The turn at which the two faces overlap instead, by `2^-52`.
const A_ROUNDING_INTO_EACH_OTHER: f64 = 0.3;

#[test]
fn the_touching_faces_lie_a_rounding_apart_at_1_1_and_2_5_and_overlap_at_0_3() {
    for (angle, offsets_add_to) in A_ROUNDING_APART {
        assert_eq!(
            what_the_touching_offsets_add_to(angle),
            offsets_add_to,
            "{angle}"
        );
    }
    assert_eq!(
        what_the_touching_offsets_add_to(A_ROUNDING_INTO_EACH_OTHER),
        f64::EPSILON
    );
}

/// The corner and the point a rounding past it on its edge are put in order
/// now, and every fuse gets past the arrangement. Before Q397 each refusal
/// named `side of plane` at a corner of one cube, before Q676 each named
/// `order along a crossing`, and now each names the search for a point in the
/// touching face that the other cube can place.
#[test]
fn a_corner_and_the_point_a_rounding_past_it_no_longer_stop_the_fuse() {
    for (angle, _) in A_ROUNDING_APART {
        for (what, refused_at) in every_fuse_refused(angle) {
            assert_eq!(
                refused_at, "a point in a piece that every solid could place",
                "{angle} {what} refused at {refused_at}"
            );
        }
    }
}

/// **At 1.1 and 2.5 every answer exists, so the refusal there is a gap.** A
/// cube is where all six of its planes have it behind them, so it lies in the
/// hull of its eight corners, each the meet of three of its planes. Asked of
/// those planes exactly
/// ([`side_of_a_plane_where_three_planes_meet`](rcad_predicates::side_of_a_plane_where_three_planes_meet)),
/// every corner of each cube is in front of the other cube's touching face, so
/// the two share no point: the union is the two as they stand, each less the
/// other is itself, and the intersection is nothing. At 0.3 the four corners
/// on each touching face are behind the other's, which is the sliver.
///
/// The fuse places the touching face by a point picked inside it, which lies
/// on its plane and so only `2^-51` from the other face, and the ray from it
/// could not say which side it starts on (the module doc). Here the question
/// is put to the planes instead, and they answer it.
#[test]
fn every_corner_of_each_cube_is_in_front_of_the_others_touching_face_only_when_apart() {
    let in_front = [Decision::Pos; 8];
    for (angle, _) in A_ROUNDING_APART {
        assert_eq!(
            sides_of_the_corners_against_the_other_touching_face(angle),
            [in_front, in_front],
            "{angle}"
        );
    }
    for corners in sides_of_the_corners_against_the_other_touching_face(A_ROUNDING_INTO_EACH_OTHER)
    {
        let behind = corners
            .iter()
            .filter(|side| **side == Decision::Neg)
            .count();
        assert_eq!(behind, 4, "{corners:?}");
    }
}

/// A limit, not a gap: every answer needs an edge `2.2e-16` long at `2.18`.
/// Since Q676 orders the corners along their edges it is refused in
/// selection, at the search for a point inside a piece; which piece was not
/// traced.
#[test]
fn two_cubes_overlapping_by_a_rounding_are_refused_for_the_sliver_between_them() {
    for (what, refused_at) in every_fuse_refused(A_ROUNDING_INTO_EACH_OTHER) {
        assert_eq!(
            refused_at, "a point inside a piece of a face",
            "{what} refused at {refused_at}"
        );
    }
}

/// Every fuse of the two cubes at `angle`, each refused `Undecidable`, with the
/// predicate the first record names.
fn every_fuse_refused(angle: f64) -> Vec<(&'static str, &'static str)> {
    let (mut model, a, b) = two_turned_cubes(angle);
    [
        ("a union b", a, b, Rule::Union),
        ("b union a", b, a, Rule::Union),
        ("a and b", a, b, Rule::Intersection),
        ("a less b", a, b, Rule::Difference),
        ("b less a", b, a, Rule::Difference),
    ]
    .into_iter()
    .map(|(what, left, right, rule)| {
        let outcome = model.fuse(
            &[left],
            &[right],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        let Err(Error::Undecidable { records }) = outcome else {
            panic!("{angle} {what} is refused Undecidable, and came back {outcome:?}");
        };
        let first = records
            .records()
            .first()
            .unwrap_or_else(|| panic!("{angle} {what} names what it could not decide"));
        (what, first.predicate())
    })
    .collect()
}

fn two_turned_cubes(angle: f64) -> (Model, SolidKey, SolidKey) {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let a = model
        .add_cube(2.0, &policy, &mut ProvenanceLog::new())
        .expect("a cube of side two");
    let b = model
        .add_cube(2.0, &policy, &mut ProvenanceLog::new())
        .expect("a cube of side two");
    let turn = Isometry::rotation(
        &Axis::new(Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 2.0, 3.0))
            .expect("a direction with length"),
        angle,
    )
    .expect("a rotation");
    model
        .place_solid(a, &turn)
        .expect("this model made the key");
    model
        .place_solid(
            b,
            &turn.compose(&Isometry::translation(Vector::new(2.0, 0.5, 0.25)).expect("a shift")),
        )
        .expect("this model made the key");
    (model, a, b)
}

/// The offsets of the one pair of faces, one from each cube, that face
/// exactly opposite ways and nearly touch, added up.
fn what_the_touching_offsets_add_to(angle: f64) -> f64 {
    let (model, a, b) = two_turned_cubes(angle);
    let mut sums = Vec::new();
    for mine in model.solid(a).expect("this model made the key").faces() {
        for theirs in model.solid(b).expect("this model made the key").faces() {
            let (mine, theirs) = (mine.plane(), theirs.plane());
            if mine.normal().components() == (-theirs.normal()).components()
                && (mine.offset() + theirs.offset()).abs() < 1e-12
            {
                sums.push(mine.offset() + theirs.offset());
            }
        }
    }
    let [only] = sums[..] else {
        panic!("{angle}: one pair of faces nearly touches, and {sums:?} do");
    };
    only
}

/// For each cube, which side of the other cube's touching face each of its
/// eight corners lies on, worked out from the planes the corner is the meet
/// of. [`Decision::Pos`] is in front of that face, outside the other cube.
///
/// A corner is any three of the cube's six planes that meet in a point: one
/// of each pair of opposite faces. Three that take both of one pair are
/// refused as meeting nowhere, which leaves eight.
fn sides_of_the_corners_against_the_other_touching_face(angle: f64) -> [[Decision; 8]; 2] {
    let (model, a, b) = two_turned_cubes(angle);
    let planes_of = |solid: SolidKey| -> Vec<Plane> {
        model
            .solid(solid)
            .expect("this model made the key")
            .faces()
            .map(|face| face.plane())
            .collect()
    };
    let (mine, theirs) = (planes_of(a), planes_of(b));
    let touching = |one: &[Plane], other: &[Plane]| -> Plane {
        let mut found = other.iter().filter(|plane| {
            one.iter().any(|mine| {
                mine.normal().components() == (-plane.normal()).components()
                    && (mine.offset() + plane.offset()).abs() < 1e-12
            })
        });
        let (Some(only), None) = (found.next(), found.next()) else {
            panic!("{angle}: one face of the other cube touches");
        };
        *only
    };
    [
        (&mine, touching(&mine, &theirs)),
        (&theirs, touching(&theirs, &mine)),
    ]
    .map(|(planes, face)| {
        let mut sides = Vec::new();
        for first in 0..planes.len() {
            for second in first + 1..planes.len() {
                for third in second + 1..planes.len() {
                    let three = [first, second, third].map(|at| planes[at].coefficients());
                    if let Ok(side) =
                        side_of_a_plane_where_three_planes_meet(three, face.coefficients())
                    {
                        sides.push(side);
                    }
                }
            }
        }
        sides
            .try_into()
            .unwrap_or_else(|sides| panic!("{angle}: a cube has eight corners, not {sides:?}"))
    })
}
