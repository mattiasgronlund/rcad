//! Two cubes of side two that a turn leaves a rounding apart (queue row Q397).
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
//! `Arrangement::passes_through` now asks the three planes the corner is the
//! meet of instead, and they decide it. The fuse then refuses one step on, in
//! `Arrangement::order_along`, ordering that corner and the point where the
//! other face crosses the corner's edge, two boxes a rounding apart along one
//! line. Both are meets of three planes that share two, so the order is the
//! same side-of-a-plane question times which way the edge runs through the
//! other face; that stage is not this row's.
//!
//! **0.3 is a limit.** The faces overlap by `2^-52`, so the union and the
//! intersection keep a sliver whose edges are about `2.2e-16` long at a
//! coordinate of about `2.18`, a spread of `1e16` against the `1e11` a model
//! supports. It is pinned by name here and is to be refused whatever the
//! later stages learn.

use rcad_geom::{Axis, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
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

/// The corner is decided now, and every fuse gets one step further before it
/// gives up. Before, each refusal named `side of plane` at a corner of one
/// cube; now each names `order along a crossing`.
#[test]
fn a_corner_a_rounding_short_of_the_other_face_no_longer_stops_the_fuse() {
    for (angle, _) in A_ROUNDING_APART {
        for (what, refused_at) in every_fuse_refused(angle) {
            assert_eq!(
                refused_at, "order along a crossing",
                "{angle} {what} refused at {refused_at}"
            );
        }
    }
}

/// A limit, not a gap: every answer needs an edge `2.2e-16` long at `2.18`.
#[test]
fn two_cubes_overlapping_by_a_rounding_are_refused_for_the_sliver_between_them() {
    for (what, refused_at) in every_fuse_refused(A_ROUNDING_INTO_EACH_OTHER) {
        assert_eq!(
            refused_at, "order along a crossing",
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
