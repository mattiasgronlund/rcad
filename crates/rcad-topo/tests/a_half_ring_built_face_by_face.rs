//! §250.7's half-ring, built face by face through [`SolidBuilder`], in all
//! three ways of writing its wall and all eight ways of walking it.
//! `docs/decisions.md` §250.7 and §250.10 are the record.
//!
//! The half-ring is the ring of radius four and tube one about `z`, cut by
//! `z = 0`, keeping `z ≥ 0`. Its flat face is `z = 0` facing `−z`, bounded by
//! the two circles `z = 0` cuts the ring in, of radius five and three, each one
//! closed edge. So the flat face is the annulus `3 ≤ r ≤ 5` at `z = 0`, with
//! `r` the distance from the `z` axis, and every answer below is read off `r`
//! against those two radii:
//!
//! | point | `r` | the flat face | the half-ring |
//! |---|---|---|---|
//! | `(4, 0, 0)` | 4 | held | on its skin |
//! | `(3.5, 0, 0)` | 3.5 | held | |
//! | `(0, 0, 0)` | 0 | not held, in the hole | outside |
//! | `(6, 0, 0)` | 6 | not held, beyond the rim | |
//! | `(5, 0, 0)` | 5 | on its boundary, the outer circle | |
//! | `(3, 0, 0)` | 3 | on its boundary, the inner circle | |
//!
//! Until §390 cut a torus face round the ring, `SolidBuilder::build` refused
//! all 24 descriptions (§250.7), so these answers were only ever asked of a
//! half-ring read back from bytes
//! (`a_half_rings_flat_face_read_back_from_bytes.rs`). Five of the 24 build
//! now. Three are the half-ring, one per way of writing the wall, and each
//! answers every question above right. The other two are not pinned:
//! [`is_every_loop_walked_backwards_round_a_seam`] says why.

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Deflection, EdgeRun, HalfSpace, Orientation, Picker, Plane, Point, Scale, Surface, Torus,
    Vector,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{
    Containment, DraftStep, Error, Invalid, Model, OnAFace, Result, SolidBuilder, SolidKey,
};
use smallvec::smallvec;

const RING: f64 = 4.0;
const TUBE: f64 = 1.0;

/// Which of the 24 descriptions build, and the name each of the others is
/// refused by. Every one that builds also passes `Model::validate`.
///
/// - **The half-ring's own walk**, outer circle `Forward` and hole `Reversed`
///   on the flat face and the wall used `Forward`, builds whichever way the
///   wall is written.
/// - **Both circles walked the same way round** on the flat face is refused
///   `AClosedEdgeIsWalkedAgainstItsSurface`, with the wall used either way: the
///   wall then keeps one side of `z = 0` at one circle and the other side at
///   the other, which no face of the ring does (§243.8).
/// - **Every loop walked backwards with the wall used `Reversed`** is refused
///   `FacesDoNotAgreeWhichWayTheyFace`.
/// - **Every loop walked backwards with the wall used `Forward`** is refused
///   `FacesDoNotAgreeWhichWayTheyFace` with no seam, and builds with one. The
///   second is not pinned here.
/// - **The half-ring's walk with the wall used `Reversed`** is refused
///   `VolumeIsNotPositive` with no seam, and `FacesDoNotAgreeWhichWayTheyFace`
///   with one.
#[test]
fn which_descriptions_build_and_what_the_others_are_refused() {
    for wall in every_wall() {
        for choice in every_choice()
            .filter(|&choice| !is_every_loop_walked_backwards_round_a_seam(wall, choice))
        {
            let mut model = a_model();
            let built = a_half_ring(&mut model, wall, choice);
            let walked_both_the_same_way =
                choice.outer_on_the_flat_face == choice.inner_on_the_flat_face;
            let walked_as_the_half_ring = choice.outer_on_the_flat_face == Orientation::Forward;
            let used_as_the_half_ring = choice.wall_in_the_shell == Orientation::Forward;
            let as_expected = if walked_both_the_same_way {
                matches!(
                    built,
                    Err(Error::Invalid(
                        Invalid::AClosedEdgeIsWalkedAgainstItsSurface
                    ))
                )
            } else if walked_as_the_half_ring && used_as_the_half_ring {
                built.is_ok()
                    && model
                        .validate(&Policy::certified(), &mut ProvenanceLog::new())
                        .is_ok()
            } else if walked_as_the_half_ring && wall == Wall::WithNoSeam {
                matches!(
                    built,
                    Err(Error::Invalid(Invalid::VolumeIsNotPositive { .. }))
                )
            } else {
                matches!(
                    built,
                    Err(Error::Invalid(Invalid::FacesDoNotAgreeWhichWayTheyFace))
                )
            };
            assert!(as_expected, "{wall:?} {choice:?}: {built:?}");
        }
    }
}

