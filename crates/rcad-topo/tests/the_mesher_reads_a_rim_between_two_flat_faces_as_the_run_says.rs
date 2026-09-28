//! A ball cut in two by a plane it does not share a face with, and which half
//! of the ball each face's triangles land on.
//!
//! The file's name is the queue row's (§222.1 item 1). The mesher kept its own
//! narrower copy of `rcad_geom::which_two_surfaces_a_run_is_between`, which
//! swapped the run's curved surface in only where both faces are flat. For two
//! flat faces the copy and the shared function agree on every run that gets
//! past `nothing_curved_says_where_it_bends`, so no draw of that kind can tell
//! them apart. Where they differ is a rim between two faces on **one** curved
//! surface, which the fuse writes as `EdgeRun::RightRoundWhereItCrosses { and }`
//! naming the plane that cut it: the copy left the pair as `[ball, ball]`, and
//! the shared function reads `[ball, plane]`.
//!
//! # What the pair is for
//!
//! Only for which way round the mesher walks a closed edge. `[ball, ball]` has
//! no crossing direction at all, so the stored-way rule gave nothing and the
//! mesher fell back to "the ball's longitude at the corner". That is the same
//! answer as the rule wherever the longitude says something. Where it says
//! exactly nothing — the corner on the ball's equator and the plane standing
//! upright, so the rim leaves the corner straight up or straight down — the
//! fallback compares the first walked point with the longitude, and the only
//! thing left in that product is how the circle curves back towards its centre.
//! So it answered the same way whichever way the walk went, and the edge was
//! walked the way the walker happened to produce it.
//!
//! # What that cost, measured
//!
//! `tests/a_rim_that_names_its_cutting_surface.rs`'s draw — a block and a ball
//! clear of it, united — with the ball half a unit to one side of the block's
//! plane `y = 0`, and the whole draw turned about `z`. Turning it puts the
//! rim's corner on the ball's equator (`Plane::basis` gives an upright plane a
//! level first direction). Of the eight draws below, the five with the ball on
//! the far side of the plane came back from the narrow copy with **the two
//! faces' triangles swapped**: the cap's face carried the rest of the ball,
//! about `9.406` of area where the cap is `π = 3.142`, and the rest's face
//! carried the cap, `3.131` where the rest is `3π = 9.425`, every point of each
//! on the wrong side of the plane. The three with the ball on the near side came
//! out right by the same luck the other way. The union of the two faces is
//! still the ball, so the volume and the closed-ness of the mesh did not move,
//! and nothing in the suite read which face a triangle belongs to.
//!
//! # The oracle
//!
//! Which side of the plane each face lies on is read here off the model and
//! the documented rule, not off the mesher: the edge is stored running the way
//! the ball's longitude increases, or where that says exactly nothing its
//! latitude (`rcad_geom::which_way_a_closed_crossing_turns`, which the fuse
//! writes with and `Model::validate` reads with); a coedge used forwards walks
//! it that way; and a face lies to the left of its wire seen from outside. The
//! unturned draw, where the longitude does say something and both copies always
//! agreed, is the check that this reading of the convention is the right one.

