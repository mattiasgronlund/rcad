//! A ball of radius 2 at the origin fused with the box `[0.5, 3]³`, whose
//! corner pokes into it (queue row Q1194, `docs/decisions.md` §35.6).
//!
//! **The ball's skin inside the box is the part of the sphere in front of three
//! planes, `x, y, z ≥ 0.5`, each with the centre behind it**: a three-sided
//! patch round `(1, 1, 1)` with its corners at `(0.5, 0.5, √3.5)` and the two
//! like it. `holds.rs`'s `is_a_family_this_follows` took a ball's skin cut by
//! two planes, or by any number through the centre, and refused this one
//! `AFaceTrimmedByACurveThisCannotFollow`, so every point whose ray met the
//! patch was refused `CannotFollowAFacesTrimming`, under the intersection and
//! the box less the ball alike. Each cut keeps a cap no bigger than half the
//! ball, such caps meet in one piece, and the patch is that piece, so it is
//! followed now: the proof is in that function's doc comment.
//!
//! **The union, and the ball less the box, are followed too** (row Q1342).
//! Both keep the ball's skin outside the box, and the fuse leaves part of it
//! as a face of its own: the part with `x, y ≥ 0.5` and `z ≤ 0.5`, cut by the
//! same three planes with the centre *in front of* `z ≤ 0.5`. Three planes
//! with the centre in front of one can leave two pieces
//! (`one_plane_with_a_balls_centre_in_front_beside_two_with_it_behind_leaves_two_pieces`
//! in `holds.rs`), and these do not because they meet at the box's corner,
//! `(0.5, 0.5, 0.5)`, inside the ball: the same proof, pushing the sphere out
//! from that corner rather than from the centre. Every point whose ray met that
//! face was refused `CannotFollowAFacesTrimming` before.

use std::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Model, ProfileBuilder, Rule, SolidKey};

/// The closed form is right where the answer is known: at `a = b = c = 0` it
/// is an eighth of the ball, and at `b = c = 0` a quarter of the cap past
/// `x = a`, whose height `h = r − a` makes it `πh²(3r − h)/3`.
#[test]
fn the_corner_of_a_ball_is_an_eighth_of_it_at_the_centre_and_a_quarter_cap_off_one_plane() {
    let eighth = corner_of_a_ball(2.0, [0.0, 0.0, 0.0]);
    assert!(
        (eighth - 4.0 * PI / 3.0).abs() <= 1e-12,
        "an eighth of a ball of radius 2 is 4π/3, and the closed form says {eighth}"
    );
    let h = 1.5;
    let quarter_cap = PI * h * h * (3.0 * 2.0 - h) / 12.0;
    let worked = corner_of_a_ball(2.0, [0.5, 0.0, 0.0]);
    assert!(
        (worked - quarter_cap).abs() <= 1e-12,
        "a quarter of the cap past x = 0.5 is {quarter_cap}, and the closed form says {worked}"
    );
}

/// Every rule, both ways round, answers one solid holding the volume worked
/// out by hand, and the solid places points as the hand does.
///
/// The corner piece is `corner_of_a_ball(2, [0.5, 0.5, 0.5])`, `0.91785…`;
/// the ball is `32π/3` and the box `2.5³`, so the union is their sum less the
/// piece, and each difference the one less the piece.
#[test]
fn a_ball_and_the_corner_of_a_box_answer_every_rule_both_ways() {
    let corner = [0.5, 0.5, 0.5];
    let piece = corner_of_a_ball(2.0, corner);
    let (ball, block) = (32.0 * PI / 3.0, 2.5f64.powi(3));
    let points = [
        [1.0, 1.0, 1.0],
        [0.6, 0.6, 0.6],
        [1.2, 1.2, 1.2],
        [-1.0, 0.0, 0.0],
        [2.5, 2.5, 2.5],
        [0.4, 1.0, 1.0],
    ];
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for ball_first in [true, false] {
            let what = format!("{rule:?}, the ball first {ball_first}");
            let (model, solid) = fused(rule, ball_first, corner, FAR_CORNER, &what);
            let volume = match (rule, ball_first) {
                (Rule::Union, _) => ball + block - piece,
                (Rule::Intersection, _) => piece,
                (Rule::Difference, true) => ball - piece,
                (Rule::Difference, false) => block - piece,
                (other, _) => panic!("not a rule this probes: {other:?}"),
            };
            assert_holds(&model, solid, volume, &what);
            for at in points {
                let placed = model
                    .solid(solid)
                    .expect("this model made the key")
                    .classify_point(
                        Point::from(at),
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    );
                assert_eq!(
                    placed,
                    by_hand(rule, ball_first, corner, FAR_CORNER, at),
                    "{what} placed {at:?}"
                );
            }
        }
    }
}