/// The flat face holds the annulus between its two circles, in each way of
/// writing the wall.
#[test]
fn the_flat_face_holds_the_annulus_between_its_two_circles() {
    let expected = [
        ([4.0, 0.0, 0.0], OnAFace::Held),
        ([3.5, 0.0, 0.0], OnAFace::Held),
        ([0.0, 0.0, 0.0], OnAFace::NotHeld),
        ([6.0, 0.0, 0.0], OnAFace::NotHeld),
        ([5.0, 0.0, 0.0], OnAFace::OnItsBoundary),
        ([3.0, 0.0, 0.0], OnAFace::OnItsBoundary),
    ];
    for wall in every_wall() {
        let (model, half_ring) = the_half_ring(wall);
        let view = model.solid(half_ring).expect("its own key");
        let flat = view
            .faces()
            .find(|face| face.surface().as_plane().is_some())
            .expect("a flat face")
            .key();
        for (point, answer) in expected {
            let held = view.face_holds_the_point(
                flat,
                PredicatePoint::from(point),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            assert!(
                matches!(held, Ok(got) if got == answer),
                "{wall:?} at {point:?}: {held:?}"
            );
        }
    }
}

/// A point on the flat face is on the half-ring's skin, and the middle of the
/// hole is outside it, in each way of writing the wall.
#[test]
fn classify_point_finds_the_flat_face_on_the_skin_and_the_hole_outside() {
    for wall in every_wall() {
        let (model, half_ring) = the_half_ring(wall);
        let view = model.solid(half_ring).expect("its own key");
        for (point, answer) in [
            (Point::new(4.0, 0.0, 0.0), Containment::OnBoundary),
            (Point::new(0.0, 0.0, 0.0), Containment::Outside),
        ] {
            let found = view.classify_point(point, &Policy::certified(), &mut ProvenanceLog::new());
            assert_eq!(found, answer, "{wall:?} at {point:?}");
        }
    }
}

/// The half-ring holds what Pappus gives, in each way of writing the wall, at
/// three deflections, each within the error its own measurement states.
///
/// The half of the tube's disc above `z = 0` has area `π r² / 2`, and its
/// centroid is `R` from the axis, because the disc is cut through its centre
/// along a line square to the axis. So it sweeps `2π · R · π r² / 2 = π² r² R`,
/// half of the whole ring's `2π² r² R`. For a ring of four round a tube of one
/// that is `4π² = 39.478`.
#[test]
fn the_half_ring_holds_what_pappus_gives() {
    let pappus = PI * PI * TUBE * TUBE * RING;
    for wall in every_wall() {
        let (model, half_ring) = the_half_ring(wall);
        let view = model.solid(half_ring).expect("its own key");
        for deflection in [0.1, 0.03, 0.01] {
            let measured = view
                .measure(
                    Deflection::of(deflection).expect("a positive length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{wall:?} at {deflection} was refused {why:?}"));
            assert!(
                (measured.volume() - pappus).abs() <= measured.volume_error(),
                "{wall:?} at {deflection}: measured {} where Pappus gives {pappus}, allowed {}",
                measured.volume(),
                measured.volume_error()
            );
        }
    }
}

/// The three ways §250.7 writes the wall.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Wall {
    /// Two wires, one circle each, with both corners on `+x`.
    WithNoSeam,
    /// One wire: round the outer circle, over the top of the tube along the
    /// seam over `(4, 0, 1)`, round the inner circle and back along the seam.
    /// Both corners are on `+x`, and the seam is picked by `x ≥ 0` and `z ≥ 0`.
    WithASeamOverPlusX,
    /// The same with both corners on `−x`, and the seam over `(−4, 0, 1)`
    /// picked by `x ≤ 0` and `z ≥ 0`. That seam is where `atan2` cuts the ring
    /// angle (§280).
    WithASeamOverMinusX,
}

fn every_wall() -> [Wall; 3] {
    [
        Wall::WithNoSeam,
        Wall::WithASeamOverPlusX,
        Wall::WithASeamOverMinusX,
    ]
}

/// Which way each circle runs on the flat face, and which way the wall is used
/// in the shell. The wall walks each circle the other way from the flat face.
#[derive(Clone, Copy, Debug)]
struct Choice {
    outer_on_the_flat_face: Orientation,
    inner_on_the_flat_face: Orientation,
    wall_in_the_shell: Orientation,
}

fn every_choice() -> impl Iterator<Item = Choice> {
    let either = |forward: bool| {
        if forward {
            Orientation::Forward
        } else {
            Orientation::Reversed
        }
    };
    (0u32..8).map(move |bits| Choice {
        outer_on_the_flat_face: either(bits & 1 == 0),
        inner_on_the_flat_face: either(bits & 2 == 0),
        wall_in_the_shell: either(bits & 4 == 0),
    })
}

