//! A ball of radius 5 less the small cap `y + 3z ≥ 15`, whose rim runs through
//! the north pole.
//!
//! The piece kept holds the south pole and the ball's centre. Its skin is
//! bounded by one closed circle through the north pole, so the mesher cuts it
//! into four patches up to the pole it holds — the family of
//! `a_cap_whose_rim_runs_through_the_other_pole.rs`, which has the mirror
//! image, a rim through the south pole, at radii 0.5, 1 and 2.
//!
//! Before this file `SolidBuilder::build`, which meshes each face coarsely to
//! check it, refused this solid `ADraftFaceCouldNotBeMeshed { CornersCoincide }`
//! for every way of writing the rim (`docs/decisions.md` §199.4 and §220.1
//! carried it as open). The plane is written through `(0, 3, 4)`, and at radius
//! 5 it holds the north pole only to a rounding, on the material's side. The
//! mesher read that pole as the one the cap reaches, though the rim runs
//! through it, and cut the skin up to it. At radius 4 or 8, or mirrored through
//! the south pole, the same shape was built.
//!
//! The solid is built face by face, as in the file it mirrors. Two faces: the
//! skin, bounded by the circle where the ball meets the plane, and the flat lid
//! across it. The circle's one corner is `(0, 3, 4)`, on the ball exactly.
//!
//! The oracle is the whole ball less the closed form of the small cap,
//! 522.582330, and the mesh must close up with the shape of a ball.

use core::f64::consts::PI;
use std::collections::HashMap;

use rcad_geom::{Deflection, EdgeRun, Orientation, Plane, Point, Scale, Sphere, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, SolidBuilder, SolidKey};

const RADIUS: f64 = 5.0;

/// The rim written each of the three ways a closed rim between a ball and a
/// plane can be: by its two faces, by the plane it crosses, and by that plane
/// turned over.
#[test]
fn a_ball_of_radius_five_less_a_cap_through_its_north_pole_meshes_closed_and_measures() {
    let lid = Plane::from_point_and_normal(Point::new(0.0, 3.0, 4.0), Vector::new(0.0, 1.0, 3.0))
        .expect("a plane");
    let closed_form = a_ball_less_a_small_cap(RADIUS, 15.0 / 10.0_f64.sqrt());
    assert_eq!(format!("{closed_form:.6}"), "522.582330");
    for (name, run) in [
        ("the faces cross", EdgeRun::RightRoundWhereItsFacesCross),
        (
            "it crosses the lid",
            EdgeRun::RightRoundWhereItCrosses {
                and: Surface::Plane(lid),
            },
        ),
        (
            "it crosses the lid turned over",
            EdgeRun::RightRoundWhereItCrosses {
                and: Surface::Plane(lid.reversed()),
            },
        ),
    ] {
        let mut model = Model::new(Scale::millimetres(125.0).expect("a positive size"));
        let solid = a_ball_less_a_cap(&mut model, lid, run);
        for deflection in [0.5, 0.05, 0.005] {
            meshes_closed_and_measures(
                &model,
                solid,
                deflection,
                closed_form,
                &format!("the rim where {name}, deflection {deflection}"),
            );
        }
    }
}

/// The part of the ball of radius 5 about the origin on the negative side of
/// `lid`, which faces out of the material.
fn a_ball_less_a_cap(model: &mut Model, lid: Plane, run: EdgeRun) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let corner = builder
        .add_vertex(Point::new(0.0, 3.0, 4.0))
        .expect("a point");
    let rim = builder
        .add_edge(corner, corner, run)
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
            Surface::Sphere(Sphere::new(Point::origin(), RADIUS).expect("a ball")),
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
        .expect("a ball less a cap is a solid");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a ball less a cap is a sound model");
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

/// The ball of `radius` less the cap cut off by a plane `distance` from its
/// centre: the cap of height `h = r − distance` is `π h² (3r − h) / 3`.
fn a_ball_less_a_small_cap(radius: f64, distance: f64) -> f64 {
    let h = radius - distance;
    4.0 * PI * radius.powi(3) / 3.0 - PI * h * h * (3.0 * radius - h) / 3.0
}
