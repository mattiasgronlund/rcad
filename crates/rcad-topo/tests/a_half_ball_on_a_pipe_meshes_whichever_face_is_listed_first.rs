//! A half ball on a pipe of its own radius, capped by a flat disc, built
//! through the public door with the ball's face listed first and then the
//! pipe's.
//!
//! The rim between the half ball and the pipe is one closed edge that names the
//! plane it lies in (`EdgeRun::RightRoundWhereItCrosses`): a ball and a pipe of
//! one radius on one axis only touch there, so neither face says where the rim
//! runs. That plane is read with one of the two faces, and until
//! `docs/decisions.md` §336 it was the ball only when the ball was listed first.
//! The mesher reads the same pair to find the poles a closed rim runs through,
//! and this rim runs through both of the ball's: the ball is written about its
//! own `z`, and the rim lies in `y = 0`.
//!
//! Measured on the base `96da3252`: with the ball listed first the solid
//! measured `20.9777 ± 1.680`, `21.1435 ± 0.455` and `21.2013 ± 0.0330` at the
//! three deflections below, each holding the closed form. With the pipe listed
//! first every one was refused `CannotMeshAFace { why: CornersCoincide }` on the
//! ball's face, because the poles were looked for on the pipe.

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Orientation, Plane, Point, Scale, Sphere, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, SolidBuilder, SolidKey};

/// The radius of the ball and of the pipe.
const RADIUS: f64 = 1.5;
/// How tall the pipe stands above the rim.
const HEIGHT: f64 = 2.0;

/// A coarse, a middling and a fine deflection, each well under the radius.
const DEFLECTIONS: [f64; 3] = [0.05, 0.0137, 0.001];

/// Which face the shell lists first.
#[derive(Debug, Clone, Copy)]
enum FirstInTheShell {
    Ball,
    Pipe,
}

/// The solid measures half a ball and a pipe's worth of space, within the error
/// it publishes, however its faces are listed.
///
/// Fails on the base with the pipe listed first.
#[test]
fn a_half_ball_on_a_pipe_measures_its_closed_form_whichever_face_is_listed_first() {
    let closed_form = core::f64::consts::PI * RADIUS * RADIUS * (2.0 * RADIUS / 3.0 + HEIGHT);
    for first in [FirstInTheShell::Ball, FirstInTheShell::Pipe] {
        let (model, key) = a_half_ball_on_a_pipe(first);
        let solid = model.solid(key).expect("this model made the key");
        for deflection in DEFLECTIONS {
            let what = format!("the {first:?} listed first, at a deflection of {deflection}");
            let measured = solid
                .measure(
                    Deflection::of(deflection).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what}: the solid does not measure: {why:?}"));
            assert!(
                (measured.volume() - closed_form).abs() <= measured.volume_error(),
                "{what}: measured {} ± {}, and the closed form is {closed_form}",
                measured.volume(),
                measured.volume_error()
            );
        }
    }
}

/// Both of the ball's poles are points of the mesh along the rim, exactly,
/// however the faces are listed.
///
/// Fails on the base with the pipe listed first, where the mesh is refused.
#[test]
fn both_poles_are_points_along_the_rim_whichever_face_is_listed_first() {
    for first in [FirstInTheShell::Ball, FirstInTheShell::Pipe] {
        let (model, key) = a_half_ball_on_a_pipe(first);
        let solid = model.solid(key).expect("this model made the key");
        for deflection in DEFLECTIONS {
            let what = format!("the {first:?} listed first, at a deflection of {deflection}");
            let mesh = solid
                .mesh(
                    Deflection::of(deflection).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what}: the solid does not mesh: {why:?}"));
            let rim = solid
                .faces()
                .flat_map(|face| face.wires())
                .flat_map(|wire| wire.coedges())
                .map(|coedge| coedge.edge())
                .find(|edge| matches!(edge.run(), EdgeRun::RightRoundWhereItCrosses { .. }))
                .expect("the rim is a closed edge that names its plane");
            let along: Vec<[f64; 3]> = mesh
                .points_along_edge(rim.key())
                .iter()
                .map(|&at| mesh.points()[at as usize].coordinates())
                .collect();
            for pole in [[0.0, 0.0, RADIUS], [0.0, 0.0, -RADIUS]] {
                assert!(
                    along.contains(&pole),
                    "{what}: the pole {pole:?} is not among the {} points along the rim",
                    along.len()
                );
            }
        }
    }
}

