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
//!
//! # Which way the rim is stored running
//!
//! The ball and the pipe only touch along the rim, so they say nothing about
//! which way it runs; the ball and the rim's plane cross in it, and they are the
//! pair `rcad_geom::which_two_surfaces_a_run_is_between` hands everyone who reads
//! the run. The rule the fuse writes a closed edge with reads that pair: the
//! ball's longitude is square to this rim at every point, because the rim is a
//! circle through both poles, so the latitude answers, and the rim is stored
//! running up towards `+z` at its corner. The ball below the rim walks it that
//! way and the pipe above it walks it back.
//!
//! This file first built the rim the other way round, which is the way the
//! walker happened to draw it. Until the mesher read the rim's pair from the
//! shared function (§222.1 item 1) it read the ball and the pipe, no rule
//! answered, and that way was kept. Reading the ball and the plane, the mesher
//! walks the rim the way the rule says, and the solid built the other way is
//! refused by `AWireWillNotLieFlatOnItsFace` on the pipe — which is what
//! `a_half_ball_on_a_pipe_with_its_rim_stored_against_the_rule_is_refused` pins.
//! `Model::validate` does not read this rim's way yet: it reads the two faces,
//! which only touch (§229.4).

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Orientation, Plane, Point, Scale, Sphere, Surface, Vector,
    which_way_a_closed_crossing_turns,
};
use rcad_predicates::{Decision, Policy, ProvenanceLog};
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

/// Each face uses the rim the way the closed-edge rule stores it, read off the
/// ball and the rim's plane, however the faces are listed.
///
/// The check on the fixture rather than on the kernel: the other tests here
/// mean something only if the solid they measure is stored the way the rule
/// says.
#[test]
fn the_rim_is_stored_the_way_the_ball_and_its_plane_say_whichever_face_is_listed_first() {
    let corner = Vector::new(RADIUS, 0.0, 0.0);
    let up = Vector::new(0.0, 1.0, 0.0);
    let crossing = corner.cross(up);
    let stored_way =
        match which_way_a_closed_crossing_turns([corner, up], corner, Vector::new(0.0, 0.0, 1.0)) {
            Decision::Pos => crossing,
            Decision::Neg => -crossing,
            Decision::Zero | Decision::Undecidable { .. } | _ => {
                panic!("the ball and the rim's plane cross at the corner")
            }
        };
    for first in [FirstInTheShell::Ball, FirstInTheShell::Pipe] {
        let (model, key) = a_half_ball_on_a_pipe(first);
        for face in model.solid(key).expect("this model made the key").faces() {
            let below = match face.surface() {
                Surface::Sphere(_) => true,
                Surface::Cylinder(_) => false,
                Surface::Plane(_) => continue,
                other => panic!("the solid has a face on {other:?}"),
            };
            let [way] = face
                .wires()
                .flat_map(|wire| wire.coedges())
                .filter(|coedge| {
                    matches!(
                        coedge.edge().run(),
                        EdgeRun::RightRoundWhereItCrosses { .. }
                    )
                })
                .map(|coedge| coedge.orientation_on_its_edge())
                .collect::<Vec<_>>()[..]
            else {
                panic!("the {first:?} listed first: a face uses the rim once");
            };
            let walked = match way {
                Orientation::Forward => stored_way,
                Orientation::Reversed => -stored_way,
            };
            let on_its_left = corner.cross(walked).dot(up);
            assert_eq!(
                on_its_left < 0.0,
                below,
                "the {first:?} listed first: the face on {:?} walks the rim {way:?}, which puts \
                 it on the wrong side of the rim",
                face.surface()
            );
        }
    }
}

/// The same solid with the rim's two uses turned round, which is the way it
/// was first built here, is refused by name rather than measured.
///
/// Walked the way the rule stores it, the ball's face would be the half above
/// the rim, inside the pipe, and it is the pipe's wire that no longer lies flat.
/// Refused rather than pinned to one refusal, because `Model::validate` may
/// come to read this rim's way and refuse it first (§229.4).
#[test]
fn a_half_ball_on_a_pipe_with_its_rim_stored_against_the_rule_is_refused() {
    for first in [FirstInTheShell::Ball, FirstInTheShell::Pipe] {
        let (_, built) = built_with_the_ball_using_the_rim(first, Orientation::Reversed);
        assert!(
            built.is_err(),
            "the {first:?} listed first: a rim stored against the rule is built into {built:?}"
        );
    }
}

/// The half of a ball of `RADIUS` about the origin below `y = 0`, a pipe of the
/// same radius up the `y` axis from the rim to `HEIGHT`, and a flat disc on top.
///
/// The rim's one corner is `(RADIUS, 0, 0)`, a quarter turn from either pole.
/// The pipe is cut open up a seam from that corner to the top circle's.
fn a_half_ball_on_a_pipe(first: FirstInTheShell) -> (Model, SolidKey) {
    let (model, key) = built_with_the_ball_using_the_rim(first, Orientation::Forward);
    let key = key.expect("a half ball on a pipe is a solid");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a half ball on a pipe is a sound model");
    (model, key)
}

/// [`a_half_ball_on_a_pipe`], with the ball using the rim `ball_uses_the_rim`
/// and the pipe using it the other way.
fn built_with_the_ball_using_the_rim(
    first: FirstInTheShell,
    ball_uses_the_rim: Orientation,
) -> (Model, rcad_topo::Result<SolidKey>) {
    let pipe_uses_the_rim = match ball_uses_the_rim {
        Orientation::Forward => Orientation::Reversed,
        Orientation::Reversed => Orientation::Forward,
    };
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
        .add_edge(
            on_the_lid,
            on_the_lid,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a closed edge");
    let seam = builder
        .add_edge(on_the_rim, on_the_lid, EdgeRun::Straight)
        .expect("a straight edge");

    let the_ball = |builder: &mut SolidBuilder| {
        let under = builder
            .add_coedge(rim, ball_uses_the_rim)
            .expect("an edge of this builder");
        let wire = builder
            .add_wire(&[DraftStep::Along(under)])
            .expect("a closed wire");
        builder.add_face(ball, &[wire]).expect("a face")
    };
    let the_pipe = |builder: &mut SolidBuilder| {
        let steps = [
            (rim, pipe_uses_the_rim),
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
    let built = builder.build(&mut model, &Policy::certified(), &mut ProvenanceLog::new());
    (model, built)
}
