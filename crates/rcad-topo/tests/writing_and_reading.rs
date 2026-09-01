//! Writing a model out and reading it back.
//!
//! The corpus is built by these constructors, not stored as files. Storing a
//! corpus in a format you intend to break is how you end up unable to break it
//! (decisions.md §6).

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Approximation, Cause, Policy, ProvenanceLog};
use rcad_topo::{FORMAT_VERSION, Model, ProfileBuilder};

fn empty_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn round_tripped(model: &Model) -> Model {
    let bytes = postcard::to_allocvec(model).expect("a model writes out");
    postcard::from_bytes(&bytes).expect("and reads back")
}

#[test]
fn a_cube_comes_back_sound_and_measuring_the_same() {
    let mut model = empty_model();
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    let before = model
        .solid(cube)
        .measure(
            a_deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid this size measures");

    let after = round_tripped(&model);

    after
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("still sound");
    let solid = after.solids().next().expect("the cube came back");
    assert_eq!(
        solid
            .measure(
                a_deflection(),
                &Policy::certified(),
                &mut ProvenanceLog::new()
            )
            .expect("a solid this size measures"),
        before
    );
    assert_eq!(after.vertex_count(), model.vertex_count());
    assert_eq!(after.edge_count(), model.edge_count());
    assert_eq!(after.face_count(), model.face_count());
    assert_eq!(after.worst_approximation(), model.worst_approximation());
}

#[test]
fn a_plate_with_two_holes_comes_back_sound_and_measuring_the_same() {
    let mut model = empty_model();
    let plate = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0]])
        .with_hole(vec![[6.0, 6.0], [8.0, 6.0], [8.0, 8.0], [6.0, 8.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a plate with two holes");
    let solid = model
        .add_extrusion(&plate, 2.0, &Isometry::identity())
        .expect("a plate two thick");
    let before = model
        .solid(solid)
        .measure(
            a_deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid this size measures");

    let after = round_tripped(&model);

    after
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("still sound");
    assert_eq!(
        after
            .solids()
            .next()
            .expect("the plate came back")
            .measure(
                a_deflection(),
                &Policy::certified(),
                &mut ProvenanceLog::new()
            )
            .expect("a solid this size measures"),
        before
    );
}

#[test]
fn a_placed_solid_comes_back_where_it_was_put() {
    let mut model = empty_model();
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    model.place_solid(
        cube,
        &Isometry::translation(Vector::new(30.0, -5.0, 7.0)).expect("a finite shift"),
    );
    let before = model
        .solid(cube)
        .measure(
            a_deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid this size measures");

    let after = round_tripped(&model);

    assert_eq!(
        after
            .solids()
            .next()
            .expect("the cube came back")
            .measure(
                a_deflection(),
                &Policy::certified(),
                &mut ProvenanceLog::new()
            )
            .expect("a solid this size measures"),
        before
    );
}

#[test]
fn approximation_tags_come_back_with_the_model() {
    let mut model = empty_model();
    model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    for vertex in model.vertices().map(|v| v.key()).collect::<Vec<_>>() {
        model
            .set_vertex_approximation(
                vertex,
                Approximation::new(2.5e-9, Cause::ImportedUncertainty),
            )
            .expect("a vertex may carry the most slop");
    }
    let edge = model.edges().next().expect("a cube has edges").key();
    model
        .set_edge_approximation(edge, Approximation::new(1e-9, Cause::TolerantFallback))
        .expect("an edge may carry less than its vertices");

    let after = round_tripped(&model);

    after
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("still sound");
    assert_eq!(after.worst_approximation(), model.worst_approximation());
    assert_eq!(after.worst_approximation(), 2.5e-9);
    let tagged = after
        .edges()
        .filter_map(rcad_topo::EdgeView::approximation)
        .collect::<Vec<_>>();
    assert_eq!(tagged.len(), 1);
    assert_eq!(tagged[0].magnitude(), 1e-9);
    assert_eq!(tagged[0].cause(), Cause::TolerantFallback);
    assert_eq!(
        after
            .vertices()
            .filter_map(rcad_topo::VertexView::approximation)
            .count(),
        8
    );
}

/// A model reading back with *less* slop than it had is the shape of the bug
/// this guards: the figure the model reports has to be the largest tag on any
/// entity, which is what a fresh read computes, and not a mark that only ever
/// rose while the model was being edited.
#[test]
fn a_tag_taken_back_down_survives_the_round_trip() {
    let mut model = empty_model();
    model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    let corner = model.vertices().next().expect("a cube has corners").key();
    model
        .set_vertex_approximation(corner, Approximation::new(1e-6, Cause::ImportedUncertainty))
        .expect("a vertex may carry slop");
    model
        .set_vertex_approximation(
            corner,
            Approximation::new(1e-12, Cause::ImportedUncertainty),
        )
        .expect("and may be tagged again with less");

    let after = round_tripped(&model);

    assert_eq!(model.worst_approximation(), 1e-12);
    assert_eq!(after.worst_approximation(), model.worst_approximation());
}

#[test]
fn a_model_written_by_a_different_version_of_the_format_is_refused() {
    let mut model = empty_model();
    model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    let mut bytes = postcard::to_allocvec(&model).expect("a model writes out");

    assert_eq!(
        bytes[0], FORMAT_VERSION as u8,
        "the version is the first thing written"
    );
    bytes[0] = FORMAT_VERSION as u8 + 1;

    // `SerdeDeCustom` rather than any other postcard error, because that is the
    // one the format check raises. A bare "it failed" would also be happy with
    // the bytes falling apart somewhere else and prove nothing about the check.
    assert_eq!(
        postcard::from_bytes::<Model>(&bytes).expect_err("a stale version is refused"),
        postcard::Error::SerdeDeCustom
    );
}

#[test]
fn the_keys_from_before_a_round_trip_do_not_belong_to_the_model_that_comes_back() {
    let mut model = empty_model();
    model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");

    let after = round_tripped(&model);

    assert_eq!(after.solid_count(), 1);
    assert_eq!(after.shell_count(), 1);
}

/// The deflection the measurements here are asked for.
///
/// Every solid in this file is a polyhedron, and a polyhedron ignores it: its
/// own fan is the answer rather than a picture of one (`docs/decisions.md`
/// §25.4).
fn a_deflection() -> Deflection {
    Deflection::of(0.1).expect("a tenth of a millimetre is a length")
}
