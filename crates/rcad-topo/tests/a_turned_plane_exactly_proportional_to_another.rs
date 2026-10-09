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
//! **A whole circle cost the same.** A ball or a bar that the plane cuts in one
//! closed curve nothing else cuts, with one block holding the whole of it on its
//! face and the other block's face elsewhere on the plane, was refused
//! `ResultIsNotManifold { count: 1 }` in every fuse: the round wall carried the
//! circle as one spelling and the flat face as the other, so the closed edge
//! was named twice (`a_ball_cut_whole_by_a_plane_two_blocks_spell_an_ulp_apart_in_scale`).
//!
//! **So did a rim three surfaces meet in.** A ball on the cap of a bar of its
//! own radius, the cap turned about `x` onto `y = z` and a block clear of both
//! turned onto it with the other spelling, was refused the same way in every
//! fuse with the bar spelling the plane `c′`: the rim's two names, the cap with
//! the bar's wall and the plane with the ball, were joined only where the two
//! planes were spelled alike or negated
//! (`a_ball_on_a_bars_cap_a_block_spells_an_ulp_apart_in_scale`).
//!
//! # What each volume is held to
//!
//! Each block takes half of the cap of height two, `26π/3`, and half of its
//! curved skin, `10π`. Every vertex of a mesh is on its ball or its bar and
//! every triangle a chord of it, so a round wall bounding a solid from outside
//! measures under its closed form and one that is a hole measures over, by no
//! more than that wall's area times [`DEFLECTION`].

use core::f64::consts::{FRAC_1_SQRT_2, FRAC_PI_2, FRAC_PI_4, PI};

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
/// `c = 0.707 106 781 186 547 5`. About `x` instead of `z` they put a face
/// `z = 0` on `y = z` spelled `(0, c, −c, 0)` the same way.
const TO_THE_SHORTER_SPELLING: [f64; 2] = [0.29, 0.495_398_163_397_448_4];

/// The two turns that put it there spelled with `c′ = 0.707 106 781 186 547 6`,
/// which is [`FRAC_1_SQRT_2`] and the `f64` just above `c`.
const TO_THE_LONGER_SPELLING: [f64; 2] = [0.45, 0.335_398_163_397_448_05];

/// How long the bar a ball sits on is, from the cap through the ball's centre.
const THE_BARS_LENGTH: f64 = 8.0;

/// Before its turns, the block whose face on `x = y` holds the whole circle a
/// round solid is cut in: `12 × 6 × 20`, every other face of it at least six
/// from the axis.
const THE_NEAR_BLOCK: [[f64; 2]; 2] = [[-6.0, -6.0], [6.0, 0.0]];

/// What [`THE_NEAR_BLOCK`] holds.
const THE_NEAR_BLOCKS_VOLUME: f64 = 1440.0;

/// Before its turns, the block half a unit clear of [`THE_NEAR_BLOCK`] whose
/// face on `x = y` misses every round solid here: `3.5 × 10 × 20`.
const THE_FAR_BLOCK: [[f64; 2]; 2] = [[6.5, -10.0], [10.0, 0.0]];

/// What [`THE_FAR_BLOCK`] holds.
const THE_FAR_BLOCKS_VOLUME: f64 = 700.0;