/// The half of a ball of `RADIUS` about the origin below `y = 0`, a pipe of the
/// same radius up the `y` axis from the rim to `HEIGHT`, and a flat disc on top.
///
/// The rim's one corner is `(RADIUS, 0, 0)`, a quarter turn from either pole.
/// The pipe is cut open up a seam from that corner to the top circle's.
fn a_half_ball_on_a_pipe(first: FirstInTheShell) -> (Model, SolidKey) {
    let origin = Point::origin();
    let up = Vector::new(0.0, 1.0, 0.0);
    let ball = Surface::Sphere(Sphere::new(origin, RADIUS).expect("a positive radius"));
    let pipe = Surface::Cylinder(
        Cylinder::new(Axis::new(origin, up).expect("y has a length"), RADIUS)
            .expect("a positive radius"),
    );
    let rims_plane =
        Surface::Plane(Plane::from_point_and_normal(origin, up).expect("y has a length"));
    let lid = Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.0, HEIGHT, 0.0), up).expect("y has a length"),
    );

    let mut builder = SolidBuilder::new();
    let on_the_rim = builder
        .add_vertex(Point::new(RADIUS, 0.0, 0.0))
        .expect("a corner");
    let on_the_lid = builder
        .add_vertex(Point::new(RADIUS, HEIGHT, 0.0))
        .expect("a corner");
    let rim = builder
        .add_edge(
            on_the_rim,
            on_the_rim,
            EdgeRun::RightRoundWhereItCrosses { and: rims_plane },
        )
        .expect("a rim is a whole closed curve, so one corner twice is an edge");
    let round_the_lid = builder
        .add_edge(on_the_lid, on_the_lid, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("a closed edge");
    let seam = builder
        .add_edge(on_the_rim, on_the_lid, EdgeRun::Straight)
        .expect("a straight edge");

    let the_ball = |builder: &mut SolidBuilder| {
        let under = builder
            .add_coedge(rim, Orientation::Reversed)
            .expect("an edge of this builder");
        let wire = builder
            .add_wire(&[DraftStep::Along(under)])
            .expect("a closed wire");
        builder.add_face(ball, &[wire]).expect("a face")
    };
    let the_pipe = |builder: &mut SolidBuilder| {
        let steps = [
            (rim, Orientation::Forward),
            (seam, Orientation::Forward),
            (round_the_lid, Orientation::Reversed),
            (seam, Orientation::Reversed),
        ]
        .map(|(edge, way)| {
            DraftStep::Along(
                builder
                    .add_coedge(edge, way)
                    .expect("an edge of this builder"),
            )
        });
        let wire = builder.add_wire(&steps).expect("a closed wire");
        builder.add_face(pipe, &[wire]).expect("a face")
    };
    let faces = match first {
        FirstInTheShell::Ball => {
            let ball = the_ball(&mut builder);
            [ball, the_pipe(&mut builder)]
        }
        FirstInTheShell::Pipe => {
            let pipe = the_pipe(&mut builder);
            [pipe, the_ball(&mut builder)]
        }
    };
    let over = builder
        .add_coedge(round_the_lid, Orientation::Forward)
        .expect("an edge of this builder");
    let wire = builder
        .add_wire(&[DraftStep::Along(over)])
        .expect("a closed wire");
    let lid = builder.add_face(lid, &[wire]).expect("a face");
    builder
        .add_shell(&[
            (faces[0], Orientation::Forward),
            (faces[1], Orientation::Forward),
            (lid, Orientation::Forward),
        ])
        .expect("the three faces close up");

    let mut model = Model::new(Scale::millimetres(10.0).expect("a positive size"));
    let key = builder
        .build(&mut model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a half ball on a pipe is a solid");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a half ball on a pipe is a sound model");
    (model, key)
}
