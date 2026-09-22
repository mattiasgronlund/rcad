//! A ball less a disc: the piece of a ball cut by one plane that holds both of
//! its poles.
//!
//! A plane that misses the ball's polar axis on the far side of its centre cuts
//! a circle that goes round neither pole, and the piece on the centre's side of
//! it keeps both. Its skin is bounded by that one circle alone, and in the
//! ball's two coordinates the circle is a loop that climbs and comes back, with
//! the material **outside** it. The triangulator fills the inside of what it is
//! handed, so the mesher refused this skin by name,
//! `ACurvedFaceGoesRightRoundWithNoSeam`, rather than fill the disc that was
//! cut away (`docs/decisions.md` §197.3, carried open to §199.4).
//!
//! It is a gap and not a limit: the skin is a whole ball's rectangle — a seam
//! from pole to pole and the two poles — with the circle as an island in it,
//! which is the shape a half ball cut again already meshes as. The mesher now
//! puts that seam in, half a turn from the circle, and cuts from the circle up
//! to each pole.
//!
//! On the base this lane started from (`4514eff4`), every one of the 96 pieces
//! below built face by face was refused by `SolidBuilder::build`, which meshes
//! each face coarsely to check it, with `ACurvedFaceGoesRightRoundWithNoSeam`;
//! and so were the four fuses at the bottom, which reach the same skin through
//! `Model::fuse`.
//!
//! What is checked is that each piece closes up with the shape of a ball and
//! measures its closed form within `3δ/r` under it. The bound is the one
//! `a_half_ball_skin_with_a_round_hole.rs` publishes: the missing shell is at
//! most the curved area times `δ`, and every piece here holds the centre of
//! each ball it keeps skin of, so its volume is at least that area times `r/3`.

use core::f64::consts::{FRAC_PI_2, PI};
use std::collections::HashMap;

use rcad_geom::{
    Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Sphere, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftStep, Error, Invalid, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey,
};

/// A ball less the cap beyond a plane through a point on its equator, for
/// three sizes, the point at each of the four quarter turns — the chart's own
/// `±π` among them — and eight planes. `turn` is how far the plane's normal is
/// turned about the polar axis away from the point, and `lean` how far it
/// leans up off the equator; the cap is small for a small turn and lean and
/// takes most of the ball for a turn near a quarter. Every plane keeps both
/// poles: `cos lean · cos turn > |sin lean|`.
#[test]
fn a_ball_less_a_cap_built_face_by_face_meshes_closed_and_measures() {
    for radius in [0.5, 1.0, 2.0] {
        for quarter in 0..4 {
            for [turn, lean] in [
                [0.05_f64, 0.0],
                [0.5, 0.0],
                [1.0, 0.0],
                [1.5, 0.0],
                [0.3, 0.3],
                [1.0, 0.4],
                [0.8, -0.5],
                [0.7, 0.6],
            ] {
                assert!(lean.cos() * turn.cos() > lean.sin().abs());
                let mut model = a_model(radius);
                let what = format!("radius {radius}, quarter {quarter}, turn {turn}, lean {lean}");
                let piece = a_ball_less_a_cap(&mut model, radius, quarter, turn, lean, &what);
                let height = radius - radius * lean.cos() * turn.cos();
                meshes_closed_and_measures(
                    &model,
                    &[piece],
                    0.002 * radius,
                    radius,
                    a_balls_volume(radius) - a_caps_volume(radius, height),
                    &what,
                );
            }
        }
    }
}

/// The two fuses `degenerate_curved_fuses.rs` pinned as refused for want of a
/// seam, answered: the union of two balls of radius two that cross, and a ball
/// of radius four with the cap beyond `x = 2` taken off it or a block put
/// over that cap.
#[test]
fn what_is_round_two_crossing_balls_and_round_a_cap_cut_off_the_axis_is_answered() {
    for (apart, lens) in [(2.0, 10.0 * PI / 3.0), (3.0, 11.0 * PI / 12.0)] {
        let mut model = a_model(2.0);
        let one = model
            .add_ball(2.0, &Isometry::identity())
            .expect("a ball of radius two");
        let other = model
            .add_ball(2.0, &shifted(Vector::new(apart, 0.0, 0.0)))
            .expect("a ball of radius two");
        let union = fused(&mut model, one, other, Rule::Union);
        meshes_closed_and_measures(
            &model,
            &union,
            0.004,
            2.0,
            2.0 * a_balls_volume(2.0) - lens,
            &format!("two balls {apart} apart, united"),
        );
    }

    let cap = a_caps_volume(4.0, 2.0);
    let block_volume = 12.0_f64.powi(3);
    for (rule, closed_form) in [
        (Rule::Difference, a_balls_volume(4.0) - cap),
        (Rule::Union, a_balls_volume(4.0) - cap + block_volume),
    ] {
        let mut model = a_model(4.0);
        let ball = model
            .add_ball(4.0, &Isometry::identity())
            .expect("a ball of radius four");
        let knife = block(&mut model, 12.0, Vector::new(2.0, -6.0, -6.0));
        let answer = fused(&mut model, ball, knife, rule);
        meshes_closed_and_measures(
            &model,
            &answer,
            0.008,
            4.0,
            closed_form,
            &format!("a ball and the block beyond x = 2, {rule:?}"),
        );
    }
}

fn a_model(radius: f64) -> Model {
    Model::new(Scale::millimetres(25.0 * radius).expect("a positive size"))
}

