//! Two flat faces on one plane, facing apart, along an arc of the circle that
//! plane leaves on a ball.
//!
//! **Why that pair was a hazard.** Two arcs of one circle between the same two
//! corners are told apart by a sign: whether the arc leaves the lower-numbered
//! corner along `∇ball × ∇plane`. The plane's gradient turns round with the way
//! a face spells the plane, so a face spelling it facing up and a face spelling
//! it facing down gave one arc two signs. When two flat faces on one plane face
//! apart and both cut the ball, the ball's skin holds both cuts as one crossing,
//! which keeps the spelling of whichever cut came first, and the flat face along
//! an arc of it in the answer can be the other one. §330.6 named the hazard and
//! built no shape for it. These are two.
//!
//! **A ball between two blocks that meet its equator from opposite sides.** The
//! ball of radius five about the origin, the block `x ∈ [−10, −3]`,
//! `z ∈ [−10, 0]` under it on the left, and the block `x ∈ [3, 10]`,
//! `z ∈ [0, 10]` over it on the right, both with `y ∈ [−10, 10]`. The first
//! block's top and the second's bottom lie on `z = 0` and face apart. The skin
//! took its equator's spelling from one of them, and the arc of the equator the
//! other one runs along was named as two edges of one face each: every one of
//! the twelve fuses below was refused `ResultIsNotManifold { count: 1 }` on the
//! base.
//!
//! **A lower half ball under two blocks side by side.** The ball less
//! everything above `z = 0`, and the blocks `x ∈ [−10, 3]` and `x ∈ [3, 10]`,
//! both with `y ∈ [−10, 10]` and `z ∈ [0, 10]`, standing on its flat face. The
//! skin's equator is spelled by that flat face, facing up, and both blocks'
//! bottoms face down. The two blocks meet the equator at `(3, ±4, 0)`, so it is
//! two arcs between the same two corners, and each arc had a pair spelling the
//! plane apart: the four faces paired crosswise, skin with the wrong block, and
//! the union was refused on the base — two faces walking one edge the same way
//! (`TheFuseWroteABrokenAnswer { broken: CoedgesRunTheSameWay }`) or
//! `Invalid(FacesDoNotAgreeWhichWayTheyFace)`, by the order of the arguments.
//!
//! **Every number is exact.** `3² + 4² = 5²`, so the corners solve onto the ball
//! with nothing to round.
//!
//! # What each volume is held to
//!
//! Each block of the first shape takes half of the cap of height two that
//! `x = ±3` cuts off the ball: the cap holds `π h² (3r − h) / 3 = 52π/3`, so
//! each piece holds `26π/3`, and its curved skin is half the cap's
//! `2πrh = 20π`, which is `10π`. The ball less both pieces holds
//! `500π/3 − 52π/3 = 448π/3` and keeps `100π − 20π = 80π` of its skin. The half
//! ball holds `250π/3` and its skin is `2πr² = 50π`.
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

/// What each block of the first shape takes out of the ball.
const ONE_PIECE: f64 = 26.0 * PI / 3.0;

/// The area of one piece's curved skin.
const ONE_PIECES_SKIN: f64 = 10.0 * PI;

/// What the ball holds once both pieces are taken out of it.
const THE_BALL_LESS_THE_TWO_PIECES: f64 =
    4.0 * PI * RADIUS * RADIUS * RADIUS / 3.0 - 2.0 * ONE_PIECE;

/// The area of the ball's skin once both pieces are taken out of it.
const THE_SKIN_THE_TWO_PIECES_LEAVE: f64 = 4.0 * PI * RADIUS * RADIUS - 2.0 * ONE_PIECES_SKIN;

/// What the ball less everything above its equator holds.
const THE_HALF_BALL: f64 = 250.0 * PI / 3.0;

/// The area of the half ball's curved skin.
const THE_HALF_BALLS_SKIN: f64 = 50.0 * PI;

