//! Two blocks turned to put a face on one plane, spelling it with four numbers
//! in a ratio that is neither `1` nor `−1`.
//!
//! **Why such a pair matters.** `intersect` calls two planes one surface when
//! their coefficients are exactly proportional, so a ball cut by both holds the
//! two cuts as one crossing, spelled by whichever cut came first. An arc of that
//! crossing is named by the plane as well as the ball, and the flat face along
//! it in the answer can be the other block's. The name folded only the exact
//! negation, and `Plane::transformed` does not scale a turned normal back to
//! length one, so §330.3 asked whether a turn could make the two spellings
//! differ by some other factor.
//!
//! **One can.** A block turned about `z` by `0.29` and then by
//! `0.495 398 163 397 448 4` puts its face `y = 0` on the plane `x = y`, and so
//! does one turned by `0.45` and then by `0.335 398 163 397 448 05`. The first
//! spells it `(−c, c, 0, 0)` and the second `(−c′, c′, 0, 0)`, with
//! `c = 0.707 106 781 186 547 5` and `c′ = 0.707 106 781 186 547 6`, an ulp
//! apart. Both turns come to a quarter of a half turn; the first part of each
//! was searched for among `0.01, 0.02, …, 1.99` with the second fine-tuned over
//! eight ulps, and of those 1592 composed turns 23 land the face on `c` and 10
//! on `c′`. A single turn about `z` near a quarter of a half turn landed on
//! neither over sixty neighbouring angles: its two components stepped past
//! each other an ulp at a time without ever being equal.
//!
//! **What it cost.** Two such blocks on one side of the plane, each taking half
//! of the cap a plane three from the centre cuts off a ball of radius five,
//! were refused `ResultIsNotManifold { count: 1 }` in every one of the twelve
//! fuses below, whichever block is named first and whichever side the ball is
//! on: the arc of the ball's crossing along the second block was named once by
//! the skin and once by the flat face, with two different names.
//!
//! # What each volume is held to
//!
//! Each block takes half of the cap of height two, `26π/3`, and half of its
//! curved skin, `10π`. Every vertex of a mesh is on the ball and every triangle
//! a chord of it, so a round wall bounding a solid from outside measures under
//! its closed form and one that is a hole measures over, by no more than that
//! wall's area times [`DEFLECTION`].

use core::f64::consts::{FRAC_1_SQRT_2, PI};

