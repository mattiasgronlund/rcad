//! Writing a model out and reading it back.
//!
//! The corpus is built by these constructors, not stored as files. Storing a
//! corpus in a format you intend to break is how you end up unable to break it
//! (decisions.md §6).

use rcad_geom::{
    Deflection, EdgeRun, HalfSpace, Isometry, Picker, Plane, Point, Scale, Surface, Vector,
};
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

/// **The round trip the version bump owes**, and the property that made the
/// field change type at all.
///
/// A ball's seam is picked out by a half-space, and a half-space stores the four
/// numbers it was handed with nothing scaled — so unlike a plane, which is
/// rebuilt and rescaled on the way in, it has to come back **bit for bit**. The
/// second half is the reason that matters: a normal written exactly square to a
/// direction stays exactly square across the round trip, and the same four
/// numbers put through a `Plane` do not.
///
/// The run goes through the format on its own, because putting one on a model
/// means naming keys the front door does not hand out. The model beside it is
/// what says the seam a ball really carries survives the trip and still
/// validates.
#[test]
fn a_half_space_that_picks_a_seam_comes_back_bit_for_bit_and_still_square() {
    let along = Vector::new(1.0, 3.0, 5.0);
    let square_to_it = Vector::new(3.0, -1.0, 0.0);
    assert_eq!(
        square_to_it.dot(along),
        0.0,
        "written square before anything stores it"
    );

    let written = HalfSpace::through_a_point_facing(Point::origin(), square_to_it)
        .expect("a normal with length");
    let meridian = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
        .expect("the y axis is a unit normal");
    let seam = EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(meridian),
        picked_by: [Picker::from(written)].into_iter().collect(),
    };

    let bytes = postcard::to_allocvec(&seam).expect("a run writes out");
    let after: EdgeRun = postcard::from_bytes(&bytes).expect("and reads back");
    let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = &after else {
        panic!("it came back a different kind of run")
    };
    let Picker::InFrontOf(came_back) = picked_by[0] else {
        panic!(
            "a seam is picked by a flat half-space, not {:?}",
            picked_by[0]
        )
    };

    assert_eq!(came_back.coefficients(), written.coefficients());
    assert_eq!(
        came_back.normal().dot(along),
        0.0,
        "and it is still exactly square to the direction it was written square to"
    );

    let mut model = empty_model();
    model
        .add_ball(2.0, &Isometry::identity())
        .expect("a ball of radius two");
    round_tripped(&model)
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a ball whose seam went through the format is still sound");
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

/// **A file saying a corner is infinitely out of place never becomes a
/// `Model`.**
///
/// An infinite magnitude is not a wider allowance, it is the certified check
/// switched off: `not_provably_off` in `validate.rs` keeps any magnitude above
/// zero, widens the corner into an interval that reaches everywhere, and every
/// on-surface test then passes for free. So a stranger's bytes could hand back
/// a model whose corners are nowhere near their faces and whose `validate`
/// says `Ok`.
///
/// The only difference between the bytes that read back and the bytes that are
/// refused is the eight bytes of that one number.
#[test]
fn a_file_saying_a_corner_is_infinitely_out_of_place_never_becomes_a_model() {
    let model = a_cube_with_one_corner_tagged(A_SLOP_NOTHING_ELSE_WRITES);

    assert!(
        postcard::from_bytes::<Model>(
            &postcard::to_allocvec(&model).expect("a model writes out")
        )
        .is_ok(),
        "the same model with a real amount of slop on it reads back"
    );
    assert_eq!(
        postcard::from_bytes::<Model>(&bytes_with_the_first_number_replaced(
            &model,
            A_SLOP_NOTHING_ELSE_WRITES,
            f64::INFINITY,
        ))
        .expect_err("an infinite amount of slop is refused"),
        postcard::Error::SerdeDeCustom
    );
}

/// **And neither does one saying a corner is a negative distance out of
/// place.**
///
/// A magnitude is a length, and `check_approximation_ordering` puts lengths in
/// order with `<`. A negative one sorts below every real amount, so a face
/// could claim more slop than the edges it is built from and the ordering
/// check would agree with it.
#[test]
fn a_file_saying_a_corner_is_a_negative_distance_out_of_place_never_becomes_a_model() {
    let model = a_cube_with_one_corner_tagged(A_SLOP_NOTHING_ELSE_WRITES);

    assert_eq!(
        postcard::from_bytes::<Model>(&bytes_with_the_first_number_replaced(
            &model,
            A_SLOP_NOTHING_ELSE_WRITES,
            -1.0,
        ))
        .expect_err("a negative amount of slop is refused"),
        postcard::Error::SerdeDeCustom
    );
}

