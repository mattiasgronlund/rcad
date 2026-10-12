//! A half ball under a frustum that stands on the ball's great circle, built
//! through the public door, with the circle's one corner on the ball's pole or
//! on its equator.
//!
//! The ball is written about its own `z`, and the cone about the `y` axis
//! through the ball's centre, with a slope of one half and its apex 3 below the
//! rim or 3 above it, so it meets the ball in the circle `y = 0` of the ball's
//! own radius. There they cross, not touch, and the rim is named by its two
//! faces (`EdgeRun::RightRoundWhereItsFacesCross`). A flat lid closes the
//! frustum at `y = 2`.
//!
//! On the ball's pole `(0, 0, RADIUS)` the ball's longitude and latitude are
//! both zero, so the rule that names which way a closed edge is stored running
//! reads nothing there, and `Model::validate` skips the rim: the ball using it
//! forward and the cone backward is the same solid as the other way round
//! with the rim's direction turned (`docs/decisions.md` §341.5). So the eight
//! ways of writing it — either face listed first, the cone's axis written up or
//! down, the ball using the rim either way — are one solid, and only four of
//! them build. The mesher asks its touch fallback before the faces' own walks,
//! and the fallback answers with the cone's longitude, which is not zero there
//! and turns round with the way the cone's axis is written. A cone is both its
//! nappes, so the axis written up or down is the one cone, and which four build
//! is not a fact about the shape (§341.6).
//!
//! **The pole tests pin that refusal, and fail on the day it is fixed.** Asking
//! the faces' walk before the touch fallback is not the fix on its own: it
//! builds all eight of the frustum that flares out, and refuses all eight of
//! the one that narrows in, whose face crosses the ball again at `y = 1.2`, so
//! that its lid's corner is on the other side of the ball from the face beside
//! the rim (§341.7).
//!
//! On the equator `(RADIUS, 0, 0)` the rule has the ball's latitude to read, so
//! the ball using the rim forward builds and the other way is refused, however
//! the cone's axis is written.

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, Orientation, Plane, Point, Scale, Sphere, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Invalid, Model, SolidBuilder, SolidKey, SolidView};

/// The radius of the ball, and of the cone where they cross.
const RADIUS: f64 = 1.5;
/// How far out the cone spreads for every unit along its axis.
const SLOPE: f64 = 0.5;
/// How tall the frustum stands above the rim.
const HEIGHT: f64 = 2.0;

/// A coarse, a middling and a fine deflection, each well under the radius.
const DEFLECTIONS: [f64; 3] = [0.05, 0.0137, 0.001];