use rcad_geom::{Axis, Deflection, Isometry, PlaneAgreement, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

const RADIUS: f64 = 5.0;

/// How fine a mesh every volume here is read off.
const DEFLECTION: f64 = 0.01;

/// What each block takes out of the ball.
const ONE_PIECE: f64 = 26.0 * PI / 3.0;

/// The area of one piece's curved skin.
const ONE_PIECES_SKIN: f64 = 10.0 * PI;

/// What each block holds, `7 × 10 × 20`.
const ONE_BLOCK: f64 = 1400.0;

/// What the ball holds once both pieces are taken out of it.
const THE_BALL_LESS_THE_TWO_PIECES: f64 =
    4.0 * PI * RADIUS * RADIUS * RADIUS / 3.0 - 2.0 * ONE_PIECE;

/// The area of the ball's skin once both pieces are taken out of it.
const THE_SKIN_THE_TWO_PIECES_LEAVE: f64 = 4.0 * PI * RADIUS * RADIUS - 2.0 * ONE_PIECES_SKIN;

/// The two turns that put a block's face `y = 0` on `x = y` spelled with
/// `c = 0.707 106 781 186 547 5`.
const TO_THE_SHORTER_SPELLING: [f64; 2] = [0.29, 0.495_398_163_397_448_4];

/// The two turns that put it there spelled with `c′ = 0.707 106 781 186 547 6`,
/// which is [`FRAC_1_SQRT_2`] and the `f64` just above `c`.
const TO_THE_LONGER_SPELLING: [f64; 2] = [0.45, 0.335_398_163_397_448_05];

/// The two blocks' faces on `x = y` hold four numbers each in a ratio of
/// `c′/c`: one plane to `intersect`, and not one spelling or its negation.
#[test]
fn two_turns_put_a_face_on_one_plane_with_coefficients_an_ulp_apart_in_scale() {
    let mut model = a_model();
    let (_, [left, right]) = the_ball_and_two_blocks_on_one_side(&mut model, true);
    let [shorter, longer] = [left, right].map(|block| {
        model
            .solid(block)
            .expect("this model made the key")
            .faces()
            .map(|face| face.plane())
            .find(|plane| plane.coefficients()[3] == 0.0)
            .expect("each block has a face through the turn's axis")
    });
    let c = f64::from_bits(FRAC_1_SQRT_2.to_bits() - 1);
    assert_eq!(shorter.coefficients(), [-c, c, 0.0, 0.0]);
    assert_eq!(
        longer.coefficients(),
        [-FRAC_1_SQRT_2, FRAC_1_SQRT_2, 0.0, 0.0]
    );
    assert_eq!(shorter.agreement_with(&longer), PlaneAgreement::SameWay);
}

/// The ball and the two blocks, every fuse of them, whichever block is named
/// first and whichever side the ball is on, each checked against its closed
/// form.
#[test]
fn a_ball_fused_with_two_blocks_spelling_its_cut_plane_an_ulp_apart_in_scale() {
    for left_first in [true, false] {
        for ball_first in [true, false] {
            let mut model = a_model();
            let (ball, blocks) = the_ball_and_two_blocks_on_one_side(&mut model, left_first);
            let answer = fused(&mut model, ball_first, ball, blocks, Rule::Intersection);
            assert_eq!(answer.len(), 2, "the two pieces do not touch");
            measures(
                volume_of(&model, &answer),
                2.0 * ONE_PIECE,
                Wall::Outside(2.0 * ONE_PIECES_SKIN),
            );

            let mut model = a_model();
            let (ball, blocks) = the_ball_and_two_blocks_on_one_side(&mut model, left_first);
            let answer = fused(&mut model, ball_first, ball, blocks, Rule::Union);
            assert_eq!(answer.len(), 1, "the ball joins the two blocks into one");
            measures(
                volume_of(&model, &answer),
                2.0 * ONE_BLOCK + THE_BALL_LESS_THE_TWO_PIECES,
                Wall::Outside(THE_SKIN_THE_TWO_PIECES_LEAVE),
            );

            let mut model = a_model();
            let (ball, blocks) = the_ball_and_two_blocks_on_one_side(&mut model, left_first);
            let answer = fused(&mut model, ball_first, ball, blocks, Rule::Difference);
            if ball_first {
                assert_eq!(answer.len(), 1, "the ball less two pieces is one solid");
                measures(
                    volume_of(&model, &answer),
                    THE_BALL_LESS_THE_TWO_PIECES,
                    Wall::Outside(THE_SKIN_THE_TWO_PIECES_LEAVE),
                );
            } else {
                assert_eq!(answer.len(), 2, "the two blocks do not touch");
                measures(
                    volume_of(&model, &answer),
                    2.0 * (ONE_BLOCK - ONE_PIECE),
                    Wall::AHole(2.0 * ONE_PIECES_SKIN),
                );
            }
        }
    }
}

/// **Two blocks on opposite sides of the plane are still refused**, and not by
/// the arc's name. The second block here is the first one's mirror, `x ≥ 3`
/// and `y ≥ 0`, so its face on `x = y` is spelled `(c′, −c′, 0, 0)` facing the
/// other way. Every fuse of the ball with the two blocks is refused
/// `ResultIsNotManifold { count: 1 }`, and a print of each side's corners
/// showed why: the ball's skin and the second block's flat faces put the two
/// corners at the ends of each arc on four different corner numbers, so the
/// skin's arc and the face's arc run between different corners before any name
/// of the arc is asked. A corner where a plane meets a ball is named by the
/// plane as each face spells it (`names_of_the_three_surfaces`), and the two
/// spellings solve onto two boxes a bit apart, so nothing joins them. With
/// both blocks turned the same way the two spellings are exact negations, the
/// boxes come out the same and all twelve fuses answer.
#[test]
fn two_blocks_facing_apart_across_a_plane_spelled_an_ulp_apart_in_scale_are_still_refused() {
    for left_first in [true, false] {
        for ball_first in [true, false] {
            for rule in [Rule::Intersection, Rule::Union, Rule::Difference] {
                let mut model = a_model();
                let ball = model
                    .add_ball(RADIUS, &Isometry::identity())
                    .expect("a ball of radius five");
                let left = block(
                    &mut model,
                    [[-10.0, -10.0], [-3.0, 0.0]],
                    TO_THE_SHORTER_SPELLING,
                );
                let right = block(
                    &mut model,
                    [[3.0, 0.0], [10.0, 10.0]],
                    TO_THE_LONGER_SPELLING,
                );
                let blocks = if left_first {
                    [left, right]
                } else {
                    [right, left]
                };
                let (arguments, tools) = if ball_first {
                    (vec![ball], blocks.to_vec())
                } else {
                    (blocks.to_vec(), vec![ball])
                };
                let refused = model.fuse(
                    &arguments,
                    &tools,
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                assert!(
                    matches!(refused, Err(Error::ResultIsNotManifold { count: 1 })),
                    "{rule:?}: {refused:?}"
                );
            }
        }
    }
}

/// Which way a round wall of this area bounds the solid being measured.
enum Wall {
    /// The solid is on the ball's side of it, so its mesh measures under.
    Outside(f64),
    /// The solid is round it, so its mesh leaves the hole too small and measures
    /// over.
    AHole(f64),
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(50.0).expect("a positive size"))
}

/// The ball of radius five about the origin, and two blocks on the side of
/// `x = y` the material of a face `y = 0` turned onto it is on: one over
/// `x ≤ −3` before the turn, spelling the plane with `c`, and one over
/// `x ≥ 3`, spelling it with `c′`, the first of them first when `left_first`.
fn the_ball_and_two_blocks_on_one_side(
    model: &mut Model,
    left_first: bool,
) -> (SolidKey, [SolidKey; 2]) {
    let ball = model
        .add_ball(RADIUS, &Isometry::identity())
        .expect("a ball of radius five");
    let left = block(
        model,
        [[-10.0, -10.0], [-3.0, 0.0]],
        TO_THE_SHORTER_SPELLING,
    );
    let right = block(model, [[3.0, -10.0], [10.0, 0.0]], TO_THE_LONGER_SPELLING);
    let blocks = if left_first {
        [left, right]
    } else {
        [right, left]
    };
    (ball, blocks)
}

/// The block over the rectangle from `low` to `high`, from `z = −10` to
/// `z = 10`, turned about `z` by each of `turns` in order.
///
/// Its face `y = 0` stays a plane through the origin however it is turned,
/// with an offset of exactly zero: the slide down `z` is along the axis every
/// turn is about, so the turned normal has no `z` part to meet it.
fn block(model: &mut Model, [low, high]: [[f64; 2]; 2], turns: [f64; 2]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![low, [high[0], low[1]], high, [low[0], high[1]]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let solid = model
        .add_extrusion(
            &rectangle,
            20.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -10.0)).expect("a finite placement"),
        )
        .expect("building a box needs no decision");
    for angle in turns {
        model
            .place_solid(
                solid,
                &Isometry::rotation(&Axis::z(), angle).expect("a finite turn"),
            )
            .expect("this model made the key");
    }
    solid
}

