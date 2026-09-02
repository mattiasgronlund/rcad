//! Getting about: solid to shell to face to wire to coedge to edge to vertex,
//! and back up by key.

use rcad_geom::{Isometry, Orientation, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, SolidKey};

fn a_cube_of_side_two() -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    (model, cube)
}

#[test]
fn walking_down_from_a_solid_reaches_every_corner_of_the_cube() {
    let (model, cube) = a_cube_of_side_two();

    let mut corners: Vec<[f64; 3]> = Vec::new();
    for shell in model.solid(cube).shells() {
        for face in shell.faces() {
            for wire in face.wires() {
                for coedge in wire.coedges() {
                    corners.push(coedge.start().point().coordinates());
                }
            }
        }
    }
    corners.sort_by(|a, b| a.partial_cmp(b).expect("no coordinate is a NaN"));
    corners.dedup();

    assert_eq!(corners.len(), 8);
}

#[test]
fn every_coedge_ends_where_the_next_one_starts() {
    let (model, cube) = a_cube_of_side_two();

    for face in model.solid(cube).faces() {
        for wire in face.wires() {
            let points: Vec<Point> = wire.coedges().map(|coedge| coedge.end().point()).collect();
            let starts: Vec<Point> = wire
                .coedges()
                .map(|coedge| coedge.start().point())
                .collect();
            for (position, end) in points.iter().enumerate() {
                assert_eq!(*end, starts[(position + 1) % starts.len()]);
            }
        }
    }
}

#[test]
fn the_shell_of_a_cube_hands_out_a_reference_for_every_face() {
    let (model, cube) = a_cube_of_side_two();
    let shell = model
        .solid(cube)
        .shells()
        .next()
        .expect("a cube has a skin");

    let refs: Vec<_> = shell.face_refs().collect();

    assert_eq!(refs.len(), shell.face_count());
    for (reference, face) in refs.iter().zip(shell.faces()) {
        assert_eq!(reference.face(), face.key());
        assert_eq!(reference.location(), Isometry::identity());
        assert_eq!(reference.orientation(), Orientation::Forward);
    }
}

#[test]
fn a_reference_carries_the_placement_it_was_made_with() {
    let (model, cube) = a_cube_of_side_two();
    let face = model.solid(cube).faces().next().expect("a cube has faces");
    let shift = Isometry::translation(Vector::new(1.0, 2.0, 3.0)).expect("a finite shift");

    let reference = rcad_topo::FaceRef::new(face.key(), shift, Orientation::Reversed);

    assert_eq!(reference.face(), face.key());
    assert_eq!(reference.location(), shift);
    assert_eq!(reference.orientation(), Orientation::Reversed);
}

#[test]
fn every_edge_of_a_cube_of_side_two_is_two_long() {
    let (model, _) = a_cube_of_side_two();

    for edge in model.edges() {
        assert_eq!(edge.distance_between_its_ends(), 2.0);
        let segment = edge
            .segment()
            .expect("a cube's edges are all finite and distinct")
            .expect("a cube has no closed edges");
        assert_eq!(segment.length(), 2.0);
        assert_eq!(segment.start(), edge.start().point());
        assert_eq!(segment.end(), edge.end().point());
    }
}

/// `EdgeView::segment` used to panic on exactly this: a round bar's two cap
/// circles are closed edges with `start == end` by definition, and
/// `Model::validate` accepts that — a closed edge is *supposed* to come
/// back to where it started, so this is the commonest closed edge the
/// kernel builds, not a broken model.
#[test]
fn a_round_bars_closed_cap_edges_have_no_segment_and_do_not_panic() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    model
        .add_round_bar(3.0, 5.0, &Isometry::identity())
        .expect("a bar three across and five tall");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a round bar this kernel built is sound");

    let mut closed = 0;
    for edge in model.edges() {
        let segment = edge.segment().expect("a round bar's edges are all finite");
        assert_eq!(segment.is_none(), edge.is_closed());
        if edge.is_closed() {
            closed += 1;
        }
    }
    assert_eq!(
        closed, 2,
        "a round bar has exactly two closed edges: its two cap circles"
    );
}

#[test]
fn a_key_taken_from_a_view_leads_back_to_the_same_thing() {
    let (model, cube) = a_cube_of_side_two();
    let face = model.solid(cube).faces().next().expect("a cube has faces");

    let wire = face.outer_wire();
    assert_eq!(wire.face(), Some(face.key()));
    assert_eq!(model.wire(wire.key()).coedge_count(), wire.coedge_count());

    let coedge = wire.coedges().next().expect("a wire has coedges");
    assert_eq!(coedge.wire(), Some(wire.key()));
    assert_eq!(
        model.coedge(coedge.key()).start().point(),
        coedge.start().point()
    );

    let vertex = coedge.start();
    assert_eq!(model.vertex(vertex.key()).point(), vertex.point());
    assert_eq!(model.vertex(vertex.key()).edges().count(), 3);

    let shell = model
        .solid(cube)
        .shells()
        .next()
        .expect("a cube has a skin");
    assert_eq!(model.shell(shell.key()).face_count(), 6);
    assert_eq!(model.solid(cube).key(), cube);
}

#[test]
fn a_face_broken_into_triangles_covers_its_whole_area() {
    let (model, cube) = a_cube_of_side_two();

    for face in model.solid(cube).faces() {
        let mut area = Vector::zero();
        face.for_each_triangle(|[a, b, c]| {
            area = area + (b - a).cross(c - a) * 0.5;
        });
        assert_eq!(area.length(), 4.0);
        assert!(area.dot(face.outward_normal()) > 0.0);
    }
}

#[test]
fn a_cube_fits_inside_the_box_round_its_own_corners() {
    let (model, cube) = a_cube_of_side_two();

    let (low, high) = model
        .solid(cube)
        .box_around_its_corners()
        .expect("a cube has corners");

    assert_eq!(low, Point::new(0.0, 0.0, 0.0));
    assert_eq!(high, Point::new(2.0, 2.0, 2.0));
}