/// The union both ways round and the ball less the box keep the ball's skin
/// outside the box as **one face**, the sphere less the three-sided patch
/// inside the box, as a ball with that patch cut off is written.
///
/// The fuse cuts the ball's one face along the whole circles `x, y, z = 0.5`,
/// eight pieces, and seven of them are outside the box. Until the joiner had a
/// ball's arm they were seven faces, with nine edges between them that had the
/// sphere on both sides, and `Q1024` and `Q1342` were both refusals met on such
/// a split face (`docs/decisions.md` §229.2).
#[test]
fn a_balls_skin_outside_the_box_is_one_face() {
    let corner = [0.5, 0.5, 0.5];
    let piece = corner_of_a_ball(2.0, corner);
    let (ball, block) = (32.0 * PI / 3.0, 2.5f64.powi(3));
    for (rule, ball_first, volume, faces) in [
        (Rule::Union, true, ball + block - piece, 7),
        (Rule::Union, false, ball + block - piece, 7),
        (Rule::Difference, true, ball - piece, 4),
    ] {
        let what = format!("{rule:?}, the ball first {ball_first}");
        let (model, solid) = fused(rule, ball_first, corner, FAR_CORNER, &what);
        assert_holds(&model, solid, volume, &what);
        let answer = model.solid(solid).expect("this model made the key");
        let on_the_ball = answer
            .faces()
            .filter(|face| matches!(face.surface(), Surface::Sphere(_)))
            .count();
        assert_eq!(on_the_ball, 1, "{what}: faces on the ball");
        assert_eq!(answer.faces().count(), faces, "{what}: faces in all");
    }
}

/// The edge of the family: `z ≥ 0` runs exactly through the centre, beside
/// `x, y ≥ 0.5`, which miss it. The box `[0.5, 3]² × [0, 3]` keeps the part of
/// the ball's skin in front of all three, and the intersection, either way
/// round, and the box less the ball hold the volume worked out by hand and
/// place points as the hand does.
#[test]
fn a_ball_kept_past_a_plane_through_its_centre_and_two_that_miss_it() {
    let corner = [0.5, 0.5, 0.0];
    let piece = corner_of_a_ball(2.0, corner);
    let block = 2.5 * 2.5 * 3.0;
    for (rule, ball_first, volume) in [
        (Rule::Intersection, true, piece),
        (Rule::Intersection, false, piece),
        (Rule::Difference, false, block - piece),
    ] {
        let what = format!("{rule:?}, the ball first {ball_first}");
        let (model, solid) = fused(rule, ball_first, corner, FAR_CORNER, &what);
        assert_holds(&model, solid, volume, &what);
        for at in [
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 0.1],
            [0.6, 0.6, 1.6],
            [1.2, 1.2, 1.2],
            [1.0, 1.0, -0.1],
            [2.5, 2.5, 2.5],
        ] {
            let placed = model
                .solid(solid)
                .expect("this model made the key")
                .classify_point(
                    Point::from(at),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
            assert_eq!(
                placed,
                by_hand(rule, ball_first, corner, FAR_CORNER, at),
                "{what} placed {at:?}"
            );
        }
    }
}

