//! Fusing two solids that touch without crossing, under every rule, beside the
//! same two solids moved one unit apart. `docs/decisions.md` §191.
//!
//! `pairs/mod.rs` keeps every curved solid a unit clear of every wall so that
//! nothing it draws is tangent to anything, and hands tangency to
//! `degenerate_curved_fuses.rs` one hand-written case at a time. But every
//! fillet in a real part is a round face tangent to two flat ones, so a touch is
//! the ordinary case in a file and not the edge case. `touching/mod.rs` draws a
//! catalogue of them, each cell twice: touching, and moved one unit clear.
//!
//! # What is asked of every drawn pair
//!
//! Under `Policy::certified()`, all three rules, both ways round:
//!
//! - **no panic**, and an answer passes `Model::validate` and logs nothing;
//! - **inclusion and exclusion** — `A ∪ B + A ∩ B = A + B`, `A \ B + A ∩ B = A`,
//!   `B \ A + A ∩ B = B` — against the share of the declared mesh error that
//!   `fuses_that_must_agree.rs` measured, never a tolerance of this file's own;
//! - where the insides are apart, `A ∩ B` is **no solid at all**, and where one
//!   is inside the other, `B \ A` is none — exact, not measured;
//! - **union and intersection both ways round** agree on how many solids and on
//!   the volume to within what the two meshes declare.
//!
//! A refusal is a result: it is counted by name, per row, and the table below is
//! the count. A cell whose control never answers has measured nothing, and
//! fails.
//!
//! # The table
//!
//! `mise exec -- just t -p rcad-topo --test fuses_that_touch --no-capture`, from
//! the fixed seed: 26 drawn shapes per cell, split between the two rows, six
//! fuses each. "answered" is out of every fuse that row made. Refreshed by
//! queue row Q1062, whose change moved two cells, the bar's cap and the
//! frustum's cap flush on a block. With that change switched off by hand the
//! bar's cap answered 36 of 66 touching and the frustum's 0 of 90, every other
//! fuse of both refused `Undecidable` at *a point in a piece that every solid
//! could place*, and every other cell printed what it prints here; what moved
//! the other cells since the table before was not traced. Queue row Q1118
//! moved one cell since, the dome's touching row, from none answered to all
//! 66; only that cell's line was printed again.
//!
//! | cell | touching | clear by one | what the touch alone refuses |
//! |---|---|---|---|
//! | two blocks, one flush on the other | 66 of 66 | 90 of 90 | — |
//! | a bar beside a block's wall | 78 of 78 | 78 of 78 | — |
//! | a bar in an inside corner (a fillet) | 84 of 84 | 72 of 72 | — |
//! | a bar's cap flush on a block | 66 of 66 | 90 of 90 | — |
//! | a ball resting on a block | 78 of 78 | 78 of 78 | — |
//! | a ball against a block's wall | 78 of 78 | 78 of 78 | — |
//! | two bars touching along a line | 60 of 60 | 96 of 96 | — |
//! | a bar inside a bar, touching | panics on its control | — | see below |
//! | two balls touching at a point | 60 of 60 | 96 of 96 | — |
//! | a ball inside a ball, touching once | 60 of 60 | 96 of 96 | — |
//! | the dome on a shaft of its radius | 66 of 66 | 90 of 90 | — |
//! | a ball against a bar's wall | 60 of 60 | 96 of 96 | — |
//! | a ball inside a bar, touching | 60 of 60 | 96 of 96 | — |
//! | a cone's apex on a block | 90 of 90 | 66 of 66 | — |
//! | a frustum's cap flush on a block | 90 of 90 | 66 of 66 | — |
//! | a bend lying on a block | 0 of 66 | 0 of 90 | nothing: both rows refuse as a quartic |
//! | a bend's cap flush against a block | 0 of 66 | 90 of 90 | `ACrossingBetweenAPlaneAndATorusThatIsAQuartic` 66 |
//!
//! Every answer that came back passed every law above. The two failures are a
//! **panic** on a bar standing inside a bar and touching nothing —
//! [`a_bar_standing_wholly_inside_a_thicker_bar_is_united_and_hollowed_without_a_panic`]
//! — which the property meets on its control row before its touching row, so
//! that cell's table is not printed until the panic is fixed. By hand, before
//! queue row Q649, the touching row (radius three, radius one at `(0, 2)`)
//! refused every fuse on *which way a crossing runs*; it has not been asked
//! again since.
//!
//! # Which refusals are gaps and which are limits
//!
//! §16.10: a limit where the answer does not exist, a gap where it does. **For
//! every cell whose insides are apart the answer exists and this kernel can
//! hold it**: the union is the two inputs as they stand, the intersection is
//! nothing, and the difference is the first input — three things the model
//! already holds, since both inputs were built and validated. The rows that do
//! answer say the fuse can reach it: two balls touching at a point, a ball
//! against a bar, a ball of radius one on a block, and every control. So every
//! refusal in the "touch alone" column of an apart cell is a **gap**:
//!
//! - A wall or a second pipe lying along a pipe's ruling: **closed**, every
//!   fuse of those three cells answered, 2026-10-03. Every union used to refuse
//!   `ResultIsNotManifold`, because both solids keep both sides of the touching
//!   line and it came out one edge with four faces round it, two pieces of the
//!   wall and the skin walking it both ways (queue row Q760 printed them).
//!   `fuse/assemble.rs` now parts such an edge into one per input face, and the
//!   union is the two solids as they stand, two lumps;
//!   `a_bar_lying_along_a_plane_or_a_bar.rs` pins it against the volumes. Until
//!   queue row Q649 every fuse of these three cells refused before that, on
//!   *which way a crossing runs*: the two surfaces' normals are parallel along
//!   the line, so their cross product is exactly nothing. The line runs along
//!   the pipe's stored axis, and `fuse/arrange.rs`'s `AlongATouch` reads it now.
//!   Handed over first, a bar touching on its seam used to stop earlier still,
//!   on *side of plane* or *side of cylinder*, asked about a box round its seam
//!   corner that a whole-number move had put there without rounding anything;
//!   `a_bar_moved_by_whole_numbers_keeps_its_seam_corner_exact.rs` is that gap,
//!   closed.
//! - A flat cap flush on a flat face with a circle round it: **closed**, every
//!   fuse of the bar's cap and the frustum's cap on a block answers since queue
//!   row Q1062. The one point found inside a cap is the middle of its circle,
//!   and where that is the middle of the block's face too it lies on both the
//!   face's diagonals, so `fuse/select.rs`'s fan of triangles over the face's
//!   corners had it on a spoke from every corner and could not say the face
//!   held it. The cap was never read as lying on the face, and the ray called
//!   the point on the block's skin. `select.rs` asks
//!   `SolidView::face_holds_the_point` where the fan cannot say.
//! - `ResultIsNotManifold` and `NotFinite` on a ball resting on a block, and
//!   `CannotNameWhichPieceOfACrossingThisIs` and *a point inside a piece of a
//!   face* on a cone's apex: every fuse of both cells answers in the table
//!   above. The hand-written rows of
//!   [`a_ball_resting_on_a_block_it_overhangs_unites_into_both_solids_as_they_stand`]
//!   all answer since Q508.
//! - `Invalid(FacesDoNotAgreeWhichWayTheyFace)` on a ball resting on a block
//!   whose wall planes cut its skin into a rim of arcs: **closed** by Q451,
//!   where the mesher learnt to open such a rim the way it opens one closed
//!   curve. Queue row Q892 counted 2 of this cell's touching differences and 4
//!   of its unions refused that way on `a7ad3f4b`;
//!   [`a_ball_whose_skin_the_blocks_walls_cut_into_a_rim_of_arcs_fuses_by_every_rule`]
//!   pins three shapes of it by every rule.
//! - `CannotMeshAFace` with `BoundaryCrossesItself` on a ball of radius four
//!   resting a unit in from two walls of a block four wide: **closed** by
//!   Q1099, where the mesher learnt to leave whole a piece of the ball's skin
//!   that no straight cut between its corners narrows, rather than cut it along
//!   a chord that leaves it. A sweep of 16 200 fuses of whole-number balls
//!   resting on whole-number blocks counted 180 refused that way on
//!   `3741c5a7`, union both ways and ball less block, and none since;
//!   [`a_ball_whose_skin_the_walls_cut_into_a_piece_no_straight_cut_narrows_fuses_by_every_rule`]
//!   pins five shapes of it by every rule.
//!
//! **The dome is known: §124.** A ball on a bar of its own radius, centred on
//! the bar's axis, shares exactly one circle with it **only** because it is
//! tangent (§128.3: `(z − h)² = R² − r²` is two circles for `R > r` and none for
//! `R < r`), so it has no transversal control at equal radius; the control here
//! widens the bar by one, where the ball sits wholly inside the pipe and crosses
//! nothing but the cap. **Of every cell here it is the only one with that
//! property**: every other touch has a crossing twin one unit the other way.
//! Its control answers all 90. The touching row used to stop where the cap
//! plane, the ball and the bar meet, refused
//! `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`; the cap is square to the axis
//! the two share and cuts both in one circle, which `three_surfaces.rs`
//! answers as `MoreThanAPoint` since Q981. The fuse then went on and refused
//! every rule `TwoCurvesTouchWithoutCrossing`: the cap's rim and the circle the
//! ball cuts the cap in are one curve drawn twice. Since Q987 the arrangement
//! draws it once, and every fuse refused one step on as an edge with one face
//! along it, because the rim came out of the bar and out of the ball under two
//! names. Since Q1118 `fuse/assemble.rs` gives the rim one name and one corner,
//! and every fuse of the touching row answers.
//! [`a_dome_on_a_shaft_of_its_radius_by_every_rule_against_the_closed_forms`]
//! checks nine domes by every rule against their closed forms. **Laid on its
//! side it answers all 72 fuses**: the ball's poles are then on the rim, and
//! the ball's side of the rim, cut in two at them, is read as one closed side
//! again since Q1368. 42 of them were then refused because the two sides of the
//! rim were read against the pipe and the ball, which disagree about which way
//! it runs, until Q1377 read both against the ball;
//! [`a_dome_lying_on_its_side_by_every_rule_against_the_closed_forms`] checks
//! them against the closed forms.
//!
//! **A bend's touch is not reached.** Every fuse of a bend lying on a block is
//! refused as a quartic, touching or one unit clear, so that cell is pinned at
//! *nothing answers* and fails the day that changes. A bend's cap against a
//! block refuses its touching row the same way. Its control, whose block is
//! clear of the bend, answers all 90. The 45 that keep the bend were refused
//! writing it back, `TheFuseWroteABrokenAnswer` with `CoedgesRunTheSameWay`,
//! the stop §243.5 and §323.5 record, until the bend's wall was used forwards
//! (§361). A bar of the tube's radius butted onto a bend's cap with a flat far
//! end is refused as a quartic for that end's plane;
//! [`a_flat_ended_bar_on_a_bends_cap_is_refused_as_a_quartic`] pins that. Cut
//! through the ring's axis instead, its pipe meets the ring in the cap's rim
//! and in a loop crossing it, which no edge of the answer runs along. Every
//! rule was refused putting points in order along the half of the loop on the
//! bar's pipe face until Q1470 left each half off the face across the cap from
//! the one carrying it;
//! [`a_bar_cut_through_the_rings_axis_on_a_bends_cap_by_every_rule_against_the_closed_forms`]
//! checks every rule against the closed forms.
//!
//! # What is not built, and why
//!
//! - **A bar lying on its side** on a block or another bar needs a quarter turn,
//!   and a quarter turn of a round solid cannot be asked for exactly
//!   (`fuses_that_must_agree.rs`'s
//!   `a_quarter_turn_of_a_round_solid_cannot_be_asked_for_exactly`). An upright
//!   bar beside a wall is the same touch turned, and is built instead.
//! - **A plane lying along a cone's ruling** needs a slanted face, which no
//!   whole-number block has; `degenerate_curved_fuses.rs` builds it by hand as
//!   `a_wedge_lying_on_a_cones_side_shares_and_leaves_what_is_worked_out_by_hand`.
//! - **A cone against a ball or a bar**: not built yet, and nothing here says
//!   whether they would reach their touch. A cone's side against another cone
//!   *is* built now, as one hand-written pair rather than a cell —
//!   [`two_cones_sharing_one_rim_circle_fuse_by_every_rule`] — and every rule
//!   of it answers. A torus against anything curved is refused by name before it could
//!   (`how_often_a_fuse_with_a_torus_refuses.rs`'s last three rows).

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;
use proptest::test_runner::{
    Config, FileFailurePersistence, RngAlgorithm, TestCaseError, TestRng, TestRunner,
};
use rcad_geom::{Axis, Deflection, Isometry, Plane, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, Model, ProfileBuilder, Rule, SolidKey};

