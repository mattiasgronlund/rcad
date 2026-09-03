//! A flat face whose whole area is in how its two arcs bulge.
//!
//! The half of §44.11 item 3's winding question `a_face_bounded_by_two_steps.rs`
//! left open. A wire of two steps has two corners, and two corners sweep exactly
//! zero however the face is turned, so the sweep stopped asking about them. This
//! is the shape behind that: **four corners, all on one line**. There are enough
//! of them to be a polygon, the sweep does ask, and the polygon still sweeps
//! nothing — the area is all in the two half-circles.
//!
//! The plate is the upper half of a disc of `OUTER` radius with the lower half
//! of a disc of `INNER` radius hung under it, standing `HEIGHT` tall. Both arcs
//! bulge away from the `x` axis they are cut by, so both walls are cylinders
//! with their material inside them and no face is turned round. It encloses
//! `π(OUTER² + INNER²)·HEIGHT/2`, and every corner sits at `(±R, 0, z)` where
//! `x² + y² − r²` cancels to nothing.

use rcad_geom::{
    Axis, Cylinder, EdgeRun, HalfSpace, Orientation, Picker, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftEdge, DraftStep, DraftWire, Model, Result, SolidBuilder, SolidKey};

const OUTER: f64 = 3.0;
const INNER: f64 = 1.0;
const HEIGHT: f64 = 5.0;

/// The sentence this lane earns its fix with.
///
/// Every one of this plate's four flat corners sits on the `x` axis, so the
/// polygon through them sweeps exactly zero and the winding this face is judged
/// by has to come from somewhere else. Before the fix this was
/// [`Invalid::OuterWireWindsAgainstTheNormal`](rcad_topo::Invalid), raised by
/// `SolidBuilder::build` and again by `Model::validate`. Now the winding is read
/// off the boundary the wire actually traces — its corners with the points
/// `rcad_geom::walk` puts along each arc — so the area is where the shape has
/// it.
#[test]
fn a_plate_bounded_by_two_half_circles_is_a_sound_solid() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());

    a_plate_bounded_by_two_half_circles(&mut model, &policy, &mut log)
        .expect("two half-circles and two straights bound a face");

    model
        .validate(&policy, &mut log)
        .expect("and the full sweep is happy with it");
}

/// That it passed for the right reason: the plate really does enclose what its
/// two half-circles say, so the winding the sweep now reads is the winding the
/// shape has.
#[test]
fn the_plate_encloses_what_its_two_half_circles_say() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let plate = a_plate_bounded_by_two_half_circles(&mut model, &policy, &mut log)
        .expect("two half-circles and two straights bound a face");

    let measured = model
        .solid(plate)
        .measure(
            rcad_geom::Deflection::of(0.001).expect("a fine mesh"),
            &policy,
            &mut log,
        )
        .expect("a plate measures");

    let space = core::f64::consts::PI * (OUTER * OUTER + INNER * INNER) / 2.0 * HEIGHT;
    assert!(
        (measured.volume() - space).abs() <= measured.volume_error(),
        "the plate encloses {space} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// The four flat corners are on one line, which is what makes this face worth a
/// test rather than a rerun of the quarter-disc's.
#[test]
fn the_plates_four_flat_corners_are_all_on_the_x_axis() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let plate = a_plate_bounded_by_two_half_circles(&mut model, &policy, &mut log)
        .expect("two half-circles and two straights bound a face");

    let flat: Vec<Vec<[f64; 3]>> = model
        .solid(plate)
        .shells()
        .flat_map(|shell| shell.faces())
        .filter(|face| matches!(face.surface(), Surface::Plane(flat) if flat.normal().components()[2] != 0.0))
        .map(|face| {
            face.outer_wire()
                .points()
                .map(|corner| corner.coordinates())
                .collect()
        })
        .collect();

    assert_eq!(flat.len(), 2, "a top and a bottom");
    for corners in flat {
        assert_eq!(corners.len(), 4, "four corners: {corners:?}");
        assert!(
            corners.iter().all(|corner| corner[1] == 0.0),
            "every corner on the x axis: {corners:?}"
        );
    }
}

fn a_working_size() -> Scale {
    Scale::millimetres(100.0).expect("100 mm is a working size")
}

