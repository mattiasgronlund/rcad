//! Fuses that have to give the same answer as each other, whatever that answer
//! turns out to be.
//!
//! §1.7's sixth exit criterion and §14.8's three extra oracles, over the
//! configuration families §17.7 asks for and reports separately. Nothing here
//! knows what the right answer to any fuse is; every test runs the same question
//! two ways round and insists the two agree.
//!
//! # What a pass is worth
//!
//! Every property here hands back `Ok(())` the moment a fuse comes back with no
//! answer — refused, or an answer no `Model` can hold — so a green run says
//! nothing on its own. Two things are counted so that it does. Each property
//! reports how many of the comparisons it set out to make it reached and fails
//! below [`SOME_COMPARISONS_MUST_HAPPEN`], the same guard and the same shape as
//! `turning_a_fuse.rs`'s next door. And each measured row judges the share of
//! its fuses that came back not-a-solid against a band, because that outcome is
//! the one the `refused == 0` assertions cannot see and it is what a regression
//! would hide behind.
//!
//! # What "agree" means, and where a tolerance would come from
//!
//! Two correct kernels — and two correct paths through one kernel — produce
//! different but equally valid face partitions, so topological equality is the
//! wrong predicate (roadmap §11.2). Two things are compared here instead:
//!
//! - **Decomposition-invariant measurements**: volume, surface area, centroid
//!   and shell count, per solid of the answer, sorted so the arena's order does
//!   not enter into it. Used where the two runs genuinely may cut the answer up
//!   differently.
//! - **The answer's geometry, bit for bit**: every face's plane, its outward
//!   normal, and its wires as rings of steps — each a corner and what the edge
//!   from it runs along, a closed one with the way it is walked — each ring
//!   turned to its own smallest rotation, the outline kept first and the holes
//!   and faces and solids sorted (§151, §163, §165). Used where the two runs must
//!   produce *the same* partition and not merely the same solid.
//!
//! **Both comparisons are exact. There is no tolerance anywhere in this file**,
//! and that is a property of the generators rather than a lucky break. The
//! lattice and degenerate families are rectilinear on the integer grid, so:
//!
//! - Every plane is axis-aligned with a whole-number offset, and every corner
//!   the fuse invents is the meet of three of them — a Cramer solve with a
//!   determinant of ±1, which lands on a whole number exactly.
//! - `measure` sums `a · (b × c)` over triangles with whole-number corners, so
//!   the six-volume sum is a whole number reached with no rounding at all, and
//!   the volume of a rectilinear solid is a whole number, so dividing by six is
//!   exact too. Surface area is `|(0, 0, ±area)|` per face, and the square root
//!   of the square of a whole number is that whole number. The centroid is one
//!   whole number divided by another, so two runs that agree about the solid
//!   agree about the rounding as well.
//!
//! **The curved families keep that**, which is not obvious and is the whole
//! point of §48. A ball's centre and radius and a bar's axis and radius are
//! whole numbers, where its skin meets the block is a circle of that same
//! radius, and the only corners on that circle are where the seam crosses it —
//! so every corner of those answers is a whole number too, and a whole-number
//! shift of them is exact. The laws that hold over a ball hold bit for bit.
//!
//! **The two rows about a curved volume are the exception, and neither of them
//! is a comparison.** `Solid::measure` is not decomposition-invariant on a
//! curved skin — the mesh of a whole ball is not the mesh of its two halves put
//! together — so inclusion and exclusion over a ball cannot be asserted exactly
//! the way it is over the lattice family.
//!
//! [`what_a_block_and_a_ball_take_up_adds_up_inside_a_stated_share_of_what_they_declare`]
//! asserts it anyway, and what makes that honest is that the four measurements
//! each declare how far out they are. The residual is checked against a
//! **stated share** of the four added up and never against the sum itself: the
//! four charge the same shells twice with opposite signs, so the sum runs two
//! orders of magnitude over the residual and a law asserted at it could not
//! fail. [`A_CURVED_LAW_MAY_USE_THIS_MUCH_OF_WHAT_THE_FOUR_DECLARE`] carries the
//! measurement and the margin.
//! [`what_a_block_and_a_ball_take_up_adds_up_only_as_well_as_the_mesh_does`] is
//! the other half of the same statement, from the other side: the gap closes
//! faster than a hundredfold when the deflection does, which says the residual
//! is the mesh and not the fuse. Neither is used to decide whether two answers
//! agree.
//!
//! The turned family has none of that, so the only properties run over it are
//! the ones that are exactly true whatever the coordinates are: two answers
//! agreeing bit for bit, and a result being empty. **Inclusion and exclusion is
//! deliberately not asserted on turned pairs.** Its four volumes are each a
//! rounded sum over a solid whose corners are already a rounding away from the
//! planes they were solved from, and a bound on the sum of the four needs a
//! bound on how badly conditioned those plane triples are — which nothing here
//! has. §17.6's tenth piece is where that bound gets derived, and it is closing
//! gate work. A tolerance guessed here instead would make the test measure the
//! tolerance (§14.1 question 1).
//!
//! # §14.1's four questions
//!
//! 1. **Exactly true, or needing a bound?** Exactly true, all of it. See above
//!    for what that costs the turned family.
//! 2. **Do the two runs take different paths?** Yes, and that is the point of
//!    picking these particular pairings. Swapping the arguments and the tools
//!    walks the selection table down different rows for the same piece.
//!    `fuse([A, B, C])` decomposes three solids against each other at once while
//!    `fuse([fuse([A, B]), C])` decomposes two, commits, and decomposes again —
//!    the second one's second fuse is looking at faces that no input ever had.
//!    `A \ (A \ B)` runs the difference row twice where `A ∩ B` runs the
//!    intersection row once. A whole-number movement of the input changes every
//!    coordinate the arrangement sorts on without changing which side of
//!    anything anything is.
//!    Writing the same configuration at another power of two sends the same
//!    question down a different level of the cascade, and four solids in a row
//!    hand a fuse an input with properties no generated solid has.
//! 3. **What family of inputs?** Several, reported separately, in
//!    `pairs/mod.rs`, and the two added last answer these four questions for
//!    themselves at their own strategies.
//! 4. **Which gap would this have caught?** §12.3's, twice over. A generator of
//!    one shape family finds one family of bugs, and `random_extrusions` threw
//!    away every input the builder refused, so a wrongly refused input was
//!    invisible. Nothing here filters: every fuse is counted, refusals are
//!    counted separately from answers, and on the exactly representable
//!    families a refusal fails the test.
//!
//! # §17.1, checked rather than argued
//!
//! Every fuse in this file, of every family, also runs
//! [`every_surface_of_the_answer_is_a_surface_of_an_input`], which is the deciding
//! argument for plane-authoritative geometry stated as an assertion. It runs
//! inside [`fused`], so it costs no wall clock at all.

// `pairs` describes its frustum through `cones::a_frustum`.
mod cones;
mod pairs;

// Shared with `tests/fuzzing/fuse.rs`, which reaches the same file by
// `#[path]` from its own directory (§129.5 item 3).
#[path = "fuzzing/comparison.rs"]
mod comparison;

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::RangeInclusive;

use proptest::prelude::*;
use proptest::test_runner::{
    Config, FileFailurePersistence, RngAlgorithm, TestCaseError, TestRng, TestRunner,
};
use rcad_geom::{
    Axis, Deflection, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point, Sphere,
    Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Invalid, Model, Rule, SolidBuilder, SolidKey};

use comparison::{
    Answer, Face, Picking, Run, Step, WhatItSitsOn, answer_of, bits, turned_to_its_smallest,
};
use pairs::{
    BallAndBlock, Configuration, CurvedPair, Elsewhere, Fixture, FrustumAndBlock, LeaningBar,
    Movement, Placed, Rigid, Shape, Skew, Turn, Turned, a_ball_a_bar_and_a_block,
    a_ball_and_a_block, a_chain_of_fuses, a_curved_pair, a_flush_pair, a_frustum_and_a_block,
    a_lattice_pair, a_lattice_trio, a_leaning_bar, a_pair_leaning_the_same_way,
    a_pair_somewhere_else, a_rigid_movement, a_skew_pair, a_solid_and_its_copy, a_turned_pair,
    a_turned_solid_and_its_copy, a_turned_solid_and_several_copies, room_for_a_leaning_bar, turned,
};

/// How many configurations a property that runs two fuses per case sees.
///
/// The whole workspace suite runs in a few seconds and that is worth keeping. A
/// fuse over these shapes costs a millisecond or two, and a debug build
/// validates the whole model after every commit, so the case count is what keeps
/// this file's share of the suite around a second rather than a minute.
const CASES: u32 = 48;

/// How many a property that runs four or more fuses per case sees.
const LONGER_CASES: u32 = 24;

/// How many the three measurement tests see by default.
///
/// Larger than the properties above, because their number is the thing being
/// reported rather than a means to an end, and they run on a fixed seed so the
/// number is the same every time.
///
/// A hundred and twenty-eight, and the ceiling is the clock and not the
/// statistics. At this count each family is 429 fuses, the whole workspace suite
/// runs in about three seconds, and a soak at eighty times this corpus is what
/// §17.7's precise figures come from. Raising the shipped number to a thousand
/// was tried and reverted: it bought a fresh-seed spread of ±1.6 points instead
/// of ±4, and it cost the default suite a minute of wall clock every run. §17.7
/// records honestly which figures the suite re-checks and which it does not.
const MEASURED_CASES: u32 = 128;

/// The environment variable that multiplies [`MEASURED_CASES`] for a soak.
///
/// A high-resolution run is one command rather than a code edit:
///
/// ```text
/// RCAD_SOAK_FACTOR=100 cargo nextest run -p rcad-topo --no-capture \
///     how_often_a_certified_fuse_of_a_turned_pair_refuses
/// ```
///
/// That is 12,800 configurations, 38,400 fuses, and the better part of ten
/// minutes. Repeating it changes nothing on its own — these tests are
/// deliberately deterministic — so a *spread* needs [`SOAK_SEED`] varied as
/// well.
const SOAK_FACTOR: &str = "RCAD_SOAK_FACTOR";

/// The environment variable that reseeds the measurement tests.
///
/// Set it to any whole number and the three measurement tests run from that
/// seed instead of the fixed one, which is how the fresh-seed spread §17.7
/// quotes was measured:
///
/// ```text
/// for seed in $(seq 1 12); do
///   RCAD_SOAK_FACTOR=100 RCAD_SOAK_SEED=$seed cargo nextest run -p rcad-topo \
///     --no-capture how_often_a_certified_fuse_of_a_turned_pair_refuses
/// done
/// ```
///
/// Unset — which is how the suite always runs it — the seed is fixed and the
/// number reproduces.
const SOAK_SEED: &str = "RCAD_SOAK_SEED";

/// How much of the turned family a certified fuse is allowed to refuse.
///
/// **This is §17.7's criterion 8 threshold**, recorded in that section and
/// asserted by [`how_often_a_certified_fuse_of_a_turned_pair_refuses`]. It is
/// also §14.7's other half — that valid input is *accepted* — because a
/// certified fuse that refused most of a family it can plainly answer would pass
/// every property in this file, a property only ever looking at the fuses that
/// came back.
///
/// Twenty-four per cent, chosen from measurement rather than from taste, and it
/// is a threshold on a **deterministic** number: [`measured`] runs from a fixed
/// seed, so the shipped corpus of 429 turned fuses refused 96 of them, 22.38%,
/// every run on every machine, when it was chosen. Just before Q1288 it was 45
/// of 447, 10.07%, and since Q1288 it is 39 of 447, 8.72%. There is no sampling
/// noise for the threshold to
/// have to clear, so what it has to clear instead is ordinary drift — 1.6 points
/// is seven fuses — and what it has to catch is a regression. The three undeserved
/// refusals recorded so far are each worth several times that on their own: the
/// two §17.7 names are about 3.5 points apiece, and §19.1's plane reader was
/// worth 4.4, which is what took this number from 28 to 24 when it was fixed.
///
/// **Seven fuses is thin, and §39 says where the room went.** §38.8 read the
/// stale prose beside it — 20.71% of 420 — and put the difference down to the
/// generator growing. Measured at M3's close the corpus was already 423 and the
/// rate was **20.6%, 87 refusals**; it is 96 now, and the nine arrived at
/// `c1e713e`, where `Axis::new` learned to reduce before it normalises. Every
/// configuration in this family is turned about an `Axis`, so that commit
/// changed the rotation each one is built with: this is a **different corpus**
/// and not the same corpus refusing more, which is why no answer got worse and
/// no property here noticed.
///
/// The threshold is deliberately **not** re-derived for it. Twenty-four per cent
/// still fires on anything 1.6 points above the measurement, which is well
/// inside the 3.5 points the smallest undeserved refusal on record was worth, so
/// it is doing its job and doing it more sharply than it has to. What a red run
/// asks for is the attribution above and not a bigger number — and a soak cannot
/// supply one, because a soak is a different corpus with a distribution of its
/// own and judging one by the other is the blending §17.7 forbids.
///
/// **It is judged on the shipped corpus only** — see
/// [`this_is_the_shipped_corpus`]. A soak is a different measurement with a
/// different distribution: six fresh seeds at ten times the corpus come out at
/// 23.4%, 23.4%, 23.9%, 24.0%, 24.0% and 24.7%, so four of the six sit above
/// this number with nothing wrong. Judging them by the shipped corpus's
/// threshold would be treating two different numbers as one, which is the thing
/// §17.7 is at pains not to do. A threshold is not a target — the measurement is
/// what is recorded, and this is only here so it cannot drift unnoticed.
const TURNED_REFUSALS_MUST_STAY_BELOW: f64 = 0.24;

/// How much of the skew family a certified fuse is allowed to refuse.
///
/// **§22's threshold**, and the same kind of number as
/// [`TURNED_REFUSALS_MUST_STAY_BELOW`] in every respect: deterministic, judged
/// on the shipped corpus only, and there so the measurement cannot drift
/// unnoticed rather than as a target.
///
/// Eighteen per cent. [`measured`] runs from a fixed seed, so the shipped corpus
/// of 429 skew fuses refused 51 of them, 11.89%, every run on every machine.
/// Measured on Q901's merge it was 441 fuses and 39 refused, 8.84%, and since
/// Q1145 it is 27 refused, 6.12%, which leaves this threshold further off than
/// the drift below argues for; it was moved down with neither. Just before
/// Q1279 it was 30 of 447, 6.71%, and since Q1279 it is 16 of 447, 3.58%;
/// the threshold was not moved then either. Since Q1357 it is 13 of 447,
/// 2.91%, and the threshold stays where it is.
/// There is no sampling noise to clear, so what the gap has to allow is ordinary
/// drift — 6.1 points is twenty-six fuses, nearly four times the headroom
/// [`TURNED_REFUSALS_MUST_STAY_BELOW`] now leaves — and what it has to catch is
/// a regression in the one family that reaches a branch nothing else does.
///
/// **Judged on the shipped corpus only**, for the reason
/// [`this_is_the_shipped_corpus`] gives, and here the two numbers are further
/// apart than for the turned family rather than nearer: six fresh seeds at ten
/// times the corpus come out at 12.6%, 13.8%, 14.5%, 15.3%, 16.2% and 16.3%, so
/// none of the six reaches this number but the worst of them is within 1.7
/// points of it. A soak that did cross it would be saying the fresh-seed
/// distribution sits higher than the shipped one, which is what
/// [`this_is_the_shipped_corpus`] says to expect and not a regression.
const SKEW_REFUSALS_MUST_STAY_BELOW: f64 = 0.18;

/// How many of the comparisons a property set out to make it has to have made.
///
/// §14.7's half of it, and the same idea and the same shape as
/// `turning_a_fuse.rs`'s `SOME_PAIRINGS_MUST_BE_COMPARED` next door. Every
/// property in this file hands back `Ok(())` the moment a fuse comes back
/// without an answer, so a regression that made every fuse refuse — or every
/// answer come back as something a `Model` cannot hold — would leave every one
/// of them green while comparing nothing at all. A pass is only worth what was
/// compared.
///
/// Six tenths, from every one of the twenty-four call sites of
/// [`Tally::report_what_it_compared`] that run through [`over`],
/// each over `PROPTEST_RNG_SEED` 1 through 20 — twenty-five series, because
/// [`a_solid_with_a_copy_of_itself_taken_away_is_nothing_at_all`] reports two,
/// a lattice one and a turned one. The number that matters is each series'
/// own *worst* run over the twenty seeds, not its average or its best:
///
/// - the two trio laws (`uniting_three_at_once_is_uniting_two_and_then_the_third`
///   and the other way of grouping the same three) are the floor-setters, at
///   76.74% worst,
/// - the degenerate pair's commutativity reaches 79.10% worst,
/// - the skew pair's commutativity reaches 82.09% worst,
/// - the turned series of the copy-taken-away law reaches 85.07% worst,
/// - the rest of the flat laws reach 86.05% to 95.35% worst,
/// - thirteen of the twenty-five series never miss a single comparison over
///   any of the twenty seeds: every curved law but the skew pair, and four
///   of the flat ones.
///
/// Half of a series whose worst is 100% is 50%. Half of the lowest series'
/// worst, 76.74%, is 38.37%. **A floor anywhere between those two numbers
/// catches a halving of every one of the twenty-five series** — none of
/// their own worst runs falls below it, and none of their own halved worsts
/// clears it. Six tenths sits **16.74 points below the lowest series' worst
/// (76.74%)** and **10 points above the highest series' halved worst
/// (50%)**, comfortably inside that window.
///
/// It still needs no per-law exception, unlike two of the five properties
/// under [`A_THINNER_LAW_STILL_COMPARES_THIS_MUCH`]: turned-pair
/// commutativity and the depth-four chain measure 64.93% and 65.12% worst,
/// only 4.9 and 5.1 points above six tenths — about one of their own
/// standard deviations (§276's 4.246 and 4.919) — where every one of these
/// twenty-five series sits at least 16.74 points clear. Raising the floor
/// that close to their own worst would be the same mistake §276 already
/// warned against, so those two properties, and the two leaning-bar
/// properties under [`A_LEANING_BAR_STILL_COMPARES_THIS_MUCH`], keep their
/// own lower floors rather than folding into this one (§279).
///
/// This corrects two figures the doc used to give, both stale: "74%" for the
/// trio laws, which twenty seeds now put at 76.74%, and "90%" for "the
/// exactly representable pairs", which folded the lattice pair (94.03%
/// worst) and the degenerate pair (79.10% worst) into one number when they
/// are two. It also drops the old defense of three tenths: that it only had
/// to catch a law that stopped comparing anything at all, not a halving,
/// because every property but the two Q160 pulled out measured comfortably
/// above 60%. That was already false for the trio laws — 76.74% halved is
/// 38.37%, which cleared three tenths — so three tenths was letting a
/// halving of them through unnoticed. Six tenths does not.
const SOME_COMPARISONS_MUST_HAPPEN: f64 = 0.6;

/// The floor for every law measured thinner than [`SOME_COMPARISONS_MUST_HAPPEN`]
/// can safely clear: turned-pair commutativity, the depth-four chain, and all
/// three properties over a frustum standing through a block (§276).
///
/// **One floor serves all five**, because their measured ranges overlap by a
/// wide margin. Twenty fresh seeds (`PROPTEST_RNG_SEED` 1 to 20) of the shipped
/// corpus, run at `just t`'s own case counts:
///
/// | law | worst | best | half the worst |
/// |---|---|---|---|
/// | turned-pair commutativity | 64.93% | 82.09% | 32.46% |
/// | the depth-four chain | 65.12% | 81.40% | 32.56% |
/// | a turned frustum and block, commutativity | 76.12% | 93.28% | 38.06% |
/// | a turned frustum and block, volume under a turn | 76.12% | 93.03% | 38.06% |
/// | a frustum and block, commutativity (not turned) | 95.52% | 100% | 47.76% |
///
/// A half is **13.8 to 61.9 points** below every one of the five laws' own
/// worst run — 2.9 standard deviations and up, using each law's own population
/// standard deviation over its twenty runs (5.08, 4.28, 3.48, 3.45 for the
/// first four; the fifth answers or refuses so consistently that a standard
/// deviation on it is not the right question, see below) — and **2.2 to 17.5
/// points above** every one of the five laws' own halved worst. It sits inside
/// all five windows at once, so it catches a halving of any one of them
/// without brushing what any of them actually does. The fifth's own margin is
/// the thinnest, 2.24 points above its halved worst, because nothing has ever
/// measured that property below 95.5% — the margin is thin only in the sense
/// that a law running this well leaves little air between "half of what it
/// does" and "all of what it does."
///
/// This retires the frustum family's own floor. A frustum standing up through a
/// block used to answer exactly when the cone above its own top cap stayed
/// clear of the block's side walls, which made the share that answers swing
/// widely; eighteen fresh-seed runs before that boundary was removed came out
/// between 30% and 46%. **The boundary is gone** — `arrange`'s third
/// `HowItRuns` variant now answers the configurations it used to turn down —
/// and the twenty runs above measure what replaced it: 76% to 100% across all
/// three frustum properties, comfortably inside this shared floor's own
/// margin.
const A_THINNER_LAW_STILL_COMPARES_THIS_MUCH: f64 = 0.5;

/// How many of the eighty-four-configuration grid's comparisons have both sides.
///
/// An exact count rather than a floor, because
/// [`every_frustum_on_the_wider_grid_answers_the_same_whichever_side_comes_first`]
/// walks a grid that is written out and not drawn, so this is the same number
/// every run on every machine. Measured: 168 comparisons meant, **114 made**, out
/// of 336 fuses of which 108 refused — every one of the 108 naming `select.rs`'s
/// *a point inside a piece of a face* — and **none** came back as something a
/// `Model` cannot hold.
///
/// It has moved twice. It was 106 made, 124 refused and none unholdable until
/// §206's bulge count, which took it to 110 made, 108 refused and **8
/// unholdable**. Those 8 were the edge-naming gap §206.4 held — two arcs of a
/// cone's rim circle between one pair of corners named as one edge — surfacing
/// here because more fuses got that far. Naming the two arcs apart closes all 8,
/// and four of them become comparisons. Every comparison that ran still agreed,
/// either side of both moves.
///
/// **It is 118 since §224.4**, and the four new comparisons are the flush pair —
/// `side: 7, up_per_unit_wider: 8` at `[3, 3]` and at `[4, 4]` — under Union and
/// Intersection. All four refused "a point in a piece that every solid could
/// place" both ways round until a flat cap was asked what it holds; then one way
/// round of each was refused `ResultIsNotManifold` until a closed curve's corner
/// was chosen once for all the pieces that bring it.
/// [`a_frustum_whose_cap_lies_in_the_blocks_roof_answers_the_same_either_way_round`]
/// checks their volumes.
///
/// **It is all 168 since §251.** The fifty that were left, twenty-five
/// configurations under both rules, were each a block wall's piece inside the
/// cone carried on above the frustum: one arc of a hyperbola and the chord the
/// roof cuts across it. `select.rs` counts that arc's bulge on its own half of
/// the cone now, and every one of the fifty agrees both ways round.
/// `tests/ordering_points_along_an_endless_bend.rs`'s
/// `every_answer_on_the_grid_holds_what_the_shapes_hold_by_hand` checks all
/// 168 answers' volumes against the frustum's closed form.
///
/// A drop here is a fuse that stopped answering, which is a regression and not a
/// draw. Re-derive it when it moves; do not relax it to a floor (§19.1).
const THE_WIDER_GRID_COMPARES_THIS_MANY: usize = 168;

/// How many of the comparisons a law over a bar leaning through a block has to
/// have made (§191, §276).
///
/// **This family refuses most of what it draws, and says so here rather than
/// shipping as coverage** (§14.7). Both laws over it run from the fixed seed
/// ([`over_a_fixed_corpus`]), so unlike every other floor in this file the
/// share compared here is not a range — the seed passed to the RNG makes no
/// difference to it — so **one run measures it for good**:
/// `mise exec -- just t swapping_the_two_sides_of_a_bar_leaning_through_a_block_changes_not_one_bit --no-capture`
/// and `mise exec -- just t what_a_block_and_a_leaning_bar_take_up_adds_up_inside_a_stated_share_of_what_they_declare --no-capture`.
///
/// **Q114 (d6c64ba4) fixed the fuse this family exercises most** — the wrong
/// `ResultIsNotManifold` §191 found — so both laws now reach the same share:
/// commutativity **118 of 140** and inclusion and exclusion **59 of 70**,
/// both 84.29%, up from the 44.78% and 26.87% the old quarter floor was sized
/// against, when most of this family's fuses answered that wrong refusal
/// rather than reaching a comparison.
///
/// **A quarter no longer catches a halving of either law.** Half of 84.29% is
/// 42.14%, well clear of a quarter (25%). What serves both at once, since they
/// now measure the identical share, is anywhere in **42.14% to 84.29%** —
/// above the halved worst, below the measured worst. **A half** sits 7.86
/// points above the halved worst and 34.29 points below the measured worst —
/// margins of the same order [`A_THINNER_LAW_STILL_COMPARES_THIS_MUCH`]'s own
/// five properties keep against their own halved worst and worst — and stays
/// clear of [`SOME_COMPARISONS_MUST_HAPPEN`] (six tenths), so this family
/// keeps its own floor rather than folding into that one (§279).
const A_LEANING_BAR_STILL_COMPARES_THIS_MUCH: f64 = 0.5;

/// How much of a curved fixture turned together a certified fuse may refuse
/// (§191).
///
/// Measured over the shipped corpus: a bar through a block **6 of 429**, a ball
/// in a block **3 of 429**, every one naming *telling the cells of a face
/// apart*, against none for either upright. Three per cent is thirteen fuses —
/// room for twice the larger count and a little over, and still fires on
/// anything that makes the turned row a real fraction of its fuses. Every one
/// of those refusals is a gap: see
/// [`every_fuse_a_turned_curved_fixture_refuses_has_an_answer_turned_from_the_upright_one`].
const TURNED_ROUND_REFUSALS_MUST_STAY_BELOW: f64 = 0.03;

/// How much of a bar leaning through a block a certified fuse may refuse
/// (§191).
///
/// Measured at **63 of 450, 14.0%**, over the shipped corpus
/// (`mise exec -- just t how_often_a_certified_fuse_of_a_bar_leaning_through_a_block_refuses --no-capture`),
/// down from 298 of 429, 69.5%, before Q114 (d6c64ba4) fixed the wrong
/// `ResultIsNotManifold` §191 found. Seventy-five per cent is 61 points above
/// the measured rate. It is a ceiling on a family that still refuses more
/// than most, so its job is only to say out loud when that gets worse;
/// [`A_LEANING_BAR_STILL_COMPARES_THIS_MUCH`] is the guard that the laws over
/// it still compare anything.
const LEANING_BAR_REFUSALS_MUST_STAY_BELOW: f64 = 0.75;

/// How often the lattice control comes back as something a `Model` cannot
/// hold, as a rate: the ends are judged with a draw's own spread, by
/// [`Tally::rate_of_answers_that_were_not_solids_must_be_within`].
///
/// Every one of them is the case §17.7's catalogue records by hand: two solids
/// meeting along a whole edge would put four faces along it, and an edge
/// belongs to exactly two.
///
/// **The ends are two thirds to three times the measured rate, rounded to two
/// figures. That is the convention all six of these bands are derived to**, and
/// each of the other five says so and shows its own arithmetic. The rate is
/// **2.23%** — 63 913 of 2 870 313 fuses, 113 fresh seeds at a [`SOAK_FACTOR`]
/// of 66 — so the ends are 1.5% and 6.7%. §303.
///
/// **It was two thirds to three times one draw, and that draw was high.** The
/// band was 2.2% to 9.8%, from the shipped corpus's 14 of 429, 3.3%, and read
/// as a tripwire with no sampling noise to cover because [`measured`] runs from
/// a fixed seed. A soak sets [`SOAK_SEED`], and then there is noise: at 25 401
/// fuses a draw's share spreads by 0.08 points around 2.23%, so a floor at 2.2%
/// failed **38 of those 113 seeds**, and §290.4's two failures were two of
/// them. At the shipped 441 fuses the spread is 0.65 points and 112 of 400
/// fresh seeds came in under it. The fixed seed's 14 is simply where one draw
/// landed.
///
/// So the ends are moved out by the draw's spread at its own size. At the
/// shipped 441 fuses that leaves a floor of one fuse and a ceiling of 45. A
/// floor at that size that caught the rate halving two times in three would
/// also fail about 3 fresh seeds in 100, which §303.3 measures. At a soak's
/// 25 401 the floor is 323 fuses and the ceiling 1 821, and a halving fires 99
/// times in a hundred.
///
/// **When one of them moves, re-derive its band from the new measurement**
/// rather than widening an end to take it in. Widening is how these six came to
/// be set once, in August, and never looked at again: the fall recorded below
/// moved every one of them, two of them by more than half, and no band fired.
///
/// **All six of these figures fell at `b4890f1`**, and they fell together.
/// Measured on that commit and on its parent `cc959c1`, over the same 423-fuse
/// corpus and with no refusal count moving: this family 20 to 14, the flush
/// family 72 to 61, elsewhere 17 to 12, turned 17 to 13, leaning alike 17 to 7,
/// skew 8 to 3. `b4890f1` is where a face began to be cut by its own solid's
/// planes as well as the other's, and the fuse it was written for united to a
/// solid one way round and to an edge with one face along it the other — which
/// is this count exactly. Fewer answers are not solids because fewer answers are
/// wrong.
///
/// The corpus going 423 to 429 is a **separate** move and a later one:
/// `01e62c1` and `9b3a5fd` recorded a proptest seed each, and a replayed seed is
/// three more fuses. It is 441 now, nineteen replayed seeds, and this family
/// still gives 14 of them.
const A_LATTICE_PAIR_IS_NOT_A_SOLID: RangeInclusive<f64> = 0.015..=0.067;

