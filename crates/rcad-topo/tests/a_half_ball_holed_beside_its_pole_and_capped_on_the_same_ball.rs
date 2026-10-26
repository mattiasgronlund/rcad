//! The half ball `z ≥ 0` of radius 5, its skin holed along the plane
//! `y + z = 7` and the hole capped on the same ball, built face by face.
//!
//! The plane cuts the ball in a small circle round `(0, 3.5, 3.5)`, of radius
//! `√0.5`, through `(0, 3, 4)`, which is the rim's one corner. The north pole
//! is short of the plane, so it stays on the skin. The solid has three faces:
//! the flat floor, the cap beyond the plane, and the skin between the equator
//! and the rim. The rim has the ball on both sides, so it is written
//! `RightRoundWhereItCrosses { and: the plane }`.
//!
//! In the ball's two coordinates the skin's equator goes right round and its
//! hole does not: the skin is a cap holding the north pole, with a hole in it.
//! When Q374's probe found this solid the mesher refused the skin
//! `AWireWillNotLieFlatOnItsFace`, for a face with one wire going right round
//! beside one that does not (Q583). Q453 (`eba7bec6`) since cuts a cap on a
//! ball with islands in it up to its pole, each island kept whole in the patch
//! that holds it (`cut_a_cap_up_to_its_pole` in `mesh.rs`), and this skin takes
//! that path: with `a_cap_on_a_ball_or_a_cone_with_islands_in_it` made to
//! answer `false`, the builder refuses the skin `AWireWillNotLieFlatOnItsFace`
//! again (Q583's lane, on the plane facing out with the hole listed second).
//!
//! **Which spellings are right.** At `(0, 3, 4)` the ball's longitude grows
//! towards `−x`, and walking the rim that way keeps the skin on the left, so
//! the skin walks it `Forward` whichever way the plane faces. The corner is off
//! the pole, so `Model::validate` reads the way the rim is stored, and the
//! builder, which runs it first, refuses the other use.
//!
//! The oracle is half the ball's volume, `250π/3 = 261.799388`.

use core::f64::consts::PI;
use std::collections::HashMap;

use rcad_geom::{Deflection, EdgeRun, Orientation, Plane, Point, Scale, Sphere, Surface, Vector};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{
    DraftStep, Error, FaceKey, Invalid, Mesh, Model, OnAFace, Result, SolidBuilder, SolidKey,
    SolidView,
};

const RADIUS: f64 = 5.0;

/// How far a point of the rim may read off the plane `y + z = 7`: the walk
/// puts it on the plane only to rounding, a few units in the last place of 7.
const ROUNDING: f64 = 1e-12;

/// The four right spellings — the plane either way round, the skin's hole
/// listed after its equator and before it — at the four deflections the
/// sibling solid holed through the pole is pinned at.
#[test]
fn every_right_spelling_meshes_closed_and_measures_half_the_ball() {
    let closed_form = 2.0 * PI * RADIUS.powi(3) / 3.0;
    assert_eq!(format!("{closed_form:.6}"), "261.799388");
    for (and, hole_first) in every_spelling() {
        let what = format!("and {:?}, hole first {hole_first}", and.normal());
        let mut model = Model::new(Scale::millimetres(125.0).expect("a positive size"));
        let solid = a_holed_half_ball(&mut model, and, Orientation::Forward, hole_first)
            .unwrap_or_else(|why| {
                panic!("{what}: a holed and capped half ball is a solid: {why:?}")
            });
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{what}: the model is sound: {why:?}"));
        let view = model.solid(solid).expect("this model made the key");
        for deflection in [1.0, 0.1, 0.01, 0.005] {
            let what = format!("{what}, deflection {deflection}");
            let mesh = meshes_closed_and_measures(view, deflection, closed_form, &what);
            each_face_keeps_its_side_of_the_plane(view, &mesh, &what);
        }
    }
}

