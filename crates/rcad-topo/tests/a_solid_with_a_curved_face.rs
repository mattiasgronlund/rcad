//! The first solid in this kernel that is not a polyhedron.
//!
//! A round bar is two vertices, three edges and three faces, and every one of
//! the structures M2 exists to build is in it: a face on a cylinder, a seam
//! used twice by that one face, and two closed edges that come back to the
//! corner they left. Nothing here is about bars. It is about whether those
//! three things can exist in the arena and survive a full sweep.

use std::collections::HashMap;

use rcad_geom::{Isometry, Point, Scale, Surface};
use rcad_predicates::{Decision, Policy, ProvenanceLog, SideOfSurface};
use rcad_topo::{Containment, Error, Model, Rule};

const RADIUS: f64 = 3.0;
const HEIGHT: f64 = 5.0;

fn a_round_bar() -> Model {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    model
        .add_round_bar(RADIUS, HEIGHT, &Isometry::identity())
        .expect("a bar three across and five tall");
    model
}

#[test]
fn a_round_bar_passes_the_full_sweep_with_nothing_approximated() {
    let model = a_round_bar();
    let mut log = ProvenanceLog::new();
    model
        .validate(&Policy::certified(), &mut log)
        .expect("a round bar is a sound solid");
    assert!(
        log.records().is_empty(),
        "a bar on the z axis needs nothing approximated, got {:?}",
        log.records()
    );
}

/// Two vertices and three edges, which is the whole reason to build a bar this
/// small: there is nowhere for a mistake to hide.
#[test]
fn a_round_bar_is_two_corners_three_edges_and_three_faces() {
    let model = a_round_bar();
    assert_eq!(model.vertices().count(), 2, "corners");
    assert_eq!(model.edges().count(), 3, "edges");
    assert_eq!(model.faces().count(), 3, "faces");
}

/// Roadmap §5.3's seam, which nothing in this kernel could build before.
///
/// The claim is not that some edge is used twice — every edge is — but that
/// **both uses belong to the same face**. That is what makes a surface closed
/// on itself rather than two surfaces meeting.
#[test]
fn the_seam_is_used_twice_by_the_one_face_that_wraps_round_it() {
    let model = a_round_bar();
    let wall = model
        .faces()
        .find(|face| matches!(face.surface(), Surface::Cylinder(_)))
        .expect("the bar has a wall");

    let mut walked: HashMap<_, usize> = HashMap::new();
    for wire in wall.wires() {
        for coedge in wire.coedges() {
            *walked.entry(coedge.edge().key()).or_default() += 1;
        }
    }

    assert_eq!(
        walked.values().sum::<usize>(),
        4,
        "round the bottom, up, round the top, and down again"
    );
    assert_eq!(
        walked.values().filter(|&&times| times == 2).count(),
        1,
        "exactly one edge is walked twice by this one face, and that is the seam"
    );
}

/// A closed edge starts and ends at one corner, and `validate` has to read that
/// as a whole circle rather than as an edge with no length.
#[test]
fn each_cap_circle_starts_and_ends_at_the_same_corner() {
    let model = a_round_bar();
    let closed: Vec<_> = model
        .edges()
        .filter(|edge| edge.start().key() == edge.end().key())
        .collect();
    assert_eq!(closed.len(), 2, "one circle round each cap");
}

/// Q16's invariant, asked directly rather than through `validate`.
///
/// Every corner of the curved face is *certified* onto the cylinder — not
/// within a tolerance of it, and not undecidable. It holds because the seam
/// sits at `x = radius, y = 0`, so `x² + y² − r²` is `r² + 0 − r²` and cancels
/// exactly, for any radius `f64` can hold.
#[test]
fn every_corner_of_the_wall_is_certified_onto_the_cylinder() {
    let model = a_round_bar();
    let wall = model
        .faces()
        .find(|face| matches!(face.surface(), Surface::Cylinder(_)))
        .expect("the bar has a wall");
    let Surface::Cylinder(pipe) = wall.surface() else {
        unreachable!("just matched on it")
    };

    let mut corners = 0;
    for wire in wall.wires() {
        for corner in wire.points() {
            assert_eq!(
                pipe.side_of(corner.coordinates().into()),
                Decision::Zero,
                "the corner at {corner:?} is not certified onto the wall"
            );
            corners += 1;
        }
    }
    assert_eq!(corners, 4, "four corners round the wall");
}