/// The box turned upside down: `[0.5, 3]² × [−3, 0.5]` holds the ball's skin
/// with `x, y ≥ 0.5` and `z ≤ 0.5`, the piece the centre is in front of
/// `z ≤ 0.5` for, and its corner `(0.5, 0.5, 0.5)` is inside the ball as
/// before. So every rule, both ways round, answers here as the hand does,
/// where without the corner every point whose ray met that piece was refused.
/// The union and the ball less the box do not read the rest of the skin as the
/// part in front of *any* of `x ≤ 0.5`, `y ≤ 0.5` and `z ≥ 0.5`: following
/// only two cuts that way changes no answer here, so `holds.rs`'s
/// `the_part_behind_three_planes_meeting_inside_a_ball_is_one_piece` is what
/// shows that reading stays right.
///
/// **The piece is the corner of a ball with one offset below the centre.**
/// Every point of the ball with `x, y ≥ 0.5` has `z ≥ 0.5` or `z ≤ 0.5`, and the
/// part with `z ≤ 0.5` is the part with `z ≥ −0.5` turned upside down; the part
/// with `z ≥ −0.5` is the part with `z ≥ 0` twice over, less the part with
/// `z ≥ 0.5`. So it is `2·corner(0.5, 0.5, 0) − corner(0.5, 0.5, 0.5)`.
#[test]
fn a_ball_and_a_box_holding_the_part_of_its_skin_below_the_corner_answer_every_rule() {
    let (low, high) = ([0.5, 0.5, -3.0], [3.0, 3.0, 0.5]);
    let piece = 2.0 * corner_of_a_ball(2.0, [0.5, 0.5, 0.0]) - corner_of_a_ball(2.0, [0.5; 3]);
    let (ball, block) = (32.0 * PI / 3.0, 2.5 * 2.5 * 3.5);
    let points = [
        [1.0, 1.0, -1.0],
        [0.6, 0.6, 0.4],
        [1.0, 1.0, 1.0],
        [-1.0, 0.0, 0.0],
        [1.0, 1.0, -1.9],
        [0.4, 1.0, -1.0],
        [2.5, 2.5, 0.0],
    ];
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for ball_first in [true, false] {
            let what = format!("{rule:?}, the ball first {ball_first}");
            let (model, solid) = fused(rule, ball_first, low, high, &what);
            let volume = match (rule, ball_first) {
                (Rule::Union, _) => ball + block - piece,
                (Rule::Intersection, _) => piece,
                (Rule::Difference, true) => ball - piece,
                (Rule::Difference, false) => block - piece,
                (other, _) => panic!("not a rule this probes: {other:?}"),
            };
            assert_holds(&model, solid, volume, &what);
            for at in points {
                let placed = model
                    .solid(solid)
                    .expect("this model made the key")
                    .classify_point(
                        Point::from(at),
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    );
                assert_eq!(
                    placed,
                    by_hand(rule, ball_first, low, high, at),
                    "{what} placed {at:?}"
                );
            }
        }
    }
}

/// The far corner of every box here but the one turned upside down.
const FAR_CORNER: [f64; 3] = [3.0; 3];