fn shifted(by: Vector) -> Isometry {
    Isometry::translation(by).expect("a finite placement")
}

/// A cube of side `side` with its low corner at `at`.
fn block(model: &mut Model, side: f64, at: Vector) -> SolidKey {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    model
        .add_extrusion(&square, side, &shifted(at))
        .expect("building a box needs no decision")
}

/// Fuse two solids, insist on one solid and a sound model, and hand it back.
fn fused(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> Vec<SolidKey> {
    let answer = model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("the {rule:?} is answered: {why:?}"))
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a fuse writes a sound model or writes nothing");
    assert_eq!(answer.len(), 1, "the {rule:?} is one solid");
    answer
}

/// The part of the ball of `radius` about the origin on the centre's side of
/// the plane through the equator's point at `quarter` quarter turns from `x`,
/// whose normal points out of the material `turn` round from that point and
/// `lean` up off the equator.
///
/// Built face by face, the skin bounded by the closed circle where the ball
/// meets the plane and the flat lid across it. The circle's one corner is the
/// equator's point, which is on the ball exactly. **Which way the circle is
/// stored running is the model's direction rule's to say**, not the caller's,
/// so the skin is tried both ways round and exactly one of the two is let
/// through; the other is refused for its two faces disagreeing.
fn a_ball_less_a_cap(
    model: &mut Model,
    radius: f64,
    quarter: usize,
    turn: f64,
    lean: f64,
    what: &str,
) -> SolidKey {
    let on_the_equator = [
        Point::new(radius, 0.0, 0.0),
        Point::new(0.0, radius, 0.0),
        Point::new(-radius, 0.0, 0.0),
        Point::new(0.0, -radius, 0.0),
    ][quarter];
    let towards = quarter as f64 * FRAC_PI_2 + turn;
    let out = Vector::new(
        lean.cos() * towards.cos(),
        lean.cos() * towards.sin(),
        lean.sin(),
    );
    let lid = Plane::from_point_and_normal(on_the_equator, out).expect("a plane");
    let ball = Sphere::new(Point::origin(), radius).expect("a ball");

    let mut built = Vec::new();
    for skin_runs in [Orientation::Forward, Orientation::Reversed] {
        let mut trying = model.clone();
        match skin_bounded_by_one_circle(&mut trying, on_the_equator, ball, lid, skin_runs) {
            Ok(solid) => built.push((trying, solid)),
            Err(Error::Invalid(Invalid::FacesDoNotAgreeWhichWayTheyFace { .. })) => (),
            Err(why) => panic!("{what}: built {skin_runs:?}, refused {why:?}"),
        }
    }
    assert_eq!(built.len(), 1, "{what}: exactly one way round builds");
    let (answer, solid) = built.pop().expect("just counted");
    *model = answer;
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a ball less a cap is a sound model");
    solid
}

fn skin_bounded_by_one_circle(
    model: &mut Model,
    corner: Point,
    ball: Sphere,
    lid: Plane,
    skin_runs: Orientation,
) -> Result<SolidKey, Error> {
    let lid_runs = if skin_runs.is_reversed() {
        Orientation::Forward
    } else {
        Orientation::Reversed
    };
    let mut builder = SolidBuilder::new();
    let corner = builder.add_vertex(corner)?;
    let rim = builder.add_edge(corner, corner, EdgeRun::RightRoundWhereItsFacesCross)?;
    let skin_step = DraftStep::Along(builder.add_coedge(rim, skin_runs)?);
    let lid_step = DraftStep::Along(builder.add_coedge(rim, lid_runs)?);
    let skin_wire = builder.add_wire(&[skin_step])?;
    let lid_wire = builder.add_wire(&[lid_step])?;
    let skin = builder.add_face(Surface::Sphere(ball), &[skin_wire])?;
    let lid = builder.add_face(Surface::Plane(lid), &[lid_wire])?;
    builder.add_shell(&[(skin, Orientation::Forward), (lid, Orientation::Forward)])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// The solids mesh into closed surfaces with the shape of a ball, and their
/// volume together is under `closed_form` by no more than `3δ/r` of it.
///
/// Closed means every side of every triangle is used once each way round; the
/// shape of a ball means points less sides plus triangles is two. A piece of
/// the skin filled inside out, or two patches either side of the seam or of a
/// cut not meeting along it, fails the first.
fn meshes_closed_and_measures(
    model: &Model,
    solids: &[SolidKey],
    deflection: f64,
    radius: f64,
    closed_form: f64,
    what: &str,
) {
    let [solid] = solids else {
        panic!("{what}: one solid, got {}", solids.len());
    };
    let view = model.solid(*solid).expect("this model made the key");
    let mesh = view
        .mesh(
            Deflection::of(deflection).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the piece meshes: {why:?}"));
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
        .unwrap_or_else(|why| panic!("{what}: the piece measures: {why:?}"))
        .volume();
    let under = (closed_form - found) / closed_form;
    assert!(
        (0.0..3.0 * deflection / radius).contains(&under),
        "{what}: measured {found}, closed form {closed_form}, {:.4}% under",
        100.0 * under
    );
}

fn a_balls_volume(radius: f64) -> f64 {
    4.0 * PI * radius.powi(3) / 3.0
}

/// The cap of height `h` off a ball of `radius`: `π h² (3r − h) / 3`.
fn a_caps_volume(radius: f64, h: f64) -> f64 {
    PI * h * h * (3.0 * radius - h) / 3.0
}
