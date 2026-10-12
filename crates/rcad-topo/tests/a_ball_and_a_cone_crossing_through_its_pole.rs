//! A half ball under a frustum that stands on the ball's great circle, built
//! through the public door, with the circle's one corner on the ball's pole.
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
//! with the rim's direction turned (`docs/decisions.md` §341.5). The cone's
//! longitude is not zero there, and it turns round with the way the cone's
//! axis is written, which is not a fact about the shape. A cone is both its
//! nappes, so the axis written up or down is the one cone (§341.6).

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, Orientation, Plane, Point, Scale, Sphere, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, SolidBuilder, SolidKey, SolidView};

/// The radius of the ball, and of the cone where they cross.
const RADIUS: f64 = 1.5;
/// How far out the cone spreads for every unit along its axis.
const SLOPE: f64 = 0.5;
/// How tall the frustum stands above the rim.
const HEIGHT: f64 = 2.0;

/// A coarse, a middling and a fine deflection, each well under the radius.
const DEFLECTIONS: [f64; 3] = [0.05, 0.0137, 0.001];

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

/// With the rim's corner on the ball's pole, the frustum that flares out builds,
/// validates, meshes each face on its own side of the rim's plane and measures
/// its closed form, whichever way the ball uses the rim, whichever face is
/// listed first and whichever way the cone's axis is written.
#[test]
fn a_ball_and_a_cone_crossing_through_its_pole_build_however_they_are_written() {
    let wrong = what_each_writing_gets_wrong(Frustum::FlaresOut, RimsCorner::OnThePole);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn probe_narrowing() {
    let wrong = what_each_writing_gets_wrong(Frustum::NarrowsIn, RimsCorner::OnThePole);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn probe_equator_narrowing() {
    let wrong = what_each_writing_gets_wrong(Frustum::NarrowsIn, RimsCorner::OnTheEquator);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn probe_equator_flaring() {
    let wrong = what_each_writing_gets_wrong(Frustum::FlaresOut, RimsCorner::OnTheEquator);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Where on the ball the rim's one corner is.
#[derive(Debug, Clone, Copy)]
enum RimsCorner {
    OnThePole,
    OnTheEquator,
}

/// Each way of writing `shape` that is refused, does not validate, meshes a
/// face on the wrong side of the rim or measures off its closed form, with
/// what went wrong.
fn what_each_writing_gets_wrong(shape: Frustum, corner: RimsCorner) -> Vec<String> {
    let lids_radius = the_lids_radius(shape);
    let closed_form = core::f64::consts::PI
        * (2.0 * RADIUS.powi(3) / 3.0
            + HEIGHT / 3.0 * (RADIUS * RADIUS + RADIUS * lids_radius + lids_radius * lids_radius));
    let mut wrong = Vec::new();
    for first in [FirstInTheShell::Ball, FirstInTheShell::Cone] {
        for axis in [ConesAxisPoints::Up, ConesAxisPoints::Down] {
            for ball_uses_the_rim in [Orientation::Forward, Orientation::Reversed] {
                let what = format!(
                    "{shape:?} {corner:?}, the {first:?} listed first, the cone's axis pointing {axis:?}, \
                     the ball using the rim {ball_uses_the_rim:?}"
                );
                let (model, built) =
                    a_half_ball_under_a_frustum(shape, corner, first, axis, ball_uses_the_rim);
                let key = match built {
                    Ok(key) => key,
                    Err(why) => {
                        wrong.push(format!("{what}: refused {why:?}"));
                        continue;
                    }
                };
                if let Err(why) = model.validate(&Policy::certified(), &mut ProvenanceLog::new()) {
                    wrong.push(format!("{what}: does not validate: {why:?}"));
                    continue;
                }
                let solid = model.solid(key).expect("this model made the key");
                for deflection in DEFLECTIONS {
                    let what = format!("{what}, at {deflection}");
                    let deflection = Deflection::of(deflection).expect("a length");
                    if let Some(why) = where_a_face_is_meshed_off_its_side(solid, deflection) {
                        wrong.push(format!("{what}: {why}"));
                        continue;
                    }
                    match solid.measure(deflection, &Policy::certified(), &mut ProvenanceLog::new())
                    {
                        Ok(measured)
                            if (measured.volume() - closed_form).abs()
                                <= measured.volume_error() => {}
                        Ok(measured) => wrong.push(format!(
                            "{what}: measured {} ± {}, and the closed form is {closed_form}",
                            measured.volume(),
                            measured.volume_error()
                        )),
                        Err(why) => wrong.push(format!("{what}: does not measure: {why:?}")),
                    }
                }
            }
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
/// disc on top, with the rim's one corner on the ball's pole `(0, 0, RADIUS)`.
/// The ball uses the rim `ball_uses_the_rim` and the cone uses it the other
/// way.
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
