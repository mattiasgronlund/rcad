//! A solid whose two-surface corner is moved in its file, read back, and fused.
//! `docs/decisions.md` §294 is the record and this file is its measurement.
//!
//! A round bar's top corner has two surfaces round it, the top cap's plane and
//! the wall's cylinder, so `place_a_corner` hands it to the fuse as the model
//! stored it and asks neither surface about it (§286). `Deserialize` does not
//! call `Model::validate`, so a file can carry that corner anywhere and the
//! model still reads back. §286.4 left open what a fuse then does with it: a
//! wrong `Ok`, `TheFuseWroteABrokenAnswer`, or a panic. That answer is what
//! decides whether option (c), asking the corner at the door with `validate`'s
//! own room, is owed.
//!
//! **No partner tried here gets a wrong `Ok` or a panic out of it.** What comes
//! back depends on where the partner's surfaces are:
//!
//! - **Where the answer keeps the moved corner**, the sweep of the answer finds
//!   it and the fuse refuses with `TheFuseWroteABrokenAnswer`, carrying the
//!   very refusal `validate` gave the bar. That error's own doc says it is "a
//!   bug here rather than bad input". Here it is bad input, so the fuse blames
//!   itself for the caller's file. The model is left as it was.
//! - **Where the partner cuts the moved corner away**, the answer is the one
//!   the sound bar gives: the same corners to the last bit, the same volume, and
//!   it validates on its own. A face is its surface (§17.1), and a corner that
//!   is cut away was never more than a cached sample of it.
//! - **Where a partner's face runs through where the corner should be**, the
//!   fuse puts a new corner there and cannot tell it from the moved one. It
//!   refuses, as `Undecidable` or as `CannotTellTheCellsOfAFaceApart`, which
//!   name the arithmetic and the arrangement and not the file.
//!
//! # What each family gave
//!
//! Q224 tried three more families and Q419 pinned what it found. Each case
//! comes out one of the three ways above, or as a refusal that names
//! something other than the file.
//!
//! - **The bar at a placement**: shifted by `(7, -5, 2.5)`, turned 0.4 about
//!   `x`, and turned then shifted, each block placed the same way. The corner
//!   is moved in the stored frame, and `validate` refuses it just as it
//!   refuses the bar at no placement. Every fuse gives what the bar at no
//!   placement gives, with two differences. Turned and shifted, the fuse's
//!   refusal of the corner moved off the cap carries a distance of
//!   `0.9999999999999982` where `validate`'s carries `1.0`: the same refusal,
//!   the distance only to rounding. And turned, the sound bar cut along its
//!   seam is `Undecidable` on `side of plane`, so every corner moved there is
//!   too. The point it asks about is a box round where the bottom seam corner
//!   `(3, 0, 0)` lands, which is on the block's face: `8.5e-14` either way
//!   turned, `2.8e-13` turned and shifted. That is the sound bar's refusal,
//!   not an answer about the moved corner. Only
//!   shifted, the sound bar answers, because a corner moved by a shift that
//!   rounds nothing is handed on exact (Q509). Before Q509 it refused
//!   `side of plane` there as well.
//! - **A tolerant policy**, `Policy::tolerant(100.0)`, on the bar at no
//!   placement: every fuse as under `certified`, except the corner moved off
//!   the cap in a cut along the seam. That one refuses with
//!   `TwoCurvesTouchWithoutCrossing` where `certified` finds the order along a
//!   crossing `Undecidable`.
//! - **A ball's south pole** `(0, 0, -3)`, moved to five places. `validate`
//!   refuses four with `VertexOffItsFaceSurface`, and the one moved onto the
//!   equator at `(3, 0, 0)` with `APoleStandsWhereTheSurfaceDoesNotPinch`.
//!   **No fuse blames itself.** In six pairs, a block far away among them, the
//!   answer is the sound ball's corner for corner to the last bit, so the
//!   moved pole never reaches it. The seventh pair, a small block round the
//!   pole, refuses with `ACurvedFaceGoesRightRoundWithNoSeam` for the sound
//!   ball too. The same under both policies.
//! - **A frustum's top corner** `(3, 0, 6)`, from `cones::a_frustum(model,
//!   0.5, 2, 6)`, moved off the wall, off the cap and a quarter turn round:
//!   the bar's outcomes, and one refusal more. Moved off the cap to
//!   `(3.5, 0, 7)`, the corner sits inside a block over the top, `z` from 5
//!   to 8, and both rules tried with that block are `Undecidable` on
//!   `a point inside a piece of a face`. The sound frustum is `Undecidable` on
//!   that predicate against a block over the rim by its seam, and so is every
//!   moved corner there. The same under both policies.
//!
//! The corner is moved in the bytes because nothing in the public API writes a
//! corner off its surfaces: every builder validates what it built.

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Invalid, Model, ProfileBuilder, Rule, SolidKey};

mod cones;