/// The half-ring with every loop of both faces walked backwards, both faces
/// still used `Forward`, and a seam in the wall: outer circle `Reversed` and
/// hole `Forward` on the flat face, and the wall walking the half-ring's own
/// wire the other way round.
///
/// **It builds, passes `Model::validate`, and is two different shapes.** It
/// measures what Pappus gives the top half, so its mesh is the half-ring. But
/// its wall, asked `face_holds_the_point`, holds `(4, 0, −1)` and not
/// `(4, 0, 1)`, and `classify_point` puts `(4, 0, −0.5)` inside it and
/// `(4, 0, 0.5)` outside: what is asked reads the bottom half off the way the
/// wire walks its circles, and the mesh draws the top half the seam picks.
/// With no seam the same walk is refused `FacesDoNotAgreeWhichWayTheyFace`, and
/// a quarter bend with every loop walked backwards is refused
/// `AClosedEdgeIsWalkedAgainstItsSurface`. That is a wrong answer and not a
/// half-ring, so it is not pinned as one; §250.10 files the fix, which turns
/// these two into refusals pinned here.
fn is_every_loop_walked_backwards_round_a_seam(wall: Wall, choice: Choice) -> bool {
    wall != Wall::WithNoSeam
        && choice.outer_on_the_flat_face == Orientation::Reversed
        && choice.inner_on_the_flat_face == Orientation::Forward
        && choice.wall_in_the_shell == Orientation::Forward
}

/// The half-ring, built with its own walk, in a model of its own.
fn the_half_ring(wall: Wall) -> (Model, SolidKey) {
    let mut model = a_model();
    let choice = Choice {
        outer_on_the_flat_face: Orientation::Forward,
        inner_on_the_flat_face: Orientation::Reversed,
        wall_in_the_shell: Orientation::Forward,
    };
    let half_ring = a_half_ring(&mut model, wall, choice)
        .unwrap_or_else(|why| panic!("{wall:?}: the half-ring was refused {why:?}"));
    (model, half_ring)
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("a working size"))
}

/// The half-ring written face by face, the flat face first and the wall
/// second, as `rings::the_top_half_of_a_ring` writes it with `choice` the
/// half-ring's own walk.
fn a_half_ring(model: &mut Model, wall: Wall, choice: Choice) -> Result<SolidKey> {
    let side = if wall == Wall::WithASeamOverMinusX {
        -1.0
    } else {
        1.0
    };
    let mut builder = SolidBuilder::new();
    let outer_corner = builder.add_vertex(Point::new(side * (RING + TUBE), 0.0, 0.0))?;
    let inner_corner = builder.add_vertex(Point::new(side * (RING - TUBE), 0.0, 0.0))?;
    let outer = builder.add_edge(
        outer_corner,
        outer_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;
    let inner = builder.add_edge(
        inner_corner,
        inner_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;

    let underneath = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("the z axis is a unit normal");
    let mut flat_wires = Vec::with_capacity(2);
    for (edge, turn) in [
        (outer, choice.outer_on_the_flat_face),
        (inner, choice.inner_on_the_flat_face),
    ] {
        let coedge = builder.add_coedge(edge, turn)?;
        flat_wires.push(builder.add_wire(&[DraftStep::Along(coedge)])?);
    }
    let flat = builder.add_face(Surface::Plane(underneath), &flat_wires)?;

    let outer_on_the_wall = choice.outer_on_the_flat_face.reversed();
    let inner_on_the_wall = choice.inner_on_the_flat_face.reversed();
    let mut wall_wires = Vec::with_capacity(2);
    if wall == Wall::WithNoSeam {
        for (edge, turn) in [(outer, outer_on_the_wall), (inner, inner_on_the_wall)] {
            let coedge = builder.add_coedge(edge, turn)?;
            wall_wires.push(builder.add_wire(&[DraftStep::Along(coedge)])?);
        }
    } else {
        let across_the_tube =
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
                .expect("the y axis is a unit normal");
        let seam = builder.add_edge(
            outer_corner,
            inner_corner,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(across_the_tube),
                picked_by: smallvec![
                    Picker::from(
                        HalfSpace::new(Vector::new(side, 0.0, 0.0), 0.0)
                            .expect("a normal with a length")
                    ),
                    Picker::from(
                        HalfSpace::new(Vector::new(0.0, 0.0, 1.0), 0.0)
                            .expect("a normal with a length")
                    ),
                ],
            },
        )?;
        let mut steps = Vec::with_capacity(4);
        for (edge, turn) in [
            (outer, outer_on_the_wall),
            (seam, Orientation::Forward),
            (inner, inner_on_the_wall),
            (seam, Orientation::Reversed),
        ] {
            steps.push(DraftStep::Along(builder.add_coedge(edge, turn)?));
        }
        wall_wires.push(builder.add_wire(&steps)?);
    }
    let round = builder.add_face(
        Surface::Torus(Torus::new(Axis::z(), RING, TUBE).expect("a tube narrower than its ring")),
        &wall_wires,
    )?;

    builder.add_shell(&[
        (flat, Orientation::Forward),
        (round, choice.wall_in_the_shell),
    ])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}
