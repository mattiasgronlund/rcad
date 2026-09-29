//! The writer's fuzz harness asks each solid what the reader asks it.
//!
//! The reader builds every solid it reads through `SolidBuilder::build`, which
//! judges the solid in a model of its own, as `Model::validate_solid` does.
//! `Model::validate` judges the whole model, and a far neighbour widens the
//! room it gives a corner off its plane (`docs/decisions.md` §242.6). So a
//! model `validate` passes can be written as a file the reader refuses, with
//! nothing wrong in the writer or the reader. Twelve of the 2026-09-26 writer
//! crash files were that (§326), and the harness now counts such a model as
//! `written_but_a_solid_unsound_alone` rather than panicking on it.
//!
//! The thirteenth file was a second cause: the room also depended on where the
//! solid sits, because `validate` read it where the corners are stored and the
//! reader where the file puts them (§326.3). `validate` and `validate_solid`
//! now read it at whichever of the two is smaller (§363), so they refuse that
//! solid as the reader does.

#![forbid(unsafe_code)]

mod fuzzing;

use std::path::PathBuf;

use rcad_geom::{
    Axis, Cylinder, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile};
use rcad_topo::{DraftStep, Invalid, Model, SolidBuilder, SolidKey};

use fuzzing::writer::one_model_decoded_and_written;

/// `crash-9018206a0ba623e51da13398cf5d9e8eb08424b5`: a unit cube whose `+y`
/// face sits on the plane at `1 + 2⁻²⁹`, beside a round bar of radius 2 and
/// height 15. The cube's corners are `2⁻²⁹ ≈ 1.86e-9` off that plane. The
/// model's largest coordinate is 15, which gives every corner about `1.5e-8`
/// of room, and the cube's own is 1, which gives it about `1e-9`.
#[test]
fn a_cube_a_hair_off_its_plane_is_counted_rather_than_failed() {
    let bytes = std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fuzz/corpus/no_model_makes_the_writer_panic")
            .join("a_unit_cube_a_hair_off_its_plane_beside_a_tall_bar"),
    )
    .expect("the seed is checked in");
    let model = postcard::from_bytes::<Model>(&bytes).expect("the seed decodes");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    model
        .validate(&policy, &mut log)
        .expect("the bar makes room for the cube's corners");
    let solids: Vec<SolidKey> = model.solids().map(|solid| solid.key()).collect();
    assert_eq!(solids.len(), 2);
    let cube_alone = model.validate_solid(solids[0], &policy, &mut log);
    assert!(
        matches!(
            cube_alone,
            Err(rcad_topo::Error::Invalid(
                Invalid::VertexOffItsFacePlane { .. }
            ))
        ),
        "the cube alone has only its own room: {cube_alone:?}"
    );
    model
        .validate_solid(solids[1], &policy, &mut log)
        .expect("the bar is sound on its own");

    let reached = one_model_decoded_and_written(&bytes);
    assert_eq!(
        (
            reached.written,
            reached.written_but_a_solid_unsound_alone,
            reached.read_back
        ),
        (1, 1, 0),
        "{reached:?}"
    );
}

/// The thirteenth file's solid, built alone: a round bar of radius 4 whose
/// top corner sits `2⁻²⁸ ≈ 3.73e-9` above its cap at 4. Its largest stored
/// coordinate is that corner's `4.0000000037`, so its room is about `4e-9`
/// and both `validate` and `validate_solid` pass it. Where it was built, the
/// file carries those same numbers and the reader passes it too.
#[test]
fn a_bar_whose_corner_is_a_hair_above_its_cap_comes_back_where_it_was_built() {
    let bytes = postcard::to_allocvec(&a_bar_a_hair_tall_placed_by(&Isometry::identity()))
        .expect("a model always encodes");
    let reached = one_model_decoded_and_written(&bytes);
    assert_eq!(
        (reached.came_back_the_same, reached.solids_the_same),
        (1, 1),
        "{reached:?}"
    );
}