const RADIUS: f64 = 3.0;
const HEIGHT: f64 = 10.0;

/// Where `add_round_bar` puts the top corner: on the seam, at the top.
const WHERE_THE_BAR_PUTS_IT: [f64; 3] = [RADIUS, 0.0, HEIGHT];

/// A millimetre out from the wall, still on the top cap's plane.
const OFF_THE_WALL: [f64; 3] = [RADIUS + 1.0, 0.0, HEIGHT];

/// A millimetre above the top cap, still on the wall's cylinder.
const OFF_THE_CAP: [f64; 3] = [RADIUS, 0.0, HEIGHT + 1.0];

/// A millimetre out from the wall and a quarter turn round it, so the corner
/// no longer sits where the seam meets the cap either.
const A_QUARTER_TURN_ROUND_AND_OFF_THE_WALL: [f64; 3] = [0.0, RADIUS + 1.0, HEIGHT];

const EVERY_MOVE: [[f64; 3]; 3] = [
    OFF_THE_WALL,
    OFF_THE_CAP,
    A_QUARTER_TURN_ROUND_AND_OFF_THE_WALL,
];

/// Where `add_ball(3, ..)` puts its south pole.
const THE_SOUTH_POLE: [f64; 3] = [0.0, 0.0, -3.0];

/// The south pole moved a millimetre out along the axis, a millimetre in,
/// onto the equator at `(3, 0, 0)` and at `(0, -3, 0)`, and off the axis in
/// the pole's own plane.
const EVERY_POLE_MOVE: [[f64; 3]; 5] = [
    [0.0, 0.0, -4.0],
    [0.0, 0.0, -2.0],
    [3.0, 0.0, 0.0],
    [0.0, -3.0, 0.0],
    [1.0, 1.0, -3.0],
];

/// Where `cones::a_frustum(model, 0.5, 2, 6)` puts its top corner: on the
/// seam, at the top, where the cone is three across.
const WHERE_THE_FRUSTUM_PUTS_IT: [f64; 3] = [3.0, 0.0, 6.0];

/// A millimetre out from the frustum's wall, still on its top cap's plane.
const THE_FRUSTUMS_CORNER_OFF_THE_WALL: [f64; 3] = [4.0, 0.0, 6.0];

/// A millimetre above the frustum's top cap, still on its cone.
const THE_FRUSTUMS_CORNER_OFF_THE_CAP: [f64; 3] = [3.5, 0.0, 7.0];

/// A millimetre out from the frustum's wall and a quarter turn round it.
const THE_FRUSTUMS_CORNER_A_QUARTER_TURN_ROUND: [f64; 3] = [0.0, 4.0, 6.0];

/// A block, as the two corners of its footprint, the height of its floor and
/// how tall it stands.
#[derive(Clone, Copy, Debug)]
struct Block {
    low: [f64; 2],
    high: [f64; 2],
    floor: f64,
    tall: f64,
}

/// Nowhere near any of the solids.
const FAR_AWAY: Block = Block {
    low: [20.0, 20.0],
    high: [22.0, 22.0],
    floor: 20.0,
    tall: 2.0,
};

/// Over the bottom two millimetres of the bar.
const OVER_THE_BOTTOM: Block = Block {
    low: [-5.0, -5.0],
    high: [5.0, 5.0],
    floor: -1.0,
    tall: 3.0,
};