/// What the ball shares with the two blocks is the two pieces they cut off it,
/// whichever block is named first and whichever side the ball is on.
#[test]
fn a_ball_shares_with_two_blocks_meeting_its_equator_from_opposite_sides_the_two_pieces_they_cut_off()
 {
    for below_first in [true, false] {
        for ball_first in [true, false] {
            let mut model = a_model();
            let (ball, blocks) = the_ball_between_two_blocks(&mut model, below_first);
            let answer = if ball_first {
                fused(&mut model, &[ball], &blocks, Rule::Intersection)
            } else {
                fused(&mut model, &blocks, &[ball], Rule::Intersection)
            };
            assert_eq!(answer.len(), 2, "the two pieces do not touch");
            measures(
                volume_of(&model, &answer),
                2.0 * ONE_PIECE,
                Wall::Outside(2.0 * ONE_PIECES_SKIN),
            );
        }
    }
}

/// The two blocks less the ball are the two blocks, `7 × 20 × 10` each, with a
/// piece taken out of each.
#[test]
fn two_blocks_meeting_a_balls_equator_from_opposite_sides_less_the_ball_lose_a_piece_each() {
    for below_first in [true, false] {
        let mut model = a_model();
        let (ball, blocks) = the_ball_between_two_blocks(&mut model, below_first);
        let answer = fused(&mut model, &blocks, &[ball], Rule::Difference);
        assert_eq!(answer.len(), 2, "the two blocks do not touch");
        measures(
            volume_of(&model, &answer),
            2.0 * (7.0 * 20.0 * 10.0 - ONE_PIECE),
            Wall::AHole(2.0 * ONE_PIECES_SKIN),
        );
    }
}

/// The ball less the two blocks is the ball without the two pieces they cut
/// off it, and the ball united with them is that and the two blocks, whichever
/// block is named first and whichever side the ball is on.
///
/// All six stopped in the mesher until Q508. The skin comes out as pieces of
/// the ball, which the joiner never joins, meeting along stretches of the
/// equator. One of them holds the north pole and goes right round, so the
/// mesher cuts it as a cap, and it read which pole the cap reaches off the
/// surface across the cap's **first** edge. That edge runs along the equator
/// between two pieces of the skin, with the ball on both sides of it, so there
/// was no other surface to read and the cap was refused
/// `ACurvedFaceGoesRightRoundWithNoSeam` (§330.6's second bullet). A temporary
/// print in `which_pole_this_cap_reaches`, since removed, showed the ball on
/// both sides of that edge in all six fuses. That function now reads the pole
/// off the walk round the cap's rim.
#[test]
fn a_ball_less_or_united_with_two_blocks_meeting_its_equator_from_opposite_sides_keeps_all_of_it_but_the_two_pieces()
 {
    for below_first in [true, false] {
        let mut model = a_model();
        let (ball, blocks) = the_ball_between_two_blocks(&mut model, below_first);
        let answer = fused(&mut model, &[ball], &blocks, Rule::Difference);
        assert_eq!(answer.len(), 1, "the ball less two pieces is one solid");
        measures(
            volume_of(&model, &answer),
            THE_BALL_LESS_THE_TWO_PIECES,
            Wall::Outside(THE_SKIN_THE_TWO_PIECES_LEAVE),
        );
        for ball_first in [true, false] {
            let mut model = a_model();
            let (ball, blocks) = the_ball_between_two_blocks(&mut model, below_first);
            let answer = if ball_first {
                fused(&mut model, &[ball], &blocks, Rule::Union)
            } else {
                fused(&mut model, &blocks, &[ball], Rule::Union)
            };
            assert_eq!(answer.len(), 1, "the ball joins the two blocks into one");
            measures(
                volume_of(&model, &answer),
                2.0 * 7.0 * 20.0 * 10.0 + THE_BALL_LESS_THE_TWO_PIECES,
                Wall::Outside(THE_SKIN_THE_TWO_PIECES_LEAVE),
            );
        }
    }
}