/// The skin holds the pole, a point just short of the rim and one on the far
/// side of the ball, and not the cap's middle or a point just beyond the rim;
/// the cap holds the opposite. A solid walked the wrong way round can still
/// measure right, so this is what says it is the solid meant.
#[test]
fn the_skin_and_the_cap_each_hold_their_own_side_of_the_rim() {
    let on_the_ball = |x: f64, y: f64| {
        PredicatePoint::from(
            Point::new(x, y, (RADIUS * RADIUS - x * x - y * y).sqrt()).coordinates(),
        )
    };
    let beyond = [
        on_the_ball(0.0, RADIUS / 2f64.sqrt()),
        on_the_ball(0.0, 3.6),
    ];
    let short = [
        on_the_ball(0.0, 0.0),
        on_the_ball(0.0, 2.9),
        on_the_ball(0.0, -3.0),
    ];
    for (and, hole_first) in every_spelling() {
        let what = format!("and {:?}, hole first {hole_first}", and.normal());
        let mut model = Model::new(Scale::millimetres(125.0).expect("a positive size"));
        let solid = a_holed_half_ball(&mut model, and, Orientation::Forward, hole_first)
            .unwrap_or_else(|why| {
                panic!("{what}: a holed and capped half ball is a solid: {why:?}")
            });
        let view = model.solid(solid).expect("this model made the key");
        let [skin, cap] = the_skin_and_the_cap(view);
        let holds = |face: FaceKey, point| {
            view.face_holds_the_point(face, point, &Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what}: the face answers at {point:?}: {why:?}"))
        };
        for point in beyond {
            assert_eq!(
                holds(skin, point),
                OnAFace::NotHeld,
                "{what}: the skin at {point:?}"
            );
            assert_eq!(
                holds(cap, point),
                OnAFace::Held,
                "{what}: the cap at {point:?}"
            );
        }
        for point in short {
            assert_eq!(
                holds(skin, point),
                OnAFace::Held,
                "{what}: the skin at {point:?}"
            );
            assert_eq!(
                holds(cap, point),
                OnAFace::NotHeld,
                "{what}: the cap at {point:?}"
            );
        }
    }
}

/// The skin walking its hole `Reversed`, against the way the rim is stored, is
/// refused by the builder in all four spellings, so it never reaches the
/// mesher through it.
#[test]
fn the_hole_walked_against_the_ball_is_refused_by_the_builder() {
    for (and, hole_first) in every_spelling() {
        let mut model = Model::new(Scale::millimetres(125.0).expect("a positive size"));
        let built = a_holed_half_ball(&mut model, and, Orientation::Reversed, hole_first);
        assert!(
            matches!(
                built,
                Err(Error::Invalid(
                    Invalid::AClosedEdgeIsWalkedAgainstItsSurface
                ))
            ),
            "and {:?}, hole first {hole_first}: {built:?}",
            and.normal()
        );
    }
}

/// The plane either way round, and the skin's hole listed after its equator
/// and before it.
fn every_spelling() -> [(Plane, bool); 4] {
    let plane = Plane::from_point_and_normal(Point::new(0.0, 3.0, 4.0), Vector::new(0.0, 1.0, 1.0))
        .expect("a plane");
    [
        (plane, false),
        (plane, true),
        (plane.reversed(), false),
        (plane.reversed(), true),
    ]
}

/// The ball's face with two wires, and the one with one.
fn the_skin_and_the_cap(solid: SolidView<'_>) -> [FaceKey; 2] {
    let on_the_ball = |wires: usize| {
        solid
            .faces()
            .find(|face| face.surface().as_plane().is_none() && face.wire_count() == wires)
            .expect("a face of the ball with that many wires")
            .key()
    };
    [on_the_ball(2), on_the_ball(1)]
}

