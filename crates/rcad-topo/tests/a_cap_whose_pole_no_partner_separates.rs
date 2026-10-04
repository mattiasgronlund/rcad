//! A ball of radius three about the origin and the block `x, y ∈ [−1, 1]`,
//! `z ∈ [−5, −2]`, which crosses it round its south pole.
//!
//! **Two caps, and no surface across either rim that tells their poles
//! apart.** The block's four walls cut the ball's skin in four arcs right
//! round the south pole. Its top misses the skin, which is lower than
//! `z = −√7 ≈ −2.65` everywhere inside the walls, and its floor is below the
//! ball. So the skin comes out as a cap round the south pole, inside the block,
//! and the rest, which holds the north pole. Each goes right round the ball,
//! and the mesher has to say which pole each one reaches.
//!
//! It used to read that off the surface across the rim's first edge, taking the
//! pole on the side of it the material is on. On `253a594e` a temporary print
//! in `BuildingAMesh::which_pole_this_cap_reaches` showed why neither cap has
//! one to read:
//!
//! - **The cap the block keeps** has the planes `x = ±1` and `y = ±1` across
//!   its four edges, and both poles are on the material's side of every one of
//!   them. The north pole, tried first, was taken for this cap round the south
//!   one, its patches were drawn inside out, and the ball and the block shared
//!   nothing: both intersections and the block less the ball were refused
//!   `Invalid(FacesDoNotAgreeWhichWayTheyFace)`.
//! - **The cap the ball keeps** has the ball across every edge. The fuse cuts
//!   the skin along the whole of each of the four circles, and never joins the
//!   pieces back, so this cap is bounded by other pieces of the same ball. With
//!   no surface across to read, the ball less the block and both unions were
//!   refused `ACurvedFaceGoesRightRoundWithNoSeam`.
//!
//! The pole is now read off the walk round the rim: the walk keeps the
//! material on its left, so a rim walked the way the longitude grows has the
//! material above it, and the walk's own count of the turn it gained says
//! which way it went.
//!
//! # What each volume is held to
//!
//! What the block takes out of the ball is the space between the skin and the
//! block's top, `∫∫ (√(9 − x² − y²) − 2)` over the square. Across `y` that has
//! a closed form, and [`the_piece_and_its_skin`] adds it up over `x` by
//! Simpson's rule, with no kernel in it. So does the area of the skin inside
//! the block.
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

const RADIUS: f64 = 3.0;

/// How fine a mesh every volume here is read off.
const DEFLECTION: f64 = 0.01;

/// The ball's volume and its whole skin.
const THE_BALL: f64 = 4.0 * PI * RADIUS * RADIUS * RADIUS / 3.0;
const THE_BALLS_SKIN: f64 = 4.0 * PI * RADIUS * RADIUS;

/// The block, `2 × 2 × 3`.
const THE_BLOCK: f64 = 12.0;

/// What is left of the ball once the block is taken away: the cap the ball
/// keeps, round the north pole, bounds it.
#[test]
fn a_ball_less_a_block_round_its_south_pole_keeps_the_cap_round_its_north_pole() {
    let (piece, skin) = the_piece_and_its_skin();
    let mut model = a_model();
    let (ball, block) = the_pair(&mut model);
    let answer = fused(&mut model, &[ball], &[block], Rule::Difference);
    assert_eq!(answer.len(), 1);
    measures(
        volume_of(&model, &answer),
        THE_BALL - piece,
        Wall::Outside(THE_BALLS_SKIN - skin),
    );
}

/// The two together, either named first: the same cap round the north pole
/// bounds the answer.
#[test]
fn a_ball_united_with_a_block_round_its_south_pole_is_the_two_less_what_they_share() {
    let (piece, skin) = the_piece_and_its_skin();
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
            THE_BALL + THE_BLOCK - piece,
            Wall::Outside(THE_BALLS_SKIN - skin),
        );
    }
}

/// What the ball and the block share, either named first, and the block less
/// the ball: the cap the block keeps, round the south pole, bounds each.
#[test]
fn a_block_round_a_balls_south_pole_shares_the_cap_round_that_pole() {
    let (piece, skin) = the_piece_and_its_skin();
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
        measures(volume_of(&model, &answer), piece, Wall::Outside(skin));
    }

    let mut model = a_model();
    let (ball, block) = the_pair(&mut model);
    let answer = fused(&mut model, &[block], &[ball], Rule::Difference);
    assert_eq!(answer.len(), 1);
    measures(
        volume_of(&model, &answer),
        THE_BLOCK - piece,
        Wall::AHole(skin),
    );
}

/// What the block takes out of the ball, and the area of the ball's skin
/// inside the block.
///
/// At each `x` the ball's section is a circle of radius `a = √(9 − x²)`, and
/// across `y ∈ [−1, 1]` how far the skin lies below the ball's middle adds up
/// to `√(a² − 1) + a² asin(1/a)`, and its area to `6 asin(1/a)`. The piece is
/// that depth less the block's top, `2` below the middle, over a square of
/// area four. Simpson's
/// rule over `x` on a thousand steps is exact far past the deflection: both are
/// smooth on `[−1, 1]`.
fn the_piece_and_its_skin() -> (f64, f64) {
    let section = |x: f64| {
        let across = (RADIUS * RADIUS - x * x).sqrt();
        let height = (across * across - 1.0).sqrt() + across * across * (1.0 / across).asin();
        let skin = 2.0 * RADIUS * (1.0 / across).asin();
        (height, skin)
    };
    let steps = 1000_u32;
    let step = 2.0 / f64::from(steps);
    let (mut height, mut skin) = (0.0, 0.0);
    for at in 0..=steps {
        let weight = if at == 0 || at == steps {
            1.0
        } else if at % 2 == 1 {
            4.0
        } else {
            2.0
        };
        let (h, s) = section(-1.0 + f64::from(at) * step);
        height += weight * h;
        skin += weight * s;
    }
    (height * step / 3.0 - 2.0 * 4.0, skin * step / 3.0)
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

/// The ball of radius three about the origin, and the block round its south
/// pole.
fn the_pair(model: &mut Model) -> (SolidKey, SolidKey) {
    let ball = model
        .add_ball(RADIUS, &Isometry::identity())
        .expect("a ball of radius three");
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let block = model
        .add_extrusion(
            &rectangle,
            3.0,
            &Isometry::translation(Vector::new(-1.0, -1.0, -5.0)).expect("a finite placement"),
        )
        .expect("building a box needs no decision");
    (ball, block)
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