/// Over the top two millimetres of the bar, and the moved corner with them.
const OVER_THE_TOP: Block = Block {
    low: [-5.0, -5.0],
    high: [5.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};

/// Over the top two millimetres of the side the seam is on, `x > 2`.
const OVER_THE_RIM_BY_THE_SEAM: Block = Block {
    low: [2.0, -5.0],
    high: [5.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};

/// Over the top two millimetres of the side away from the seam, `x < -2`.
const OVER_THE_RIM_AWAY_FROM_THE_SEAM: Block = Block {
    low: [-5.0, -5.0],
    high: [-2.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};

/// A notch in the top of the rim a quarter turn round from the seam, where
/// [`A_QUARTER_TURN_ROUND_AND_OFF_THE_WALL`] claims the corner is.
const A_NOTCH_A_QUARTER_TURN_ROUND: Block = Block {
    low: [-1.0, 2.0],
    high: [1.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};

/// The whole `y > 0` half of the bar, so one face of the block runs along the
/// seam and through where the corner should be.
const THE_HALF_PAST_THE_SEAM: Block = Block {
    low: [-5.0, 0.0],
    high: [5.0, 5.0],
    floor: -1.0,
    tall: 12.0,
};

/// Over the ball's top, `z` from 2 to 5.
const OVER_THE_BALLS_TOP: Block = Block {
    low: [-5.0, -5.0],
    high: [5.0, 5.0],
    floor: 2.0,
    tall: 3.0,
};

/// Under the ball's bottom, `z` from -5 to -2, and the south pole in it.
const UNDER_THE_BALLS_BOTTOM: Block = Block {
    low: [-5.0, -5.0],
    high: [5.0, 5.0],
    floor: -5.0,
    tall: 3.0,
};

/// Two millimetres square round the south pole, `z` from -5 to -2.
const ROUND_THE_SOUTH_POLE: Block = Block {
    low: [-1.0, -1.0],
    high: [1.0, 1.0],
    floor: -5.0,
    tall: 3.0,
};

/// The whole `x > 0` half of the ball.
const THE_BALLS_X_ABOVE_NOUGHT_HALF: Block = Block {
    low: [0.0, -5.0],
    high: [5.0, 5.0],
    floor: -5.0,
    tall: 10.0,
};

/// The whole `y > 0` half of the ball.
const THE_BALLS_Y_ABOVE_NOUGHT_HALF: Block = Block {
    low: [-5.0, 0.0],
    high: [5.0, 5.0],
    floor: -5.0,
    tall: 10.0,
};

/// Across the frustum's bottom cap, `z` from 1 to 3.
const ACROSS_THE_FRUSTUMS_BOTTOM: Block = Block {
    low: [-5.0, -5.0],
    high: [5.0, 5.0],
    floor: 1.0,
    tall: 2.0,
};

/// Over the frustum's top, `z` from 5 to 8.
const OVER_THE_FRUSTUMS_TOP: Block = Block {
    low: [-5.0, -5.0],
    high: [5.0, 5.0],
    floor: 5.0,
    tall: 3.0,
};

/// Over the frustum's rim on the side the seam is on, `x > 2`.
const OVER_THE_FRUSTUMS_RIM_BY_THE_SEAM: Block = Block {
    low: [2.0, -5.0],
    high: [5.0, 5.0],
    floor: 5.0,
    tall: 3.0,
};

/// Over the frustum's rim on the side away from the seam, `x < -1`.
const OVER_THE_FRUSTUMS_RIM_AWAY_FROM_THE_SEAM: Block = Block {
    low: [-5.0, -5.0],
    high: [-1.0, 5.0],
    floor: 5.0,
    tall: 3.0,
};

/// The whole `y > 0` half of the frustum, so one face of the block runs along
/// the seam and through where the top corner should be.
const THE_HALF_OF_THE_FRUSTUM_PAST_THE_SEAM: Block = Block {
    low: [-5.0, 0.0],
    high: [5.0, 5.0],
    floor: 1.0,
    tall: 8.0,
};

/// A solid with one corner that has two surfaces round it, as its builder
/// makes it.
#[derive(Clone, Copy, Debug)]
enum Shape {
    /// The round bar, radius three and height ten, at a placement. Its corner
    /// is the top one, on the seam.
    Bar(Isometry),
    /// A ball of radius three about the origin. Its corner is the south pole.
    Ball,
    /// `cones::a_frustum(model, 0.5, 2, 6)`. Its corner is the top one, on the
    /// seam.
    Frustum,
}

/// What one fuse of a solid read back with a moved corner comes to.
#[derive(Clone, Copy, Debug)]
enum Outcome {
    /// `TheFuseWroteABrokenAnswer`, carrying the very refusal `validate` gives
    /// the file, with the model left byte for byte as it was.
    ItBlamesItself,
    /// The answer the same fuse gives the sound solid: the same corners to the
    /// last bit, the same volume, and every solid of it validates.
    TheSoundAnswer,
    /// `Error::Undecidable`, with a record naming this predicate.
    Undecidable(&'static str),
    CannotTellTheCellsOfAFaceApart,
    TwoCurvesTouchWithoutCrossing,
    ACurvedFaceGoesRightRoundWithNoSeam,
}

#[test]
fn a_bar_whose_top_corner_is_moved_a_millimetre_in_its_file_reads_back_and_does_not_validate() {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for placement in every_placement() {
            let bar = Shape::Bar(placement);
            does_not_validate(
                bar,
                OFF_THE_WALL,
                &policy,
                &Invalid::VertexOffItsFaceSurface,
            );
            does_not_validate(
                bar,
                OFF_THE_CAP,
                &policy,
                &Invalid::VertexOffItsFacePlane { distance: 1.0 },
            );
            does_not_validate(
                bar,
                A_QUARTER_TURN_ROUND_AND_OFF_THE_WALL,
                &policy,
                &Invalid::VertexOffItsFaceSurface,
            );
            validates(bar, &policy);
        }
    }
}

/// **The error is the wrong one to hand a caller whose file is broken.**
/// `TheFuseWroteABrokenAnswer` is documented as "a bug here rather than bad
/// input", and what it carries here is the refusal `validate` gives the bar
/// before any fuse. No model is harmed: the refusal leaves it byte for byte
/// as it was, which is the fuse's "Atomic" promise, and nothing corrupt is
/// written. §294 says why this is not the gap option (c) closes.
#[test]
fn a_fuse_that_keeps_the_moved_corner_blames_itself_for_the_broken_file() {
    for (shape, policy) in [
        (Shape::Bar(Isometry::identity()), Policy::certified()),
        (Shape::Bar(the_shift()), Policy::certified()),
        (Shape::Bar(the_turn()), Policy::certified()),
        (Shape::Bar(Isometry::identity()), Policy::tolerant(100.0)),
    ] {
        for moved_to in EVERY_MOVE {
            for (partner, rule) in [
                (FAR_AWAY, Rule::Union),
                (OVER_THE_BOTTOM, Rule::Difference),
                (OVER_THE_TOP, Rule::Intersection),
            ] {
                check(
                    shape,
                    moved_to,
                    partner,
                    rule,
                    &policy,
                    Outcome::ItBlamesItself,
                );
            }
        }
    }
}

/// Turned and then shifted, the fuse's refusal of the corner moved off the cap
/// is `validate`'s to within rounding and not to the bit: the distance it
/// carries is `0.9999999999999982` where `validate` says `1.0`.
#[test]
fn a_turned_and_shifted_bar_blames_itself_for_a_corner_off_the_cap_with_the_distance_rounded() {
    let policy = Policy::certified();
    let bar = Shape::Bar(the_turn_then_the_shift());
    does_not_validate(
        bar,
        OFF_THE_CAP,
        &policy,
        &Invalid::VertexOffItsFacePlane { distance: 1.0 },
    );
    for (partner, rule) in [
        (FAR_AWAY, Rule::Union),
        (OVER_THE_BOTTOM, Rule::Difference),
        (OVER_THE_TOP, Rule::Intersection),
    ] {
        blames_itself_with(
            bar,
            OFF_THE_CAP,
            partner,
            rule,
            &policy,
            &Invalid::VertexOffItsFacePlane {
                distance: 0.999_999_999_999_998_2,
            },
        );
        for moved_to in [OFF_THE_WALL, A_QUARTER_TURN_ROUND_AND_OFF_THE_WALL] {
            check(
                bar,
                moved_to,
                partner,
                rule,
                &policy,
                Outcome::ItBlamesItself,
            );
        }
    }
}

#[test]
fn a_fuse_that_cuts_the_moved_corner_away_gives_the_answer_the_sound_bar_gives() {
    for (shape, policy) in every_way_the_bar_was_tried() {
        for moved_to in EVERY_MOVE {
            for (partner, rule) in [
                (OVER_THE_TOP, Rule::Difference),
                (OVER_THE_RIM_BY_THE_SEAM, Rule::Difference),
                (OVER_THE_RIM_AWAY_FROM_THE_SEAM, Rule::Difference),
                (OVER_THE_RIM_BY_THE_SEAM, Rule::Union),
                (OVER_THE_RIM_BY_THE_SEAM, Rule::Intersection),
                (A_NOTCH_A_QUARTER_TURN_ROUND, Rule::Difference),
            ] {
                check(
                    shape,
                    moved_to,
                    partner,
                    rule,
                    &policy,
                    Outcome::TheSoundAnswer,
                );
            }
        }
    }
}

/// A block whose face runs along the seam, through the point where the top
/// corner should be.
///
/// The fuse puts a corner of its own at `(3, 0, 10)`, where that face crosses
/// the top circle, and the moved corner is still the start of the same circle.
/// Moved off the wall it sits in the block's face as well, and the arrangement
/// cannot tell the two regions of a face apart. Moved off the cap it sits on
/// the wall above the new corner, and ordering the two along the circle's
/// crossing comes out `Undecidable` with nothing to widen. Moved a quarter turn
/// round, it is inside the half the block takes, and the answer is the sound
/// bar's. **None of the three refusals names the file**, and neither is wrong
/// about what it could not do. Shifted, bar and block alike, it is the same.
#[test]
fn a_cut_along_the_seam_refuses_a_corner_moved_in_the_cut_and_answers_one_moved_out_of_it() {
    let policy = Policy::certified();
    for placement in [Isometry::identity(), the_shift()] {
        let bar = Shape::Bar(placement);
        for rule in [Rule::Difference, Rule::Intersection] {
            let cut = |moved_to, outcome| {
                check(
                    bar,
                    moved_to,
                    THE_HALF_PAST_THE_SEAM,
                    rule,
                    &policy,
                    outcome,
                );
            };
            cut(OFF_THE_WALL, Outcome::CannotTellTheCellsOfAFaceApart);
            cut(OFF_THE_CAP, Outcome::Undecidable("order along a crossing"));
            cut(
                A_QUARTER_TURN_ROUND_AND_OFF_THE_WALL,
                Outcome::TheSoundAnswer,
            );
        }
    }
}

/// Under a tolerant policy the corner moved off the cap is not an order it
/// cannot decide but a circle that touches the seam without crossing it.
#[test]
fn under_a_tolerant_policy_a_corner_moved_off_the_cap_in_a_cut_along_the_seam_touches_without_crossing()
 {
    let policy = Policy::tolerant(100.0);
    let bar = Shape::Bar(Isometry::identity());
    for rule in [Rule::Difference, Rule::Intersection] {
        let cut = |moved_to, outcome| {
            check(
                bar,
                moved_to,
                THE_HALF_PAST_THE_SEAM,
                rule,
                &policy,
                outcome,
            );
        };
        cut(OFF_THE_WALL, Outcome::CannotTellTheCellsOfAFaceApart);
        cut(OFF_THE_CAP, Outcome::TwoCurvesTouchWithoutCrossing);
        cut(
            A_QUARTER_TURN_ROUND_AND_OFF_THE_WALL,
            Outcome::TheSoundAnswer,
        );
    }
}

/// **This is the sound bar's refusal, not an answer about a moved corner.**
/// Turned, the bar's bottom seam corner `(3, 0, 0)` lands on the block's face
/// and is handed on as a box `8.5e-14` either way round where it lands
/// (`2.8e-13` when shifted as well), so which
/// side of that face it is on comes out `Undecidable`. The sound bar refuses
/// that way, and every moved corner with it, turned alone or turned and then
/// shifted. Shifted alone it answers, which is Q509's exact corner.
#[test]
fn a_turned_sound_bar_cut_along_its_seam_cannot_tell_which_side_of_the_plane_its_corner_is() {
    let policy = Policy::certified();
    for placement in [the_turn(), the_turn_then_the_shift()] {
        let bar = Shape::Bar(placement);
        for rule in [Rule::Difference, Rule::Intersection] {
            for moved_to in [
                WHERE_THE_BAR_PUTS_IT,
                OFF_THE_WALL,
                OFF_THE_CAP,
                A_QUARTER_TURN_ROUND_AND_OFF_THE_WALL,
            ] {
                check(
                    bar,
                    moved_to,
                    THE_HALF_PAST_THE_SEAM,
                    rule,
                    &policy,
                    Outcome::Undecidable("side of plane"),
                );
            }
        }
    }
}

#[test]
fn a_ball_whose_south_pole_is_moved_in_its_file_reads_back_and_does_not_validate() {
    let off_its_surface = Invalid::VertexOffItsFaceSurface;
    let on_the_equator = Invalid::APoleStandsWhereTheSurfaceDoesNotPinch {
        at: Point::new(3.0, 0.0, 0.0),
        pinch: Some(Point::new(0.0, 0.0, 3.0)),
    };
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for (moved_to, refusal) in EVERY_POLE_MOVE.into_iter().zip([
            &off_its_surface,
            &off_its_surface,
            &on_the_equator,
            &off_its_surface,
            &off_its_surface,
        ]) {
            does_not_validate(Shape::Ball, moved_to, &policy, refusal);
        }
        validates(Shape::Ball, &policy);
    }
}

/// **A moved pole never reaches the answer.** Even with a block far away,
/// where the whole ball is kept, the answer's corners are the sound ball's to
/// the last bit. The one pair refused is refused for the sound ball too.
#[test]
fn a_ball_whose_south_pole_is_moved_fuses_to_the_sound_balls_answer() {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for (partner, rule, outcome) in [
            (FAR_AWAY, Rule::Union, Outcome::TheSoundAnswer),
            (
                OVER_THE_BALLS_TOP,
                Rule::Difference,
                Outcome::TheSoundAnswer,
            ),
            (
                UNDER_THE_BALLS_BOTTOM,
                Rule::Difference,
                Outcome::TheSoundAnswer,
            ),
            (
                UNDER_THE_BALLS_BOTTOM,
                Rule::Intersection,
                Outcome::TheSoundAnswer,
            ),
            (
                ROUND_THE_SOUTH_POLE,
                Rule::Difference,
                Outcome::ACurvedFaceGoesRightRoundWithNoSeam,
            ),
            (
                THE_BALLS_X_ABOVE_NOUGHT_HALF,
                Rule::Difference,
                Outcome::TheSoundAnswer,
            ),
            (
                THE_BALLS_Y_ABOVE_NOUGHT_HALF,
                Rule::Difference,
                Outcome::TheSoundAnswer,
            ),
        ] {
            check(Shape::Ball, THE_SOUTH_POLE, partner, rule, &policy, outcome);
            for moved_to in EVERY_POLE_MOVE {
                check(Shape::Ball, moved_to, partner, rule, &policy, outcome);
            }
        }
    }
}

#[test]
fn a_frustum_whose_top_corner_is_moved_in_its_file_reads_back_and_does_not_validate() {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        let frustum = Shape::Frustum;
        does_not_validate(
            frustum,
            THE_FRUSTUMS_CORNER_OFF_THE_WALL,
            &policy,
            &Invalid::VertexOffItsFaceSurface,
        );
        does_not_validate(
            frustum,
            THE_FRUSTUMS_CORNER_OFF_THE_CAP,
            &policy,
            &Invalid::VertexOffItsFacePlane { distance: 1.0 },
        );
        does_not_validate(
            frustum,
            THE_FRUSTUMS_CORNER_A_QUARTER_TURN_ROUND,
            &policy,
            &Invalid::VertexOffItsFaceSurface,
        );
        validates(frustum, &policy);
    }
}

/// Each row is a partner, a rule, what the sound frustum gives, and what the
/// corner moved off the wall, off the cap and a quarter turn round gives.
#[test]
fn a_frustum_whose_top_corner_is_moved_gives_what_the_bar_gives_and_one_refusal_more() {
    use Outcome::{
        CannotTellTheCellsOfAFaceApart as CannotTell, ItBlamesItself as Blamed,
        TheSoundAnswer as Sound,
    };
    let inside_a_piece = Outcome::Undecidable("a point inside a piece of a face");
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for (partner, rule, sound, [off_the_wall, off_the_cap, a_quarter_turn_round]) in [
            (FAR_AWAY, Rule::Union, Sound, [Blamed, Blamed, Blamed]),
            (
                ACROSS_THE_FRUSTUMS_BOTTOM,
                Rule::Difference,
                Sound,
                [Blamed, Blamed, Blamed],
            ),
            (
                OVER_THE_FRUSTUMS_TOP,
                Rule::Difference,
                Sound,
                [Sound, inside_a_piece, Sound],
            ),
            (
                OVER_THE_FRUSTUMS_TOP,
                Rule::Intersection,
                Sound,
                [Blamed, inside_a_piece, Blamed],
            ),
            (
                OVER_THE_FRUSTUMS_RIM_BY_THE_SEAM,
                Rule::Difference,
                inside_a_piece,
                [inside_a_piece, inside_a_piece, inside_a_piece],
            ),
            (
                OVER_THE_FRUSTUMS_RIM_AWAY_FROM_THE_SEAM,
                Rule::Difference,
                Sound,
                [Sound, Sound, Sound],
            ),
            (
                THE_HALF_OF_THE_FRUSTUM_PAST_THE_SEAM,
                Rule::Difference,
                Sound,
                [CannotTell, CannotTell, Sound],
            ),
            (
                THE_HALF_OF_THE_FRUSTUM_PAST_THE_SEAM,
                Rule::Intersection,
                Sound,
                [CannotTell, CannotTell, Sound],
            ),
        ] {
            let frustum = |moved_to, outcome| {
                check(Shape::Frustum, moved_to, partner, rule, &policy, outcome);
            };
            frustum(WHERE_THE_FRUSTUM_PUTS_IT, sound);
            frustum(THE_FRUSTUMS_CORNER_OFF_THE_WALL, off_the_wall);
            frustum(THE_FRUSTUMS_CORNER_OFF_THE_CAP, off_the_cap);
            frustum(
                THE_FRUSTUMS_CORNER_A_QUARTER_TURN_ROUND,
                a_quarter_turn_round,
            );
        }
    }
}

