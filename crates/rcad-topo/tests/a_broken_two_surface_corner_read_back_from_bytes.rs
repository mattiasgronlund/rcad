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
//!   very refusal `validate` gave the bar. That error's own doc says it does
//!   not say whether the fault is this kernel's or the input's. Here it is
//!   the input's, so the fuse blames itself for the caller's file. The model
//!   is left as it was.
//! - **Where the partner cuts the moved corner away**, the answer is the one
//!   the sound bar gives: the same corners to the last bit, the same volume, and
//!   it validates on its own. A face is its surface (§17.1), and a corner that
//!   is cut away was never more than a cached sample of it.
//! - **Where a partner's face runs through where the corner should be**, the
//!   fuse puts a new corner there and cannot tell it from the moved one. It
//!   refuses, as `Undecidable`, as `CannotTellTheCellsOfAFaceApart` or as
//!   `ResultIsNotManifold`, which name the arithmetic and the arrangement and
//!   not the file.
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
//!   placement gives, with three differences. Turned and shifted, the fuse's
//!   refusal of the corner moved off the cap carries a distance of
//!   `0.9999999999999982` where `validate`'s carries `1.0`: the same refusal,
//!   the distance only to rounding. Turned, the sound bar cut along its seam
//!   answers, because the block's face is the bar's seam plane turned to the
//!   bit and the corner is taken as on it by construction (§294.9); the corner
//!   moved a quarter turn round then gives the sound answer to a rounding
//!   rather than to the bit. Turned and shifted, the same cut answers the
//!   same way, and the corner moved off the wall refuses `ResultIsNotManifold`
//!   rather than `CannotTellTheCellsOfAFaceApart`. There the block's face is
//!   an ulp off the bar's seam plane moved, with one normal to the bit, so the
//!   bottom seam corner `(3, 0, 0)`'s side of it is which offset is the larger
//!   (Q2374). Until Q2374 the fuse asked a box `2.8e-13` either way round
//!   where that corner lands instead, and the sound bar and every moved
//!   corner refused `side of plane`. Shifted alone, the sound bar answers,
//!   because a corner moved by a shift that rounds nothing is handed on exact
//!   (Q509). Before Q509 it refused `side of plane` there as well.
//! - **A tolerant policy**, `Policy::tolerant(100.0)`, on the bar at no
//!   placement: every fuse as under `certified`, except the corner moved off
//!   the cap in a cut along the seam. That one refuses with
//!   `TwoCurvesTouchWithoutCrossing` where `certified` finds the order along a
//!   crossing `Undecidable`.
//! - **A ball's south pole** `(0, 0, -3)`, moved to five places. `validate`
//!   refuses four with `VertexOffItsFaceSurface`, and the one moved onto the
//!   equator at `(3, 0, 0)` with `APoleStandsWhereTheSurfaceDoesNotPinch`.
//!   **No fuse blames itself.** In all seven pairs, a block far away and a
//!   small block round the pole among them, the answer is the sound ball's
//!   corner for corner to the last bit, so the moved pole never reaches it.
//!   The same under both policies. Until Q508 the small block round the pole
//!   refused with `ACurvedFaceGoesRightRoundWithNoSeam` for the sound ball
//!   too, because the mesher could not tell which pole the cap the ball keeps
//!   reaches.
//! - **A frustum's top corner** `(3, 0, 6)`, from `cones::a_frustum(model,
//!   0.5, 2, 6)`, moved off the wall, off the cap and a quarter turn round:
//!   the bar's outcomes. Moved off the cap to `(3.5, 0, 7)`, the corner sits
//!   inside a block over the top, `z` from 5 to 8; the frustum less that block
//!   is the sound frustum's answer, and the two together blame themselves with
//!   `validate`'s own refusal. Against a block over the rim by the seam, the
//!   sound frustum and every moved corner give the sound frustum's answer. The
//!   same under both policies. All of these were `Undecidable` on `a point
//!   inside a piece of a face` until a piece no point was found in was placed
//!   by a side it shares (§365.3, Q712); the sound frustum's two answers that
//!   moved measure what the frustum and the block say.
//!
//!   **The seam was never what refused the block over the rim by the seam.**
//!   That block's wall at `x = 2` cuts the cone below its floor, down to
//!   `(2, 0, 4)`, and leaves a piece of the cone between that cut and the
//!   circle at `z = 5` that no step of one radius from its two corners
//!   reached. The seam turned to `−x` or `+y`, or the block put over the far
//!   rim at `x = −2`, leaves the same piece. The block over the rim away from
//!   the seam stops at `x = −1`, where its wall's cut reaches down to the
//!   bottom rim at `(−1, 0, 2)`, and a point the fuse offered was inside every
//!   piece that cut left, so it answered. Since Q929 the fuse takes the same
//!   steps shorter as well, and one of them is inside the piece
//!   (`a_frustums_wall_cut_below_a_blocks_floor.rs`).
//!
//!   **Turned 0.4 about `x`, the frustum cut along its seam answers under both
//!   policies, as the bar does** (§294.12, Q1361). The block's face along the
//!   seam runs through the cone's apex, so the two cross in two rulings
//!   through it, along directions no `f64`s write. Until Q1171
//!   `rcad_geom::intersect` refused them, `CannotNameWhichPieceOfACrossingThisIs`,
//!   under both policies (Q1044, §294.10); it names the four halves the apex
//!   cuts them into now (§294.11). Until Q1164 the fuse then stopped at `side
//!   of plane` at the bottom seam corner's box, the bar's refusal before
//!   §294.9; a cone's seam corner carries its seam plane now, as a pipe's does.
//!   Until Q1361 `certified` then stopped at the turn between the two halves
//!   that leave the apex into the frustum's nappe, which is nothing and was
//!   asked as two enclosures; two rulings of one plane holding the axis are
//!   read as turning by nothing now. The sound frustum gives half the frustum,
//!   `26π/3`, and the corners moved off the wall and off the cap give their
//!   unturned outcomes.
//!
//!   **Turned and then shifted, the block's face is one ulp off the moved
//!   seam plane, as the bar's is, though the other way** (§294.13). Since
//!   Q2374 the bottom seam corner's side of it is read off the two offsets, as
//!   the bar's is, and under `certified` the sound frustum and the corner
//!   moved a quarter turn round refuse `side of a half-space` at the box round
//!   where the apex lands instead; until then every corner refused `side of
//!   plane` at the bottom seam corner. Under `tolerant(100)` it answers, as
//!   the bar does: half the frustum, `26π/3`, from the sound frustum and the
//!   corner moved a quarter turn round. Under both policies the corners moved
//!   off the wall and off the cap refuse `ResultIsNotManifold`. Until Q1405 the sound frustum refused `CannotNameWhichPieceOfACrossingThisIs`
//!   there: meshing its answer, the walk along the cut asked a hyperbola's
//!   vertex a rounding from the cone's apex which side of the apex it is on, to
//!   pick a branch the ends already pick (§294.14). The ends pick it now
//!   (§294.15).
//!
//! The corner is moved in the bytes because nothing in the public API writes a
//! corner off its surfaces: every builder validates what it built.

