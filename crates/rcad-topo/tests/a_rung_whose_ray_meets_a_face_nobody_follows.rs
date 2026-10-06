//! A piece only a rung was found inside, whose rung's ray meets the skin of a
//! ball cut by two planes (queue rows Q1040 and Q1162, `docs/decisions.md`
//! §35.6). The file's name is what it used to pin: since Q1162 `holds.rs`
//! follows that skin, and nobody fails to follow it any more.
//!
//! Three solids:
//!
//! - **the half ball**, a ball of radius 3 at the origin cut to `z <= 0`;
//! - **the small ball**, radius 1.5 at `(2, 0, 0)`, which pokes out of the half
//!   ball's rim: its skin crosses the half ball's flat face and its skin where
//!   all three meet, at `(2.6875, ±1.3332, 0)`;
//! - **the cut ball**, a ball of radius 20 at `(16, 0, 19.5)` cut to
//!   `x <= 26`, `y <= 10`. Its skin is bounded by an arc of each of two circles
//!   that cross at `(26, 10, 19.5 ± √200)`. It touches neither of the other
//!   two: it dips half a unit below `z = 0` only round `(16, 0)`, while its box
//!   runs from `x = −4` to `26`, `y = −20` to `10` and `z = −0.5` up, so every
//!   point of the small ball's skin past the half ball is inside that box, and
//!   a ray from one meets the cut ball's skin.
//!
//! **Until Q1162 that skin was refused by name.** Its two planes miss the
//! ball's centre, and `holds.rs`'s `is_a_family_this_follows` took a ball's
//! skin only when every plane cutting it ran through the centre, so the cut
//! ball could place no point whose ray met its skin, and every fuse with a
//! piece placed against it was refused `AFaceTrimmedByACurveThisCannotFollow`.
//! Two planes leave the part of a sphere in front of both in one piece
//! whatever they keep, and the skin is that part, so it is followed now.
//!
//! **Every fuse the cut ball is in now does what the two balls do without
//! it.** Under intersection, and the half ball less the other two, the answer
//! is right, held to its volume worked out by hand. The union, either way
//! round, and the other two less the half ball are refused
//! `ResultIsNotManifold { count: 4 }`, and so is the same fuse with the cut
//! ball left out: that refusal is the small ball's and the half ball's, and
//! not the cut ball's.

use std::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Point, Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, Model, ProfileBuilder, Rule, SolidKey};

/// Each rule, both ways round, with the cut ball and without it.
///
/// The small ball and the big one share a lens: the two skins cross on the
/// plane `x = (4 + 9 − 2.25)/4 = 2.6875`, the big ball's cap past it is
/// `π · 0.3125² · (9 − 0.3125)/3` and the small ball's on the near side
/// `π · 2.1875² · (4.5 − 2.1875)/3`, which add up to `1525π/384`. Both centres
/// are on `z = 0`, so the half ball holds half of it, `1525π/768`. The half
/// ball is `18π` and the small ball `4.5π`.
#[test]
fn a_ball_poking_out_of_a_half_ball_beside_a_cut_ball_answers_or_refuses_as_without_it() {
    let shared = 1525.0 * PI / 768.0;
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for small_ball_first in [true, false] {
            let what = format!("{rule:?}, the small ball first {small_ball_first}");
            let (model, fused) = fuse_the_three(rule, small_ball_first, true);
            match (rule, small_ball_first) {
                (Rule::Intersection, _) => {
                    let answer = fused.unwrap_or_else(|why| panic!("{what}: {why:?}"));
                    let [solid] = answer[..] else {
                        panic!("{what}: one solid, got {answer:?}");
                    };
                    assert_holds(&model, solid, shared, &what);
                }
                (Rule::Difference, false) => {
                    let answer = fused.unwrap_or_else(|why| panic!("{what}: {why:?}"));
                    let [solid] = answer[..] else {
                        panic!("{what}: one solid, got {answer:?}");
                    };
                    assert_holds(&model, solid, 18.0 * PI - shared, &what);
                }
                (Rule::Union | Rule::Difference, _) => {
                    assert!(
                        matches!(fused, Err(Error::ResultIsNotManifold { count: 4 })),
                        "{what}: {fused:?}"
                    );
                    let (_, alone) = fuse_the_three(rule, small_ball_first, false);
                    assert!(
                        matches!(alone, Err(Error::ResultIsNotManifold { count: 4 })),
                        "{what}, without the cut ball: {alone:?}"
                    );
                }
                (other, _) => panic!("not a rule this probes: {other:?}"),
            }
        }
    }
}

/// Points on the small ball's skin, in the cut ball and outside it, each of
/// whose rays meets the cut ball's skin, against what the cut ball holds by
/// hand: within 20 of `(16, 0, 19.5)` and in the box.
///
/// `(3.5, 0, 0)` is `√(12.5² + 19.5²) = √536.5 ≈ 23.2` from the centre, so it is
/// outside; until Q1162 the cut ball could not place it. `(30, 0, 19.5)` is in
/// the ball but past `x = 26`; `(16, 0, 19.5)` and `(20, 5, 30)`, `√141.25` from
/// the centre, are in both.
#[test]
fn the_cut_ball_places_points_its_rays_meet_its_skin_from() {
    let Three {
        model, cut_ball, ..
    } = three();
    for (at, expected) in [
        ([3.5, 0.0, 0.0], Containment::Outside),
        ([30.0, 0.0, 19.5], Containment::Outside),
        ([16.0, 0.0, 19.5], Containment::Inside),
        ([20.0, 5.0, 30.0], Containment::Inside),
    ] {
        let placed = model
            .solid(cut_ball)
            .expect("this model made the key")
            .classify_uncertain_point(
                Point::from(at),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
        assert_eq!(placed, expected, "the cut ball placed {at:?}");
    }
}

/// The small ball and the cut ball, or the small ball alone, fused with the
/// half ball, and the model the answer is in.
fn fuse_the_three(
    rule: Rule,
    small_ball_first: bool,
    with_the_cut_ball: bool,
) -> (Model, Result<Vec<SolidKey>, Error>) {
    let Three {
        mut model,
        half_ball,
        small_ball,
        cut_ball,
    } = three();
    let mut balls = vec![small_ball];
    if with_the_cut_ball {
        balls.push(cut_ball);
    }
    let (arguments, tools) = if small_ball_first {
        (balls, vec![half_ball])
    } else {
        (vec![half_ball], balls)
    };
    let fused = model
        .fuse(
            &arguments,
            &tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|answer| answer.solids().to_vec());
    (model, fused)
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

/// `solid` validates and measures `volume`, give or take what the mesh says.
fn assert_holds(model: &Model, solid: SolidKey, volume: f64, what: &str) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what} left a model that does not validate: {why}"));
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.001).expect("a fine mesh"),
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