/// The same for the family built to share coordinates, which is full of them.
///
/// Measured at 14.2% — 61 of 429 — which is what "deliberately degenerate"
/// buys: the offsets are whole extents, so edge-on-edge and corner-on-corner
/// are the ordinary case here rather than the rare one.
///
/// **This band is a deliberate exception to the file's own convention, kept
/// narrower on purpose.** Two thirds to three times of 14.2% gives 9.5% to
/// 43% — 41 fuses to 184 — and every other one of these six is derived to
/// exactly that rule. This one is not: it stays at 10% to 28%, 43 to 120
/// fuses, because nobody has measured a reason to let this family answer
/// "not a solid" as often as 184 times out of 429. The band was briefly
/// widened to the convention's 9.5%–43% during a mechanical re-derivation of
/// all six and that widening was reverted here — the tidy-up moved a number
/// nobody had a reason to move. Re-derive the other five from a new
/// measurement freely; re-derive *this* one only alongside a measurement
/// that says why more than 120 is fine.
///
/// **Measured at a soak for Q258.** Eight fresh seeds at [`SOAK_FACTOR`] 66
/// (25 407 fuses a seed) pool to 24 570 of 203 256, 12.09%, each seed between
/// 11.83% and 12.52% — notably lower than the single draw's 14.2% this band
/// was checked against, but still well inside 10%–28%. Twenty-four seeds at
/// factor 1 (447 fuses, the shipped size) spread from 9.84% to 14.09%, and
/// two of them — seeds 9 and 21, both 44 of 447 — land under the fixed 10%
/// floor by chance alone, an ordinary draw from a 12% family rather than a
/// sign anything moved. That is why this row now goes through
/// [`band_for_a_draw_of`] like the other five: the rate stays 10%–28%,
/// unwidened, and only the comparison scales to the draw's own size.
const A_FLUSH_PAIR_IS_NOT_A_SOLID: RangeInclusive<f64> = 0.10..=0.28;

/// The same for the lattice pairs written somewhere else in `f64`.
///
/// Measured at 2.8% — 12 of 429 — against the lattice family's 3.3%, and the two
/// are the same figure rather than two. The configurations are drawn from the
/// same generator and the same seed, but this family draws two more numbers per
/// case, so the two corpora are not the same 128 configurations. What is exactly
/// the same is the reason: moving everything by a power of two cannot turn an
/// answer into one a `Model` can hold or the other way about.
///
/// The band is its own, though, because the convention is applied to each
/// family's own measurement: two thirds to three times 2.8% is 1.9% to 8.4%,
/// which is 9 fuses to 36. The lattice family's band would have been a slightly
/// higher one and reading either as the other's is what §17.7 forbids.
///
/// **Measured at a soak for Q258.** Eight fresh seeds at [`SOAK_FACTOR`] 66
/// pool to 4 540 of 203 256, 2.23%, each seed between 2.01% and 2.38% — a
/// bit lower than the 2.8% this band was built from, comfortably inside
/// 1.9%–8.4%. Twenty-four seeds at factor 1 (447 fuses, the shipped size)
/// spread from 1.79% to 3.80%, and two of them — seeds 3 and 11, both 8 of
/// 447 — land under the fixed 1.9% floor by chance alone, an ordinary draw
/// from a family whose rate has not moved. This row now goes through
/// [`band_for_a_draw_of`] for the same reason the lattice and turned rows
/// do: the band stays 1.9%–8.4%, and only the comparison scales to the
/// draw's own size.
const A_PAIR_ELSEWHERE_IS_NOT_A_SOLID: RangeInclusive<f64> = 0.019..=0.084;

/// The same for the turned family, a rate judged the same way.
///
/// **The rate is 0.78%** — 726 of 92 844 fuses, twelve fresh seeds (11 to 22)
/// at a [`SOAK_FACTOR`] of 20, between 48 and 78 a seed — so the convention
/// gives 0.52% to 2.3%. The shipped corpus gives 6 of 441.
///
/// It was 2.45%, 29 847 of 1 219 248 fuses at a factor of 66, and the band
/// 1.6% to 7.3%, until an input corner with exactly three planes round it was
/// named by those planes (`docs/decisions.md` §351.6). The shipped corpus gave
/// 14 of 441 then. In the configuration §351.6 traced, a corner of one solid
/// lying on an edge of the other was named as a vertex by one solid's pieces
/// and as the meet of three planes by the other's, so one corner was two and
/// edges were left with one face each. **Two kinds of fuse moved, and they move
/// the rate opposite ways**: fuses that were not a solid and are answered now,
/// and answers that are not a solid now. Each of the second kind measured is
/// two solids touching along an edge, refused `count: 4` unturned and now
/// turned as well. Over the 67 pairs of
/// [`swapping_the_two_sides_of_a_turned_pair_changes_not_one_bit`], counting a
/// rule both ways round as one: ten that answered one way round only and five
/// that answered neither way answer both ways now, and two unions that
/// answered both ways answer neither, so 24 fuses not a solid became 8.
const A_TURNED_PAIR_IS_NOT_A_SOLID: RangeInclusive<f64> = 0.0052..=0.023;

/// How many of its own spreads a draw may land past an end of a not-a-solid
/// rate band before [`Tally::rate_of_answers_that_were_not_solids_must_be_within`]
/// calls it a move.
///
/// Three. The ends already sit at two thirds and three times the rate, so the
/// spread only has to cover a draw straying past an end by chance. At three,
/// with each row's rate measured at a soak and no configuration giving more
/// than one answer that is not a solid, that is under one run in a million
/// for every row that uses it at every factor from 1 to 660 (§303.6).
const A_DRAW_STRAYS_THIS_MANY_SPREADS: f64 = 3.0;

/// The same for the family whose outlines lean over.
///
/// Measured at 0.7% over the shipped corpus — 3 of 429 — which is **a fifth**
/// of the lattice family's 3.3%, and the reason is the point of the family. An
/// answer is not a solid here almost always because two solids meet along a
/// whole edge and four faces would run along it, and that needs the two edges to
/// be the same edge exactly. Leaning the two solids by different amounts is
/// precisely what stops them sharing an edge. It is still the lowest of the six
/// families by some way, and it is the only one where the ratio to the lattice
/// family is what the family was built to be about.
///
/// **The ends are the count's own spread, and they are not two thirds to three
/// times the rate.** That convention is what the other five rows use and what
/// this row used until §136.5, where it gave `0.47%–2.1%` — three fuses to nine
/// — from a single draw of three. It is the wrong tool here: the rate is a
/// twentieth of the lattice family's, so the expected *count* at 429 fuses is
/// about three and a half, and a band whose floor is two fuses sits inside that
/// count's own sampling spread rather than outside it.
///
/// Measured rather than argued, `RCAD_SOAK_SEED` varied the way [`SOAK_SEED`]
/// documents. **Twelve fresh seeds at the shipped 429 fuses give 1, 5, 3, 1, 4,
/// 4, 5, 4, 6, 1, 1, 3** — a spread of one to six, with four of the twelve
/// landing on one. **Six seeds at ten times the corpus, 3 885 fuses, give
/// 0.72%, 1.00%, 0.88%, 0.72%, 0.93% and 0.72%**, so the underlying rate is
/// 0.83% and has not moved from the 0.7% this band was first built on. What
/// moved was which draw the fixed seed takes.
///
/// So the ends are one fuse to twelve — `0.23%` to `2.8%` at 429 fuses — the
/// observed spread with the ceiling at twice its maximum. The floor is one fuse
/// because one fuse is the question this row actually asks: whether the family
/// still reaches the four-faces-along-one-edge configuration it was built to
/// reach. Zero is the answer that fires.
///
/// **So the floor is a count, [`A_SKEW_PAIR_IS_NOT_A_SOLID_AT_LEAST`], and not
/// a share.** It was written `0.0023`, one fuse in 429, and the number of fuses
/// is not fixed at 429: cases replayed from
/// `fuses_that_must_agree.proptest-regressions` are counted too, so it moves
/// whenever a seed line is added. With the file at eighteen lines the row
/// counted 1 of 438, under `0.0023`, and failed with the family reaching its
/// configuration exactly as it was built to. The ceiling is left a share, which
/// a larger count only moves further inside.
///
/// **The sentence this replaces was true and still misled.** It said a band
/// this tight is not a flake waiting to happen, because [`measured`] runs from
/// a fixed seed and a count of three is a count of three on every machine. That
/// is right about re-running and wrong about safety: a deterministic draw from
/// a family whose counts run one to six moves anywhere in that range whenever
/// the corpus is reshuffled, and §121, §130, §131 and §132 reshuffled it. The
/// row then failed for six weeks' worth of arithmetic work having changed the
/// draw, not the kernel's behaviour. §136.5.
const A_SKEW_PAIR_IS_NOT_A_SOLID: RangeInclusive<f64> = 0.0..=0.028;

/// How many of the skew family's fuses must come back not a solid: one, the
/// floor [`A_SKEW_PAIR_IS_NOT_A_SOLID`] explains.
const A_SKEW_PAIR_IS_NOT_A_SOLID_AT_LEAST: usize = 1;

/// The same for its control, where both solids lean the same way.
///
/// Measured at 1.6% — 7 of 429 — which sits between the lattice family's 3.3%
/// and the skew family's 0.7% rather than on the lattice family's figure, where
/// the arithmetic the control claims puts it. Leaning both solids by the same
/// amount is an affine map of the whole pair, and an affine map preserves
/// incidence exactly, so the edges that coincided still coincide and the answers
/// that were not solids still are not. Two things pull it below 3.3%: this
/// family draws one more number per case, so the two corpora are not the same
/// 128 configurations — the same caveat [`A_PAIR_ELSEWHERE_IS_NOT_A_SOLID`]
/// carries — and a fuse that refuses never gets far enough to find out whether
/// its answer was a solid, which this family does 24 times. Whether those two
/// account for the whole of a gap that is now half the count is not measured
/// here; it was 17 against 20 when the figure was last taken. **§22.3 is the
/// section to read beside this number**, because a gap between this row and the
/// lattice row is what caught the bug that section records — 1.9% against 4.8%
/// then, 1.6% against 3.3% now.
///
/// Two thirds to three times 1.6% is 1.1% to 4.9%, 5 fuses to 21. It was 1.2%
/// to 8%, which is 6 to 34 — so the convention buys one more fuse of room below
/// and takes thirteen away above. This row and the skew row are the two the
/// convention would have caught at `b4890f1`: bands derived from the parent
/// commit's own measurement, 4.0% here and 1.9% there, put floors at 2.7% and
/// 1.3%, and `b4890f1` landed at 1.7% and 0.7%. Both trees were built and both
/// rows measured; the other four stayed inside, the lattice and elsewhere rows
/// by under a fuse.
///
/// **Measured at a soak for Q258.** Eight fresh seeds at [`SOAK_FACTOR`] 66
/// pool to 4 515 of 203 256, 2.22%, each seed between 2.10% and 2.31% —
/// noticeably higher than the 1.6% this band was built from, but still well
/// inside 1.1%–4.9%. Twenty-four seeds at factor 1 (447 fuses, the shipped
/// size) spread from 1.57% to 4.03%: every one of them stayed inside the
/// fixed band, but at over half the room on each side, a small draw this
/// family hasn't yet given is only a bad roll away. This row now goes
/// through [`band_for_a_draw_of`] alongside the other two Q258 measured: the
/// band stays 1.1%–4.9%, and only the comparison scales to the draw's own
/// size.
const A_PAIR_LEANING_ALIKE_IS_NOT_A_SOLID: RangeInclusive<f64> = 0.011..=0.049;

#[test]
fn what_the_union_and_the_overlap_take_up_is_what_the_two_solids_take_up() {
    let tally = over(CASES, a_lattice_pair(), |configuration, tally| {
        let (model, keys) = configuration.build();
        let (one, other) = (keys[0], keys[1]);
        tally.meaning_to_compare();

        let Some(united) = fused(
            tally,
            Coordinates::AreExactlyRepresentable,
            &model,
            &[one],
            &[other],
            Rule::Union,
        )?
        .answer() else {
            return Ok(());
        };
        let Some(shared) = fused(
            tally,
            Coordinates::AreExactlyRepresentable,
            &model,
            &[one],
            &[other],
            Rule::Intersection,
        )?
        .answer() else {
            return Ok(());
        };

        tally.comparing();
        prop_assert_eq!(
            volume_of(&united.0, &united.1) + volume_of(&shared.0, &shared.1),
            volume_of(&model, &[one]) + volume_of(&model, &[other]),
            "the union and the overlap together are the two solids together"
        );
        Ok(())
    });
    tally.report_what_it_compared("inclusion and exclusion, lattice pairs");
}

#[test]
fn uniting_the_first_two_and_then_the_third_is_uniting_the_last_two_and_then_the_first() {
    let tally = over(LONGER_CASES, a_lattice_trio(), |configuration, tally| {
        let (model, keys) = configuration.build();
        let (one, two, three) = (keys[0], keys[1], keys[2]);
        tally.meaning_to_compare();

        let coordinates = Coordinates::AreExactlyRepresentable;
        let Some(left) = united_in_turn(tally, coordinates, &model, &[one], &[two], &[three])?
        else {
            return Ok(());
        };
        let Some(right) = united_in_turn(tally, coordinates, &model, &[two], &[three], &[one])?
        else {
            return Ok(());
        };

        tally.comparing();
        prop_assert_eq!(
            invariants_of(&left.0, &left.1),
            invariants_of(&right.0, &right.1),
            "uniting three solids does not depend on the order they are united in"
        );
        Ok(())
    });
    tally.report_what_it_compared("associativity of union, lattice trios");
}

#[test]
fn taking_away_what_is_left_after_taking_the_other_away_leaves_what_the_two_share() {
    let tally = over(LONGER_CASES, a_lattice_pair(), |configuration, tally| {
        let (model, keys) = configuration.build();
        let (one, other) = (keys[0], keys[1]);
        tally.meaning_to_compare();

        let Some(bitten) = fused(
            tally,
            Coordinates::AreExactlyRepresentable,
            &model,
            &[one],
            &[other],
            Rule::Difference,
        )?
        .answer() else {
            return Ok(());
        };
        let Some(twice_bitten) = fused(
            tally,
            Coordinates::AreExactlyRepresentable,
            &bitten.0,
            &[one],
            &bitten.1,
            Rule::Difference,
        )?
        .answer() else {
            return Ok(());
        };
        let Some(shared) = fused(
            tally,
            Coordinates::AreExactlyRepresentable,
            &model,
            &[one],
            &[other],
            Rule::Intersection,
        )?
        .answer() else {
            return Ok(());
        };

        tally.comparing();
        prop_assert_eq!(
            invariants_of(&twice_bitten.0, &twice_bitten.1),
            invariants_of(&shared.0, &shared.1),
            "A \\ (A \\ B) is A ∩ B"
        );
        Ok(())
    });
    tally.report_what_it_compared("A \\ (A \\ B) = A ∩ B, lattice pairs");
}

#[test]
fn turning_a_pair_a_quarter_turn_and_moving_it_a_whole_number_changes_not_one_bit() {
    let tally = over(
        LONGER_CASES,
        (a_lattice_pair(), a_rigid_movement()),
        moving_a_pair_leaves_the_answer_alone,
    );
    tally.report_what_it_compared(
        "transform invariance, lattice pairs under quarter turns and whole-number shifts",
    );
}

/// The same property over [`a_pair_somewhere_else`], which is the whole reason
/// that family exists.
///
/// The two runs are the same configuration written at two places in `f64`, and
/// the claim is bit for bit with no bound anywhere — see the family's own doc
/// comment for why that is exactly true and for what a different `k` costs the
/// cascade. §12.4 says outright that no end-to-end outcome flip was ever built
/// for a predicate whose budget carries the wrong power of the working size;
/// this is the one that would catch it.
#[test]
fn writing_a_pair_somewhere_else_in_f64_and_then_moving_it_changes_not_one_bit() {
    let tally = over(
        LONGER_CASES,
        (a_pair_somewhere_else(), a_rigid_movement()),
        moving_a_pair_leaves_the_answer_alone,
    );
    tally.report_what_it_compared(
        "transform invariance, pairs somewhere else in f64 under quarter turns and shifts",
    );
}

/// Fuse the pair, move the pair and fuse it again, and insist the second answer
/// is the first one moved.
fn moving_a_pair_leaves_the_answer_alone(
    (configuration, movement): (Configuration, Rigid),
    tally: &Tally,
) -> Result<(), TestCaseError> {
    moving_a_built_pair_leaves_the_answer_alone(
        tally,
        Coordinates::AreExactlyRepresentable,
        &|| configuration.build(),
        &|| configuration.built_after(&movement),
        &movement.in_the_world_of(configuration.elsewhere),
    )
}

/// The same property over a pair built by hand rather than from a
/// [`Configuration`], so the curved families can run it too.
///
/// `still` builds the pair where it stands and `moved` builds it after
/// `movement`; the two closures are what the curved families supply instead of
/// [`Configuration::built_after`].
fn moving_a_built_pair_leaves_the_answer_alone(
    tally: &Tally,
    coordinates: Coordinates,
    still: &dyn Fn() -> (Model, Vec<SolidKey>),
    moved: &dyn Fn() -> (Model, Vec<SolidKey>),
    movement: &Movement,
) -> Result<(), TestCaseError> {
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        tally.meaning_to_compare();
        let (before_model, keys) = still();
        let before = fused(
            tally,
            coordinates,
            &before_model,
            &[keys[0]],
            &[keys[1]],
            rule,
        )?;

        let (after_model, keys) = moved();
        let after = fused(
            tally,
            coordinates,
            &after_model,
            &[keys[0]],
            &[keys[1]],
            rule,
        )?;

        both_agree_about_whether_there_is_an_answer(
            tally,
            coordinates,
            &before,
            &after,
            rule,
            "moving it",
        )?;
        let (Some(before), Some(after)) = (before.answer(), after.answer()) else {
            continue;
        };

        tally.comparing();
        let expected = answer_of(&before.0, &before.1).moved_by(movement);
        same_answer(&expected, &answer_of(&after.0, &after.1), rule, "moving it")?;
    }
    Ok(())
}

#[test]
fn swapping_the_two_sides_of_a_lattice_pair_changes_not_one_bit() {
    let tally = over(CASES, a_lattice_pair(), |configuration, tally| {
        both_ways_round(tally, Coordinates::AreExactlyRepresentable, configuration)
    });
    tally.report_what_it_compared("commutativity, lattice pairs");
}

/// Commutativity over the turned pairs.
///
/// **Since Q1288 a fuse here may be refused for an answer with an edge too
/// short beside its coordinates**, counted by name ([`a_flat_refusal`]) rather
/// than failed: 8 of its 276 fuses are, and 12 are refused in all, from 20.
#[test]
fn swapping_the_two_sides_of_a_turned_pair_changes_not_one_bit() {
    let tally = over(CASES, a_turned_pair(), |configuration, tally| {
        both_ways_round(tally, Coordinates::AreUnderAGeneralRotation, configuration)
    });
    tally.report_what_it_compared_at_least(
        "commutativity, turned pairs",
        A_THINNER_LAW_STILL_COMPARES_THIS_MUCH,
    );
}

/// §48.2 item 5's defect, at the first configuration that found it, fixed.
///
/// A frame and a comb, turned together by one sixty-fourth of a turn about
/// `(-2, -1, 0)`. What the two share is two solids. Comb against frame gave the
/// two, and frame against comb gave [`Error::ResultIsNotManifold`]. Whether four
/// faces end up along one edge is a fact about the answer, and the answer does
/// not know which argument came first.
///
/// **The same pair unturned gives the two solids either way round**, which is
/// the control: the two are in the same places relative to each other, and the
/// only thing between the two rows is that the rotation leaves nothing exactly
/// representable.
///
/// **Two** things split one corner of this pair in two:
///
/// - The two solids are flush on a plane and name it opposite ways round, so a
///   corner on it was solved from `P, Q, R` by one face and from `−P, Q, R` by
///   another. Three planes are named whichever way each of them faces now (§48,
///   W5, `assemble::names_of_the_three_surfaces`).
/// - The two solids also have an input **vertex** each at one shared corner —
///   `11v1` and `23v1` — and the rotation lands the two a last bit apart, so
///   neither the names nor `assemble::exactly_at`'s coordinates joined them.
///   An input corner with exactly three planes round it is named by those
///   planes now and put where the three alone put it (§351.6), and that brought
///   the first order back to the two solids.
///
/// With both gone, [`both_agree_about_whether_there_is_an_answer`] fails on
/// this family rather than counting.
#[test]
fn turning_a_frame_and_a_comb_shares_the_same_two_solids_whichever_comes_first() {
    let frame = Placed {
        shape: Shape::Frame { width: 4, depth: 3 },
        skew: Skew::none(),
        height: 1,
        at: [0, 0, 0],
    };
    let comb = Placed {
        shape: Shape::Comb {
            teeth: 3,
            tooth_height: 2,
        },
        skew: Skew::none(),
        height: 1,
        at: [0, -2, 0],
    };
    let pair = |turn| Configuration {
        solids: vec![frame, comb],
        turn,
        elsewhere: Elsewhere::here(),
    };

    let (model, keys) = pair(None).build();
    for (arguments, tools) in [(keys[0], keys[1]), (keys[1], keys[0])] {
        assert_eq!(
            what_a_fuse_came_back_with(&model, arguments, tools, Rule::Intersection),
            WhatCameBack::Solids(2),
            "the control: unturned, either order shares the same two solids"
        );
    }

    let (model, keys) = pair(Some(Turn {
        direction: [-2, -1, 0],
        steps: 1,
    }))
    .build();
    for (arguments, tools) in [(keys[0], keys[1]), (keys[1], keys[0])] {
        assert_eq!(
            what_a_fuse_came_back_with(&model, arguments, tools, Rule::Intersection),
            WhatCameBack::Solids(2),
            "turned, either order shares the same two solids"
        );
    }
    let forwards = shared(&model, keys[0], keys[1]);
    let backwards = shared(&model, keys[1], keys[0]);
    if let Err(why) = same_answer(
        &answer_of(&forwards.0, &forwards.1),
        &answer_of(&backwards.0, &backwards.1),
        Rule::Intersection,
        "swapping the frame and the comb",
    ) {
        panic!("{why}");
    }
}

