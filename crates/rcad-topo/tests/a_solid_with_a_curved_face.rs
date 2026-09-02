//! The first solid in this kernel that is not a polyhedron.
//!
//! A round bar is two vertices, three edges and three faces, and every one of
//! the structures M2 exists to build is in it: a face on a cylinder, a seam
//! used twice by that one face, and two closed edges that come back to the
//! corner they left. Nothing here is about bars. It is about whether those
//! three things can exist in the arena and survive a full sweep.

use std::collections::HashMap;

use rcad_geom::{Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Decision, Policy, ProvenanceLog, SideOfSurface};
use rcad_topo::{Model, Rule};

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

fn a_ball() -> Model {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    model
        .add_ball(RADIUS, &Isometry::identity())
        .expect("a ball of radius three");
    model
}

#[test]
fn a_ball_passes_the_full_sweep_with_nothing_approximated() {
    let model = a_ball();
    let mut log = ProvenanceLog::new();
    model
        .validate(&Policy::certified(), &mut log)
        .expect("a ball is a sound solid");
    assert!(
        log.records().is_empty(),
        "a ball about the origin needs nothing approximated, got {:?}",
        log.records()
    );
}

/// One face, one edge, two corners — and the two corners are the poles.
#[test]
fn a_ball_is_one_face_with_one_seam_between_two_poles() {
    let model = a_ball();
    assert_eq!(model.face_count(), 1, "one face wraps the whole ball");
    assert_eq!(model.edge_count(), 1, "the seam, and nothing else");
    assert_eq!(model.vertex_count(), 2, "the two poles");

    let mut poles: Vec<Point> = model.vertices().map(|vertex| vertex.point()).collect();
    poles.sort_by(|one, other| one.coordinates()[2].total_cmp(&other.coordinates()[2]));
    assert_eq!(poles[0], Point::new(0.0, 0.0, -RADIUS));
    assert_eq!(poles[1], Point::new(0.0, 0.0, RADIUS));
}

/// Roadmap §4.5's pole, and the reason Q19 gave it its own kind of step.
///
/// The wire has four steps and two coedges. The two that are not coedges are
/// the poles, and they are **not** edges of zero length — that shape still
/// means "broken" and nothing else, which is what keeps roadmap §7.4's
/// catalogue entry for it worth checking.
#[test]
fn the_balls_boundary_stands_still_at_each_pole() {
    let model = a_ball();
    let face = model.faces().next().expect("the ball has a face");
    let wire = face.wires().next().expect("and the face has a wire");

    assert_eq!(wire.step_count(), 4, "down, pole, up, pole");
    assert_eq!(wire.coedge_count(), 2, "and only two of those are edges");

    assert!(
        model.edges().all(|edge| !edge.is_closed()),
        "the seam is half a circle, not a whole one"
    );
    assert!(
        model
            .edges()
            .all(|edge| edge.distance_between_its_ends() > 0.0),
        "no edge of a ball has its two ends in the same place"
    );
}

/// The seam is used twice by the one face, as the bar's is — but here both
/// ends are the same two corners, which is the case Q18 is about.
#[test]
fn the_balls_seam_is_used_twice_by_the_one_face() {
    let model = a_ball();
    let face = model.faces().next().expect("the ball has a face");

    let mut walked: HashMap<_, usize> = HashMap::new();
    for wire in face.wires() {
        for coedge in wire.coedges() {
            *walked.entry(coedge.edge().key()).or_default() += 1;
        }
    }
    assert_eq!(walked.len(), 1, "one edge");
    assert_eq!(walked.values().next(), Some(&2), "walked twice");
}