/// The half ball united with the two blocks standing on it is the half ball and
/// the slab `20 × 20 × 10` the two blocks make, whichever block is named first
/// and whichever side the half ball is on.
#[test]
fn a_lower_half_ball_united_with_two_blocks_standing_on_it_side_by_side_is_the_two_together() {
    for left_first in [true, false] {
        for half_first in [true, false] {
            let mut model = a_model();
            let (half, blocks) = the_half_ball_under_two_blocks(&mut model, left_first);
            let answer = if half_first {
                fused(&mut model, &[half], &blocks, Rule::Union)
            } else {
                fused(&mut model, &blocks, &[half], Rule::Union)
            };
            assert_eq!(answer.len(), 1);
            measures(
                volume_of(&model, &answer),
                20.0 * 20.0 * 10.0 + THE_HALF_BALL,
                Wall::Outside(THE_HALF_BALLS_SKIN),
            );
        }
    }
}

/// The other fuses of the half ball and the blocks, which answered on the base
/// and still do: the two only touch, so they share nothing and each less the
/// other is itself.
#[test]
fn a_lower_half_ball_and_two_blocks_standing_on_it_share_nothing_and_take_nothing_from_each_other()
{
    for left_first in [true, false] {
        for half_first in [true, false] {
            let mut model = a_model();
            let (half, blocks) = the_half_ball_under_two_blocks(&mut model, left_first);
            let shared = if half_first {
                fused(&mut model, &[half], &blocks, Rule::Intersection)
            } else {
                fused(&mut model, &blocks, &[half], Rule::Intersection)
            };
            assert!(
                shared.is_empty(),
                "two solids that only touch share nothing"
            );
        }

        let mut model = a_model();
        let (half, blocks) = the_half_ball_under_two_blocks(&mut model, left_first);
        let answer = fused(&mut model, &[half], &blocks, Rule::Difference);
        assert_eq!(answer.len(), 1);
        measures(
            volume_of(&model, &answer),
            THE_HALF_BALL,
            Wall::Outside(THE_HALF_BALLS_SKIN),
        );

        let mut model = a_model();
        let (half, blocks) = the_half_ball_under_two_blocks(&mut model, left_first);
        let answer = fused(&mut model, &blocks, &[half], Rule::Difference);
        assert_eq!(answer.len(), 1, "the two blocks side by side are one slab");
        measures(
            volume_of(&model, &answer),
            20.0 * 20.0 * 10.0,
            Wall::Outside(0.0),
        );
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

/// The ball of radius five about the origin, and the two blocks that meet its
/// equator from opposite sides: under it where `x ≤ −3` and over it where
/// `x ≥ 3`, the one under it first when `below_first`.
fn the_ball_between_two_blocks(model: &mut Model, below_first: bool) -> (SolidKey, [SolidKey; 2]) {
    let ball = model
        .add_ball(RADIUS, &Isometry::identity())
        .expect("a ball of radius five");
    let below = block(model, [7.0, 20.0, 10.0], Vector::new(-10.0, -10.0, -10.0));
    let above = block(model, [7.0, 20.0, 10.0], Vector::new(3.0, -10.0, 0.0));
    let blocks = if below_first {
        [below, above]
    } else {
        [above, below]
    };
    (ball, blocks)
}

/// The ball of radius five less everything above `z = 0`, and the two blocks
/// that stand on its flat face side by side, split at `x = 3`: the one on the
/// left first when `left_first`.
fn the_half_ball_under_two_blocks(
    model: &mut Model,
    left_first: bool,
) -> (SolidKey, [SolidKey; 2]) {
    let ball = model
        .add_ball(RADIUS, &Isometry::identity())
        .expect("a ball of radius five");
    let lid = block(model, [20.0, 20.0, 10.0], Vector::new(-10.0, -10.0, 0.0));
    let half = fused(model, &[ball], &[lid], Rule::Difference);
    let [half] = half[..] else {
        panic!(
            "the ball less its top half is one solid, not {}",
            half.len()
        );
    };
    let left = block(model, [13.0, 20.0, 10.0], Vector::new(-10.0, -10.0, 0.0));
    let right = block(model, [7.0, 20.0, 10.0], Vector::new(3.0, -10.0, 0.0));
    let blocks = if left_first {
        [left, right]
    } else {
        [right, left]
    };
    (half, blocks)
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
/// than the wall's area times [`DEFLECTION`]. A solid with no round wall is
/// held to its closed form exactly.
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