/// What a certified intersection of `arguments` with `tools` hands back, in a
/// copy of `model`.
fn shared(model: &Model, arguments: SolidKey, tools: SolidKey) -> (Model, Vec<SolidKey>) {
    let mut model = model.clone();
    let solids = model
        .fuse(
            &[arguments],
            &[tools],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the frame and the comb share two solids")
        .solids()
        .to_vec();
    (model, solids)
}

/// The same defect with **no rotation and no tilt at all**, fixed.
///
/// A block two by one by two at the origin, and a frame three by three and one
/// thick two behind it. Every corner of both solids is a small whole number,
/// every wall is axis-aligned, and every meet of three of their planes is a
/// Cramer solve with a determinant of ±1 — this is the family §17.7's control
/// row asserts the kernel decides all of, and it does decide it. It still united
/// to one solid one way round and to [`Error::ResultIsNotManifold`] the other,
/// so the order-dependence §48.2 item 5 predicted was **not** a story about
/// rounding: nothing rounds here.
///
/// What it was: the frame's own hole wall at `x = 1` cut the block's wall and
/// nothing of the block sat on `x = 1`, so the frame's own wall was left whole,
/// and where the answer took one side of that line from the frame's face and
/// the other from the block's the two no longer met along one edge. Every face
/// is cut by every plane of both solids now, its own included (§48, W1), so both
/// sides of the line are diced alike. `tests/a_face_flush_against_another_solids_face.rs`
/// checks the answer against arithmetic done by hand — volume 10, area 38, one
/// shell — rather than against the other order, because agreeing with each other
/// is what the two orders did wrong here.
///
/// This says only what this file can say: the two orders agree, and what they
/// agree on is a solid.
#[test]
fn a_lattice_pair_unites_to_the_same_solid_whichever_way_round_it_is_handed_over() {
    let block = Placed {
        shape: Shape::Block { width: 2, depth: 1 },
        skew: Skew::none(),
        height: 2,
        at: [0, 0, 0],
    };
    let frame = Placed {
        shape: Shape::Frame { width: 3, depth: 3 },
        skew: Skew::none(),
        height: 1,
        at: [0, -2, 0],
    };
    let (model, keys) = Configuration {
        solids: vec![block, frame],
        turn: None,
        elsewhere: Elsewhere::here(),
    }
    .build();

    for (arguments, tools) in [(keys[0], keys[1]), (keys[1], keys[0])] {
        assert_eq!(
            what_a_fuse_came_back_with(&model, arguments, tools, Rule::Union),
            WhatCameBack::Solids(1)
        );
    }
}

/// What one certified fuse of two solids came back with.
fn what_a_fuse_came_back_with(
    model: &Model,
    arguments: SolidKey,
    tools: SolidKey,
    rule: Rule,
) -> WhatCameBack {
    let mut model = model.clone();
    match model.fuse(
        &[arguments],
        &[tools],
        rule,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) {
        Ok(fused) => WhatCameBack::Solids(fused.solids().len()),
        Err(Error::ResultIsNotManifold { .. }) => WhatCameBack::NotASolid,
        Err(why) => WhatCameBack::WentWrong(format!("{why}")),
    }
}

/// The outcome of one named fuse, in the three shapes the pins above tell apart.
#[derive(Debug, PartialEq, Eq)]
enum WhatCameBack {
    /// An answer, and how many solids were in it.
    Solids(usize),
    /// An answer no `Model` can hold.
    NotASolid,
    /// Anything else, kept as what it said.
    WentWrong(String),
}

#[test]
fn swapping_the_two_sides_of_a_degenerate_pair_changes_not_one_bit() {
    let tally = over(CASES, a_flush_pair(), |configuration, tally| {
        both_ways_round(tally, Coordinates::AreExactlyRepresentable, configuration)
    });
    tally.report_what_it_compared("commutativity, pairs sharing coordinates");
}

#[test]
fn a_solid_united_with_a_copy_of_itself_is_itself() {
    let tally = over(CASES, a_solid_and_its_copy(), |configuration, tally| {
        let (model, keys) = configuration.build();
        let (one, copy) = (keys[0], keys[1]);
        let itself = invariants_of(&model, &[one]);
        tally.meaning_to_compare();

        let Some(united) = fused(
            tally,
            Coordinates::AreExactlyRepresentable,
            &model,
            &[one],
            &[copy],
            Rule::Union,
        )?
        .answer() else {
            return Ok(());
        };
        prop_assert_eq!(invariants_of(&united.0, &united.1), itself.clone());

        let Some(shared) = fused(
            tally,
            Coordinates::AreExactlyRepresentable,
            &model,
            &[one],
            &[copy],
            Rule::Intersection,
        )?
        .answer() else {
            return Ok(());
        };
        prop_assert_eq!(invariants_of(&shared.0, &shared.1), itself);

        tally.comparing();
        same_answer(
            &answer_of(&united.0, &united.1),
            &answer_of(&shared.0, &shared.1),
            Rule::Union,
            "uniting a solid with a copy of itself against sharing with it",
        )?;
        Ok(())
    });
    tally.report_what_it_compared("idempotence, a lattice solid and a copy of it");
}

/// The one property in this file that needs no coordinate to be exactly
/// representable: an empty answer is empty whatever the arithmetic did.
///
/// So it runs over the turned family too, and the tally is what says whether it
/// saw anything there. Read that number rather than the pass — a family the
/// kernel refuses outright leaves this test asserting nothing at all, which is
/// §14.7's worry in its purest form. What keeps that visible instead of silent
/// is that the refusals are counted and printed rather than filtered away.
#[test]
fn a_solid_with_a_copy_of_itself_taken_away_is_nothing_at_all() {
    for (name, strategy) in [
        ("lattice", a_solid_and_its_copy().boxed()),
        ("turned", a_turned_solid_and_its_copy().boxed()),
    ] {
        let coordinates = if name == "turned" {
            Coordinates::AreUnderAGeneralRotation
        } else {
            Coordinates::AreExactlyRepresentable
        };
        let tally = over(CASES, strategy, |configuration, tally| {
            let (model, keys) = configuration.build();
            tally.meaning_to_compare();
            let Some(nothing) = fused(
                tally,
                coordinates,
                &model,
                &[keys[0]],
                &[keys[1]],
                Rule::Difference,
            )?
            .answer() else {
                return Ok(());
            };
            tally.comparing();
            prop_assert!(
                nothing.1.is_empty(),
                "a solid with itself taken away leaves nothing, got {} solids",
                nothing.1.len()
            );
            Ok(())
        });
        tally.report_what_it_compared(&format!("A \\ A = ∅, {name} solid and a copy of it"));
    }
}

#[test]
fn a_solid_united_with_what_it_shares_with_another_is_itself() {
    let tally = over(LONGER_CASES, a_lattice_pair(), |configuration, tally| {
        let (model, keys) = configuration.build();
        let (one, other) = (keys[0], keys[1]);
        tally.meaning_to_compare();

        let Some(shared) = fused(
            tally,
            Coordinates::AreExactlyRepresentable,
            &model,
            &[one],
            &[other],
            Rule::Intersection,
        )?
        .answer() else {
            return Ok(());
        };
        let Some(swallowed) = fused(
            tally,
            Coordinates::AreExactlyRepresentable,
            &shared.0,
            &[one],
            &shared.1,
            Rule::Union,
        )?
        .answer() else {
            return Ok(());
        };

        tally.comparing();
        prop_assert_eq!(
            invariants_of(&swallowed.0, &swallowed.1),
            invariants_of(&model, &[one]),
            "A ∪ (A ∩ B) is A"
        );
        Ok(())
    });
    tally.report_what_it_compared("absorption, lattice pairs");
}

#[test]
fn uniting_three_at_once_is_uniting_two_and_then_the_third() {
    let tally = over(LONGER_CASES, a_lattice_trio(), |configuration, tally| {
        let (model, keys) = configuration.build();
        tally.meaning_to_compare();

        let Some(at_once) = fused(
            tally,
            Coordinates::AreExactlyRepresentable,
            &model,
            &keys,
            &[],
            Rule::Union,
        )?
        .answer() else {
            return Ok(());
        };
        let Some(first_two) = fused(
            tally,
            Coordinates::AreExactlyRepresentable,
            &model,
            &keys[..2],
            &[],
            Rule::Union,
        )?
        .answer() else {
            return Ok(());
        };
        let mut then_the_third = first_two.1.clone();
        then_the_third.push(keys[2]);
        let Some(in_steps) = fused(
            tally,
            Coordinates::AreExactlyRepresentable,
            &first_two.0,
            &then_the_third,
            &[],
            Rule::Union,
        )?
        .answer() else {
            return Ok(());
        };

        tally.comparing();
        prop_assert_eq!(
            invariants_of(&at_once.0, &at_once.1),
            invariants_of(&in_steps.0, &in_steps.1),
            "fuse([A, B, C]) is fuse([fuse([A, B]), C])"
        );
        Ok(())
    });
    tally.report_what_it_compared("fuse([A,B,C]) = fuse([fuse([A,B]),C]), lattice trios");
}

/// `((A ∪ B) \ B) ∪ B` is `A ∪ B`, three fuses deep.
///
/// Deeper than anything else here, and each fuse hands the next one faces no
/// input ever had. The middle one is the interesting step: it takes the union
/// away again, so its argument is an answer whose faces were joined out of
/// pieces of both inputs, and what it leaves has to add back up.
#[test]
fn taking_a_solid_out_of_a_union_and_putting_it_back_gives_the_union_again() {
    let tally = over(LONGER_CASES, a_lattice_pair(), |configuration, tally| {
        let (model, keys) = configuration.build();
        let (one, other) = (keys[0], keys[1]);
        tally.meaning_to_compare();

        let Some(united) = fused(
            tally,
            Coordinates::AreExactlyRepresentable,
            &model,
            &[one],
            &[other],
            Rule::Union,
        )?
        .answer() else {
            return Ok(());
        };
        let Some(bitten) = fused(
            tally,
            Coordinates::AreExactlyRepresentable,
            &united.0,
            &united.1,
            &[other],
            Rule::Difference,
        )?
        .answer() else {
            return Ok(());
        };
        let Some(back) = fused(
            tally,
            Coordinates::AreExactlyRepresentable,
            &bitten.0,
            &bitten.1,
            &[other],
            Rule::Union,
        )?
        .answer() else {
            return Ok(());
        };

        tally.comparing();
        prop_assert_eq!(
            invariants_of(&back.0, &back.1),
            invariants_of(&united.0, &united.1),
            "((A ∪ B) \\ B) ∪ B is A ∪ B"
        );
        Ok(())
    });
    tally.report_what_it_compared("((A ∪ B) \\ B) ∪ B = A ∪ B, lattice pairs");
}

/// `fuse([A, B, C, D])` against uniting the four one at a time.
///
/// The deepest chain in the suite. The fourth fuse of the stepwise run is
/// looking at an answer three fuses old, which is where a corner with four or
/// more planes round it, a face joined out of coplanar pieces, and a shell built
/// by `nest_the_shells` all live at once.
#[test]
fn uniting_four_at_once_is_uniting_them_one_at_a_time() {
    let tally = over(LONGER_CASES, a_chain_of_fuses(), |configuration, tally| {
        let (model, keys) = configuration.build();
        tally.meaning_to_compare();

        let Some(at_once) = fused(
            tally,
            Coordinates::AreExactlyRepresentable,
            &model,
            &keys,
            &[],
            Rule::Union,
        )?
        .answer() else {
            return Ok(());
        };

        let mut so_far = (model, vec![keys[0]]);
        for &next in &keys[1..] {
            let Some(wider) = fused(
                tally,
                Coordinates::AreExactlyRepresentable,
                &so_far.0,
                &so_far.1,
                &[next],
                Rule::Union,
            )?
            .answer() else {
                return Ok(());
            };
            so_far = wider;
        }

        tally.comparing();
        prop_assert_eq!(
            invariants_of(&at_once.0, &at_once.1),
            invariants_of(&so_far.0, &so_far.1),
            "fuse([A, B, C, D]) is A ∪ B ∪ C ∪ D taken one at a time"
        );
        Ok(())
    });
    tally.report_what_it_compared_at_least(
        "fuse([A,B,C,D]) = uniting them one at a time, lattice chains",
        A_THINNER_LAW_STILL_COMPARES_THIS_MUCH,
    );
}

/// Uniting a turned solid with bit-identical copies of itself moves nothing
/// after the first union, however many copies follow.
///
/// **The one property in this file that is exactly true under a general
/// rotation.** Every copy is built from the same numbers by the same code, so
/// every copy carries bit-identical planes and bit-identical corners; the fuse
/// has nothing to round differently between them. That makes `A ∪ A ∪ A ∪ …` a
/// bit-for-bit claim rather than one needing a tolerance, which is why this is
/// the shape of the test rather than a volume comparison.
///
/// **Why it is measured from the first union and not from `A`.** §17.1 says the
/// plane is authoritative and a stored corner is a cached sample of it, so the
/// first fuse re-derives every corner as the meet of the planes around it and is
/// entitled to land an ulp away from the corner the input had cached. Measured
/// on a unit block turned one sixty-fourth of a turn: the input caches the far
/// corner's `y` as `1.0932018670017576`, the meet of that corner's three stored
/// planes is `1.09320186700175740…`, and the fuse stores the `f64` nearest that,
/// which is `1.0932018670017574`. The input's cached corner is the ulp that is
/// wrong, and correcting it is the whole point of re-deriving. What §17.1 forbids
/// is that correction *repeating*: "the rounding in a vertex is the rounding of
/// one Cramer solve — never a sum over the operation history". So the second
/// union and every union after it has to change nothing at all, and that is what
/// this asserts.
///
/// The other half of the claim — that the planes never move, at any step, from
/// the ones the input carried — is [`every_surface_of_the_answer_is_a_surface_of_an_input`],
/// which runs inside [`fused`] on all of these fuses. It could not until
/// `Plane::transformed` stopped re-deriving a moved plane's offset through a
/// point on it: `FaceView::plane` is `surface.transformed(&location)`, so that
/// ulp was charged on every read and a chain of fuses walked every
/// non-axis-aligned plane one more ulp per fuse. `rcad-geom`'s
/// `moving_a_plane_by_the_identity_twenty_times_leaves_every_coefficient_alone`
/// is the two-line version of the same statement.
#[test]
fn uniting_a_turned_solid_with_more_copies_of_itself_settles_after_the_first_union() {
    let tally = over(
        LONGER_CASES,
        a_turned_solid_and_several_copies(),
        |configuration, tally| {
            let (model, keys) = configuration.build();
            let Some(mut so_far) = fused(
                tally,
                Coordinates::AreUnderAGeneralRotation,
                &model,
                &[keys[0]],
                &[keys[1]],
                Rule::Union,
            )?
            .answer() else {
                return Ok(());
            };
            let settled = answer_of(&so_far.0, &so_far.1);

            for &copy in &keys[2..] {
                tally.meaning_to_compare();
                let Some(wider) = fused(
                    tally,
                    Coordinates::AreUnderAGeneralRotation,
                    &so_far.0,
                    &so_far.1,
                    &[copy],
                    Rule::Union,
                )?
                .answer() else {
                    return Ok(());
                };
                tally.comparing();
                same_answer(
                    &settled,
                    &answer_of(&wider.0, &wider.1),
                    Rule::Union,
                    "uniting a turned solid with another bit-identical copy of itself",
                )?;
                so_far = wider;
            }
            Ok(())
        },
    );
    tally.report_what_it_compared("A ∪ A ∪ A ∪ …, a turned solid and copies of it");
}

/// §17.7's control row, and §14.7's acceptance oracle for the fuse.
///
/// Every meet in a rectilinear configuration on the integer grid is exactly
/// representable, so a certified policy must decide all of it. One refusal is
/// one bug in our filters, and the empty log says the same thing a second way:
/// nothing here was decided by a policy accepting an enclosure, because nothing
/// here needed to be.
#[test]
fn every_certified_fuse_of_a_lattice_pair_is_decided_and_records_nothing() {
    let tally = measured(a_lattice_pair(), |configuration, tally| {
        every_rule(tally, Coordinates::AreExactlyRepresentable, configuration)
    });
    tally.report("§17.7 control: lattice pairs, every rule");
    tally.rate_of_answers_that_were_not_solids_must_be_within(A_LATTICE_PAIR_IS_NOT_A_SOLID);
    assert_eq!(
        tally.refused.get(),
        0,
        "a lattice-aligned fuse that refuses is a bug in our filters, not a fact about geometry"
    );
}

/// The lattice band at the two sizes §303 measured, without running a soak.
///
/// §290.4's two soak draws, 555 and 556 of 25 401, fell under the old fixed
/// floor of 2.2% while the family's rate is 2.23%. Judged against the rate with
/// the draw's own spread, they and the lowest and highest of §303's 113 soaked
/// seeds are inside, and a draw at half the rate is not. At the shipped 441
/// fuses, none at all still fires. No lattice configuration gives two answers
/// that are not solids (§303.6), so they come together one at a time.
#[test]
fn the_lattice_band_takes_in_every_soaked_seed_and_still_fires_on_half_the_rate() {
    let share = |not_a_solid: usize, fuses: usize| not_a_solid as f64 / fuses as f64;
    let soak = band_for_a_draw_of(25_401, 1.0, &A_LATTICE_PAIR_IS_NOT_A_SOLID);
    for not_a_solid in [519, 555, 556, 615] {
        assert!(
            soak.contains(&share(not_a_solid, 25_401)),
            "{not_a_solid} of 25 401 is a draw §303 measured, and it is outside {soak:?}"
        );
    }
    assert!(
        !soak.contains(&share(283, 25_401)),
        "283 of 25 401 is half the lattice family's rate, and {soak:?} takes it in"
    );
    let shipped = band_for_a_draw_of(441, 1.0, &A_LATTICE_PAIR_IS_NOT_A_SOLID);
    assert!(shipped.contains(&share(14, 441)));
    assert!(shipped.contains(&share(1, 441)));
    assert!(
        !shipped.contains(&share(0, 441)),
        "a family that no longer reaches four faces along one edge at all has to fire"
    );
}

/// The turned family's configurations as `4066f85b` fused them, eight seeds
/// at a [`SOAK_FACTOR`] of 66: 2 286 gave one answer that was not a solid,
/// 537 gave two and 539 all three. On that tree a draw's count spread 1.23
/// times as wide as the binomial's, over 200 fresh draws of 128
/// configurations, and that is the spread the band has to allow (§303.6).
/// Counted from those configurations, each such answer came with 1.87 of
/// them, itself included, so the band's spread is the binomial's times the
/// square root of that.
#[test]
fn answers_that_are_not_solids_coming_three_to_a_configuration_widen_the_band() {
    let tally = Tally::default();
    for (of_three, configurations) in [(0, 64_374), (1, 2_286), (2, 537), (3, 539)] {
        for _ in 0..configurations {
            tally.a_configuration_was_not_a_solid_this_often(of_three);
        }
        tally.fuses.set(tally.fuses.get() + 3 * configurations);
        tally
            .not_a_solid
            .set(tally.not_a_solid.get() + of_three * configurations);
    }
    let together = tally.how_many_not_solids_come_together();
    assert_eq!(together, 9_285.0 / 4_977.0);

    let rate = 0.016..=0.073;
    let stray =
        |together: f64| band_for_a_draw_of(tally.fuses.get(), together, &rate).end() - rate.end();
    let widened = stray(together) / stray(1.0);
    assert!(
        (widened - together.sqrt()).abs() < 1e-12,
        "the ceiling moved out {widened} times as far, not the square root of {together}"
    );
}

/// A draw whose answers that are not solids were not counted configuration
/// by configuration cannot say how they came together, so it cannot be judged.
#[test]
#[should_panic(expected = "counted with its configuration")]
fn an_answer_that_is_not_a_solid_counted_outside_its_configuration_stops_the_band() {
    let tally = Tally::default();
    tally.fuses.set(3);
    tally.not_a_solid.set(1);
    tally.how_many_not_solids_come_together();
}

/// The same for the family built to be degenerate. Coplanar faces and shared
/// corners are exactly representable too, so the control claim holds here as
/// well — what changes is how many of these answers are not solids at all.
#[test]
fn every_certified_fuse_of_a_pair_sharing_coordinates_is_decided() {
    let tally = measured(a_flush_pair(), |configuration, tally| {
        every_rule(tally, Coordinates::AreExactlyRepresentable, configuration)
    });
    tally.report("§17.7 degenerate: pairs sharing coordinates, every rule");
    tally.rate_of_answers_that_were_not_solids_must_be_within(A_FLUSH_PAIR_IS_NOT_A_SOLID);
    assert_eq!(
        tally.refused.get(),
        0,
        "two solids sharing coordinates exactly leave nothing that cannot be decided"
    );
}

/// A fourth measured row for §17.7: the lattice pairs written somewhere else in
/// `f64`.
///
/// A control, and the strictest one in the file. Multiplying by a power of two
/// and shifting by a power of two are both exact, so every meet is exactly
/// representable at every size the family reaches — a refusal is a bug in our
/// filters at that size, which is a sharper statement than the same sentence
/// about the lattice family alone, because the cascade does not decide these
/// fuses at the same level.
#[test]
fn every_certified_fuse_of_a_pair_somewhere_else_in_f64_is_decided() {
    let tally = measured(a_pair_somewhere_else(), |configuration, tally| {
        every_rule(tally, Coordinates::AreExactlyRepresentable, configuration)
    });
    tally.report("§17.7 control: pairs somewhere else in f64, every rule");
    tally.rate_of_answers_that_were_not_solids_must_be_within(A_PAIR_ELSEWHERE_IS_NOT_A_SOLID);
    assert_eq!(
        tally.refused.get(),
        0,
        "moving a configuration by a power of two cannot make it undecidable"
    );
}

/// §17.7's headline rate, and the one number in this file that is a measurement
/// rather than a check.
///
/// It runs on a fixed seed so that the number is the same every time it is
/// asked for and the figure written into §17.7 reproduces. The properties above
/// run on a fixed seed too, since §339: finding inputs nobody has thought of is
/// the soak's job, and a property that goes red or green by the draw cannot say
/// which commit broke it.
///
/// The refusals are broken down by which predicate gave up, because the rate on
/// its own cannot tell a timid filter from genuine undecidability (§14.7), and
/// the breakdown is the half anybody could act on. One thing it makes plain is
/// worth carrying to criterion 8: this family is the lattice family turned, and
/// the lattice family contains a great many configurations that are *exactly*
/// degenerate — faces flush against each other, corners on faces. Turning those
/// leaves them geometrically degenerate while their two stored planes stop
/// agreeing in the last bits, which is where most of the refusals come from. So
/// the rate is as much a statement about how degenerate the corpus is as about
/// the filters, and the two families have to be read together.
///
/// **Since Q1288 it is 39 of 447 refused, from 45**, all 45 then under *a
/// point inside a piece of a face*. A plane of one solid now places a piece
/// with every corner on or in front of it and one strictly, so a piece a
/// rounding across whose corners lie on that solid's planes is placed. 33
/// still name the point, and 6 are [`AN_EDGE_TOO_SHORT_BESIDE_ITS_COORDINATES`]:
/// fuses that get as far as an answer and find its shortest edge `7.7e15` to
/// `2.9e16` times shorter than its largest coordinate, a limit
/// ([`a_flat_refusal`]).
#[test]
fn how_often_a_certified_fuse_of_a_turned_pair_refuses() {
    let tally = measured(a_turned_pair(), |configuration, tally| {
        every_rule(tally, Coordinates::AreUnderAGeneralRotation, configuration)
    });
    tally.report("§17.7 headline: turned pairs, every rule");
    tally.rate_of_answers_that_were_not_solids_must_be_within(A_TURNED_PAIR_IS_NOT_A_SOLID);

    let refused = tally.refused.get() as f64 / tally.fuses.get() as f64;
    if this_is_the_shipped_corpus() {
        assert!(
            refused < TURNED_REFUSALS_MUST_STAY_BELOW,
            "{:.1}% of {} turned fuses refused, over the {:.0}% §17.7 records",
            refused * 100.0,
            tally.fuses.get(),
            TURNED_REFUSALS_MUST_STAY_BELOW * 100.0
        );
    }
}

/// §22's headline: the one family whose two solids are not square to each other.
///
/// The tilt and the arrangement are two different costs and this row is both of
/// them added together, which is why it is never read without
/// [`how_often_a_certified_fuse_of_a_pair_leaning_the_same_way_refuses`] beside
/// it. That row holds the arrangement fixed and prices the tilt alone; the
/// difference is what leaning the two solids *differently* costs.
///
/// The breakdown is the half anybody can act on (§14.7), and here it says
/// something the total cannot. §17.7 records that every refusal the turned
/// family has ever produced named `"side of plane"`, with
/// `"a point inside a piece of a face"` turning up rarely and no seed ever
/// naming a third. This family's leading cause was `"order along a crossing"`,
/// which that corpus essentially never reaches — and reaching it is the point.
/// Three cut lines through one point put two intersections at the same place
/// on a line, and their two boxes overlap, so no reading of the boxes can put
/// them in order.
///
/// **The planes can, and since Q901 they are asked.** Each of the two is the
/// meet of the face's plane, the crossing's partner and one plane more, and
/// `which_meet_comes_first_along_a_line` settles their order exactly, a tie
/// included. On the shipped corpus that took every refusal off the order: 441
/// fuses, 39 refused then, and all 39 named `"a point inside a piece of a
/// face"`, one step further on. That is what the cause below now pins.
///
/// **Since Q1145 it is 27 refused, all 27 still under that name.** The 33 the
/// corpus refused just before each stopped first, as Q1069 traced them, on a
/// flat piece where three cut lines meet at one point, which leaves a side a
/// rounding long, and a
/// turn about that side settled nothing in the fan that was to prove a point
/// inside. Two turns of opposite sign now put the point outside a triangle of
/// the fan without it. `tests/three_cut_lines_through_one_corner_of_a_skew_pair.rs`
/// pins the two pairs that answer now, all six of their fuses, and a third
/// pair that got past that piece and stopped on the next.
///
/// **Since Q1279 it is 16 of 447 refused, from 30**, all 30 then under that
/// name. Q1146 found the stops on slivers that hold no `f64` point, where
/// three planes meet by design, and a plane of one solid now places a piece
/// with every corner on or in front of it and one strictly. 15 still name the
/// point, and one is [`AN_EDGE_TOO_SHORT_BESIDE_ITS_COORDINATES`], at a spread
/// of `5 · 2^51`, which is what the third pair's union is refused at: an edge
/// `2^-51` long beside a coordinate of 5, a limit ([`a_flat_refusal`]).
///
/// **Since Q1357 it is 13 of 447, from 16**: 12 name the point, and the edge
/// too short is still one. The three are the rules of one pair, a comb and a
/// leaning ell standing on its top, which stopped on a triangle a rounding
/// across in the plane of the ell's floor and well off it. A flat piece no
/// other rule places now takes the placement of the piece of its own face
/// across a side that no plane of the other solid holds but the face's own
/// (`tests/a_sliver_on_the_plane_of_a_face_it_misses.rs`).
#[test]
fn how_often_a_certified_fuse_of_a_skew_pair_refuses() {
    let tally = measured(a_skew_pair(), |configuration, tally| {
        every_rule(tally, Coordinates::SitOnTiltedWalls, configuration)
    });
    tally.report("§22 headline: skew pairs, every rule");
    tally.share_of_answers_that_were_not_solids_must_be_within(A_SKEW_PAIR_IS_NOT_A_SOLID);
    assert!(
        tally.not_a_solid.get() >= A_SKEW_PAIR_IS_NOT_A_SOLID_AT_LEAST,
        "none of {} skew fuses came back not a solid; the family no longer reaches four faces \
         along one edge",
        tally.fuses.get()
    );

    let refused = tally.refused.get() as f64 / tally.fuses.get() as f64;
    if this_is_the_shipped_corpus() {
        assert!(
            refused < SKEW_REFUSALS_MUST_STAY_BELOW,
            "{:.1}% of {} skew fuses refused, over the {:.0}% §22 records",
            refused * 100.0,
            tally.fuses.get(),
            SKEW_REFUSALS_MUST_STAY_BELOW * 100.0
        );
        tally.the_cause_named_most_often_is(A_POINT_INSIDE_A_PIECE);
    }
}

/// §22's control, and the row that makes the headline mean anything.
///
/// Both solids lean the same way, so the pair is an affine image of a lattice
/// pair: the same shapes overlapping in the same way, the same cut lines on
/// every face in the same order. The lattice row decides all of it and refuses
/// zero. The only thing left between the two rows is that the walls are on
/// tilted planes, whose unit normals are rounded and cannot be otherwise.
///
/// So this is not asserted to be zero, and it should not be. What it is for is
/// the subtraction: whatever this row costs is the price of the tilt, and the
/// headline row minus this one is the price of the arrangement.
#[test]
fn how_often_a_certified_fuse_of_a_pair_leaning_the_same_way_refuses() {
    let tally = measured(a_pair_leaning_the_same_way(), |configuration, tally| {
        every_rule(tally, Coordinates::SitOnTiltedWalls, configuration)
    });
    tally.report("§22 control: both leaning alike, every rule");
    tally.rate_of_answers_that_were_not_solids_must_be_within(A_PAIR_LEANING_ALIKE_IS_NOT_A_SOLID);
}

/// Commutativity over the one family that can put three cut lines through a
/// point.
///
/// The sharpest question in the file (§17.4) asked of the arrangement that had
/// never been asked it. A splitter that cut a face by one line and then cut the
/// pieces by the next computes a different partition depending which line it
/// took first — and where three lines are concurrent, "which first" is a tie
/// rather than a comparison, so the two ways round are as far apart as this
/// suite can push them.
///
/// **Every last bit**, like every other family here.
///
/// It compared the corners only for as long as §22.6's defect stood: two walls
/// of the two inputs could be the same plane and carry coefficients an ulp
/// apart, so the fuse kept whichever belonged to whichever argument came first,
/// and the answer depended on the order. The plane is canonical now — see
/// [`the_same_wall_built_from_a_longer_edge_is_the_same_plane`] — so the last
/// bit of a coefficient is a fair thing to ask about, and the diff was the one
/// word that section predicted.
///
/// **Since Q1279 a fuse here may be refused for an answer with an edge too
/// short beside its coordinates**, counted by name ([`a_flat_refusal`]) rather
/// than failed: 2 of its 276 fuses are, and 22 are refused in all, from 28.
#[test]
fn swapping_the_two_sides_of_a_skew_pair_changes_not_one_bit() {
    let tally = over(CASES, a_skew_pair(), |configuration, tally| {
        both_ways_round(tally, Coordinates::SitOnTiltedWalls, configuration)
    });
    tally.report_what_it_compared("A ∪ B against B ∪ A, skew pairs");
}

/// §31.7 criterion 15's first name, over the curved family.
///
/// The sharpest question in the file asked of the first family whose corners are
/// not exactly representable *and* whose faces are not all flat. Swapping the
/// two sides sends the same pieces down different rows of the selection table
/// and hands the arrangement its cut surfaces the other way round, and the
/// answer has to be the same one down to the last bit of the cylinder's radius.
///
/// **It really compares things**, and that is not something a curved family can
/// be assumed to do (§14.7, and lane 5c's finding that a family can read clean
/// because it never reaches the hard case). Two things say so here: the family's
/// own measured row is at nothing refused, so both sides of every comparison
/// arrive, and [`Coordinates::LieOnARoundWall`] makes a refusal a failure rather
/// than a skipped case.
#[test]
fn swapping_the_two_sides_of_a_curved_pair_changes_not_one_bit() {
    let tally = over(CASES, a_curved_pair(), |pair, tally| {
        // Union and intersection only, for the reason `both_ways_round` runs
        // those two next door: `A \ B` and `B \ A` are different questions and
        // nothing says their answers agree.
        for rule in [Rule::Union, Rule::Intersection] {
            tally.meaning_to_compare();
            let (model, keys) = pair.build();
            let (block, bar) = (keys[0], keys[1]);

            let Some(forwards) = fused(
                tally,
                Coordinates::LieOnARoundWall,
                &model,
                &[block],
                &[bar],
                rule,
            )?
            .answer() else {
                continue;
            };
            let Some(backwards) = fused(
                tally,
                Coordinates::LieOnARoundWall,
                &model,
                &[bar],
                &[block],
                rule,
            )?
            .answer() else {
                continue;
            };

            tally.comparing();
            same_answer(
                &answer_of(&forwards.0, &forwards.1),
                &answer_of(&backwards.0, &backwards.1),
                rule,
                "swapping the two sides of a curved pair",
            )?;
        }
        Ok(())
    });
    tally.report_what_it_compared("commutativity, a bar through a block");
}

/// §31.7 criterion 15's third name, at the one pair it is about.
///
/// A block and a bar through it, united each way round, at a fixed size rather
/// than a generated one, so that a failure names a shape rather than a seed. The
/// same claim as the property above at one point of its domain — and worth its
/// own test because it is the criterion's own name and the criterion should be
/// findable.
#[test]
fn uniting_a_block_and_a_bar_the_other_way_round_gives_the_same_solid() {
    let pair = CurvedPair {
        side: 8,
        radius: 2,
        at: [4, 4],
    };
    for rule in [Rule::Union, Rule::Intersection] {
        let (model, keys) = pair.build();
        let (block, bar) = (keys[0], keys[1]);
        let forwards = plainly_fused(&model, &[block], &[bar], rule);
        let backwards = plainly_fused(&model, &[bar], &[block], rule);
        same_answer(
            &answer_of(&forwards.0, &forwards.1),
            &answer_of(&backwards.0, &backwards.1),
            rule,
            "uniting a block and a bar the other way round",
        )
        .expect("the same pair fused the other way round is the same answer");
    }
}

/// §31.7 criterion 15's second name, over a bar: a bar with a bit-identical
/// copy of itself taken away leaves nothing at all.
///
/// This was pinned as a refusal on "a point in a piece that every solid could
/// place" until §224.4, which found it was the bar's flat caps and not its
/// wall: a cap's one corner winds round nothing. The control is the same
/// question with the bar replaced by a block.
#[test]
fn a_bar_with_a_copy_of_itself_taken_away_is_nothing_at_all() {
    let pair = CurvedPair {
        side: 8,
        radius: 2,
        at: [0, 0],
    };

    let (mut model, keys) = pair.built_as_a_block_and_a_copy_of_it();
    let nothing = model
        .fuse(
            &[keys[0]],
            &[keys[1]],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a block with a copy of itself taken away is an ordinary answer");
    assert!(
        nothing.solids().is_empty(),
        "the control: a block with a copy of itself taken away leaves nothing"
    );

    let (model, keys) = pair.built_as_a_bar_and_a_copy_of_it();
    let nothing = plainly_fused(&model, &[keys[0]], &[keys[1]], Rule::Difference);
    assert!(
        nothing.1.is_empty(),
        "a bar with a copy of itself taken away leaves nothing"
    );
}

/// The same three questions of a **ball**, and the ball answers all three where
/// the bar answers none.
///
/// Idempotence is one of §48.3's laws and nothing had asked it of a round solid.
/// A ball with a bit-identical copy of itself taken away leaves nothing at all,
/// united with it is itself, and shared with it is itself — the whole of what
/// [`a_solid_united_with_a_copy_of_itself_is_itself`] and
/// [`a_solid_with_a_copy_of_itself_taken_away_is_nothing_at_all`] assert over
/// the flat families, asserted here for real rather than pinned.
///
/// The bar refused `A \ A` on `"a point in a piece that every solid could
/// place"` until §224.4 and the ball never did. That read as a statement about a surface that
/// closes round a seam, and it was not: the ball has no flat face, and the
/// bar's refusal was its flat caps, each a disk with one corner that winding
/// counted as holding nothing.
#[test]
fn a_ball_and_a_copy_of_it_unite_share_and_cancel_the_way_a_solid_should() {
    let pair = BallAndBlock {
        side: 8,
        radius: 2,
        at: [4, 4],
    };

    let (model, keys) = pair.built_as_a_ball_and_a_copy_of_it();
    let itself = invariants_of(&model, &[keys[0]]);

    let nothing = plainly_fused(&model, &[keys[0]], &[keys[1]], Rule::Difference);
    assert!(
        nothing.1.is_empty(),
        "a ball with a copy of itself taken away leaves nothing, got {} solids",
        nothing.1.len()
    );

    let united = plainly_fused(&model, &[keys[0]], &[keys[1]], Rule::Union);
    assert_eq!(
        invariants_of(&united.0, &united.1),
        itself,
        "a ball united with a copy of itself is itself"
    );

    let shared = plainly_fused(&model, &[keys[0]], &[keys[1]], Rule::Intersection);
    assert_eq!(
        invariants_of(&shared.0, &shared.1),
        itself,
        "a ball sharing with a copy of itself is itself"
    );

    same_answer(
        &answer_of(&united.0, &united.1),
        &answer_of(&shared.0, &shared.1),
        Rule::Union,
        "uniting a ball with a copy of itself against sharing with it",
    )
    .expect("the two are the same answer, bit for bit");
}

/// An empty answer over a curved pair is a success and not an error.
///
/// §17.4 settles that for the flat case and nothing had asked it of a curved
/// one. The bar stands a whole unit clear of the block along `z`, so what the
/// two share is nothing, and "nothing" has to arrive as `Ok` with no solids
/// rather than as a refusal or as a solid enclosing no space.
#[test]
fn what_a_curved_pair_that_misses_shares_is_nothing_and_that_is_a_success() {
    let mut model = Model::new(rcad_geom::Scale::millimetres(100.0).expect("a working size"));
    let block = model
        .add_cube(8.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a block eight across");
    let bar = model
        .add_round_bar(
            2.0,
            4.0,
            &rcad_geom::Isometry::translation(Vector::new(4.0, 4.0, 9.0)).expect("a shift"),
        )
        .expect("a bar standing clear above it");

    let mut log = ProvenanceLog::new();
    let shared = model
        .fuse(
            &[block],
            &[bar],
            Rule::Intersection,
            &Policy::certified(),
            &mut log,
        )
        .expect("two solids that miss each other share nothing, which is an answer");
    assert!(shared.solids().is_empty());
    assert!(log.records().is_empty(), "nothing here had to be settled");
}

/// A ball half sunk into a block fuses at all, which nothing had asked.
///
/// The floor of the second curved family, landed by the dispatcher of §48 ahead
/// of the lane that wires it into the properties, so the lane starts against a
/// fixture that is known to work rather than one it has to debug first. It says
/// only that the three rules answer and that what comes back is a solid — the
/// laws themselves are §48.4's L1.
#[test]
fn a_ball_half_sunk_into_a_block_fuses_under_every_rule() {
    let pair = BallAndBlock {
        side: 8,
        radius: 2,
        at: [4, 4],
    };

    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        let (mut model, keys) = pair.build();
        let answer = model
            .fuse(
                &keys[..1],
                &keys[1..],
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("a {rule:?} of a ball and a block: {why}"));
        assert!(
            !answer.solids().is_empty(),
            "a {rule:?} of a ball half sunk into a block leaves something behind"
        );
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("what a {rule:?} left is not a sound model: {why}"));
    }
}

/// Swapping the two sides of a ball and a block, bit for bit.
///
/// **§48.3's sentence, at the first law that keeps it.** The curved family that
/// ran commutativity before this one is a pipe, whose wall curves one way; a
/// ball's curves both, and where its skin meets the block's floor is a whole
/// circle rather than two arcs. Swapping the sides sends the same pieces down
/// different rows of the selection table and the answer has to come back the
/// same down to the last bit of the ball's radius.
///
/// [`Coordinates::LieOnARoundWall`] makes a refusal a failure and
/// [`every_certified_fuse_of_a_ball_and_a_block_is_decided_and_records_nothing`]
/// is the row that says both sides really arrive, so this is not a family
/// reading clean by never reaching its subject (§14.7).
#[test]
fn swapping_the_two_sides_of_a_block_and_a_ball_changes_not_one_bit() {
    let tally = over(CASES, a_ball_and_a_block(), |pair, tally| {
        for rule in [Rule::Union, Rule::Intersection] {
            tally.meaning_to_compare();
            let (model, keys) = pair.build();
            let (block, ball) = (keys[0], keys[1]);

            let forwards = fused(
                tally,
                Coordinates::LieOnARoundWall,
                &model,
                &[block],
                &[ball],
                rule,
            )?;
            let backwards = fused(
                tally,
                Coordinates::LieOnARoundWall,
                &model,
                &[ball],
                &[block],
                rule,
            )?;
            both_agree_about_whether_there_is_an_answer(
                tally,
                Coordinates::LieOnARoundWall,
                &forwards,
                &backwards,
                rule,
                "swapping the two sides",
            )?;
            let (Some(forwards), Some(backwards)) = (forwards.answer(), backwards.answer()) else {
                continue;
            };

            tally.comparing();
            same_answer(
                &answer_of(&forwards.0, &forwards.1),
                &answer_of(&backwards.0, &backwards.1),
                rule,
                "swapping the two sides of a ball and a block",
            )?;
        }
        Ok(())
    });
    tally.report_what_it_compared("commutativity, a ball in a block");
}

/// The ball family's measured row, and what makes the row above worth its pass.
///
/// The same discipline as [`every_certified_fuse_of_a_curved_pair_is_decided_and_records_nothing`]
/// next door: a fixed seed, every rule, a refusal is a failure, and no band on
/// answers that are not solids because there are none.
#[test]
fn every_certified_fuse_of_a_ball_and_a_block_is_decided_and_records_nothing() {
    let tally = measured(a_ball_and_a_block(), |pair, tally| {
        let (model, keys) = pair.build();
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            fused(
                tally,
                Coordinates::LieOnARoundWall,
                &model,
                &[keys[0]],
                &[keys[1]],
                rule,
            )?;
        }
        Ok(())
    });
    tally.report("§48 control: a ball half sunk into a block, every rule");
    assert_eq!(
        tally.refused.get(),
        0,
        "a ball sitting on a block's floor, clear of every side wall, is decided by proof alone"
    );
    assert_eq!(
        tally.not_a_solid.get(),
        0,
        "no arrangement of this family puts four faces along one edge"
    );
}

