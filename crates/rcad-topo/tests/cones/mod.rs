//! Cone-faced solids described through the public builder, for the test files
//! that need one.
//!
//! Two shapes, and the difference between them is the whole of what a cone adds
//! to this kernel.
//!
//! A **frustum** — a cone with its tip cut off — has the same structure as a
//! round bar: two cap circles, a seam up one ruling, and one face on the curved
//! surface. Nothing about it is singular.
//!
//! A **pointed cone** ends at its apex, where the surface pinches to a point and
//! has no normal. That is the same species of shape as a ball's pole, and it is
//! answered with the same machinery: the mesher's `where_this_surface_pinches`
//! says a cone pinches at `v = 0`, and `cut_a_cap_up_to_its_pole` cuts the one
//! loop of the face up to it exactly as it cuts a ball's cap. **A cone's apex
//! needs no seam and no `WireStep::Pole`**, which is where it differs from a
//! ball: a ball's face goes right round *both* coordinates and has to be cut
//! open along a meridian to be a rectangle at all, while a cone's face is
//! bounded below by its own base circle and the apex is one end of the chart
//! rather than a hole in it. So a pointed cone is two faces, one edge and one
//! vertex, and the apex is not a vertex of the model.
//!
//! # Every cone here has a power-of-two slope, on purpose
//!
//! `Model::validate` asks that every corner of a curved face be **certified** on
//! its surface — no wobble allowed. A bar gets that for free: its corner sits at
//! `(r, 0, z)` and `x² + y² − r²` cancels whatever `r` is. A cone's corner has to
//! satisfy `radius = slope · height`, a relation between three stored numbers, so
//! one of them is a product that has to round to nothing. A power-of-two slope
//! makes `slope · height` exact for every height `f64` can hold. A slope of `0.6`
//! would not, and the corner would be certified *off* its own face —
//! `a_cone_whose_corner_does_not_sit_on_it_is_refused_by_the_sweep` is that
//! demonstration.

#![allow(dead_code)]

use rcad_geom::{Axis, Cone, EdgeRun, Orientation, Plane, Point, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, SolidBuilder, SolidKey};