/// With the rim's corner on the ball's pole, the frustum that flares out builds,
/// validates, meshes each face on its own side of the rim's plane and measures
/// its closed form where the ball uses the rim reversed with the cone's axis
/// up, or forward with it down, in either face order. The other four writings
/// of the one solid are refused `AWireWillNotLieFlatOnItsFace` (§341.7).
#[test]
fn a_frustum_flaring_out_of_a_balls_pole_builds_in_the_half_of_its_writings_its_axis_picks() {
    let wrong = what_each_writing_gets_wrong(
        Frustum::FlaresOut,
        RimsCorner::OnThePole,
        the_half_the_cones_axis_picks,
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The frustum that narrows in, cornered on the pole, builds in the same four
/// writings as the one that flares out, and is refused in the same other four,
/// though its face crosses the ball a second time (§341.7).
#[test]
fn a_frustum_narrowing_from_a_balls_pole_builds_in_the_half_of_its_writings_its_axis_picks() {
    let wrong = what_each_writing_gets_wrong(
        Frustum::NarrowsIn,
        RimsCorner::OnThePole,
        the_half_the_cones_axis_picks,
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// With the rim's corner on the ball's equator, both frustums build, validate,
/// mesh each face on its own side and measure their closed form where the ball
/// uses the rim forward, and are refused `AClosedEdgeIsWalkedAgainstItsSurface`
/// where it uses the rim reversed, whichever face is listed first and whichever
/// way the cone's axis is written.
#[test]
fn cornered_on_its_equator_a_ball_and_a_cone_build_the_one_way_the_ball_winds() {
    for shape in [Frustum::FlaresOut, Frustum::NarrowsIn] {
        let wrong = what_each_writing_gets_wrong(
            shape,
            RimsCorner::OnTheEquator,
            |_, ball_uses_the_rim| match ball_uses_the_rim {
                Orientation::Forward => ItShould::Build,
                Orientation::Reversed => ItShould::BeRefusedAsWalkedAgainstItsSurface,
            },
        );
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    }
}

/// Which way the frustum goes from the rim up to the lid.
#[derive(Debug, Clone, Copy)]
enum Frustum {
    /// Wider at the lid, its apex below the ball. The cone meets the ball again
    /// below the rim, where no face of either is.
    FlaresOut,
    /// Narrower at the lid, its apex above the rim. The cone meets the ball
    /// again at `y = 1.2`, through the middle of the cone's face: the face is
    /// inside the ball below that and outside it above, so the lid's corner is
    /// on the other side of the ball from the face beside the rim.
    NarrowsIn,
}

/// Where on the ball the rim's one corner is.
#[derive(Debug, Clone, Copy)]
enum RimsCorner {
    OnThePole,
    OnTheEquator,
}

/// Which face the shell lists first.
#[derive(Debug, Clone, Copy)]
enum FirstInTheShell {
    Ball,
    Cone,
}

/// Which way the cone's axis is written from its apex.
#[derive(Debug, Clone, Copy)]
enum ConesAxisPoints {
    Up,
    Down,
}

/// What one way of writing the solid comes to today.
#[derive(Debug, Clone, Copy)]
enum ItShould {
    /// It builds, validates, meshes each face on its own side of the rim's
    /// plane and measures its closed form.
    Build,
    /// It is refused `AWireWillNotLieFlatOnItsFace`.
    BeRefusedAsAWireThatWillNotLieFlat,
    /// It is refused `AClosedEdgeIsWalkedAgainstItsSurface`.
    BeRefusedAsWalkedAgainstItsSurface,
}

/// At the pole: the ball using the rim reversed with the cone's axis up, or
/// forward with it down, builds; the other two are refused.
fn the_half_the_cones_axis_picks(
    axis: ConesAxisPoints,
    ball_uses_the_rim: Orientation,
) -> ItShould {
    match (axis, ball_uses_the_rim) {
        (ConesAxisPoints::Up, Orientation::Reversed)
        | (ConesAxisPoints::Down, Orientation::Forward) => ItShould::Build,
        (ConesAxisPoints::Up, Orientation::Forward)
        | (ConesAxisPoints::Down, Orientation::Reversed) => {
            ItShould::BeRefusedAsAWireThatWillNotLieFlat
        }
    }
}

/// Each way of writing `shape` with its rim cornered at `corner` that does not
/// come to what `it_should` says of its axis and its ball's use of the rim,
/// with what it came to.
fn what_each_writing_gets_wrong(
    shape: Frustum,
    corner: RimsCorner,
    it_should: impl Fn(ConesAxisPoints, Orientation) -> ItShould,
) -> Vec<String> {
    let mut wrong = Vec::new();
    for first in [FirstInTheShell::Ball, FirstInTheShell::Cone] {
        for axis in [ConesAxisPoints::Up, ConesAxisPoints::Down] {
            for ball_uses_the_rim in [Orientation::Forward, Orientation::Reversed] {
                let what = format!(
                    "{shape:?} {corner:?}, the {first:?} listed first, the cone's axis pointing {axis:?}, \
                     the ball using the rim {ball_uses_the_rim:?}"
                );
                let should = it_should(axis, ball_uses_the_rim);
                let (model, built) =
                    a_half_ball_under_a_frustum(shape, corner, first, axis, ball_uses_the_rim);
                match (should, built) {
                    (ItShould::Build, Ok(key)) => wrong.extend(
                        what_a_built_writing_gets_wrong(&model, key, shape)
                            .into_iter()
                            .map(|why| format!("{what}: {why}")),
                    ),
                    (
                        ItShould::BeRefusedAsAWireThatWillNotLieFlat,
                        Err(Error::AWireWillNotLieFlatOnItsFace { .. }),
                    )
                    | (
                        ItShould::BeRefusedAsWalkedAgainstItsSurface,
                        Err(Error::Invalid(Invalid::AClosedEdgeIsWalkedAgainstItsSurface)),
                    ) => {}
                    (should, built) => {
                        wrong.push(format!(
                            "{what}: it should {should:?}, and came to {built:?}"
                        ));
                    }
                }
            }
        }
    }
    wrong
}

/// Where the solid `key` of `shape` does not validate, meshes a face on the
/// wrong side of the rim or measures off its closed form, at each deflection,
/// what went wrong.
fn what_a_built_writing_gets_wrong(model: &Model, key: SolidKey, shape: Frustum) -> Vec<String> {
    let lids_radius = the_lids_radius(shape);
    let closed_form = core::f64::consts::PI
        * (2.0 * RADIUS.powi(3) / 3.0
            + HEIGHT / 3.0 * (RADIUS * RADIUS + RADIUS * lids_radius + lids_radius * lids_radius));
    if let Err(why) = model.validate(&Policy::certified(), &mut ProvenanceLog::new()) {
        return vec![format!("does not validate: {why:?}")];
    }
    let solid = model.solid(key).expect("this model made the key");
    let mut wrong = Vec::new();
    for deflection in DEFLECTIONS {
        let at = deflection;
        let deflection = Deflection::of(deflection).expect("a length");
        if let Some(why) = where_a_face_is_meshed_off_its_side(solid, deflection) {
            wrong.push(format!("at {at}: {why}"));
            continue;
        }
        match solid.measure(deflection, &Policy::certified(), &mut ProvenanceLog::new()) {
            Ok(measured) if (measured.volume() - closed_form).abs() <= measured.volume_error() => {}
            Ok(measured) => wrong.push(format!(
                "at {at}: measured {} ± {}, and the closed form is {closed_form}",
                measured.volume(),
                measured.volume_error()
            )),
            Err(why) => wrong.push(format!("at {at}: does not measure: {why:?}")),
        }
    }
    wrong
}

/// The radius of the frustum's lid at `HEIGHT`, the cone's apex being
/// `RADIUS / SLOPE` from the rim.
fn the_lids_radius(shape: Frustum) -> f64 {
    match shape {
        Frustum::FlaresOut => RADIUS + SLOPE * HEIGHT,
        Frustum::NarrowsIn => RADIUS - SLOPE * HEIGHT,
    }
}

/// Where a triangle of the ball's face has its middle above the rim's plane
/// `y = 0`, or one of the cone or the lid below it, what was found, or where
/// the solid does not mesh, why.
///
/// A rim walked the wrong way round can still let the ball's face lie flat, as
/// the other half of the ball, so the side each face's triangles are on is
/// what tells the two apart.
fn where_a_face_is_meshed_off_its_side(
    solid: SolidView<'_>,
    deflection: Deflection,
) -> Option<String> {
    let mesh = match solid.mesh(deflection, &Policy::certified(), &mut ProvenanceLog::new()) {
        Ok(mesh) => mesh,
        Err(why) => return Some(format!("does not mesh: {why:?}")),
    };
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
            Surface::Sphere(_) if middle >= 0.0 => {
                return Some(format!(
                    "a triangle of the ball has its middle at y = {middle}"
                ));
            }
            Surface::Cone(_) | Surface::Plane(_) if middle <= 0.0 => {
                return Some(format!(
                    "a triangle of the cone or the lid has its middle at y = {middle}"
                ));
            }
            Surface::Sphere(_) | Surface::Cone(_) | Surface::Plane(_) => {}
            Surface::Cylinder(_) | Surface::Torus(_) | _ => {
                panic!("a face this solid was not built with")
            }
        }
    }
    None
}

/// The half of a ball of `RADIUS` about the origin below `y = 0`, the frustum
/// `shape` of a cone about the `y` axis from the rim up to `HEIGHT`, and a flat
/// disc on top, with the rim's one corner at `corner`. The ball uses the rim
/// `ball_uses_the_rim` and the cone uses it the other way.
///
/// The lid's circle is stored running the way the cone's longitude increases
/// at its corner, which turns with the cone's axis, so the lid's use of it
/// turns with the axis too.
fn a_half_ball_under_a_frustum(
    shape: Frustum,
    corner: RimsCorner,
    first: FirstInTheShell,
    axis: ConesAxisPoints,
    ball_uses_the_rim: Orientation,
) -> (Model, rcad_topo::Result<SolidKey>) {
    let origin = Point::origin();
    let up = Vector::new(0.0, 1.0, 0.0);
    let ball = Surface::Sphere(Sphere::new(origin, RADIUS).expect("a positive radius"));
    let apex = Point::new(
        0.0,
        match shape {
            Frustum::FlaresOut => -RADIUS / SLOPE,
            Frustum::NarrowsIn => RADIUS / SLOPE,
        },
        0.0,
    );
    let (along, lid_uses_its_circle) = match axis {
        ConesAxisPoints::Up => (up, Orientation::Forward),
        ConesAxisPoints::Down => (-up, Orientation::Reversed),
    };
    let cone = Surface::Cone(
        Cone::new(Axis::new(apex, along).expect("y has a length"), SLOPE)
            .expect("a positive slope"),
    );
    let lid = Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.0, HEIGHT, 0.0), up).expect("y has a length"),
    );

    let [rims_corner, lids_corner] = match corner {
        RimsCorner::OnThePole => [
            Point::new(0.0, 0.0, RADIUS),
            Point::new(0.0, HEIGHT, the_lids_radius(shape)),
        ],
        RimsCorner::OnTheEquator => [
            Point::new(RADIUS, 0.0, 0.0),
            Point::new(the_lids_radius(shape), HEIGHT, 0.0),
        ],
    };
    let mut builder = SolidBuilder::new();
    let on_the_rim = builder.add_vertex(rims_corner).expect("a corner");
    let on_the_lid = builder.add_vertex(lids_corner).expect("a corner");
    let rim = builder
        .add_edge(
            on_the_rim,
            on_the_rim,
            EdgeRun::RightRoundWhereItsFacesCross,
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
    let the_cone = |builder: &mut SolidBuilder| {
        let steps = [
            (rim, ball_uses_the_rim.reversed()),
            (seam, Orientation::Forward),
            (round_the_lid, lid_uses_its_circle.reversed()),
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
        builder.add_face(cone, &[wire]).expect("a face")
    };
    let faces = match first {
        FirstInTheShell::Ball => {
            let ball = the_ball(&mut builder);
            [ball, the_cone(&mut builder)]
        }
        FirstInTheShell::Cone => {
            let cone = the_cone(&mut builder);
            [cone, the_ball(&mut builder)]
        }
    };
    let over = builder
        .add_coedge(round_the_lid, lid_uses_its_circle)
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
