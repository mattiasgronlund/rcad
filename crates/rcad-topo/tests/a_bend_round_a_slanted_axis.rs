//! A bend of pipe round an axis that is not along a coordinate axis
//! (`docs/decisions.md` §243.7, §328).
//!
//! Ring four, tube one, round the axis `(0, 1, 1)` through the origin. The
//! wall runs from the corner `(3, 0, 0)` to the corner `(1, 2, −2)`, both on
//! the inner equator, a little over seventy degrees round the ring: those are
//! the only points of the inner equator whose coordinates are written exactly
//! in binary short of half a turn, and a corner has to be certified on its
//! surfaces. Each cap is on a plane through the axis: the near one faces
//! `(0, −1, 1)` and the far one `(−4, 1, −1)`.
//!
//! Both caps' normals are worked out square to the axis, and neither comes out
//! of the rounding exactly square to it as far as an interval can tell: the dot
//! of each with the stored axis straddles zero. Asked exactly, both are zero,
//! because each normal's two slanted components round the same way the axis's
//! do. The wall used to be refused for that straddle.

use rcad_geom::{
    Axis, Cylinder, EdgeRun, HalfSpace, Orientation, Picker, Plane, Point, Scale, Surface, Torus,
    Vector,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, DraftStep, FaceKey, Model, OnAFace, Result, SolidBuilder, SolidKey};
use smallvec::smallvec;