/// The plate, face by face.
///
/// Eight corners, twelve edges, six faces: the flat top and bottom, a wall for
/// each half-circle, and the rectangle each straight end sweeps out.
fn a_plate_bounded_by_two_half_circles(
    model: &mut Model,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SolidKey> {
    let mut builder = SolidBuilder::new();

    let ground = a_plane_at(0.0, Vector::new(0.0, 0.0, -1.0));
    let lid = a_plane_at(HEIGHT, Vector::new(0.0, 0.0, 1.0));

    let corners = [
        (OUTER, 0.0),
        (INNER, 0.0),
        (-INNER, 0.0),
        (-OUTER, 0.0),
        (OUTER, HEIGHT),
        (INNER, HEIGHT),
        (-INNER, HEIGHT),
        (-OUTER, HEIGHT),
    ];
    let mut at = Vec::with_capacity(corners.len());
    for (x, z) in corners {
        at.push(builder.add_vertex(Point::new(x, 0.0, z))?);
    }

    let edges = [
        (0, 3, an_arc_over(&ground)),
        (3, 2, EdgeRun::Straight),
        (2, 1, an_arc_under(&ground)),
        (1, 0, EdgeRun::Straight),
        (4, 7, an_arc_over(&lid)),
        (7, 6, EdgeRun::Straight),
        (6, 5, an_arc_under(&lid)),
        (5, 4, EdgeRun::Straight),
        (0, 4, EdgeRun::Straight),
        (1, 5, EdgeRun::Straight),
        (2, 6, EdgeRun::Straight),
        (3, 7, EdgeRun::Straight),
    ];
    let mut edge = Vec::with_capacity(edges.len());
    for (from, to, run) in edges {
        edge.push(builder.add_edge(at[from], at[to], run)?);
    }

    let walked = |builder: &mut SolidBuilder, steps: &[(usize, Orientation)]| {
        let steps: Vec<(DraftEdge, Orientation)> = steps
            .iter()
            .map(|&(which, orientation)| (edge[which], orientation))
            .collect();
        a_wire(builder, &steps)
    };

    let below = walked(
        &mut builder,
        &[
            (3, Orientation::Reversed),
            (2, Orientation::Reversed),
            (1, Orientation::Reversed),
            (0, Orientation::Reversed),
        ],
    )?;
    let above = walked(
        &mut builder,
        &[
            (4, Orientation::Forward),
            (5, Orientation::Forward),
            (6, Orientation::Forward),
            (7, Orientation::Forward),
        ],
    )?;
    let over = walked(
        &mut builder,
        &[
            (0, Orientation::Forward),
            (11, Orientation::Forward),
            (4, Orientation::Reversed),
            (8, Orientation::Reversed),
        ],
    )?;
    let under = walked(
        &mut builder,
        &[
            (2, Orientation::Forward),
            (9, Orientation::Forward),
            (6, Orientation::Reversed),
            (10, Orientation::Reversed),
        ],
    )?;
    let near_end = walked(
        &mut builder,
        &[
            (3, Orientation::Forward),
            (8, Orientation::Forward),
            (7, Orientation::Reversed),
            (9, Orientation::Reversed),
        ],
    )?;
    let far_end = walked(
        &mut builder,
        &[
            (1, Orientation::Forward),
            (10, Orientation::Forward),
            (5, Orientation::Reversed),
            (11, Orientation::Reversed),
        ],
    )?;

    let cut = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, -1.0, 0.0))
            .expect("an axis direction has length"),
    );
    let faces = [
        builder.add_face(Surface::Plane(ground), &[below])?,
        builder.add_face(Surface::Plane(lid), &[above])?,
        builder.add_face(a_pipe_of_radius(OUTER), &[over])?,
        builder.add_face(a_pipe_of_radius(INNER), &[under])?,
        builder.add_face(cut, &[near_end])?,
        builder.add_face(cut, &[far_end])?,
    ];

    let skin: Vec<_> = faces
        .into_iter()
        .map(|face| (face, Orientation::Forward))
        .collect();
    builder.add_shell(&skin)?;
    builder.build(model, policy, log)
}

fn a_plane_at(height: f64, facing: Vector) -> Plane {
    Plane::from_point_and_normal(Point::new(0.0, 0.0, height), facing)
        .expect("an axis direction has length")
}

/// The half of the circle `plane` cuts that lies over the `x` axis.
fn an_arc_over(plane: &Plane) -> EdgeRun {
    an_arc_on(plane, 1.0)
}

/// And the half under it.
fn an_arc_under(plane: &Plane) -> EdgeRun {
    an_arc_on(plane, -1.0)
}

fn an_arc_on(plane: &Plane, side: f64) -> EdgeRun {
    EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(*plane),
        picked_by: smallvec::smallvec![Picker::from(
            HalfSpace::through_a_point_facing(Point::origin(), Vector::new(0.0, side, 0.0))
                .expect("an axis direction has length")
        )],
    }
}

fn a_pipe_of_radius(radius: f64) -> Surface {
    Surface::Cylinder(Cylinder::new(Axis::z(), radius).expect("a positive radius"))
}

fn a_wire(builder: &mut SolidBuilder, steps: &[(DraftEdge, Orientation)]) -> Result<DraftWire> {
    let mut walked: Vec<DraftStep> = Vec::with_capacity(steps.len());
    for &(edge, orientation) in steps {
        walked.push(DraftStep::Along(builder.add_coedge(edge, orientation)?));
    }
    builder.add_wire(&walked)
}