use std::f64::consts::PI;

use rcad_geom::{
    Axis, Cone, Cylinder, Deflection, Isometry, NamedRun, Plane, Point, Scale, Surface,
    SurfaceMeet, Vector, how_a_plane_leans_against_a_cone, how_a_plane_leans_against_a_pipe,
    intersect, walk, where_a_cones_apex_stands_against_a_plane,
};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
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

/// The whole `y < 0` half of the bar, so one face of the block runs along the
/// seam the other way round from [`THE_HALF_PAST_THE_SEAM`]'s.
const THE_HALF_SHORT_OF_THE_SEAM: Block = Block {
    low: [-5.0, -5.0],
    high: [5.0, 0.0],
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
    /// `cones::a_frustum(model, 0.5, 2, 6)`, at a placement. Its corner is the
    /// top one, on the seam.
    Frustum(Isometry),
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
    ResultIsNotManifold,
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
/// `TheFuseWroteABrokenAnswer` is documented as not saying whether the fault
/// is this kernel's or the input's, and what it carries here is the refusal
/// `validate` gives the bar before any fuse. No model is harmed: the refusal
/// leaves it byte for byte as it was, which is the fuse's "Atomic" promise,
/// and nothing corrupt is written. §294 says why this is not the gap option
/// (c) closes.
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
/// Turned it is the same but for the last bit or two of a corner
/// (`a_turned_bar_cut_along_its_seam_gives_what_the_unturned_bar_gives_turned`).
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

/// Turned 0.4 about `x`, the bar cut along its seam gives what the unturned bar
/// gives, turned.
///
/// The sound bar less the half past its seam, and with it, is half the bar,
/// `45π`, and its corners are the unturned answer's corners turned. Its bottom
/// seam corner `(3, 0, 0)` lands on the block's face, handed on as a box
/// `8.5e-14` either way, and the face is the bar's seam plane moved the same way
/// to the bit. Asked which side of it the box is, the fuse refused `side of
/// plane`; the corner is on that plane by construction, and is taken so
/// (§294.9).
///
/// The top corner moved in the file then gives the unturned bar's outcomes:
/// off the wall and off the cap the same refusals, and a quarter turn round
/// the sound answer — but the sound answer to a rounding, not to the bit. Cut
/// away, the moved corner leaves the top seam corner of the answer to be
/// placed where the block's face crosses the top circle, two ulps from where
/// the sound bar's own corner is written.
#[test]
fn a_turned_bar_cut_along_its_seam_gives_what_the_unturned_bar_gives_turned() {
    let policy = Policy::certified();
    let half_the_bar = PI * RADIUS * RADIUS * HEIGHT / 2.0;
    let turned = Shape::Bar(the_turn());
    for rule in [Rule::Difference, Rule::Intersection] {
        let answer_of = |shape: Shape, moved_to, partner| {
            fuse(shape, moved_to, partner, rule, &policy).unwrap_or_else(|why| {
                panic!(
                    "{shape:?} with its corner at {moved_to:?}, {partner:?} {rule:?}, refused: \
                     {why:?}"
                )
            })
        };
        for partner in [THE_HALF_PAST_THE_SEAM, THE_HALF_SHORT_OF_THE_SEAM] {
            let sound = answer_of(turned, WHERE_THE_BAR_PUTS_IT, partner);
            let unturned = answer_of(
                Shape::Bar(Isometry::identity()),
                WHERE_THE_BAR_PUTS_IT,
                partner,
            );
            holds(&sound, half_the_bar, rule, &policy);
            holds(&unturned, half_the_bar, rule, &policy);
            the_same_corners_to_a_rounding(&sound, &unturned, &the_turn(), rule);
        }

        let sound = answer_of(turned, WHERE_THE_BAR_PUTS_IT, THE_HALF_PAST_THE_SEAM);
        let cut_away = answer_of(
            turned,
            A_QUARTER_TURN_ROUND_AND_OFF_THE_WALL,
            THE_HALF_PAST_THE_SEAM,
        );
        holds(&cut_away, half_the_bar, rule, &policy);
        the_same_corners_to_a_rounding(&cut_away, &sound, &Isometry::identity(), rule);

        check(
            turned,
            OFF_THE_WALL,
            THE_HALF_PAST_THE_SEAM,
            rule,
            &policy,
            Outcome::CannotTellTheCellsOfAFaceApart,
        );
        check(
            turned,
            OFF_THE_CAP,
            THE_HALF_PAST_THE_SEAM,
            rule,
            &policy,
            Outcome::Undecidable("order along a crossing"),
        );
    }
}

/// **Turned and then shifted, the bar cut along its seam gives what the turned
/// bar gives, shifted** (Q2374). The bar's bottom seam corner `(3, 0, 0)`
/// lands on the block's face give or take a rounding and is handed on as a box
/// `2.8e-13` either way round where it lands. The face is not the bar's seam
/// plane moved: the shift rounds the two offsets apart by an ulp. But the two
/// have one normal to the bit, and the corner is on its seam plane by
/// construction (§294.9), so which side of the face it is on is which of the
/// two offsets is the larger. Until Q2374 the fuse asked the box instead, and
/// the sound bar and every moved corner refused `side of plane`.
///
/// The sound bar less the half past its seam, and with it, is half the bar,
/// `45π`, and so it is less and with the half short of its seam; its corners
/// are the turned bar's shifted, to a rounding. The corner moved a quarter turn
/// round is cut away and gives the sound answer to a rounding. Moved off the
/// cap it refuses the order along a crossing, as turned alone; moved off the
/// wall it refuses `ResultIsNotManifold`, where turned alone it refuses
/// `CannotTellTheCellsOfAFaceApart`
/// (`a_turned_bar_cut_along_its_seam_gives_what_the_unturned_bar_gives_turned`).
#[test]
fn a_turned_and_shifted_bar_cut_along_its_seam_gives_what_the_turned_bar_gives_shifted() {
    let policy = Policy::certified();
    let half_the_bar = PI * RADIUS * RADIUS * HEIGHT / 2.0;
    let bar = Shape::Bar(the_turn_then_the_shift());
    for rule in [Rule::Difference, Rule::Intersection] {
        let answer_of = |shape: Shape, moved_to, partner| {
            fuse(shape, moved_to, partner, rule, &policy).unwrap_or_else(|why| {
                panic!(
                    "{shape:?} with its corner at {moved_to:?}, {partner:?} {rule:?}, refused: \
                     {why:?}"
                )
            })
        };
        for partner in [THE_HALF_PAST_THE_SEAM, THE_HALF_SHORT_OF_THE_SEAM] {
            let sound = answer_of(bar, WHERE_THE_BAR_PUTS_IT, partner);
            let turned = answer_of(Shape::Bar(the_turn()), WHERE_THE_BAR_PUTS_IT, partner);
            holds(&sound, half_the_bar, rule, &policy);
            the_same_corners_to_a_rounding(&sound, &turned, &the_shift(), rule);
        }

        let sound = answer_of(bar, WHERE_THE_BAR_PUTS_IT, THE_HALF_PAST_THE_SEAM);
        let cut_away = answer_of(
            bar,
            A_QUARTER_TURN_ROUND_AND_OFF_THE_WALL,
            THE_HALF_PAST_THE_SEAM,
        );
        holds(&cut_away, half_the_bar, rule, &policy);
        the_same_corners_to_a_rounding(&cut_away, &sound, &Isometry::identity(), rule);

        check(
            bar,
            OFF_THE_WALL,
            THE_HALF_PAST_THE_SEAM,
            rule,
            &policy,
            Outcome::ResultIsNotManifold,
        );
        check(
            bar,
            OFF_THE_CAP,
            THE_HALF_PAST_THE_SEAM,
            rule,
            &policy,
            Outcome::Undecidable("order along a crossing"),
        );
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
/// the last bit. So are they with a small block cut out round the pole itself.
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
                Outcome::TheSoundAnswer,
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
        let frustum = Shape::Frustum(Isometry::identity());
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
fn a_frustum_whose_top_corner_is_moved_gives_what_the_bar_gives() {
    use Outcome::{
        CannotTellTheCellsOfAFaceApart as CannotTell, ItBlamesItself as Blamed,
        TheSoundAnswer as Sound,
    };
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
                [Sound, Sound, Sound],
            ),
            (
                OVER_THE_FRUSTUMS_TOP,
                Rule::Intersection,
                Sound,
                [Blamed, Blamed, Blamed],
            ),
            (
                OVER_THE_FRUSTUMS_RIM_BY_THE_SEAM,
                Rule::Difference,
                Sound,
                [Sound, Sound, Sound],
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
                check(
                    Shape::Frustum(Isometry::identity()),
                    moved_to,
                    partner,
                    rule,
                    &policy,
                    outcome,
                );
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

/// The sound frustum less a block over its top, and less a block over its rim
/// by the seam, each hold what the frustum and the block say. The first is
/// what the corner moved off the cap gives, and the second what every corner
/// gives, so this is what `TheSoundAnswer` holds those to.
///
/// The frustum is `π/12 · (6³ − 2³) = 52π/3`, and below `z = 5` it is
/// `π/12 · (5³ − 2³) = 39π/4`. The block over the rim takes `∫₅⁶ seg(z/2) dz`, `seg(r)` being the
/// part of a circle of radius `r` past `x = 2`, `r² acos(2/r) − 2√(r² − 4)`;
/// with `F(r) = r³/3 · acos(2/r) − 4r/3 · √(r² − 4) + 8/3 · ln(r + √(r² − 4))`
/// that is `2 (F(3) − F(2.5))`.
#[test]
fn the_sound_frustum_less_a_block_over_its_top_or_its_rim_holds_what_the_two_say() {
    let f = |r: f64| {
        let leg = (r * r - 4.0).sqrt();
        r.powi(3) / 3.0 * (2.0 / r).acos() - 4.0 * r / 3.0 * leg + 8.0 / 3.0 * (r + leg).ln()
    };
    let frustum = 52.0 * PI / 3.0;
    for (partner, volume) in [
        (OVER_THE_FRUSTUMS_TOP, 39.0 * PI / 4.0),
        (
            OVER_THE_FRUSTUMS_RIM_BY_THE_SEAM,
            frustum - 2.0 * (f(3.0) - f(2.5)),
        ),
    ] {
        for policy in [Policy::certified(), Policy::tolerant(100.0)] {
            let (mut model, solid) = read_back(
                Shape::Frustum(Isometry::identity()),
                WHERE_THE_FRUSTUM_PUTS_IT,
            );
            let block = a_block(&mut model, partner, &Isometry::identity());
            let what = format!("{partner:?} under {policy:?}");
            let answer = model
                .fuse(
                    &[solid],
                    &[block],
                    Rule::Difference,
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what} refused: {why:?}"));
            let [solid] = answer.solids() else {
                panic!("{what}: {:?}", answer.solids());
            };
            model
                .validate(&policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what} does not validate: {why:?}"));
            let measured = model
                .solid(*solid)
                .expect("the fuse made this key")
                .measure(
                    Deflection::of(0.001).expect("a fine mesh"),
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what} does not measure: {why:?}"));
            assert!(
                (measured.volume() - volume).abs() <= measured.volume_error(),
                "{what} holds {volume} and measures {} give or take {}",
                measured.volume(),
                measured.volume_error()
            );
        }
    }
}

/// **Turned 0.4 about `x`, the frustum cut along its seam gives what the
/// unturned frustum gives, turned, under both policies.** Until Q1171 every
/// fuse here refused `CannotNameWhichPieceOfACrossingThisIs` where the block's
/// face crosses the cone; `intersect` names the four half rulings now
/// (`the_face_along_a_turned_frustums_seam_crosses_its_cone_in_four_half_rulings`).
/// Until Q1164 the fuse then refused `side of plane` at the box round the
/// bottom seam corner `(1, 0, 2)` turned, the round bar's refusal before
/// §294.9. A seam corner on a cone carries its seam plane now, as one on a
/// pipe does, and the block's face is that plane turned, so the corner is
/// taken as on it unasked (§294.12).
///
/// **Until Q1361 `certified` then refused at the apex `(0, 0, 0)`**, on
/// `which way round a node two ways out of it turn`. On the frustum's wall,
/// the two half rulings that leave the apex into the frustum's nappe both lie
/// in the block's face, which holds the axis, so about the axis they are half
/// a turn apart and the turn between them is nothing. Each is an enclosure, so
/// worked out the turn straddles nothing. It is read as nothing now, because
/// the face holds the axis exactly as stored
/// (`a_plane_through_a_frustums_apex_that_misses_its_axis_turns_its_two_rulings_by_something`
/// is the plane that does not).
///
/// The sound frustum answers both rules with half the frustum, `26π/3`,
/// measured on a fine mesh, validates, and has the unturned answer's corners
/// turned to a rounding. The corner moved a quarter turn round is cut away and
/// gives the sound answer to a rounding, not to the bit, since the top seam
/// corner is then placed where the block's face crosses the top circle: the
/// turned bar's one difference. The corners moved off the wall and off the
/// cap give `CannotTellTheCellsOfAFaceApart`, as they do unturned
/// (`a_frustum_whose_top_corner_is_moved_gives_what_the_bar_gives`).
#[test]
fn a_turned_frustum_cut_along_its_seam_gives_what_the_unturned_frustum_gives_turned() {
    let half_the_frustum = 26.0 * PI / 3.0;
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for rule in [Rule::Difference, Rule::Intersection] {
            let answer_of = |placement, moved_to| {
                fuse(
                    Shape::Frustum(placement),
                    moved_to,
                    THE_HALF_OF_THE_FRUSTUM_PAST_THE_SEAM,
                    rule,
                    &policy,
                )
                .unwrap_or_else(|why| {
                    panic!(
                        "the frustum at {placement:?} with its corner at {moved_to:?}, {rule:?} \
                         under {policy:?}, refused: {why:?}"
                    )
                })
            };
            let unturned = answer_of(Isometry::identity(), WHERE_THE_FRUSTUM_PUTS_IT);
            holds(&unturned, half_the_frustum, rule, &policy);

            let turned = answer_of(the_turn(), WHERE_THE_FRUSTUM_PUTS_IT);
            holds(&turned, half_the_frustum, rule, &policy);
            turned
                .model
                .validate(&policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|why| {
                    panic!("{rule:?} under {policy:?} does not validate: {why:?}")
                });
            the_same_corners_to_a_rounding(&turned, &unturned, &the_turn(), rule);

            let cut_away = answer_of(the_turn(), THE_FRUSTUMS_CORNER_A_QUARTER_TURN_ROUND);
            holds(&cut_away, half_the_frustum, rule, &policy);
            the_same_corners_to_a_rounding(&cut_away, &turned, &Isometry::identity(), rule);

            for moved_to in [
                THE_FRUSTUMS_CORNER_OFF_THE_WALL,
                THE_FRUSTUMS_CORNER_OFF_THE_CAP,
            ] {
                check(
                    Shape::Frustum(the_turn()),
                    moved_to,
                    THE_HALF_OF_THE_FRUSTUM_PAST_THE_SEAM,
                    rule,
                    &policy,
                    Outcome::CannotTellTheCellsOfAFaceApart,
                );
            }
        }
    }
}

/// **Turned and then shifted, the frustum cut along its seam gets past its
/// bottom seam corner, as the bar does, and refuses at its apex** (§294.13,
/// Q2374). Under `certified` the sound frustum and the corner moved a quarter
/// turn round refuse `side of a half-space`, with two records, each on a box
/// `2e-13` across round where the apex `(0, 0, 0)` lands, which the block's
/// face along the seam straddles. The corners moved off the wall and off the
/// cap refuse `ResultIsNotManifold`.
///
/// The face has the normal of the frustum's seam plane, moved and turned
/// round, bit for bit, and an offset one ulp further out along it, because the
/// block's placement is the shift and the turn composed with the block's own
/// shift to `(-5, 0, 1)`, and that rounds. So the face is not the seam plane,
/// and the corner is not taken as on it unasked (§294.12). Since Q2374 the
/// bottom seam corner's side of the face is which of the two offsets is the
/// larger, as the turned and shifted bar's is
/// (`a_turned_and_shifted_bar_cut_along_its_seam_gives_what_the_turned_bar_gives_shifted`);
/// until then every corner here refused `side of plane`, on a box `4.6e-13`
/// across round where the bottom seam corner `(1, 0, 2)` lands. The face runs
/// along the frustum's axis and misses it: the apex is behind it, so the block
/// reaches a rounding past the axis, and the face crosses the cone in a
/// hyperbola, whose two branches `intersect` names. The bar's face is one ulp
/// the other way (§294.9).
#[test]
fn a_turned_and_shifted_frustum_cut_along_its_seam_cannot_tell_which_side_of_a_half_space_its_apex_is()
 {
    let placement = the_turn_then_the_shift();
    let (mut model, frustum) = read_back(Shape::Frustum(placement), WHERE_THE_FRUSTUM_PUTS_IT);
    let block = a_block(
        &mut model,
        THE_HALF_OF_THE_FRUSTUM_PAST_THE_SEAM,
        &placement,
    );
    let the_seam_plane = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
        .expect("y = 0 holds the frustum's axis and both its seam corners")
        .transformed(&placement)
        .reversed();
    let face = the_face_with_the_normal_of(&model, block, &the_seam_plane);
    assert_eq!(
        face.offset(),
        the_seam_plane.offset().next_up(),
        "the block's face is one ulp further out than the seam plane"
    );
    let cone = the_frustums_cone(&model, frustum);
    let a_pipe_round_the_axis = Cylinder::new(cone.axis(), 1.0).expect("a pipe");
    assert_eq!(
        how_a_plane_leans_against_a_pipe(&face, &a_pipe_round_the_axis),
        Decision::Zero,
        "the face runs along the axis"
    );
    assert_eq!(
        where_a_cones_apex_stands_against_a_plane(&face, &cone),
        Decision::Neg,
        "and misses it, with the apex behind the face"
    );
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        let met = intersect(
            &Surface::Plane(face),
            &Surface::Cone(cone),
            &policy,
            &mut ProvenanceLog::new(),
        );
        let Ok(SurfaceMeet::Curves(branches)) = met else {
            panic!("{policy:?}: the hyperbola is named: {met:?}");
        };
        assert_eq!(branches.len(), 2, "{policy:?}: its two branches");
    }

    let where_the_apex_lands = placement.apply_to_point(Point::origin()).coordinates();
    for rule in [Rule::Difference, Rule::Intersection] {
        for moved_to in [
            WHERE_THE_FRUSTUM_PUTS_IT,
            THE_FRUSTUMS_CORNER_A_QUARTER_TURN_ROUND,
        ] {
            let case = format!("{rule:?} with the top corner at {moved_to:?}");
            let refused = fuse(
                Shape::Frustum(placement),
                moved_to,
                THE_HALF_OF_THE_FRUSTUM_PAST_THE_SEAM,
                rule,
                &Policy::certified(),
            );
            let Err(Error::Undecidable { records }) = refused else {
                panic!("{case}: {:?}", refused.map(|answer| answer.solids.len()));
            };
            let [_, _] = records.records() else {
                panic!("{case}: two records, {records:?}");
            };
            for record in records.records() {
                assert_eq!(record.predicate(), "side of a half-space", "{case}");
                let [apex] = record.inputs() else {
                    panic!("{case}: one point asked about, {record:?}");
                };
                assert!(apex.uncertainty() < 5e-13, "{case}: {apex:?}");
                assert!(
                    apex.enclosure()
                        .iter()
                        .zip(where_the_apex_lands)
                        .all(|(side, at)| side.lo() <= at && at <= side.hi()),
                    "{case}: {apex:?} holds {where_the_apex_lands:?}"
                );
                assert!(
                    matches!(face.side_of_point(*apex), Decision::Undecidable { .. }),
                    "{case}: the block's face straddles {apex:?}"
                );
            }
        }
        for moved_to in [
            THE_FRUSTUMS_CORNER_OFF_THE_WALL,
            THE_FRUSTUMS_CORNER_OFF_THE_CAP,
        ] {
            check(
                Shape::Frustum(placement),
                moved_to,
                THE_HALF_OF_THE_FRUSTUM_PAST_THE_SEAM,
                rule,
                &Policy::certified(),
                Outcome::ResultIsNotManifold,
            );
        }
    }
}

/// **Under `tolerant(100)` the turned and shifted frustum cut along its seam
/// answers, as the turned and shifted bar does** (§294.15, Q1405), where
/// `certified` cannot tell which side of a half-space the apex is
/// (`a_turned_and_shifted_frustum_cut_along_its_seam_cannot_tell_which_side_of_a_half_space_its_apex_is`).
/// The sound frustum answers both rules with half the frustum, `26π/3` on a
/// fine mesh, its whole model validates, and its corners are the turned
/// frustum's shifted, to a rounding. The corner moved a quarter turn round is
/// cut away and gives the sound answer to a rounding. The corners moved off the
/// wall and off the cap give `ResultIsNotManifold`, as under `certified`, where
/// turned alone they give `CannotTellTheCellsOfAFaceApart` (Q2374). The bar,
/// placed the same way and cut by the half past its seam, is half the bar
/// under both rules.
///
/// **Until Q1405 the sound frustum and the corner moved a quarter turn round
/// refused `CannotNameWhichPieceOfACrossingThisIs`** (§294.13, §294.14), when
/// `nest_the_shells` meshed the answer to measure it. The mesher walks the edge
/// the face cuts along the cone, between corners on the bottom and top circles,
/// as one branch of the hyperbola the face cuts, and `rcad_geom::walk` picked
/// the branch by asking each branch's vertex which side of the apex it is on.
/// The face misses the apex by `5e-16`, so each vertex is within `1e-15` of the
/// apex and placed only to `9.9e-14`, and neither policy could tell. The branch
/// is named by the side of the apex the corners are on now
/// (`walking_the_cut_along_a_turned_and_shifted_frustums_seam_names_the_branch_its_corners_are_on`).
#[test]
fn under_a_tolerant_policy_a_turned_and_shifted_frustum_cut_along_its_seam_answers_as_the_bar_does()
{
    let policy = Policy::tolerant(100.0);
    let placement = the_turn_then_the_shift();
    let half_the_frustum = 26.0 * PI / 3.0;
    for rule in [Rule::Difference, Rule::Intersection] {
        let bar = fuse(
            Shape::Bar(placement),
            WHERE_THE_BAR_PUTS_IT,
            THE_HALF_PAST_THE_SEAM,
            rule,
            &policy,
        )
        .unwrap_or_else(|why| panic!("the bar, {rule:?}, refused: {why:?}"));
        holds(&bar, PI * RADIUS * RADIUS * HEIGHT / 2.0, rule, &policy);

        let answer_of = |placement, moved_to| {
            fuse(
                Shape::Frustum(placement),
                moved_to,
                THE_HALF_OF_THE_FRUSTUM_PAST_THE_SEAM,
                rule,
                &policy,
            )
            .unwrap_or_else(|why| {
                panic!(
                    "the frustum at {placement:?} with its corner at {moved_to:?}, {rule:?}, \
                     refused: {why:?}"
                )
            })
        };
        let sound = answer_of(placement, WHERE_THE_FRUSTUM_PUTS_IT);
        holds(&sound, half_the_frustum, rule, &policy);
        sound
            .model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{rule:?} does not validate: {why:?}"));
        let turned = answer_of(the_turn(), WHERE_THE_FRUSTUM_PUTS_IT);
        the_same_corners_to_a_rounding(&sound, &turned, &the_shift(), rule);

        let cut_away = answer_of(placement, THE_FRUSTUMS_CORNER_A_QUARTER_TURN_ROUND);
        holds(&cut_away, half_the_frustum, rule, &policy);
        the_same_corners_to_a_rounding(&cut_away, &sound, &Isometry::identity(), rule);

        for moved_to in [
            THE_FRUSTUMS_CORNER_OFF_THE_WALL,
            THE_FRUSTUMS_CORNER_OFF_THE_CAP,
        ] {
            check(
                Shape::Frustum(placement),
                moved_to,
                THE_HALF_OF_THE_FRUSTUM_PAST_THE_SEAM,
                rule,
                &policy,
                Outcome::ResultIsNotManifold,
            );
        }
    }
}

/// **Walking the cut along the seam names the branch its corners are on by
/// which side of the apex they are, and asks no point a rounding from the
/// apex** (§294.15, Q1405). The block's face misses the cone's apex by
/// `5e-16`, so the two cross in a hyperbola whose two vertices stand about
/// `1e-15` either side of the apex. `rcad_geom::walk`, handed that crossing
/// between the two seam corners, finds both corners on the axis's side of the
/// apex, which names the branch opening against the face's `and` direction,
/// since that direction always runs against the axis. Under either policy and
/// whichever of the two runs `intersect` names it is handed, it walks that
/// branch with nothing to record: every point it puts down is on the cone's
/// wall between the bottom circle and the top one, on the face, and on the
/// cone, to the walk's own deviation.
///
/// **Until Q1405 it refused `CannotNameWhichPieceOfACrossingThisIs`**
/// (§294.14). It asked each branch's vertex which side of the apex it is on,
/// and a vertex is placed only to `9.9e-14`, so its box held the apex:
/// `certified` could not tell, and `tolerant(100)` read it as on the plane
/// through the apex, which names no branch either. The fuse met this when it
/// meshed its answer to nest the shells.
#[test]
fn walking_the_cut_along_a_turned_and_shifted_frustums_seam_names_the_branch_its_corners_are_on() {
    let placement = the_turn_then_the_shift();
    let (mut model, frustum) = read_back(Shape::Frustum(placement), WHERE_THE_FRUSTUM_PUTS_IT);
    let block = a_block(
        &mut model,
        THE_HALF_OF_THE_FRUSTUM_PAST_THE_SEAM,
        &placement,
    );
    let the_seam_plane = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
        .expect("y = 0 holds the frustum's axis and both its seam corners")
        .transformed(&placement)
        .reversed();
    let face = the_face_with_the_normal_of(&model, block, &the_seam_plane);
    let cone = the_frustums_cone(&model, frustum);
    assert_eq!(
        how_a_plane_leans_against_a_cone(&face, &cone),
        Decision::Neg,
        "the face is steeper than the cone, so a face that misses the apex crosses it in a \
         hyperbola"
    );
    let apex = placement.apply_to_point(Point::origin());
    assert_eq!(cone.apex().coordinates(), apex.coordinates());
    let up = cone.axis().direction() * cone.axis().direction().length().recip();
    let the_seam_corners = [Point::new(1.0, 0.0, 2.0), Point::new(3.0, 0.0, 6.0)]
        .map(|corner| PredicatePoint::from(placement.apply_to_point(corner).coordinates()));
    let (plane, slope) = (face, cone.slope());
    let [face, cone] = [Surface::Plane(face), Surface::Cone(cone)];

    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        let met = intersect(&face, &cone, &policy, &mut ProvenanceLog::new());
        let Ok(SurfaceMeet::Curves(branches)) = met else {
            panic!("{policy:?}: the hyperbola is named: {met:?}");
        };
        for branch in branches {
            let run = NamedRun::new(branch.run().clone(), the_seam_corners)
                .expect("a branch of a hyperbola does not go right round");
            let mut log = ProvenanceLog::new();
            let walked = walk(
                &run,
                [&face, &cone],
                Deflection::of(0.01).expect("a length"),
                &policy,
                &mut log,
            )
            .unwrap_or_else(|why| panic!("{policy:?}: the walk refused: {why:?}"));
            assert!(
                log.records().is_empty(),
                "{policy:?}: nothing to record, {log:?}"
            );
            assert!(!walked.points().is_empty(), "{policy:?}: {walked:?}");
            let room = 2.0 * walked.deviation().magnitude();
            for point in walked.points() {
                let height = (*point - apex).dot(up);
                assert!(
                    2.0 < height && height < 6.0,
                    "{policy:?}: {point:?} is {height} up the axis, off the frustum's wall"
                );
                assert!(
                    plane.signed_distance_to(*point).abs() <= room,
                    "{policy:?}: {point:?} is off the face"
                );
                let out_from_the_axis = ((*point - apex) - up * height).length();
                assert!(
                    (out_from_the_axis - slope * height).abs() <= room,
                    "{policy:?}: {point:?} is off the cone"
                );
            }
        }
    }
}

/// **A plane through the apex that misses the axis turns its two rulings by
/// something, and the fuse works that turn out.** The frustum turned 0.4 about
/// `x`, and the half past its seam left unturned: the block's face `y = 0`
/// runs through the apex at the origin and is steeper than the cone, so the two
/// cross in two rulings through it, as along the seam. But the axis is turned
/// 0.4 out of the face, so `n·w` is something, and the two halves into the
/// frustum's nappe are not mirrors across the axis. About it they are less than
/// half a turn apart, and the turn at the apex is not nothing. Read as nothing,
/// as a turn between two rulings of any one plane through the apex would be,
/// the difference refuses `TwoCurvesTouchWithoutCrossing` on the frustum's
/// wall: measured with the axis check taken out of `arrange.rs`.
///
/// In the frustum's own frame the face is `y = z·tan 0.4`, which cuts each
/// circle `z = h` square to the axis, of radius `h/2`, along a chord
/// `h·tan 0.4` from its middle. So the piece past the face is `K·h²` across at
/// every height, `K = acos(2·tan 0.4)/4 − tan 0.4·√(1/4 − tan² 0.4)`, and
/// `K·(6³ − 2³)/3` from the bottom cap to the top. That piece is what the
/// block takes, and what is left of `52π/3`.
#[test]
fn a_plane_through_a_frustums_apex_that_misses_its_axis_turns_its_two_rulings_by_something() {
    let policy = Policy::certified();
    let lean = 0.4f64.tan();
    let across = (2.0 * lean).acos() / 4.0 - lean * (0.25 - lean * lean).sqrt();
    let past_the_face = across * (6.0f64.powi(3) - 2.0f64.powi(3)) / 3.0;
    for (rule, volume) in [
        (Rule::Difference, 52.0 * PI / 3.0 - past_the_face),
        (Rule::Intersection, past_the_face),
    ] {
        let (mut model, frustum) = read_back(Shape::Frustum(the_turn()), WHERE_THE_FRUSTUM_PUTS_IT);
        let block = a_block(
            &mut model,
            THE_HALF_OF_THE_FRUSTUM_PAST_THE_SEAM,
            &Isometry::identity(),
        );
        let (cone, face) = the_cone_and_the_face_through_its_apex(&model, frustum, block);
        assert_eq!(
            how_a_plane_leans_against_a_cone(&face, &cone),
            Decision::Neg,
            "the face is steeper than the cone, so they cross in two rulings"
        );
        let a_pipe_round_the_axis = Cylinder::new(cone.axis(), 1.0).expect("a pipe");
        assert_ne!(
            how_a_plane_leans_against_a_pipe(&face, &a_pipe_round_the_axis),
            Decision::Zero,
            "the face misses the axis"
        );

        let answer = model
            .fuse(
                &[frustum],
                &[block],
                rule,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{rule:?} refused: {why:?}"));
        let [solid] = answer.solids() else {
            panic!("{rule:?}: {:?}", answer.solids());
        };
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{rule:?} does not validate: {why:?}"));
        let measured = model
            .solid(*solid)
            .expect("the fuse made this key")
            .measure(
                Deflection::of(0.001).expect("a fine mesh"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{rule:?} does not measure: {why:?}"));
        assert!(
            (measured.volume() - volume).abs() <= measured.volume_error(),
            "{rule:?} holds {volume} and measures {} give or take {}",
            measured.volume(),
            measured.volume_error()
        );
    }
}

/// **The turned frustum's cone and the block's face along the seam cross in
/// four half rulings, which `rcad_geom::intersect` names.** That face runs
/// through the cone's apex at the origin and is steeper than the cone, so the
/// two cross in two rulings through the apex. Unturned, `intersect` names the
/// two whole, along directions `f64` writes down exactly on both surfaces.
/// Turned, no such direction exists (§294.11), and it names the four halves the
/// apex cuts the two into, each by a corner of two flat surfaces through the
/// apex. It refused the turned pair until Q1171 (§294.10).
#[test]
fn the_face_along_a_turned_frustums_seam_crosses_its_cone_in_four_half_rulings() {
    let policy = Policy::certified();
    for placement in [Isometry::identity(), the_turn()] {
        let (mut model, frustum) = read_back(Shape::Frustum(placement), WHERE_THE_FRUSTUM_PUTS_IT);
        let block = a_block(
            &mut model,
            THE_HALF_OF_THE_FRUSTUM_PAST_THE_SEAM,
            &placement,
        );
        let (cone, face) = the_cone_and_the_face_through_its_apex(&model, frustum, block);
        assert_eq!(
            how_a_plane_leans_against_a_cone(&face, &cone),
            Decision::Neg,
            "{placement:?}: the face is steeper than the cone, so they cross in two rulings"
        );
        let met = intersect(
            &Surface::Plane(face),
            &Surface::Cone(cone),
            &policy,
            &mut ProvenanceLog::new(),
        );
        let Ok(SurfaceMeet::Curves(runs)) = met else {
            panic!("{placement:?}: the crossing is named: {met:?}");
        };
        let how_many = if placement == Isometry::identity() {
            2
        } else {
            4
        };
        assert_eq!(
            runs.len(),
            how_many,
            "{placement:?}: two whole rulings unturned, four halves turned"
        );
    }
}

/// The frustum's cone, its apex at the origin, and the one face of the block
/// that runs through that apex, exactly.
fn the_cone_and_the_face_through_its_apex(
    model: &Model,
    frustum: SolidKey,
    block: SolidKey,
) -> (Cone, Plane) {
    let cone = the_frustums_cone(model, frustum);
    let through_the_apex: Vec<Plane> = the_planes_of(model, block)
        .into_iter()
        .filter(|flat| flat.offset() == 0.0)
        .collect();
    let [face] = through_the_apex.as_slice() else {
        panic!("one face through the origin: {through_the_apex:?}");
    };
    assert_eq!(cone.apex(), Point::origin());
    assert_eq!(
        where_a_cones_apex_stands_against_a_plane(face, &cone),
        Decision::Zero,
        "the face runs through the apex"
    );
    (cone, *face)
}

/// The one face of `block` whose plane has `like`'s normal, bit for bit.
fn the_face_with_the_normal_of(model: &Model, block: SolidKey, like: &Plane) -> Plane {
    let bits = |flat: &Plane| {
        let normal = flat.normal();
        [normal.x(), normal.y(), normal.z()].map(f64::to_bits)
    };
    let alike: Vec<Plane> = the_planes_of(model, block)
        .into_iter()
        .filter(|flat| bits(flat) == bits(like))
        .collect();
    let [face] = alike.as_slice() else {
        panic!("one face with the normal of {like:?}: {alike:?}");
    };
    *face
}

/// The frustum's one cone, as placed.
fn the_frustums_cone(model: &Model, frustum: SolidKey) -> Cone {
    let cones: Vec<Cone> = model
        .solid(frustum)
        .expect("this model made the key")
        .faces()
        .filter_map(|face| {
            let Surface::Cone(cone) = face.surface() else {
                return None;
            };
            Some(cone)
        })
        .collect();
    let [cone] = cones.as_slice() else {
        panic!("one cone: {cones:?}");
    };
    *cone
}

/// The planes of `solid`'s flat faces, as placed.
fn the_planes_of(model: &Model, solid: SolidKey) -> Vec<Plane> {
    model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .filter_map(|face| face.surface().as_plane().copied())
        .collect()
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
        | Outcome::ResultIsNotManifold => {
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
        Outcome::ResultIsNotManifold => matches!(refused, Error::ResultIsNotManifold { .. }),
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

/// The answer is one solid, `validate_solid` accepts it, and it measures
/// `volume` to within what a fine mesh can say.
fn holds(answer: &Answer, volume: f64, rule: Rule, policy: &Policy) {
    let [solid] = answer.solids[..] else {
        panic!("{rule:?}: {} solids", answer.solids.len());
    };
    answer
        .model
        .validate_solid(solid, policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{rule:?} does not validate: {why:?}"));
    let measured = answer
        .model
        .solid(solid)
        .expect("the fuse made this key")
        .measure(
            Deflection::of(0.001).expect("a fine mesh"),
            policy,
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{rule:?} does not measure: {why:?}"));
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "{rule:?} holds {volume} and measures {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// Further than a corner of the turned bar can land from where the turn puts
/// it: the widest box the turn moves one into is `2.8e-13` either way, round
/// the top seam corner.
const A_ROUNDING_OF_THE_TURN: f64 = 1e-12;

/// Each corner of `theirs`, moved by `movement`, is within
/// [`A_ROUNDING_OF_THE_TURN`] of exactly one corner of `mine`, and the two have
/// as many corners.
fn the_same_corners_to_a_rounding(mine: &Answer, theirs: &Answer, movement: &Isometry, rule: Rule) {
    let point = |bits: &[u64; 3]| {
        let [x, y, z] = bits.map(f64::from_bits);
        Point::new(x, y, z)
    };
    assert_eq!(mine.corners.len(), theirs.corners.len(), "{rule:?}");
    for corner in &theirs.corners {
        let there = movement.apply_to_point(point(corner));
        let near = mine
            .corners
            .iter()
            .filter(|other| (point(other) - there).length() <= A_ROUNDING_OF_THE_TURN)
            .count();
        assert_eq!(near, 1, "{rule:?}: the corner {there:?}");
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
            Shape::Frustum(placement) => {
                let frustum = cones::a_frustum(model, 0.5, 2.0, 6.0);
                model
                    .place_solid(frustum, &placement)
                    .expect("this model made the key");
            }
        }
    }

    /// Where the builder puts the corner the file moves, in the stored frame.
    fn corner(self) -> [f64; 3] {
        match self {
            Shape::Bar(_) => WHERE_THE_BAR_PUTS_IT,
            Shape::Ball => THE_SOUTH_POLE,
            Shape::Frustum(_) => WHERE_THE_FRUSTUM_PUTS_IT,
        }
    }

    /// Where the solid is placed, and so where its partner is placed too.
    fn placement(self) -> Isometry {
        match self {
            Shape::Bar(placement) | Shape::Frustum(placement) => placement,
            Shape::Ball => Isometry::identity(),
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