/// No placement at all, the shift, the turn, and the turn then the shift.
fn every_placement() -> [Isometry; 4] {
    [
        Isometry::identity(),
        the_shift(),
        the_turn(),
        the_turn_then_the_shift(),
    ]
}

/// The bar at every placement under `certified`, and at none under
/// `tolerant(100)`.
fn every_way_the_bar_was_tried() -> [(Shape, Policy); 5] {
    let [none, shift, turn, turn_then_shift] = every_placement().map(Shape::Bar);
    [
        (none, Policy::certified()),
        (shift, Policy::certified()),
        (turn, Policy::certified()),
        (turn_then_shift, Policy::certified()),
        (none, Policy::tolerant(100.0)),
    ]
}

/// A shift that rounds nothing: whole numbers and a half.
fn the_shift() -> Isometry {
    Isometry::translation(Vector::new(7.0, -5.0, 2.5)).expect("a plain translation")
}

/// A turn of 0.4 about `x`, which rounds.
fn the_turn() -> Isometry {
    Isometry::rotation(&Axis::x(), 0.4).expect("a turn about x")
}

fn the_turn_then_the_shift() -> Isometry {
    the_shift().compose(&the_turn())
}

fn does_not_validate(shape: Shape, moved_to: [f64; 3], policy: &Policy, refusal: &Invalid) {
    let (model, _) = read_back(shape, moved_to);
    match model.validate(policy, &mut ProvenanceLog::new()) {
        Err(Error::Invalid(found)) => {
            assert_eq!(&found, refusal, "{shape:?} with its corner at {moved_to:?}");
        }
        other => panic!("{shape:?} with its corner at {moved_to:?} validates as {other:?}"),
    }
}