/// The volume of the part of a ball of radius `r` about the origin with
/// `x ≥ a`, `y ≥ b` and `z ≥ c`, for `a, b, c ≥ 0` and the corner `(a, b, c)`
/// inside the ball, in closed form.
///
/// **The divergence theorem gives it from the piece's skin.** The volume is a
/// third of the flux of the position `p` out through the skin. On the sphere
/// `p` is the outward normal times `r`, so that part gives `r·S`, `S` the area
/// of the patch; on the face on `x = a` the outward normal is `−x̂`, so it
/// gives `−a` times that face's area, and so for the other two. A face on a
/// plane through the centre gives nothing.
///
/// **Each flat face is the corner of a disc**: on `x = a` the disc of radius
/// `ρ = √(r² − a²)` kept to `y ≥ b`, `z ≥ c`, which is `∫ (√(ρ² − y²) − c) dy`
/// from `b` to `√(ρ² − c²)`, an arcsine and a square root.
///
/// **The patch's area is Gauss–Bonnet's**: `S/r²`, the sides' turning and the
/// corners' turning add up to `2π`. The side on `x = a` is an arc of a circle
/// of radius `ρ` turning `a/(rρ)` per unit length, and it runs from the corner
/// on `z = c` to the one on `y = b`, through `π/2 − asin(b/ρ) − asin(c/ρ)` of
/// that circle. At the corner on `x = a` and `y = b` the two arcs meet at the
/// angle whose cosine is `ab/(ρ_a ρ_b)`, the two planes' normals each taken
/// along the sphere there, so the corner turns `π` less that angle.
fn corner_of_a_ball(r: f64, offsets: [f64; 3]) -> f64 {
    let radius_on = offsets.map(|offset| (r * r - offset * offset).sqrt());
    let [mut corners, mut turning, mut flat] = [0.0f64; 3];
    for this in 0..3 {
        let [next, last] = [(this + 1) % 3, (this + 2) % 3];
        let (d, rho) = (offsets[this], radius_on[this]);
        corners += (d * offsets[next] / (rho * radius_on[next])).acos();
        let arc = PI / 2.0 - (offsets[next] / rho).asin() - (offsets[last] / rho).asin();
        turning += d / r * arc;
        let below = |u: f64| (u * (rho * rho - u * u).sqrt() + rho * rho * (u / rho).asin()) / 2.0;
        let top = (rho * rho - offsets[last] * offsets[last]).sqrt();
        let area = below(top) - below(offsets[next]) - offsets[last] * (top - offsets[next]);
        flat += d * area;
    }
    let patch = r * r * (2.0 * PI - 3.0 * PI + corners - turning);
    (r * patch - flat) / 3.0
}

/// Where `at` is against the answer of `rule` on the ball of radius 2 and the
/// box from `low` to `high`, worked out by hand: in the ball is within 2 of
/// the origin, in the box is between `low` and `high` on each axis. No point
/// asked is on either skin.
fn by_hand(
    rule: Rule,
    ball_first: bool,
    low: [f64; 3],
    high: [f64; 3],
    at: [f64; 3],
) -> Containment {
    let in_ball = at.iter().map(|x| x * x).sum::<f64>() < 4.0;
    let in_box = (0..3).all(|axis| low[axis] < at[axis] && at[axis] < high[axis]);
    let inside = match (rule, ball_first) {
        (Rule::Union, _) => in_ball || in_box,
        (Rule::Intersection, _) => in_ball && in_box,
        (Rule::Difference, true) => in_ball && !in_box,
        (Rule::Difference, false) => in_box && !in_ball,
        (other, _) => panic!("not a rule this probes: {other:?}"),
    };
    if inside {
        Containment::Inside
    } else {
        Containment::Outside
    }
}

/// The ball and the box from `low` to `high` fused under `rule`, the ball
/// first or the box, and the one solid that answers, in a model that
/// validates.
fn fused(
    rule: Rule,
    ball_first: bool,
    low: [f64; 3],
    high: [f64; 3],
    what: &str,
) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let ball = model
        .add_ball(2.0, &Isometry::identity())
        .expect("a ball of radius 2");
    let block = block(&mut model, low, high);
    let (arguments, tools) = if ball_first {
        (vec![ball], vec![block])
    } else {
        (vec![block], vec![ball])
    };
    let answer = model
        .fuse(
            &arguments,
            &tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: {why:?}"))
        .solids()
        .to_vec();
    let [solid] = answer[..] else {
        panic!("{what}: one solid, got {answer:?}");
    };
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what} left a model that does not validate: {why}"));
    (model, solid)
}

/// That `solid` measures `volume`, within what a mesh a thousandth fine says
/// it might be off by.
fn assert_holds(model: &Model, solid: SolidKey, volume: f64, what: &str) {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.001).expect("a deflection above zero"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what} does not measure: {why}"));
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "{what} holds {volume} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// A block from `low` to `high`, a rectangle extruded up `z`.
fn block(model: &mut Model, low: [f64; 3], high: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [low[0], low[1]],
        [high[0], low[1]],
        [high[0], high[1]],
        [low[0], high[1]],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(
            &rectangle,
            high[2] - low[2],
            &Isometry::translation(Vector::new(0.0, 0.0, low[2])).expect("a finite shift"),
        )
        .expect("a block")
}