/// Fuse the ball with the two blocks, the ball first when `ball_first`, under a
/// certified policy, and insist the model it leaves is valid.
fn fused(
    model: &mut Model,
    ball_first: bool,
    ball: SolidKey,
    blocks: [SolidKey; 2],
    rule: Rule,
) -> Vec<SolidKey> {
    let (arguments, tools) = if ball_first {
        (vec![ball], blocks.to_vec())
    } else {
        (blocks.to_vec(), vec![ball])
    };
    let answer = model
        .fuse(
            &arguments,
            &tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("this {rule:?} is answered: {why:?}"))
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a fuse writes a sound model or writes nothing");
    answer
}

fn volume_of(model: &Model, solids: &[SolidKey]) -> f64 {
    solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(DEFLECTION).expect("a positive length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("the answer measures")
                .volume()
        })
        .sum()
}

/// Insist a volume is off its closed form on the side `wall` says, by no more
/// than the wall's area times [`DEFLECTION`].
fn measures(found: f64, closed_form: f64, wall: Wall) {
    let (off, area) = match wall {
        Wall::Outside(area) => (closed_form - found, area),
        Wall::AHole(area) => (found - closed_form, area),
    };
    assert!(
        (0.0..=area * DEFLECTION).contains(&off),
        "measured {found}, closed form {closed_form}, {off} off on the wrong side or too far"
    );
}
