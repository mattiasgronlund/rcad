//! What `rcad_testing` asked for (`docs/decisions.md` §242), reached with
//! `rcad` as the only dependency.
//!
//! This file names [`Length`], [`ROOM_FOR_ATAN2`] and [`FaceMeasurement`] from
//! the facade and nowhere else, so a name dropped from the re-export list stops
//! the build here rather than in the harness.

use rcad::{
    Deflection, FaceMeasurement, Length, Model, Policy, ProvenanceLog, ROOM_FOR_ATAN2, Scale,
    SolidKey,
};

fn a_cube_of_side_two() -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    (model, cube)
}

#[test]
fn each_face_of_a_cube_measures_four_and_they_add_up_to_its_skin() {
    let (model, cube) = a_cube_of_side_two();
    let solid = model.solid(cube).expect("this model made the key");
    let how_fine = Deflection::of(0.1).expect("a tenth of a millimetre");

    let mut skin = 0.0;
    for face in solid.faces() {
        let measured: FaceMeasurement = solid
            .measure_one_face(
                face.key(),
                how_fine,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a face of a cube measures");
        assert_eq!(measured.area(), 4.0);
        assert_eq!(measured.area_error(), 0.0);
        let middle = measured.centroid();
        let from_the_cubes_middle =
            ((middle.x() - 1.0).powi(2) + (middle.y() - 1.0).powi(2) + (middle.z() - 1.0).powi(2))
                .sqrt();
        assert_eq!(from_the_cubes_middle, 1.0, "{middle:?}");
        assert_eq!(measured.centroid_wobble(), 0.0);
        skin += measured.area();
    }
    let whole = solid
        .measure(how_fine, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube measures");
    assert_eq!(skin, whole.surface_area());
}

#[test]
fn the_rim_of_a_cube_face_is_eight_long() {
    let (model, cube) = a_cube_of_side_two();
    for face in model.solid(cube).expect("this model made the key").faces() {
        let rim: Length = face.outer_wire().length().expect("four straight edges");
        assert!(
            (rim.length() - 8.0).abs() <= rim.error(),
            "{rim:?} against four sides of two"
        );
    }
}

#[test]
fn the_room_allowed_for_atan2_is_the_eight_roundings_it_publishes() {
    assert_eq!(ROOM_FOR_ATAN2, 8.0 * f64::EPSILON);
}