/// Inclusion and exclusion over a round pair: `vol(A ∪ B) + vol(A ∩ B)` is
/// `vol(A) + vol(B)`, asserted to a **stated share** of what the four
/// measurements declare.
///
/// The lattice row asserts the same law exactly, and the reason it can is at the
/// top of this file: a rectilinear solid's six-volume is a whole number reached
/// with no rounding. A ball has no such volume. `Solid::measure` meshes a curved
/// skin to a [`Deflection`], and the mesh of a whole ball is not the mesh of its
/// two halves put together, so the four volumes miss each other by an amount
/// that is a fact about the mesh.
///
/// **Each of the four says how far out it is**, though, and that is what makes
/// the law assertable rather than only measurable: `Measurement::volume_error`
/// is a bound on how much material one meshed solid hides, so the four sides
/// added up bound the law's residual. The summed bound is not what is asserted
/// against — see [`A_CURVED_LAW_MAY_USE_THIS_MUCH_OF_WHAT_THE_FOUR_DECLARE`] for
/// why that would be vacuous and where the share comes from.
///
/// **The block is flat and declares nothing**, so three of the four terms carry
/// the tolerance and the fourth contributes exactly zero — which is the honest
/// shape of it, since a cube's volume is exact.
#[test]
fn what_a_block_and_a_ball_take_up_adds_up_inside_a_stated_share_of_what_they_declare() {
    let worst = Cell::new(0.0f64);
    let worst_at = RefCell::new(String::new());
    let grid: RefCell<BTreeMap<(u32, String), f64>> = RefCell::new(BTreeMap::new());
    let tally = over(CASES, a_ball_and_a_block(), |pair, tally| {
        let (model, keys) = pair.build();
        let (block, ball) = (keys[0], keys[1]);
        tally.meaning_to_compare();

        let coordinates = Coordinates::LieOnARoundWall;
        let Some(united) =
            fused(tally, coordinates, &model, &[block], &[ball], Rule::Union)?.answer()
        else {
            return Ok(());
        };
        let Some(shared) = fused(
            tally,
            coordinates,
            &model,
            &[block],
            &[ball],
            Rule::Intersection,
        )?
        .answer() else {
            return Ok(());
        };
        tally.comparing();

        for how_fine in HOW_FINE_THE_LAW_IS_ASKED_AT {
            let deflection = Deflection::of(how_fine).expect("a length");
            let sides = [
                how_much_it_holds(&united.0, &united.1, deflection),
                how_much_it_holds(&shared.0, &shared.1, deflection),
                how_much_it_holds(&model, &[block], deflection),
                how_much_it_holds(&model, &[ball], deflection),
            ];
            let out_by = sides[0].0 + sides[1].0 - sides[2].0 - sides[3].0;
            let declared: f64 = sides.iter().map(|(_, error)| error).sum();

            // §47.4 item 1: a tolerance nothing can exceed has measured nothing,
            // and an infinite one is the plainest form of that. A ball's skin
            // bends at its own radius, which is one or two here and never near
            // the deflection, so this is a guard and not a case.
            prop_assert!(
                declared.is_finite() && declared > 0.0,
                "at a deflection of {} the four measurements declared {declared}, and \
                 nothing is worth asserting against that",
                how_fine
            );
            let share = out_by.abs() / declared;
            {
                let mut grid = grid.borrow_mut();
                let cell = grid
                    .entry((pair.radius, format!("{how_fine}")))
                    .or_insert(0.0);
                *cell = cell.max(share);
            }
            if share > worst.get() {
                worst.set(share);
                *worst_at.borrow_mut() = format!("{pair:?} at a deflection of {how_fine}");
            }
            prop_assert!(
                share < A_CURVED_LAW_MAY_USE_THIS_MUCH_OF_WHAT_THE_FOUR_DECLARE,
                "at a deflection of {how_fine} inclusion and exclusion is {out_by} out over \
                 {pair:?}, which is {share} of the {declared} the four measurements declared \
                 and over the share this family has ever used"
            );
        }
        Ok(())
    });
    tally.report_what_it_compared("inclusion and exclusion, a ball in a block");
    println!(
        "    the worst share of the declared error used was {:.4} at {}",
        worst.get(),
        worst_at.borrow()
    );
    for ((radius, how_fine), share) in grid.borrow().iter() {
        println!("    radius {radius} at {how_fine}: {share:.5}");
    }
}

/// The deflections the law above is asked at.
///
/// Three over two decades, and more than one because **the share is not monotone
/// in the deflection and the worst one is at a different deflection for each
/// radius the family draws.** Measured over five deflections from `0.3` to
/// `0.003`, the share of the declared error the residual uses runs:
///
/// ```text
/// deflection    0.3     0.1     0.03    0.01    0.003
/// radius 1     0.0250  0.0084  0.0059  0.0048  0.0052
/// radius 2     0.0053  0.0154  0.0218  0.0180  0.0042
/// ```
///
/// A single deflection would have found a third of the worst. `0.003` is left
/// out because it uses less of the bound than anything else and costs the file
/// four seconds of wall clock.
const HOW_FINE_THE_LAW_IS_ASKED_AT: [f64; 3] = [0.3, 0.03, 0.01];

/// How much of what the four measurements declare inclusion and exclusion over a
/// round pair is allowed to use.
///
/// **The whole point of this number is that the declared error is not a
/// tolerance.** Each of the four sides is out by at most its own
/// `Measurement::volume_error`, so the four added up bound the residual — and
/// they bound it so loosely that asserting against the sum would be vacuous in
/// §47.4 item 1's sense even though the sum is finite.
/// `what_a_deflection_costs_a_measured_volume.rs` measured why: the four sides
/// charge the same shells twice with opposite signs and they cancel, so the sum
/// runs 70 to 150 times the residual for a block and a bar. A law asserted at
/// the sum would pass a fuse a hundred times more wrong than this one.
///
/// So the share is measured over this family rather than copied from that one.
/// The worst of the grid in [`HOW_FINE_THE_LAW_IS_ASKED_AT`] is **0.0250** — a
/// ball of radius one at a deflection of `0.3` — and one twentieth is a
/// **margin of two** over it. That is the same margin
/// `what_a_deflection_costs_a_measured_volume.rs` keeps over the same kind of
/// number next door, and it is **twenty times tighter than the summed bound**: a
/// fuse that mislaid a twentieth of the ball would fail this and pass that.
///
/// **Two is a thin margin and it is affordable here**, which is worth saying
/// out loud because it is not affordable everywhere. The share turns out to be a
/// function of the ball's radius and the deflection and almost nothing else —
/// eight fresh-seed runs move it in the third significant figure, because the
/// block is flat and declares nothing, and where the ball stands on the floor
/// changes what is meshed but not how much of it hides. The family draws only
/// two radii, so **every corpus enumerates the whole grid** and there is no
/// unlucky draw for a margin to have to cover. What the margin has to cover is a
/// change to the mesher, and a change to the mesher is a thing somebody should
/// look at.
const A_CURVED_LAW_MAY_USE_THIS_MUCH_OF_WHAT_THE_FOUR_DECLARE: f64 = 0.05;

/// How much space an answer takes up, and how much of it the mesh could be
/// hiding, added up over the answer's solids.
///
/// Two numbers rather than one because the second is what makes the first worth
/// asserting anything about: `Measurement::volume_error` is the mesher's own
/// statement of how much material sits between its triangles and the surface
/// they stand in for.
fn how_much_it_holds(model: &Model, solids: &[SolidKey], how_fine: Deflection) -> (f64, f64) {
    solids
        .iter()
        .map(|&solid| {
            let measured = model
                .solid(solid)
                .expect("this model made the key")
                .measure(how_fine, &Policy::certified(), &mut ProvenanceLog::new())
                .expect("a solid this size measures");
            (measured.volume(), measured.volume_error())
        })
        .fold((0.0, 0.0), |(volume, error), (more, worse)| {
            (volume + more, error + worse)
        })
}

/// How fast the gap the row above asserts a share of actually closes.
///
/// The share says the residual is small against what the mesh declares. This
/// says the residual is the **mesh** and not the fuse, which is a different
/// statement and needs a different measurement: **the gap falls by more than a
/// hundredfold when the deflection falls a hundredfold**. A fuse that had lost
/// material would leave a gap that stayed where it was.
#[test]
fn what_a_block_and_a_ball_take_up_adds_up_only_as_well_as_the_mesh_does() {
    let pair = BallAndBlock {
        side: 8,
        radius: 2,
        at: [4, 4],
    };
    let (model, keys) = pair.build();
    let (block, ball) = (keys[0], keys[1]);
    let united = plainly_fused(&model, &[block], &[ball], Rule::Union);
    let shared = plainly_fused(&model, &[block], &[ball], Rule::Intersection);

    let how_far_out = |deflection| {
        let apart = volume_at(&united.0, &united.1, deflection)
            + volume_at(&shared.0, &shared.1, deflection)
            - volume_at(&model, &[block], deflection)
            - volume_at(&model, &[ball], deflection);
        apart.abs()
    };
    let coarse = how_far_out(a_deflection());
    let fine =
        how_far_out(Deflection::of(0.001).expect("a thousandth of a millimetre is a length"));

    assert!(
        coarse > 1e-6,
        "if the four volumes already added up, this row should assert the law instead: {coarse}"
    );
    assert!(
        fine * 100.0 < coarse,
        "a hundredfold finer mesh closed the gap from {coarse} to {fine}, which is under a \
         hundredfold — so the gap is not only the mesh and something else is wrong"
    );
}

/// Absorption, over a round solid: `A ∪ (A ∩ B)` is `A` — asserted, and it
/// compares on 67 of 67.
///
/// The first fuse answers every time. The second is handed half a ball whose
/// flat face lies exactly in the block's floor and whose skin lies exactly on a
/// piece of the answer's own boundary. That second fuse refused on every draw,
/// naming `"a point in a piece that every solid could place"`, until §351 made selection ask
/// whether a **flat** piece lies on a face before firing the ray, as it already
/// did for a curved one: 67 of 67 on the shipped corpus before, 25 of 67 after.
///
/// **The 25 answer since row Q1062**, and all 67 are the block, as the law
/// says. Measured with Q1062's new step switched off by hand, the 25 came back
/// under the same name. `select.rs` wound a flat face's corners round the
/// point to ask whether the face holds it, and a point on a spoke of every fan
/// of those corners, such as the middle of a rectangle, could not be said to be
/// held; it now asks `SolidView::face_holds_the_point` there. The control is
/// the same law over the lattice family, which
/// [`a_solid_united_with_what_it_shares_with_another_is_itself`] asserts in
/// full, so the difference between the two rows is the curve and nothing else.
#[test]
fn a_block_united_with_what_it_shares_with_a_ball_is_the_block() {
    let coordinates = Coordinates::LieOnARoundWallAndMayCoincide;
    let tally = over(CASES, a_ball_and_a_block(), |pair, tally| {
        let (model, keys) = pair.build();
        let (block, ball) = (keys[0], keys[1]);
        tally.meaning_to_compare();

        let Some(shared) = fused(
            tally,
            coordinates,
            &model,
            &[block],
            &[ball],
            Rule::Intersection,
        )?
        .answer() else {
            return Ok(());
        };
        let Some(swallowed) = fused(
            tally,
            coordinates,
            &shared.0,
            &[block],
            &shared.1,
            Rule::Union,
        )?
        .answer() else {
            return Ok(());
        };

        tally.comparing();
        prop_assert_eq!(
            invariants_of(&swallowed.0, &swallowed.1),
            invariants_of(&model, &[block]),
            "A ∪ (A ∩ B) is A"
        );
        Ok(())
    });
    tally.report_what_it_compared("absorption over a ball");
    tally.refuses_no_more_than(NOTHING_REFUSES);
}

/// A ceiling on the share refused that only a share of exactly nothing is
/// under, so [`Tally::refuses_no_more_than`] asks that no fuse refused.
const NOTHING_REFUSES: f64 = f64::MIN_POSITIVE;

/// `A \ (A \ B)` is `A ∩ B`, over a round solid — asserted, and it compares on
/// 67 of 67.
///
/// Two ways to the same half ball: cut the dent and then cut the dent's
/// complement back out of the block, against sharing the block with the ball
/// once. The second cut is handed the dent, whose skin is the ball's own: a
/// dome whose only corner is on the rim where the block's floor cuts it, so no
/// corner says which side of the floor the dome keeps. Until row Q1024 that
/// cut refused on [`A_FACE_NOBODY_FOLLOWS`] on 67 of 67: a point in a piece was
/// found, and the ray that was to place it met the dome. `holds.rs` now reads
/// the dome's side off the way its wire walks the rim, and every draw gets
/// both sides and agrees.
/// [`taking_away_what_is_left_after_taking_the_other_away_leaves_what_the_two_share`]
/// is the flat control.
#[test]
fn taking_the_dent_out_of_a_block_again_leaves_what_the_block_and_the_ball_share() {
    let coordinates = Coordinates::LieOnARoundWallAndMayCoincide;
    let tally = over(CASES, a_ball_and_a_block(), |pair, tally| {
        let (model, keys) = pair.build();
        let (block, ball) = (keys[0], keys[1]);
        tally.meaning_to_compare();

        let Some(dented) = fused(
            tally,
            coordinates,
            &model,
            &[block],
            &[ball],
            Rule::Difference,
        )?
        .answer() else {
            return Ok(());
        };
        let Some(twice_bitten) = fused(
            tally,
            coordinates,
            &dented.0,
            &[block],
            &dented.1,
            Rule::Difference,
        )?
        .answer() else {
            return Ok(());
        };
        let Some(shared) = fused(
            tally,
            coordinates,
            &model,
            &[block],
            &[ball],
            Rule::Intersection,
        )?
        .answer() else {
            return Ok(());
        };

        tally.comparing();
        prop_assert_eq!(
            invariants_of(&twice_bitten.0, &twice_bitten.1),
            invariants_of(&shared.0, &shared.1),
            "A \\ (A \\ B) is A ∩ B"
        );
        Ok(())
    });
    tally.report_what_it_compared("A \\ (A \\ B) = A ∩ B over a ball");
}

/// `((A ∪ B) \ B) ∪ B` is `A ∪ B`, over a round solid — asserted since row
/// Q1080, and it compares on 67 of 67.
///
/// The deepest chain any curved family has been asked for. The middle fuse
/// takes the ball back out of the answer the first one built.
///
/// **It stopped earlier until row Q1024**, at
/// [`WHICH_SIDE_OF_ITS_TRIMMING_A_FACE_LIES_ON`]: the dome the union left is a
/// face on the ball's sphere whose only corner is on the rim where the block's
/// floor cuts it, so no corner said which side of the floor the dome keeps
/// (§30.8 item 10). `holds.rs` now reads that side off the way the dome's wire
/// walks the rim, and the fuse gets one step further. Past that, it refused
/// on 67 of 67, naming `"a point in a piece that every solid could place"` (0 compared): a
/// point in a piece was found and no ray could place it, every middle fuse
/// landing on the half of a bar's wall that Q1080 answers (Q1066).
///
/// **It answers since row Q1080**, which stopped
/// `count_where_the_ray_meets_it` (`classify.rs`) from grazing a ray on a
/// face that holds none of the point: 67 of 67 comparisons get both sides,
/// 201 fuses and 0 refused, and the law holds on each.
#[test]
fn taking_a_ball_out_of_its_union_with_a_block_and_putting_it_back_gives_the_union_again() {
    let coordinates = Coordinates::LieOnARoundWallAndMayCoincide;
    let tally = over(CASES, a_ball_and_a_block(), |pair, tally| {
        let (model, keys) = pair.build();
        let (block, ball) = (keys[0], keys[1]);
        tally.meaning_to_compare();

        let Some(united) =
            fused(tally, coordinates, &model, &[block], &[ball], Rule::Union)?.answer()
        else {
            return Ok(());
        };
        let Some(bitten) = fused(
            tally,
            coordinates,
            &united.0,
            &united.1,
            &[ball],
            Rule::Difference,
        )?
        .answer() else {
            return Ok(());
        };
        let Some(back) = fused(
            tally,
            coordinates,
            &bitten.0,
            &bitten.1,
            &[ball],
            Rule::Union,
        )?
        .answer() else {
            return Ok(());
        };

        tally.comparing();
        prop_assert_eq!(
            invariants_of(&back.0, &back.1),
            invariants_of(&united.0, &united.1),
            "((A ∪ B) \\ B) ∪ B is A ∪ B"
        );
        Ok(())
    });
    tally.report_what_it_compared("((A ∪ B) \\ B) ∪ B = A ∪ B over a ball");
    tally.refuses_no_more_than(NOTHING_REFUSES);
}

/// Associativity of union over a **curved trio**, which no law in this file had
/// ever been able to ask.
///
/// §48.2 item 1's row, at its narrowest: every property here that compares
/// invariants needs a trio or a pair, and every curved fixture was a pair, so
/// `(A ∪ B) ∪ C` against `A ∪ (B ∪ C)` had never once been asked of a round
/// face. [`a_ball_a_bar_and_a_block`] is the trio, and its own doc says why the
/// ball and the bar are kept out of each other's way.
///
/// **Compared face by face with [`the_faces_of`], and not with [`answer_of`]**,
/// because the two groupings may legitimately cut the answer into different
/// faces — roadmap §11.2's point, and the reason this file picks between the
/// comparisons per property rather than once. Not with [`invariants_of`]
/// either: the two groupings hand the same faces back in a different order,
/// and a whole solid's measure sums them in that order, so it differs in the
/// last place where nothing about the solid does. That was the first thing
/// this property found once it ran.
///
/// # It runs, and what stopped it was not the trio
///
/// Until §340.5's fix, **one of the four fuses this property runs per case did
/// not answer**, and that left it comparing nothing, so it was pinned at that.
/// The one that stopped was the **second** fuse of the grouping that sinks the
/// ball into the block first: the half ball hanging under the floor has its one
/// corner on the floor, so no corner said which half of the ball it was, the
/// arrangement built the upper half, and the fuse caught its own broken answer
/// as `Invalid(FacesDoNotAgreeWhichWayTheyFace)`. Which way the half ball's
/// wire walks its circle says which half, and `bake.rs` hands that on now.
/// `a_ball_and_a_bar_in_one_block.rs` has the measurement.
///
/// # What this pin said before that, and why it changed
///
/// Until §340 the same fuse refused [`CANNOT_TELL_THE_CELLS_OF_A_FACE_APART`]:
/// the bar's circle in a floor with a round hole agreed with the
/// inside and the outside of the floor's square, and the old glue put both in
/// one cell. Before that, as the next paragraphs say, it was the triple.
///
///
/// It used to refuse on [`WHERE_THREE_SURFACES_MEET_NEEDS_TWO_OF_THEM_FLAT`], at
/// **half** the fuses rather than a quarter, and the doc here explained at length
/// that the cause was one face crossed by two round surfaces and that *"how far
/// apart the two round solids are makes no difference"*. Both sentences were true
/// when written and are false now. `place_the_meets` no longer gives up over a
/// triple whose two partner surfaces provably share nothing — `intersect` is
/// asked, and only `SurfaceMeet::Nothing` skips — so the far-apart cases that
/// refused for a corner that could not exist now answer. **The refusal moved one
/// step later; it did not go away**, and the grouping that never puts a hole in a
/// face now answers both its fuses.
///
/// The old note said the obvious way round was itself refused: putting the ball
/// in a *side* wall would leave each plane one round crossing, but a vertical
/// plane through a ball cuts both of its poles. Since §195 that cut answers —
/// `degenerate_curved_fuses.rs`'s `a_ball_cut_by_a_plane_down_its_polar_axis_is_answered`
/// — and this pin has not been rebuilt round it.
#[test]
fn uniting_a_ball_and_a_bar_with_a_block_does_not_depend_on_which_two_go_first() {
    let coordinates = Coordinates::LieOnARoundWallAndMayCoincide;
    let tally = over(LONGER_CASES, a_ball_a_bar_and_a_block(), |trio, tally| {
        let (model, keys) = trio.build();
        let (block, ball, bar) = (keys[0], keys[1], keys[2]);
        tally.meaning_to_compare();

        let left = united_in_turn(tally, coordinates, &model, &[block], &[ball], &[bar])?;
        let right = united_in_turn(tally, coordinates, &model, &[ball], &[bar], &[block])?;
        let (Some(left), Some(right)) = (left, right) else {
            return Ok(());
        };

        tally.comparing();
        prop_assert_eq!(
            the_faces_of(&left.0, &left.1),
            the_faces_of(&right.0, &right.1),
            "uniting a block, a ball and a bar does not depend on which two go first"
        );
        Ok(())
    });
    tally.report_what_it_compared(
        "associativity of union over a ball and a bar in a block, the ball sunk in first",
    );
}

/// Every face of an answer measured on its own, its area and where it balances
/// as bits, sorted, with the solid's shell count; one entry per solid, sorted.
///
/// **What [`invariants_of`] compares, without the order the faces are added
/// in.** The two groupings of a block, a ball and a bar hand back the same
/// eleven faces on the same surfaces, the floor first in one and the ball first
/// in the other, and a sum taken in another order rounds differently: on
/// `a_ball_and_a_bar_in_one_block.rs`'s block, at its deflection of `0.02`, the
/// ball-first grouping measures `357.5937108432019` and the other
/// `357.59371084320134`. A face measured on
/// its own is the same triangles and the same arithmetic in either list, so
/// this still compares bit for bit.
fn the_faces_of(model: &Model, solids: &[SolidKey]) -> Vec<(usize, Vec<[u64; 4]>)> {
    let mut each: Vec<(usize, Vec<[u64; 4]>)> = solids
        .iter()
        .map(|&key| {
            let solid = model.solid(key).expect("this model made the key");
            let mut faces: Vec<[u64; 4]> = solid
                .faces()
                .map(|face| {
                    let measured = solid
                        .measure_one_face(
                            face.key(),
                            a_deflection(),
                            &Policy::certified(),
                            &mut ProvenanceLog::new(),
                        )
                        .expect("a face of a solid this size measures");
                    let [x, y, z] = measured.centroid().coordinates();
                    [bits(measured.area()), bits(x), bits(y), bits(z)]
                })
                .collect();
            faces.sort_unstable();
            (solid.shells().count(), faces)
        })
        .collect();
    each.sort_unstable();
    each
}