mod touching;

use touching::{Apart, Drawn, Insides, Shape, Slope, both_rows};

/// How many shapes each cell draws, touching and clear together.
///
/// [`SOAK_FACTOR`] multiplies it and [`SOAK_SEED`] replaces the fixed seed, the
/// way `fuses_that_must_agree.rs` does. With neither set — which is how the gate
/// runs it — this is 24 shapes from the one fixed seed, and the table at the top
/// of this file is what that prints.
const CASES: u32 = 24;

/// The environment variable that multiplies [`CASES`] for a soak.
///
/// ```text
/// RCAD_SOAK_FACTOR=100 cargo nextest run -p rcad-topo --test fuses_that_touch
/// ```
const SOAK_FACTOR: &str = "RCAD_SOAK_FACTOR";

/// The environment variable that reseeds every cell.
///
/// These tests are deterministic on purpose, so running the same soak twice
/// draws the same shapes twice. A *spread* needs this varied as well.
const SOAK_SEED: &str = "RCAD_SOAK_SEED";

#[test]
fn two_blocks_one_on_the_other() {
    every_rule_over(
        "two blocks, one flush on the other",
        touching::a_block_on_a_block(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn a_bar_standing_beside_a_blocks_wall() {
    every_rule_over(
        "a bar beside a block's wall",
        touching::a_bar_beside_a_wall(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn a_bar_in_an_inside_corner_touching_both_walls() {
    every_rule_over(
        "a bar in an inside corner, the fillet",
        touching::a_bar_in_an_inside_corner(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn a_bar_standing_on_a_block() {
    every_rule_over(
        "a bar's cap flush on a block",
        touching::a_bar_standing_on_a_block(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn a_ball_resting_on_a_block() {
    every_rule_over(
        "a ball resting on a block",
        touching::a_ball_on_a_block(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn a_ball_against_a_blocks_wall() {
    every_rule_over(
        "a ball against a block's wall",
        touching::a_ball_against_a_wall(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn two_bars_touching_along_a_line() {
    every_rule_over(
        "two bars touching along a line",
        touching::two_bars_touching(),
        Expect::TheControlAnswers,
    );
}

#[test]
#[ignore = "open: §191, its control row is a bar wholly inside a bar, which panics in fuse/assemble.rs"]
fn a_bar_inside_a_bar_touching_its_wall() {
    every_rule_over(
        "a bar inside a bar, touching",
        touching::a_bar_inside_a_bar(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn two_balls_touching_at_a_point() {
    every_rule_over(
        "two balls touching at a point",
        touching::two_balls_touching(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn a_ball_inside_a_larger_ball_touching_once() {
    every_rule_over(
        "a ball inside a ball, touching once",
        touching::a_ball_inside_a_ball(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn a_ball_on_a_bar_of_the_same_radius() {
    every_rule_over(
        "the dome on a shaft of its radius",
        touching::a_dome_on_a_shaft(),
        Expect::TheControlAnswers,
    );
}

/// The dome on a shaft of its radius, every rule both ways round, against the
/// closed forms.
///
/// A ball of radius `R` centred on the top of a bar of radius `R` and height
/// `h` shares one circle with the bar, the rim of its top cap, and is tangent
/// to the pipe all the way round it. So the answers are a hemisphere and a
/// bar: `A ∪ B = πR²h + ⅔πR³`, `A ∩ B = ⅔πR³` (the lower half of the ball,
/// inside the bar since `h > R`), `bar \ ball = πR²h − ⅔πR³`, and
/// `ball \ bar = ⅔πR³`. An answer is checked against those, to within what
/// its mesh declares.
///
/// **What used to stop every fuse, traced by Q987.** The cap's plane cuts the
/// ball in the cap's own rim, and so does the pipe it is tangent to. Drawn as
/// two crossings of the cap, each node of the rim had two ways out leaving it
/// the same way and bending alike, and `fuse/arrange.rs` refused all 66 fuses
/// of the touching row `TwoCurvesTouchWithoutCrossing`. The same circle twice
/// is one curve: the arrangement folds it into one crossing now, on the cap, on
/// the pipe and on the ball, and every face is cut into the pieces it has.
///
/// **What stopped it next was joining those pieces, traced by Q1118.** The
/// rim came out of the bar's two faces named by the cap's plane and the pipe,
/// at the bar's seam corner `(R, 0, h)`, and out of the ball named by the
/// cap's plane and the ball, at `(0, R, h)` where that pair starts a circle.
/// `fuse/assemble.rs` named a closed edge by the two surfaces either side, so
/// that was two edges with one face each, and all 54 fuses here were refused
/// `ResultIsNotManifold { count: 1 }`. It gives the two names one now, and
/// the rim one corner, the seam's. Past that the union and the bar less the
/// ball keep the pipe and the ball either side of the rim, which touch and do
/// not cross, and were refused `ACrossingAlongThisEdgeCouldNotBeFollowed`
/// until the rim was written as the cap's plane against a face.
#[test]
fn a_dome_on_a_shaft_of_its_radius_by_every_rule_against_the_closed_forms() {
    let policy = Policy::certified();
    let third = std::f64::consts::PI / 3.0;
    for radius in 1..=3 {
        for height in radius + 1..=radius + 3 {
            let (r, h) = (f64::from(radius), f64::from(height));
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let shaft = model
                .add_round_bar(r, h, &Isometry::identity())
                .expect("a whole-number bar");
            let dome = model
                .add_ball(
                    r,
                    &Isometry::translation(Vector::new(0.0, 0.0, h)).expect("a whole-number shift"),
                )
                .expect("a whole-number ball");
            let bar = 3.0 * third * r * r * h;
            let half_ball = 2.0 * third * r * r * r;
            for (one, other, rule, holds) in [
                (shaft, dome, Rule::Union, bar + half_ball),
                (dome, shaft, Rule::Union, bar + half_ball),
                (shaft, dome, Rule::Intersection, half_ball),
                (dome, shaft, Rule::Intersection, half_ball),
                (shaft, dome, Rule::Difference, bar - half_ball),
                (dome, shaft, Rule::Difference, half_ball),
            ] {
                let mut fused = model.clone();
                let shaft_first = one == shaft;
                let answer = fused
                    .fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| {
                        panic!(
                            "{rule:?} of R = {r}, h = {h} (shaft first: {shaft_first}) refused \
                             {why}; every fuse of these nine domes answers since Q1118"
                        )
                    });
                fused
                    .validate(&policy, &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| panic!("{rule:?} does not validate: {why}"));
                assert_eq!(
                    answer.solids().len(),
                    1,
                    "{rule:?} of R = {r}, h = {h} (shaft first: {shaft_first}) is one solid"
                );
                let (volume, error) = volume_and_error(&fused, answer.solids());
                assert!(
                    (volume - holds).abs() <= error,
                    "{rule:?} of R = {r}, h = {h} (shaft first: {shaft_first}) measures {volume} \
                     against {holds}, out by more than the {error} its mesh declares"
                );
            }
        }
    }
}

/// The dome on a shaft of its radius lying on its side, every rule both ways
/// round, along `±x` and `±y`, for `R` from one to three: **all 72 fuses answer
/// the closed forms and validate.**
///
/// A bar of radius `R` and length `h = R + 1` holds `πR²h`, and a ball of radius
/// `R` centred on the middle of its end cap is cut by the cap's plane into two
/// half balls of `⅔πR³` each. The half towards the bar is inside it, since a
/// point of it is at most `R` from the cap's middle, so at most `R` from the
/// bar's axis and at most `R < h` along it; the other half is outside it. The
/// two touch along the cap's rim and cross nowhere else. So `A ∪ B = πR²h +
/// ⅔πR³`, `A ∩ B = ⅔πR³`, `bar \ ball = πR²h − ⅔πR³` and `ball \ bar = ⅔πR³`:
/// for `R = 1`, `2π + ⅔π = 8.378`, `2.094`, `4.189` and `2.094`. Each answer is
/// one solid and is checked against these to within what its mesh declares,
/// the same closed forms
/// [`a_dome_on_a_shaft_of_its_radius_by_every_rule_against_the_closed_forms`]
/// holds the upright dome to.
///
/// A ball's poles are always straight up and down from its centre, so a ball
/// centred on the end cap of a bar lying along `x` or `y` has both poles on the
/// cap's rim, and the cap's plane runs down its polar axis. `fuse/arrange.rs`
/// cuts the ball's side of the rim in two at the poles
/// (`across_a_circle_that_starts_at_a_pole`, §194.4): from the pole it starts at
/// to the other, `HalfWayRoundAClosedCurve`, and back. The bar's side of the rim
/// is one closed side at the bar's seam corner. `fuse/assemble.rs`'s
/// `the_name_each_closed_curve_goes_by` joined the names of closed sides only, so
/// the two halves and the closed side were three edges, each with one face, and
/// all 72 fuses here were refused `ResultIsNotManifold { count: 1 }` (Q1143).
/// `fuse/assemble.rs` now reads the two halves as one closed side wherever
/// another side closes on the same curve whole, the rim gets the seam corner,
/// and each side of it is read at that corner.
///
/// **What stopped 42 of them next was which way the rim runs** (Q1368, Q1377).
/// Each side is read against the curved surface of its own pair: the bar's cap
/// and pipe against the pipe, the ball's against the ball. The rim is a
/// meridian of the ball, which the ball's longitude is square to, so the ball's
/// side is read against its latitude where the corner is off the poles and
/// against its lid, the cap's plane, where it is a pole. Those agree with the
/// pipe's longitude only for the bar running along `−y`. Along `+y` they
/// disagreed, both faces walked the rim the same way, and the sweep refused
/// every fuse `TheFuseWroteABrokenAnswer` with `CoedgesRunTheSameWay`; along
/// `x`, where the seam corner is a pole, the union and the differences were
/// refused `FacesDoNotAgreeWhichWayTheyFace`. `fuse/assemble.rs`'s
/// `every_closed_edge_read_off_one_spelling` now reads the side read against
/// the pipe again against the ball, the surface `Model::validate` reads the rim
/// against for every rule here.
#[test]
fn a_dome_lying_on_its_side_by_every_rule_against_the_closed_forms() {
    let policy = Policy::certified();
    let third = std::f64::consts::PI / 3.0;
    for radius in 1..=3 {
        for (axis, quarters, along) in [
            (Axis::x(), 1, "-y"),
            (Axis::x(), 3, "+y"),
            (Axis::y(), 1, "+x"),
            (Axis::y(), 3, "-x"),
        ] {
            let (r, h) = (f64::from(radius), f64::from(radius + 1));
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let lying = Isometry::quarter_turns(&axis, quarters).expect("an axis along x or y");
            let shaft = model
                .add_round_bar(r, h, &lying)
                .expect("a whole-number bar");
            let end = lying.apply_to_point(Point::new(0.0, 0.0, h)) - Point::origin();
            let dome = model
                .add_ball(
                    r,
                    &Isometry::translation(end).expect("a whole-number shift"),
                )
                .expect("a whole-number ball");
            let bar = 3.0 * third * r * r * h;
            let half_ball = 2.0 * third * r * r * r;
            for (one, other, rule, holds) in [
                (shaft, dome, Rule::Union, bar + half_ball),
                (dome, shaft, Rule::Union, bar + half_ball),
                (shaft, dome, Rule::Intersection, half_ball),
                (dome, shaft, Rule::Intersection, half_ball),
                (shaft, dome, Rule::Difference, bar - half_ball),
                (dome, shaft, Rule::Difference, half_ball),
            ] {
                let which = format!(
                    "{rule:?} of R = {r} along {along} (shaft first: {})",
                    one == shaft
                );
                let mut fused = model.clone();
                let answer = fused
                    .fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| panic!("{which} refused {why}"));
                fused
                    .validate(&policy, &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| panic!("{which} does not validate: {why}"));
                assert_eq!(answer.solids().len(), 1, "{which} is one solid");
                let (volume, error) = volume_and_error(&fused, answer.solids());
                assert!(
                    (volume - holds).abs() <= error,
                    "{which} measures {volume} against {holds}, out by more than the {error} its \
                     mesh declares"
                );
            }
        }
    }
}

/// A bar of a bend's tube radius butted onto the bend's `+x` cap, its far end
/// flat — pinned at **every rule both ways round refused as a quartic**.
///
/// A fuse with a ring in it crosses every surface of the other solid with the
/// ring, wherever the two faces are. So a bar with a flat far end is refused
/// `ACrossingBetweenAPlaneAndATorusThatIsAQuartic` for that end's plane, which
/// lies square to the cap and off the ring's axis however far from the bend it
/// is. Queue row Q1143 built it; this fails the day it moves.
#[test]
fn a_flat_ended_bar_on_a_bends_cap_is_refused_as_a_quartic() {
    let policy = Policy::certified();
    for (ring, tube) in [(3, 1), (5, 2)] {
        let (mut model, bend, along_minus_y) = a_bend_and_where_its_cap_faces(ring, tube);
        let (ring, tube) = (f64::from(ring), f64::from(tube));
        let bar = model
            .add_round_bar(tube, 2.0, &along_minus_y)
            .expect("a whole-number bar");
        let wanted = rcad_geom::Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic;
        for (one, other) in [(bar, bend), (bend, bar)] {
            for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                let refused = model
                    .clone()
                    .fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new())
                    .map(|answer| answer.solids().len());
                assert!(
                    matches!(&refused, Err(Error::Geometry(why)) if *why == wanted),
                    "{rule:?} of ring {ring}, tube {tube} (bar first: {}) came back \
                     {refused:?}, not {wanted:?}",
                    one == bar
                );
            }
        }
    }
}

/// A bar of a bend's tube radius butted onto the bend's `+x` cap, its far end
/// cut by the plane `x + y = 0` through the ring's axis: **every rule both ways
/// round, certified and tolerant, answers the closed forms and validates.**
///
/// The bend of ring `R` and tube `r` runs round `z` from its `+x` cap on `y = 0`
/// to its `+y` cap, and holds `π²Rr²/2`. The bar's axis runs along `−y` through
/// `(R, 0, 0)`, the middle of the cap, and the bar holds `−x ≤ y ≤ 0` over the
/// disk `(x − R)² + z² ≤ r²`, so `∫∫ x dA = πr²R`. The two meet only on the cap,
/// the bend on `y ≥ 0` and the bar on `y ≤ 0`. So `A ∪ B` is one solid of
/// `πr²R + π²Rr²/2`, `A ∩ B` is no solid, `bar \ bend` is the bar and
/// `bend \ bar` is the bend: for ring 3/1, `3π = 9.425` and `1.5π² = 14.80`; for
/// ring 5/2, `20π = 62.83` and `10π² = 98.70`.
///
/// **The bar's pipe meets the ring in more than the cap's rim.** On the pipe,
/// `(x − R)² + z² = r²`, and on the ring `(√(x² + y²) − R)² + z² = r²`, so
/// `√(x² + y²) − R = ±(x − R)`. The `+` sign is the plane `y = 0`, where the two
/// are tangent along the rim. The `−` sign is a loop, `x = R − y²/4R` and
/// `z² = r² − y⁴/16R²`, which crosses the rim at `(R, 0, ±r)`: `(9, 12, 4)` is
/// on both ring 12/5 and the radius-5 pipe through `(12, 0, 0)` along `y`. Its
/// half with `y > 0` is on the bend's ring face, past the bar's cap, and its
/// half with `y < 0` on the bar's pipe face, past the bend's cap, so no edge of
/// an answer runs along it. Until queue row Q1453 the pipe and the ring were
/// refused `ACrossingBetweenATorusAndAnotherCurvedSurface` under every rule,
/// and until Q1470 `ACrossingThisCannotBeOrderedAlong` along that loop.
#[test]
fn a_bar_cut_through_the_rings_axis_on_a_bends_cap_by_every_rule_against_the_closed_forms() {
    for (ring, tube) in [(3, 1), (5, 2)] {
        let (mut model, bend, along_minus_y) = a_bend_and_where_its_cap_faces(ring, tube);
        let (ring, tube) = (f64::from(ring), f64::from(tube));
        let bar = model
            .add_bar_cut_off_by(
                tube,
                [
                    Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
                        .expect("a unit normal"),
                    Plane::from_point_and_normal(
                        Point::new(0.0, 0.0, ring),
                        Vector::new(-1.0, 0.0, 1.0),
                    )
                    .expect("a normal with a length"),
                ],
                &along_minus_y,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a bar whose far end lies on the plane x + y = 0");
        let pi = std::f64::consts::PI;
        let bar_holds = pi * tube * tube * ring;
        let bend_holds = pi * pi * ring * tube * tube / 2.0;
        for (name, policy) in [
            ("certified", Policy::certified()),
            ("tolerant", Policy::tolerant(model.scale().nominal_size())),
        ] {
            for (one, other, rule, solids, holds) in [
                (bar, bend, Rule::Union, 1, bar_holds + bend_holds),
                (bend, bar, Rule::Union, 1, bar_holds + bend_holds),
                (bar, bend, Rule::Intersection, 0, 0.0),
                (bend, bar, Rule::Intersection, 0, 0.0),
                (bar, bend, Rule::Difference, 1, bar_holds),
                (bend, bar, Rule::Difference, 1, bend_holds),
            ] {
                let which = format!(
                    "{rule:?} of ring {ring}, tube {tube}, {name} (bar first: {})",
                    one == bar
                );
                let mut fused = model.clone();
                let answer = fused
                    .fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| panic!("{which} refused {why}"));
                fused
                    .validate(&policy, &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| panic!("{which} does not validate: {why}"));
                assert_eq!(
                    answer.solids().len(),
                    solids,
                    "{which} keeps {solids} solids"
                );
                let (volume, error) = volume_and_error(&fused, answer.solids());
                assert!(
                    (volume - holds).abs() <= error,
                    "{which} measures {volume} against {holds}, out by more than the {error} its \
                     mesh declares"
                );
            }
        }
    }
}

/// The quarter bend of `touching`'s cap-against-a-block control row, with the
/// block clear of it that nothing here fuses, and the placing that stands a bar
/// on the middle of the bend's `+x` cap running along `−y`.
fn a_bend_and_where_its_cap_faces(ring: u32, tube: u32) -> (Model, SolidKey, Isometry) {
    let built = Drawn {
        shape: Shape::ABendsCapAgainstABlock { ring, tube },
        apart: Apart::ClearByOne,
    }
    .build();
    let along_minus_y = Isometry::translation(Vector::new(f64::from(ring), 0.0, 0.0))
        .expect("a whole-number shift")
        .compose(&Isometry::quarter_turns(&Axis::x(), 1).expect("x is an axis"));
    (built.model, built.second, along_minus_y)
}

#[test]
fn a_ball_against_a_bars_wall() {
    every_rule_over(
        "a ball against a bar's wall",
        touching::a_ball_against_a_bar(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn a_ball_inside_a_bar_touching_its_wall() {
    every_rule_over(
        "a ball inside a bar, touching",
        touching::a_ball_inside_a_bar(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn a_pointed_cone_standing_on_its_apex_on_a_block() {
    every_rule_over(
        "a cone's apex on a block",
        touching::a_cones_apex_on_a_block(),
        Expect::TheControlAnswers,
    );
}

/// A pointed cone of slope a half and height two, its apex on the top of a
/// 4-block whose walls `x = −1` and `y = 1` are tangent to the cone's rim
/// circle at `(−1, 0, 2)` and `(0, 1, 2)`, far above the block.
///
/// The wall planes cross the cone's surface in curves that touch the rim at
/// those two points, so the rim is cut there as well as at its own one corner
/// `(1, 0, 2)`. Joining the cap's pieces back into one face then walked
/// straight through all three corners, and the rim came back as one edge with
/// no ends: a panic in `fuse/assemble.rs`, *"a run of edges that became one
/// edge has two ends left"*, found by the property above from seed
/// `9bb0dbcd…3910` once the cone's box reached its apex (§202.3).
///
/// The inputs meet only at the apex, so every answer is known without the
/// fuse: the union is both solids as they stand, the intersection is nothing,
/// and each difference is its first solid. Checked by solid count, by how many
/// faces each solid keeps, and by where points fall against the two closed
/// forms.
#[test]
fn a_pointed_cone_on_its_apex_whose_rim_touches_two_walls_fuses_by_every_rule() {
    let built = Drawn {
        shape: Shape::AConesApexOnABlock {
            slope: Slope::Half,
            at: [1, 3],
        },
        apart: Apart::Touching,
    }
    .build();
    let (block, cone) = (built.first, built.second);
    let in_block = |p: Point| {
        let [x, y, z] = p.coordinates();
        (-1.0..=3.0).contains(&x) && (-3.0..=1.0).contains(&y) && (-4.0..=0.0).contains(&z)
    };
    let in_cone = |p: Point| {
        let [x, y, z] = p.coordinates();
        (0.0..=2.0).contains(&z) && x.hypot(y) <= 0.5 * z
    };
    let samples = [
        Point::new(0.1, 0.07, 1.03),
        Point::new(0.3, -0.2, 1.9),
        Point::new(0.5, -1.5, -2.0),
        Point::new(2.9, 0.9, -0.1),
        Point::new(0.9, 0.9, 1.9),
        Point::new(-0.9, 0.3, 1.5),
        Point::new(0.2, 0.1, 0.1),
        Point::new(3.5, 0.0, -1.0),
    ];
    let union: fn(bool, bool) -> bool = |a, b| a || b;
    let intersection: fn(bool, bool) -> bool = |a, b| a && b;
    let difference: fn(bool, bool) -> bool = |a, b| a && !b;
    for (one, other, rule, solids, faces, holds) in [
        (block, cone, Rule::Union, 2, vec![2, 6], union),
        (cone, block, Rule::Union, 2, vec![2, 6], union),
        (block, cone, Rule::Intersection, 0, vec![], intersection),
        (cone, block, Rule::Intersection, 0, vec![], intersection),
        (block, cone, Rule::Difference, 1, vec![6], difference),
        (cone, block, Rule::Difference, 1, vec![2], difference),
    ] {
        let mut model = built.model.clone();
        let policy = Policy::certified();
        let answer = model
            .fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{rule:?} refused: {why}"));
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{rule:?} does not validate: {why}"));
        let pieces = answer.solids().to_vec();
        assert_eq!(pieces.len(), solids, "{rule:?}: how many solids");
        let mut got: Vec<usize> = pieces
            .iter()
            .map(|&piece| model.solid(piece).expect("an answer's key").faces().count())
            .collect();
        got.sort_unstable();
        assert_eq!(got, faces, "{rule:?}: how many faces each solid has");

        let first_is_block = one == block;
        for p in samples {
            let (a, b) = if first_is_block {
                (in_block(p), in_cone(p))
            } else {
                (in_cone(p), in_block(p))
            };
            let want = holds(a, b);
            let inside = pieces.iter().any(|&piece| {
                match model.solid(piece).expect("an answer's key").classify_point(
                    p,
                    &policy,
                    &mut ProvenanceLog::new(),
                ) {
                    Containment::Inside => true,
                    Containment::Outside => false,
                    said @ (Containment::OnBoundary
                    | Containment::Undecidable { .. }
                    | Containment::CannotFollowAFacesTrimming { .. }
                    | Containment::CannotFireARayAtAFacesSurface { .. }) => {
                        panic!("{rule:?}: {p:?} is clear of every face and came back {said:?}")
                    }
                    other => panic!("{rule:?}: {p:?} came back {other:?}"),
                }
            });
            assert_eq!(
                inside, want,
                "{rule:?} (block first: {first_is_block}) at {p:?}"
            );
        }
    }
}

/// Two pointed cones base to base on one circle, their rim vertices half a turn
/// apart — the shape that would hand §204.1's join a run of pieces carrying two
/// input edges. **Every rule answers, both ways round.**
///
/// §204.1 joins a closed curve that was cut into pieces back into one edge only
/// **when every piece came from one input edge**. A run whose pieces are an
/// edge of *both* solids is the nearest way past that: cut the shared closed
/// edge at each solid's own vertex and every piece carries two input edges, so
/// the join keeps no corner and `assemble.rs` panics with *"a run of edges that
/// became one edge has two ends left"* the way it did before §204.1.
///
/// **Both cut points have to have nothing else leaving them**, and a rim vertex
/// has something else leaving it wherever the face inside the rim carries a
/// seam ending there — a bar, a frustum, a ball and a bend all do, so one of
/// the two cut points stays a corner and the run keeps it. A pointed cone's
/// side is the one face here with a closed wire and no seam, so both solids
/// have to be pointed cones on one circle, which is two cone faces meeting one
/// curve. The same family with a sphere in place of the second cone is
/// [`a_ball_on_a_bar_of_the_same_radius`].
///
/// **That curve used to be refused by name and is named now** (Q1122): two
/// cones of one slope on one axis, apexes four apart, cross in the one circle
/// halfway between them, which is this rim. The fuse then gets past it, and
/// what it gives is measured here rather than predicted: the two lumps touch
/// along a curve and nowhere else, so their intersection holds nothing, each
/// difference is the first cone, `8π/3`, unchanged, and the union is one solid
/// of both, `16π/3`. A second pair whose rim vertices are **one** point is
/// asked the same, so the half turn is not what it turns on.
///
/// **The union used to stop** in `fuse/assemble.rs` with
/// `ResultIsNotManifold { count: 1 }` — no panic, so the join §204.1 feared
/// did not turn this into a crash. Both caps lie flush on one plane and go, so
/// the rim comes out of each cone named by its cap's plane and that cone: two
/// names, two edges with one face each. Queue row Q1118 gave a rim that one
/// plane cuts two cones in, a ball or a pipe in one name and one corner.
#[test]
fn two_cones_sharing_one_rim_circle_fuse_by_every_rule() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let lower = a_pointed_cone_with_its_vertex_at(
        &mut model,
        Point::new(0.0, 0.0, 0.0),
        2.0,
        Point::new(2.0, 0.0, 2.0),
    );
    let half_a_turn_round = a_pointed_cone_with_its_vertex_at(
        &mut model,
        Point::new(0.0, 0.0, 4.0),
        -2.0,
        Point::new(-2.0, 0.0, 2.0),
    );
    let at_one_point = a_pointed_cone_with_its_vertex_at(
        &mut model,
        Point::new(0.0, 0.0, 4.0),
        -2.0,
        Point::new(2.0, 0.0, 2.0),
    );
    let one_cone = 8.0 * std::f64::consts::PI / 3.0;
    let policy = Policy::certified();
    for upper in [half_a_turn_round, at_one_point] {
        for (one, other) in [(lower, upper), (upper, lower)] {
            let fused = |rule: Rule| {
                let mut fused = model.clone();
                let answer = fused
                    .fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new())
                    .map(|answer| answer.solids().to_vec());
                (fused, answer)
            };

            let (joined, union) = fused(Rule::Union);
            let solids = union.unwrap_or_else(|why| panic!("the union: {why}"));
            assert_eq!(solids.len(), 1, "{solids:?}");
            joined
                .validate(&policy, &mut ProvenanceLog::new())
                .expect("a fuse hands back a sound model");
            let (volume, error) = volume_and_error(&joined, &solids);
            assert!(
                (volume - 2.0 * one_cone).abs() <= error,
                "the two cones hold {} and their union holds {volume} give or take {error}",
                2.0 * one_cone
            );

            let (_, intersection) = fused(Rule::Intersection);
            assert!(
                intersection.as_ref().is_ok_and(Vec::is_empty),
                "two cones touching along a rim share no volume: {intersection:?}"
            );

            let (left, difference) = fused(Rule::Difference);
            let solids = difference.unwrap_or_else(|why| panic!("the difference: {why}"));
            assert_eq!(solids.len(), 1, "{solids:?}");
            left.validate(&policy, &mut ProvenanceLog::new())
                .expect("a fuse hands back a sound model");
            let (volume, error) = volume_and_error(&left, &solids);
            assert!(
                (volume - one_cone).abs() <= error,
                "the first cone holds {one_cone} and its difference holds {volume} give or take \
                 {error}"
            );
        }
    }
}

/// A pointed cone of slope one, its apex at `apex`, its base cap `far` up the
/// `z` axis from there — `far` may point down — and its one vertex wherever
/// `rim_at` says on the rim circle.
fn a_pointed_cone_with_its_vertex_at(
    model: &mut Model,
    apex: Point,
    far: f64,
    rim_at: Point,
) -> SolidKey {
    use rcad_geom::{Axis, Cone, EdgeRun, Orientation, Plane, Surface};
    use rcad_topo::{DraftStep, SolidBuilder};

    let up = Vector::new(0.0, 0.0, 1.0);
    let skin = Cone::new(Axis::new(apex, up).expect("z is a direction"), 1.0).expect("a slope");
    let mut builder = SolidBuilder::new();

    let rim = builder.add_vertex(rim_at).expect("a finite point");
    let circle = builder
        .add_edge(rim, rim, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the cap plane cuts the cone in one closed curve");

    let [outward, inward] = if far > 0.0 {
        [Orientation::Forward, Orientation::Reversed]
    } else {
        [Orientation::Reversed, Orientation::Forward]
    };
    let at = apex + up * far;
    let plane = Plane::from_point_and_normal(at, up * far.signum()).expect("a unit normal");
    let cap_coedge = builder.add_coedge(circle, outward).expect("an edge built");
    let cap_wire = builder
        .add_wire(&[DraftStep::Along(cap_coedge)])
        .expect("one closed edge is a wire");
    let cap = builder
        .add_face(Surface::Plane(plane), &[cap_wire])
        .expect("a cap is a plane trimmed by its circle");

    let side_coedge = builder.add_coedge(circle, inward).expect("an edge built");
    let side_wire = builder
        .add_wire(&[DraftStep::Along(side_coedge)])
        .expect("one closed edge is a wire");
    let side = builder
        .add_face(Surface::Cone(skin), &[side_wire])
        .expect("a face on the cone, bounded by its base circle and pinched at the apex");

    builder
        .add_shell(&[(cap, Orientation::Forward), (side, Orientation::Forward)])
        .expect("two faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a pointed cone described this way is a solid")
}

#[test]
fn a_frustums_cap_flush_on_a_block() {
    every_rule_over(
        "a frustum's cap flush on a block",
        touching::a_frustums_cap_on_a_block(),
        Expect::TheControlAnswers,
    );
}

/// Pinned at nothing answered, touch and control alike: every fuse in both
/// rows is refused `ACrossingBetweenAPlaneAndATorusThatIsAQuartic`, so the
/// touch is never reached. This fails the day that changes, which is the day
/// this cell starts measuring its tangency.
#[test]
fn a_bend_of_pipe_lying_on_a_block() {
    every_rule_over(
        "a bend lying on a block",
        touching::a_bend_lying_on_a_block(),
        Expect::NothingAnswers,
    );
}

/// A bend's cap flush against a block, and the block slid one unit clear along
/// the cap's plane.
///
/// The touching row refuses every fuse as a quartic: the block touches the
/// bend, so the two are cut against each other, and three of the block's walls
/// run along the bend's axis and off it. The control's block is clear of the
/// bend, so nothing is cut, and since Q326 no fuse asks whether a block's wall
/// is the bend's ring (`do_they_face_the_same_way` in `fuse/select.rs`). Until
/// then that question was the quartic, and the control answered nothing.
///
/// Now the control answers all six fuses: both intersections are nothing, the
/// block less the bend is the whole block, its volume exact, and the union both
/// ways round and the bend less the block keep the bend. Those three were
/// refused writing it back, `TheFuseWroteABrokenAnswer` with
/// `CoedgesRunTheSameWay`, where a bend fused with a copy of itself stopped
/// (§243.5, §323.5), until the bend's wall was used forwards (§361).
#[test]
fn a_bends_cap_flush_against_a_block() {
    every_rule_over(
        "a bend's cap flush against a block",
        touching::a_bends_cap_against_a_block(),
        Expect::TheControlAnswers,
    );
}

/// A thin bar standing wholly inside a thick one, touching nothing, is united
/// and hollowed out without a panic.
///
/// Found by the control row of
/// [`a_bar_inside_a_bar_touching_its_wall`] and shrunk to this: a bar of radius
/// one and height two standing on the axis of a bar of radius two and height
/// four, one unit up, so every face of the thin bar is clear of every face of
/// the thick one. Off the axis it panics the same way — radius three with the
/// thin bar at `(0, 1)`, `(1, 0)` or `(−1, 0)`. `A ∩ B` and `B \ A` answer;
/// `A ∪ B` and `A \ B` panic in `fuse/assemble.rs` with *"a ring of a joined
/// face came back winding neither way: Zero"*, both ways round for the union.
///
/// The answer is plain: the union is the thick bar and the difference is the
/// thick bar with a bar-shaped hole sealed inside it. A ball standing inside
/// the same bar is answered under every rule, so it is the two pipes and not
/// the hole.
#[test]
#[ignore = "open: §191, a bar wholly inside a thicker bar panics under union and difference"]
fn a_bar_standing_wholly_inside_a_thicker_bar_is_united_and_hollowed_without_a_panic() {
    for rule in [Rule::Union, Rule::Difference] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let thick = model
            .add_round_bar(2.0, 4.0, &Isometry::identity())
            .expect("a bar of radius two");
        let thin = model
            .add_round_bar(
                1.0,
                2.0,
                &Isometry::translation(Vector::new(0.0, 0.0, 1.0)).expect("a whole-number shift"),
            )
            .expect("a bar of radius one");
        let answer = model
            .fuse(
                &[thick],
                &[thin],
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{rule:?} of a bar inside a bar refused: {why}"));
        assert_eq!(answer.solids().len(), 1, "{rule:?} is one solid");
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{rule:?} left a model that does not validate: {why}"));
    }
}

/// A ball of radius two resting on a block it hangs over the edge of unites
/// into the two solids as they stand, measured against the cube's volume and
/// the ball's.
///
/// A cube of side two and a ball of radius two whose south pole rests on the
/// middle of the cube's top, at `(1, 1, 2)`. Every number is whole and the touch
/// is exact. The union was refused all along until Q508, a **gap** (§16.10)
/// whose name moved three times: `ResultIsNotManifold` first, in assembly,
/// where the two arcs of one ball-and-plane circle between the same two corners
/// were given one edge name, until §198 told them apart; then
/// `AWireWillNotLieFlatOnItsFace`, out of the mesher, until §208.4 let a ring of
/// arcs be walked; then `ACurvedFaceGoesRightRoundWithNoSeam`, until Q508
/// changed how the mesher reads which pole a cap of the ball reaches
/// (`which_pole_this_cap_reaches` in `mesh.rs`, the one function Q508 changed).
///
/// **The ball moved over to `(1, 2)` on a cube of side four was refused too, and
/// answers since §217.** It was `Geometry(NotFinite)` — a coordinate worked out
/// as infinity or not a number out of whole-number input, raised in
/// `Arrangement::the_whole_surface` and pinned then as measured rather than as
/// understood. What moved it is the only thing §217 changed here: the fuse now
/// writes the surface that cut a closed rim. That block's wall plane cuts the
/// ball in a circle whose two faces are the one sphere, and a rim whose two
/// faces name one surface is exactly what the new run exists for. **Why that
/// makes this particular coordinate finite was not chased.**
///
/// Rows that refused once and answer now are the cheapest guard there is
/// against their refusing again, so they are kept beside the rows that always
/// answered: the same ball **one unit higher**, a ball of **radius one** resting
/// on the same point, and the ball resting on a cube of **side four** where it
/// no longer hangs over an edge. In each a side wall's plane crosses the ball
/// or not while the top touches it; a fuse crosses planes and not faces, so a
/// wall nowhere near the ball still cuts it.
#[test]
fn a_ball_resting_on_a_block_it_overhangs_unites_into_both_solids_as_they_stand() {
    let unites_into_both = |side: f64, centre: [f64; 3], radius: f64| {
        let case = format!("a ball of {radius} at {centre:?} on a cube of {side}");
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let cube = model
            .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a whole-number cube");
        let [x, y, z] = centre;
        let ball = model
            .add_ball(
                radius,
                &Isometry::translation(Vector::new(x, y, z)).expect("a whole-number shift"),
            )
            .expect("a whole-number ball");
        let united = model
            .fuse(
                &[cube],
                &[ball],
                Rule::Union,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{case}: refused {}", named(&why)));
        let solids = united.solids().to_vec();
        assert_eq!(solids.len(), 2, "{case}: two solids that touch at a point");
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{case}: the union does not validate: {why}"));
        let (volume, error) = volume_and_error(&model, &solids);
        let both = side.powi(3) + 4.0 / 3.0 * std::f64::consts::PI * radius.powi(3);
        assert!(
            (volume - both).abs() <= error,
            "{case}: the union measures {volume} against {both}, out by more than the {error} \
             its measurements declare"
        );
    };

    unites_into_both(2.0, [1.0, 1.0, 4.0], 2.0);
    unites_into_both(4.0, [1.0, 2.0, 6.0], 2.0);
    unites_into_both(2.0, [1.0, 1.0, 5.0], 2.0);
    unites_into_both(4.0, [1.0, 2.0, 7.0], 2.0);
    unites_into_both(2.0, [1.0, 1.0, 3.0], 1.0);
    unites_into_both(4.0, [1.0, 2.0, 5.0], 1.0);
    unites_into_both(4.0, [2.0, 2.0, 6.0], 2.0);
}

/// A ball of radius two resting on the block `[0, 3] × [0, 2] × [0, 1]` at
/// `(1, 1, 3)`, and two more of the same family, answer every rule both ways
/// round: the union is the two solids as they stand, `6 + 32π/3`, the
/// intersection is nothing, and each difference is its first solid.
///
/// The fuse cuts the ball along the planes of the block's walls and not only
/// where the walls are, so `x = 0`, `y = 0` and `y = 2`, each a unit from its
/// centre, cut circles of radius `√3` in it that cross one another, and `x = 3`
/// touches it at `(3, 1, 3)` the way the top touches its south pole. The
/// piece of the ball's skin that holds both poles is the ball less those three
/// caps, bounded by **one rim of ten arcs** — not one closed curve, and holding
/// the material outside it in the mesher's chart (it encloses `−7.63` there).
/// Until Q451 the mesher opened a ball's skin along a seam only when its rim was
/// one closed curve; this one fell through to the paths below that one, which
/// Q451's own commit message says filled such a rim inside out, and the check
/// of the fuse's own answer refused union both ways and the ball less the block
/// `Invalid(FacesDoNotAgreeWhichWayTheyFace)`. Measured on `a7ad3f4b`,
/// where every shape below refused those three fuses; a bisect of the first
/// shape to `d0eb84cc` names `75a593bd` the first that answers. Its change is
/// the `it_passes_each_of_its_points_once` arm of the first test in
/// `cut_off_what_goes_too_far_round` in `mesh.rs`, and with that arm switched
/// off this test refuses the first union the same way again (queue row Q892).
///
/// The three shapes: the row's own; a ball of radius three at `(2, 1)` on the
/// block `[0, 4] × [0, 4] × [0, 2]`, cut by the planes `x = 0`, `x = 4` and
/// `y = 0` and touched by `y = 4`; and a ball of radius two at `(3, 3)` on that
/// block, cut by `x = 4` and `y = 4` only. The ten arcs and the `−7.63` were
/// printed for the first; the other two were only seen to refuse the same three
/// fuses on `a7ad3f4b` and to answer here.
#[test]
fn a_ball_whose_skin_the_blocks_walls_cut_into_a_rim_of_arcs_fuses_by_every_rule() {
    for (size, at, radius) in [
        ([3, 2, 1], [1, 1], 2),
        ([4, 4, 2], [2, 1], 3),
        ([4, 4, 2], [3, 3], 2),
    ] {
        a_ball_resting_on_a_block_fuses_by_every_rule(size, at, radius);
    }
}

/// A ball of radius four at `(1, 1, 5)` resting on the block
/// `[0, 4] × [0, 4] × [0, 1]`, and four more of the same family, answer every
/// rule both ways round: the union is the two solids as they stand,
/// `16 + 256π/3` for the first, the intersection is nothing, and each
/// difference is its first solid.
///
/// The planes `x = 0` and `y = 0`, a unit from the centre, and `x = 4` and
/// `y = 4`, three units from it, cut the ball's skin into pieces, and one of
/// them, past `x = 0` between `y = 0` and `y = 4`, goes `2.03` round in the
/// mesher's chart, so the mesher cuts it narrower. The lens `y = 4` cuts off it
/// reaches in past the only chord that would, and until Q1099 that chord was
/// kept and the piece refused `CannotMeshAFace` with `BoundaryCrossesItself`,
/// union both ways and the ball less the block. Now the piece is left whole
/// (`cut_in_two` in `mesh.rs`, whose doc has the numbers). A sweep of every
/// whole-number ball of radius one to four resting strictly inside the top of
/// a whole-number block, sides two to six and heights one to three, refused
/// 180 of its 16 200 fuses that way on `3741c5a7`, all of them a ball of
/// radius four a unit in from a wall of a side four long, and answers all of
/// them with the change.
///
/// The five shapes: the row's own; the same ball on the far corner of that
/// block; on a block `6 × 4 × 2`, where no plane reaches the ball's seam;
/// on `4 × 6 × 3` a unit in from its far corner; and on `5 × 4 × 1`. The
/// sweep printed each of them refusing those three fuses on `3741c5a7`.
#[test]
fn a_ball_whose_skin_the_walls_cut_into_a_piece_no_straight_cut_narrows_fuses_by_every_rule() {
    for (size, at) in [
        ([4, 4, 1], [1, 1]),
        ([4, 4, 1], [3, 3]),
        ([6, 4, 2], [1, 1]),
        ([4, 6, 3], [3, 5]),
        ([5, 4, 1], [4, 3]),
    ] {
        a_ball_resting_on_a_block_fuses_by_every_rule(size, at, 4);
    }
}

/// A ball of `radius` resting at `at` on the top of the block from the origin
/// to `size`, fused with it by every rule both ways round, each answer checked
/// against the closed form: the union is the two as they stand, the
/// intersection is nothing, and each difference is its first solid.
fn a_ball_resting_on_a_block_fuses_by_every_rule(size: [i32; 3], at: [i32; 2], radius: i32) {
    let case = format!("a ball of {radius} at {at:?} on a block of {size:?}");
    let [x, y, z] = size.map(f64::from);
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [x, 0.0], [x, y], [0.0, y]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let block = model
        .add_extrusion(&rectangle, z, &Isometry::identity())
        .expect("building a box needs no decision");
    let r = f64::from(radius);
    let [across, along] = at.map(f64::from);
    let ball = model
        .add_ball(
            r,
            &Isometry::translation(Vector::new(across, along, z + r))
                .expect("a whole-number shift"),
        )
        .expect("a whole-number ball");
    let block_holds = x * y * z;
    let ball_holds = 4.0 / 3.0 * std::f64::consts::PI * r.powi(3);
    for (one, other, rule, solids, holds) in [
        (block, ball, Rule::Union, 2, block_holds + ball_holds),
        (ball, block, Rule::Union, 2, block_holds + ball_holds),
        (block, ball, Rule::Intersection, 0, 0.0),
        (ball, block, Rule::Intersection, 0, 0.0),
        (block, ball, Rule::Difference, 1, block_holds),
        (ball, block, Rule::Difference, 1, ball_holds),
    ] {
        let held = match fused(&model, one, other, rule) {
            Ok(Outcome::Answered(held)) => held,
            Ok(Outcome::Refused(why)) => panic!("{case}: {rule:?} refused {why}"),
            Err(why) => panic!("{case}: {rule:?} {why}"),
        };
        let block_first = one == block;
        assert_eq!(
            held.solids, solids,
            "{case}: {rule:?} (block first: {block_first}), how many solids"
        );
        assert!(
            (held.volume - holds).abs() <= held.error,
            "{case}: {rule:?} (block first: {block_first}) measures {} against {holds}, \
             out by more than the {} its measurements declare",
            held.volume,
            held.error
        );
    }
}

/// What a cell's rows must have done for the run to have measured anything.
enum Expect {
    /// The control answered and at least one law was checked on it, so the
    /// oracle ran on this cell.
    TheControlAnswers,
    /// No fuse answered in either row: the cell is pinned at a refusal that
    /// comes before its touch.
    NothingAnswers,
}

fn every_rule_over(cell: &str, shapes: impl Strategy<Value = Shape>, expect: Expect) {
    let tally = Tally::default();
    // From one fixed seed, so the table at the top of this file is what a run
    // prints rather than one draw of it.
    let mut runner = TestRunner::new_with_rng(
        Config {
            cases: cases_to_draw(),
            source_file: Some(file!()),
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
                "proptest-regressions",
            ))),
            ..Config::default()
        },
        seeded(),
    );
    if let Err(why) = runner.run(&both_rows(shapes), |drawn| every_rule_of(&drawn, &tally)) {
        panic!("{cell}: {why}");
    }
    tally.report(cell);
    match expect {
        Expect::TheControlAnswers => assert!(
            tally.laws_checked(Apart::ClearByOne) > 0,
            "{cell}: not one law was checked on the control, so nothing was compared"
        ),
        Expect::NothingAnswers => assert_eq!(
            tally.answered.get(),
            0,
            "{cell}: a fuse answered, so this cell reaches its touch now and should be \
             measured rather than pinned"
        ),
    }
}

/// [`CASES`], multiplied by whatever [`SOAK_FACTOR`] says.
///
/// # Panics
///
/// If the variable is not a whole number above zero, or if the product is more
/// than a run can count. A soak that silently ran the shipped corpus while the
/// operator read the number as the wide one is worse than one that stopped
/// (§48.2, §50.9).
fn cases_to_draw() -> u32 {
    let Ok(factor) = std::env::var(SOAK_FACTOR) else {
        return CASES;
    };
    let factor: u32 = factor
        .parse()
        .unwrap_or_else(|_| panic!("{SOAK_FACTOR} has to be a whole number, not {factor:?}"));
    assert!(factor > 0, "{SOAK_FACTOR} has to be at least one");
    CASES
        .checked_mul(factor)
        .unwrap_or_else(|| panic!("{CASES} cases times {factor} is more than one run can count"))
}

/// The fixed seed, or the one [`SOAK_SEED`] names.
///
/// # Panics
///
/// If the variable is set to something that is not a whole number.
fn seeded() -> TestRng {
    let Ok(seed) = std::env::var(SOAK_SEED) else {
        return TestRng::deterministic_rng(RngAlgorithm::ChaCha);
    };
    let seed: u64 = seed
        .parse()
        .unwrap_or_else(|_| panic!("{SOAK_SEED} has to be a whole number, not {seed:?}"));
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    TestRng::from_seed(RngAlgorithm::ChaCha, &bytes)
}

/// Every rule both ways round, and every law that has both its sides.
fn every_rule_of(drawn: &Drawn, tally: &Tally) -> Result<(), TestCaseError> {
    let built = drawn.build();
    let (a, b) = (built.first, built.second);
    let apart = drawn.apart;
    tally.drawn(apart);

    let mut got = BTreeMap::new();
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for (one, other, way) in [(a, b, Way::Forwards), (b, a, Way::Backwards)] {
            let outcome = fused(&built.model, one, other, rule)?;
            tally.outcome(apart, rule, &outcome);
            if let Outcome::Answered(held) = outcome {
                got.insert((name_of(rule), way), held);
            }
        }
    }

    let [first, second] = [a, b].map(|solid| how_much_it_holds(&built.model, &[solid]));
    let answer = |rule, way| got.get(&(name_of(rule), way));

    if let (Some(united), Some(shared)) = (
        answer(Rule::Union, Way::Forwards),
        answer(Rule::Intersection, Way::Forwards),
    ) {
        tally.law(apart);
        adds_up(
            &[united, shared],
            &[&first, &second],
            "A ∪ B + A ∩ B = A + B",
        )?;
    }
    if let (Some(left), Some(shared)) = (
        answer(Rule::Difference, Way::Forwards),
        answer(Rule::Intersection, Way::Forwards),
    ) {
        tally.law(apart);
        adds_up(&[left, shared], &[&first], "A \\ B + A ∩ B = A")?;
    }
    if let (Some(left), Some(shared)) = (
        answer(Rule::Difference, Way::Backwards),
        answer(Rule::Intersection, Way::Forwards),
    ) {
        tally.law(apart);
        adds_up(&[left, shared], &[&second], "B \\ A + A ∩ B = B")?;
    }

    for rule in [Rule::Union, Rule::Intersection] {
        if let (Some(forwards), Some(backwards)) =
            (answer(rule, Way::Forwards), answer(rule, Way::Backwards))
        {
            tally.law(apart);
            prop_assert_eq!(
                forwards.solids,
                backwards.solids,
                "{:?} of {:?} gave {} solids one way round and {} the other",
                rule,
                drawn,
                forwards.solids,
                backwards.solids
            );
            let between = (forwards.volume - backwards.volume).abs();
            prop_assert!(
                between <= forwards.error + backwards.error,
                "{rule:?} of {drawn:?} holds {} one way round and {} the other, further apart \
                 than the {} the two meshes declare between them",
                forwards.volume,
                backwards.volume,
                forwards.error + backwards.error
            );
        }
    }

    let nothing_left = match built.insides {
        Insides::Apart => vec![
            (Rule::Intersection, Way::Forwards),
            (Rule::Intersection, Way::Backwards),
        ],
        Insides::SecondInsideFirst => vec![(Rule::Difference, Way::Backwards)],
        Insides::Overlap => vec![],
    };
    for (rule, way) in nothing_left {
        if let Some(held) = answer(rule, way) {
            tally.law(apart);
            prop_assert_eq!(
                held.solids,
                0,
                "{:?} ({:?}) of {:?} holds no volume and came back with {} solids",
                rule,
                way,
                drawn,
                held.solids
            );
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Way {
    Forwards,
    Backwards,
}

enum Outcome {
    Answered(Held),
    Refused(String),
}

/// How much an answer holds, how much of that its mesh may be hiding, and how
/// many solids it is.
struct Held {
    volume: f64,
    error: f64,
    solids: usize,
}

/// Fuse a copy of `model` under a certified policy.
///
/// An answer must pass `Model::validate` and must have recorded nothing it
/// could not prove. A refusal is a result: it is named and handed back.
fn fused(
    model: &Model,
    one: SolidKey,
    other: SolidKey,
    rule: Rule,
) -> Result<Outcome, TestCaseError> {
    let mut model = model.clone();
    let mut log = ProvenanceLog::new();
    match model.fuse(&[one], &[other], rule, &Policy::certified(), &mut log) {
        Ok(fused) => {
            prop_assert!(
                log.is_empty(),
                "a certified {rule:?} recorded {} decisions it could not prove",
                log.len()
            );
            let solids = fused.solids().to_vec();
            if let Err(why) = model.validate(&Policy::certified(), &mut ProvenanceLog::new()) {
                return Err(TestCaseError::fail(format!(
                    "a {rule:?} answered with a model that does not validate: {why}"
                )));
            }
            let (volume, error) = volume_and_error(&model, &solids);
            Ok(Outcome::Answered(Held {
                volume,
                error,
                solids: solids.len(),
            }))
        }
        Err(why) => Ok(Outcome::Refused(named(&why))),
    }
}

/// An error's variant; for an undecided predicate which ones gave up, and for
/// a broken answer what the sweep refused it for.
fn named(why: &Error) -> String {
    let written = format!("{why:?}");
    let name: String = written.chars().take_while(|c| *c != '{').collect();
    let open = name.matches('(').count() - name.matches(')').count();
    let name = format!("{}{}", name.trim_end(), ")".repeat(open));
    if let Error::TheFuseWroteABrokenAnswer { broken } = why {
        let broken: String = format!("{broken:?}")
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect();
        return format!("{name} ({broken})");
    }
    let Error::Undecidable { records } = why else {
        return name;
    };
    let gave_up: BTreeSet<_> = records.records().iter().map(|r| r.predicate()).collect();
    format!(
        "{name} ({})",
        gave_up.into_iter().collect::<Vec<_>>().join(", ")
    )
}

fn how_much_it_holds(model: &Model, solids: &[SolidKey]) -> Held {
    let (volume, error) = volume_and_error(model, solids);
    Held {
        volume,
        error,
        solids: solids.len(),
    }
}

/// The deflection every volume here is measured at.
///
/// One of the deflections the share below was measured over.
const A_DEFLECTION: f64 = 0.1;

/// How much of what the measurements declare a volume law may be out by.
///
/// **Not a tolerance chosen here.** It is
/// `A_CURVED_LAW_MAY_USE_THIS_MUCH_OF_WHAT_THE_FOUR_DECLARE` in
/// `fuses_that_must_agree.rs`, whose doc says where it comes from: each side of
/// a law is out by at most its own `Measurement::volume_error`, and the worst
/// share of the summed errors that family ever used is 0.025. Copied rather
/// than shared because that file is a test binary and not a module; a change
/// there should be made here too.
const A_LAW_MAY_USE_THIS_MUCH_OF_WHAT_ITS_SIDES_DECLARE: f64 = 0.05;

fn volume_and_error(model: &Model, solids: &[SolidKey]) -> (f64, f64) {
    let deflection = Deflection::of(A_DEFLECTION).expect("a length");
    solids
        .iter()
        .map(|&solid| {
            let measured = model
                .solid(solid)
                .expect("this model made the key")
                .measure(deflection, &Policy::certified(), &mut ProvenanceLog::new())
                .expect("a solid this size measures");
            (measured.volume(), measured.volume_error())
        })
        .fold((0.0, 0.0), |(volume, error), (more, worse)| {
            (volume + more, error + worse)
        })
}

/// What `added` holds together is what `against` holds together.
///
/// When every side is flat the measurements declare nothing and the volumes are
/// whole numbers, so the law holds exactly.
fn adds_up(added: &[&Held], against: &[&Held], law: &str) -> Result<(), TestCaseError> {
    let out_by = added.iter().map(|held| held.volume).sum::<f64>()
        - against.iter().map(|held| held.volume).sum::<f64>();
    let declared: f64 = added.iter().chain(against).map(|held| held.error).sum();
    if declared == 0.0 {
        prop_assert_eq!(
            out_by,
            0.0,
            "{} is out by {} with every side flat",
            law,
            out_by
        );
    } else {
        prop_assert!(
            out_by.abs() < A_LAW_MAY_USE_THIS_MUCH_OF_WHAT_ITS_SIDES_DECLARE * declared,
            "{law} is out by {out_by}, which is {} of the {declared} its sides declare",
            out_by.abs() / declared
        );
    }
    Ok(())
}

fn name_of(rule: Rule) -> &'static str {
    match rule {
        Rule::Union => "union",
        Rule::Intersection => "intersection",
        Rule::Difference => "difference",
        _ => unreachable!("only the three rules above are asked"),
    }
}

/// What each row of a cell came back with.
#[derive(Default)]
struct Tally {
    drawn: RefCell<BTreeMap<Apart, u32>>,
    outcomes: RefCell<BTreeMap<(Apart, String), u32>>,
    laws: RefCell<BTreeMap<Apart, u32>>,
    fuses: Cell<u32>,
    answered: Cell<u32>,
}

impl Tally {
    fn drawn(&self, apart: Apart) {
        *self.drawn.borrow_mut().entry(apart).or_default() += 1;
    }

    fn outcome(&self, apart: Apart, rule: Rule, outcome: &Outcome) {
        self.fuses.set(self.fuses.get() + 1);
        let what = match outcome {
            Outcome::Answered(_) => {
                self.answered.set(self.answered.get() + 1);
                format!("{} answered", name_of(rule))
            }
            Outcome::Refused(why) => format!("{} refused: {why}", name_of(rule)),
        };
        *self.outcomes.borrow_mut().entry((apart, what)).or_default() += 1;
    }

    fn law(&self, apart: Apart) {
        *self.laws.borrow_mut().entry(apart).or_default() += 1;
    }

    fn laws_checked(&self, apart: Apart) -> u32 {
        self.laws.borrow().get(&apart).copied().unwrap_or(0)
    }

    fn report(&self, cell: &str) {
        println!("{cell}: {} fuses", self.fuses.get());
        for apart in [Apart::Touching, Apart::ClearByOne] {
            println!(
                "  {apart:?}: {} shapes, {} laws checked",
                self.drawn.borrow().get(&apart).copied().unwrap_or(0),
                self.laws_checked(apart)
            );
            for ((row, what), count) in self.outcomes.borrow().iter() {
                if *row == apart {
                    println!("    {count:>3}  {what}");
                }
            }
        }
    }
}