/// The same bar moved by `(−4, −3, −2)`, the move the fuzzer gave it. The
/// file writes it where it now sits, so its largest coordinate is 3 and the
/// reader gives it about `3e-9` of room, less than the `2⁻²⁸` its corner is
/// off. Both of this kernel's checks used to pass it, because both read the
/// corners where they are stored, and the harness panicked on it (§326.3).
/// They now give it the room of where it sits, and refuse it for the same
/// corner and the same distance the reader does (§363).
#[test]
fn the_same_bar_moved_toward_the_origin_is_refused_by_validate_as_by_the_reader() {
    let moved = Isometry::translation(Vector::new(-4.0, -3.0, -2.0)).expect("a finite offset");
    let model = a_bar_a_hair_tall_placed_by(&moved);
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let whole = model.validate(&policy, &mut log);
    let bar = model.solids().next().expect("one solid").key();
    let alone = model.validate_solid(bar, &policy, &mut log);
    for refusal in [whole, alone] {
        let Err(rcad_topo::Error::Invalid(Invalid::VertexOffItsFacePlane { distance })) = refusal
        else {
            panic!("the kernel refuses the bar's corner off its cap, not {refusal:?}");
        };
        assert_eq!(distance, 2.0_f64.powi(-28));
    }

    let reached = one_model_decoded_and_written(
        &postcard::to_allocvec(&model).expect("a model always encodes"),
    );
    assert_eq!(
        (
            reached.written,
            reached.written_but_unsound,
            reached.read_back
        ),
        (1, 1, 0),
        "{reached:?}"
    );

    let text = rcad_step::write(&model, "a bar a hair tall").expect("the writer writes a bar");
    let file = StepFile::read(text.as_bytes()).expect("the writer's own file parses");
    let refusal = Import::read(&file, &policy, &mut log).map(|_| ());
    let Err(Refusal::TheSolidTheFileDescribesIsNotOne { why, .. }) = &refusal else {
        panic!("the reader refuses the bar as a solid, not {refusal:?}");
    };
    let rcad_topo::Error::Invalid(Invalid::VertexOffItsFacePlane { distance }) = **why else {
        panic!("the reader refuses the bar's corner off its cap, not {why:?}");
    };
    assert_eq!(distance, 2.0_f64.powi(-28));
}

/// A round bar of radius 4, 4 tall, with its top corner `2⁻²⁸` above the cap,
/// then moved by `placement`.
fn a_bar_a_hair_tall_placed_by(placement: &Isometry) -> Model {
    let height = 4.0;
    let radius = 4.0;
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let mut builder = SolidBuilder::new();
    let bottom_corner = builder
        .add_vertex(Point::new(radius, 0.0, 0.0))
        .expect("a finite point");
    let top_corner = builder
        .add_vertex(Point::new(radius, 0.0, height + 2.0_f64.powi(-28)))
        .expect("a finite point");
    let seam = builder
        .add_edge(bottom_corner, top_corner, EdgeRun::Straight)
        .expect("a ruling of a cylinder is straight");
    let bottom_circle = builder
        .add_edge(
            bottom_corner,
            bottom_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a cap plane cuts the cylinder in one closed curve");
    let top_circle = builder
        .add_edge(
            top_corner,
            top_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a cap plane cuts the cylinder in one closed curve");

    let mut cap = |z: f64, outward: f64, circle, turn| {
        let plane =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, z), Vector::new(0.0, 0.0, outward))
                .expect("the z axis is a unit normal");
        let coedge = builder
            .add_coedge(circle, turn)
            .expect("an edge of this build");
        let wire = builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("one closed edge is a wire");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a cap is a plane trimmed by its circle")
    };
    let bottom = cap(0.0, -1.0, bottom_circle, Orientation::Reversed);
    let top = cap(height, 1.0, top_circle, Orientation::Forward);

    let round_the_wall: Vec<DraftStep> = [
        (bottom_circle, Orientation::Forward),
        (seam, Orientation::Forward),
        (top_circle, Orientation::Reversed),
        (seam, Orientation::Reversed),
    ]
    .into_iter()
    .map(|(edge, turn)| {
        DraftStep::Along(
            builder
                .add_coedge(edge, turn)
                .expect("an edge of this build"),
        )
    })
    .collect();
    let wire = builder
        .add_wire(&round_the_wall)
        .expect("round one end, along the seam, round the other, and back");
    let side = builder
        .add_face(
            Surface::Cylinder(Cylinder::new(Axis::z(), radius).expect("a positive radius")),
            &[wire],
        )
        .expect("a face on the cylinder");
    builder
        .add_shell(&[
            (bottom, Orientation::Forward),
            (top, Orientation::Forward),
            (side, Orientation::Forward),
        ])
        .expect("three faces close");
    let bar = builder
        .build(&mut model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("the corner is inside the room its own coordinate gives it");
    model
        .place_solid(bar, placement)
        .expect("the key came from this model");
    model
}
