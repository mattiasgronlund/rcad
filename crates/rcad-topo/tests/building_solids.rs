//! Building planar solids out of profiles, and refusing profiles that are not
//! profiles.

use rcad_geom::{Isometry, Point, Scale};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Profile, ProfileBuilder};

fn empty_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn profile_of(outline: Vec<[f64; 2]>) -> Profile {
    ProfileBuilder::new(outline)
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a sound outline")
}

#[test]
fn a_cube_of_side_two_holds_eight_of_space_and_twentyfour_of_skin() {
    let mut model = empty_model();
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");

    let measured = model.solid(cube).measure();

    assert_eq!(measured.volume(), 8.0);
    assert_eq!(measured.surface_area(), 24.0);
    assert_eq!(measured.centroid(), Point::new(1.0, 1.0, 1.0));
}

#[test]
fn a_cube_has_eight_corners_twelve_edges_and_six_faces() {
    let mut model = empty_model();
    model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");

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
    model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");

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
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
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
    model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");

    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a sound cube");
}

#[test]
fn a_plate_with_a_square_hole_has_the_hole_missing_from_its_volume() {
    let mut model = empty_model();
    let plate = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[4.0, 4.0], [6.0, 4.0], [6.0, 6.0], [4.0, 6.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a plate with a hole in it");

    let solid = model
        .add_extrusion(&plate, 3.0, &Isometry::identity())
        .expect("a plate three thick");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a sound plate");

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
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a plate with two holes");

    let solid = model
        .add_extrusion(&plate, 2.0, &Isometry::identity())
        .expect("a plate two thick");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a sound plate");

    assert_eq!(model.solid(solid).measure().volume(), (100.0 - 8.0) * 2.0);
    for edge in model.edges() {
        assert_eq!(edge.coedges().count(), 2);
    }
}

#[test]
fn a_right_triangle_is_a_perfectly_good_profile() {
    let mut model = empty_model();
    let triangle = profile_of(vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]);

    let solid = model
        .add_extrusion(&triangle, 2.0, &Isometry::identity())
        .expect("a triangular prism");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a sound prism");

    assert_eq!(model.solid(solid).measure().volume(), 1.0);
}

#[test]
fn a_parallelogram_is_a_perfectly_good_profile() {
    let mut model = empty_model();
    let leaning = profile_of(vec![[0.0, 0.0], [1.0, 1.0], [1.0, 2.0], [0.0, 1.0]]);

    let solid = model
        .add_extrusion(&leaning, 3.0, &Isometry::identity())
        .expect("a leaning block");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a sound block");

    assert_eq!(model.solid(solid).measure().volume(), 3.0);
}

