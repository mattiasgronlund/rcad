//! The small cap `y + 3z ≥ 15` of a ball of radius 5 about the origin: the
//! piece `a_ball_less_a_cap_whose_rim_runs_through_its_north_pole.rs` cuts
//! away, pinned on its own.
//!
//! Its rim runs through the north pole, and the cap is only
//! `h = 5 − 15/√10 ≈ 0.257` high, so its volume, `π h² (3r − h) / 3 ≈ 1.016`,
//! is about a five-hundredth of the ball's.
//!
//! The solid is built face by face, as in the file it is cut from. Two faces:
//! the skin, bounded by the circle where the ball meets the plane, and the
//! flat lid across it. The circle's one corner is `(0, 3, 4)`, on the ball
//! exactly. The rim is the same edge the ball less this cap is built with, and
//! the skin here is the rest of the ball, so the skin and the lid each use it
//! the other way round from there.
//!
//! The oracle is the closed form of the cap, and the mesh must close up with
//! the shape of a ball. The measure is held to the error it publishes,
//! [`Measurement::volume_error`](rcad_topo::Measurement::volume_error), and
//! not to the `3δ/r` share the files about pieces of a ball use. That share is
//! those tests' own argument, published nowhere in `measure.rs` or `mesh.rs`,
//! and it rests on the piece holding the ball's centre, which a cap this small
//! does not. At a deflection of 0.01 the cap measures 0.88% under, where `3δ/r`
//! is 0.6%.

use core::f64::consts::PI;
use std::collections::HashMap;

use rcad_geom::{Deflection, EdgeRun, Orientation, Plane, Point, Scale, Sphere, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Invalid, Model, SolidBuilder, SolidKey};

const RADIUS: f64 = 5.0;

/// The rim written each of the three ways a closed rim between a ball and a
/// plane can be: by its two faces, by the plane it crosses, and by that plane
/// turned over.
#[test]
fn the_small_cap_through_a_balls_north_pole_meshes_closed_and_measures() {
    let closed_form = a_small_cap(RADIUS, 15.0 / 10.0_f64.sqrt());
    assert_eq!(format!("{closed_form:.6}"), "1.016446");
    for (name, run) in every_way_to_write_the_rim() {
        let mut model = Model::new(Scale::millimetres(125.0).expect("a positive size"));
        let solid = a_small_cap_of_a_ball(&mut model, run, Orientation::Forward)
            .unwrap_or_else(|why| panic!("the rim where {name}: a small cap is a solid: {why:?}"));
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a small cap of a ball is a sound model");
        for deflection in [0.5, 0.05, 0.01, 0.005] {
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

/// The skin and the lid using the rim the way the ball less this cap does,
/// which leaves the skin facing the wrong way for the piece it bounds.
#[test]
fn the_small_cap_with_its_faces_round_the_rim_the_wrong_way_is_refused() {
    for (name, run) in every_way_to_write_the_rim() {
        let mut model = Model::new(Scale::millimetres(125.0).expect("a positive size"));
        let refused = a_small_cap_of_a_ball(&mut model, run, Orientation::Reversed);
        assert!(
            matches!(
                refused,
                Err(Error::Invalid(Invalid::FacesDoNotAgreeWhichWayTheyFace))
            ),
            "the rim where {name}: {refused:?}"
        );
    }
}

fn every_way_to_write_the_rim() -> [(&'static str, EdgeRun); 3] {
    [
        ("the faces cross", EdgeRun::RightRoundWhereItsFacesCross),
        (
            "it crosses the lid",
            EdgeRun::RightRoundWhereItCrosses {
                and: Surface::Plane(the_lid()),
            },
        ),
        (
            "it crosses the lid turned over",
            EdgeRun::RightRoundWhereItCrosses {
                and: Surface::Plane(the_lid().reversed()),
            },
        ),
    ]
}

/// The part of the ball of radius 5 about the origin on the negative side of
/// [`the_lid`], with the skin using the rim `skin_way` and the lid the other
/// way.
fn a_small_cap_of_a_ball(
    model: &mut Model,
    run: EdgeRun,
    skin_way: Orientation,
) -> rcad_topo::Result<SolidKey> {
    let mut builder = SolidBuilder::new();
    let corner = builder
        .add_vertex(Point::new(0.0, 3.0, 4.0))
        .expect("a point");
    let rim = builder
        .add_edge(corner, corner, run)
        .expect("a closed edge");
    let skin_step = DraftStep::Along(
        builder
            .add_coedge(rim, skin_way)
            .expect("an edge of this builder"),
    );
    let lid_step = DraftStep::Along(
        builder
            .add_coedge(rim, skin_way.reversed())
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
        .add_face(Surface::Plane(the_lid()), &[lid_wire])
        .expect("a face with one wire");
    builder
        .add_shell(&[(skin, Orientation::Forward), (lid, Orientation::Forward)])
        .expect("two faces close up");
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// The plane `y + 3z = 15` through the rim's corner, facing out of the cap.
fn the_lid() -> Plane {
    Plane::from_point_and_normal(Point::new(0.0, 3.0, 4.0), Vector::new(0.0, -1.0, -3.0))
        .expect("a plane")
}

/// The solid meshes into a closed surface with the shape of a ball, and its
/// volume is under `closed_form` by no more than the error its measure
/// publishes.
///
/// Closed means every side of every triangle is used once each way round; the
/// shape of a ball means points less sides plus triangles is two. Under,
/// because the cap is convex and every corner of its mesh is on its surface, so
/// the mesh encloses no space the cap does not.
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
    println!(
        "{what}: measured {found:.6}, published error {:.6}, closed form {closed_form:.6}, {:.4}% under",
        measured.volume_error(),
        100.0 * (closed_form - found) / closed_form
    );
    assert!(
        found < closed_form,
        "{what}: measured {found}, over the closed form {closed_form}"
    );
    assert!(
        closed_form - found <= measured.volume_error(),
        "{what}: measured {found}, closed form {closed_form}, published error {}",
        measured.volume_error()
    );
}

/// The cap cut off a ball of `radius` by a plane `distance` from its centre:
/// the cap of height `h = r − distance` is `π h² (3r − h) / 3`.
fn a_small_cap(radius: f64, distance: f64) -> f64 {
    let h = radius - distance;
    PI * h * h * (3.0 * radius - h) / 3.0
}
