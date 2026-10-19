//! When the sweep's coarse mesh crosses on one face and a finer mesh crosses on
//! another, the refusal names the face a finer mesh still cannot get past.
//!
//! `docs/decisions.md` §192.2. `Model::validate` meshes a curved solid as
//! coarsely as a mesh gets, and when that mesh's sides cross it halves the
//! deflection and asks again. It used to hand back the **coarse** refusal when
//! no finer mesh cleared it. On an LED file that named the disc, which meshes
//! at every finer deflection, while the dome was what stopped the file.
//!
//! The plate is `the_volume_sweep_meshes_fine_enough_to_not_cross_itself.rs`'s
//! half disc of radius three, with a hole whose walls lean outward. At the
//! ground the hole is the square `[1.4, 1.8]` each way, inside the rim but
//! straddling the coarse chord `x + y = 3`. At the lid it is `[1.4, 2.3]`, and
//! its far corner `(2.3, 2.3)` is `3.25` from the axis, out through the rim. So
//! the ground crosses at the coarsest deflection only, and the lid crosses at
//! every one. The ground comes first in the shell, so it was the face the
//! coarse mesh named.
//!
//! **Since Q2384 the mesh itself walks a crossed flat face's arcs again,
//! finer**, so the coarse mesh clears the ground and names the lid on its own.
//! The finding below still says what the sweep hands back, but it no longer
//! tells the sweep's finest refusal from its coarse one: both are the lid's.
//!
//! **Built with the builder rather than read off a file**, so the shape is the
//! whole of the finding.

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, HalfSpace, Orientation, Picker, Plane, Point, Scale,
    Surface, Vector,
};
use rcad_mesh::CannotTriangulate;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftEdge, DraftVertex, DraftWire, Error, Model, Result, SolidBuilder, SolidKey};

const RADIUS: f64 = 3.0;
const HEIGHT: f64 = 1.0;

/// The position the ground was handed to the builder at.
const GROUND: usize = 0;
/// The position the lid was handed to the builder at.
const LID: usize = 1;

/// The finding: the lid is named, because the ground meshes as soon as the
/// deflection is fine enough and the lid never does.
#[test]
fn a_hole_leaning_out_through_the_rim_is_refused_by_the_lid() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());

    let refused = a_half_disc_with_a_leaning_hole(&mut model, &policy, &mut log);

    assert!(
        matches!(
            refused,
            Err(Error::ADraftFaceCouldNotBeMeshed {
                face: LID,
                why: CannotTriangulate::BoundaryCrossesItself { .. }
                    | CannotTriangulate::LoopsTouch { .. },
                ..
            })
        ),
        "got {refused:?}"
    );
}

/// The control: the plate reaches its subject. Built with the lid's hole
/// straight up, so nothing leans out, the plate is sound; and meshed as
/// coarsely as a mesh gets, the ground's arc is walked into more than the one
/// point a coarse walk puts on a half turn, because the ground's sides crossed
/// at that one point and the mesh walked them again finer.
///
/// Without this the test above could pass on a plate whose ground had drifted
/// clear of the coarse chord, where the lid would be the only face that ever
/// crossed. Before `SolidView::mesh` walked a crossed flat face's arcs again
/// finer (Q2384), the coarse mesh refused the ground, and this pinned that
/// refusal.
#[test]
fn at_the_coarsest_deflection_the_grounds_arc_is_walked_finer_where_it_crosses() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let plate = a_half_disc_with_a_hole(&mut model, &policy, &mut log, (1.4, 1.8))
        .expect("a hole inside the rim at the lid too is a sound plate");
    let solid = model.solid(plate).expect("this model made the key");

    let mesh = solid
        .mesh(
            Deflection::of(f64::MAX).expect("a positive length"),
            &policy,
            &mut log,
        )
        .expect("the plate meshes");

    let ground = solid.faces().nth(GROUND).expect("the plate has a ground");
    let arc = ground
        .wires()
        .flat_map(|wire| wire.coedges())
        .map(|coedge| coedge.edge())
        .find(|edge| edge.run() != EdgeRun::Straight)
        .expect("the ground's rim has an arc");
    let walked_into = mesh.points_along_edge(arc.key()).len();
    assert!(walked_into > 1, "{walked_into} points");
}

fn a_working_size() -> Scale {
    Scale::millimetres(100.0).expect("100 mm is a working size")
}