#[test]
fn a_triangular_hole_in_a_square_outline_is_a_perfectly_good_profile() {
    let mut model = empty_model();
    let plate = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[4.0, 4.0], [6.0, 4.0], [4.0, 6.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a plate with a triangular hole");

    let solid = model
        .add_extrusion(&plate, 3.0, &Isometry::identity())
        .expect("a plate three thick");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a sound plate");

    assert_eq!(model.solid(solid).measure().volume(), (100.0 - 2.0) * 3.0);
}

/// A corner can be as sharp as it likes as long as the two edges only share
/// their end. This one is a hair away from folding back and is still sound;
/// [`a_profile_that_doubles_back_along_itself_is_refused`] is the same corner
/// once it has actually folded.
#[test]
fn a_corner_almost_folded_back_on_itself_is_still_a_corner() {
    let sliver = profile_of(vec![[0.0, 0.0], [2.0, 0.0], [1.0, 1e-9], [1.0, 1.0]]);

    assert_eq!(sliver.outline().len(), 4);
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
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a sound L");

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
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a plate with a hole");

    assert_eq!(twice_the_area(&plate.holes()[0]), -8.0);
}

#[test]
fn a_loop_of_two_points_is_refused() {
    let refused = ProfileBuilder::new(vec![[0.0, 0.0], [1.0, 0.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new());

    assert!(matches!(refused, Err(Error::TooFewPoints { count: 2 })));
}

#[test]
fn a_profile_that_repeats_a_point_is_refused() {
    let refused = ProfileBuilder::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 0.0], [1.0, 1.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new());

    assert!(matches!(refused, Err(Error::RepeatedPoint { .. })));
}

#[test]
fn a_profile_with_a_point_that_is_not_a_number_is_refused() {
    let refused = ProfileBuilder::new(vec![[0.0, 0.0], [f64::NAN, 0.0], [1.0, 1.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new());

    assert!(matches!(
        refused,
        Err(Error::PointNotFinite { position: 1 })
    ));
}

#[test]
fn a_bowtie_profile_is_refused() {
    let refused = ProfileBuilder::new(vec![[0.0, 0.0], [2.0, 0.0], [0.0, 2.0], [2.0, 2.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new());

    assert!(matches!(refused, Err(Error::ProfileCrossesItself)));
}

#[test]
fn a_profile_that_doubles_back_along_itself_is_refused() {
    let refused = ProfileBuilder::new(vec![[0.0, 0.0], [2.0, 0.0], [1.0, 0.0], [1.0, 1.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new());

    assert!(matches!(refused, Err(Error::ProfileCrossesItself)));
}

/// The corner a ring's direction is read off — the lowest point, leftmost of
/// those — is never straight, and `ring_runs_anticlockwise` says so with
/// `unreachable!`. It is never straight because both its neighbours sit at or
/// above it, so three of them on one line puts both neighbours on the same ray
/// out of the corner, with one between the corner and the other. That is a
/// fold, and the crossing check has already refused it by the time the
/// direction is read.
#[test]
fn a_profile_lying_flat_at_its_lowest_leftmost_corner_is_refused_as_a_crossing() {
    let refused = ProfileBuilder::new(vec![[0.0, 0.0], [2.0, 0.0], [2.0, 1.0], [1.0, 0.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new());

    assert!(matches!(refused, Err(Error::ProfileCrossesItself)));
}

/// Every short ring on a small grid, which is where collinear corners are thick
/// on the ground. None of them may reach that `unreachable!`, and none of them
/// may come back with anything other than a sound profile or one of the two
/// refusals a ring of grid points can earn.
#[test]
fn no_short_ring_of_grid_points_reaches_a_flat_lowest_leftmost_corner() {
    let grid: Vec<[f64; 2]> = (0..4)
        .flat_map(|x| (0..4).map(move |y| [f64::from(x), f64::from(y)]))
        .collect();
    let policy = Policy::certified();

    let mut rings: Vec<Vec<[f64; 2]>> = Vec::new();
    for first in &grid {
        for second in &grid {
            for third in &grid {
                rings.push(vec![*first, *second, *third]);
                for fourth in &grid {
                    rings.push(vec![*first, *second, *third, *fourth]);
                }
            }
        }
    }

    // A sound profile is one whose direction was read, so counting them counts
    // the times the corner in question was actually looked at. Without this the
    // sweep could pass by never getting that far.
    let mut directions_read = 0;
    for ring in rings {
        match ProfileBuilder::new(ring).build(&policy, &mut ProvenanceLog::new()) {
            Ok(_) => directions_read += 1,
            Err(Error::ProfileCrossesItself | Error::RepeatedPoint { .. }) => {}
            other => panic!("a ring of grid points came back with {other:?}"),
        }
    }

    assert!(
        directions_read > 1000,
        "only {directions_read} rings held up"
    );
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
    .build(&Policy::certified(), &mut ProvenanceLog::new());

    assert!(matches!(refused, Err(Error::ProfileCrossesItself)));
}

#[test]
fn a_hole_outside_the_outline_is_refused() {
    let refused = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[20.0, 20.0], [22.0, 20.0], [22.0, 22.0], [20.0, 22.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new());

    assert!(matches!(
        refused,
        Err(Error::HoleNotInsideOutline { hole: 0 })
    ));
}

#[test]
fn a_hole_that_pokes_through_the_outline_is_refused() {
    let refused = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[8.0, 4.0], [12.0, 4.0], [12.0, 6.0], [8.0, 6.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new());

    assert!(matches!(refused, Err(Error::ProfileCrossesItself)));
}

#[test]
fn a_hole_inside_another_hole_is_refused() {
    let refused = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[2.0, 2.0], [8.0, 2.0], [8.0, 8.0], [2.0, 8.0]])
        .with_hole(vec![[3.0, 3.0], [7.0, 3.0], [7.0, 7.0], [3.0, 7.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new());

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

    let refused = model.add_cube(-1.0, &Policy::certified(), &mut ProvenanceLog::new());

    assert!(matches!(refused, Err(Error::HeightNotPositive { .. })));
}