/// The same, for a radius that is not a whole number and not a power of two.
///
/// Nothing about the cancellation needs the radius to be tidy — `r * r` is one
/// rounding whichever `r` is, and the same rounding both times.
#[test]
fn an_untidy_radius_is_certified_onto_its_own_cylinder_too() {
    for radius in [0.1, 1.0 / 3.0, 7.937_005_259_840_998, 1e-8, 1e8] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        model
            .add_round_bar(radius, 1.0, &Isometry::identity())
            .unwrap_or_else(|why| panic!("a bar of radius {radius}: {why}"));
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("a bar of radius {radius} is sound: {why}"));
    }
}

#[test]
fn a_bar_refuses_a_height_that_is_not_a_positive_number() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    for height in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(
            model
                .add_round_bar(RADIUS, height, &Isometry::identity())
                .is_err(),
            "a bar {height} tall is not a bar"
        );
    }
}

#[test]
fn a_bar_refuses_a_radius_that_is_not_a_positive_number() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    for radius in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(
            model
                .add_round_bar(radius, HEIGHT, &Isometry::identity())
                .is_err(),
            "a bar of radius {radius} is not a bar"
        );
    }
}

/// A bar written out and read back is the same bar.
///
/// The format grew a field this milestone — an edge now says what it runs
/// along — so this is the check that the new one survives the trip.
#[test]
fn a_bar_written_out_and_read_back_is_the_same_bar() {
    let model = a_round_bar();
    let written = postcard::to_stdvec(&model).expect("a bar writes out");
    let read_back: Model = postcard::from_bytes(&written).expect("and reads back");

    read_back
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a bar that has been through the format is still a bar");
    assert_eq!(read_back.edges().count(), 3);
    assert_eq!(
        read_back
            .edges()
            .filter(|edge| edge.start().key() == edge.end().key())
            .count(),
        2,
        "both cap circles are still closed"
    );
}

/// Q11 and Q21: every operation that only handles flat faces refuses, through
/// one shared check, and says so rather than crashing or guessing.
///
/// Before the guard existed the fuse **panicked** here — `FaceView::plane`
/// unwrapped a cylinder — and the classifier silently answered as though the
/// bar were the handful of chords its corners happen to span. §4.1 ranks that
/// second one below the crash.
#[test]
fn every_operation_that_needs_flat_faces_refuses_a_round_bar() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar = model
        .add_round_bar(RADIUS, HEIGHT, &Isometry::identity())
        .expect("a bar");
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");

    assert_eq!(
        model.solid(bar).classify_point(
            Point::origin(),
            &Policy::certified(),
            &mut ProvenanceLog::new()
        ),
        Containment::NeedsFlatFaces,
        "the classifier casts rays at flat faces and says so"
    );

    for rule in [Rule::Union, Rule::Difference, Rule::Intersection] {
        for (arguments, tools) in [([bar], [cube]), ([cube], [bar])] {
            let refusal = model.fuse(
                &arguments,
                &tools,
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            assert!(
                matches!(refusal, Err(Error::FuseNeedsFlatFaces)),
                "{rule:?} with the bar on one side gave {refusal:?}"
            );
        }
    }
}

/// The corner of the bar is where the two caps' planes and the wall all meet,
/// and it is exact in every one of them.
#[test]
fn a_bar_sits_where_it_was_asked_to_sit() {
    let model = a_round_bar();
    let mut corners: Vec<Point> = model.vertices().map(|vertex| vertex.point()).collect();
    corners.sort_by(|one, other| one.coordinates()[2].total_cmp(&other.coordinates()[2]));
    assert_eq!(corners[0], Point::new(RADIUS, 0.0, 0.0));
    assert_eq!(corners[1], Point::new(RADIUS, 0.0, HEIGHT));
}