/// Every frustum the family draws really does have skin inside the block and
/// skin outside it — asserted over the family, before any law is run over it.
///
/// **§47.2 is why this exists.** `AFrustumAgainstABlock` was pinned at a count,
/// the count moved the right way, and the sentence written beside it — *"a block
/// can now take a piece out of a frustum"* — was false: all six answers were the
/// one row of the grid where the two solids miss each other altogether. An
/// assertion on a count is still a count. A fixture family nobody has shown
/// overlaps can go quietly empty and every property over it stays green.
///
/// This says what a count cannot. The block is a cube, so the box round it *is*
/// the block, and a point strictly inside that box is strictly inside the
/// material. The frustum's skin is asked for as a mesh and every point of it is
/// classified: some are strictly inside the block and some are strictly outside
/// it, which is what *"these two overlap and neither swallows the other"* means.
///
/// The two ends it finds are the two the family is built around — the narrow cap
/// sits a whole unit below the floor, and the wide cap sits inside the block a
/// whole unit clear of every side wall.
#[test]
fn every_frustum_of_the_family_has_skin_inside_the_block_and_skin_outside_it() {
    let mut runner = TestRunner::new(configured(CASES));
    let checked = Cell::new(0usize);
    let inside = Cell::new(0usize);
    let outside = Cell::new(0usize);

    let ran = runner.run(&a_frustum_and_a_block(), |pair| {
        let (model, keys) = pair.build();
        let (block, frustum) = (keys[0], keys[1]);

        let (low, high) = model
            .solid(block)
            .expect("this model made the key")
            .box_around_it()
            .expect("a cube has faces to measure");
        let (low, high) = (low.coordinates(), high.coordinates());
        let skin = model
            .solid(frustum)
            .expect("this model made the key")
            .mesh(
                a_deflection(),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a frustum this size meshes");

        let mut in_the_block = 0usize;
        let mut out_of_the_block = 0usize;
        for point in skin.points() {
            let at = point.coordinates();
            if (0..3).all(|axis| low[axis] < at[axis] && at[axis] < high[axis]) {
                in_the_block += 1;
            } else if (0..3).any(|axis| at[axis] < low[axis] || high[axis] < at[axis]) {
                out_of_the_block += 1;
            }
        }

        checked.set(checked.get() + 1);
        inside.set(inside.get() + in_the_block);
        outside.set(outside.get() + out_of_the_block);
        prop_assert!(
            in_the_block > 0,
            "not one point of this frustum's skin is inside the block, so the two \
             miss each other: {pair:?}"
        );
        prop_assert!(
            out_of_the_block > 0,
            "not one point of this frustum's skin is outside the block, so the block \
             swallows it whole: {pair:?}"
        );
        Ok(())
    });
    if let Err(why) = ran {
        panic!("{why}");
    }
    println!(
        "a frustum against a block: {} configurations, {} skin points inside the block \
         and {} outside it",
        checked.get(),
        inside.get(),
        outside.get()
    );
    // At least, and not exactly: the persistence file's saved seeds are replayed
    // before any novel case, so this family draws 61 for a corpus of 48.
    assert!(
        checked.get() >= CASES as usize,
        "only {} of the {CASES} configurations were checked, so this pin is not over \
         the family",
        checked.get()
    );
}

/// Swapping the two sides of a block and a frustum, bit for bit.
///
/// §47.7's second row closed: *"cones are absent from
/// `fuses_that_must_agree.rs`"*. This is the first law of any kind to run over
/// the third curved family, and it was written expecting to pin a refusal — the
/// brief said so, `how_often_a_fuse_with_a_cone_refuses.rs` then measured its
/// own frustum family at 6 answers out of 96, and §47.2 says all six of those
/// were the row where the block misses the frustum altogether. It answers 66
/// now, 42 of them where the block cuts the frustum, since the fuse stopped
/// naming a hyperbola arc by its chord (`docs/decisions.md` §221).
///
/// **Three fifths of this family answers, and every answer commutes.** Two
/// fifths when this was written, and the third came from `arrange` learning to
/// order points along a bend: one fresh-seed run reached 74 of its 126
/// comparisons, where eighteen of them gave 30% to 46% before. This one draws
/// its own seed every run, so read that as a band moving and not as a number.
/// The difference from the file above is the fixture rather than the kernel:
/// this family's frustum
/// really does stand up through the block's floor
/// ([`every_frustum_of_the_family_has_skin_inside_the_block_and_skin_outside_it`]
/// classifies every point of its skin and finds some inside and some outside),
/// and it is clear of every side wall by a whole unit. Which of them answer is
/// not left as a rate: `tests/ordering_points_along_an_endless_bend.rs` walks a
/// grid of 84 configuration by configuration.
///
/// **The rest no longer refuse on [`ORDERING_ALONG_AN_ENDLESS_BEND`].** They did
/// when this was written — §47.7's first row — and `arrange`'s third
/// `HowItRuns` variant took that refusal off the grid entirely. What is left
/// stops in `select.rs` and in triangulation, which
/// `how_often_a_fuse_with_a_cone_refuses.rs` names one family at a time.
///
/// **Neither side of a comparison has ever been the odd one out.** Every case
/// here either answers both ways round or refuses both ways round — 100 answered
/// fuses making 50 comparisons the first time it ran — which is worth saying
/// because whether the fuse gives up is a fact about the arithmetic and the two
/// sides genuinely take different paths through it.
#[test]
fn swapping_the_two_sides_of_a_block_and_a_frustum_changes_not_one_bit() {
    let coordinates = Coordinates::LieOnAConesWall;
    let tally = over(CASES, a_frustum_and_a_block(), |pair, tally| {
        for rule in [Rule::Union, Rule::Intersection] {
            tally.meaning_to_compare();
            let (model, keys) = pair.build();
            let (block, frustum) = (keys[0], keys[1]);

            let forwards = fused(tally, coordinates, &model, &[block], &[frustum], rule)?;
            let backwards = fused(tally, coordinates, &model, &[frustum], &[block], rule)?;
            both_agree_about_whether_there_is_an_answer(
                tally,
                coordinates,
                &forwards,
                &backwards,
                rule,
                "swapping the two sides",
            )?;
            let (Some(forwards), Some(backwards)) = (forwards.answer(), backwards.answer()) else {
                continue;
            };

            tally.comparing();
            same_answer(
                &answer_of(&forwards.0, &forwards.1),
                &answer_of(&backwards.0, &backwards.1),
                rule,
                "swapping the two sides of a block and a frustum",
            )?;
        }
        Ok(())
    });
    tally.report_what_it_compared_at_least(
        "commutativity, a frustum up through a block",
        A_THINNER_LAW_STILL_COMPARES_THIS_MUCH,
    );
}

/// The same law over the eighty-four-configuration grid, which the family
/// cannot draw.
///
/// **This grid used to pin a rule, and the rule is dead.** §49.6 published *the
/// fuse answers if and only if the cone, carried up to the block's roof, is
/// still narrower than the distance to the nearest side wall*, and this file
/// asserted it over these eighty-four. `arrange`'s third `HowItRuns` variant
/// puts points in order along a hyperbola now, so nineteen of the forty-four the
/// rule turned down answer instead.
/// `tests/ordering_points_along_an_endless_bend.rs` is where that death is
/// measured — none refused for want of an order, fifty-nine answering, and the
/// nineteen the rule was wrong about — and re-asserting any of it here would be a
/// second copy of one measurement.
///
/// What is left for *this* file is the law this file is for, asked over the grid
/// rather than over [`a_frustum_and_a_block`].
///
/// # Why the grid rather than the generator
///
/// The generator keeps the top cap at least a unit below the block's roof, and
/// it never pairs a slope of one eighth with a block under eight across. So the
/// row above cannot draw a top cap sitting exactly in the roof plane and cannot
/// draw the steepest cone in the smallest block. Both are on this grid, both are
/// among the configurations the bend newly reaches, and a commutativity law that
/// has never been asked of them has its hole in exactly the new place.
///
/// The grid is the same eighty-four: the three slopes, every block from six to
/// twelve across, and the frustum's axis at every whole place that leaves the
/// wide cap a unit clear of both walls, with the narrow cap fixed at one and the
/// wide at two.
///
/// **Written out rather than drawn**, so the count it compares is the same
/// number every run on every machine. That is why it is an equality and not a
/// floor: a drop in it is a regression and not a draw. Re-derive it when it
/// moves; do not relax it (§19.1).
#[test]
fn every_frustum_on_the_wider_grid_answers_the_same_whichever_side_comes_first() {
    let coordinates = Coordinates::LieOnAConesWall;
    let tally = Tally::default();
    for up_per_unit_wider in [2u32, 4, 8] {
        for side in 6u32..=12 {
            for at in 3u32..=(side - 3) {
                let pair = FrustumAndBlock {
                    side,
                    narrow: 1,
                    wide: 2,
                    up_per_unit_wider,
                    at: [at, at],
                };
                for rule in [Rule::Union, Rule::Intersection] {
                    tally.meaning_to_compare();
                    let (model, keys) = pair.build();
                    let (block, frustum) = (keys[0], keys[1]);

                    let forwards = fused(&tally, coordinates, &model, &[block], &[frustum], rule)
                        .unwrap_or_else(|why| panic!("{pair:?}: {why}"));
                    let backwards = fused(&tally, coordinates, &model, &[frustum], &[block], rule)
                        .unwrap_or_else(|why| panic!("{pair:?}: {why}"));
                    both_agree_about_whether_there_is_an_answer(
                        &tally,
                        coordinates,
                        &forwards,
                        &backwards,
                        rule,
                        "swapping the two sides",
                    )
                    .unwrap_or_else(|why| panic!("{pair:?}: {why}"));
                    let (Some(forwards), Some(backwards)) = (forwards.answer(), backwards.answer())
                    else {
                        continue;
                    };

                    tally.comparing();
                    same_answer(
                        &answer_of(&forwards.0, &forwards.1),
                        &answer_of(&backwards.0, &backwards.1),
                        rule,
                        "swapping the two sides of a block and a frustum on the wider grid",
                    )
                    .unwrap_or_else(|why| panic!("{pair:?}: {why}"));
                }
            }
        }
    }
    tally.report("commutativity, the eighty-four-configuration frustum grid");
    assert_eq!(
        tally.compared.get(),
        THE_WIDER_GRID_COMPARES_THIS_MANY,
        "the grid set out to make {} comparisons and made {} of them",
        tally.meant_to_compare.get(),
        tally.compared.get()
    );
}

/// The flush pair of the wider grid, whose top cap lies in the block's roof,
/// answers the same whichever side comes first, with the volume worked out by
/// hand.
///
/// The frustum's cap circle starts at its seam vertex, and the block roof has
/// the circle the cone cuts it in, started where the two surfaces alone put it.
/// Selection keeps one of the two coincident disks, whichever came first, so a
/// union kept the roof's disk one way round and the cap the other, and an
/// intersection the same the other way. The side whose disk and ring disagreed
/// about the circle's corner named two edges with one face each and was refused
/// `ResultIsNotManifold`. A closed curve's corner is now chosen once, before
/// any is named, by what it is.
///
/// The frustum runs from radius one at a unit below the floor to radius two at
/// the roof, eight up; so the part below the floor is `π/3 · (1 + 9/8 + 81/64)`
/// and the rest is `56π/3` less that.
#[test]
fn a_frustum_whose_cap_lies_in_the_blocks_roof_answers_the_same_either_way_round() {
    let below_the_floor = std::f64::consts::PI / 3.0 * (1.0 + 1.125 + 1.125 * 1.125);
    let the_frustum = 56.0 * std::f64::consts::PI / 3.0;
    for at in [3, 4] {
        let pair = FrustumAndBlock {
            side: 7,
            narrow: 1,
            wide: 2,
            up_per_unit_wider: 8,
            at: [at, at],
        };
        for (rule, closed_form) in [
            (Rule::Union, 343.0 + below_the_floor),
            (Rule::Intersection, the_frustum - below_the_floor),
        ] {
            let (model, keys) = pair.build();
            let forwards = plainly_fused(&model, &keys[..1], &keys[1..], rule);
            let backwards = plainly_fused(&model, &keys[1..], &keys[..1], rule);
            same_answer(
                &answer_of(&forwards.0, &forwards.1),
                &answer_of(&backwards.0, &backwards.1),
                rule,
                "swapping a block and a frustum flush in its roof",
            )
            .unwrap_or_else(|why| panic!("{pair:?}: {why}"));
            assert_eq!(forwards.1.len(), 1, "{rule:?} of {pair:?} is one solid");
            let measured = forwards
                .0
                .solid(forwards.1[0])
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.01).expect("a positive length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a solid this size measures");
            assert!(
                (measured.volume() - closed_form).abs() <= measured.volume_error(),
                "{rule:?} of {pair:?}: {} ± {} against {closed_form}",
                measured.volume(),
                measured.volume_error()
            );
        }
    }
}

/// Roadmap §11.2's first instruction, over a round solid, four milestones late.
///
/// *"Start with transform invariance. Kernels overfit badly to axis-aligned and
/// origin-centred configurations."* Until §48 that had never been asked of a
/// curved face at all: [`Answer::moved_by`] panicked on one, so a pure
/// translation of a ball would have brought the suite down.
///
/// **Bit for bit, with no tolerance**, like every other row here. The pair's
/// numbers are all small whole ones and the shift is a whole number, so a
/// translation is one addition per coordinate and rounds nowhere
/// (`Isometry::apply_to_point`'s own doc, and `rcad-geom`'s
/// `a_movement_with_no_turn_moves_a_point_by_one_addition_and_nothing_else`).
/// The corners the fuse invents are whole numbers too: a bar up `z` meets the
/// block's floor and roof in circles of the bar's own radius, and the only
/// corners on those circles are where the bar's seam crosses them.
///
/// **A shift and no quarter turn**, which is the one thing weaker here than the
/// rectilinear rows next door, and the reason is
/// [`a_quarter_turn_of_a_round_solid_cannot_be_asked_for_exactly`] rather than
/// an afternoon.
#[test]
fn shifting_a_block_and_a_bar_a_whole_number_changes_not_one_bit() {
    let tally = over(
        CASES,
        (a_curved_pair(), a_whole_number_shift()),
        |(pair, shift), tally| {
            moving_a_built_pair_leaves_the_answer_alone(
                tally,
                Coordinates::LieOnARoundWall,
                &|| pair.build(),
                &|| shifted(pair.build(), &shift),
                &shift,
            )
        },
    );
    tally.report_what_it_compared(
        "transform invariance, a bar through a block under whole-number shifts",
    );
}

/// The same over the second curved family, whose wall curves both ways.
///
/// The ball's centre is a whole-number point on the block's floor and its radius
/// is a whole number, so the same argument holds: where its skin meets the floor
/// is a circle of its own radius, and the corners on that circle are where its
/// seam crosses it.
#[test]
fn shifting_a_block_and_a_ball_a_whole_number_changes_not_one_bit() {
    let tally = over(
        CASES,
        (a_ball_and_a_block(), a_whole_number_shift()),
        |(pair, shift), tally| {
            moving_a_built_pair_leaves_the_answer_alone(
                tally,
                Coordinates::LieOnARoundWall,
                &|| pair.build(),
                &|| shifted(pair.build(), &shift),
                &shift,
            )
        },
    );
    tally.report_what_it_compared(
        "transform invariance, a ball in a block under whole-number shifts",
    );
}

/// Why the two rows above shift and never turn.
///
/// A rectilinear family turns exactly because [`Configuration::built_after`]
/// applies the quarter turn to the outline's *whole numbers* before anything is
/// built, so no `f64` rounds on the way in. A round solid has no whole numbers
/// to turn: `add_round_bar` and `add_ball` take a placement, and the only turn
/// `rcad-geom` will build is [`Isometry::rotation`], which takes an angle. This
/// is what that costs at the one angle the suite would want.
///
/// So an exactly turned round solid is **not reachable through the public API**,
/// and that is a limit and not a gap (`docs/decisions.md` §16.10): the family of
/// fixes is *"name the turn some other way"*, and every member of it — a
/// quaternion, a matrix, four composed eighth-turns — has to arrive through
/// [`Isometry::rotation`], because that is the only constructor with a turn in
/// it. The other family, *"rebuild the round solid turned"*, does not exist
/// either: a bar built at a turned placement has its seam where the builder puts
/// it, on `+x`, and not where the turn would have carried it, so the two answers
/// differ over a seam and not over any arithmetic.
#[test]
fn a_quarter_turn_of_a_round_solid_cannot_be_asked_for_exactly() {
    let z = Axis::new(Point::origin(), Vector::new(0.0, 0.0, 1.0)).expect("an axis along z");
    let quarter =
        Isometry::rotation(&z, core::f64::consts::FRAC_PI_2).expect("a quarter turn about z");

    let turned = quarter
        .apply_to_point(Point::new(1.0, 0.0, 0.0))
        .coordinates();
    assert_ne!(
        turned,
        [0.0, 1.0, 0.0],
        "if this ever holds, the curved rows above can turn as well as shift"
    );
    // `y` lands on one exactly; it is `x` that misses, by one step of the grid
    // at one, which is `2⁻⁵²`. A corner of the answer at a hundred would miss by
    // a hundred times that, and the comparison above has no room for any of it.
    assert_eq!(turned[1], 1.0);
    assert_eq!(turned[0], f64::EPSILON);
}

/// A whole-number shift and no turn, which is all of a [`Movement`] a round
/// solid can be moved by today.
///
/// Four either way along each axis. Small on purpose: the block is four to eight
/// across, so the pair stays well inside the working size and every coordinate
/// stays a small whole number.
fn a_whole_number_shift() -> impl Strategy<Value = Movement> {
    [-4i32..=4, -4i32..=4, -4i32..=4].prop_map(|shift| Movement {
        quarters: 0,
        shift: shift.map(f64::from),
    })
}

/// The same pair with every solid of it moved by `shift`.
///
/// `place_solid` composes the shift onto the placement each solid already
/// carries, and composing two movements with no turn adds their translations and
/// does nothing else, so a whole-number shift of a whole-number placement is
/// exact.
fn shifted((mut model, keys): (Model, Vec<SolidKey>), shift: &Movement) -> (Model, Vec<SolidKey>) {
    let away = Isometry::translation(Vector::new(shift.shift[0], shift.shift[1], shift.shift[2]))
        .expect("a whole-number shift");
    for &key in &keys {
        model
            .place_solid(key, &away)
            .expect("this model made the key");
    }
    (model, keys)
}

/// The curved family's measured row, and the thing that makes every property
/// above worth its pass.
///
/// The same discipline as §17.7's control rows: a fixed seed, every rule, and a
/// refusal is a failure. §36.27 measures this arrangement at nothing refused and
/// nothing recorded, and this is the same claim from a different generator with
/// a different corpus — which is worth having, because §36.27's four families
/// draw from one hand-written sequence and this one draws from proptest.
///
/// **There is no band on answers that are not solids, because there are none.**
/// A bar a whole unit clear of every wall of the block it goes through never
/// leaves four faces along one edge, and asserting zero is a sharper statement
/// than any band would be.
#[test]
fn every_certified_fuse_of_a_curved_pair_is_decided_and_records_nothing() {
    let tally = measured(a_curved_pair(), |pair, tally| {
        let (model, keys) = pair.build();
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            fused(
                tally,
                Coordinates::LieOnARoundWall,
                &model,
                &[keys[0]],
                &[keys[1]],
                rule,
            )?;
        }
        Ok(())
    });
    tally.report("§37 control: a bar through a block, every rule");
    assert_eq!(
        tally.refused.get(),
        0,
        "a bar through a block, clear of every wall, is decided by proof alone"
    );
    assert_eq!(
        tally.not_a_solid.get(),
        0,
        "no arrangement of this family puts four faces along one edge"
    );
}

/// §191: how often a certified fuse gives up once a bar and its block are turned
/// together, beside [`every_certified_fuse_of_a_curved_pair_is_decided_and_records_nothing`]
/// as the unturned control.
#[test]
fn how_often_a_certified_fuse_of_a_turned_bar_and_block_refuses() {
    let tally = measured(turned(a_curved_pair()), |fixture, tally| {
        every_rule_of(tally, Coordinates::LieOnATurnedRoundWall, &|| {
            fixture.build()
        })
    });
    tally.report("§191: a bar through a block, turned together, every rule");
    tally.refuses_no_more_than(TURNED_ROUND_REFUSALS_MUST_STAY_BELOW);
    if tally.refused.get() > 0 {
        tally.the_cause_named_most_often_is(CANNOT_TELL_THE_CELLS_OF_A_FACE_APART);
    }
}

/// The same for a ball half sunk into a block, with
/// [`every_certified_fuse_of_a_ball_and_a_block_is_decided_and_records_nothing`]
/// as the control.
#[test]
fn how_often_a_certified_fuse_of_a_turned_ball_and_block_refuses() {
    let tally = measured(turned(a_ball_and_a_block()), |fixture, tally| {
        every_rule_of(tally, Coordinates::LieOnATurnedRoundWall, &|| {
            fixture.build()
        })
    });
    tally.report("§191: a ball in a block, turned together, every rule");
    tally.refuses_no_more_than(TURNED_ROUND_REFUSALS_MUST_STAY_BELOW);
    if tally.refused.get() > 0 {
        tally.the_cause_named_most_often_is(CANNOT_TELL_THE_CELLS_OF_A_FACE_APART);
    }
}

/// The same for a frustum up through a block. Its control is
/// [`swapping_the_two_sides_of_a_block_and_a_frustum_changes_not_one_bit`],
/// the only row that fuses the unturned family, and it refuses on its own.
#[test]
fn how_often_a_certified_fuse_of_a_turned_frustum_and_block_refuses() {
    let control = measured(a_frustum_and_a_block(), |fixture, tally| {
        every_rule_of(tally, Coordinates::LieOnAConesWall, &|| fixture.build())
    });
    control.report("§191 control: a frustum up through a block, every rule");
    let tally = measured(turned(a_frustum_and_a_block()), |fixture, tally| {
        every_rule_of(tally, Coordinates::LieOnATurnedRoundWall, &|| {
            fixture.build()
        })
    });
    tally.report("§191: a frustum up through a block, turned together, every rule");
}

/// §191's second row: a bar leaning through a block, which no other family
/// draws, beside [`every_certified_fuse_of_a_curved_pair_is_decided_and_records_nothing`]
/// as the control.
#[test]
fn how_often_a_certified_fuse_of_a_bar_leaning_through_a_block_refuses() {
    let tally = measured(a_leaning_bar(), |fixture, tally| {
        every_rule_of(tally, Coordinates::LieOnATurnedRoundWall, &|| {
            fixture.build()
        })
    });
    tally.report("§191: a bar leaning through a block, every rule");
    tally.refuses_no_more_than(LEANING_BAR_REFUSALS_MUST_STAY_BELOW);
}

/// Swapping the two sides of a turned bar and block, bit for bit.
#[test]
fn swapping_the_two_sides_of_a_turned_bar_and_block_changes_not_one_bit() {
    let tally = over(CASES, turned(a_curved_pair()), |fixture, tally| {
        both_ways_round_of(tally, Coordinates::LieOnATurnedRoundWall, &|| {
            fixture.build()
        })
    });
    tally.report_what_it_compared("commutativity, a bar through a block, turned together");
}

/// Swapping the two sides of a turned ball and block, bit for bit.
#[test]
fn swapping_the_two_sides_of_a_turned_ball_and_block_changes_not_one_bit() {
    let tally = over(CASES, turned(a_ball_and_a_block()), |fixture, tally| {
        both_ways_round_of(tally, Coordinates::LieOnATurnedRoundWall, &|| {
            fixture.build()
        })
    });
    tally.report_what_it_compared("commutativity, a ball in a block, turned together");
}

/// Swapping the two sides of a turned frustum and block, bit for bit.
#[test]
fn swapping_the_two_sides_of_a_turned_frustum_and_block_changes_not_one_bit() {
    let tally = over(CASES, turned(a_frustum_and_a_block()), |fixture, tally| {
        both_ways_round_of(tally, Coordinates::LieOnATurnedRoundWall, &|| {
            fixture.build()
        })
    });
    tally.report_what_it_compared_at_least(
        "commutativity, a frustum up through a block, turned together",
        A_THINNER_LAW_STILL_COMPARES_THIS_MUCH,
    );
}

/// Swapping the two sides of a bar leaning through a block, bit for bit.
#[test]
fn swapping_the_two_sides_of_a_bar_leaning_through_a_block_changes_not_one_bit() {
    let tally = over_a_fixed_corpus(CASES, a_leaning_bar(), |fixture, tally| {
        both_ways_round_of(tally, Coordinates::LieOnATurnedRoundWall, &|| {
            fixture.build()
        })
    });
    tally.report_what_it_compared_at_least(
        "commutativity, a bar leaning through a block",
        A_LEANING_BAR_STILL_COMPARES_THIS_MUCH,
    );
}

/// A turned bar and block take up what the unturned ones do, rule by rule,
/// inside the share of the declared mesh error the curved law already uses.
///
/// §191. The tolerance is [`A_CURVED_LAW_MAY_USE_THIS_MUCH_OF_WHAT_THE_FOUR_DECLARE`]
/// and not a new one (§14.1 question 1): the two answers are one solid put in
/// two places, so how far apart their measured volumes are is bounded by what
/// the two measurements declare, exactly as the four sides of inclusion and
/// exclusion are.
#[test]
fn turning_a_bar_and_block_together_changes_no_answer_s_volume() {
    let tally = over(CASES, turned(a_curved_pair()), |fixture, tally| {
        turning_it_moves_no_volume(tally, &fixture)
    });
    tally.report_what_it_compared("volume under a turn, a bar through a block");
}

/// The same for a ball half sunk into a block.
#[test]
fn turning_a_ball_and_block_together_changes_no_answer_s_volume() {
    let tally = over(CASES, turned(a_ball_and_a_block()), |fixture, tally| {
        turning_it_moves_no_volume(tally, &fixture)
    });
    tally.report_what_it_compared("volume under a turn, a ball in a block");
}

/// The same for a frustum up through a block.
#[test]
fn turning_a_frustum_and_block_together_changes_no_answer_s_volume() {
    let tally = over(CASES, turned(a_frustum_and_a_block()), |fixture, tally| {
        turning_it_moves_no_volume(tally, &fixture)
    });
    tally.report_what_it_compared_at_least(
        "volume under a turn, a frustum up through a block",
        A_THINNER_LAW_STILL_COMPARES_THIS_MUCH,
    );
}

/// Inclusion and exclusion over a bar leaning through a block, to the same
/// share of the declared mesh error as
/// [`what_a_block_and_a_ball_take_up_adds_up_inside_a_stated_share_of_what_they_declare`].
#[test]
fn what_a_block_and_a_leaning_bar_take_up_adds_up_inside_a_stated_share_of_what_they_declare() {
    let tally = over_a_fixed_corpus(CASES, a_leaning_bar(), |fixture, tally| {
        let (model, keys) = fixture.build();
        let (block, bar) = (keys[0], keys[1]);
        tally.meaning_to_compare();
        let coordinates = Coordinates::LieOnATurnedRoundWall;
        let Some(united) =
            fused(tally, coordinates, &model, &[block], &[bar], Rule::Union)?.answer()
        else {
            return Ok(());
        };
        let Some(shared) = fused(
            tally,
            coordinates,
            &model,
            &[block],
            &[bar],
            Rule::Intersection,
        )?
        .answer() else {
            return Ok(());
        };
        tally.comparing();
        for how_fine in HOW_FINE_THE_LAW_IS_ASKED_AT {
            let deflection = Deflection::of(how_fine).expect("a length");
            within_the_share_they_declare(
                &[
                    (how_much_it_holds(&united.0, &united.1, deflection), 1.0),
                    (how_much_it_holds(&shared.0, &shared.1, deflection), 1.0),
                    (how_much_it_holds(&model, &[block], deflection), -1.0),
                    (how_much_it_holds(&model, &[bar], deflection), -1.0),
                ],
                &format!("inclusion and exclusion over {fixture:?} at a deflection of {how_fine}"),
            )?;
        }
        Ok(())
    });
    tally.report_what_it_compared_at_least(
        "inclusion and exclusion, a bar leaning through a block",
        A_LEANING_BAR_STILL_COMPARES_THIS_MUCH,
    );
}

/// A ball half sunk into a block and united with it measures the same at a
/// deflection of `0.3` turned as upright, inside what the two measurements
/// declare between them.
///
/// §191's volume row found the turned one refused, with
/// `MeshError::BoundaryCrossesItself` on the dome, and pinned it here as a gap:
/// a rigid movement does not change whether a mesh within a deflection exists.
/// The dome is a cap, cut into four patches by meridians up to its pole from
/// the rim corners nearest the four quarter turns. On a rim this coarse those
/// corners are not a quarter apart, and each patch's pole side ran a quarter
/// turn back from the meridian it arrived up, past the one it had to leave
/// down, and folded back on itself. The side now ends at the meridian the
/// patch leaves by (`mesh.rs`, `cut_a_cap_up_to_its_pole`).
///
/// **The tolerance is the two measurements' own bounds added.** Each
/// `volume_error` bounds `|true − measured|` for its own mesh at this
/// deflection, and the true volume is the same both ways, so the two measured
/// volumes can be apart by at most the sum.
#[test]
fn a_turned_ball_united_with_a_block_holds_what_the_upright_one_does() {
    let fixture = Turned {
        fixture: BallAndBlock {
            side: 5,
            radius: 1,
            at: [3, 2],
        },
        turn: Turn {
            direction: [2, -2, 3],
            steps: 38,
        },
    };
    let coarse = Deflection::of(0.3).expect("a length");
    let meshed = |(model, keys): (Model, Vec<SolidKey>)| {
        let (model, solids) = plainly_fused(&model, &keys[..1], &keys[1..], Rule::Union);
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("the union is a sound model");
        let [solid] = solids[..] else {
            panic!(
                "a ball and a block unite into one solid, not {}",
                solids.len()
            )
        };
        model
            .solid(solid)
            .expect("this model made the key")
            .measure(coarse, &Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("the union meshes at 0.3: {why}"))
    };

    let upright = meshed(fixture.fixture.built());
    let turned = meshed(fixture.build());
    let declared = upright.volume_error() + turned.volume_error();
    // §47.4 item 1: a tolerance nothing can exceed has measured nothing.
    assert!(
        declared.is_finite() && declared > 0.0 && declared < upright.volume(),
        "the two measurements declared {declared}, and nothing is worth asserting against that"
    );
    let apart = (turned.volume() - upright.volume()).abs();
    assert!(
        apart <= declared,
        "turned the union holds {}, upright {}: {apart} apart, past the {declared} the two declare",
        turned.volume(),
        upright.volume()
    );
}

/// Every fuse a turned curved fixture refuses has an answer the kernel can
/// hold — so every such refusal is a **gap and not a limit** (§16.10), and this
/// is the demonstration rather than a paragraph.
///
/// A turned fixture's solids are the upright fixture's solids with one more
/// placement composed on, the same `Isometry` for each. So the upright answer,
/// placed by that same `Isometry`, *is* the turned answer, written in the
/// numbers the kernel itself would have to write it in: nothing is rounded
/// that the turned inputs did not already round. This takes every turned fuse
/// that did not answer, fuses the upright fixture instead, turns what comes
/// back and asks `Model::validate` of it. Where the upright fixture refuses as
/// well — the frustum does, on its own — there is nothing to demonstrate and
/// the case is counted apart.
#[test]
fn every_fuse_a_turned_curved_fixture_refuses_has_an_answer_turned_from_the_upright_one() {
    let bar = the_refusals_the_upright_answer_turned_would_have_answered(turned(a_curved_pair()));
    let ball =
        the_refusals_the_upright_answer_turned_would_have_answered(turned(a_ball_and_a_block()));
    let frustum =
        the_refusals_the_upright_answer_turned_would_have_answered(turned(a_frustum_and_a_block()));
    for (what, (demonstrated, both_refused)) in [
        ("a bar through a block", bar),
        ("a ball in a block", ball),
        ("a frustum up through a block", frustum),
    ] {
        println!(
            "{what}, turned: {demonstrated} refusals shown to have an answer, {both_refused} \
             refused upright as well"
        );
    }
    // Since §202.2 the turned bar and ball refuse nothing on the shipped corpus,
    // so only the frustum still has refusals to demonstrate. Theirs are pinned at
    // zero rather than dropped, because a refusal coming back is what this
    // counts.
    assert_eq!(
        (bar.0, ball.0),
        (0, 0),
        "a turned bar or ball refused a fuse its upright control answers"
    );
    assert!(
        frustum.0 > 0,
        "the turned frustum refuses something its upright control answers, which is what \
         makes this a demonstration rather than an empty loop"
    );
}

/// Over the shipped corpus: how many turned refusals the upright answer, turned,
/// showed to have an answer, and how many refused upright too.
///
/// # Panics
///
/// If the upright answer, turned, is not a sound model — which would make the
/// refusal a limit after all, or the fixture wrong.
fn the_refusals_the_upright_answer_turned_would_have_answered<P: Fixture + core::fmt::Debug>(
    fixtures: impl Strategy<Value = Turned<P>>,
) -> (usize, usize) {
    let demonstrated = Cell::new(0usize);
    let both_refused = Cell::new(0usize);
    // The shipped corpus and not [`CASES`]: at forty-eight the bar and the ball
    // draw no refusal at all, and there is nothing to demonstrate.
    measured(fixtures, |fixture, tally| {
        let (turned, turned_keys) = fixture.build();
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            let coordinates = Coordinates::LieOnATurnedRoundWall;
            let answered = fused(
                tally,
                coordinates,
                &turned,
                &[turned_keys[0]],
                &[turned_keys[1]],
                rule,
            )?
            .answer();
            if answered.is_some() {
                continue;
            }
            let (upright, upright_keys) = fixture.fixture.built();
            let Some((mut model, solids)) = fused(
                tally,
                coordinates,
                &upright,
                &[upright_keys[0]],
                &[upright_keys[1]],
                rule,
            )?
            .answer() else {
                both_refused.set(both_refused.get() + 1);
                continue;
            };
            let turn = fixture.turn.isometry();
            for &solid in &solids {
                model
                    .place_solid(solid, &turn)
                    .expect("this model made the key");
            }
            if let Err(why) = model.validate(&Policy::certified(), &mut ProvenanceLog::new()) {
                return Err(TestCaseError::fail(format!(
                    "the upright {rule:?} of {fixture:?}, turned, is not a sound model: {why}"
                )));
            }
            demonstrated.set(demonstrated.get() + 1);
        }
        Ok(())
    });
    (demonstrated.get(), both_refused.get())
}

