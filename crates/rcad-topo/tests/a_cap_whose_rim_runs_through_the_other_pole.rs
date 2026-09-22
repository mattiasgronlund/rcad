//! A ball cut by a plane through one of its poles, keeping the other pole.
//!
//! The piece kept is a cap on the ball whose rim runs through the pole it does
//! **not** hold. Its skin is bounded by one closed circle, so the mesher cuts
//! it into four patches up to the pole it holds, and which way round those
//! four go is read off which way the rim runs. The rim leaves the pole it runs
//! through up the meridian half a turn from the one it came down, so in the
//! ball's two coordinates its walk crosses a pole side there, and on a coarse
//! walk that side is most of a turn.
//!
//! Before this file every one of the 63 caps below was refused by
//! `SolidBuilder::build`, which meshes each face coarsely to check it, for two
//! reasons (`docs/decisions.md` §198.1 and §199.4 carried the first as open):
//!
//! - `runs_up` added the rim's corners up the short way round, which reads a
//!   pole side wider than half a turn as the long way back. It said `0` for a
//!   rim the walk had counted a whole turn up, the cap was cut the wrong way
//!   round, and 12 of the 63 were refused `FacesDoNotAgreeWhichWayTheyFace`.
//! - The other 51 were cut up to the far pole from the corners nearest each
//!   quarter turn, and with most of the turn spent on the pole side several
//!   quarters found the same corner: 45 refused `CornersCoincide` and 6
//!   `BoundaryCrossesItself`. The cuts now leave from the pole side itself,
//!   and never from the pole corner, whose meridian the rim runs up.
//!
//! The solid is built face by face, because `Model::add_ball` puts a seam
//! down the ball whose end is the very pole the rim passes through, and a fuse
//! refuses that with `CannotTellTheCellsOfAFaceApart` before anything is
//! meshed. Two faces: the skin, bounded by the closed circle where the ball
//! meets the plane, and the flat lid across it. The circle's one corner is the
//! pole itself, which is on the ball exactly and on the plane exactly for a
//! radius that is a power of two.
//!
//! The oracle is the closed form of a cap, and the mesh must close up with the
//! shape of a ball.

use core::f64::consts::PI;
use std::collections::HashMap;

use rcad_geom::{Deflection, EdgeRun, Orientation, Plane, Point, Scale, Sphere, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, SolidBuilder, SolidKey};

/// Leaning the plane by `lean` off square to the polar axis, and turning the
/// way it leans by `turn` about that axis, so that the rim leaves the pole it
/// runs through along meridians everywhere round the ball, the chart's own
/// `±π` among them.
#[test]
fn a_ball_cut_through_its_south_pole_keeps_its_north_pole_and_measures_a_cap() {
    for radius in [0.5, 1.0, 2.0] {
        for lean in [0.3, 0.7, 1.1] {
            for turn in [0.0, 0.4, 1.3, PI / 2.0, 2.5, PI, -2.0] {
                let mut model = a_model(radius);
                let cap = a_cap_through_the_south_pole(&mut model, radius, lean, turn);
                meshes_closed_and_measures(
                    &model,
                    cap,
                    0.001 * radius,
                    radius,
                    a_caps_volume(radius, radius + radius * lean.cos()),
                    &format!("radius {radius}, leaning {lean}, turned {turn}"),
                );
            }
        }
    }
}

fn a_model(radius: f64) -> Model {
    Model::new(Scale::millimetres(25.0 * radius).expect("a positive size"))
}

/// The part of the ball of `radius` about the origin on the north pole's side
/// of a plane through its south pole. The plane's normal leans `lean` off the
/// downward polar axis towards the direction `turn` round from `x`, and points
/// out of the material.
fn a_cap_through_the_south_pole(model: &mut Model, radius: f64, lean: f64, turn: f64) -> SolidKey {
    let south = Point::new(0.0, 0.0, -radius);
    let lid = Plane::from_point_and_normal(
        south,
        Vector::new(
            lean.sin() * turn.cos(),
            lean.sin() * turn.sin(),
            -lean.cos(),
        ),
    )
    .expect("a plane");

    let mut builder = SolidBuilder::new();
    let corner = builder.add_vertex(south).expect("a pole is a point");
    let rim = builder
        .add_edge(corner, corner, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("a closed edge");
    let skin_step = DraftStep::Along(
        builder
            .add_coedge(rim, Orientation::Reversed)
            .expect("an edge of this builder"),
    );
    let lid_step = DraftStep::Along(
        builder
            .add_coedge(rim, Orientation::Forward)
            .expect("an edge of this builder"),
    );
    let skin_wire = builder.add_wire(&[skin_step]).expect("a closed wire");
    let lid_wire = builder.add_wire(&[lid_step]).expect("a closed wire");
    let skin = builder
        .add_face(
            Surface::Sphere(Sphere::new(Point::origin(), radius).expect("a ball")),
            &[skin_wire],
        )
        .expect("a face with one wire");
    let lid = builder
        .add_face(Surface::Plane(lid), &[lid_wire])
        .expect("a face with one wire");
    builder
        .add_shell(&[(skin, Orientation::Forward), (lid, Orientation::Forward)])
        .expect("two faces close up");
    let solid = builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cap of a ball is a solid");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cap of a ball is a sound model");
    solid
}

/// The solid meshes into a closed surface with the shape of a ball, and its
/// volume is under `closed_form` by no more than `3δ/r` of it.
///
/// Closed means every side of every triangle is used once each way round; the
/// shape of a ball means points less sides plus triangles is two. The bound is
/// `a_half_ball_skin_with_a_round_hole.rs`'s: the missing shell is at most the
/// curved area times `δ`, and a cap holding the ball's centre has volume at
/// least that area times `r/3`.
fn meshes_closed_and_measures(
    model: &Model,
    solid: SolidKey,
    deflection: f64,
    radius: f64,
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
        .unwrap_or_else(|why| panic!("{what}: the cap meshes: {why:?}"));
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

    let found = view
        .measure(
            Deflection::of(deflection).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the cap measures: {why:?}"))
        .volume();
    let under = (closed_form - found) / closed_form;
    assert!(
        (0.0..3.0 * deflection / radius).contains(&under),
        "{what}: measured {found}, closed form {closed_form}, {:.4}% under",
        100.0 * under
    );
}

/// The cap of height `h` off a ball of `radius`: `π h² (3r − h) / 3`.
fn a_caps_volume(radius: f64, h: f64) -> f64 {
    PI * h * h * (3.0 * radius - h) / 3.0
}