/// The two blocks' faces on `x = y` hold four numbers each in a ratio of
/// `c′/c`: one plane to `intersect`, and not one spelling or its negation.
#[test]
fn two_turns_put_a_face_on_one_plane_with_coefficients_an_ulp_apart_in_scale() {
    let mut model = a_model();
    let (_, [left, right]) = the_ball_and_two_blocks_on_one_side(&mut model, true);
    let [shorter, longer] = [left, right].map(|block| the_face_through_the_origin(&model, block));
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

/// **A ball cut in a whole great circle, the plane spelled one way by its skin
/// and the other by the face along it.** The ball is turned about `x` so that
/// neither pole is on `x = y`, and nothing but the plane cuts the circle: every
/// other plane of the two blocks is at least six from the centre. The near
/// block, spelling the plane with `c`, holds the whole circle on its face; the
/// far block, spelling it with `c′`, touches nothing, but nothing proves it
/// apart from the near one, so its planes cut the ball's skin too and the skin
/// carried the circle as `c′` in both orders of the blocks. Before the closed
/// edge's name folded the plane at any scale, a print of it showed the skin's
/// side and the flat face's side named apart in the bits of `c` and `c′`, and
/// all twelve fuses were refused `ResultIsNotManifold { count: 1 }`.
///
/// The ball's half inside the near block is `250π/3` with a skin of `50π`, and
/// so is the half outside.
#[test]
fn a_ball_cut_whole_by_a_plane_two_blocks_spell_an_ulp_apart_in_scale() {
    every_fuse_with_the_two_blocks_answers(&CutWhole {
        made: a_ball_with_its_poles_off_the_plane,
        whole: 4.0 * PI * RADIUS * RADIUS * RADIUS / 3.0,
        in_the_near_block: 2.0 * PI * RADIUS * RADIUS * RADIUS / 3.0,
        wall_inside: 2.0 * PI * RADIUS * RADIUS,
        wall_outside: 2.0 * PI * RADIUS * RADIUS,
        pieces_outside: 1,
    });
}

/// **A bar crossing the plane square to it, cut in a whole circle the same
/// way.** The bar is of radius two, from eight on one side of `x = y` to eight
/// on the other, and the near block's face `y = −6` cuts its wall in a second
/// circle, parallel to the first, which meets nothing either. Named either way
/// round only, all twelve fuses were refused `ResultIsNotManifold { count: 1 }`.
///
/// The bar holds `64π`, of which the near block holds the six between the two
/// circles, `24π` behind a wall of `24π`; the ten outside it are two pieces
/// behind a wall of `40π`.
#[test]
fn a_bar_cut_whole_by_a_plane_two_blocks_spell_an_ulp_apart_in_scale() {
    every_fuse_with_the_two_blocks_answers(&CutWhole {
        made: a_bar_square_to_the_plane,
        whole: 64.0 * PI,
        in_the_near_block: 24.0 * PI,
        wall_inside: 24.0 * PI,
        wall_outside: 40.0 * PI,
        pieces_outside: 2,
    });
}

/// **Two blocks overlapping, their faces on `x = y` side by side, are refused
/// one way round with nothing round about.** The near block, and a block
/// reaching back over it to `x = 5.5` before the turns, so that the two share
/// `0.5 × 6 × 20` and their faces on `x = y`, spelled with `c` and `c′`, overlap.
/// The near block first, their intersection and the near block less the other
/// are refused `ResultIsNotManifold { count: 1 }`; the other first, their union
/// is. The rest answer their closed forms. Nobody has traced it: no closed
/// crossing is in it, so the name of one is not the cause.
#[test]
fn two_blocks_overlapping_on_a_plane_spelled_an_ulp_apart_in_scale_are_refused_one_way_round() {
    let reaching_back = [[5.5, -10.0], [10.0, 0.0]];
    let (shared, its_volume) = (60.0, 900.0);
    for (near_first, rule, closed_form) in [
        (true, Rule::Intersection, None),
        (
            true,
            Rule::Union,
            Some(THE_NEAR_BLOCKS_VOLUME + its_volume - shared),
        ),
        (true, Rule::Difference, None),
        (false, Rule::Intersection, Some(shared)),
        (false, Rule::Union, None),
        (false, Rule::Difference, Some(its_volume - shared)),
    ] {
        let mut model = a_model();
        let near = block(&mut model, THE_NEAR_BLOCK, TO_THE_SHORTER_SPELLING);
        let other = block(&mut model, reaching_back, TO_THE_LONGER_SPELLING);
        let (one, other) = if near_first {
            (near, other)
        } else {
            (other, near)
        };
        let answer = model.fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        let Some(closed_form) = closed_form else {
            assert!(
                matches!(answer, Err(Error::ResultIsNotManifold { count: 1 })),
                "{rule:?}, the near block first {near_first}: {answer:?}"
            );
            continue;
        };
        let solids = answer
            .unwrap_or_else(|why| panic!("this {rule:?} is answered: {why:?}"))
            .solids()
            .to_vec();
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a fuse writes a sound model or writes nothing");
        assert_eq!(solids.len(), 1, "{rule:?}");
        let found = volume_of(&model, &solids);
        assert!(
            (found - closed_form).abs() <= 1e-9 * closed_form,
            "{rule:?}: measured {found}, closed form {closed_form}"
        );
    }
}

/// **A ball on a bar's cap, the cap's plane spelled one way by the bar and the
/// other by a block that touches neither.** The bar is of the ball's radius,
/// eight long, with the ball's centre on its bottom cap, so the ball's half on
/// the bar's side is inside the bar and the cap's rim is a great circle of the
/// ball as well as the end of the bar's wall: Q1118's dome on a shaft. The rim
/// is where three surfaces meet, and the faces along it name it by two pairs,
/// the cap's plane with the bar's wall and the plane with the ball. Both pairs
/// are spelled with the plane as each face's side was cut, and the block's face
/// on `y = z`, which cuts the ball's skin in that same circle, spells it with the
/// other of `c` and `c′`. Joining the two names only where the two spellings were
/// one or each other's negation, every fuse with the bar spelling the plane with
/// `c′` and the block with `c` was refused `ResultIsNotManifold { count: 1 }`,
/// whichever of the two is named first and whichever side the ball is on. The
/// bar spelling it with `c` and the block with `c′` answered, and so did both
/// spelling it with `c`.
///
/// The ball's half outside the bar is `250π/3` behind a skin of `50π`; the bar
/// holds `200π` behind a wall of `80π`, and the block `20 × 3.5 × 20`.
#[test]
fn a_ball_on_a_bars_cap_a_block_spells_an_ulp_apart_in_scale() {
    let c = f64::from_bits(FRAC_1_SQRT_2.to_bits() - 1);
    let half_the_ball = 2.0 * PI * RADIUS * RADIUS * RADIUS / 3.0;
    let its_skin = 2.0 * PI * RADIUS * RADIUS;
    let the_bar = PI * RADIUS * RADIUS * THE_BARS_LENGTH;
    let the_bars_wall = 2.0 * PI * RADIUS * THE_BARS_LENGTH;
    let the_block = 1400.0;
    for (bar_turns, block_turns, [the_bars_c, the_blocks_c]) in [
        (
            TO_THE_LONGER_SPELLING,
            TO_THE_SHORTER_SPELLING,
            [FRAC_1_SQRT_2, c],
        ),
        (
            TO_THE_SHORTER_SPELLING,
            TO_THE_LONGER_SPELLING,
            [c, FRAC_1_SQRT_2],
        ),
    ] {
        let mut model = a_model();
        let [bar, block] =
            the_bar_and_the_block_on_y_equals_z(&mut model, bar_turns, block_turns, true);
        assert_eq!(
            the_face_through_the_origin(&model, bar).coefficients(),
            [0.0, the_bars_c, -the_bars_c, 0.0]
        );
        assert_eq!(
            the_face_through_the_origin(&model, block).coefficients(),
            [0.0, the_blocks_c, -the_blocks_c, 0.0]
        );

        for bar_first in [true, false] {
            for ball_first in [true, false] {
                let fuse = |rule| {
                    let mut model = a_model();
                    let ball = model
                        .add_ball(RADIUS, &Isometry::identity())
                        .expect("a ball of radius five");
                    let others = the_bar_and_the_block_on_y_equals_z(
                        &mut model,
                        bar_turns,
                        block_turns,
                        bar_first,
                    );
                    let answer = fused(&mut model, ball_first, ball, others, rule);
                    (answer.len(), volume_of(&model, &answer))
                };

                let (count, volume) = fuse(Rule::Intersection);
                assert_eq!(count, 1, "only the bar holds any of the ball");
                measures(volume, half_the_ball, Wall::Outside(its_skin));

                let (count, volume) = fuse(Rule::Union);
                assert_eq!(count, 2, "the block stands apart");
                measures(
                    volume,
                    the_bar + half_the_ball + the_block,
                    Wall::Outside(the_bars_wall + its_skin),
                );

                let (count, volume) = fuse(Rule::Difference);
                if ball_first {
                    assert_eq!(count, 1, "the dome on the cap is one solid");
                    measures(volume, half_the_ball, Wall::Outside(its_skin));
                } else {
                    assert_eq!(count, 2, "the block stands apart");
                    measures(
                        volume,
                        the_bar - half_the_ball + the_block,
                        Wall::OutsideAndAHole {
                            outside: the_bars_wall,
                            hole: its_skin,
                        },
                    );
                }
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
    /// One wall of each kind, so the mesh measures under by no more than the
    /// first's share and over by no more than the second's.
    OutsideAndAHole { outside: f64, hole: f64 },
}

/// A round solid that `x = y` cuts in one whole closed curve, and what each
/// fuse of it with [`THE_NEAR_BLOCK`] and [`THE_FAR_BLOCK`] is held to.
struct CutWhole {
    made: fn(&mut Model) -> SolidKey,
    /// What the whole of it holds.
    whole: f64,
    /// What of it the near block holds.
    in_the_near_block: f64,
    /// The area of its round wall inside the near block.
    wall_inside: f64,
    /// The area of its round wall outside the near block.
    wall_outside: f64,
    /// How many solids what is left of it outside the near block is.
    pieces_outside: usize,
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

/// The plane of `solid`'s face through the origin, which every turn here is
/// about: `x = y` for a block turned about `z`, `y = z` for a solid turned about
/// `x`.
fn the_face_through_the_origin(model: &Model, solid: SolidKey) -> rcad_geom::Plane {
    model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .map(|face| face.plane())
        .find(|plane| plane.coefficients()[3] == 0.0)
        .expect("each solid turned here has a face through the turns' axis")
}

/// Fuse the round solid `round.made` builds with [`THE_NEAR_BLOCK`] and
/// [`THE_FAR_BLOCK`] under every rule, whichever block is named first and
/// whichever side the round solid is on, and hold each answer to its closed
/// form.
fn every_fuse_with_the_two_blocks_answers(round: &CutWhole) {
    let mut model = a_model();
    let [near, far] = the_near_and_the_far_block(&mut model, true);
    let c = f64::from_bits(FRAC_1_SQRT_2.to_bits() - 1);
    assert_eq!(
        the_face_through_the_origin(&model, near).coefficients(),
        [-c, c, 0.0, 0.0]
    );
    assert_eq!(
        the_face_through_the_origin(&model, far).coefficients(),
        [-FRAC_1_SQRT_2, FRAC_1_SQRT_2, 0.0, 0.0]
    );

    let both_blocks = THE_NEAR_BLOCKS_VOLUME + THE_FAR_BLOCKS_VOLUME;
    let outside = round.whole - round.in_the_near_block;
    for near_first in [true, false] {
        for round_first in [true, false] {
            let fuse = |rule| {
                let mut model = a_model();
                let solid = (round.made)(&mut model);
                let blocks = the_near_and_the_far_block(&mut model, near_first);
                let answer = fused(&mut model, round_first, solid, blocks, rule);
                (answer.len(), volume_of(&model, &answer))
            };

            let (count, volume) = fuse(Rule::Intersection);
            assert_eq!(count, 1, "only the near block holds any of it");
            measures(
                volume,
                round.in_the_near_block,
                Wall::Outside(round.wall_inside),
            );

            let (count, volume) = fuse(Rule::Union);
            assert_eq!(count, 2, "the far block stands apart");
            measures(
                volume,
                both_blocks + outside,
                Wall::Outside(round.wall_outside),
            );

            let (count, volume) = fuse(Rule::Difference);
            if round_first {
                assert_eq!(count, round.pieces_outside);
                measures(volume, outside, Wall::Outside(round.wall_outside));
            } else {
                assert_eq!(count, 2, "the far block stands apart");
                measures(
                    volume,
                    both_blocks - round.in_the_near_block,
                    Wall::AHole(round.wall_inside),
                );
            }
        }
    }
}

/// The ball of radius five about the origin, turned about `x` so that neither
/// pole is on `x = y` and the plane cuts it in a circle no pole starts.
fn a_ball_with_its_poles_off_the_plane(model: &mut Model) -> SolidKey {
    model
        .add_ball(
            RADIUS,
            &Isometry::rotation(&Axis::x(), 0.3).expect("a finite turn"),
        )
        .expect("a ball of radius five")
}

/// A bar of radius two along the normal of `x = y`, from eight on one side of
/// it to eight on the other.
fn a_bar_square_to_the_plane(model: &mut Model) -> SolidKey {
    let bar = model
        .add_round_bar(
            2.0,
            16.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -8.0)).expect("a finite placement"),
        )
        .expect("a bar of radius two");
    for (axis, angle) in [(Axis::x(), -FRAC_PI_2), (Axis::z(), FRAC_PI_4)] {
        model
            .place_solid(
                bar,
                &Isometry::rotation(&axis, angle).expect("a finite turn"),
            )
            .expect("this model made the key");
    }
    bar
}

/// [`THE_NEAR_BLOCK`] turned to spell `x = y` with `c` and [`THE_FAR_BLOCK`]
/// turned to spell it with `c′`, the near one first when `near_first`.
fn the_near_and_the_far_block(model: &mut Model, near_first: bool) -> [SolidKey; 2] {
    let near = block(model, THE_NEAR_BLOCK, TO_THE_SHORTER_SPELLING);
    let far = block(model, THE_FAR_BLOCK, TO_THE_LONGER_SPELLING);
    if near_first { [near, far] } else { [far, near] }
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
    turned(model, solid, &Axis::z(), turns);
    solid
}

/// A bar of the ball's radius standing on its bottom cap at the origin, and a
/// `20 × 3.5 × 20` block standing on the plane of that cap from six and a half
/// to ten off the bar's axis, the bar turned about `x` by `bar_turns` and the
/// block by `block_turns` to put the cap on `y = z`; the bar first when
/// `bar_first`.
///
/// Neither is slid, so each one's face on `y = z` keeps an offset of exactly
/// zero. The block is clear of the bar by a unit and a half, and of the ball
/// about the origin by as much.
fn the_bar_and_the_block_on_y_equals_z(
    model: &mut Model,
    bar_turns: [f64; 2],
    block_turns: [f64; 2],
    bar_first: bool,
) -> [SolidKey; 2] {
    let bar = model
        .add_round_bar(RADIUS, THE_BARS_LENGTH, &Isometry::identity())
        .expect("a bar of radius five");
    let rectangle =
        ProfileBuilder::new(vec![[-10.0, 6.5], [10.0, 6.5], [10.0, 10.0], [-10.0, 10.0]])
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a rectangle");
    let block = model
        .add_extrusion(&rectangle, 20.0, &Isometry::identity())
        .expect("building a box needs no decision");
    turned(model, bar, &Axis::x(), bar_turns);
    turned(model, block, &Axis::x(), block_turns);
    if bar_first {
        [bar, block]
    } else {
        [block, bar]
    }
}

/// Turn `solid` about `axis` by each of `turns` in order.
fn turned(model: &mut Model, solid: SolidKey, axis: &Axis, turns: [f64; 2]) {
    for angle in turns {
        model
            .place_solid(
                solid,
                &Isometry::rotation(axis, angle).expect("a finite turn"),
            )
            .expect("this model made the key");
    }
}

/// Fuse `round` with `others`, `round` first when `round_first`, under a
/// certified policy, and insist the model it leaves is valid.
fn fused(
    model: &mut Model,
    round_first: bool,
    round: SolidKey,
    others: [SolidKey; 2],
    rule: Rule,
) -> Vec<SolidKey> {
    let (arguments, tools) = if round_first {
        (vec![round], others.to_vec())
    } else {
        (others.to_vec(), vec![round])
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
    let (under, over) = match wall {
        Wall::Outside(area) => (area, 0.0),
        Wall::AHole(area) => (0.0, area),
        Wall::OutsideAndAHole { outside, hole } => (outside, hole),
    };
    let off = closed_form - found;
    assert!(
        (-over * DEFLECTION..=under * DEFLECTION).contains(&off),
        "measured {found}, closed form {closed_form}, {off} under it, on the wrong side or too far"
    );
}