/// A sound union of a ball and a block, moved by a half turn, is still sound.
///
/// §191's volume row found the case on a fresh seed and shrank it to this: a
/// half turn about `(1, 0, 1)` carries the ball's own axis from `z` to `x`,
/// give or take the rounding of `cos π`. The turned fixture's union was refused
/// then, and the way to classify that refusal (§16.10) is to turn the upright
/// union by the same `Isometry` and ask whether the kernel holds it. It did
/// not: the upright union validated, `place_solid` accepted the turn, and the
/// moved model failed with `AClosedEdgeIsWalkedAgainstItsSurface`. A rigid
/// movement cannot turn a sound solid into a broken one, and it was
/// `validate`'s winding check that was wrong: it read the direction rule where
/// the solid sits, where a ball is wound about the world's `z`, and not where
/// the faces are stored, where its edges were walked. The two inputs turned the
/// same way validated, and so did a lone unit ball turned the same way — it is
/// the dome the union leaves, not the ball.
///
/// **A second turn did the same**: three quarters of a turn about `-y`, which
/// the volume row shrank to on another fresh seed. Both turns carry the ball's
/// axis onto the world's `x`, give or take the rounding of the angle — the
/// §124 shape exactly. **The third** is the plainest turn that does it, half
/// over about `x`, which turns the ball's own `z` to point down.
///
/// The two seeds are not kept in `fuses_that_must_agree.proptest-regressions`:
/// every property here replays that file first, so a seed added there reshuffles
/// the fixed-seed rows of every other family — it moved the skew row out of its
/// band on the first try (§136.5's hazard). They are these fixtures instead.
#[test]
fn a_ball_and_block_united_and_then_turned_half_over_is_still_a_sound_model() {
    for turn in [
        Turn {
            direction: [-2, 0, -2],
            steps: 32,
        },
        Turn {
            direction: [0, -1, 0],
            steps: 48,
        },
        Turn {
            direction: [1, 0, 0],
            steps: 32,
        },
    ] {
        let fixture = Turned {
            fixture: BallAndBlock {
                side: 4,
                radius: 1,
                at: [2, 2],
            },
            turn,
        };
        let (inputs, _) = fixture.build();
        inputs
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("the two inputs turned are sound");

        let (upright, keys) = fixture.fixture.built();
        let (mut model, solids) = plainly_fused(&upright, &keys[..1], &keys[1..], Rule::Union);
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("the upright union is sound");
        let turn = fixture.turn.isometry();
        for &solid in &solids {
            model
                .place_solid(solid, &turn)
                .expect("this model made the key");
        }
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| {
                panic!(
                    "a sound union moved by {:?} is still sound: {why}",
                    fixture.turn
                )
            });
    }
}

/// The same union turned about every axis `{-1, 0, 1}³` names, by every
/// sixty-fourth of a turn short of a whole one, is still sound: 26 axes and 63
/// steps, 1638 turns.
///
/// The three turns above are where §191 happened on the defect; this is how
/// wide it was. While `validate` read a ball's winding where the solid sits,
/// 430 of these 1638 were refused `AClosedEdgeIsWalkedAgainstItsSurface`, and
/// every one of the 430 measured the upright union's volume.
#[test]
fn a_ball_and_block_union_turned_about_any_of_26_axes_is_still_a_sound_model() {
    let fixture = BallAndBlock {
        side: 4,
        radius: 1,
        at: [2, 2],
    };
    let (upright, keys) = fixture.built();
    let (model, solids) = plainly_fused(&upright, &keys[..1], &keys[1..], Rule::Union);
    let mut refused = Vec::new();
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                if [x, y, z] == [0, 0, 0] {
                    continue;
                }
                for steps in 1..64 {
                    let turn = Turn {
                        direction: [x, y, z],
                        steps,
                    };
                    let mut moved = model.clone();
                    for &solid in &solids {
                        moved
                            .place_solid(solid, &turn.isometry())
                            .expect("this model made the key");
                    }
                    if let Err(why) =
                        moved.validate(&Policy::certified(), &mut ProvenanceLog::new())
                    {
                        refused.push((turn, why));
                    }
                }
            }
        }
    }
    assert!(
        refused.is_empty(),
        "{} of 1638 turns of a sound union were refused, the first {:?}",
        refused.len(),
        refused.first()
    );
}

/// A bar leaning through a block, clear of every side wall by more than a unit,
/// united with the block is one solid, of the volume Cavalieri's principle
/// gives: the cube's own volume, plus however much of the bar's cylinder —
/// cross-section `π·radius²` all along its own length — sits outside the
/// slab the floor and the roof cut from it. That slab's axial span is
/// `side · secθ`, `θ` the lean off upright, because the bar travels `secθ`
/// along its own axis for every unit it climbs.
///
/// §191 found this fuse wrong: it answered `ResultIsNotManifold`, *"an edge
/// would have 8 faces along it"*, for a union whose every crossing is
/// transversal — the floor and the roof cut the bar in ellipses whose nearest
/// point to a side wall is more than a unit off it, and the caps stand more
/// than a unit clear of the block — so no edge of the true union can have more
/// than two faces along it. **Fixed by Q114 (d6c64ba4)**: the wall below the
/// block crosses the planes `x = 0` and `y = 0` in two ellipses meeting at
/// `(0, 0, -6.23)` and `(0, 0, -2.77)`, and an arc named without its plane put
/// arcs of both ellipses onto one edge.
///
/// Found by walking every whole-number place the [`a_leaning_bar`] family's
/// room allows for radius one and two, leans `(1, 0)` and `(1, 1)`, one to
/// four up and blocks six to sixteen across: two of those fuses came back
/// wrong this way, and this is the first of them.
#[test]
fn a_bar_leaning_through_a_block_unites_with_it_into_a_solid() {
    let fixture = LeaningBar {
        side: 15,
        radius: 1,
        at: [6, 6],
        lean: [1, 1],
        up: 2,
    };
    let room = room_for_a_leaning_bar(fixture.side, fixture.radius, fixture.lean, fixture.up);
    assert!(
        room.iter().all(|room| room.contains(&6)),
        "the fixture is one the family draws, a unit clear of every side wall"
    );
    let (mut model, keys) = fixture.build();
    let fused = model
        .fuse(
            &keys[..1],
            &keys[1..],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("every crossing here is transversal, so the union is a solid");
    assert_eq!(
        fused.solids().len(),
        1,
        "a bar through a block is one solid"
    );
    let solid = fused.solids()[0];
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the union is a sound model");

    let side = f64::from(fixture.side);
    let [across, along] = fixture.lean.map(f64::from);
    let secant_of_the_lean =
        across.hypot(along).hypot(f64::from(fixture.up)) / f64::from(fixture.up);
    let closed_form = side.powi(3)
        + std::f64::consts::PI
            * f64::from(fixture.radius).powi(2)
            * (fixture.length() - side * secant_of_the_lean);
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid this size measures");
    assert!(
        (measured.volume() - closed_form).abs() <= measured.volume_error(),
        "a bar leaning through a block: {} ± {} against {closed_form}",
        measured.volume(),
        measured.volume_error()
    );
}

/// Ask all three rules of whatever `build` builds, the first solid against the
/// second, and keep nothing but the tally.
fn every_rule_of(
    tally: &Tally,
    coordinates: Coordinates,
    build: &dyn Fn() -> (Model, Vec<SolidKey>),
) -> Result<(), TestCaseError> {
    let (model, keys) = build();
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        fused(tally, coordinates, &model, &[keys[0]], &[keys[1]], rule)?;
    }
    Ok(())
}

/// Every rule over a fixture and over the same fixture turned: the same number
/// of solids, and volumes apart by no more than the share of what the two
/// measurements declare.
fn turning_it_moves_no_volume<P: Fixture + core::fmt::Debug>(
    tally: &Tally,
    fixture: &Turned<P>,
) -> Result<(), TestCaseError> {
    let coordinates = Coordinates::LieOnATurnedRoundWall;
    let (still, still_keys) = fixture.fixture.built();
    let (turned, turned_keys) = fixture.build();
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        tally.meaning_to_compare();
        let Some(still) = fused(
            tally,
            coordinates,
            &still,
            &[still_keys[0]],
            &[still_keys[1]],
            rule,
        )?
        .answer() else {
            continue;
        };
        let Some(turned) = fused(
            tally,
            coordinates,
            &turned,
            &[turned_keys[0]],
            &[turned_keys[1]],
            rule,
        )?
        .answer() else {
            continue;
        };
        tally.comparing();
        prop_assert_eq!(
            still.1.len(),
            turned.1.len(),
            "a {:?} of {:?} came back in a different number of solids turned",
            rule,
            fixture
        );
        for how_fine in HOW_FINE_THE_LAW_IS_ASKED_AT {
            let deflection = Deflection::of(how_fine).expect("a length");
            let still = how_much_it_holds(&still.0, &still.1, deflection);
            let Some(turned) = how_much_it_holds_if_it_meshes(&turned.0, &turned.1, deflection)?
            else {
                *tally
                    .gave_up
                    .borrow_mut()
                    .entry(A_MESH_OF_THE_TURNED_ANSWER_REFUSED)
                    .or_default() += 1;
                continue;
            };
            within_the_share_they_declare(
                &[(still, 1.0), (turned, -1.0)],
                &format!("a {rule:?} of {fixture:?} turned, at a deflection of {how_fine}"),
            )?;
        }
    }
    Ok(())
}

/// [`how_much_it_holds`], or nothing when the mesher refuses a face of it.
///
/// Only ever asked of a **turned** answer whose upright twin has just measured
/// at the same deflection — [`how_much_it_holds`] panics otherwise — so every
/// refusal let past here is a gap by
/// [`a_turned_ball_united_with_a_block_holds_what_the_upright_one_does`]'s
/// argument: a rigid movement does not change whether a mesh exists. Two kinds
/// have turned up, `BoundaryCrossesItself` and a loop with a corner twice.
fn how_much_it_holds_if_it_meshes(
    model: &Model,
    solids: &[SolidKey],
    how_fine: Deflection,
) -> Result<Option<(f64, f64)>, TestCaseError> {
    let mut held = (0.0, 0.0);
    for &solid in solids {
        match model
            .solid(solid)
            .expect("this model made the key")
            .measure(how_fine, &Policy::certified(), &mut ProvenanceLog::new())
        {
            Ok(measured) => {
                held.0 += measured.volume();
                held.1 += measured.volume_error();
            }
            Err(
                Error::CannotMeshAFace { .. } | Error::TwoPatchesOfTheMeshDrewTheSameSide { .. },
            ) => return Ok(None),
            Err(why) => {
                return Err(TestCaseError::fail(format!(
                    "an answer would not measure: {why}"
                )));
            }
        }
    }
    Ok(Some(held))
}

/// Volumes added up with their signs miss zero by no more than
/// [`A_CURVED_LAW_MAY_USE_THIS_MUCH_OF_WHAT_THE_FOUR_DECLARE`] of the mesh
/// error they declare between them.
fn within_the_share_they_declare(
    sides: &[((f64, f64), f64)],
    what: &str,
) -> Result<(), TestCaseError> {
    let out_by: f64 = sides.iter().map(|((volume, _), sign)| sign * volume).sum();
    let declared: f64 = sides.iter().map(|((_, error), _)| error).sum();
    // §47.4 item 1: a tolerance nothing can exceed has measured nothing.
    prop_assert!(
        declared.is_finite() && declared > 0.0,
        "{what}: the measurements declared {declared}, and nothing is worth asserting against that"
    );
    let share = out_by.abs() / declared;
    prop_assert!(
        share < A_CURVED_LAW_MAY_USE_THIS_MUCH_OF_WHAT_THE_FOUR_DECLARE,
        "{what}: {out_by} out, which is {share} of the {declared} the measurements declared"
    );
    Ok(())
}

/// Fuse without a tally, for the tests that name one pair rather than a family.
///
/// # Panics
///
/// If the fuse refuses, or records a decision it could not prove.
fn plainly_fused(
    model: &Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
) -> (Model, Vec<SolidKey>) {
    let mut model = model.clone();
    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(arguments, tools, rule, &Policy::certified(), &mut log)
        .unwrap_or_else(|why| panic!("this {rule:?} is decided by proof alone: {why}"));
    assert!(
        log.records().is_empty(),
        "a certified {rule:?} recorded {} decisions it could not prove",
        log.records().len()
    );
    let solids = fused.solids().to_vec();
    (model, solids)
}

/// §22.6's defect, at the one line of arithmetic it came from, now closed.
///
/// `build_side_face` hands `Plane::from_point_and_normal` the vector
/// `(along_y, -along_x, 0)` straight off the profile, so the length is whatever
/// that outline's edge happened to be. Normalizing alone made what got stored
/// depend on that length: a wall standing on an edge one long and a wall on a
/// parallel edge three long are the same plane and used to get coefficients an
/// ulp apart.
///
/// Every rectilinear family in this file was safe from it by luck rather than
/// by design — an axis-aligned normal of any length normalizes exactly, because
/// dividing `(0, 3, 0)` by `3` is exact — and the skew family is the first
/// thing here that can tilt a wall.
///
/// The constructor now scales by an exact power of two and divides by a length
/// carried to a hundred bits, so the answer depends on the ratios alone *and* is
/// the nearest `f64` to the exact one. This is the topology-side guard on the
/// first half; `rcad-geom` has the arithmetic in
/// `a_plane_is_the_same_however_long_the_normal_naming_it_was`.
#[test]
fn the_same_wall_built_from_a_longer_edge_is_the_same_plane() {
    let plane_of = |length: f64| {
        Plane::from_point_and_normal(Point::origin(), Vector::new(length, length, 0.0))
            .expect("a diagonal normal has length")
            .coefficients()
    };

    assert_eq!(plane_of(1.0), plane_of(2.0), "two is one doubled, exactly");
    assert_eq!(
        plane_of(1.0),
        plane_of(3.0),
        "the same plane, named off a longer edge"
    );
    // Both land on the same number, and since §121 it is the nearest `f64` to
    // 1/√2 rather than the ulp below. Canonical and closest are not in tension —
    // a correctly rounded normalize is a function of the exact ray, so two
    // proportional spellings still store the same bits.
    //
    // The bit this used to lose was never lost to the ordering of the
    // arithmetic; it was lost to rounding a square root and then rounding a
    // division. Carrying a hundred bits of the root is what recovers it.
    let stored = plane_of(1.0)[0];
    assert_eq!(stored, core::f64::consts::FRAC_1_SQRT_2);
    assert_eq!(
        stored.to_bits() - 1,
        0.707_106_781_186_547_5_f64.to_bits(),
        "the ulp below is what a plane stored until §121, and naming it here \
         makes a slide back to it something somebody has to look at"
    );
}

/// The search for a point inside a piece of a face, which the skew family's
/// refusals name since Q901.
///
/// Named here because [`how_often_a_certified_fuse_of_a_skew_pair_refuses`]
/// asserts on it rather than only printing it, and a bare string in an
/// assertion is a thing that goes stale silently.
///
/// It was `"order along a crossing"`, the predicate that settles which of two
/// points on a crossing comes first, until Q901 had that order asked of the
/// planes two points a rounding apart are the meets of.
const A_POINT_INSIDE_A_PIECE: &str = "a point inside a piece of a face";

/// What a flat fuse refuses when its answer keeps an edge too short to stand
/// beside its own coordinates: [`Invalid::DynamicRangeTooWide`], as
/// [`a_flat_refusal`] names it.
const AN_EDGE_TOO_SHORT_BESIDE_ITS_COORDINATES: &str =
    "an answer with an edge too short beside its coordinates";

// `"a point in a piece that every solid could place"` is what the fuse gives
// up on when it is handed two curved faces lying on the same surface (§36.16's
// row; `a_bar_with_a_copy_of_itself_taken_away_is_nothing_at_all` is the
// plainest shape that reaches it, and every law that puts an answer back
// against one of its own inputs reaches it too). It was `"a point inside a
// piece of a face"` until §217, which split that one name in two. No test
// still names it as a constant (Q1080 answered the last refusal that did).

/// What a fuse refuses when a point inside a piece was found and the ray that
/// was to place it met a face whose trimming nothing here follows:
/// [`Error::AFaceTrimmedByACurveThisCannotFollow`], as [`fused`] names it.
///
/// Counted under `"a point in a piece that every solid could place"` until row
/// Q882 named the face rather than calling the point undecided
/// (`docs/decisions.md` §35.6).
const A_FACE_NOBODY_FOLLOWS: &str = "a face trimmed by a curve this cannot follow";

/// The refusal §30.8 item 10 owns, as [`fused`] names it.
///
/// Not a predicate that gave up but a whole face the arrangement could not
/// place: [`Error::NothingSaysWhichSideOfItsTrimmingAFaceLiesOn`]. It is counted
/// alongside the predicates because the question a report answers is *what
/// stopped this*, and that does not care which of the two shapes the answer
/// arrives in.
const WHICH_SIDE_OF_ITS_TRIMMING_A_FACE_LIES_ON: &str = "which side of its trimming a face lies on";

/// What stops a fuse when one face is crossed by two different round surfaces.
///
/// `rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat`, §31.2 row 8's
/// gap, as [`fused`] names it.
///
/// The arrangement asks where three surfaces meet once per **pair of crossings
/// on a face** (`fuse/arrange.rs`'s `place_the_meets`), and the answer is only
/// worked out when two of the three are flat. A block's floor crossed by a ball
/// and by a bar is a plane, a sphere and a pipe — one plane short — and that is
/// what [`uniting_a_ball_and_a_bar_with_a_block_does_not_depend_on_which_two_go_first`]
/// stopped at, until a triple whose two round surfaces share nothing was
/// skipped rather than asked.
const WHERE_THREE_SURFACES_MEET_NEEDS_TWO_OF_THEM_FLAT: &str =
    "where three surfaces meet, with fewer than two of them flat";

/// What stops a fuse when two walks on one face cannot be told into cells.
const CANNOT_TELL_THE_CELLS_OF_A_FACE_APART: &str = "telling the cells of a face apart";

/// What stops a fuse when a face is cut by a curve that never comes back.
///
/// [`Error::ACrossingThisCannotBeOrderedAlong`], as [`fused`] names it. A
/// straight crossing is ordered by the axis it climbs fastest along and a closed
/// conic by the area three of its points sweep; a hyperbola branch is neither,
/// and §47.7's first row is the work that would settle it.
///
/// **Not the same refusal as [`A_CROSSING_THAT_DOES_NOT_CLOSE`]**, even though
/// the same geometry reaches both. This one is the fuse saying it has no
/// ordering; that one is `rcad-geom` saying the curve is open before the fuse
/// asks. §47.4 item 4 is why they are counted apart.
const ORDERING_ALONG_AN_ENDLESS_BEND: &str = "nothing can put points in order along this curve";

/// What `rcad-geom` says when it is asked to walk round a crossing that does not
/// come back to where it started.
///
/// `rcad_geom::Error::TheCrossingOfTheseTwoSurfacesIsNotClosed`. A plane down the
/// side of a cone cuts it in a hyperbola, and *"round a closed curve"* is the
/// only conic order `arrange` has.
const A_CROSSING_THAT_DOES_NOT_CLOSE: &str = "a crossing that does not close";

/// What a fuse over a bar leaning through a block refuses on that no upright
/// curved family does: `Error::AWireWillNotLieFlatOnItsFace`, a face of the
/// answer whose wire does not close in the face's own two coordinates (§191).
const A_WIRE_THAT_WILL_NOT_LIE_FLAT: &str = "a wire that will not lie flat on its face";

/// Another refusal only the leaning bar reaches,
/// `Error::CurvedFaceWithAHoleGoesTooFarRound` (§191). The upright bar never
/// does: its rims are circles square to its own axis.
const A_HOLED_FACE_TOO_FAR_ROUND: &str = "a curved face with a hole too far round to mesh";

/// What a ball turned half over refuses on: `Error::ACurvedFaceGoesRightRoundWithNoSeam`
/// (§191). A gap where the ball and block reach it (§16.10): their upright
/// union, turned the same way, is a sound model
/// ([`a_ball_and_block_united_and_then_turned_half_over_is_still_a_sound_model`]).
const A_ROUND_FACE_WITH_NO_SEAM: &str = "a curved face right round with no seam to open at";

/// A fuse over a ball turned so its own axis lies along a world axis, give or
/// take an ulp, stopping because a face it built would not mesh (§191). A gap
/// for the same reason as [`A_ROUND_FACE_WITH_NO_SEAM`].
const A_TURNED_FACE_THE_FUSE_COULD_NOT_MESH: &str = "a face of the answer the fuse could not mesh";

/// Not a fuse refusing but the mesher, over an answer the fuse gave: a face of
/// a turned answer it could not cut into triangles where the upright answer
/// meshed (§191). Counted beside the fuses' refusals so that a skipped
/// comparison says why.
const A_MESH_OF_THE_TURNED_ANSWER_REFUSED: &str =
    "measuring the answer: the mesher refused a face of it";

/// Whether this run is the shipped measurement rather than a soak.
///
/// [`TURNED_REFUSALS_MUST_STAY_BELOW`] is a statement about one deterministic
/// number over one fixed corpus, and a soak measures something else: a bigger
/// corpus with a different mean and a spread of its own. Both are printed; only
/// the shipped one is judged.
fn this_is_the_shipped_corpus() -> bool {
    std::env::var(SOAK_FACTOR).is_err() && std::env::var(SOAK_SEED).is_err()
}

/// `A ∪ B` and `B ∪ A`, then `A ∩ B` and `B ∩ A`, bit for bit.
///
/// Swapping the two sides is the sharpest question this suite asks, because a
/// splitter that cut a face by one line and then cut the pieces by the next
/// would compute a genuinely different partition each way round and there would
/// be nothing to fix (§17.4). Comparing volumes would not see it.
fn both_ways_round(
    tally: &Tally,
    coordinates: Coordinates,
    configuration: Configuration,
) -> Result<(), TestCaseError> {
    both_ways_round_of(tally, coordinates, &|| configuration.build())
}

/// The same, over whatever `build` builds, the first two solids of it swapped.
fn both_ways_round_of(
    tally: &Tally,
    coordinates: Coordinates,
    build: &dyn Fn() -> (Model, Vec<SolidKey>),
) -> Result<(), TestCaseError> {
    both_ways_round_of_noting_which_way_the_second_call_went(
        tally,
        coordinates,
        build,
        &mut |_, _| {},
    )
}

/// The same, and telling `note` the two groups of solids each of the two
/// calls actually hands to [`fused`] — the forward one, then the one that is
/// supposed to be swapped — so a test can catch a caller that quietly stopped
/// swapping them, without needing the two answers to come out different
/// (§129.5).
///
/// A union or an intersection that really commutes gives the same answer
/// whether it is asked once or twice, so nothing that reads the *answer* can
/// tell "asked the other way round" from "asked the same way twice". This
/// reads what is actually handed to [`fused`] instead.
fn both_ways_round_of_noting_which_way_the_second_call_went(
    tally: &Tally,
    coordinates: Coordinates,
    build: &dyn Fn() -> (Model, Vec<SolidKey>),
    note: &mut dyn FnMut(&[SolidKey], &[SolidKey]),
) -> Result<(), TestCaseError> {
    for rule in [Rule::Union, Rule::Intersection] {
        tally.meaning_to_compare();
        let (model, keys) = build();
        let (one, other) = (keys[0], keys[1]);

        // Both calls go through `ask`, so what `note` hears is what `fused`
        // is handed; two spellings of the pair could drift apart unseen.
        let mut ask = |arguments: &[SolidKey], tools: &[SolidKey]| {
            note(arguments, tools);
            fused(tally, coordinates, &model, arguments, tools, rule)
        };
        let forwards = ask(&[one], &[other])?;
        let backwards = ask(&[other], &[one])?;
        both_agree_about_whether_there_is_an_answer(
            tally,
            coordinates,
            &forwards,
            &backwards,
            rule,
            "swapping the two sides",
        )?;
        let (Some(forwards), Some(backwards)) = (forwards.answer(), backwards.answer()) else {
            continue;
        };

        tally.comparing();
        same_answer(
            &answer_of(&forwards.0, &forwards.1),
            &answer_of(&backwards.0, &backwards.1),
            rule,
            "swapping the two sides",
        )?;
    }
    Ok(())
}

/// **The both-ways-round check must ask [`fused`] with the two sides truly
/// swapped, not the same pair twice.**
///
/// A union of two overlapping cubes gives the same answer whichever cube is
/// named first, so a test that only compares the two *answers* cannot tell a
/// real swap from asking the same way round twice — that sameness is exactly
/// what the commutativity check is for (§129.5). This instead reads the two
/// groups of solids [`both_ways_round_of_noting_which_way_the_second_call_went`]
/// actually hands to [`fused`].
#[test]
fn the_second_call_asks_the_pair_the_other_way_round() {
    let mut model = Model::new(rcad_geom::Scale::millimetres(100.0).expect("a working size"));
    let one = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    let other = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    model
        .place_solid(
            other,
            &Isometry::translation(Vector::new(1.0, 1.0, 1.0)).expect("a finite placement"),
        )
        .expect("this model made the key");

    let tally = Tally::default();
    let mut asked: Vec<(Vec<SolidKey>, Vec<SolidKey>)> = Vec::new();
    both_ways_round_of_noting_which_way_the_second_call_went(
        &tally,
        Coordinates::AreExactlyRepresentable,
        &|| (model.clone(), vec![one, other]),
        &mut |arguments, tools| asked.push((arguments.to_vec(), tools.to_vec())),
    )
    .expect("two overlapping cubes on the integer grid fuse cleanly under a certified policy");

    assert_eq!(
        asked.len(),
        4,
        "two rules, each asking forward and then backward, is four notes: {asked:?}"
    );
    for pair in asked.chunks(2) {
        let [forward, backward] = pair else {
            panic!("expected a forward call and a swapped call together: {asked:?}");
        };
        let (forward_arguments, forward_tools) = forward;
        let (backward_arguments, backward_tools) = backward;
        assert_ne!(
            forward_arguments, forward_tools,
            "the two sides of a two-sided fuse must be different solids, or this test \
             proves nothing"
        );
        assert_eq!(
            (forward_arguments, forward_tools),
            (backward_tools, backward_arguments),
            "the second call did not swap the two sides: asked {asked:?}"
        );
        assert_ne!(
            (forward_arguments, forward_tools),
            (backward_arguments, backward_tools),
            "the second call asked the very same pair as the first, so it was never \
             asked the other way round: {asked:?}"
        );
    }
}

/// **Every run of the same code draws the same configurations** (§339).
///
/// [`over`] builds its runner with `TestRunner::new`, which takes its seed from
/// proptest's `Config::default()`: whatever `PROPTEST_RNG_SEED` names, and a
/// fresh one from the operating system when it names none. So two runners built
/// one after the other draw the same values exactly when the seed is fixed, and
/// what fixes it is one line of `.cargo/config.toml`.
#[test]
fn two_runs_of_a_property_draw_the_same_values() {
    let drawn = || {
        let seen = RefCell::new(Vec::new());
        over(8, any::<u64>(), |value, _| {
            seen.borrow_mut().push(value);
            Ok(())
        });
        seen.into_inner()
    };
    assert_eq!(
        drawn(),
        drawn(),
        "two runners built the way `over` builds them drew different values, so every \
         run of the suite tests different cases: `.cargo/config.toml` should set \
         `PROPTEST_RNG_SEED`"
    );
}

/// Ask all three rules of one configuration and keep nothing but the tally.
fn every_rule(
    tally: &Tally,
    coordinates: Coordinates,
    configuration: Configuration,
) -> Result<(), TestCaseError> {
    let (model, keys) = configuration.build();
    let before = tally.not_a_solid.get();
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        fused(tally, coordinates, &model, &[keys[0]], &[keys[1]], rule)?;
    }
    tally.a_configuration_was_not_a_solid_this_often(tally.not_a_solid.get() - before);
    Ok(())
}

/// Unite the first two, then unite that with the third.
fn united_in_turn(
    tally: &Tally,
    coordinates: Coordinates,
    model: &Model,
    first: &[SolidKey],
    second: &[SolidKey],
    third: &[SolidKey],
) -> Result<Option<(Model, Vec<SolidKey>)>, TestCaseError> {
    let Some(pair) = fused(tally, coordinates, model, first, second, Rule::Union)?.answer() else {
        return Ok(None);
    };
    Ok(fused(tally, coordinates, &pair.0, &pair.1, third, Rule::Union)?.answer())
}

/// Fuse a copy of `model`, counting what came back.
///
/// A copy, so that every fuse in a property starts from the same model rather
/// than from whatever the last one left behind. A [`Fused`] that is not an
/// answer says *which* of the two ways there was no answer — refused, or an
/// answer no `Model` can hold — because those two are different facts and one
/// property's worth of them tells the two apart (see
/// [`both_agree_about_whether_there_is_an_answer`]). Both are counted, neither
/// swept away.
///
/// Every answer goes through
/// [`every_surface_of_the_answer_is_a_surface_of_an_input`], so §17.1's deciding
/// argument is checked by every fuse in this file rather than by a test of its
/// own.
fn fused(
    tally: &Tally,
    coordinates: Coordinates,
    model: &Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
) -> Result<Fused, TestCaseError> {
    let mut model = model.clone();
    let mut log = ProvenanceLog::new();
    tally.fuses.set(tally.fuses.get() + 1);

    match model.fuse(arguments, tools, rule, &Policy::certified(), &mut log) {
        Ok(fused) => {
            let solids = fused.solids().to_vec();
            if !log.is_empty() {
                return Err(TestCaseError::fail(format!(
                    "a certified {rule:?} recorded {} decisions it could not prove",
                    log.len()
                )));
            }
            every_surface_of_the_answer_is_a_surface_of_an_input(
                &model,
                &[arguments, tools].concat(),
                &solids,
                rule,
            )?;
            Ok(Fused::Answered(Box::new(model), solids))
        }
        Err(Error::Undecidable { records }) => {
            tally.refused.set(tally.refused.get() + 1);
            for record in records.records() {
                *tally
                    .gave_up
                    .borrow_mut()
                    .entry(record.predicate())
                    .or_default() += 1;
            }
            match coordinates {
                Coordinates::AreUnderAGeneralRotation
                | Coordinates::SitOnTiltedWalls
                | Coordinates::LieOnARoundWallAndMayCoincide
                | Coordinates::LieOnAConesWall
                | Coordinates::LieOnATurnedRoundWall => Ok(Fused::Refused),
                Coordinates::AreExactlyRepresentable | Coordinates::LieOnARoundWall => {
                    Err(TestCaseError::fail(format!(
                        "a {rule:?} over a family that has never refused refused, with {} records",
                        records.len()
                    )))
                }
            }
        }
        Err(Error::ResultIsNotManifold { .. }) => {
            tally.not_a_solid.set(tally.not_a_solid.get() + 1);
            match coordinates {
                Coordinates::AreUnderAGeneralRotation
                | Coordinates::SitOnTiltedWalls
                | Coordinates::AreExactlyRepresentable
                | Coordinates::LieOnARoundWallAndMayCoincide
                | Coordinates::LieOnAConesWall
                | Coordinates::LieOnATurnedRoundWall => Ok(Fused::NotASolid),
                Coordinates::LieOnARoundWall => Err(TestCaseError::fail(format!(
                    "a {rule:?} over a family that has never handed back four faces along \
                     one edge did, and that is not something this kernel can hold"
                ))),
            }
        }
        Err(why) => {
            let allowed = match coordinates {
                Coordinates::LieOnATurnedRoundWall => a_turned_curved_refusal(&why),
                Coordinates::LieOnARoundWallAndMayCoincide | Coordinates::LieOnAConesWall => {
                    a_curved_refusal(&why)
                }
                Coordinates::AreUnderAGeneralRotation | Coordinates::SitOnTiltedWalls => {
                    a_flat_refusal(&why)
                }
                Coordinates::AreExactlyRepresentable | Coordinates::LieOnARoundWall => None,
            };
            let Some(named) = allowed else {
                return Err(TestCaseError::fail(format!("{rule:?} went wrong: {why}")));
            };
            tally.refused.set(tally.refused.get() + 1);
            *tally.gave_up.borrow_mut().entry(named).or_default() += 1;
            Ok(Fused::Refused)
        }
    }
}

