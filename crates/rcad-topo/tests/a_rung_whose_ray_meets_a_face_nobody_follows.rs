//! A piece only a rung was found inside, whose rung's ray meets a face nobody
//! follows (queue row Q1040, `docs/decisions.md` §35.6).
//!
//! Three solids:
//!
//! - **the half ball**, a ball of radius 3 at the origin cut to `z <= 0`;
//! - **the small ball**, radius 1.5 at `(2, 0, 0)`, which pokes out of the half
//!   ball's rim: its skin crosses the half ball's flat face and its skin where
//!   all three meet, at `(2.6875, ±1.3332, 0)`;
//! - **the cut ball**, a ball of radius 20 at `(16, 0, 19.5)` cut to
//!   `x <= 26`, `y <= 10`. Its skin is bounded by an arc of each of two circles
//!   that cross, which `holds.rs` does not follow, the face
//!   `a_walk_neither_winding_nor_the_axis_places.rs`'s
//!   `a_ball_face_bounded_by_two_arcs_places_a_hole_by_the_arcs_own_circles`
//!   is refused on. It touches neither of the other two: it dips half a unit
//!   below `z = 0` only round `(16, 0)`, while its box runs from `x = −4` to
//!   `26`, `y = −20` to `10` and `z = −0.5` up, so every point of the small
//!   ball's skin past the half ball is inside that box, and a ray from one
//!   meets the cut ball's skin.
//!
//! **The small ball's skin outside the half ball is one piece, and only a rung
//! is found inside it.** The piece is the part of the skin above `z = 0` *or*
//! outside the big ball's skin, and every point `candidate_points` offers is
//! on one of the two and not the other, so none is proved inside. A step half
//! as long, `(3.486, 0, 0.203)`, is past both, and is the first point proved
//! inside (a print in `place_the_piece`, since taken out).
//!
//! **Fused with the small ball and the cut ball as the arguments**, that piece
//! is placed against the cut ball first, in its own group. The rung's ray met
//! the cut ball's skin, no side the piece shares is with the cut ball, its
//! corners' rays meet the same skin, and the cut ball is not all flat. Until
//! Q1040 every rule refused `Undecidable` under "a point in a piece that every
//! solid could place", which reads as "loosen the policy" about a question no
//! policy was ever asked. It now names the face, as a point offered before the
//! rungs already did.
//!
//! **The other way round**, with the half ball as the argument, a piece of the
//! half ball meets the cut ball first and is refused by that same name, as it
//! was before.
//!
//! The answer exists: the cut ball is clear of the other two, and the small
//! ball and the half ball share a lens. So each refusal is the gap that skin
//! is, and moves when `holds.rs` follows it.

use rcad_geom::{Isometry, Scale, Surface, Vector};
use rcad_predicates::{Point, Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, Model, ProfileBuilder, Rule, SolidKey};

/// Every rule, both ways round, is refused naming the cut ball's skin.
#[test]
fn a_ball_poking_out_of_a_half_ball_beside_a_cut_ball_is_refused_naming_its_skin() {
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for small_ball_first in [true, false] {
            let Three {
                mut model,
                half_ball,
                small_ball,
                cut_ball,
            } = three();
            let (arguments, tools) = if small_ball_first {
                (vec![small_ball, cut_ball], vec![half_ball])
            } else {
                (vec![half_ball], vec![small_ball, cut_ball])
            };
            let what = format!("{rule:?}, the small ball first {small_ball_first}");
            let refused = model.fuse(
                &arguments,
                &tools,
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            let Err(Error::AFaceTrimmedByACurveThisCannotFollow { face }) = refused else {
                panic!("{what}: {refused:?}");
            };
            assert_is_the_cut_balls_skin(&model, face, &what);
        }
    }
}

/// A point on the small ball's skin past the half ball's rim, `(3.5, 0, 0)`,
/// is in the cut ball's box, and its ray meets the cut ball's skin.
#[test]
fn the_cut_ball_cannot_place_a_point_on_the_small_balls_skin() {
    let Three {
        model, cut_ball, ..
    } = three();
    let placed = model
        .solid(cut_ball)
        .expect("this model made the key")
        .classify_uncertain_point(
            Point::from([3.5, 0.0, 0.0]),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
    let Containment::CannotFollowAFacesTrimming { face } = placed else {
        panic!("the cut ball placed (3.5, 0, 0): {placed:?}");
    };
    assert_is_the_cut_balls_skin(&model, face, "the point (3.5, 0, 0)");
}

struct Three {
    model: Model,
    half_ball: SolidKey,
    small_ball: SolidKey,
    cut_ball: SolidKey,
}

fn three() -> Three {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let ball = model
        .add_ball(3.0, &Isometry::identity())
        .expect("a ball of radius 3");
    let below = block(&mut model, [-10.0, -10.0, -10.0], [10.0, 10.0, 0.0]);
    let half_ball = one_solid(&mut model, ball, below, Rule::Intersection);
    let small_ball = model
        .add_ball(1.5, &at(2.0, 0.0, 0.0))
        .expect("a ball of radius 1.5");
    let big = model
        .add_ball(20.0, &at(16.0, 0.0, 19.5))
        .expect("a ball of radius 20");
    let corner = block(&mut model, [-200.0, -200.0, -200.0], [26.0, 10.0, 200.0]);
    let cut_ball = one_solid(&mut model, big, corner, Rule::Intersection);
    Three {
        model,
        half_ball,
        small_ball,
        cut_ball,
    }
}

fn assert_is_the_cut_balls_skin(model: &Model, face: rcad_topo::FaceKey, what: &str) {
    let surface = model
        .face(face)
        .expect("the refusal names a face of the model")
        .surface();
    assert!(
        matches!(surface, Surface::Sphere(ball) if ball.radius() == 20.0),
        "{what}: named {surface:?}, not the cut ball's skin"
    );
}

fn one_solid(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> SolidKey {
    let answer = model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("building a solid: {why:?}"))
        .solids()
        .to_vec();
    let [solid] = answer[..] else {
        panic!("one solid, got {answer:?}");
    };
    solid
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
        .add_extrusion(&rectangle, high[2] - low[2], &at(0.0, 0.0, low[2]))
        .expect("a block")
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}
