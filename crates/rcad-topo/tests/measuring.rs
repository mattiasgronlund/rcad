//! Volume, area and centroid, and the fact that moving a solid does not change
//! the first two (roadmap §11.2 says start here, and it costs nothing).

use rcad_geom::{Axis, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, SolidKey};

fn a_cube_of_side_two() -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cube = model.add_cube(2.0).expect("a cube of side two");
    (model, cube)
}

fn a_third_of_a_turn_about_a_slanted_axis() -> Isometry {
    let axis = Axis::new(Point::new(0.5, -1.5, 2.0), Vector::new(1.0, 1.0, 1.0))
        .expect("a direction with length");
    Isometry::rotation(&axis, core::f64::consts::TAU / 3.0).expect("a rotation")
}

#[test]
fn a_cube_rotated_by_a_third_of_a_turn_has_the_same_volume_and_area() {
    let (mut model, cube) = a_cube_of_side_two();
    let before = model.solid(cube).measure();

    let turn = a_third_of_a_turn_about_a_slanted_axis();
    model.place_solid(cube, &turn);
    let after = model.solid(cube).measure();

    assert!((after.volume() - before.volume()).abs() < 1e-12);
    assert!((after.surface_area() - before.surface_area()).abs() < 1e-12);
    assert!(
        after
            .centroid()
            .distance_to(turn.apply_to_point(before.centroid()))
            < 1e-12
    );
}

#[test]
fn a_rotated_cube_is_still_a_sound_solid() {
    let (mut model, cube) = a_cube_of_side_two();

    model.place_solid(cube, &a_third_of_a_turn_about_a_slanted_axis());

    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rotation cannot break a solid");
}

#[test]
fn moving_a_plate_with_a_hole_leaves_its_volume_alone() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let plate = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[4.0, 4.0], [6.0, 4.0], [6.0, 6.0], [4.0, 6.0]])
        .build(&Policy::certified())
        .expect("a plate with a hole");
    let solid = model
        .add_extrusion(&plate, 3.0, &Isometry::identity())
        .expect("a plate three thick");
    let before = model.solid(solid).measure();

    model.place_solid(solid, &a_third_of_a_turn_about_a_slanted_axis());
    let after = model.solid(solid).measure();

    assert!((after.volume() - before.volume()).abs() < 1e-11);
    assert!((after.surface_area() - before.surface_area()).abs() < 1e-11);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("still sound");
}

#[test]
fn moving_a_solid_twice_is_the_same_as_moving_it_once_with_both_movements() {
    let shift = Isometry::translation(Vector::new(3.0, -7.0, 11.0)).expect("a finite shift");
    let turn = a_third_of_a_turn_about_a_slanted_axis();

    let (mut step_by_step, first) = a_cube_of_side_two();
    step_by_step.place_solid(first, &shift);
    step_by_step.place_solid(first, &turn);

    let (mut all_at_once, second) = a_cube_of_side_two();
    all_at_once.place_solid(second, &turn.compose(&shift));

    let stepped = step_by_step.solid(first).measure();
    let together = all_at_once.solid(second).measure();
    assert_eq!(stepped, together);
}

#[test]
fn the_middle_of_a_placed_cube_moves_with_it() {
    let (mut model, cube) = a_cube_of_side_two();
    let shift = Isometry::translation(Vector::new(100.0, 0.0, 0.0)).expect("a finite shift");

    model.place_solid(cube, &shift);

    assert_eq!(
        model.solid(cube).measure().centroid(),
        Point::new(101.0, 1.0, 1.0)
    );
}

#[test]
fn a_bigger_cube_holds_the_cube_of_its_side() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let small = model.add_cube(1.0).expect("a cube of side one");
    let large = model.add_cube(3.0).expect("a cube of side three");

    assert_eq!(model.solid(small).measure().volume(), 1.0);
    assert_eq!(model.solid(large).measure().volume(), 27.0);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("two sound cubes");
}