use rcad_geom::{Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{FaceView, Mesh, Model, Rule, SolidKey, SolidView};

const RADIUS: f64 = 1.0;

/// How far the ball's centre stands off the block's plane `y = 0`, before the
/// draw is turned. The cap on the far side of the plane is `RADIUS − OFF` high.
const OFF: f64 = 0.5;

const DEFLECTION: f64 = 0.01;

/// The draw turned about `z`, so the plane that cuts the ball stands upright
/// with a normal along no axis, and the rim's corner sits on the equator.
///
/// Turned and off to either side as listed: not every turn leaves the ball cut
/// at all (`a_rim_that_names_its_cutting_surface.rs` names that and does not
/// answer it), and these are the ones that do.
#[test]
fn each_half_of_a_ball_cut_upright_through_its_equator_is_meshed_on_its_own_side() {
    let draws = [
        (0.3, 1.0),
        (0.3, -1.0),
        (0.5, 1.0),
        (0.5, -1.0),
        (2.0, 1.0),
        (2.0, -1.0),
        (4.0, -1.0),
        (5.5, -1.0),
    ];
    let wrong: Vec<String> = draws
        .into_iter()
        .flat_map(|(turned, side)| {
            let what = format!("turned {turned} about z, the ball {side} × {OFF} off y = 0");
            let (model, ball) = a_block_and_a_ball_clear_of_it_united(turned, side * OFF);
            let rim = the_rim_between_the_halves(&model, ball, &what);
            assert_eq!(
                rim.along_the_longitude, 0.0,
                "{what}: the corner is meant to be where the ball's longitude says nothing"
            );
            what_is_wrong_with_each_half(&model, ball, &rim, &what)
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The draw as the fuzzer found it, where the rim's corner is the bottom of
/// the circle and the longitude says which way the rim runs.
///
/// Both readings of the pair always agreed here, so this is the check on the
/// oracle rather than on the mesher.
#[test]
fn each_half_of_a_ball_cut_where_its_longitude_says_which_way_is_meshed_on_its_own_side() {
    for side in [1.0, -1.0] {
        let what = format!("unturned, the ball {side} × {OFF} off y = 0");
        let (model, ball) = a_block_and_a_ball_clear_of_it_united(0.0, side * OFF);
        let rim = the_rim_between_the_halves(&model, ball, &what);
        assert_ne!(
            rim.along_the_longitude, 0.0,
            "{what}: the corner is meant to be where the ball's longitude says which way"
        );
        let wrong = what_is_wrong_with_each_half(&model, ball, &rim, &what);
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    }
}

/// A cube of side 2 from `(-2, -2, 0)` and a ball of `RADIUS` at
/// `(1, off, 1)`, both turned `turned` about `z`, united; and the solid of the
/// answer that is the ball.
fn a_block_and_a_ball_clear_of_it_united(turned: f64, off: f64) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let policy = Policy::certified();
    let everything = Isometry::rotation(&rcad_geom::Axis::z(), turned).expect("a finite turn");
    let block = model
        .add_cube(2.0, &policy, &mut ProvenanceLog::new())
        .expect("a cube of side 2");
    model
        .place_solid(
            block,
            &everything.compose(
                &Isometry::translation(Vector::new(-2.0, -2.0, 0.0)).expect("a finite shift"),
            ),
        )
        .expect("this model made the key");
    let ball = model
        .add_ball(
            RADIUS,
            &everything.compose(
                &Isometry::translation(Vector::new(1.0, off, 1.0)).expect("a finite shift"),
            ),
        )
        .expect("a ball");
    let fused = model
        .fuse(
            &[block],
            &[ball],
            Rule::Union,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a block and a ball that miss each other unite");
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("the answer is a sound model");
    let balls: Vec<SolidKey> = fused
        .solids()
        .iter()
        .copied()
        .filter(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .faces()
                .all(|face| matches!(face.surface(), Surface::Sphere(_)))
        })
        .collect();
    let [ball] = balls[..] else {
        panic!("the union is the block and the ball apart: {balls:?}");
    };
    (model, ball)
}

/// The one closed edge between the two halves, read off the model.
struct Rim {
    corner: Point,
    centre: Point,
    cut: Plane,
    /// The way the edge is stored running at its corner, by the rule the fuse
    /// writes it with.
    stored_way: Vector,
    /// The wire's direction against the ball's longitude at the corner.
    along_the_longitude: f64,
}

fn the_rim_between_the_halves(model: &Model, ball: SolidKey, what: &str) -> Rim {
    let view = model.solid(ball).expect("this model made the key");
    assert_eq!(view.faces().count(), 2, "{what}: the ball comes back in two halves");
    let face = view.faces().next().expect("two faces");
    let Surface::Sphere(skin) = face.surface() else {
        panic!("{what}: a face of the ball is on the ball");
    };
    let edge = face
        .wires()
        .flat_map(|wire| wire.coedges())
        .map(|coedge| coedge.edge())
        .next()
        .expect("each half is bounded by the rim");
    let EdgeRun::RightRoundWhereItCrosses {
        and: Surface::Plane(cut),
    } = edge.run()
    else {
        panic!("{what}: the rim names the plane that cut it, and was {:?}", edge.run());
    };

    let corner = edge.start().point();
    let out = corner - skin.centre();
    let wire = out.cross(cut.normal());
    let longitude = Vector::new(0.0, 0.0, 1.0).cross(out);
    let along_the_longitude = wire.dot(longitude);
    let reference = if along_the_longitude == 0.0 {
        wire.dot(out.cross(longitude))
    } else {
        along_the_longitude
    };
    assert_ne!(reference, 0.0, "{what}: the rim's corner is not at a pole");
    Rim {
        corner,
        centre: skin.centre(),
        cut,
        stored_way: if reference > 0.0 { wire } else { -wire },
        along_the_longitude,
    }
}

/// Each face whose triangles are not all on the side of the cut its wire puts
/// it, or that does not measure the closed form of that side.
fn what_is_wrong_with_each_half(
    model: &Model,
    ball: SolidKey,
    rim: &Rim,
    what: &str,
) -> Vec<String> {
    let view = model.solid(ball).expect("this model made the key");
    let mesh = view
        .mesh(
            Deflection::of(DEFLECTION).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the ball meshes: {why:?}"));
    let strays = mesh.deviation().magnitude();
    let cap = 2.0 * core::f64::consts::PI * RADIUS * (RADIUS - OFF);
    let whole = 4.0 * core::f64::consts::PI * RADIUS * RADIUS;

    let mut wrong = Vec::new();
    for face in view.faces() {
        let side = the_side_of_the_cut_this_face_is_on(view, face, rim, what);
        let beyond_the_cut = side * (rim.centre - rim.corner).dot(rim.cut.normal()) < 0.0;
        let (area, worst) = what_this_face_was_meshed_as(&mesh, face, rim, side);
        let closed_form = if beyond_the_cut { cap } else { whole - cap };
        let under = (closed_form - area) / closed_form;
        if worst < -strays || !(0.0..3.0 * DEFLECTION / RADIUS).contains(&under) {
            wrong.push(format!(
                "{what}: the face on the {side} side of the cut measures {area} against \
                 {closed_form}, and a point of it is {worst} on its own side where the mesh \
                 strays no more than {strays}"
            ));
        }
    }
    wrong
}

/// `1.0` or `-1.0`: which side of the cutting plane, against its normal, the
/// face lies on — to the left of its own wire, seen from outside.
fn the_side_of_the_cut_this_face_is_on(
    view: SolidView<'_>,
    face: FaceView<'_>,
    rim: &Rim,
    what: &str,
) -> f64 {
    assert_eq!(
        view.faces()
            .filter(|each| each.key() == face.key())
            .map(|each| each.orientation())
            .collect::<Vec<_>>(),
        [Orientation::Forward],
        "{what}: a ball's faces are used forwards"
    );
    let [coedge] = face
        .wires()
        .flat_map(|wire| wire.coedges())
        .collect::<Vec<_>>()[..]
    else {
        panic!("{what}: each half has the rim and nothing else");
    };
    let walked = match coedge.orientation_on_its_edge() {
        Orientation::Forward => rim.stored_way,
        Orientation::Reversed => -rim.stored_way,
        other => panic!("{what}: a coedge turned {other:?}"),
    };
    let left = (rim.corner - rim.centre).cross(walked);
    let side = left.dot(rim.cut.normal());
    assert_ne!(side, 0.0, "{what}: the wire leaves the corner off the plane");
    side.signum()
}

/// How much area the triangles of `face` cover, and how far on its own side of
/// the cut its worst point is: negative where a point is on the wrong side.
fn what_this_face_was_meshed_as(mesh: &Mesh, face: FaceView<'_>, rim: &Rim, side: f64) -> (f64, f64) {
    let [a, b, c, d] = rim.cut.coefficients();
    let mut area = 0.0;
    let mut worst = f64::INFINITY;
    for triangle in mesh.triangles().iter().filter(|each| each.face() == face.key()) {
        let [p, q, r] = triangle.corners().map(|index| mesh.points()[index as usize]);
        area += (q - p).cross(r - p).length() / 2.0;
        for at in [p, q, r] {
            worst = worst.min(side * (a * at.x() + b * at.y() + c * at.z() + d));
        }
    }
    (area, worst)
}