fn a_half_disc_with_a_leaning_hole(
    model: &mut Model,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SolidKey> {
    a_half_disc_with_a_hole(model, policy, log, (1.4, 2.3))
}

/// Half a disc of `RADIUS`, `HEIGHT` tall on the ground plane, with a hole
/// through it that is the square `[1.4, 1.8]` each way at the ground and
/// `[1.4, lid_span.1]` at the lid.
///
/// Each wall of the hole joins two parallel sides, so it is flat however far
/// it leans. The walls face into the hole, because the material is outside it.
fn a_half_disc_with_a_hole(
    model: &mut Model,
    policy: &Policy,
    log: &mut ProvenanceLog,
    lid_span: (f64, f64),
) -> Result<SolidKey> {
    let mut builder = SolidBuilder::new();

    let east = builder.add_vertex(Point::new(RADIUS, 0.0, 0.0))?;
    let west = builder.add_vertex(Point::new(-RADIUS, 0.0, 0.0))?;
    let east_above = builder.add_vertex(Point::new(RADIUS, 0.0, HEIGHT))?;
    let west_above = builder.add_vertex(Point::new(-RADIUS, 0.0, HEIGHT))?;

    let ground = a_plane_at(0.0, Vector::new(0.0, 0.0, -1.0));
    let lid = a_plane_at(HEIGHT, Vector::new(0.0, 0.0, 1.0));
    let arc_below = builder.add_edge(east, west, the_arc_in(&ground))?;
    let arc_above = builder.add_edge(east_above, west_above, the_arc_in(&lid))?;
    let chord_below = builder.add_edge(east, west, EdgeRun::Straight)?;
    let chord_above = builder.add_edge(east_above, west_above, EdgeRun::Straight)?;
    let up_east = builder.add_edge(east, east_above, EdgeRun::Straight)?;
    let up_west = builder.add_edge(west, west_above, EdgeRun::Straight)?;

    // Anticlockwise seen from above.
    let square = |(low, high): (f64, f64)| [(low, low), (high, low), (high, high), (low, high)];
    let square_below = square((1.4, 1.8));
    let square_above = square(lid_span);
    let below: [DraftVertex; 4] =
        square_below.map(|(x, y)| builder.add_vertex(Point::new(x, y, 0.0)).expect("finite"));
    let above: [DraftVertex; 4] = square_above.map(|(x, y)| {
        builder
            .add_vertex(Point::new(x, y, HEIGHT))
            .expect("finite")
    });
    let mut around_below = Vec::with_capacity(4);
    let mut around_above = Vec::with_capacity(4);
    let mut up = Vec::with_capacity(4);
    for corner in 0..4 {
        let next = (corner + 1) % 4;
        around_below.push(builder.add_edge(below[corner], below[next], EdgeRun::Straight)?);
        around_above.push(builder.add_edge(above[corner], above[next], EdgeRun::Straight)?);
        up.push(builder.add_edge(below[corner], above[corner], EdgeRun::Straight)?);
    }

    let ground_rim = a_wire(
        &mut builder,
        &[
            (arc_below, Orientation::Reversed),
            (chord_below, Orientation::Forward),
        ],
    )?;
    // The ground faces down, so its hole runs anticlockwise seen from above.
    let ground_hole = a_wire(
        &mut builder,
        &around_below
            .iter()
            .map(|&edge| (edge, Orientation::Forward))
            .collect::<Vec<_>>(),
    )?;
    let lid_rim = a_wire(
        &mut builder,
        &[
            (arc_above, Orientation::Forward),
            (chord_above, Orientation::Reversed),
        ],
    )?;
    let lid_hole = a_wire(
        &mut builder,
        &around_above
            .iter()
            .rev()
            .map(|&edge| (edge, Orientation::Reversed))
            .collect::<Vec<_>>(),
    )?;
    let wall = a_wire(
        &mut builder,
        &[
            (arc_below, Orientation::Forward),
            (up_west, Orientation::Forward),
            (arc_above, Orientation::Reversed),
            (up_east, Orientation::Reversed),
        ],
    )?;
    let flat_side = a_wire(
        &mut builder,
        &[
            (chord_below, Orientation::Reversed),
            (up_east, Orientation::Forward),
            (chord_above, Orientation::Forward),
            (up_west, Orientation::Reversed),
        ],
    )?;

    let mut faces = vec![
        builder.add_face(Surface::Plane(ground), &[ground_rim, ground_hole])?,
        builder.add_face(Surface::Plane(lid), &[lid_rim, lid_hole])?,
        builder.add_face(
            Surface::Cylinder(Cylinder::new(Axis::z(), RADIUS).expect("a positive radius")),
            &[wall],
        )?,
        builder.add_face(
            Surface::Plane(
                Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, -1.0, 0.0))
                    .expect("an axis direction has length"),
            ),
            &[flat_side],
        )?,
    ];
    for corner in 0..4 {
        let next = (corner + 1) % 4;
        let side = a_wire(
            &mut builder,
            &[
                (around_below[corner], Orientation::Reversed),
                (up[corner], Orientation::Forward),
                (around_above[corner], Orientation::Forward),
                (up[next], Orientation::Reversed),
            ],
        )?;
        let (x, y) = square_below[corner];
        let (next_x, next_y) = square_below[next];
        let (x_above, y_above) = square_above[corner];
        let along = Vector::new(next_x - x, next_y - y, 0.0);
        let rising = Vector::new(x_above - x, y_above - y, HEIGHT);
        // With the wall upright, `rising × along` is `along` turned a quarter
        // anticlockwise, which points into the square, where the material is not.
        let inward = rising.cross(along);
        let plane =
            Plane::from_point_and_normal(Point::new(x, y, 0.0), inward).expect("a side has length");
        faces.push(builder.add_face(Surface::Plane(plane), &[side])?);
    }

    let shell: Vec<_> = faces
        .into_iter()
        .map(|face| (face, Orientation::Forward))
        .collect();
    builder.add_shell(&shell)?;
    builder.build(model, policy, log)
}

fn a_plane_at(height: f64, facing: Vector) -> Plane {
    Plane::from_point_and_normal(Point::new(0.0, 0.0, height), facing)
        .expect("an axis direction has length")
}

/// The half of the circle where `plane` cuts the wall, picked out by `y ≥ 0`.
fn the_arc_in(plane: &Plane) -> EdgeRun {
    EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(*plane),
        picked_by: smallvec::smallvec![Picker::from(
            HalfSpace::through_a_point_facing(Point::origin(), Vector::new(0.0, 1.0, 0.0))
                .expect("an axis direction has length"),
        )],
    }
}

fn a_wire(builder: &mut SolidBuilder, steps: &[(DraftEdge, Orientation)]) -> Result<DraftWire> {
    let mut walked = Vec::with_capacity(steps.len());
    for &(edge, orientation) in steps {
        walked.push(rcad_topo::DraftStep::Along(
            builder.add_coedge(edge, orientation)?,
        ));
    }
    builder.add_wire(&walked)
}