/// Both poles are certified onto the sphere, exactly.
#[test]
fn both_poles_are_certified_onto_the_ball() {
    for radius in [RADIUS, 0.1, 1.0 / 3.0, 1e-8, 1e8] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        model
            .add_ball(radius, &Isometry::identity())
            .unwrap_or_else(|why| panic!("a ball of radius {radius}: {why}"));

        let face = model.faces().next().expect("the ball has a face");
        let Surface::Sphere(skin) = face.surface() else {
            panic!("a ball's face sits on a sphere")
        };
        for corner in face.wires().flat_map(|wire| wire.points()) {
            assert_eq!(
                skin.side_of(corner.coordinates().into()),
                Decision::Zero,
                "the pole at {corner:?} on a ball of radius {radius}"
            );
        }

        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("a ball of radius {radius} is sound: {why}"));
    }
}

#[test]
fn a_ball_written_out_and_read_back_is_the_same_ball() {
    let model = a_ball();
    let written = postcard::to_stdvec(&model).expect("a ball writes out");
    let read_back: Model = postcard::from_bytes(&written).expect("and reads back");

    read_back
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a ball that has been through the format is still a ball");

    let wire = read_back
        .faces()
        .next()
        .expect("still one face")
        .wires()
        .next()
        .expect("still one wire");
    assert_eq!(wire.step_count(), 4, "the poles survived the trip");
    assert_eq!(wire.coedge_count(), 2);
}

#[test]
fn a_ball_refuses_a_radius_that_is_not_a_positive_number() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    for radius in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(
            model.add_ball(radius, &Isometry::identity()).is_err(),
            "a ball of radius {radius} is not a ball"
        );
    }
}

/// Q11 and Q21's list is empty, and this is the entry that closes it.
///
/// Before the guard existed the fuse **panicked** here — `FaceView::plane`
/// unwrapped a cylinder — and the classifier silently answered as though the
/// bar were the handful of chords its corners happen to span. §4.1 ranks that
/// second one below the crash.
///
/// **Three operations have come off this list, in three different milestones,
/// and the pattern is worth naming.** `measure` refused a curved solid at M2's
/// gate because a bar has no fan to sum (§25.5); it sums over a mesh now, and
/// `measuring.rs` is where that is checked. **The classifier came off at M4**:
/// it fires rays at curved surfaces and asks each face what it holds, so what
/// used to be `Containment::NeedsFlatFaces` is a real answer now and
/// `point_in_a_curved_solid.rs` is where it is asserted (§35). **And the fuse
/// came off at the end of M4**, which is the whole of what that milestone was
/// (§31.1): `Error::FuseNeedsFlatFaces` is gone and there is no variant left to
/// assert on, so what stands in its place is the operation working.
#[test]
fn the_fuse_takes_a_curved_solid_now() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let block = model
        .add_cube(8.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a block eight across");
    let bar = model
        .add_round_bar(
            1.0,
            12.0,
            &Isometry::translation(Vector::new(4.0, 4.0, -2.0)).expect("a shift"),
        )
        .expect("a bar through the middle of it");

    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(
            &[block],
            &[bar],
            Rule::Difference,
            &Policy::certified(),
            &mut log,
        )
        .expect("a block with a round hole taken out of it");

    assert_eq!(fused.solids().len(), 1, "a block with a hole is one solid");
    assert!(
        log.records().is_empty(),
        "a lattice-aligned curved pair needs nothing approximated, got {:?}",
        log.records()
    );
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the answer is a sound model");
}

