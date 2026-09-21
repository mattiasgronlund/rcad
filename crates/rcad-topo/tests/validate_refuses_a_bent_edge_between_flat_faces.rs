//! The sweep refuses a circle drawn on a flat face that nothing curved places.
//!
//! The slab is `a_bent_edge_between_two_flat_faces_is_refused.rs`'s: a 10 × 10
//! × 2 block whose top is a disc of radius two and the square round it, both on
//! the plane `z = 2`, split by four arcs. Named only by that plane, the arcs are
//! curves whose centre and size are nowhere in the model. `SolidView::mesh`
//! refuses that since §187.1, and `Model::validate` used to pass it; the other
//! half of §176.5's item is the sweep saying so too.
//!
//! [`SolidBuilder::build`] is how a caller puts a solid in a model, and it runs
//! [`Model::validate`] over the description before writing it, so the build's
//! refusal *is* the sweep's.

use rcad_geom::{Axis, Cylinder, EdgeRun, HalfSpace, Picker, Plane, Point, Surface};
use rcad_geom::{Orientation, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Invalid, Model, Result, SolidBuilder, SolidKey};

const RADIUS: f64 = 2.0;
const TOP: f64 = 2.0;

#[test]
fn a_circle_named_only_by_the_plane_it_is_drawn_on_does_not_validate() -> Result<()> {
    let mut model = Model::new(rcad_geom::Scale::millimetres(100.0)?);

    let built = a_slab_with_a_circle_on_top(&mut model, Surface::Plane(the_top()));

    assert!(
        matches!(
            built,
            Err(Error::Invalid(
                Invalid::AnEdgeBendsButNothingAlongItIsCurved
            ))
        ),
        "expected the sweep to refuse the circle by name, got {built:?}"
    );
    assert_eq!(
        model.solids().count(),
        0,
        "a refusal leaves the model as it was"
    );
    Ok(())
}

#[test]
fn the_same_circle_named_by_its_cylinder_validates() -> Result<()> {
    let policy = Policy::certified();
    let mut model = Model::new(rcad_geom::Scale::millimetres(100.0)?);
    let round = Cylinder::new(
        Axis::new(Point::new(0.0, 0.0, TOP), Vector::new(0.0, 0.0, 1.0))?,
        RADIUS,
    )?;

    a_slab_with_a_circle_on_top(&mut model, Surface::Cylinder(round))?;

    model.validate(&policy, &mut ProvenanceLog::new())
}

fn the_top() -> Plane {
    Plane::from_point_and_normal(Point::new(0.0, 0.0, TOP), Vector::new(0.0, 0.0, 1.0))
        .expect("the z axis has a length")
}

/// A 10 × 10 × 2 slab whose top is a disc of radius two and the square round
/// it, the circle between them named by `and`.
fn a_slab_with_a_circle_on_top(model: &mut Model, and: Surface) -> Result<SolidKey> {
    let mut builder = SolidBuilder::new();
    let mut low = Vec::new();
    let mut high = Vec::new();
    for [x, y] in [[-5.0, -5.0], [5.0, -5.0], [5.0, 5.0], [-5.0, 5.0]] {
        low.push(builder.add_vertex(Point::new(x, y, 0.0))?);
        high.push(builder.add_vertex(Point::new(x, y, TOP))?);
    }
    let mut bottom_edges = Vec::new();
    let mut top_edges = Vec::new();
    let mut upright = Vec::new();
    for side in 0..4 {
        let next = (side + 1) % 4;
        bottom_edges.push(builder.add_edge(low[side], low[next], EdgeRun::Straight)?);
        top_edges.push(builder.add_edge(high[side], high[next], EdgeRun::Straight)?);
        upright.push(builder.add_edge(low[side], high[side], EdgeRun::Straight)?);
    }

    let on_the_circle = [[RADIUS, 0.0], [0.0, RADIUS], [-RADIUS, 0.0], [0.0, -RADIUS]]
        .map(|[x, y]| Point::new(x, y, TOP));
    let mut round = Vec::new();
    for corner in on_the_circle {
        round.push(builder.add_vertex(corner)?);
    }
    let mut arcs = Vec::new();
    for quarter in 0..4 {
        let next = (quarter + 1) % 4;
        let (from, to) = (on_the_circle[quarter], on_the_circle[next]);
        let facing = (to - from).cross(Vector::new(0.0, 0.0, 1.0));
        let picking = HalfSpace::through_a_point_facing(from, facing)?;
        arcs.push(builder.add_edge(
            round[quarter],
            round[next],
            EdgeRun::PartOfWhereItCrosses {
                and,
                picked_by: smallvec::smallvec![Picker::from(picking)],
            },
        )?);
    }

    let forward = Orientation::Forward;
    let reversed = Orientation::Reversed;
    let mut faces = Vec::new();

    let mut steps = Vec::new();
    for side in (0..4).rev() {
        steps.push(DraftStep::Along(
            builder.add_coedge(bottom_edges[side], reversed)?,
        ));
    }
    let wire = builder.add_wire(&steps)?;
    let floor = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))?;
    faces.push((builder.add_face(Surface::Plane(floor), &[wire])?, forward));

    for side in 0..4 {
        let next = (side + 1) % 4;
        let steps = [
            DraftStep::Along(builder.add_coedge(bottom_edges[side], forward)?),
            DraftStep::Along(builder.add_coedge(upright[next], forward)?),
            DraftStep::Along(builder.add_coedge(top_edges[side], reversed)?),
            DraftStep::Along(builder.add_coedge(upright[side], reversed)?),
        ];
        let wire = builder.add_wire(&steps)?;
        let outward = match side {
            0 => Vector::new(0.0, -1.0, 0.0),
            1 => Vector::new(1.0, 0.0, 0.0),
            2 => Vector::new(0.0, 1.0, 0.0),
            _ => Vector::new(-1.0, 0.0, 0.0),
        };
        let wall = Plane::from_point_and_normal(Point::origin() + outward * 5.0, outward)?;
        faces.push((builder.add_face(Surface::Plane(wall), &[wire])?, forward));
    }

    let mut square = Vec::new();
    for &side in &top_edges {
        square.push(DraftStep::Along(builder.add_coedge(side, forward)?));
    }
    let square = builder.add_wire(&square)?;
    let mut hole = Vec::new();
    for quarter in (0..4).rev() {
        hole.push(DraftStep::Along(
            builder.add_coedge(arcs[quarter], reversed)?,
        ));
    }
    let hole = builder.add_wire(&hole)?;
    faces.push((
        builder.add_face(Surface::Plane(the_top()), &[square, hole])?,
        forward,
    ));

    let mut disc = Vec::new();
    for &quarter in &arcs {
        disc.push(DraftStep::Along(builder.add_coedge(quarter, forward)?));
    }
    let disc = builder.add_wire(&disc)?;
    faces.push((
        builder.add_face(Surface::Plane(the_top()), &[disc])?,
        forward,
    ));

    builder.add_shell(&faces)?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}
