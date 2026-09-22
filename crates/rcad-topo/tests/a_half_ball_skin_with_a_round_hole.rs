//! A half ball's skin with a round hole in it, cut up so that it meshes.
//!
//! Cut a ball by a plane through its polar axis and the skin left is bounded by
//! one great circle, walked as two edges that meet at the poles. Cut that half
//! ball again by a plane that misses the poles and keeps both of them, and the
//! skin gets a hole: the whole small circle the second plane cuts. In the
//! ball's two coordinates the outer loop is a rectangle half a turn wide whose
//! top and bottom are the two poles, and the hole is an island inside it.
//!
//! No patch may go more than a quarter turn round a ball, and a face with a
//! hole that is not a ring of two whole circles used to be refused for it
//! (`CurvedFaceWithAHoleGoesTooFarRound`). It is cut now, from the hole's
//! highest corner up to the top pole and from its lowest down to the bottom
//! one, into two pieces with one loop each. What is checked here is that the
//! pieces meet: the mesh closes up, and the solid measures its closed form.

use core::f64::consts::PI;
use std::collections::HashMap;

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

const RADIUS: f64 = 1.0;

/// Every solid here measures within `3δ/r` under its closed form, for the
/// reason `a_half_ball_cut_again_away_from_its_poles.rs` gives: the missing
/// shell is at most the curved area times `δ`, and every solid here has volume
/// at least that area times `r/3`.
fn how_far_under(deflection: f64) -> f64 {
    3.0 * deflection / RADIUS
}

/// The half ball `x >= 0` cut by `x <= at`, for cuts near the flat face, near
/// the far pole of the ball's `x` axis and between, each at three
/// coarsenesses. The hole's corner, where the fuse starts the circle, sits at
/// its top every time.
#[test]
fn a_half_ball_cut_square_to_its_flat_face_meshes_closed_wherever_the_cut_is() {
    for at in [0.1, 0.3, 0.5, 0.7, 0.9] {
        for deflection in [0.1, 0.01, 0.001] {
            let mut model = a_model();
            let half = half_a_ball(&mut model);
            let knife = a_block_up_to(&mut model, at, &Isometry::identity());
            let answer = cut(&mut model, &half, knife);
            meshes_closed_and_measures(
                &model,
                answer,
                deflection,
                half_a_balls_volume() - a_caps_volume(RADIUS - at),
                &format!("x <= {at} at a deflection of {deflection}"),
            );
        }
    }
}

/// The same cut at `x = 0.5`, leaning about the `y` axis by `lean` and then
/// turned about the `z` axis by `turn`, both in radians, so that the hole is no
/// longer square to the ball's equator nor centred on the meridian half way
/// between the poles' two edges. The circle stays clear of the flat face while
/// the cut's normal is less than 30 degrees off the `x` axis: its lowest `x` is
/// `0.5·n_x − (√0.75)·√(1 − n_x²)`, positive while `n_x > √0.75`, and
/// `n_x = cos lean · cos turn`, so a turn about the polar axis counts as much
/// as the lean does. Every placement here keeps it clear.
#[test]
fn a_half_ball_cut_by_a_leaning_plane_that_misses_its_flat_face_meshes_closed() {
    let twenty_degrees = 20.0_f64.to_radians();
    for [lean, turn] in [
        [0.0, 0.3],
        [0.0, -0.4],
        [-0.2, 0.0],
        [twenty_degrees, 0.0],
        [twenty_degrees, 0.3],
        [0.4, 0.0],
        [0.35, 0.3],
    ] {
        let mut model = a_model();
        let half = half_a_ball(&mut model);
        let knife = a_block_up_to(&mut model, 0.5, &leaning(lean, turn));
        let answer = cut(&mut model, &half, knife);
        meshes_closed_and_measures(
            &model,
            answer,
            0.001,
            half_a_balls_volume() - a_caps_volume(RADIUS - 0.5),
            &format!("leaning {lean}, turned {turn}"),
        );
    }
}

/// Twenty degrees of lean turned back by `0.4` puts the cut's normal just past
/// 30 degrees off the `x` axis, so the circle dips `0.001` past the flat face
/// and the skin is left with no hole at all: one piece bounded by the flat
/// face's half circle and the long way round the cut. Every corner of that
/// piece is on the flat face, the two poles and the two places the cut crosses
/// it, and it used to be refused `Undecidable` because no point the fuse tried
/// could be proved inside it.
#[test]
fn a_half_ball_cut_by_a_plane_leaning_twenty_degrees_and_turned_back_just_past_its_flat_face() {
    let (lean, turn) = (20.0_f64.to_radians(), -0.4);
    let mut model = a_model();
    let half = half_a_ball(&mut model);
    let knife = a_block_up_to(&mut model, 0.5, &leaning(lean, turn));
    let answer = cut(&mut model, &half, knife);
    meshes_closed_and_measures(
        &model,
        answer,
        0.001,
        what_a_leaning_cut_leaves(lean, turn),
        "leaning twenty degrees, turned back",
    );
}

/// Twenty degrees of lean with every turn from `-0.5` to `0.5`. From `±0.4`
/// out the circle crosses the flat face and the skin has no hole; inside that
/// the hole is cut up to both poles. Both kinds answer and measure.
#[test]
fn a_half_ball_cut_by_a_plane_leaning_twenty_degrees_answers_at_every_turn() {
    let lean = 20.0_f64.to_radians();
    for twentieths in -10..=10 {
        let turn = f64::from(twentieths) * 0.05;
        let mut model = a_model();
        let half = half_a_ball(&mut model);
        let knife = a_block_up_to(&mut model, 0.5, &leaning(lean, turn));
        let answer = cut(&mut model, &half, knife);
        meshes_closed_and_measures(
            &model,
            answer,
            0.001,
            what_a_leaning_cut_leaves(lean, turn),
            &format!("leaning twenty degrees, turned {turn}"),
        );
    }
}