/// A frustum about `+z`: apex at the origin, `slope` out per unit up, with its
/// two cap circles at `near` and `far` up the axis, built certified.
///
/// # Panics
///
/// If the kernel refuses the solid. Reach for [`a_frustum_under`] instead when
/// the refusal is the thing being tested — a shape whose ends are far enough
/// apart is refused for its dynamic range, and that is a result and not a bug.
pub fn a_frustum(model: &mut Model, slope: f64, near: f64, far: f64) -> SolidKey {
    a_frustum_under(
        model,
        slope,
        near,
        far,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a frustum described this way is a solid")
}

/// The same frustum, built under `policy`, handing back whatever the kernel
/// made of it.
///
/// Described face by face through [`SolidBuilder`], which is the door an
/// importer comes through — so this is §42.12 criterion 1 for a surface kind
/// that did not exist when that criterion was written.
///
/// # Errors
///
/// Whatever [`SolidBuilder::build`] makes of the description. A frustum whose
/// near cap is tiny beside its height spreads too wide for `f64` and comes back
/// as [`rcad_topo::Invalid::DynamicRangeTooWide`].
///
/// # Panics
///
/// If `near` and `far` do not describe two different ends of a frustum. That is
/// a caller writing down a shape that is not one, not a kernel refusal.
pub fn a_frustum_under(
    model: &mut Model,
    slope: f64,
    near: f64,
    far: f64,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> rcad_topo::Result<SolidKey> {
    assert!(0.0 < near && near < far, "a frustum has two different ends");
    let skin = Cone::new(Axis::z(), slope).expect("a positive slope");
    let mut builder = SolidBuilder::new();

    let bottom_corner = builder
        .add_vertex(Point::new(slope * near, 0.0, near))
        .expect("a finite point");
    let top_corner = builder
        .add_vertex(Point::new(slope * far, 0.0, far))
        .expect("a finite point");
    let seam = builder
        .add_edge(bottom_corner, top_corner, EdgeRun::Straight)
        .expect("a ruling of a cone is straight, which is the whole of what a seam here is");
    let bottom_circle = builder
        .add_edge(
            bottom_corner,
            bottom_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("the cap plane cuts the cone in one closed curve");
    let top_circle = builder
        .add_edge(
            top_corner,
            top_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("the cap plane cuts the cone in one closed curve");

    let cap = |builder: &mut SolidBuilder, at: f64, outward: f64, edge, turn| {
        let plane =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, at), Vector::new(0.0, 0.0, outward))
                .expect("the z axis is a unit normal");
        let coedge = builder
            .add_coedge(edge, turn)
            .expect("an edge of this build");
        let wire = builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("one closed edge is a wire");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a cap is a plane trimmed by its circle")
    };
    let bottom = cap(
        &mut builder,
        near,
        -1.0,
        bottom_circle,
        Orientation::Reversed,
    );
    let top = cap(&mut builder, far, 1.0, top_circle, Orientation::Forward);

    let round_the_wall: Vec<DraftStep> = [
        (bottom_circle, Orientation::Forward),
        (seam, Orientation::Forward),
        (top_circle, Orientation::Reversed),
        (seam, Orientation::Reversed),
    ]
    .into_iter()
    .map(|(edge, turn)| {
        DraftStep::Along(
            builder
                .add_coedge(edge, turn)
                .expect("an edge of this build"),
        )
    })
    .collect();
    let wire = builder
        .add_wire(&round_the_wall)
        .expect("round the bottom, up the seam, back round the top, down the seam");
    let side = builder
        .add_face(Surface::Cone(skin), &[wire])
        .expect("a face on the cone");

    builder
        .add_shell(&[
            (bottom, Orientation::Forward),
            (top, Orientation::Forward),
            (side, Orientation::Forward),
        ])
        .expect("three faces close");
    builder.build(model, policy, log)
}

/// A cone that ends in a point: a base cap at `far` up the axis, and one face on
/// the cone reaching down to the apex.
///
/// **Two faces, one edge, one vertex, and no seam.** The apex is not described
/// at all — it is where the surface's own second coordinate runs out, and the
/// mesher finds it by asking the surface where it pinches.
pub fn a_pointed_cone(model: &mut Model, slope: f64, far: f64) -> SolidKey {
    let skin = Cone::new(Axis::z(), slope).expect("a positive slope");
    let mut builder = SolidBuilder::new();

    let rim = builder
        .add_vertex(Point::new(slope * far, 0.0, far))
        .expect("a finite point");
    let circle = builder
        .add_edge(rim, rim, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the cap plane cuts the cone in one closed curve");

    let plane = Plane::from_point_and_normal(Point::new(0.0, 0.0, far), Vector::new(0.0, 0.0, 1.0))
        .expect("the z axis is a unit normal");
    let cap_coedge = builder
        .add_coedge(circle, Orientation::Forward)
        .expect("an edge of this build");
    let cap_wire = builder
        .add_wire(&[DraftStep::Along(cap_coedge)])
        .expect("one closed edge is a wire");
    let cap = builder
        .add_face(Surface::Plane(plane), &[cap_wire])
        .expect("a cap is a plane trimmed by its circle");

    let side_coedge = builder
        .add_coedge(circle, Orientation::Reversed)
        .expect("an edge of this build");
    let side_wire = builder
        .add_wire(&[DraftStep::Along(side_coedge)])
        .expect("one closed edge is a wire");
    let side = builder
        .add_face(Surface::Cone(skin), &[side_wire])
        .expect("a face on the cone, bounded by its base circle and pinched at the apex");

    builder
        .add_shell(&[(cap, Orientation::Forward), (side, Orientation::Forward)])
        .expect("two faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a pointed cone described this way is a solid")
}

/// Take a cone-faced solid apart **through views alone** and describe it again.
///
/// Nothing here reads an arena: the shell, its face references, each face's
/// surface, wires, steps, coedges, edges and corners all come through
/// `FaceView` and its friends, and what goes back in is a fresh
/// [`SolidBuilder`] description. That is the round trip §42.12 criterion 1 asks
/// for, and doing it over a cone is what says the accessors carry a cone as well
/// as they carry a ball.
///
/// **What an edge runs along is re-derived from the shape and not read back from
/// the model**, which is what an importer has to do anyway: `EdgeView::run` is
/// `pub(crate)` because a stored run is written in the frame its face is stored
/// in. Every edge of a frustum is one of two things — a closed curve where its
/// two faces cross, or a straight ruling — and which one it is comes off
/// `EdgeView::is_closed`.
///
/// # Panics
///
/// If the solid has more than one shell, or anything in it is placed somewhere
/// other than the identity. This walk reads stored geometry, so it is only
/// truthful about a solid that sits where it is stored.
pub fn describe_again(model: &Model, solid: SolidKey) -> rcad_topo::Result<SolidBuilder> {
    use std::collections::HashMap;

    use rcad_geom::Isometry;
    use rcad_topo::{DraftEdge, DraftVertex, EdgeKey, VertexKey, WireStepView};

    let view = model.solid(solid)?;
    let mut shells = view.shells();
    let shell = shells.next().expect("a solid has a shell");
    assert!(shells.next().is_none(), "these solids have one shell each");
    assert_eq!(shell.location(), Isometry::identity());

    let mut builder = SolidBuilder::new();
    let mut corners: HashMap<VertexKey, DraftVertex> = HashMap::new();
    let mut edges: HashMap<EdgeKey, DraftEdge> = HashMap::new();
    let mut faces = Vec::new();

    for reference in shell.face_refs() {
        assert_eq!(reference.location(), Isometry::identity());
        let face = model.face(reference.face())?;
        let mut wires = Vec::new();
        for wire in face.wires() {
            let mut steps = Vec::new();
            for step in wire.steps() {
                let WireStepView::Along(coedge) = step else {
                    panic!("a frustum's wires are made of coedges and nothing else: {step:?}")
                };
                // `CoedgeView::edge` turns the edge the way the coedge uses it,
                // so its ends are the coedge's ends and not the edge's own.
                let stored = model.edge(coedge.edge().key())?;
                let described = match edges.get(&stored.key()) {
                    Some(&described) => described,
                    None => {
                        let mut corner = |vertex: rcad_topo::VertexView<'_>| {
                            if let Some(&already) = corners.get(&vertex.key()) {
                                return Ok(already);
                            }
                            let fresh = builder.add_vertex(vertex.point())?;
                            corners.insert(vertex.key(), fresh);
                            Ok::<_, rcad_topo::Error>(fresh)
                        };
                        let start = corner(stored.start())?;
                        let end = corner(stored.end())?;
                        let run = if stored.is_closed()? {
                            EdgeRun::RightRoundWhereItsFacesCross
                        } else {
                            EdgeRun::Straight
                        };
                        let fresh = builder.add_edge(start, end, run)?;
                        edges.insert(stored.key(), fresh);
                        fresh
                    }
                };
                steps.push(DraftStep::Along(
                    builder.add_coedge(described, coedge.orientation_on_its_edge())?,
                ));
            }
            wires.push(builder.add_wire(&steps)?);
        }
        faces.push((
            builder.add_face(face.surface(), &wires)?,
            reference.orientation(),
        ));
    }
    builder.add_shell(&faces)?;
    Ok(builder)
}