/// The one refusal a flat fuse whose corners are not all exact is allowed to
/// hit by name, rather than as a predicate giving up: an answer with an edge
/// too short to stand beside its largest coordinate in `f64`,
/// [`Invalid::DynamicRangeTooWide`]. Anything else is a bug, so this hands back
/// `None` for it and [`fused`] fails.
///
/// **A limit and not a gap**, by `Assembly::check_dynamic_range`'s argument in
/// `fuse/assemble.rs`: the ratio is a length over a length, so no scaling of
/// the answer fits the two in `f64` together, and the short edge is where the
/// two inputs' stored planes really put it.
///
/// **The turned and skew families reach it since Q1279 and Q1288**, which let
/// `by_a_plane` in `fuse/select.rs` place a piece with corners on the plane, so
/// fuses that stopped for want of a point inside a piece of a face now get as
/// far as the answer, and some answers keep a side a rounding long.
/// `tests/three_cut_lines_through_one_corner_of_a_skew_pair.rs` pins one: an
/// edge `2^-51` long beside a coordinate of 5.
fn a_flat_refusal(why: &Error) -> Option<&'static str> {
    matches!(why, Error::Invalid(Invalid::DynamicRangeTooWide { .. }))
        .then_some(AN_EDGE_TOO_SHORT_BESIDE_ITS_COORDINATES)
}

/// The refusals a curved fuse is allowed to hit today, by name.
///
/// Every one is a gap the kernel's own error documentation already owns: §30.8
/// item 10 for the first, §33.1 for the second, §31.2 row 8 for the third,
/// §35.6 for a face nobody follows, and §47.2 for the last two. Anything else is a bug, so this hands back `None` for
/// it and [`fused`] fails.
///
/// **The last two are kept apart rather than folded into one name**, which is
/// §47.4 item 4's warning taken seriously: an open bend and a quartic are two
/// unrelated problems, and a report that named them the same way would let a
/// count move from one to the other unseen.
fn a_curved_refusal(why: &Error) -> Option<&'static str> {
    match why {
        Error::NothingSaysWhichSideOfItsTrimmingAFaceLiesOn { .. } => {
            Some(WHICH_SIDE_OF_ITS_TRIMMING_A_FACE_LIES_ON)
        }
        Error::AFaceMissesPartOfWhereItsSurfacesCross { .. } => {
            Some("a face running along all of where its surfaces cross")
        }
        Error::Geometry(rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat) => {
            Some(WHERE_THREE_SURFACES_MEET_NEEDS_TWO_OF_THEM_FLAT)
        }
        Error::CannotTellTheCellsOfAFaceApart { .. } => Some(CANNOT_TELL_THE_CELLS_OF_A_FACE_APART),
        Error::AFaceTrimmedByACurveThisCannotFollow { .. } => Some(A_FACE_NOBODY_FOLLOWS),
        Error::ACrossingThisCannotBeOrderedAlong => Some(ORDERING_ALONG_AN_ENDLESS_BEND),
        Error::Geometry(rcad_geom::Error::TheCrossingOfTheseTwoSurfacesIsNotClosed) => {
            Some(A_CROSSING_THAT_DOES_NOT_CLOSE)
        }
        Error::Undecidable { .. } | Error::ResultIsNotManifold { .. } => {
            unreachable!("those two are matched above")
        }
        _ => None,
    }
}

/// The refusals a curved fuse whose round wall is not square to the world is
/// allowed to hit: [`a_curved_refusal`]'s, and the ones §191 found that no
/// upright family reaches.
///
/// Kept apart from [`a_curved_refusal`] so that a refusal found here does not
/// quietly become allowed on the upright families, where it has never been
/// seen and would be new.
fn a_turned_curved_refusal(why: &Error) -> Option<&'static str> {
    match why {
        Error::AWireWillNotLieFlatOnItsFace { .. } => Some(A_WIRE_THAT_WILL_NOT_LIE_FLAT),
        Error::CurvedFaceWithAHoleGoesTooFarRound { .. } => Some(A_HOLED_FACE_TOO_FAR_ROUND),
        Error::ACurvedFaceGoesRightRoundWithNoSeam { .. } => Some(A_ROUND_FACE_WITH_NO_SEAM),
        Error::CannotMeshAFace { .. } | Error::TwoPatchesOfTheMeshDrewTheSameSide { .. } => {
            Some(A_TURNED_FACE_THE_FUSE_COULD_NOT_MESH)
        }
        _ => a_curved_refusal(why),
    }
}

/// What one fuse came back with.
///
/// Three outcomes and not two. §48.2 item 5: the arm that catches
/// [`Error::ResultIsNotManifold`] used to hand back the same `None` a refusal
/// does, so a property could not tell the two apart and `A ∪ B` coming back as
/// something no `Model` can hold while `B ∪ A` answered was skipped in silence.
/// Order-dependence, invisible to the suite whose one job is finding it.
enum Fused {
    /// The fuse answered: the model it left behind and the solids of the answer.
    ///
    /// Boxed because a `Model` is four hundred bytes and the other two arms are
    /// empty, and every fuse in the file builds one of these.
    Answered(Box<Model>, Vec<SolidKey>),
    /// The kernel would not decide it. Counted, and a failure on the families
    /// that have never refused.
    Refused,
    /// The answer was not something a `Model` can hold — four faces along one
    /// edge, and the rest of §17.7's catalogue.
    NotASolid,
}

impl Fused {
    fn answer(self) -> Option<(Model, Vec<SolidKey>)> {
        match self {
            Fused::Answered(model, solids) => Some((*model, solids)),
            Fused::Refused | Fused::NotASolid => None,
        }
    }
}

/// Two runs of the same question may not disagree about whether the answer is a
/// solid.
///
/// §48.2 item 5's gap, closed. Whether four faces end up along one edge is a
/// fact about the answer and not about the path taken to it, so two runs that
/// both decide have to agree about it: one of them handing back a solid while
/// the other says the answer is not one is order-dependence in the arrangement,
/// whatever family it happens on.
///
/// A refusal on one side is *not* held to this. That is a fact about the
/// arithmetic and the two sides genuinely take different paths through it, which
/// is why the families where a refusal is an outcome to count exist at all.
///
/// # Which family this fails on, and which it only counts on
///
/// **It counts on the lattice and skew families and fails on the rest.** Turning
/// it on found the disagreement immediately on the flat families — on the
/// turned pairs, on the skew pairs, and, once the shape was shrunk, on the
/// lattice pairs, where nothing rounds at all. The lattice end of that is fixed
/// and now says so
/// ([`a_lattice_pair_unites_to_the_same_solid_whichever_way_round_it_is_handed_over`]).
/// The turned end is fixed too
/// ([`turning_a_frame_and_a_comb_shares_the_same_two_solids_whichever_comes_first`],
/// §351.6): the swap property drew none over 979 turned pairs at a
/// [`SOAK_FACTOR`] of 20, nor did any property of this file at a factor of 5,
/// so the turned family fails here now.
///
/// A property that is red for a reason already written down stops catching
/// the reasons that are not. So what the lattice and skew families get is a
/// count in their report, and what the turned and curved families get is a
/// failure — the curved ones have never handed back an answer no `Model` can
/// hold at all, so a disagreement there would be new.
///
/// **The counted rate is the number to watch.** It is printed by [`Tally::report`]
/// and it is zero on a fixed kernel.
fn both_agree_about_whether_there_is_an_answer(
    tally: &Tally,
    coordinates: Coordinates,
    one: &Fused,
    other: &Fused,
    rule: Rule,
    what_changed: &str,
) -> Result<(), TestCaseError> {
    match (one, other) {
        (Fused::NotASolid, Fused::Answered(..)) | (Fused::Answered(..), Fused::NotASolid) => {}
        _ => return Ok(()),
    }

    tally
        .disagreed_about_being_a_solid
        .set(tally.disagreed_about_being_a_solid.get() + 1);
    match coordinates {
        Coordinates::AreExactlyRepresentable | Coordinates::SitOnTiltedWalls => Ok(()),
        Coordinates::AreUnderAGeneralRotation
        | Coordinates::LieOnARoundWall
        | Coordinates::LieOnARoundWallAndMayCoincide
        | Coordinates::LieOnAConesWall
        | Coordinates::LieOnATurnedRoundWall => Err(TestCaseError::fail(format!(
            "{what_changed} turned a {rule:?} whose answer is a solid into one whose \
             answer no `Model` can hold, which cannot depend on the path taken"
        ))),
    }
}

/// §17.1's deciding argument, checked rather than argued.
///
/// The settled reason `Surface::Plane` is authoritative and the stored corners
/// are cached samples of it is drift across chained fuses: "a split face
/// inherits its parent's plane bit for bit, so however long the chain the planes
/// in the model stay the planes the user built, undegraded." That sentence is
/// the whole case against vertex-authoritative geometry and nothing in the tree
/// checked it, so this does: every face of every answer has to carry the four
/// coefficients of some input face's plane, bit for bit, or their exact
/// negation.
///
/// The negation is not slack. A rule can keep a face facing the other way, and
/// `Plane::reversed` negates all four coefficients, which is exact. Nothing else
/// is allowed: not a refitted plane, not a renormalized one, not one whose last
/// bit moved.
///
/// It runs inside [`fused`] rather than as a test of its own, so every fuse this
/// file already performs checks it and the wall clock does not move.
///
/// # Why it runs on turned pairs too
///
/// It could not until `Plane::transformed` stopped re-deriving a moved plane's
/// offset through a point on it. `FaceView::plane` is
/// `surface.transformed(&location)`, so a turned input read at its placement had
/// been through arithmetic the answer read at the identity had not, and a
/// mismatch said nothing about what the fuse stored. A read is a read now, at
/// any placement, so the turned families are held to the same bit-for-bit
/// statement as the rectilinear ones — which is where it bites, because a
/// turned normal is not a signed unit axis and nothing about it is exact by
/// construction.
fn every_surface_of_the_answer_is_a_surface_of_an_input(
    model: &Model,
    inputs: &[SolidKey],
    answer: &[SolidKey],
    rule: Rule,
) -> Result<(), TestCaseError> {
    let mut theirs: BTreeSet<Vec<u64>> = BTreeSet::new();
    for &solid in inputs {
        for face in model.solid(solid).expect("this model made the key").faces() {
            let surface = face.surface();
            theirs.insert(WhatItSitsOn::of(&surface).written_out());
            if let Some(plane) = surface.as_plane() {
                theirs.insert(
                    WhatItSitsOn::Flat(plane.coefficients().map(|coefficient| -coefficient))
                        .written_out(),
                );
            }
        }
    }

    for &solid in answer {
        for face in model.solid(solid).expect("this model made the key").faces() {
            let surface = face.surface();
            if !theirs.contains(&WhatItSitsOn::of(&surface).written_out()) {
                return Err(TestCaseError::fail(format!(
                    "a {rule:?} put a face on {surface:?}, which is no input face's surface \
                     and no input face's plane reversed"
                )));
            }
        }
    }
    Ok(())
}

/// What kind of numbers the configuration is made of, and so whether a refusal
/// is an outcome to count or a test failure.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Coordinates {
    /// Rectilinear on the integer grid, where a certified policy has to decide
    /// every fuse (§17.7's control row).
    AreExactlyRepresentable,
    /// Under a general rotation, where a refusal is the thing being measured.
    AreUnderAGeneralRotation,
    /// Whole-number corners standing on walls that are not axis-aligned, where
    /// a refusal is again an outcome to count rather than a failure.
    SitOnTiltedWalls,
    /// Whole-number corners with one round wall among them.
    ///
    /// **A refusal is a failure here too**, and the corners are *not* exactly
    /// representable: where a block's flat wall meets a bar's round one the
    /// answer is a square root. What makes the claim fair anyway is that §36.27
    /// measures this arrangement at nothing refused and nothing recorded, so a
    /// refusal would be a regression rather than a fact about the geometry —
    /// which is the same thing the rectilinear control rows say, reached by a
    /// different route.
    LieOnARoundWall,
    /// The same, in a chain of fuses deep enough that one of them is handed two
    /// curved faces lying on the same surface.
    ///
    /// **A refusal is an outcome to count here**, unlike
    /// [`Coordinates::LieOnARoundWall`], and the difference is §36.16's row:
    /// two coincident curved faces are refused today, by name, and
    /// [`a_bar_with_a_copy_of_itself_taken_away_is_nothing_at_all`] pins the
    /// plainest shape that reaches it. Every law that puts an answer back
    /// against one of its own inputs — absorption, `A \ (A \ B)`,
    /// `((A ∪ B) \ B) ∪ B` — reaches it too, so a chain like that is measured
    /// rather than asserted, and the vacuity floor is what says whether the
    /// measurement is worth anything.
    LieOnARoundWallAndMayCoincide,
    /// Whole-number corners with a **cone's** wall among them.
    ///
    /// **A refusal is an outcome to count here**, and unlike every other family
    /// in this file it is the only outcome there is. A block that really
    /// overlaps a frustum has a side wall down the cone, a plane down the side
    /// of a cone cuts it in a hyperbola, and an open bend is not something
    /// `arrange` can put points in order along (§47.2, §47.7's first row).
    ///
    /// An answer no `Model` can hold is counted rather than failed for the same
    /// reason: nothing has ever seen one from this family, so there is no
    /// measurement saying it would be new.
    LieOnAConesWall,
    /// A round wall whose axis is not an axis of the world: a curved fixture
    /// under one general rotation, or a bar leaning through a block (§191).
    ///
    /// **A refusal is an outcome to count**, by `Undecidable` or by one of the
    /// curved refusals' own names, because nothing had ever drawn one of these
    /// and so nothing says a refusal would be new. An answer no `Model` can
    /// hold is counted too. **Two runs disagreeing about whether the answer is
    /// a solid is a failure**, as on every curved family: that is a fact about
    /// the answer and not about the arithmetic.
    LieOnATurnedRoundWall,
}

/// What every fuse a property ran came back with.
///
/// Counted rather than filtered. §12.3's second lesson is that
/// `prop_filter_map(… .ok())` silently discards every input the kernel refused,
/// which is the exact mechanism by which a wrongly refused input stays
/// invisible.
#[derive(Default)]
struct Tally {
    fuses: Cell<usize>,
    refused: Cell<usize>,
    not_a_solid: Cell<usize>,
    /// How often two runs of one question disagreed about whether the answer
    /// was a solid at all.
    ///
    /// §48.2 item 5's blind spot, now a number. Zero is the only right value —
    /// see [`both_agree_about_whether_there_is_an_answer`] for which families
    /// it fails on and which it only counts on.
    disagreed_about_being_a_solid: Cell<usize>,
    /// How many configurations [`every_rule`] asked had none, one, two or all
    /// three of their fuses come back not a solid.
    ///
    /// The three fuses of one configuration are not three draws: a union, an
    /// overlap and a difference of the same two solids. Whether they come back
    /// not a solid together or one at a time is what sets how far a draw's
    /// count strays, so a not-a-solid band is only as right as this is
    /// measured (§303.6).
    not_a_solid_of_three: Cell<[usize; 4]>,
    /// Comparisons a property set out to make, and made.
    ///
    /// Every property here returns `Ok(())` the moment a fuse comes back with
    /// no answer, so a property whose fuses all stopped answering would assert
    /// nothing at all and pass. These two are what say it did not.
    meant_to_compare: Cell<usize>,
    compared: Cell<usize>,
    /// Which predicate each refusal named, and how often.
    ///
    /// Criterion 8 asks for a rate, and a rate on its own cannot tell a timid
    /// filter from genuine undecidability (§14.7). The name of the predicate
    /// that gave up is the cheapest thing that can.
    gave_up: RefCell<BTreeMap<&'static str, usize>>,
}

impl Tally {
    /// The property is about to try a comparison. Call it before the first
    /// fuse the comparison needs.
    fn meaning_to_compare(&self) {
        self.meant_to_compare.set(self.meant_to_compare.get() + 1);
    }

    /// The property got both sides and is about to compare them.
    fn comparing(&self) {
        self.compared.set(self.compared.get() + 1);
    }

    /// One configuration's three fuses are asked, and `of_three` of them came
    /// back not a solid.
    fn a_configuration_was_not_a_solid_this_often(&self, of_three: usize) {
        let mut counts = self.not_a_solid_of_three.get();
        counts[of_three] += 1;
        self.not_a_solid_of_three.set(counts);
    }

    fn report(&self, what: &str) {
        let fuses = self.fuses.get();
        println!(
            "{what}: {fuses} fuses, {} refused ({:.1}%), {} not a solid ({:.1}%)",
            self.refused.get(),
            100.0 * self.refused.get() as f64 / fuses as f64,
            self.not_a_solid.get(),
            100.0 * self.not_a_solid.get() as f64 / fuses as f64
        );
        if self.disagreed_about_being_a_solid.get() > 0 {
            println!(
                "    {} of them disagreed with their other way round about whether the \
                 answer is a solid",
                self.disagreed_about_being_a_solid.get()
            );
        }
        let [none, one, two, three] = self.not_a_solid_of_three.get();
        if none + one + two + three > 0 {
            println!(
                "    configurations with none, one, two and three of their fuses not a solid: \
                 {none}, {one}, {two}, {three}"
            );
        }
        // Two decimal places, because the rarest cause a soak turns up runs at
        // about one fuse in eight hundred and one place would round it to the
        // same 0.1% as a cause ten times commoner.
        for (predicate, count) in self.gave_up.borrow().iter() {
            println!(
                "    {count} ({:.2}% of all fuses) named \"{predicate}\"",
                100.0 * *count as f64 / fuses as f64
            );
        }
    }

    /// Report, and insist the property actually compared something.
    ///
    /// This is the shape `turning_a_fuse.rs` already uses, next door and for
    /// the same reason (§14.7). Every property in this file hands back `Ok(())`
    /// when a fuse comes back with no answer, so the pass says nothing on its
    /// own; what says something is how many of the comparisons the property set
    /// out to make it reached.
    fn report_what_it_compared(&self, what: &str) {
        self.report_what_it_compared_at_least(what, SOME_COMPARISONS_MUST_HAPPEN);
    }

    /// The same against a floor the caller names.
    ///
    /// Two families need it, each for the same reason: their laws measure too
    /// thin for even the raised [`SOME_COMPARISONS_MUST_HAPPEN`] to sit below
    /// their own worst without brushing it. `A_THINNER_LAW_STILL_COMPARES_THIS_MUCH`
    /// serves five properties this way and `A_LEANING_BAR_STILL_COMPARES_THIS_MUCH`
    /// two more (§276). §279 raised the shared floor itself high enough to
    /// catch a halving of every other property in the file, curved included,
    /// so no further per-law floor was needed there — one shared number now
    /// covers the rest of the curved family along with the flat one. A floor
    /// a property brushes is a flake and not a guard, and moving the shared
    /// number to suit one family would blunt it for the dozen-odd properties
    /// that still measure well clear of it.
    fn report_what_it_compared_at_least(&self, what: &str, floor: f64) {
        self.report(what);
        let meant_to = self.meant_to_compare.get();
        assert!(
            meant_to > 0,
            "{what} never said it was going to compare anything, so this floor \
             is measuring nothing"
        );
        println!(
            "    {} of {meant_to} comparisons reached ({:.0}%)",
            self.compared.get(),
            100.0 * self.compared.get() as f64 / meant_to as f64
        );
        assert!(
            self.compared.get() as f64 >= floor * meant_to as f64,
            "{what}: only {} of {meant_to} comparisons had an answer on both sides, \
             so this property barely asserted anything",
            self.compared.get()
        );
    }

    /// Insist the share refused is under `ceiling`, on the shipped corpus only
    /// ([`this_is_the_shipped_corpus`] says why).
    fn refuses_no_more_than(&self, ceiling: f64) {
        let refused = self.refused.get() as f64 / self.fuses.get() as f64;
        if this_is_the_shipped_corpus() {
            assert!(
                refused < ceiling,
                "{:.1}% of {} fuses refused, over the {:.0}% this row was measured under",
                refused * 100.0,
                self.fuses.get(),
                ceiling * 100.0
            );
        }
    }

    /// Insist `predicate` is the one the most refusals named.
    ///
    /// A rate says how often the kernel gives up and the breakdown says on
    /// what, and for one family the breakdown is the claim. §22's skew pairs
    /// exist to reach an arrangement no other family can draw — three cut lines
    /// through one point, where putting two intersections in order along a line
    /// is a tie rather than a comparison. If that ever stops being what this
    /// family mostly refuses on, the family has quietly become its own control
    /// and its headline number is measuring the tilt over again.
    ///
    /// Judged on the shipped corpus only, for the same reason the thresholds
    /// are: this is deterministic there — 30 naming it against 12 naming
    /// `"side of plane"` — while a soak is a different corpus with a spread of
    /// its own. Four soaked seeds put the margin far wider than the shipped one
    /// rather than narrower, so this is the conservative place to judge it.
    ///
    /// The 30 has not moved since §17.7; the 21 beside it came down to 12 when
    /// the gathering step stopped asking `Plane::agreement_with` and then
    /// `planes_are_parallel` about every pair of faces and started asking
    /// [`intersect`](rcad_geom::intersect), which settles the same three cases
    /// in one call and gives up in fewer places on the way (§36).
    fn the_cause_named_most_often_is(&self, predicate: &str) {
        let gave_up = self.gave_up.borrow();
        let most = gave_up
            .iter()
            .max_by_key(|&(_, count)| *count)
            .map(|(name, _)| *name);
        assert_eq!(
            most,
            Some(predicate),
            "the refusals mostly named {most:?} and not {predicate:?}, so this family \
             has stopped reaching the branch it was built for: {gave_up:?}"
        );
    }

    /// Insist the share of fuses whose answer was not a solid is one a draw of
    /// this many fuses could give from a family whose rate is inside `rate`.
    ///
    /// A draw lands off its family's rate by chance, by a spread of
    /// `sqrt(rate * (1 - rate) / fuses)`: about 0.65 points at the shipped 441
    /// fuses and 0.08 at a soak's 25 401 (§303). Fixed ends are too tight for
    /// the one or too loose for the other, so each end is moved out by
    /// [`A_DRAW_STRAYS_THIS_MANY_SPREADS`] of its own spread at this draw's size.
    ///
    /// That spread is the binomial's, which counts every fuse as a draw of its
    /// own. A configuration is the draw, and its three fuses can come back not
    /// a solid together, so the spread is widened by
    /// [`Tally::how_many_not_solids_come_together`] (§303.6).
    ///
    /// The floor never goes below one fuse. None at all is the family no longer
    /// reaching the configuration it was built for, and that fires at any size.
    fn rate_of_answers_that_were_not_solids_must_be_within(&self, rate: RangeInclusive<f64>) {
        self.share_of_answers_that_were_not_solids_must_be_within(band_for_a_draw_of(
            self.fuses.get(),
            self.how_many_not_solids_come_together(),
            &rate,
        ));
    }

    /// For each fuse that came back not a solid, how many of its
    /// configuration's three fuses did, itself included, on average.
    ///
    /// One when no configuration gave more than one, which is nearly every
    /// draw of every family on this tree; three when every one that gave any
    /// gave all three. A draw's
    /// configurations are drawn one after another from one seed, so its count
    /// is a sum of independent counts out of three, and its variance is at most
    /// this many times the binomial's. With `X` a configuration's count and `p`
    /// the rate, `E[X²]` is this times `E[X] = 3p`, so `Var X = this · 3p −
    /// 9p²`, which is at most the binomial's `3p(1 − p)` times this because
    /// this is at most three (§303.6).
    ///
    /// One for a draw with none at all, which the floor fails anyway.
    ///
    /// # Panics
    ///
    /// If a fuse came back not a solid outside [`every_rule`], whose
    /// configuration was not counted, so the spread would leave it out.
    fn how_many_not_solids_come_together(&self) -> f64 {
        let counts = self.not_a_solid_of_three.get();
        let (fuses, together) = (0..4).fold((0, 0), |(fuses, together), of_three| {
            (
                fuses + of_three * counts[of_three],
                together + of_three * of_three * counts[of_three],
            )
        });
        assert_eq!(
            fuses,
            self.not_a_solid.get(),
            "every fuse that came back not a solid has to be counted with its configuration"
        );
        if fuses == 0 {
            return 1.0;
        }
        together as f64 / fuses as f64
    }

    /// Insist the share of fuses whose answer was not a solid this kernel can
    /// hold is still about what it has always been.
    ///
    /// The other half of the vacuity guard, and the half that is not about a
    /// property at all. [`Coordinates::AreExactlyRepresentable`] turns a refusal into a failure and
    /// the control rows assert `refused == 0`, but
    /// [`Error::ResultIsNotManifold`] is neither: it is `None`, counted, printed
    /// and never judged. A regression that made *every* lattice fuse come back
    /// not-a-solid would leave `refused == 0` true and every metamorphic
    /// property vacuously green.
    ///
    /// It is a band and not a zero because it is a real answer to a real
    /// question: four faces along one edge is not something a `Model` can hold,
    /// and the corpus is built to produce exactly that (§17.7's table has two of
    /// them by hand). Where the ends come from is at each call site.
    fn share_of_answers_that_were_not_solids_must_be_within(&self, band: RangeInclusive<f64>) {
        let share = self.not_a_solid.get() as f64 / self.fuses.get() as f64;
        assert!(
            band.contains(&share),
            "{} of {} fuses came back not a solid ({:.3}%), outside the {:.3}%–{:.3}% \
             this family has always given",
            self.not_a_solid.get(),
            self.fuses.get(),
            100.0 * share,
            100.0 * band.start(),
            100.0 * band.end()
        );
    }
}

/// The shares a draw of `fuses` fuses may give from a family whose not-a-solid
/// rate is inside `rate`: each end moved out by
/// [`A_DRAW_STRAYS_THIS_MANY_SPREADS`] of its own spread at that size, and the
/// floor never under one fuse.
///
/// The spread is the binomial's widened by the square root of `together`,
/// [`Tally::how_many_not_solids_come_together`].
fn band_for_a_draw_of(
    fuses: usize,
    together: f64,
    rate: &RangeInclusive<f64>,
) -> RangeInclusive<f64> {
    assert!(
        (1.0..=3.0).contains(&together),
        "a configuration has three fuses, so {together} cannot be how many come back not a \
         solid together"
    );
    let fuses = fuses as f64;
    let stray =
        |end: f64| A_DRAW_STRAYS_THIS_MANY_SPREADS * (together * end * (1.0 - end) / fuses).sqrt();
    let floor = (rate.start() - stray(*rate.start())).max(1.0 / fuses);
    let ceiling = rate.end() + stray(*rate.end());
    floor..=ceiling
}

/// Run `property` over `cases` configurations from the seed `PROPTEST_RNG_SEED`
/// names, and hand back what its fuses did.
///
/// `.cargo/config.toml` names one, so every run of the same code draws the same
/// configurations, and a soak names its own in the shell (§339).
///
/// `cases` is scaled by [`SOAK_FACTOR`] the same way [`measured_cases`] scales
/// [`MEASURED_CASES`], so a soak reaches these properties too instead of just
/// moving their fixed-size draw around (§290.1).
fn over<T: core::fmt::Debug>(
    cases: u32,
    strategy: impl Strategy<Value = T>,
    property: impl Fn(T, &Tally) -> Result<(), TestCaseError>,
) -> Tally {
    over_scaled_by(cases, soak_factor(), strategy, property)
}

/// [`over`]'s own scaling and run, pulled out with `factor` as a plain
/// argument in place of reading [`SOAK_FACTOR`] itself.
///
/// A test can hand this a literal factor and count how many times `property`
/// actually ran, which checks the scaling reaches the run rather than just
/// checking [`scaled_by_soak_factor`]'s arithmetic on its own — the gap
/// `over_scaled_by_runs_the_literal_cases_times_the_factor` closes (§290,
/// Q266): a revert of `over`'s call into this scaling, with that arithmetic
/// test kept, used to still pass.
fn over_scaled_by<T: core::fmt::Debug>(
    cases: u32,
    factor: u32,
    strategy: impl Strategy<Value = T>,
    property: impl Fn(T, &Tally) -> Result<(), TestCaseError>,
) -> Tally {
    run(
        TestRunner::new(configured(scaled_by_soak_factor(cases, factor))),
        strategy,
        property,
    )
}

/// Q266: the only test of `over`'s scaling used to be
/// `scaled_by_soak_factor_gives_the_literal_back_at_one_and_sixty_six_times_it_at_sixty_six`,
/// which checks [`scaled_by_soak_factor`] alone — so `over` could stop
/// calling it and every test would still pass. This one runs
/// [`over_scaled_by`] itself, with a literal factor rather than
/// `RCAD_SOAK_FACTOR`, and counts how many times the property was actually
/// called.
///
/// The count at one factor alone is not `CASES * factor`:
/// `fuses_that_must_agree.proptest-regressions` carries persisted failure
/// seeds that `configured`'s `failure_persistence` replays once regardless of
/// `cases`, on top of the fresh draw. That replay count does not depend on
/// `factor`, so it cancels out of the difference between two factors, which
/// is what this asserts instead.
#[test]
fn over_scaled_by_runs_the_literal_cases_times_the_factor() {
    let runs_at = |factor: u32| {
        let runs = Cell::new(0u32);
        over_scaled_by(CASES, factor, any::<u8>(), |_, _tally| {
            runs.set(runs.get() + 1);
            Ok(())
        });
        runs.get()
    };
    let (at_one, at_three) = (runs_at(1), runs_at(3));
    assert_eq!(
        at_three - at_one,
        CASES * 2,
        "over_scaled_by ran {at_one} times at factor 1 and {at_three} at \
         factor 3; the difference should be the literal cases times the \
         change in factor, {}",
        CASES * 2
    );
}

