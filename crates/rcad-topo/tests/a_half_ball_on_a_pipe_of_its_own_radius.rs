//! A half ball on a pipe of its own radius, capped by a flat disc, built
//! through the public door, and what `Model::validate` reads its rim against.
//!
//! The rim is one closed edge between the ball's face and the pipe's. A ball
//! and a pipe of one radius on one axis only touch there, so the rim names the
//! plane it lies in (`EdgeRun::RightRoundWhereItCrosses`), and the ball and
//! that plane are the pair `rcad_geom::which_two_surfaces_a_run_is_between`
//! hands everyone who reads the run. The ball is written about its own `z` and
//! the rim lies in `y = 0`, so the rim runs through both of the ball's poles.
//! Its one corner is put in one of two places. At `(RADIUS, 0, 0)`, on the
//! ball's equator, the ball's longitude is square to the rim and the rule reads
//! the latitude. At `(0, 0, RADIUS)`, on the ball's pole, both are zero, the
//! rule reads nothing, and the solid is sound whichever way the ball uses the
//! rim (`docs/decisions.md` §341.5).
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
use rcad_topo::{DraftStep, Error, Invalid, Model, SolidBuilder, SolidKey, SolidView};

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

/// With the rim's corner on the ball's pole, where neither the ball's longitude
/// nor its latitude says which way the rim runs, the solid builds, validates,
/// meshes each face on its own side of the rim's plane and measures its closed
/// form, whichever way the ball uses the rim, whichever face is listed first and
/// whichever way the rim's plane faces.
///
/// Validate's rule answers nothing at the pole and skips the rim, so both ways
/// of using it are one solid. Fails where the mesher keeps the walker's order
/// there: that order turns round with the rim's plane, and two of the four ways
/// were refused `AWireWillNotLieFlatOnItsFace` on the pipe, which two depending
/// on the plane (`docs/decisions.md` §341.4, §341.5).
#[test]
fn a_half_ball_on_a_pipe_cornered_on_its_pole_builds_however_it_is_written() {
    let closed_form = core::f64::consts::PI * RADIUS * RADIUS * (2.0 * RADIUS / 3.0 + HEIGHT);
    for first in [FirstInTheShell::Ball, FirstInTheShell::Pipe] {
        for facing in [RimsPlaneFaces::Up, RimsPlaneFaces::Down] {
            for ball_uses_the_rim in [Orientation::Forward, Orientation::Reversed] {
                let what = format!(
                    "the {first:?} listed first, the rim's plane facing {facing:?}, the ball \
                     using the rim {ball_uses_the_rim:?}"
                );
                let (model, built) = a_half_ball_on_a_pipe_cornered(
                    RimsCorner::OnThePole,
                    first,
                    facing,
                    ball_uses_the_rim,
                );
                let key = built.unwrap_or_else(|why| panic!("{what}: refused {why:?}"));
                model
                    .validate(&Policy::certified(), &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| panic!("{what}: the model does not validate: {why:?}"));
                let solid = model.solid(key).expect("this model made the key");
                for deflection in DEFLECTIONS {
                    let what = format!("{what}, at {deflection}");
                    let deflection = Deflection::of(deflection).expect("a length");
                    each_face_is_meshed_on_its_own_side_of_the_rim(solid, deflection, &what);
                    let measured = solid
                        .measure(deflection, &Policy::certified(), &mut ProvenanceLog::new())
                        .unwrap_or_else(|why| {
                            panic!("{what}: the solid does not measure: {why:?}")
                        });
                    assert!(
                        (measured.volume() - closed_form).abs() <= measured.volume_error(),
                        "{what}: measured {} ± {}, and the closed form is {closed_form}",
                        measured.volume(),
                        measured.volume_error()
                    );
                }
            }
        }
    }
}

/// Every triangle of the ball's face has its middle below the rim's plane
/// `y = 0`, and every triangle of the pipe and the lid above it.
///
/// A rim walked the wrong way round still lets the ball's face lie flat, as the
/// other half of the ball, so the side each face's triangles are on is what
/// tells the two apart.
fn each_face_is_meshed_on_its_own_side_of_the_rim(
    solid: SolidView<'_>,
    deflection: Deflection,
    what: &str,
) {
    let mesh = solid
        .mesh(deflection, &Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: the solid does not mesh: {why:?}"));
    for triangle in mesh.triangles() {
        let face = solid
            .faces()
            .find(|face| face.key() == triangle.face())
            .expect("a triangle names a face of the solid it was cut from");
        let middle = triangle
            .corners()
            .iter()
            .map(|&corner| mesh.points()[corner as usize].coordinates()[1])
            .sum::<f64>()
            / 3.0;
        match face.surface() {
            Surface::Sphere(_) => assert!(
                middle < 0.0,
                "{what}: a triangle of the ball has its middle at y = {middle}"
            ),
            Surface::Cylinder(_) | Surface::Plane(_) => assert!(
                middle > 0.0,
                "{what}: a triangle of the pipe or the lid has its middle at y = {middle}"
            ),
            Surface::Cone(_) | Surface::Torus(_) | _ => {
                panic!("{what}: a face this solid was not built with")
            }
        }
    }
}

/// The half of a ball of `RADIUS` about the origin below `y = 0`, a pipe of the
/// same radius up the `y` axis from the rim to `HEIGHT`, and a flat disc on top,
/// with the rim's corner on the ball's equator.
///
/// The rule stores the rim running up towards `+z` at that corner, which the
/// ball below the rim walks forwards.
fn a_half_ball_on_a_pipe(
    first: FirstInTheShell,
    facing: RimsPlaneFaces,
    ball_uses_the_rim: Orientation,
) -> (Model, rcad_topo::Result<SolidKey>) {
    a_half_ball_on_a_pipe_cornered(RimsCorner::OnTheEquator, first, facing, ball_uses_the_rim)
}

/// Where on the ball the rim's one corner is, and so the seam's two ends.
#[derive(Debug, Clone, Copy)]
enum RimsCorner {
    /// `(RADIUS, 0, 0)`, where the ball's longitude is square to the rim and
    /// the rule reads the latitude.
    OnTheEquator,
    /// `(0, 0, RADIUS)`, where the ball's longitude and latitude are both zero
    /// and the rule reads nothing.
    OnThePole,
}

/// The half ball on a pipe with the rim's corner where `corner` says. The ball
/// uses the rim `ball_uses_the_rim` and the pipe uses it the other way.
fn a_half_ball_on_a_pipe_cornered(
    corner: RimsCorner,
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

    let [rims_corner, lids_corner] = match corner {
        RimsCorner::OnTheEquator => [
            Point::new(RADIUS, 0.0, 0.0),
            Point::new(RADIUS, HEIGHT, 0.0),
        ],
        RimsCorner::OnThePole => [
            Point::new(0.0, 0.0, RADIUS),
            Point::new(0.0, HEIGHT, RADIUS),
        ],
    };
    let mut builder = SolidBuilder::new();
    let on_the_rim = builder.add_vertex(rims_corner).expect("a corner");
    let on_the_lid = builder.add_vertex(lids_corner).expect("a corner");
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
