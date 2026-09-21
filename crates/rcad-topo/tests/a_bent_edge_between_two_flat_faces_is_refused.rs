//! A circle drawn on a flat face, with a flat face on both sides of it.
//!
//! A slab's top is split in two by a circle: a disc inside it and the rest of
//! the top round it, both on the one plane `z = 2`. That is how a KiCad LED
//! arrives from a file (`docs/decisions.md` §176.2), and a plane cannot say where
//! a curve drawn on it bends — so the circle is placed by a curved surface its
//! own run names, or by nothing.
//!
//! **Named by its own plane, it used to mesh as a square.** The four arcs got no
//! points between their corners, and the mesh published a deviation that the
//! circle's middle missed by `2 − √2` of its radius. It is refused now, by name.
//! Named by its cylinder, the same circle is walked round.

use rcad_geom::{Axis, Cylinder, Deflection, EdgeRun, HalfSpace, Picker, Plane, Point, Surface};
use rcad_geom::{Orientation, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, EdgeKey, Error, Model, Result, SolidBuilder, SolidKey};

const RADIUS: f64 = 2.0;
const TOP: f64 = 2.0;

#[test]
fn a_circle_named_only_by_the_plane_it_is_drawn_on_is_refused_rather_than_meshed_as_a_square()
-> Result<()> {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(rcad_geom::Scale::millimetres(100.0)?);
    let top = the_top();
    let slab = a_slab_with_a_circle_on_top(&mut model, Surface::Plane(top))?;

    let meshed = model.solid(slab)?.mesh(
        Deflection::for_scale(model.scale()),
        &policy,
        &mut log,
    );

    assert!(
        matches!(meshed, Err(Error::AnEdgeBendsButEveryFaceAlongItIsFlat { .. })),
        "expected the circle to be refused by name, got {:?}",
        meshed.map(|mesh| mesh.points().len())
    );
    Ok(())
}

#[test]
fn the_same_circle_named_by_its_cylinder_is_walked_round() -> Result<()> {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(rcad_geom::Scale::millimetres(100.0)?);
    let round = Cylinder::new(
        Axis::new(Point::new(0.0, 0.0, TOP), Vector::new(0.0, 0.0, 1.0))?,
        RADIUS,
    )?;
    let slab = a_slab_with_a_circle_on_top(&mut model, Surface::Cylinder(round))?;
    model.validate(&policy, &mut log)?;

    let solid = model.solid(slab)?;
    let mesh = solid.mesh(Deflection::for_scale(model.scale()), &policy, &mut log)?;
    let allowed = mesh.deviation().magnitude();

    let arcs: Vec<EdgeKey> = the_arcs(&model, slab)?;
    assert_eq!(arcs.len(), 4, "the circle is four arcs");
    for arc in arcs {
        let along = mesh.points_along_edge(arc);
        assert!(!along.is_empty(), "an arc of the circle got no points: {arc:?}");
        for &at in along {
            let point = mesh.points()[at as usize];
            let [x, y, z] = point.coordinates();
            let off = ((x * x + y * y).sqrt() - RADIUS).abs().max((z - TOP).abs());
            assert!(
                off <= allowed,
                "{point:?} is {off} off the circle, past the published {allowed}"
            );
        }
    }
    Ok(())
}

fn the_top() -> Plane {
    Plane::from_point_and_normal(Point::new(0.0, 0.0, TOP), Vector::new(0.0, 0.0, 1.0))
        .expect("the z axis has a length")
}

/// The four arcs, found as the edges with no corner of the slab's box at an end.
fn the_arcs(model: &Model, slab: SolidKey) -> Result<Vec<EdgeKey>> {
    let mut arcs = Vec::new();
    for face in model.solid(slab)?.faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                let [x, y, _] = edge.start().point().coordinates();
                if (x * x + y * y).sqrt() == RADIUS && !arcs.contains(&edge.key()) {
                    arcs.push(edge.key());
                }
            }
        }
    }
    Ok(arcs)
}

/// A 10 × 10 × 2 slab whose top is a disc of radius two and the square round
/// it, the circle between them named by `and`.
fn a_slab_with_a_circle_on_top(model: &mut Model, and: Surface) -> Result<SolidKey> {
    let mut builder = SolidBuilder::new();
    let box_corner = |builder: &mut SolidBuilder, x: f64, y: f64, z: f64| {
        builder.add_vertex(Point::new(x, y, z))
    };
    let low = [
        box_corner(&mut builder, -5.0, -5.0, 0.0)?,
        box_corner(&mut builder, 5.0, -5.0, 0.0)?,
        box_corner(&mut builder, 5.0, 5.0, 0.0)?,
        box_corner(&mut builder, -5.0, 5.0, 0.0)?,
    ];
    let high = [
        box_corner(&mut builder, -5.0, -5.0, TOP)?,
        box_corner(&mut builder, 5.0, -5.0, TOP)?,
        box_corner(&mut builder, 5.0, 5.0, TOP)?,
        box_corner(&mut builder, -5.0, 5.0, TOP)?,
    ];
    let mut bottom_edges = Vec::new();
    let mut top_edges = Vec::new();
    let mut upright = Vec::new();
    for side in 0..4 {
        let next = (side + 1) % 4;
        bottom_edges.push(builder.add_edge(low[side], low[next], EdgeRun::Straight)?);
        top_edges.push(builder.add_edge(high[side], high[next], EdgeRun::Straight)?);
        upright.push(builder.add_edge(low[side], high[side], EdgeRun::Straight)?);
    }

    let on_the_circle = [
        [RADIUS, 0.0],
        [0.0, RADIUS],
        [-RADIUS, 0.0],
        [0.0, -RADIUS],
    ]
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
        steps.push(DraftStep::Along(builder.add_coedge(bottom_edges[side], reversed)?));
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
    for side in 0..4 {
        square.push(DraftStep::Along(builder.add_coedge(top_edges[side], forward)?));
    }
    let square = builder.add_wire(&square)?;
    let mut hole = Vec::new();
    for quarter in (0..4).rev() {
        hole.push(DraftStep::Along(builder.add_coedge(arcs[quarter], reversed)?));
    }
    let hole = builder.add_wire(&hole)?;
    faces.push((builder.add_face(Surface::Plane(the_top()), &[square, hole])?, forward));

    let mut disc = Vec::new();
    for quarter in 0..4 {
        disc.push(DraftStep::Along(builder.add_coedge(arcs[quarter], forward)?));
    }
    let disc = builder.add_wire(&disc)?;
    faces.push((builder.add_face(Surface::Plane(the_top()), &[disc])?, forward));

    builder.add_shell(&faces)?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}