fn validates(shape: Shape, policy: &Policy) {
    let (sound, _) = read_back(shape, shape.corner());
    sound
        .validate(policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|error| {
            panic!("the same bytes with the corner left alone are a sound {shape:?}: {error:?}")
        });
}

/// Fuse `shape`, read back with its corner at `moved_to`, with `partner`
/// placed where the shape is, and check the fuse comes to `outcome`.
fn check(
    shape: Shape,
    moved_to: [f64; 3],
    partner: Block,
    rule: Rule,
    policy: &Policy,
    outcome: Outcome,
) {
    match outcome {
        Outcome::ItBlamesItself => {
            let (model, _) = read_back(shape, moved_to);
            let Err(Error::Invalid(what_validate_says)) =
                model.validate(policy, &mut ProvenanceLog::new())
            else {
                panic!("{shape:?} with its corner at {moved_to:?} is broken");
            };
            blames_itself_with(shape, moved_to, partner, rule, policy, &what_validate_says);
        }
        Outcome::TheSoundAnswer => {
            the_same_answer_as_the_sound_solid(shape, moved_to, partner, rule, policy);
        }
        Outcome::Undecidable(_)
        | Outcome::CannotTellTheCellsOfAFaceApart
        | Outcome::TwoCurvesTouchWithoutCrossing
        | Outcome::ACurvedFaceGoesRightRoundWithNoSeam => {
            refuses_as(shape, moved_to, partner, rule, policy, outcome);
        }
    }
}

