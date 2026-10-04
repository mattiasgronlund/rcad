//! The half ball `z ≥ 0` of radius 5, its skin holed along the plane
//! `y + 3z = 15` and the hole capped on the same ball, built face by face.
//!
//! The plane cuts the ball in a circle through the north pole and through
//! `(0, 3, 4)`, the two ends of one of its diameters. So the solid has three
//! faces: the flat floor, the cap inside the circle, and the skin between the
//! equator and the circle, whose hole runs through the pole. The rim between
//! the skin and the cap has the ball on both sides, so it is written
//! `RightRoundWhereItCrosses { and: the plane }`.
//!
//! In the ball's two coordinates the skin is a band: the equator goes right
//! round, and so does the hole, because a smooth curve through a pole leaves it
//! half a turn round from the meridian it came up. The hole's side along the
//! pole, round the skin, is half a turn and a little. The mesher cuts a band
//! into four patches, a side at a time, each side the short way round, and the
//! short way round a pole side longer than half a turn is the other way. The
//! patch folded across itself, and every spelling was refused
//! `BoundaryCrossesItself` by `SolidBuilder::build` and by `SolidView::mesh`
//! at every deflection (Q582). Now the band gets a corner wherever such a side
//! passes a quarter, and a cut that lands on the end of a pole side lands on
//! the corner below it instead (`cut_a_ring_across` in `mesh.rs`).
//!
//! **Which spellings are right.** With the corner at `(0, 3, 4)` the rim is
//! stored running the way the ball's longitude grows there, whichever way the
//! plane faces, and the skin walks it `Reversed`; `Forward` is refused
//! `AClosedEdgeIsWalkedAgainstItsSurface`. With the corner on the pole the
//! ball's longitude says nothing, and the walk's own order is the one kept:
//! measured from the pole, it leaves towards `+x` for the plane facing
//! `(0, 1, 3)` and towards `−x` for it turned over. The skin walks the hole the
//! same way round in space in every right spelling, so on the pole it is
//! `Reversed` for the first and `Forward` for the second. The other two pole
//! spellings get past `Model::validate` today, which is Q651's to settle.
//!
//! The oracle is half the ball's volume, `250π/3 = 261.799388`.

use core::f64::consts::PI;
use std::collections::HashMap;

use rcad_geom::{Deflection, EdgeRun, Orientation, Plane, Point, Scale, Sphere, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, SolidBuilder, SolidKey};

const RADIUS: f64 = 5.0;

/// The four ways of writing the rim with its corner at `(0, 3, 4)` and the two
/// with it on the pole, each with the skin's hole listed after its equator and
/// before it, at the four deflections the refusal was found at.
#[test]
fn every_right_spelling_meshes_closed_and_measures_half_the_ball() {
    let closed_form = 2.0 * PI * RADIUS.powi(3) / 3.0;
    assert_eq!(format!("{closed_form:.6}"), "261.799388");
    let plane = Plane::from_point_and_normal(Point::new(0.0, 3.0, 4.0), Vector::new(0.0, 1.0, 3.0))
        .expect("a plane");
    let off_the_pole = Point::new(0.0, 3.0, 4.0);
    let on_the_pole = Point::new(0.0, 0.0, RADIUS);
    for (corner, and, hole_on_the_skin) in [
        (off_the_pole, plane, Orientation::Reversed),
        (off_the_pole, plane.reversed(), Orientation::Reversed),
        (on_the_pole, plane, Orientation::Reversed),
        (on_the_pole, plane.reversed(), Orientation::Forward),
    ] {
        for hole_first in [false, true] {
            let what = format!(
                "corner {corner:?}, and {:?}, hole {hole_on_the_skin:?} on the skin, \
                 hole first {hole_first}",
                and.normal()
            );
            let mut model = Model::new(Scale::millimetres(125.0).expect("a positive size"));
            let solid = a_holed_half_ball(&mut model, corner, and, hole_on_the_skin, hole_first);
            for deflection in [1.0, 0.1, 0.01, 0.005] {
                meshes_closed_and_measures(
                    &model,
                    solid,
                    deflection,
                    closed_form,
                    &format!("{what}, deflection {deflection}"),
                );
            }
        }
    }
}

/// The half ball `z ≥ 0` with its skin holed along `and` and the hole capped
/// on the same ball, the rim's one corner at `corner`.
fn a_holed_half_ball(
    model: &mut Model,
    corner: Point,
    and: Plane,
    hole_on_the_skin: Orientation,
    hole_first: bool,
) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let on_the_equator = builder
        .add_vertex(Point::new(RADIUS, 0.0, 0.0))
        .expect("a point");
    let equator = builder
        .add_edge(
            on_the_equator,
            on_the_equator,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a closed edge");
    let corner = builder.add_vertex(corner).expect("a point");
    let rim = builder
        .add_edge(
            corner,
            corner,
            EdgeRun::RightRoundWhereItCrosses {
                and: Surface::Plane(and),
            },
        )
        .expect("a closed edge");
    let mut walking = |edge, orientation| {
        let coedge = builder
            .add_coedge(edge, orientation)
            .expect("an edge of this builder");
        builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("a closed wire")
    };
    let equator_on_the_skin = walking(equator, Orientation::Forward);
    let equator_on_the_floor = walking(equator, Orientation::Reversed);
    let hole = walking(rim, hole_on_the_skin);
    let rim_of_the_cap = walking(rim, hole_on_the_skin.reversed());
    let ball = Surface::Sphere(Sphere::new(Point::origin(), RADIUS).expect("a ball"));
    let skin_wires = if hole_first {
        [hole, equator_on_the_skin]
    } else {
        [equator_on_the_skin, hole]
    };
    let skin = builder
        .add_face(ball.clone(), &skin_wires)
        .expect("a face with two wires");
    let cap = builder
        .add_face(ball, &[rim_of_the_cap])
        .expect("a face with one wire");
    let floor = builder
        .add_face(
            Surface::Plane(
                Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
                    .expect("a plane"),
            ),
            &[equator_on_the_floor],
        )
        .expect("a face with one wire");
    builder
        .add_shell(&[
            (skin, Orientation::Forward),
            (cap, Orientation::Forward),
            (floor, Orientation::Forward),
        ])
        .expect("three faces close up");
    let solid = builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a holed and capped half ball is a solid");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a holed and capped half ball is a sound model");
    solid
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
    model: &Model,
    solid: SolidKey,
    deflection: f64,
    closed_form: f64,
    what: &str,
) {
    let view = model.solid(solid).expect("this model made the key");
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
}