/// `cases` configurations from the fixed seed, whatever [`SOAK_SEED`] says.
///
/// For a law over a family that refuses most of what it draws, where a fresh
/// seed would move the share compared by more than any floor could allow.
fn over_a_fixed_corpus<T: core::fmt::Debug>(
    cases: u32,
    strategy: impl Strategy<Value = T>,
    property: impl Fn(T, &Tally) -> Result<(), TestCaseError>,
) -> Tally {
    run(
        TestRunner::new_with_rng(
            configured(cases),
            TestRng::deterministic_rng(RngAlgorithm::ChaCha),
        ),
        strategy,
        property,
    )
}

/// The same from a fixed seed, so the number that comes out is the same every
/// time it is asked for.
///
/// [`SOAK_FACTOR`] multiplies the corpus and [`SOAK_SEED`] replaces the seed;
/// with neither set — which is how the suite runs it — this is
/// [`MEASURED_CASES`] configurations from the one fixed seed.
fn measured<T: core::fmt::Debug>(
    strategy: impl Strategy<Value = T>,
    property: impl Fn(T, &Tally) -> Result<(), TestCaseError>,
) -> Tally {
    run(
        TestRunner::new_with_rng(configured(measured_cases()), fresh_seeded_soak()),
        strategy,
        property,
    )
}

/// [`MEASURED_CASES`], multiplied by whatever [`SOAK_FACTOR`] says.
///
/// # Panics
///
/// Whatever [`soak_factor`] and [`scaled_by_soak_factor`] panic for: the
/// variable set to something that is not a whole number above zero, or to one
/// whose product is more than a run can count. A soak that silently ran the
/// default corpus would be worse than one that stopped, and a product that
/// wrapped runs **fewer** cases than the shipped corpus while the operator reads
/// the number as the wide one (§48.2, §50.9). Measured: at a factor of
/// 33 554 432 this used to hand proptest a corpus of nought, and the control row
/// went green over the 15 replayed regression seeds — 45 fuses where the shipped
/// corpus is 429.
fn measured_cases() -> u32 {
    scaled_by_soak_factor(MEASURED_CASES, soak_factor())
}

/// [`SOAK_FACTOR`], or one if it is not set.
///
/// # Panics
///
/// If the variable is set to something that is not a whole number above zero.
fn soak_factor() -> u32 {
    let Ok(factor) = std::env::var(SOAK_FACTOR) else {
        return 1;
    };
    let factor: u32 = factor
        .parse()
        .unwrap_or_else(|_| panic!("{SOAK_FACTOR} has to be a whole number, not {factor:?}"));
    assert!(factor > 0, "{SOAK_FACTOR} has to be at least one");
    factor
}

/// `cases`, multiplied by `factor`, with no environment read of its own — the
/// callers that need [`SOAK_FACTOR`] itself go through [`soak_factor`] first.
///
/// # Panics
///
/// If the product is more than a run can count.
fn scaled_by_soak_factor(cases: u32, factor: u32) -> u32 {
    cases
        .checked_mul(factor)
        .unwrap_or_else(|| panic!("{cases} cases times {factor} is more than one run can count"))
}

#[test]
fn scaled_by_soak_factor_gives_the_literal_back_at_one_and_sixty_six_times_it_at_sixty_six() {
    assert_eq!(scaled_by_soak_factor(CASES, 1), CASES);
    assert_eq!(scaled_by_soak_factor(CASES, 66), CASES * 66);
    assert_eq!(scaled_by_soak_factor(LONGER_CASES, 1), LONGER_CASES);
    assert_eq!(scaled_by_soak_factor(LONGER_CASES, 66), LONGER_CASES * 66);
    assert_eq!(scaled_by_soak_factor(MEASURED_CASES, 1), MEASURED_CASES);
    assert_eq!(
        scaled_by_soak_factor(MEASURED_CASES, 66),
        MEASURED_CASES * 66
    );
}

/// The fixed seed, or the one [`SOAK_SEED`] names.
///
/// # Panics
///
/// If the variable is set to something that is not a whole number.
fn fresh_seeded_soak() -> TestRng {
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

fn run<T: core::fmt::Debug>(
    mut runner: TestRunner,
    strategy: impl Strategy<Value = T>,
    property: impl Fn(T, &Tally) -> Result<(), TestCaseError>,
) -> Tally {
    let tally = Tally::default();
    if let Err(why) = runner.run(&strategy, |value| property(value, &tally)) {
        panic!("{why}");
    }
    tally
}

fn configured(cases: u32) -> Config {
    Config {
        cases,
        source_file: Some(file!()),
        // Said out loud rather than left to the default, which looks for the
        // crate root above the test file, does not find one, and warns on every
        // run.
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
            "proptest-regressions",
        ))),
        ..Config::default()
    }
}

/// How much space an answer takes up, all of its solids together.
fn volume_of(model: &Model, solids: &[SolidKey]) -> f64 {
    volume_at(model, solids, a_deflection())
}

/// The same, measured to a deflection the caller names.
///
/// Only [`what_a_block_and_a_ball_take_up_adds_up_only_as_well_as_the_mesh_does`]
/// wants that, and it wants it because the deflection is the thing it is
/// measuring.
fn volume_at(model: &Model, solids: &[SolidKey], deflection: Deflection) -> f64 {
    solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(deflection, &Policy::certified(), &mut ProvenanceLog::new())
                .expect("a solid this size measures")
                .volume()
        })
        .sum()
}

/// The measurements of an answer that do not depend on how it was cut up.
///
/// One entry per solid, sorted, because two ways of reaching the same answer may
/// hand the same solids back in a different order.
fn invariants_of(model: &Model, solids: &[SolidKey]) -> Vec<Measured> {
    let mut measured: Vec<Measured> = solids
        .iter()
        .map(|&solid| {
            let solid = model.solid(solid).expect("this model made the key");
            let measure = solid
                .measure(
                    a_deflection(),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a solid this size measures");
            Measured {
                volume: measure.volume(),
                surface_area: measure.surface_area(),
                centroid: measure.centroid().coordinates(),
                shells: solid.shells().count(),
            }
        })
        .collect();
    measured.sort_by_key(Measured::order);
    measured
}

/// One solid of an answer, measured.
#[derive(Clone, Debug, PartialEq)]
struct Measured {
    volume: f64,
    surface_area: f64,
    centroid: [f64; 3],
    shells: usize,
}

impl Measured {
    fn order(&self) -> ([u64; 5], usize) {
        (
            [
                bits(self.volume),
                bits(self.surface_area),
                bits(self.centroid[0]),
                bits(self.centroid[1]),
                bits(self.centroid[2]),
            ],
            self.shells,
        )
    }
}

/// `comparison::same_answer`, with proptest's error where the shared version
/// returns a plain `String`.
fn same_answer(
    expected: &Answer,
    found: &Answer,
    rule: Rule,
    what_changed: &str,
) -> Result<(), TestCaseError> {
    comparison::same_answer(expected, found, rule, what_changed).map_err(TestCaseError::fail)
}

/// `Run::moved_by`, `Picking::moved_by`, `WhatItSitsOn::moved_by` and
/// `Answer::moved_by` below are this file's own extensions of the types
/// `comparison` declares — added as their own `impl` blocks rather than beside
/// `of` and `written_out` there, because a [`Movement`] is this file's own type
/// and the shared module cannot depend on it (see that module's doc comment).
impl Run {
    fn moved_by(&self, movement: &Movement) -> Run {
        match self {
            Run::Straight => Run::Straight,
            Run::RightRound { forward } => Run::RightRound { forward: *forward },
            Run::PartOf { and, picked_by } => Run::PartOf {
                and: and.moved_by(movement),
                picked_by: picked_by
                    .iter()
                    .map(|picking| picking.moved_by(movement))
                    .collect(),
            },
            Run::RightRoundWhereItCrosses { and, forward } => Run::RightRoundWhereItCrosses {
                and: and.moved_by(movement),
                forward: *forward,
            },
        }
    }
}

impl Picking {
    /// `n · p ≥ d` moved is `n' · p ≥ d + n' · t`, and the rest move as the
    /// surfaces they are.
    fn moved_by(&self, movement: &Movement) -> Picking {
        match self {
            Picking::InFrontOf([a, b, c, d]) => {
                let [a, b, c] = movement.direction([*a, *b, *c]);
                let along = a * movement.shift[0] + b * movement.shift[1] + c * movement.shift[2];
                Picking::InFrontOf([a, b, c, d + along])
            }
            Picking::Outside(surface) => Picking::Outside(surface.moved_by(movement)),
            Picking::InsideAPipe(surface) => Picking::InsideAPipe(surface.moved_by(movement)),
        }
    }
}

impl WhatItSitsOn {
    /// Where the surface ends up after an exactly representable movement: the
    /// points that name it move, the directions turn, and every scalar — a
    /// radius, a slope, a tube's thickness — is carried over, because a rigid
    /// movement does not change a length.
    fn moved_by(&self, movement: &Movement) -> WhatItSitsOn {
        match *self {
            WhatItSitsOn::Flat(coefficients) => WhatItSitsOn::Flat(movement.plane(coefficients)),
            WhatItSitsOn::Ball(centre, radius) => {
                WhatItSitsOn::Ball(movement.point(centre), radius)
            }
            WhatItSitsOn::Pipe(origin, way, radius) => {
                WhatItSitsOn::Pipe(movement.point(origin), movement.direction(way), radius)
            }
            WhatItSitsOn::Cone(apex, way, slope) => {
                WhatItSitsOn::Cone(movement.point(apex), movement.direction(way), slope)
            }
            WhatItSitsOn::Ring(centre, way, big, small) => {
                WhatItSitsOn::Ring(movement.point(centre), movement.direction(way), big, small)
            }
        }
    }
}

impl Answer {
    /// The same answer after an exactly representable movement.
    ///
    /// A [`Movement`] is a quarter turn about `z` and a whole-number shift, and
    /// every surface here moves under it by the same two steps `Surface::transformed`
    /// takes in `rcad-geom`: the points that name it move, the directions that
    /// name it turn, and every scalar — a radius, a slope, a tube's thickness —
    /// is carried over untouched because a rigid movement does not change a
    /// length. A quarter turn about `z` is a swap and a negation, so all of that
    /// is exact.
    fn moved_by(&self, movement: &Movement) -> Answer {
        Answer {
            solids: self
                .solids
                .iter()
                .map(|faces| {
                    faces
                        .iter()
                        .map(|face| Face {
                            sits: face.sits.moved_by(movement),
                            reversed: face.reversed,
                            wires: face
                                .wires
                                .iter()
                                .map(|wire| {
                                    wire.iter()
                                        .map(|step| Step {
                                            at: movement.point(step.at),
                                            run: step.run.moved_by(movement),
                                        })
                                        .collect()
                                })
                                .collect(),
                        })
                        .collect()
                })
                .collect(),
        }
    }
}

/// §151.5's second question: a face whose hole is handed over as its outline is
/// not the face it was, and the comparison has to say so.
///
/// **Which wire comes first means something in this kernel.** `FaceView::wires`
/// documents the first as the outline and the rest as holes, the fuse's
/// `rings_round` puts the outline first on purpose, and `holds.rs` counts a
/// round wire as `+1` in first place and `-1` anywhere else. For a flat face of
/// three corners or more the winding would give the swap away anyway, because
/// `Model::validate` refuses an outline winding against the normal. **A round
/// wire has one corner and no winding to read**, so for a washer the order is
/// the only place the difference lives — and sorting every wire threw it away.
#[test]
fn a_washer_whose_hole_is_handed_over_as_its_outline_is_another_answer() {
    let washer = |wires: Vec<Vec<Step>>| Answer {
        solids: vec![vec![Face {
            sits: WhatItSitsOn::Flat([0.0, 0.0, 1.0, 0.0]),
            reversed: false,
            wires,
        }]],
    };
    let rim = vec![round([2.0, 0.0, 0.0], true)];
    let hole = vec![round([1.0, 0.0, 0.0], false)];
    assert_ne!(
        washer(vec![rim.clone(), hole.clone()]).canonical(),
        washer(vec![hole, rim]).canonical(),
        "a washer with its rim and its hole swapped wrote out as the same face"
    );
}

/// **A ring of two corners walked the other way is another ring**, now that
/// each step carries its run.
///
/// With corners alone it was not: turning a ring to its smallest rotation keeps
/// which corner follows which, and with two corners the ring walked backwards
/// *is* a rotation of itself (§163.4). Walked backwards, the arc that left the
/// first corner leaves the second instead, and that is what is written down now
/// (§165).
#[test]
fn a_ring_of_two_corners_walked_the_other_way_is_another_ring() {
    let (one, other) = ([0.0, 0.0, 1.0], [0.0, 0.0, -1.0]);
    let arc_facing = |x: f64, y: f64| Run::PartOf {
        and: WhatItSitsOn::Ball([0.0, 0.0, 0.0], 1.0),
        picked_by: vec![Picking::InFrontOf([x, y, 0.0, 0.0])],
    };
    let step = |at, run| Step { at, run };
    assert_ne!(
        turned_to_its_smallest(&[
            step(one, arc_facing(1.0, 0.0)),
            step(other, arc_facing(0.0, 1.0))
        ]),
        turned_to_its_smallest(&[
            step(one, arc_facing(0.0, 1.0)),
            step(other, arc_facing(1.0, 0.0))
        ]),
        "a ring of two arcs wrote out the same walked either way"
    );
}

/// §163.4's open question, answered: **two whole valid answers that differ can
/// write out the same when the edges' runs are left out.**
///
/// A quarter of a ball and three quarters of it, both cut by the planes `x = 0`
/// and `y = 0`. The quarter is the ball where `x ≥ 0` **and** `y ≥ 0`; the
/// three quarters are where `x ≥ 0` **or** `y ≥ 0`. Every face of one has a twin
/// in the other on the same surface, used the same way round:
///
/// - the skin, one sphere face bounded by two half-meridians between the poles;
/// - a half-disc on `x = 0` facing `−x`, because both solids lie on the `x ≥ 0`
///   side of it there;
/// - a half-disc on `y = 0` facing `−y`, for the same reason.
///
/// And every wire is a ring of the same two corners, the poles — so the corners
/// cannot say which way it runs either. What differs is which half-meridian
/// each edge follows: `x = 0` on the `y ≥ 0` side for the quarter and on the
/// `y ≤ 0` side for the three quarters, and the same on `y = 0`. The two solids
/// have no inside in common but the quarter itself and differ by half a ball.
///
/// Built through [`SolidBuilder`] rather than by a fuse, because all four fuses
/// asked to cut the kernel's ball through both its poles refused
/// (`docs/decisions.md` §165.4).
#[test]
fn a_quarter_of_a_ball_and_three_quarters_of_it_are_two_answers() {
    let policy = Policy::certified();
    let mut model = Model::new(rcad_geom::Scale::millimetres(100.0).expect("a working size"));
    let quarter = a_wedge_of_a_ball(&mut model, [[1.0, 0.0], [0.0, 1.0]], &policy);
    let three_quarters = a_wedge_of_a_ball(&mut model, [[0.0, -1.0], [-1.0, 0.0]], &policy);
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("both wedges are sound solids");

    let quarter = answer_of(&model, &[quarter]);
    let three_quarters = answer_of(&model, &[three_quarters]);
    assert_ne!(
        quarter.canonical(),
        three_quarters.canonical(),
        "a quarter of a ball and three quarters of it wrote out as the same answer"
    );
}

/// The part of a unit ball between two half-planes that hang off the `z` axis,
/// swept anticlockwise seen from above from `between[0]` to `between[1]`.
///
/// Each of the two is named by the way it points away from the axis, and must
/// be one of the four axis directions so that every number here is exact. The
/// sphere face is bounded the way the kernel's own ball is: down one meridian,
/// a pole, up the other, a pole.
fn a_wedge_of_a_ball(model: &mut Model, between: [[f64; 2]; 2], policy: &Policy) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let at = |builder: &mut SolidBuilder, z: f64| {
        builder
            .add_vertex(Point::new(0.0, 0.0, z))
            .expect("a pole is a point")
    };
    let north = at(&mut builder, 1.0);
    let south = at(&mut builder, -1.0);

    // Out of the wedge across the first half-plane is clockwise of it, and out
    // across the second is anticlockwise of it.
    let [first, second] = between;
    let outward = [[first[1], -first[0]], [-second[1], second[0]]];
    let wall = |facing: [f64; 2]| {
        Plane::from_point_and_normal(Point::origin(), Vector::new(facing[0], facing[1], 0.0))
            .expect("an axis direction has length")
    };
    let mut meridians = Vec::with_capacity(2);
    for (way, facing) in between.into_iter().zip(outward) {
        let run = EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(wall(facing)),
            picked_by: smallvec::smallvec![Picker::from(
                HalfSpace::through_a_point_facing(
                    Point::origin(),
                    Vector::new(way[0], way[1], 0.0)
                )
                .expect("an axis direction has length")
            )],
        };
        meridians.push(
            builder
                .add_edge(north, south, run)
                .expect("a half-meridian between the poles"),
        );
    }
    let axis = builder
        .add_edge(north, south, EdgeRun::Straight)
        .expect("the axis between the poles");

    let along = |builder: &mut SolidBuilder, edge, orientation| {
        DraftStep::Along(
            builder
                .add_coedge(edge, orientation)
                .expect("an edge of this builder"),
        )
    };
    let skin_steps = [
        along(&mut builder, meridians[0], Orientation::Forward),
        DraftStep::Pole(south),
        along(&mut builder, meridians[1], Orientation::Reversed),
        DraftStep::Pole(north),
    ];
    let first_wall_steps = [
        along(&mut builder, meridians[0], Orientation::Reversed),
        along(&mut builder, axis, Orientation::Forward),
    ];
    let second_wall_steps = [
        along(&mut builder, meridians[1], Orientation::Forward),
        along(&mut builder, axis, Orientation::Reversed),
    ];
    let mut faces = Vec::with_capacity(3);
    for (surface, steps) in [
        (
            Surface::Sphere(Sphere::new(Point::origin(), 1.0).expect("a unit ball")),
            &skin_steps[..],
        ),
        (Surface::Plane(wall(outward[0])), &first_wall_steps[..]),
        (Surface::Plane(wall(outward[1])), &second_wall_steps[..]),
    ] {
        let wire = builder.add_wire(steps).expect("a closed ring of steps");
        faces.push((
            builder
                .add_face(surface, &[wire])
                .expect("a face with one wire"),
            Orientation::Forward,
        ));
    }
    builder.add_shell(&faces).expect("three faces close up");
    builder
        .build(model, policy, &mut ProvenanceLog::new())
        .expect("a wedge of a ball is a solid")
}

/// The other half of §165: **which way a round wire is walked is the whole of
/// what tells two balls' overlap from the two balls together.**
///
/// Two balls of radius 5, one about the origin and one six up, cross in a
/// circle of radius 4 at `z = 3`, with a corner at `(4, 0, 3)`. The lens where
/// they overlap and the solid they make together are both two sphere faces
/// joined along that one closed edge, each face used forwards with one wire of
/// one corner. The only thing that differs is which way the circle is walked:
/// a face keeps its material on its left, so the small cap of each ball is the
/// circle walked one way and the big cap is the circle walked the other.
///
/// The corners cannot see that, and until the runs were written down nothing
/// else in the comparison could either. The measured volumes say the two are
/// the lens and the union, and not a guess about which way round is which.
#[test]
fn where_two_balls_overlap_and_the_two_balls_together_are_two_answers() {
    let policy = Policy::certified();
    let mut model = Model::new(rcad_geom::Scale::millimetres(100.0).expect("a working size"));
    let overlap = two_balls_joined_along_one_circle(&mut model, Orientation::Forward, &policy);
    let together = two_balls_joined_along_one_circle(&mut model, Orientation::Reversed, &policy);
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("both are sound solids");

    let cap = |height: f64| core::f64::consts::PI * height * height * (15.0 - height) / 3.0;
    for (solid, space) in [(overlap, 2.0 * cap(2.0)), (together, 2.0 * cap(8.0))] {
        let measured = model
            .solid(solid)
            .expect("this model made the key")
            .measure(a_fine_deflection(), &policy, &mut ProvenanceLog::new())
            .expect("two caps measure");
        assert!(
            (measured.volume() - space).abs() <= measured.volume_error(),
            "expected {space}, measured {} give or take {}",
            measured.volume(),
            measured.volume_error()
        );
    }

    assert_ne!(
        answer_of(&model, &[overlap]).canonical(),
        answer_of(&model, &[together]).canonical(),
        "the overlap of two balls and the two balls together wrote out as the same answer"
    );
}

/// The pickers of a run are a set: a piece is on or inside every one of them,
/// and which was listed first says nothing about the piece.
#[test]
fn a_run_picked_by_the_same_two_half_spaces_in_either_order_is_one_run() {
    let picked_by = |first: [f64; 4], second: [f64; 4]| Run::PartOf {
        and: WhatItSitsOn::Ball([0.0, 0.0, 0.0], 1.0),
        picked_by: vec![Picking::InFrontOf(first), Picking::InFrontOf(second)],
    };
    let (facing_x, facing_y) = ([1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0]);
    assert_eq!(
        picked_by(facing_x, facing_y).written_out(),
        picked_by(facing_y, facing_x).written_out()
    );
}

/// Two balls of radius 5 about the origin and `(0, 0, 6)`, joined along the
/// circle where they cross, with the first ball's face walking that circle
/// `first` and the second's the other way.
fn two_balls_joined_along_one_circle(
    model: &mut Model,
    first: Orientation,
    policy: &Policy,
) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let corner = builder
        .add_vertex(Point::new(4.0, 0.0, 3.0))
        .expect("a point on both balls");
    let circle = builder
        .add_edge(corner, corner, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the circle where the two balls cross");
    let second = match first {
        Orientation::Forward => Orientation::Reversed,
        Orientation::Reversed => Orientation::Forward,
    };
    let mut faces = Vec::with_capacity(2);
    for (centre, turn) in [
        (Point::origin(), first),
        (Point::new(0.0, 0.0, 6.0), second),
    ] {
        let coedge = builder
            .add_coedge(circle, turn)
            .expect("an edge of this builder");
        let wire = builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("one closed edge is a ring");
        let ball = Sphere::new(centre, 5.0).expect("a positive radius");
        faces.push((
            builder
                .add_face(Surface::Sphere(ball), &[wire])
                .expect("a cap with one wire"),
            Orientation::Forward,
        ));
    }
    builder.add_shell(&faces).expect("two caps close up");
    builder
        .build(model, policy, &mut ProvenanceLog::new())
        .expect("two caps joined along a circle are a solid")
}

/// Fine enough that a measured volume tells a lens from a union by a wide
/// margin; `a_deflection` is for polyhedra, which ignore it.
fn a_fine_deflection() -> Deflection {
    Deflection::of(0.01).expect("a hundredth of a millimetre is a length")
}

/// §151.5's first question, over answers the kernel really gives: can the
/// order things landed in the arena make one answer write out as two?
///
/// Every solid's faces, every face's holes and the solids themselves are dealt
/// out in a new order, and every ring starts from a new corner. The outline
/// stays first, because its place is part of the answer (see
/// [`a_washer_whose_hole_is_handed_over_as_its_outline_is_another_answer`]).
#[test]
fn dealing_an_answer_out_in_another_order_does_not_change_it() {
    let tally = over(
        CASES,
        (a_lattice_pair(), any::<u64>()),
        |(configuration, seed), tally| {
            let (model, keys) = configuration.build();
            for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                tally.meaning_to_compare();
                let Some((fused_model, solids)) = fused(
                    tally,
                    Coordinates::AreExactlyRepresentable,
                    &model,
                    &[keys[0]],
                    &[keys[1]],
                    rule,
                )?
                .answer() else {
                    continue;
                };

                tally.comparing();
                let answer = answer_of(&fused_model, &solids);
                same_answer(
                    &answer,
                    &dealt_out_again(&answer, seed),
                    rule,
                    "dealing the answer out in another order",
                )?;
            }
            Ok(())
        },
    );
    tally.report_what_it_compared("arena order, lattice pairs");
}

/// The same at one answer built by hand, so that every level is certain to be
/// reached: two solids, three faces in one of them, a face with two holes, and
/// rings of four corners.
///
/// A generated lattice pair only sometimes has two solids and almost never a
/// face with two holes, so the property above can pass with a sort missing.
/// This one cannot: taking out any one of the three sorts, or the turning of a
/// ring, fails it (§163.3).
#[test]
fn every_order_of_one_answer_with_two_holes_in_a_face_writes_out_the_same() {
    let answer = two_solids_one_with_a_face_holding_two_holes();
    let written = answer.canonical();
    for seed in 0..256 {
        assert_eq!(
            dealt_out_again(&answer, seed).canonical(),
            written,
            "dealing the answer out from seed {seed} changed how it wrote out"
        );
    }
}

/// Only data: the comparison reads numbers, so these faces need not close up
/// into a solid, and they do not.
fn two_solids_one_with_a_face_holding_two_holes() -> Answer {
    let square = |x: f64, y: f64, side: f64| {
        straight(&[
            [x, y, 0.0],
            [x + side, y, 0.0],
            [x + side, y + side, 0.0],
            [x, y + side, 0.0],
        ])
    };
    let backwards = |mut ring: Vec<Step>| {
        ring.reverse();
        ring
    };
    Answer {
        solids: vec![
            vec![
                Face {
                    sits: WhatItSitsOn::Flat([0.0, 0.0, 1.0, 0.0]),
                    reversed: false,
                    wires: vec![
                        square(0.0, 0.0, 9.0),
                        backwards(square(1.0, 1.0, 2.0)),
                        backwards(square(5.0, 5.0, 2.0)),
                    ],
                },
                Face {
                    sits: WhatItSitsOn::Flat([1.0, 0.0, 0.0, -9.0]),
                    reversed: false,
                    wires: vec![straight(&[
                        [9.0, 0.0, 0.0],
                        [9.0, 9.0, 0.0],
                        [9.0, 9.0, 1.0],
                        [9.0, 0.0, 1.0],
                    ])],
                },
                Face {
                    sits: WhatItSitsOn::Ball([4.0, 4.0, 4.0], 1.0),
                    reversed: true,
                    wires: vec![vec![round([4.0, 4.0, 5.0], false)]],
                },
            ],
            vec![
                Face {
                    sits: WhatItSitsOn::Flat([0.0, 0.0, -1.0, 0.0]),
                    reversed: false,
                    wires: vec![backwards(square(20.0, 0.0, 1.0))],
                },
                Face {
                    sits: WhatItSitsOn::Flat([0.0, 1.0, 0.0, 0.0]),
                    reversed: false,
                    wires: vec![straight(&[
                        [20.0, 0.0, 0.0],
                        [21.0, 0.0, 0.0],
                        [21.0, 0.0, 1.0],
                    ])],
                },
            ],
        ],
    }
}

/// A ring of straight edges through `corners`, in order.
fn straight(corners: &[[f64; 3]]) -> Vec<Step> {
    corners
        .iter()
        .map(|&at| Step {
            at,
            run: Run::Straight,
        })
        .collect()
}

/// A round wire: one closed edge from `at` back to itself, walked the way it is
/// stored running or against it.
fn round(at: [f64; 3], forward: bool) -> Step {
    Step {
        at,
        run: Run::RightRound { forward },
    }
}

/// The same answer as an arena might have handed it over: every list whose
/// order is arbitrary put in another order, and every ring started from another
/// corner. `seed` picks which.
fn dealt_out_again(answer: &Answer, seed: u64) -> Answer {
    let mut dice = Dice(seed);
    let mut solids: Vec<Vec<Face>> = answer
        .solids
        .iter()
        .map(|faces| {
            let mut faces: Vec<Face> = faces
                .iter()
                .map(|face| {
                    let mut wires: Vec<Vec<Step>> = face
                        .wires
                        .iter()
                        .map(|ring| {
                            let mut ring = ring.clone();
                            let start = dice.below(ring.len());
                            ring.rotate_left(start);
                            ring
                        })
                        .collect();
                    dice.deal(&mut wires[1..]);
                    Face {
                        wires,
                        ..face.clone()
                    }
                })
                .collect();
            dice.deal(&mut faces);
            faces
        })
        .collect();
    dice.deal(&mut solids);
    Answer { solids }
}

/// SplitMix64, which is all a shuffle here needs: an order no arena chose, the
/// same one every time for the same seed.
struct Dice(u64);

impl Dice {
    fn deal<T>(&mut self, items: &mut [T]) {
        for last in (1..items.len()).rev() {
            let pick = self.below(last + 1);
            items.swap(last, pick);
        }
    }

    /// # Panics
    ///
    /// If `count` is nought, since there is nothing below it to pick.
    fn below(&mut self, count: usize) -> usize {
        let count = u64::try_from(count).expect("a list length fits in 64 bits");
        usize::try_from(self.roll() % count).expect("below a list length fits in a usize")
    }

    fn roll(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// One ring, written down from two of its own corners, has to come back as one
/// list.
///
/// The ring below goes through the origin twice. A union makes such a ring
/// where one solid covers the corner of a frame and leaves its top face a C
/// whose ends touch, upright or turned:
/// `the_turned_pair_whose_union_lost_a_face_one_way_round.rs` pins one.
#[test]
fn a_ring_that_goes_through_one_corner_twice_reads_the_same_from_either_side() {
    let here = [0.0, 0.0, 0.0];
    let ring = straight(&[here, [1.0, 0.0, 0.0], here, [0.0, 1.0, 0.0]]);
    let from_the_other_side = straight(&[here, [0.0, 1.0, 0.0], here, [1.0, 0.0, 0.0]]);
    assert_eq!(
        turned_to_its_smallest(&ring),
        turned_to_its_smallest(&from_the_other_side),
        "two walks that reach the repeated corner from different sides wrote \
         the same ring down as two different lists"
    );
}

/// The deflection the measurements here are asked for.
///
/// Every solid in this file is a polyhedron, and a polyhedron ignores it: its
/// own fan is the answer rather than a picture of one (`docs/decisions.md`
/// §25.4).
fn a_deflection() -> Deflection {
    Deflection::of(0.1).expect("a tenth of a millimetre is a length")
}