/// The volume of the half ball `x >= 0` on the near side of the plane half a
/// radius out along `(cos lean · cos turn, …)`: the half ball less the cap the
/// plane cuts off, less only the part of that cap the half ball has.
///
/// That part is the whole cap while the circle is clear of the flat face. Past
/// it, the cap's slice at height `t` along the normal is a disk of radius
/// `ρ = √(1 − t²)` whose centre is `h = t·n_x/√(1 − n_x²)` from the flat face,
/// measured in the slice, and the part of the disk past the face is the
/// segment `ρ² acos(h/ρ) − h √(ρ² − h²)`. That is summed over `t` by
/// Simpson's rule, whose error is far below what meshing loses.
fn what_a_leaning_cut_leaves(lean: f64, turn: f64) -> f64 {
    let n_x = lean.cos() * turn.cos();
    let past_the_face_up_to = (1.0 - n_x * n_x).sqrt();
    let segment = |t: f64| {
        let rho = (1.0 - t * t).sqrt();
        let h = t * n_x / (1.0 - n_x * n_x).sqrt();
        if h >= rho {
            return 0.0;
        }
        rho * rho * (h / rho).acos() - h * (rho * rho - h * h).sqrt()
    };
    let from = 0.5 * RADIUS;
    let mut the_part_past_the_face = 0.0;
    if past_the_face_up_to > from {
        let steps = 2000;
        let width = (past_the_face_up_to - from) / f64::from(steps);
        for step in 0..=steps {
            let weight = if step == 0 || step == steps {
                1.0
            } else if step % 2 == 1 {
                4.0
            } else {
                2.0
            };
            the_part_past_the_face += weight * segment(from + width * f64::from(step));
        }
        the_part_past_the_face *= width / 3.0;
    }
    half_a_balls_volume() - (a_caps_volume(RADIUS - 0.5) - the_part_past_the_face)
}

/// Lean by `lean` about the `y` axis, then turn by `turn` about the `z` axis.
fn leaning(lean: f64, turn: f64) -> Isometry {
    Isometry::rotation(&Axis::z(), turn)
        .expect("a finite turn")
        .compose(&Isometry::rotation(&Axis::y(), lean).expect("a finite lean"))
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(25.0 * RADIUS).expect("a positive size"))
}

/// The ball of radius [`RADIUS`] intersected with the block `0 <= x <= 6r`,
/// `|y|, |z| <= 6r`.
fn half_a_ball(model: &mut Model) -> Vec<SolidKey> {
    let ball = model
        .add_ball(RADIUS, &Isometry::identity())
        .expect("a ball");
    let knife = block(
        model,
        [6.0 * RADIUS, 12.0 * RADIUS, 12.0 * RADIUS],
        &Isometry::translation(Vector::new(0.0, -6.0 * RADIUS, -6.0 * RADIUS))
            .expect("a finite placement"),
    );
    let answer = model
        .fuse(
            &[ball],
            &[knife],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a ball cut through its polar axis is answered")
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a fuse writes a sound model or writes nothing");
    answer
}

/// The block `-7r <= x <= at`, `|y|, |z| <= 7r`, then moved by `placed`.
fn a_block_up_to(model: &mut Model, at: f64, placed: &Isometry) -> SolidKey {
    let to_its_place =
        Isometry::translation(Vector::new(-7.0 * RADIUS, -7.0 * RADIUS, -7.0 * RADIUS))
            .expect("a finite placement");
    block(
        model,
        [7.0 * RADIUS + at, 14.0 * RADIUS, 14.0 * RADIUS],
        &placed.compose(&to_its_place),
    )
}

fn block(model: &mut Model, [along_x, along_y, along_z]: [f64; 3], placed: &Isometry) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [along_x, 0.0],
        [along_x, along_y],
        [0.0, along_y],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(&rectangle, along_z, placed)
        .expect("building a box needs no decision")
}

/// Intersect the half ball with `knife`, insist on one solid and a sound model.
fn cut(model: &mut Model, half: &[SolidKey], knife: SolidKey) -> SolidKey {
    let answer = model
        .fuse(
            half,
            &[knife],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a half ball cut again away from its poles is answered")
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a fuse writes a sound model or writes nothing");
    assert_eq!(answer.len(), 1);
    answer[0]
}

/// The solid meshes into a closed surface with the shape of a ball, and its
/// volume is under `closed_form` by no more than the deflection allows.
///
/// Closed means every side of every triangle is used once each way round; the
/// shape of a ball means points less sides plus triangles is two. A piece of
/// the skin meshed inside out, or the two pieces either side of a cut not
/// meeting along it, fails the first.
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
        .unwrap_or_else(|why| panic!("{what}: the answer meshes: {why:?}"));
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
        .unwrap_or_else(|why| panic!("{what}: the answer measures: {why:?}"))
        .volume();
    let under = (closed_form - found) / closed_form;
    assert!(
        (0.0..how_far_under(deflection)).contains(&under),
        "{what}: measured {found}, closed form {closed_form}, {:.4}% under",
        100.0 * under
    );
}

fn half_a_balls_volume() -> f64 {
    2.0 / 3.0 * PI * RADIUS.powi(3)
}

/// The cap of height `h` off a ball: `π h² (3r − h) / 3`.
fn a_caps_volume(h: f64) -> f64 {
    PI * h * h * (3.0 * RADIUS - h) / 3.0
}