/// A box round the corners is not a box round a curved solid, and the name has
/// to keep saying so.
///
/// This used to be called `bounding_box`, and it was one until a curved face
/// existed. A bar's two corners are both on its seam, so the box round them is
/// a line the bar bulges a whole radius out of; a ball's are its poles, and the
/// box round those misses the equator entirely. Anything that took the old name
/// at its word got a confident wrong answer, which `docs/decisions.md` §4.1
/// ranks below a crash.
#[test]
fn the_box_round_a_curved_solids_corners_does_not_hold_the_solid() {
    let bar = a_round_bar();
    let (low, high) = bar
        .solids()
        .next()
        .expect("the bar")
        .box_around_its_corners()
        .expect("the bar has corners");
    assert_eq!(low, Point::new(RADIUS, 0.0, 0.0));
    assert_eq!(high, Point::new(RADIUS, 0.0, HEIGHT));
    assert!(
        low.coordinates()[0] > -RADIUS,
        "the far side of the wall is at x = -{RADIUS} and this box starts at \
         x = {}",
        low.coordinates()[0]
    );

    // A ball is worse than wrong: its one face has two corners and two corners
    // make no triangle, so there is no box at all. `None` used to mean "no
    // faces", and a ball has one.
    let ball = a_ball();
    assert_eq!(
        ball.solids()
            .next()
            .expect("the ball")
            .box_around_its_corners(),
        None
    );
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

/// A box round one face of a curved solid, which is what a broad phase reads
/// and what the fuse's baking step used to work out from triangles.
///
/// The wall of a bar fans into two triangles either side of its seam, and both
/// of them are *on* the seam — a straight line the wall bulges a whole radius
/// out of. The cap fans into nothing at all, because a wire that closes on one
/// edge has one corner and one corner is no triangle. So the old box round the
/// wall was a line and the old box round a cap was empty, and a pair of faces
/// skipped on either would leave one of two faces along an edge split and the
/// other not.
#[test]
fn a_box_round_one_curved_face_holds_the_face_and_the_box_round_its_triangles_does_not() {
    let model = a_round_bar();
    let bar = model.solids().next().expect("the bar");

    for face in bar.faces() {
        let (low, high) = bar
            .box_around_one_face(face)
            .expect("every face of a bar is somewhere");
        let mut round_its_triangles = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
        face.for_each_triangle(|triangle| {
            for corner in triangle {
                for axis in 0..3 {
                    round_its_triangles.0[axis] =
                        round_its_triangles.0[axis].min(corner.coordinates()[axis]);
                    round_its_triangles.1[axis] =
                        round_its_triangles.1[axis].max(corner.coordinates()[axis]);
                }
            }
        });

        match face.surface() {
            // The corners' shadows on the axis are `z = 0` and `z = 5`, opened
            // out by the radius on every axis — so the box is taller than the
            // bar. Loose and sound, which is what a box that only decides which
            // pairs to look at has to be.
            Surface::Cylinder(_) => {
                assert_eq!(low, Point::new(-RADIUS, -RADIUS, -RADIUS));
                assert_eq!(high, Point::new(RADIUS, RADIUS, HEIGHT + RADIUS));
                assert_eq!(
                    round_its_triangles.0,
                    [RADIUS, 0.0, 0.0],
                    "the fan of a wall is its seam, and the seam is at x = {RADIUS}"
                );
            }
            // A cap's circle is held by the wall it is shared with.
            Surface::Plane(_) => {
                for axis in 0..2 {
                    assert!(low.coordinates()[axis] <= -RADIUS);
                    assert!(high.coordinates()[axis] >= RADIUS);
                }
                assert!(low.coordinates()[2].is_finite() && high.coordinates()[2].is_finite());
                assert_eq!(
                    round_its_triangles.0,
                    [f64::INFINITY; 3],
                    "a round cap fans into no triangles at all"
                );
            }
            Surface::Cone(cone) => panic!("a bar has no {cone:?}"),
            Surface::Torus(ring) => panic!("a bar has no {ring:?}"),
            other => panic!("a bar has no {other:?}"),
        }
    }
}

/// The box round the whole solid is the boxes round its faces put together, so
/// the two cannot drift apart.
#[test]
fn the_box_round_a_bar_is_the_boxes_round_its_faces_put_together() {
    let model = a_round_bar();
    let bar = model.solids().next().expect("the bar");
    let mut low = [f64::INFINITY; 3];
    let mut high = [f64::NEG_INFINITY; 3];
    for face in bar.faces() {
        let (mine, theirs) = bar
            .box_around_one_face(face)
            .expect("every face of a bar is somewhere");
        for axis in 0..3 {
            low[axis] = low[axis].min(mine.coordinates()[axis]);
            high[axis] = high[axis].max(theirs.coordinates()[axis]);
        }
    }
    assert_eq!(
        bar.box_around_it(),
        Some((Point::from(low), Point::from(high)))
    );
}