/// **A file whose corner is not at a finite point never becomes a `Model`
/// either.**
///
/// `Point` is read back with `#[serde(from = "[f64; 3]")]`, which cannot
/// refuse anything, and that is deliberate — a fallible branch on every
/// coordinate in the kernel is a real cost and the condition is one a sweep
/// already asks. What was missing is the sweep being on this path:
/// `Model::validate` catches it and `Deserialize` never calls `validate`.
///
/// The marker is a corner of the prism and nothing else in the file. The
/// arenas are written in the order `ModelData` declares them — the scale, then
/// the vertices, then everything that names a plane — so the first place these
/// eight bytes appear is a corner, not a surface.
#[test]
fn a_file_whose_corner_is_not_at_a_finite_point_never_becomes_a_model() {
    let mut model = empty_model();
    let triangle = ProfileBuilder::new(vec![[0.0, 0.0], [A_CORNER_NOTHING_ELSE_WRITES, 0.0], [
        2.9, 5.1,
    ]])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a triangle");
    model
        .add_extrusion(&triangle, 2.0, &Isometry::identity())
        .expect("a triangular bar");

    assert!(
        postcard::from_bytes::<Model>(
            &postcard::to_allocvec(&model).expect("a model writes out")
        )
        .is_ok(),
        "the same bar with all its corners in place reads back"
    );
    assert_eq!(
        postcard::from_bytes::<Model>(&bytes_with_the_first_number_replaced(
            &model,
            A_CORNER_NOTHING_ELSE_WRITES,
            f64::INFINITY,
        ))
        .expect_err("a corner that is nowhere is refused"),
        postcard::Error::SerdeDeCustom
    );
}

/// Guarding the door did not move the bytes.
///
/// [`Approximation`] gained `#[serde(try_from, into)]` over `(f64, Cause)`, and
/// a two-field struct and a two-element tuple are the same bytes in postcard —
/// which is why [`FORMAT_VERSION`] did not have to go up for it. Every model
/// already on disk still reads.
#[test]
fn a_guarded_approximation_is_still_the_same_bytes_it_always_was() {
    let tag = Approximation::new(2.5e-9, Cause::ImportedUncertainty);

    assert_eq!(
        postcard::to_allocvec(&tag).expect("a tag writes out"),
        postcard::to_allocvec(&(2.5e-9, Cause::ImportedUncertainty)).expect("and so does a pair"),
    );
    assert_eq!(FORMAT_VERSION, 3);
}

/// A slop nothing else in a cube's bytes writes, so patching it is unambiguous.
const A_SLOP_NOTHING_ELSE_WRITES: f64 = 3.5e-9;

/// A corner nothing else in a triangular bar's bytes writes. The bar's other
/// numbers are 0, 2.9, 5.1 and 2, and its side planes carry normals scaled to
/// length one, so none of them is this.
const A_CORNER_NOTHING_ELSE_WRITES: f64 = 7.3;

fn a_cube_with_one_corner_tagged(slop: f64) -> Model {
    let mut model = empty_model();
    model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    let corner = model.vertices().next().expect("a cube has corners").key();
    model
        .set_vertex_approximation(corner, Approximation::new(slop, Cause::ImportedUncertainty))
        .expect("a vertex may carry the most slop");
    model
}

/// The model's bytes with the first `f64` reading `marker` replaced by
/// `instead`.
fn bytes_with_the_first_number_replaced(model: &Model, marker: f64, instead: f64) -> Vec<u8> {
    let mut bytes = postcard::to_allocvec(model).expect("a model writes out");
    let wanted = marker.to_le_bytes();
    let at = bytes
        .windows(wanted.len())
        .position(|window| window == wanted)
        .unwrap_or_else(|| panic!("{marker} is written somewhere in the model"));
    bytes[at..at + wanted.len()].copy_from_slice(&instead.to_le_bytes());
    bytes
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