fn refuses_as(
    shape: Shape,
    moved_to: [f64; 3],
    partner: Block,
    rule: Rule,
    policy: &Policy,
    refusal: Outcome,
) {
    let case = format!("{shape:?} with its corner at {moved_to:?}, {partner:?} {rule:?}");
    let refused = match fuse(shape, moved_to, partner, rule, policy) {
        Ok(answer) => panic!(
            "{case} answered with {} solids where {refusal:?} was expected",
            answer.solids.len()
        ),
        Err(error) => error,
    };
    let as_expected = match refusal {
        Outcome::Undecidable(predicate) => matches!(
            &refused,
            Error::Undecidable { records }
                if records.records().iter().any(|record| record.predicate() == predicate)
        ),
        Outcome::CannotTellTheCellsOfAFaceApart => {
            matches!(refused, Error::CannotTellTheCellsOfAFaceApart { .. })
        }
        Outcome::TwoCurvesTouchWithoutCrossing => {
            matches!(refused, Error::TwoCurvesTouchWithoutCrossing { .. })
        }
        Outcome::ACurvedFaceGoesRightRoundWithNoSeam => {
            matches!(refused, Error::ACurvedFaceGoesRightRoundWithNoSeam { .. })
        }
        Outcome::ItBlamesItself | Outcome::TheSoundAnswer => {
            unreachable!("{refusal:?} is not a refusal")
        }
    };
    assert!(
        as_expected,
        "{case}: expected {refusal:?}, refused with {refused:?}"
    );
}

