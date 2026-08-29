//! Building planar solids out of profiles, and refusing profiles that are not
//! profiles.

use rcad_geom::{Isometry, Point, Scale};
use rcad_predicates::Policy;
use rcad_topo::{Error, Model, Profile, ProfileBuilder};

fn empty_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn profile_of(outline: Vec<[f64; 2]>) -> Profile {
    ProfileBuilder::new(outline)
        .build(&Policy::certified())
        .expect("a sound outline")
}

#[test]
fn a_cube_of_side_two_holds_eight_of_space_and_twentyfour_of_skin() {
    let mut model = empty_model();
    let cube = model.add_cube(2.0).expect("a cube of side two");

    let measured = model.solid(cube).measure();

    assert_eq!(measured.volume(), 8.0);
    assert_eq!(measured.surface_area(), 24.0);
    assert_eq!(measured.centroid(), Point::new(1.0, 1.0, 1.0));
}

#[test]
fn a_cube_has_eight_corners_twelve_edges_and_six_faces() {
    let mut model = empty_model();
    model.add_cube(2.0).expect("a cube of side two");

    assert_eq!(model.vertex_count(), 8);
    assert_eq!(model.edge_count(), 12);
    assert_eq!(model.coedge_count(), 24);
    assert_eq!(model.wire_count(), 6);
    assert_eq!(model.face_count(), 6);
    assert_eq!(model.shell_count(), 1);
    assert_eq!(model.solid_count(), 1);
}

#[test]
fn every_edge_of_a_cube_is_used_by_two_coedges_running_opposite_ways() {
    let mut model = empty_model();
    model.add_cube(2.0).expect("a cube of side two");

    for edge in model.edges() {
        let ways: Vec<_> = edge
            .coedges()
            .map(|coedge| model.coedge(coedge).orientation_on_its_edge())
            .collect();
        assert_eq!(ways.len(), 2);
        assert_ne!(ways[0], ways[1], "both coedges run the same way");
    }
}

#[test]
fn every_face_of_a_cube_points_away_from_the_middle_of_it() {
    let mut model = empty_model();
    let cube = model.add_cube(2.0).expect("a cube of side two");
    let middle = model.solid(cube).measure().centroid();

    for face in model.solid(cube).faces() {
        let corner = face
            .outer_wire()
            .points()
            .next()
            .expect("a wire has points");
        let outwards = (corner - middle).dot(face.outward_normal());
        assert!(outwards > 0.0, "a face points back into the solid");
    }
}

#[test]
fn a_cube_passes_the_full_validity_sweep() {
    let mut model = empty_model();
    model.add_cube(2.0).expect("a cube of side two");

    model.validate(&Policy::certified()).expect("a sound cube");
}

#[test]
fn a_plate_with_a_square_hole_has_the_hole_missing_from_its_volume() {
    let mut model = empty_model();
    let plate = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[4.0, 4.0], [6.0, 4.0], [6.0, 6.0], [4.0, 6.0]])
        .build(&Policy::certified())
        .expect("a plate with a hole in it");

    let solid = model
        .add_extrusion(&plate, 3.0, &Isometry::identity())
        .expect("a plate three thick");
    model.validate(&Policy::certified()).expect("a sound plate");

    let measured = model.solid(solid).measure();
    assert_eq!(measured.volume(), (100.0 - 4.0) * 3.0);
    assert_eq!(
        measured.surface_area(),
        2.0 * (100.0 - 4.0) + 40.0 * 3.0 + 8.0 * 3.0
    );
    assert_eq!(measured.centroid(), Point::new(5.0, 5.0, 1.5));
}

#[test]
fn a_plate_with_two_holes_still_has_two_coedges_on_every_edge() {
    let mut model = empty_model();
    let plate = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0]])
        .with_hole(vec![[6.0, 6.0], [8.0, 6.0], [8.0, 8.0], [6.0, 8.0]])
        .build(&Policy::certified())
        .expect("a plate with two holes");

    let solid = model
        .add_extrusion(&plate, 2.0, &Isometry::identity())
        .expect("a plate two thick");
    model.validate(&Policy::certified()).expect("a sound plate");

    assert_eq!(model.solid(solid).measure().volume(), (100.0 - 8.0) * 2.0);
    for edge in model.edges() {
        assert_eq!(edge.coedges().count(), 2);
    }
}

#[test]
fn an_l_shape_is_measured_right_even_though_it_is_not_convex() {
    let mut model = empty_model();
    let ell = profile_of(vec![
        [0.0, 0.0],
        [2.0, 0.0],
        [2.0, 1.0],
        [1.0, 1.0],
        [1.0, 2.0],
        [0.0, 2.0],
    ]);

    let solid = model
        .add_extrusion(&ell, 1.0, &Isometry::identity())
        .expect("an L one thick");
    model.validate(&Policy::certified()).expect("a sound L");

    let measured = model.solid(solid).measure();
    assert_eq!(measured.volume(), 3.0);
    assert!((measured.centroid().x() - 2.5 / 3.0).abs() < 1e-12);
    assert!((measured.centroid().y() - 2.5 / 3.0).abs() < 1e-12);
    assert_eq!(measured.centroid().z(), 0.5);
}