/// The wall holds the stretch of ring between its caps, and every point asked
/// is exactly on the ring.
///
/// Those points are few. Every point of the ring with coordinates a sixteenth
/// or coarser is on the inner or the outer equator, at `(±3, 0, 0)`,
/// `(±5, 0, 0)`, `(1, ±2, ∓2)` or `(−1, ±2, ∓2)`, so the oracle is the angle
/// each sits at round the axis: zero and seventy degrees are on the wall's
/// circles, the rest are off the wall.
#[test]
fn the_wall_holds_the_stretch_of_ring_between_its_caps() {
    let (model, solid, wall) = a_bend_round_y_plus_z();
    let bend = model.solid(solid).expect("this model made the key");
    let mut wrong = Vec::new();
    for (point, expected, why) in [
        ([3.0, 0.0, 0.0], OnAFace::OnItsBoundary, "the near corner"),
        (
            [5.0, 0.0, 0.0],
            OnAFace::OnItsBoundary,
            "across the near circle from its corner",
        ),
        ([1.0, 2.0, -2.0], OnAFace::OnItsBoundary, "the far corner"),
        (
            [-1.0, 2.0, -2.0],
            OnAFace::NotHeld,
            "on the inner equator, forty degrees past the far cap",
        ),
        (
            [1.0, -2.0, 2.0],
            OnAFace::NotHeld,
            "on the inner equator, seventy degrees behind the near cap",
        ),
        ([-3.0, 0.0, 0.0], OnAFace::NotHeld, "half a turn round"),
        ([-5.0, 0.0, 0.0], OnAFace::NotHeld, "half a turn round"),
        (
            [-1.0, -2.0, 2.0],
            OnAFace::NotHeld,
            "on the inner equator, a hundred and ten degrees behind the near cap",
        ),
    ] {
        let answer = bend.face_holds_the_point(
            wall,
            PredicatePoint::from(point),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        if !matches!(answer, Ok(found) if found == expected) {
            wrong.push(format!("{point:?}, {why}: {answer:?}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// A point in the tube half way between the caps is inside the bend, and the
/// same point mirrored across the near cap's plane is not.
///
/// `(3.25, 1.625, −1.625)` is on the ring's middle plane, `3.98` from the axis
/// and so inside the tube, and on the plane half way round from one cap to the
/// other. Mirroring across the near cap's plane `y = z` swaps `y` and `z`.
/// `(0, 3, −3)` is on the middle plane too, `4.24` from the axis, a quarter
/// turn round and past the far cap.
#[test]
fn a_point_in_the_tube_between_the_caps_is_inside_and_its_mirror_is_not() {
    let (model, solid, _) = a_bend_round_y_plus_z();
    let bend = model.solid(solid).expect("this model made the key");
    let mut wrong = Vec::new();
    for (point, expected, why) in [
        (
            Point::new(3.25, 1.625, -1.625),
            Containment::Inside,
            "in the tube half way between the caps",
        ),
        (
            Point::new(3.25, -1.625, 1.625),
            Containment::Outside,
            "the same, mirrored across the near cap",
        ),
        (
            Point::new(0.0, 3.0, -3.0),
            Containment::Outside,
            "in the tube a quarter turn round, past the far cap",
        ),
    ] {
        let found = bend.classify_point(point, &Policy::certified(), &mut ProvenanceLog::new());
        if found != expected {
            wrong.push(format!("{point:?}, {why}: {found:?}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// The bend, described face by face through [`SolidBuilder`] the way
/// `tests/rings/mod.rs` describes its bends about `+z`, and its wall.
fn a_bend_round_y_plus_z() -> (Model, SolidKey, FaceKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let solid = match described(&mut model) {
        Ok(solid) => solid,
        Err(refused) => panic!("the bend round (0, 1, 1) was refused: {refused:?}"),
    };
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a sound bend");
    let wall = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .find(|face| matches!(face.surface(), Surface::Torus(_)))
        .expect("a bend has a wall")
        .key();
    (model, solid, wall)
}

/// Two closed edges, one round each cap, and a seam along the inner equator
/// from the near corner to the far one: a bar's structure, as a bend about `+z`
/// has.
///
/// The seam's pickers say which of the two circles the ring's middle plane cuts
/// — the one inside the pipe of the ring's own radius — and which piece of it:
/// `x ≥ 0` holds the whole of the short way round from `(3, 0, 0)` to
/// `(1, 2, −2)` and not the long way.
fn described(model: &mut Model) -> Result<SolidKey> {
    let along = Vector::new(0.0, 1.0, 1.0);
    let axis = Axis::new(Point::origin(), along).expect("a direction with a length");
    let ring = Torus::new(axis, 4.0, 1.0).expect("a tube narrower than its ring");
    let through_the_axis = |facing: Vector| {
        Plane::from_point_and_normal(Point::origin(), facing).expect("a normal with a length")
    };
    let mut builder = SolidBuilder::new();

    let near_corner = builder.add_vertex(Point::new(3.0, 0.0, 0.0))?;
    let far_corner = builder.add_vertex(Point::new(1.0, 2.0, -2.0))?;
    let seam = builder.add_edge(
        near_corner,
        far_corner,
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(through_the_axis(along)),
            picked_by: smallvec![
                Picker::InsideAPipe(Cylinder::new(axis, 4.0).expect("the ring's own radius")),
                Picker::from(
                    HalfSpace::new(Vector::new(1.0, 0.0, 0.0), 0.0)
                        .expect("a normal with a length")
                ),
            ],
        },
    )?;
    let near_circle = builder.add_edge(
        near_corner,
        near_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;
    let far_circle = builder.add_edge(
        far_corner,
        far_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;

    let cap = |builder: &mut SolidBuilder, facing: Vector, edge, turn| -> Result<_> {
        let coedge = builder.add_coedge(edge, turn)?;
        let wire = builder.add_wire(&[DraftStep::Along(coedge)])?;
        builder.add_face(Surface::Plane(through_the_axis(facing)), &[wire])
    };
    let near_cap = cap(
        &mut builder,
        Vector::new(0.0, -1.0, 1.0),
        near_circle,
        Orientation::Reversed,
    )?;
    let far_cap = cap(
        &mut builder,
        Vector::new(-4.0, 1.0, -1.0),
        far_circle,
        Orientation::Forward,
    )?;

    let mut round_the_wall = Vec::with_capacity(4);
    for (edge, turn) in [
        (near_circle, Orientation::Forward),
        (seam, Orientation::Forward),
        (far_circle, Orientation::Reversed),
        (seam, Orientation::Reversed),
    ] {
        round_the_wall.push(DraftStep::Along(builder.add_coedge(edge, turn)?));
    }
    let wire = builder.add_wire(&round_the_wall)?;
    let wall = builder.add_face(Surface::Torus(ring), &[wire])?;

    builder.add_shell(&[
        (near_cap, Orientation::Forward),
        (far_cap, Orientation::Forward),
        (wall, Orientation::Reversed),
    ])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}