/// The fuse refuses with `TheFuseWroteABrokenAnswer` carrying `broken`, and
/// leaves the model as it was.
fn blames_itself_with(
    shape: Shape,
    moved_to: [f64; 3],
    partner: Block,
    rule: Rule,
    policy: &Policy,
    broken: &Invalid,
) {
    let (mut model, solid) = read_back(shape, moved_to);
    let block = a_block(&mut model, partner, &shape.placement());
    let before = postcard::to_allocvec(&model).expect("a model writes out");
    match model.fuse(&[solid], &[block], rule, policy, &mut ProvenanceLog::new()) {
        Err(Error::TheFuseWroteABrokenAnswer { broken: found }) => assert_eq!(
            &found, broken,
            "{shape:?} with its corner at {moved_to:?}, {partner:?} {rule:?}: the answer is \
             refused for the solid's own fault"
        ),
        other => panic!(
            "{shape:?} with its corner at {moved_to:?}, {partner:?} {rule:?} came back {other:?}"
        ),
    }
    assert_eq!(
        postcard::to_allocvec(&model).expect("a model writes out"),
        before,
        "a refused fuse leaves the model as it was"
    );
}

/// Fuse the solid with its corner moved, and the same solid with its corner
/// left alone, against the same block, and check the two answers are one.
///
/// "The same" is the same corners to the last bit, taken as a set because the
/// fuse may start a cut circle at a different corner, and the same volume to
/// within what that different start does to the mesh the volume is summed over.
fn the_same_answer_as_the_sound_solid(
    shape: Shape,
    moved_to: [f64; 3],
    partner: Block,
    rule: Rule,
    policy: &Policy,
) {
    let case = format!("{shape:?} with its corner at {moved_to:?}, {partner:?} {rule:?}");
    let broken = fuse(shape, moved_to, partner, rule, policy)
        .unwrap_or_else(|error| panic!("{case} refused: {error:?}"));
    let sound = fuse(shape, shape.corner(), partner, rule, policy)
        .unwrap_or_else(|error| panic!("{case}: the sound solid refused: {error:?}"));
    assert_eq!(
        broken.solids.len(),
        sound.solids.len(),
        "{case}: how many solids"
    );
    assert_eq!(
        broken.corners, sound.corners,
        "{case}: the corners of the answer"
    );
    assert!(
        (broken.volume - sound.volume).abs() <= 1e-12 * sound.volume,
        "{case}: volume {} against {}",
        broken.volume,
        sound.volume
    );
    for solid in broken.solids {
        broken
            .model
            .validate_solid(solid, policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|error| panic!("{case}: the answer on its own is sound: {error:?}"));
    }
}

