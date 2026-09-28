//! A half ball on a pipe of its own radius, capped by a flat disc, built
//! through the public door, and what `Model::validate` reads its rim against.
//!
//! The rim is one closed edge between the ball's face and the pipe's. A ball
//! and a pipe of one radius on one axis only touch there, so the rim names the
//! plane it lies in (`EdgeRun::RightRoundWhereItCrosses`), and the ball and
//! that plane are the pair `rcad_geom::which_two_surfaces_a_run_is_between`
//! hands everyone who reads the run. The ball is written about its own `z` and
//! the rim lies in `y = 0`, so the rim runs through both of the ball's poles,
//! and its one corner `(RADIUS, 0, 0)` is on the ball's equator, where the
//! ball's longitude is square to the rim and the rule reads the latitude.
//!
//! **Validate used to read the rim against the face across**, the pipe for the
//! ball's use and the ball for the pipe's. They only touch, so the rule had
//! nothing to read and the rim's way went unchecked; only the mesh refused a
//! rim walked against the rule, as a wire that will not lie flat. It reads the
//! rim against the plane now (`docs/decisions.md` §229.4), the same pair the
//! mesher walks. Before the mesher read that pair too, Q15 measured the two
//! disagreeing: validate reading the latitude and the mesher the longitude, and
//! this solid built in neither way round.
//!
//! The plane is written facing both ways. Which way it faces turns both the
//! gradient the rule reads and the side each face is on, so it may not change
//! the answer.

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Orientation, Plane, Point, Scale, Sphere, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Invalid, Model, SolidBuilder, SolidKey};

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

/// Which way the rim's plane faces along the pipe's axis.
#[derive(Debug, Clone, Copy)]
enum RimsPlaneFaces {
    Up,
    Down,
}

/// Walked the way the rule stores the rim, the solid builds, validates and
/// measures half a ball and a pipe's worth of space within the error it
/// publishes, whichever face is listed first and whichever way the rim's plane
/// faces.
#[test]
fn a_half_ball_on_a_pipe_builds_and_measures_its_closed_form_however_it_is_written() {
    let closed_form = core::f64::consts::PI * RADIUS * RADIUS * (2.0 * RADIUS / 3.0 + HEIGHT);
    for first in [FirstInTheShell::Ball, FirstInTheShell::Pipe] {
        for facing in [RimsPlaneFaces::Up, RimsPlaneFaces::Down] {
            let what = format!("the {first:?} listed first, the rim's plane facing {facing:?}");
            let (model, built) = a_half_ball_on_a_pipe(first, facing, Orientation::Forward);
            let key = built.unwrap_or_else(|why| panic!("{what}: refused {why:?}"));
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what}: the model does not validate: {why:?}"));
            let solid = model.solid(key).expect("this model made the key");
            for deflection in DEFLECTIONS {
                let measured = solid
                    .measure(
                        Deflection::of(deflection).expect("a length"),
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    )
                    .unwrap_or_else(|why| {
                        panic!("{what}, at {deflection}: the solid does not measure: {why:?}")
                    });
                assert!(
                    (measured.volume() - closed_form).abs() <= measured.volume_error(),
                    "{what}, at {deflection}: measured {} ± {}, and the closed form is \
                     {closed_form}",
                    measured.volume(),
                    measured.volume_error()
                );
            }
        }
    }
}

/// Walked against the rule, the solid is refused by the check that reads which
/// way a closed edge runs, by name, however it is written.
///
/// Fails where validate reads the rim against the face across: the ball and the
/// pipe only touch, the check passes the rim, and the mesh refuses the pipe's
/// wire as one that will not lie flat.
#[test]
fn a_half_ball_on_a_pipe_walked_against_the_rule_is_refused_by_the_closed_edge_check() {
    for first in [FirstInTheShell::Ball, FirstInTheShell::Pipe] {
        for facing in [RimsPlaneFaces::Up, RimsPlaneFaces::Down] {
            let (_, built) = a_half_ball_on_a_pipe(first, facing, Orientation::Reversed);
            assert!(
                matches!(
                    built,
                    Err(Error::Invalid(
                        Invalid::AClosedEdgeIsWalkedAgainstItsSurface
                    ))
                ),
                "the {first:?} listed first, the rim's plane facing {facing:?}: {built:?}"
            );
        }
    }
}

/// The half of a ball of `RADIUS` about the origin below `y = 0`, a pipe of the
/// same radius up the `y` axis from the rim to `HEIGHT`, and a flat disc on top.
/// The ball uses the rim `ball_uses_the_rim` and the pipe uses it the other way.
///
/// The rule stores the rim running up towards `+z` at its corner, which the
/// ball below the rim walks forwards.
fn a_half_ball_on_a_pipe(
    first: FirstInTheShell,
    facing: RimsPlaneFaces,
    ball_uses_the_rim: Orientation,
) -> (Model, rcad_topo::Result<SolidKey>) {
    let origin = Point::origin();
    let up = Vector::new(0.0, 1.0, 0.0);
    let ball = Surface::Sphere(Sphere::new(origin, RADIUS).expect("a positive radius"));
    let pipe = Surface::Cylinder(
        Cylinder::new(Axis::new(origin, up).expect("y has a length"), RADIUS)
            .expect("a positive radius"),
    );
    let rims_normal = match facing {
        RimsPlaneFaces::Up => up,
        RimsPlaneFaces::Down => -up,
    };
    let rims_plane =
        Surface::Plane(Plane::from_point_and_normal(origin, rims_normal).expect("y has a length"));
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
            (rim, ball_uses_the_rim.reversed()),
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
