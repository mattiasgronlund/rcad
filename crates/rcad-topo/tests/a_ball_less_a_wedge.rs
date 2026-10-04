//! A ball of radius five less the block `x ∈ [−10, −3]`, `y ∈ [−10, 10]`,
//! `z ∈ [0, 10]`: the ball with the part of it where `x ≤ −3` and `z ≥ 0` taken
//! away.
//!
//! **Two circles on one ball, on two planes, through the same two corners.** The
//! block's walls `x = −3` and `z = 0` cut the ball in a circle of radius four and
//! in its equator, and both pass through `(−3, ±4, 0)`. What is left of the
//! ball's skin runs along an arc of each between those two corners: the upper
//! half of the small circle, over `(−3, 0, 4)`, and the stretch of the equator
//! beyond `x = −3`, through `(−5, 0, 0)`. Both leave the lower-numbered corner
//! the same way round their own circle, so the curved surface and that sign,
//! which is all an arc was named by, named them as one edge with four faces
//! along it (§247.6, §251.6). The plane each arc lies on is what tells them
//! apart.
//!
//! **Every number is exact.** `3² + 4² = 5²`, so the corners solve onto the ball
//! with nothing to round.
//!
//! # What each volume is held to
//!
//! The piece the block takes out of the ball is half of the cap of height two
//! that `x = −3` cuts off, halved again by `z = 0`: the cap holds
//! `π h² (3r − h) / 3 = 52π/3`, so the piece holds `26π/3`, and its curved skin
//! is half the cap's `2πrh = 20π`, which is `10π`.
//!
//! Every vertex of a mesh is on the ball and every triangle a chord of it, so a
//! meshed round wall sits inside the ball, by no more than [`DEFLECTION`]. A
//! solid whose round wall bounds it from outside measures under its closed form,
//! one whose round wall is a hole in it measures over, and either way by no more
//! than that wall's area times the deflection.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

const RADIUS: f64 = 5.0;

/// How fine a mesh every volume here is read off.
const DEFLECTION: f64 = 0.01;

/// What the block takes out of the ball.
const THE_PIECE: f64 = 26.0 * PI / 3.0;

/// The area of the piece's curved skin.
const THE_PIECES_SKIN: f64 = 10.0 * PI;

/// The ball's volume and its whole skin.
const THE_BALL: f64 = 4.0 * PI * RADIUS * RADIUS * RADIUS / 3.0;
const THE_BALLS_SKIN: f64 = 4.0 * PI * RADIUS * RADIUS;

/// The block, `7 × 20 × 10`.
const THE_BLOCK: f64 = 7.0 * 20.0 * 10.0;

/// The ball less the block is the ball less the piece, `500π/3 − 26π/3 = 158π`,
/// bounded by all of the ball's skin but the piece's and under it by no more
/// than `90π` times the deflection. The two united, either named first, are
/// that and the block, `158π + 1400`, bounded by the same skin.
///
/// The difference used to refuse `ResultIsNotManifold { count: 4 }` on the one
/// edge both arcs were named as. With the plane in an arc's name `assemble`
/// built the answer, and the sweep of it refused the skin (§330). What is left
/// of the skin is three pieces of the ball, which the joiner never joins: the
/// part above the equator beyond `x = −3`, and the two parts below it either
/// side of `x = −3`. The first goes right round the ball and holds the north
/// pole, so the mesher cuts it as a cap, and it read which pole the cap reaches
/// off a surface across the cap's rim. Its rim has two edges: the equator's
/// long arc, with the ball on both sides, and the arc on `x = −3`, with both
/// poles on the same side of that plane. So neither could say, and the
/// difference and both unions were refused `ACurvedFaceGoesRightRoundWithNoSeam`.
/// The mesher now reads the pole off the walk round the rim instead
/// (`a_cap_whose_pole_no_partner_separates.rs`).
#[test]
fn a_ball_less_or_united_with_a_block_over_one_corner_of_it_is_measured_against_the_piece() {
    let mut model = a_model();
    let (ball, block) = the_pair(&mut model);
    let answer = fused(&mut model, &[ball], &[block], Rule::Difference);
    assert_eq!(answer.len(), 1);
    measures(
        volume_of(&model, &answer),
        THE_BALL - THE_PIECE,
        Wall::Outside(THE_BALLS_SKIN - THE_PIECES_SKIN),
    );

    for ball_first in [true, false] {
        let mut model = a_model();
        let (ball, block) = the_pair(&mut model);
        let (argument, tool) = if ball_first {
            (ball, block)
        } else {
            (block, ball)
        };
        let answer = fused(&mut model, &[argument], &[tool], Rule::Union);
        assert_eq!(answer.len(), 1);
        measures(
            volume_of(&model, &answer),
            THE_BALL + THE_BLOCK - THE_PIECE,
            Wall::Outside(THE_BALLS_SKIN - THE_PIECES_SKIN),
        );
    }
}

/// The other three fuses of the pair, which answered before the plane was in an
/// arc's name and still do.
///
/// What the two share is the piece, whichever is the argument. The block less
/// the ball is the block, `7 × 20 × 10`, with the piece taken out of it.
#[test]
fn the_ball_and_the_block_share_the_piece_the_difference_takes_away() {
    for ball_first in [true, false] {
        let mut model = a_model();
        let (ball, block) = the_pair(&mut model);
        let (argument, tool) = if ball_first {
            (ball, block)
        } else {
            (block, ball)
        };
        let answer = fused(&mut model, &[argument], &[tool], Rule::Intersection);
        assert_eq!(answer.len(), 1);
        measures(
            volume_of(&model, &answer),
            THE_PIECE,
            Wall::Outside(THE_PIECES_SKIN),
        );
    }

    let mut model = a_model();
    let (ball, block) = the_pair(&mut model);
    let answer = fused(&mut model, &[block], &[ball], Rule::Difference);
    assert_eq!(answer.len(), 1);
    measures(
        volume_of(&model, &answer),
        THE_BLOCK - THE_PIECE,
        Wall::AHole(THE_PIECES_SKIN),
    );
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

/// The ball of radius five about the origin, and the block over its corner
/// `x ≤ −3`, `z ≥ 0`.
fn the_pair(model: &mut Model) -> (SolidKey, SolidKey) {
    let ball = model
        .add_ball(RADIUS, &Isometry::identity())
        .expect("a ball of radius five");
    let block = block(model, [7.0, 20.0, 10.0], Vector::new(-10.0, -10.0, 0.0));
    (ball, block)
}

fn block(model: &mut Model, [along_x, along_y, along_z]: [f64; 3], at: Vector) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [along_x, 0.0],
        [along_x, along_y],
        [0.0, along_y],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(
            &rectangle,
            along_z,
            &Isometry::translation(at).expect("a finite placement"),
        )
        .expect("building a box needs no decision")
}

/// Fuse under a certified policy and insist the model it leaves is valid.
fn fused(
    model: &mut Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
) -> Vec<SolidKey> {
    let answer = model
        .fuse(
            arguments,
            tools,
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
        (0.0..area * DEFLECTION).contains(&off),
        "measured {found}, closed form {closed_form}, {off} off on the wrong side or too far"
    );
}
