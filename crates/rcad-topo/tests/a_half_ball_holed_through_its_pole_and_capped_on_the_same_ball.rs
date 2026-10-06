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
//! ball's longitude and latitude both vanish, so nothing about the edge says
//! which way it is stored running, and `Model::validate` skips it. The mesher
//! reads the way off the skin instead: its corner on the equator is on the
//! plane's negative side, so its use walks the rim along `outward × ∇plane`,
//! and the edge runs the way that use says (§341.5). So the skin's use of the
//! hole is a label on the pole, and all four pole spellings are one solid —
//! `the_four_ways_of_writing_the_rim_on_the_pole_are_one_solid` is the
//! demonstration, and `docs/decisions.md` §220.6 the argument (Q941).
//!
//! The oracle is half the ball's volume, `250π/3 = 261.799388`.

use core::f64::consts::PI;
use std::collections::HashMap;

use rcad_geom::{Deflection, EdgeRun, Orientation, Plane, Point, Scale, Sphere, Surface, Vector};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{
    DraftStep, Error, FaceKey, FaceView, Mesh, Model, OnAFace, SolidBuilder, SolidKey, SolidView,
};

const RADIUS: f64 = 5.0;

/// The two ways of writing the rim with its corner at `(0, 3, 4)` and the four
/// with it on the pole, each with the skin's hole listed after its equator and
/// before it, at the four deflections the refusal was found at.
#[test]
fn every_right_spelling_meshes_closed_and_measures_half_the_ball() {
    let closed_form = 2.0 * PI * RADIUS.powi(3) / 3.0;
    assert_eq!(format!("{closed_form:.6}"), "261.799388");
    let plane = the_rims_plane();
    let off_the_pole = Point::new(0.0, 3.0, 4.0);
    let on_the_pole = Point::new(0.0, 0.0, RADIUS);
    for (corner, and, hole_on_the_skin) in [
        (off_the_pole, plane, Orientation::Reversed),
        (off_the_pole, plane.reversed(), Orientation::Reversed),
        (on_the_pole, plane, Orientation::Reversed),
        (on_the_pole, plane, Orientation::Forward),
        (on_the_pole, plane.reversed(), Orientation::Reversed),
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

/// The four ways of writing the rim on the pole — the plane either way round,
/// the skin's use of the hole either way round — are one solid, and not two
/// right spellings beside two wrong ones that happen to measure right.
///
/// **What a face holds is what tells**, because a solid walked the wrong way
/// round can still measure right. In all four, the skin holds the three points
/// beside the pole off the cap and neither of the two on it; the cap's own
/// answer is the same in all four; the mesh's triangles are the same triangles,
/// corner for corner and wound the same way, for the two uses of one plane;
/// every skin triangle is short of the plane and every cap triangle beyond it;
/// and the skin walks away from the pole towards `−x`, the way that keeps the
/// cap on its right. The rim's points leave the pole the way the skin's use
/// says the edge runs, so the use is a label and the walk in space is not.
///
/// The cap's answer is a refusal, `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn`:
/// its one corner is on the plane, and the rule that would read its side off its
/// wire reads nothing on the pole. That is a gap of its own (§220.6) and the
/// same in every spelling, which is all this asks of it.
#[test]
fn the_four_ways_of_writing_the_rim_on_the_pole_are_one_solid() {
    let plane = the_rims_plane();
    let near_the_pole = |x: f64, y: f64| {
        let point = Point::new(x, y, (RADIUS * RADIUS - x * x - y * y).sqrt());
        PredicatePoint::from(point.coordinates())
    };
    let on_the_cap = [near_the_pole(0.0, 0.3), near_the_pole(0.0, 1.5)];
    let off_the_cap = [
        near_the_pole(0.0, -0.3),
        near_the_pole(0.3, 0.0),
        near_the_pole(-0.3, 0.0),
    ];
    for hole_first in [false, true] {
        for and in [plane, plane.reversed()] {
            let mut meshes = Vec::new();
            for hole_on_the_skin in [Orientation::Reversed, Orientation::Forward] {
                let what = format!(
                    "and {:?}, hole {hole_on_the_skin:?} on the skin, hole first {hole_first}",
                    and.normal()
                );
                let mut model = Model::new(Scale::millimetres(125.0).expect("a positive size"));
                let solid = a_holed_half_ball(
                    &mut model,
                    Point::new(0.0, 0.0, RADIUS),
                    and,
                    hole_on_the_skin,
                    hole_first,
                );
                let view = model.solid(solid).expect("this model made the key");
                let [skin, cap] = the_skin_and_the_cap(view);
                let holds = |face: FaceKey, point| {
                    view.face_holds_the_point(
                        face,
                        point,
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    )
                };
                for point in on_the_cap {
                    assert_eq!(
                        holds(skin.key(), point).expect("the skin answers"),
                        OnAFace::NotHeld,
                        "{what}: the skin does not hold {point:?}, on the cap"
                    );
                }
                for point in off_the_cap {
                    assert_eq!(
                        holds(skin.key(), point).expect("the skin answers"),
                        OnAFace::Held,
                        "{what}: the skin holds {point:?}, off the cap"
                    );
                }
                for point in on_the_cap.into_iter().chain(off_the_cap) {
                    let answer = holds(cap.key(), point);
                    assert!(
                        matches!(
                            answer,
                            Err(Error::NothingSaysWhichSideOfItsTrimmingAFaceLiesOn { face })
                                if face == cap.key()
                        ),
                        "{what}: the cap's answer at {point:?}: {answer:?}"
                    );
                }

                let mesh = view
                    .mesh(
                        Deflection::of(0.05).expect("a positive length"),
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    )
                    .unwrap_or_else(|why| panic!("{what}: the solid meshes: {why:?}"));
                for triangle in mesh.triangles() {
                    let beyond = triangle
                        .corners()
                        .map(|corner| mesh.points()[corner as usize])
                        .iter()
                        .map(|point| point.y() + 3.0 * point.z())
                        .sum::<f64>()
                        > 45.0;
                    if triangle.face() == skin.key() {
                        assert!(!beyond, "{what}: a skin triangle beyond the plane");
                    } else if triangle.face() == cap.key() {
                        assert!(beyond, "{what}: a cap triangle short of the plane");
                    }
                }

                let rim = cap
                    .outer_wire()
                    .coedges()
                    .next()
                    .expect("the cap's wire is the rim")
                    .edge();
                let skins_use = skin
                    .wires()
                    .flat_map(|wire| wire.coedges().collect::<Vec<_>>())
                    .find(|coedge| coedge.edge().key() == rim.key())
                    .expect("the skin walks the rim")
                    .orientation_on_its_edge();
                let along = mesh.points_along_edge(rim.key());
                let leaves_towards = mesh.points()[along[1] as usize].x();
                let skin_leaves_towards = match skins_use {
                    Orientation::Forward => leaves_towards,
                    Orientation::Reversed => -leaves_towards,
                };
                assert!(
                    skin_leaves_towards < 0.0,
                    "{what}: the rim's points leave the pole towards x {leaves_towards}, \
                     and the skin walks them {skins_use:?}"
                );
                meshes.push(the_triangles_where_they_are(&mesh));
            }
            assert!(
                meshes[0] == meshes[1],
                "and {:?}, hole first {hole_first}: the two uses of the hole mesh the same \
                 triangles",
                and.normal()
            );
        }
    }
}

/// The plane `y + 3z = 15`, facing away from the ball's centre.
fn the_rims_plane() -> Plane {
    Plane::from_point_and_normal(Point::new(0.0, 3.0, 4.0), Vector::new(0.0, 1.0, 3.0))
        .expect("a plane")
}

/// The ball's face with two wires, and the one with one.
fn the_skin_and_the_cap(solid: SolidView<'_>) -> [FaceView<'_>; 2] {
    let on_the_ball = |wires: usize| {
        solid
            .faces()
            .find(|face| face.surface().as_plane().is_none() && face.wire_count() == wires)
            .expect("a face of the ball with that many wires")
    };
    [on_the_ball(2), on_the_ball(1)]
}

/// Every triangle of `mesh` as its three corners in space, each turned to
/// start at its least corner so that its winding is kept, and sorted: two meshes
/// with the same list are the same triangles, whatever their points are
/// numbered.
fn the_triangles_where_they_are(mesh: &Mesh) -> Vec<[[u64; 3]; 3]> {
    let mut triangles: Vec<[[u64; 3]; 3]> = mesh
        .triangles()
        .iter()
        .map(|triangle| {
            let mut corners = triangle.corners().map(|corner| {
                mesh.points()[corner as usize]
                    .coordinates()
                    .map(f64::to_bits)
            });
            let least = (0..3).min_by_key(|&at| corners[at]).expect("three corners");
            corners.rotate_left(least);
            corners
        })
        .collect();
    triangles.sort_unstable();
    triangles
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
        .add_face(ball, &skin_wires)
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
