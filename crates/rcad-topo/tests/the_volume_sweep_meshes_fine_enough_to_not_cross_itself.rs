//! A plate with a hole near its rim, which the sweep's own coarse mesh cut
//! across.
//!
//! `docs/decisions.md` §187.2. `Model::validate` asks a curved solid whether it
//! encloses space by meshing it as coarsely as a mesh gets, and at that
//! deflection an arc is stepped a quarter turn at a time. The chord of a quarter
//! turn runs well inside the arc, and a hole sitting between the chord and the
//! arc is cut in two by it: the flat face's outer boundary crosses its hole, the
//! triangulator refuses it, and the sweep refused a sound solid for a mesh it
//! chose itself. Twenty KiCad LED files stopped there.
//!
//! The plate is half a disc of radius three, standing on the ground plane,
//! bounded by the half-turn arc over `+y` and the diameter along `x`. The hole
//! is a square of side `0.4` centred on `(1.6, 1.6)`. At the coarsest deflection
//! the arc is two chords through `(0, 3)`, and the first of them runs along
//! `x + y = 3`, which the square straddles: its corners read `2.8` and `3.6`.
//! The square's furthest corner is `1.8·√2 ≈ 2.55` from the axis, well inside
//! the rim, so the solid itself is sound.
//!
//! **Built with the builder rather than read off a file**, so the shape is the
//! whole of the finding and a file's other trouble cannot stand in for it.

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, HalfSpace, Orientation, Picker, Plane, Point, Scale,
    Surface, Vector,
};
use rcad_mesh::CannotTriangulate;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftEdge, DraftVertex, DraftWire, Error, Model, Result, SolidBuilder, SolidKey};

const RADIUS: f64 = 3.0;
const HEIGHT: f64 = 1.0;

/// The control, and what says the plate reaches its subject: meshed as
/// coarsely as a mesh gets, the rim's arc is walked into more points than the
/// one a coarse walk puts on it, because the chords through that one point cut
/// across the hole.
///
/// The same plate with its hole near the middle, clear of the chords, gets the
/// one point, `(0, 3)`. Without this the test below could pass on a plate whose
/// hole had drifted clear of the chord, and would say nothing about the sweep.
/// Before `SolidView::mesh` walked a crossed flat face's arcs again finer
/// (Q2384), the coarse mesh of the plate was refused, `BoundaryCrossesItself`
/// on the ground, and this pinned that refusal.
#[test]
fn at_the_coarsest_deflection_the_rim_is_walked_finer_where_its_chord_cuts_across_the_hole() {
    assert_eq!(
        points_the_coarsest_mesh_puts_on_the_ground_arc(Hole::ClearOfTheChord),
        1
    );
    let through_the_hole = points_the_coarsest_mesh_puts_on_the_ground_arc(Hole::InsideTheRim);
    assert!(through_the_hole > 1, "{through_the_hole} points");
}

/// How many points [`SolidView::mesh`](rcad_topo::SolidView::mesh), as coarse
/// as it gets, puts between the two ends of the arc under the plate.
fn points_the_coarsest_mesh_puts_on_the_ground_arc(hole: Hole) -> usize {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let plate =
        a_half_disc_with_a_square_hole(&mut model, &policy, &mut log, hole).expect("a sound plate");
    let solid = model.solid(plate).expect("this model made the key");
    let mesh = solid
        .mesh(
            Deflection::of(f64::MAX).expect("a positive length"),
            &policy,
            &mut log,
        )
        .expect("the plate meshes");
    let ground = solid.faces().next().expect("the plate has a ground");
    let arc = ground
        .wires()
        .flat_map(|wire| wire.coedges())
        .map(|coedge| coedge.edge())
        .find(|edge| edge.run() != EdgeRun::Straight)
        .expect("the ground's rim has an arc");
    mesh.points_along_edge(arc.key()).len()
}

/// The finding: the sweep meshes finer until the chords clear the hole, and
/// the plate is valid.
///
/// Before the sweep retried, `SolidBuilder::build` — which runs the sweep on
/// what it wrote — refused this plate with the control's refusal.
#[test]
fn a_hole_the_coarse_chord_cuts_across_does_not_stop_the_sweep() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());

    a_half_disc_with_a_square_hole(&mut model, &policy, &mut log, Hole::InsideTheRim)
        .expect("the sweep meshes finer rather than refuse a sound plate");

    model
        .validate(&policy, &mut log)
        .expect("and the full sweep says the same");
}

/// The other half of the promise: a hole that really does poke out through the
/// rim crosses it at every deflection, so no finer mesh clears it, and the
/// sweep still refuses by the coarse mesh's own name.
///
/// The square is stretched out to `[1.4, 2.3]` each way, so its far corner is
/// `2.3·√2 ≈ 3.25` from the axis — outside a rim of three — while its near
/// corner still reads `2.8` against the coarse chord's `3`. So it straddles the
/// chord at the coarsest deflection and the arc at every finer one. A hole
/// wholly outside the chord would not do: the coarse mesh then crosses nothing
/// and the sweep refuses it for its faces disagreeing instead, which is a
/// different test.
#[test]
fn a_hole_poking_out_through_the_rim_is_still_refused() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());

    let refused =
        a_half_disc_with_a_square_hole(&mut model, &policy, &mut log, Hole::ThroughTheRim);

    assert!(
        matches!(
            refused,
            Err(Error::ADraftFaceCouldNotBeMeshed {
                why: CannotTriangulate::BoundaryCrossesItself { .. }
                    | CannotTriangulate::LoopsTouch { .. },
                ..
            })
        ),
        "got {refused:?}"
    );
}

fn a_working_size() -> Scale {
    Scale::millimetres(100.0).expect("100 mm is a working size")
}

#[derive(Clone, Copy)]
enum Hole {
    /// Near the middle, clear of the coarse chords: its far corner reads `1.2`
    /// against their `3`.
    ClearOfTheChord,
    /// Straddling the coarse chord, inside the arc.
    InsideTheRim,
    /// Straddling the arc itself.
    ThroughTheRim,
}

impl Hole {
    /// The square's lowest and highest `x`, which are its `y` as well.
    fn span(self) -> (f64, f64) {
        match self {
            Hole::ClearOfTheChord => (0.2, 0.6),
            Hole::InsideTheRim => (1.4, 1.8),
            Hole::ThroughTheRim => (1.4, 2.3),
        }
    }
}

/// Half a disc of `RADIUS`, `HEIGHT` tall on the ground plane, with a square
/// hole straight through it.
///
/// The half disc is `a_face_bounded_by_two_steps.rs`'s plate: an arc over `+y`
/// and the diameter closing it. The hole's four walls face into the hole,
/// because the material is outside it.
fn a_half_disc_with_a_square_hole(
    model: &mut Model,
    policy: &Policy,
    log: &mut ProvenanceLog,
    hole: Hole,
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

    let (low, high) = hole.span();
    // Anticlockwise seen from above.
    let square = [(low, low), (high, low), (high, high), (low, high)];
    let below: [DraftVertex; 4] =
        square.map(|(x, y)| builder.add_vertex(Point::new(x, y, 0.0)).expect("finite"));
    let above: [DraftVertex; 4] = square.map(|(x, y)| {
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
        let (x, y) = square[corner];
        let (next_x, next_y) = square[next];
        // Rotating the side's direction a quarter turn anticlockwise points it
        // into the square, which is where the material is not.
        let inward = Vector::new(-(next_y - y), next_x - x, 0.0);
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