struct Answer {
    model: Model,
    solids: Vec<SolidKey>,
    corners: Vec<[u64; 3]>,
    volume: f64,
}

/// The solids fusing `shape` with its corner at `moved_to` and `partner`
/// gives, their corners as bits, sorted, and their volume.
fn fuse(
    shape: Shape,
    moved_to: [f64; 3],
    partner: Block,
    rule: Rule,
    policy: &Policy,
) -> Result<Answer, Error> {
    let mut log = ProvenanceLog::new();
    let (mut model, solid) = read_back(shape, moved_to);
    let block = a_block(&mut model, partner, &shape.placement());
    let solids = model
        .fuse(&[solid], &[block], rule, policy, &mut log)?
        .solids()
        .to_vec();
    let mut corners = Vec::new();
    let mut volume = 0.0;
    for &solid in &solids {
        let view = model.solid(solid).expect("the fuse made this key");
        corners.extend(
            view.faces()
                .flat_map(|face| face.wires())
                .flat_map(|wire| wire.coedges())
                .map(|coedge| coedge.start().point().coordinates().map(f64::to_bits)),
        );
        volume += view
            .measure(Deflection::for_scale(model.scale()), policy, &mut log)
            .expect("the answer measures")
            .volume();
    }
    corners.sort_unstable();
    corners.dedup();
    Ok(Answer {
        model,
        solids,
        corners,
        volume,
    })
}

/// `shape`, written out with postcard, its corner overwritten in the bytes
/// with `moved_to`, and read back.
///
/// The corner is the only place its three numbers sit side by side in the
/// file. That is asserted, so the overwrite cannot land on a surface.
fn read_back(shape: Shape, moved_to: [f64; 3]) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    shape.build(&mut model);
    let mut bytes = postcard::to_allocvec(&model).expect("a model writes out");

    let stored = the_bytes_of(shape.corner());
    let places: Vec<usize> = bytes
        .windows(stored.len())
        .enumerate()
        .filter(|(_, window)| *window == stored.as_slice())
        .map(|(at, _)| at)
        .collect();
    let [at] = places.as_slice() else {
        panic!("{shape:?}'s corner is written once, and was found at {places:?}");
    };
    bytes[*at..*at + stored.len()].copy_from_slice(&the_bytes_of(moved_to));

    let read_back: Model =
        postcard::from_bytes(&bytes).expect("reading back does not ask where a corner is");
    let solid = read_back
        .solids()
        .next()
        .expect("the solid came back")
        .key();
    (read_back, solid)
}

/// A point the way postcard writes one: three `f64`s, little-endian.
fn the_bytes_of(point: [f64; 3]) -> Vec<u8> {
    point
        .iter()
        .flat_map(|coordinate| coordinate.to_le_bytes())
        .collect()
}

impl Shape {
    fn build(self, model: &mut Model) {
        match self {
            Shape::Bar(placement) => {
                model
                    .add_round_bar(RADIUS, HEIGHT, &placement)
                    .expect("a bar of radius three and height ten");
            }
            Shape::Ball => {
                model
                    .add_ball(3.0, &Isometry::identity())
                    .expect("a ball of radius three");
            }
            Shape::Frustum => {
                cones::a_frustum(model, 0.5, 2.0, 6.0);
            }
        }
    }

    /// Where the builder puts the corner the file moves, in the stored frame.
    fn corner(self) -> [f64; 3] {
        match self {
            Shape::Bar(_) => WHERE_THE_BAR_PUTS_IT,
            Shape::Ball => THE_SOUTH_POLE,
            Shape::Frustum => WHERE_THE_FRUSTUM_PUTS_IT,
        }
    }

    /// Where the solid is placed, and so where its partner is placed too.
    fn placement(self) -> Isometry {
        match self {
            Shape::Bar(placement) => placement,
            Shape::Ball | Shape::Frustum => Isometry::identity(),
        }
    }
}

/// `block`, placed by `placement` after its own shift to its floor.
fn a_block(model: &mut Model, block: Block, placement: &Isometry) -> SolidKey {
    let [wide, deep] = [block.high[0] - block.low[0], block.high[1] - block.low[1]];
    let footprint = ProfileBuilder::new(vec![[0.0, 0.0], [wide, 0.0], [wide, deep], [0.0, deep]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let own = Isometry::translation(Vector::new(block.low[0], block.low[1], block.floor))
        .expect("a plain translation");
    model
        .add_extrusion(&footprint, block.tall, &placement.compose(&own))
        .expect("a block")
}