#[test]
fn a_profile_given_clockwise_is_turned_the_right_way_round() {
    let clockwise = profile_of(vec![[0.0, 0.0], [0.0, 2.0], [2.0, 2.0], [2.0, 0.0]]);
    let anticlockwise = profile_of(vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]]);

    assert_eq!(twice_the_area(clockwise.outline()), 8.0);
    assert_eq!(twice_the_area(anticlockwise.outline()), 8.0);

    let mut model = empty_model();
    let solid = model
        .add_extrusion(&clockwise, 1.0, &Isometry::identity())
        .expect("a block");
    assert_eq!(model.solid(solid).measure().volume(), 4.0);
}

/// Twice the area a ring sweeps out. Positive when it runs anticlockwise.
fn twice_the_area(ring: &[[f64; 2]]) -> f64 {
    (0..ring.len())
        .map(|index| {
            let here = ring[index];
            let next = ring[(index + 1) % ring.len()];
            here[0] * next[1] - next[0] * here[1]
        })
        .sum()
}

#[test]
fn a_hole_given_anticlockwise_is_turned_the_right_way_round() {
    let plate = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[4.0, 4.0], [6.0, 4.0], [6.0, 6.0], [4.0, 6.0]])
        .build(&Policy::certified())
        .expect("a plate with a hole");

    assert_eq!(twice_the_area(&plate.holes()[0]), -8.0);
}

#[test]
fn a_loop_of_two_points_is_refused() {
    let refused = ProfileBuilder::new(vec![[0.0, 0.0], [1.0, 0.0]]).build(&Policy::certified());

    assert!(matches!(refused, Err(Error::TooFewPoints { count: 2 })));
}

#[test]
fn a_profile_that_repeats_a_point_is_refused() {
    let refused = ProfileBuilder::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 0.0], [1.0, 1.0]])
        .build(&Policy::certified());

    assert!(matches!(refused, Err(Error::RepeatedPoint { .. })));
}

#[test]
fn a_profile_with_a_point_that_is_not_a_number_is_refused() {
    let refused = ProfileBuilder::new(vec![[0.0, 0.0], [f64::NAN, 0.0], [1.0, 1.0]])
        .build(&Policy::certified());

    assert!(matches!(
        refused,
        Err(Error::PointNotFinite { position: 1 })
    ));
}

#[test]
fn a_bowtie_profile_is_refused() {
    let refused = ProfileBuilder::new(vec![[0.0, 0.0], [2.0, 0.0], [0.0, 2.0], [2.0, 2.0]])
        .build(&Policy::certified());

    assert!(matches!(refused, Err(Error::ProfileCrossesItself)));
}

#[test]
fn a_profile_that_doubles_back_along_itself_is_refused() {
    let refused = ProfileBuilder::new(vec![[0.0, 0.0], [2.0, 0.0], [1.0, 0.0], [1.0, 1.0]])
        .build(&Policy::certified());

    assert!(matches!(refused, Err(Error::ProfileCrossesItself)));
}

#[test]
fn a_profile_pinched_against_itself_is_refused() {
    let refused = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [2.0, 0.0],
        [1.0, 1.0],
        [2.0, 2.0],
        [0.0, 2.0],
        [1.0, 1.0],
    ])
    .build(&Policy::certified());

    assert!(matches!(refused, Err(Error::ProfileCrossesItself)));
}

#[test]
fn a_hole_outside_the_outline_is_refused() {
    let refused = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[20.0, 20.0], [22.0, 20.0], [22.0, 22.0], [20.0, 22.0]])
        .build(&Policy::certified());

    assert!(matches!(
        refused,
        Err(Error::HoleNotInsideOutline { hole: 0 })
    ));
}

#[test]
fn a_hole_that_pokes_through_the_outline_is_refused() {
    let refused = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[8.0, 4.0], [12.0, 4.0], [12.0, 6.0], [8.0, 6.0]])
        .build(&Policy::certified());

    assert!(matches!(refused, Err(Error::ProfileCrossesItself)));
}

#[test]
fn a_hole_inside_another_hole_is_refused() {
    let refused = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[2.0, 2.0], [8.0, 2.0], [8.0, 8.0], [2.0, 8.0]])
        .with_hole(vec![[3.0, 3.0], [7.0, 3.0], [7.0, 7.0], [3.0, 7.0]])
        .build(&Policy::certified());

    assert!(matches!(
        refused,
        Err(Error::HoleInsideAnotherHole { hole: 1 })
    ));
}

#[test]
fn an_extrusion_of_no_height_is_refused() {
    let mut model = empty_model();
    let square = profile_of(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);

    let refused = model.add_extrusion(&square, 0.0, &Isometry::identity());

    assert!(matches!(refused, Err(Error::HeightNotPositive { height }) if height == 0.0));
    assert_eq!(model.solid_count(), 0);
}

#[test]
fn a_cube_of_no_side_is_refused() {
    let mut model = empty_model();

    let refused = model.add_cube(-1.0);

    assert!(matches!(refused, Err(Error::HeightNotPositive { .. })));
}