/// The half ball `z ≥ 0` with its skin holed along `and`, walking the hole
/// `hole_on_the_skin`, and the hole capped on the same ball, the rim's one
/// corner at `(0, 3, 4)`.
fn a_holed_half_ball(
    model: &mut Model,
    and: Plane,
    hole_on_the_skin: Orientation,
    hole_first: bool,
) -> Result<SolidKey> {
    let mut builder = SolidBuilder::new();
    let on_the_equator = builder.add_vertex(Point::new(RADIUS, 0.0, 0.0))?;
    let equator = builder.add_edge(
        on_the_equator,
        on_the_equator,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;
    let corner = builder.add_vertex(Point::new(0.0, 3.0, 4.0))?;
    let rim = builder.add_edge(
        corner,
        corner,
        EdgeRun::RightRoundWhereItCrosses {
            and: Surface::Plane(and),
        },
    )?;
    let mut walking = |edge, orientation| -> Result<_> {
        let coedge = builder.add_coedge(edge, orientation)?;
        builder.add_wire(&[DraftStep::Along(coedge)])
    };
    let equator_on_the_skin = walking(equator, Orientation::Forward)?;
    let equator_on_the_floor = walking(equator, Orientation::Reversed)?;
    let hole = walking(rim, hole_on_the_skin)?;
    let rim_of_the_cap = walking(rim, hole_on_the_skin.reversed())?;
    let ball = Surface::Sphere(Sphere::new(Point::origin(), RADIUS).expect("a ball"));
    let skin_wires = if hole_first {
        [hole, equator_on_the_skin]
    } else {
        [equator_on_the_skin, hole]
    };
    let skin = builder.add_face(ball, &skin_wires)?;
    let cap = builder.add_face(ball, &[rim_of_the_cap])?;
    let floor = builder.add_face(
        Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
                .expect("a plane"),
        ),
        &[equator_on_the_floor],
    )?;
    builder.add_shell(&[
        (skin, Orientation::Forward),
        (cap, Orientation::Forward),
        (floor, Orientation::Forward),
    ])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// The solid meshes into a closed surface with the shape of a ball, its volume
/// is under `closed_form` by no more than `3δ/r` of it, and `closed_form` is
/// within the error the measure publishes.
///
/// Closed means every side of every triangle is used once each way round; the
/// shape of a ball means points less sides plus triangles is two. The `3δ/r`
/// bound is `a_cap_whose_rim_runs_through_the_other_pole.rs`'s: the missing
/// shell is at most the curved area times `δ`, and a piece holding the ball's
/// centre has volume at least that area times `r/3`.
fn meshes_closed_and_measures(
    view: SolidView<'_>,
    deflection: f64,
    closed_form: f64,
    what: &str,
) -> Mesh {
    let mesh = view
        .mesh(
            Deflection::of(deflection).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the solid meshes: {why:?}"));
    let mut times_used: HashMap<(u32, u32), i32> = HashMap::new();
    for triangle in mesh.triangles() {
        let [first, second, third] = triangle.corners();
        for (from, to) in [(first, second), (second, third), (third, first)] {
            *times_used.entry((from.min(to), from.max(to))).or_default() +=
                if from < to { 1 } else { -1 };
        }
    }
    for (side, balance) in &times_used {
        assert_eq!(
            *balance, 0,
            "{what}: the side {side:?} is used {balance} more times one way than the other"
        );
    }
    let points = mesh.points().len() as i64;
    assert_eq!(
        points - times_used.len() as i64 + mesh.triangles().len() as i64,
        2,
        "{what}: {points} points, {} sides, {} triangles",
        times_used.len(),
        mesh.triangles().len()
    );

    let measured = view
        .measure(
            Deflection::of(deflection).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the solid measures: {why:?}"));
    let found = measured.volume();
    let under = (closed_form - found) / closed_form;
    assert!(
        (0.0..3.0 * deflection / RADIUS).contains(&under),
        "{what}: measured {found}, closed form {closed_form}, {:.4}% under",
        100.0 * under
    );
    assert!(
        (closed_form - found).abs() <= measured.volume_error(),
        "{what}: measured {found}, closed form {closed_form}, published error {}",
        measured.volume_error()
    );
    mesh
}

/// Every corner of a skin triangle is short of the plane `y + z = 7` or on it,
/// every corner of a cap triangle beyond it or on it, and both faces have
/// triangles. A skin filled across its hole, or a cap filled as the rest of
/// the ball, puts corners on the wrong side.
fn each_face_keeps_its_side_of_the_plane(view: SolidView<'_>, mesh: &Mesh, what: &str) {
    let [skin, cap] = the_skin_and_the_cap(view);
    let mut triangles = [0; 2];
    for triangle in mesh.triangles() {
        let beyond = triangle.corners().map(|corner| {
            let point = mesh.points()[corner as usize];
            point.y() + point.z() - 7.0
        });
        if triangle.face() == skin {
            triangles[0] += 1;
            assert!(
                beyond.iter().all(|&by| by <= ROUNDING),
                "{what}: a skin triangle beyond the plane by {beyond:?}"
            );
        } else if triangle.face() == cap {
            triangles[1] += 1;
            assert!(
                beyond.iter().all(|&by| by >= -ROUNDING),
                "{what}: a cap triangle short of the plane by {beyond:?}"
            );
        }
    }
    assert!(
        triangles.iter().all(|&count| count > 0),
        "{what}: skin and cap triangles {triangles:?}"
    );
}
