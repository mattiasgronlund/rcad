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
//!   normal, and its wires as rings of corners, each ring turned to its own
//!   smallest rotation, the outline kept first and the holes and faces and
//!   solids sorted (§151, §163). Used where the two runs must
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

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::ops::RangeInclusive;

use proptest::prelude::*;
use proptest::test_runner::{
    Config, FileFailurePersistence, RngAlgorithm, TestCaseError, TestRng, TestRunner,
};
use rcad_geom::{Axis, Deflection, Isometry, Orientation, Plane, Point, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

use pairs::{
    BallAndBlock, Configuration, CurvedPair, Elsewhere, FrustumAndBlock, Movement, Placed, Rigid,
    Shape, Skew, Turn, a_ball_a_bar_and_a_block, a_ball_and_a_block, a_chain_of_fuses,
    a_curved_pair, a_flush_pair, a_frustum_and_a_block, a_lattice_pair, a_lattice_trio,
    a_pair_leaning_the_same_way, a_pair_somewhere_else, a_rigid_movement, a_skew_pair,
    a_solid_and_its_copy, a_turned_pair, a_turned_solid_and_its_copy,
    a_turned_solid_and_several_copies,
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
/// seed, so the shipped corpus of 429 turned fuses refuses 96 of them, 22.38%,
/// every run on every machine. There is no sampling noise for the threshold to
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
/// of 429 skew fuses refuses 51 of them, 11.89%, every run on every machine.
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
/// Three tenths, from measurement over this tree, and the number that matters is
/// the *worst* property rather than the average. These properties run from a
/// fresh seed every time, so this floor is the one thing in the file that could
/// go flaky, and it is sized against eight fresh-seed runs of each:
///
/// - the properties over the exactly representable pairs reach 90% to 100%,
/// - the trio properties, which need three fuses to line up, reach 71% to 79%,
/// - commutativity over turned pairs reaches 58% to 78%, because a quarter of
///   turned fuses refuse and a comparison needs both sides,
/// - the depth-four chain is the floor-setter at 50% to 69%: four solids united
///   in a row give an edge with four faces along it often enough that a sixth of
///   its fuses have no answer to give.
///
/// Three tenths is about five standard deviations under the worst of those. It
/// is far enough not to fire on a hard corpus, and nowhere near far enough to
/// let a property that has stopped comparing anything through: halving the rate
/// of any of them trips it.
const SOME_COMPARISONS_MUST_HAPPEN: f64 = 0.3;

/// The same for the one family that sits under [`SOME_COMPARISONS_MUST_HAPPEN`].
///
/// A frustum standing up through a block used to answer exactly when the cone
/// above its own top cap stayed clear of the block's side walls. Which side of
/// that boundary a drawn case landed on was decided almost entirely by the one
/// slope the case drew, so the share that answers swung with 48 draws of a
/// three-valued number rather than with 126 comparisons, and it swung widely.
///
/// **Eighteen fresh-seed runs came out between 30% and 46%**, and 30% turned up
/// twice. That clears [`SOME_COMPARISONS_MUST_HAPPEN`] by four hundredths of one
/// comparison — a floor a property brushes rather than a guard, and shipping it
/// would have been a flake. A fifth is a **margin of ten points under the worst
/// of the eighteen**, about two of that spread's own standard deviations, and it
/// still fires on anything that halves the rate, which is what a floor is for.
///
/// **The boundary that sized it is gone and the eighteen runs were not redone.**
/// `arrange`'s third `HowItRuns` variant answers the configurations that
/// boundary turned down, so this family compares more than it did and a fifth is
/// slacker than it was. It is left where it was measured rather than moved on a
/// guess: raising it needs the eighteen fresh-seed runs done again, and a floor
/// nobody re-measured is worth less than one that is merely generous.
const A_FRUSTUM_FAMILY_STILL_COMPARES_THIS_MUCH: f64 = 0.2;

/// How many of the eighty-four-configuration grid's comparisons have both sides.
///
/// An exact count rather than a floor, because
/// [`every_frustum_on_the_wider_grid_answers_the_same_whichever_side_comes_first`]
/// walks a grid that is written out and not drawn, so this is the same number
/// every run on every machine. Measured: 168 comparisons meant, **106 made**,
/// out of 336 fuses of which 124 refused — every one of the 124 naming
/// `select.rs`'s *a point inside a piece of a face*, and none of them coming
/// back as something a `Model` cannot hold.
///
/// A drop here is a fuse that stopped answering, which is a regression and not a
/// draw. Re-derive it when it moves; do not relax it to a floor (§19.1).
const THE_WIDER_GRID_COMPARES_THIS_MANY: usize = 106;

/// How much of the lattice control comes back as something a `Model` cannot
/// hold.
///
/// Measured at 3.3% over the shipped corpus — 14 of 429 — and the band is two
/// thirds to three times that, 2.2% to 9.8%, rounded to two figures. **That is
/// the convention all six of these bands are derived to**, and each of the
/// other five says so and shows its own arithmetic. Every one of the 14 is the
/// case §17.7's catalogue records by hand: two solids meeting along a whole
/// edge would put four faces along it, and an edge belongs to exactly two.
///
/// The band is a tripwire, not a confidence interval — there is no sampling
/// noise to cover, because [`measured`] runs from a fixed seed and gives the
/// same 429 fuses on every machine. Two thirds to three times is a judgement
/// about how far a real change may move one of these counts before somebody
/// should look at it. In fuses, this one fires under 10 and over 42.
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
/// three more fuses. Nothing since has moved any of the six.
const A_LATTICE_PAIR_IS_NOT_A_SOLID: RangeInclusive<f64> = 0.022..=0.098;

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
const A_PAIR_ELSEWHERE_IS_NOT_A_SOLID: RangeInclusive<f64> = 0.019..=0.084;

/// The same for the turned family.
///
/// Measured at 3.3% — 14 of 429 — which is the lattice family's count exactly,
/// and this family is a rotation of that one. It used to be the lower of the two
/// and no longer is, even though getting on for a quarter of this family's fuses
/// still refuse before they get far enough to find out whether the answer is a
/// solid. Do not read the equality as anything: this is the one of the six that
/// the two seeds after `b4890f1` also moved, 13 on 423 and 10 on 426 before it
/// landed on 14 on 429, so it went past the lattice family's count rather than
/// settling on it. [`A_LATTICE_PAIR_IS_NOT_A_SOLID`] has the attribution.
///
/// Same count as the lattice family, so the convention gives the same band:
/// 2.2% to 9.8%, 10 fuses to 42. It was 1.5% to 9%, a floor at under half the
/// measurement and the loosest of the six. This family has been at 17 of 423,
/// 13 of 423, 10 of 426 and now 14 of 429, and the old floor left room for all
/// of them and half again.
const A_TURNED_PAIR_IS_NOT_A_SOLID: RangeInclusive<f64> = 0.022..=0.098;

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
/// So the ends are one fuse to twelve — `0.23%` to `2.8%` — the observed spread
/// with the ceiling at twice its maximum. The floor is one fuse because one
/// fuse is the question this row actually asks: whether the family still
/// reaches the four-faces-along-one-edge configuration it was built to reach.
/// Zero is the answer that fires.
///
/// **The sentence this replaces was true and still misled.** It said a band
/// this tight is not a flake waiting to happen, because [`measured`] runs from
/// a fixed seed and a count of three is a count of three on every machine. That
/// is right about re-running and wrong about safety: a deterministic draw from
/// a family whose counts run one to six moves anywhere in that range whenever
/// the corpus is reshuffled, and §121, §130, §131 and §132 reshuffled it. The
/// row then failed for six weeks' worth of arithmetic work having changed the
/// draw, not the kernel's behaviour. §136.5.
const A_SKEW_PAIR_IS_NOT_A_SOLID: RangeInclusive<f64> = 0.0023..=0.028;

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

#[test]
fn swapping_the_two_sides_of_a_turned_pair_changes_not_one_bit() {
    let tally = over(CASES, a_turned_pair(), |configuration, tally| {
        both_ways_round(tally, Coordinates::AreUnderAGeneralRotation, configuration)
    });
    tally.report_what_it_compared("commutativity, turned pairs");
}

/// §48.2 item 5's defect, at the first configuration that found it, **and it is
/// wrong**.
///
/// A frame and a comb, turned together by one sixty-fourth of a turn about
/// `(-2, -1, 0)`. What the two share is two solids, and whether the kernel can
/// hold them depends on which of the pair is handed over first: comb against
/// frame gives the two, frame against comb gives
/// [`Error::ResultIsNotManifold`]. Whether four faces end up along one edge is a
/// fact about the answer, and the answer does not know which argument came
/// first.
///
/// **The same pair unturned gives the two solids either way round**, which is
/// the control: the two are in the same places relative to each other, and the
/// only thing between the two rows is that the rotation leaves nothing exactly
/// representable.
///
/// This is pinned rather than fixed, and what is left of it is now named.
/// **Two** things split one corner of this pair in two, and only one of them is
/// still here:
///
/// - The two solids are flush on a plane and name it opposite ways round, so a
///   corner on it was solved from `P, Q, R` by one face and from `−P, Q, R` by
///   another. That is gone: three planes are named whichever way each of them
///   faces now (§48, W5, `assemble::names_of_the_three_surfaces`), which is what
///   brings the second row below back to the two solids.
/// - The two solids also have an input **vertex** each at one shared corner —
///   `11v1` and `23v1` — and the rotation lands the two a last bit apart, so
///   neither the names nor `assemble::exactly_at`'s coordinates join them. That
///   one is roadmap §7.4's vertex-on-face entry, it needs no fuse to reach, and
///   it is what the first row below is.
///
/// **The day that is fixed this test fails**, and what replaces it is deleting
/// [`both_agree_about_whether_there_is_an_answer`]'s exemption for this family.
#[test]
fn turning_a_pair_makes_what_the_two_share_depend_on_which_one_comes_first() {
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
    assert_eq!(
        what_a_fuse_came_back_with(&model, keys[0], keys[1], Rule::Intersection),
        WhatCameBack::NotASolid,
        "pinning what happens today, and it is wrong"
    );
    assert_eq!(
        what_a_fuse_came_back_with(&model, keys[1], keys[0], Rule::Intersection),
        WhatCameBack::Solids(2),
        "the same two solids, handed over the other way round"
    );
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
    tally.report_what_it_compared("fuse([A,B,C,D]) = uniting them one at a time, lattice chains");
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
    tally.share_of_answers_that_were_not_solids_must_be_within(A_LATTICE_PAIR_IS_NOT_A_SOLID);
    assert_eq!(
        tally.refused.get(),
        0,
        "a lattice-aligned fuse that refuses is a bug in our filters, not a fact about geometry"
    );
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
    tally.share_of_answers_that_were_not_solids_must_be_within(A_FLUSH_PAIR_IS_NOT_A_SOLID);
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
    tally.share_of_answers_that_were_not_solids_must_be_within(A_PAIR_ELSEWHERE_IS_NOT_A_SOLID);
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
/// asked for and the figure written into §17.7 reproduces; the properties above
/// run on a fresh seed every time, because their job is to find inputs nobody
/// has thought of and a frozen seed would stop them finding any.
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
#[test]
fn how_often_a_certified_fuse_of_a_turned_pair_refuses() {
    let tally = measured(a_turned_pair(), |configuration, tally| {
        every_rule(tally, Coordinates::AreUnderAGeneralRotation, configuration)
    });
    tally.report("§17.7 headline: turned pairs, every rule");
    tally.share_of_answers_that_were_not_solids_must_be_within(A_TURNED_PAIR_IS_NOT_A_SOLID);

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
/// naming a third. This family's leading cause is `"order along a crossing"`, which
/// that corpus essentially never reaches — and reaching it is the point. Three
/// cut lines through one point put two intersections at the same place on a
/// line, and an ordering that is a tie is not a comparison any amount of
/// precision settles.
#[test]
fn how_often_a_certified_fuse_of_a_skew_pair_refuses() {
    let tally = measured(a_skew_pair(), |configuration, tally| {
        every_rule(tally, Coordinates::SitOnTiltedWalls, configuration)
    });
    tally.report("§22 headline: skew pairs, every rule");
    tally.share_of_answers_that_were_not_solids_must_be_within(A_SKEW_PAIR_IS_NOT_A_SOLID);

    let refused = tally.refused.get() as f64 / tally.fuses.get() as f64;
    if this_is_the_shipped_corpus() {
        assert!(
            refused < SKEW_REFUSALS_MUST_STAY_BELOW,
            "{:.1}% of {} skew fuses refused, over the {:.0}% §22 records",
            refused * 100.0,
            tally.fuses.get(),
            SKEW_REFUSALS_MUST_STAY_BELOW * 100.0
        );
        tally.the_cause_named_most_often_is(ORDER_ALONG_A_LINE);
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
    tally.share_of_answers_that_were_not_solids_must_be_within(A_PAIR_LEANING_ALIKE_IS_NOT_A_SOLID);
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

/// §31.7 criterion 15's second name, and **it cannot be met today**.
///
/// A bar with a bit-identical copy of itself taken away should leave nothing at
/// all, which is what the same question gives for every flat family in this
/// file. What happens instead is a refusal: `Undecidable`, naming a point inside
/// a piece of a face, at the bar's own seam corner. Two coincident curved faces
/// are §36.16's row, and two copies of one bar is the plainest shape that
/// reaches it.
///
/// So this test asserts the refusal **by name** rather than reporting a green
/// run over a property that compared nothing — which is the shape §14.7 asks for
/// when a family stops reaching its subject. The control is the same question
/// with the bar replaced by a block, which answers and leaves nothing, so the
/// difference between the two rows is the curve and nothing else.
///
/// The day the refusal is closed this test fails, and what replaces it is the
/// assertion the criterion actually asks for.
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

    let (mut model, keys) = pair.built_as_a_bar_and_a_copy_of_it();
    match model.fuse(
        &[keys[0]],
        &[keys[1]],
        Rule::Difference,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) {
        Err(Error::Undecidable { records }) => assert_eq!(
            records
                .records()
                .last()
                .expect("a refusal names a decision")
                .predicate(),
            "a point inside a piece of a face",
            "pinning what happens today: `A \\ A` over a bar is refused, not answered"
        ),
        Ok(fused) => panic!(
            "the refusal is closed — assert the criterion instead, got {} solids",
            fused.solids().len()
        ),
        Err(why) => panic!("`A \\ A` over a bar refused with {why}"),
    }
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
/// **This is what makes the row above a statement about a pipe.** The bar
/// refuses `A \ A` on [`A_POINT_INSIDE_A_PIECE_OF_A_FACE`] and the ball does
/// not, so §36.16's row is not *"two coincident curved faces"* — it is two
/// coincident faces on a surface that closes round a seam, and a ball's skin has
/// no seam a difference has to walk along. Which of those two the fix is aimed
/// at is the difference between one surface's problem and every surface's, and
/// nothing in the tree had asked.
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

/// Absorption, over a round solid: `A ∪ (A ∩ B)` is `A` — **and it cannot run
/// today**.
///
/// The first fuse answers every time. The second is handed half a ball whose
/// flat face lies exactly in the block's floor and whose skin lies exactly on a
/// piece of the answer's own boundary, and that is §36.16's coincident-surface
/// row: it refuses, naming [`A_POINT_INSIDE_A_PIECE_OF_A_FACE`], on **61 of 61**
/// of the shipped corpus.
///
/// So this pins the refusal over the whole family rather than reporting a green
/// run over a property that compared nothing (§14.7) — see
/// [`Tally::nothing_was_compared_because_it_all_refused`]. The control is the
/// same law over the lattice family, which
/// [`a_solid_united_with_what_it_shares_with_another_is_itself`] asserts for
/// real, so the difference between the two rows is the curve and nothing else.
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
    tally.nothing_was_compared_because_it_all_refused(
        "absorption over a ball, which refuses",
        A_POINT_INSIDE_A_PIECE_OF_A_FACE,
    );
}

/// `A \ (A \ B)` is `A ∩ B`, over a round solid — **and it cannot run today**
/// either, for the same reason.
///
/// Two ways to the same half ball: cut the dent and then cut the dent's
/// complement back out of the block, against sharing the block with the ball
/// once. The second cut is handed the dent, whose skin is the ball's own, so it
/// refuses on [`A_POINT_INSIDE_A_PIECE_OF_A_FACE`] on 61 of 61. Pinned, with
/// [`taking_away_what_is_left_after_taking_the_other_away_leaves_what_the_two_share`]
/// as the flat control.
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
    tally.nothing_was_compared_because_it_all_refused(
        "A \\ (A \\ B) = A ∩ B over a ball, which refuses",
        A_POINT_INSIDE_A_PIECE_OF_A_FACE,
    );
}

/// `((A ∪ B) \ B) ∪ B` is `A ∪ B`, over a round solid — **and it stops at a
/// different refusal from the other two**, which is why it is worth having
/// separately.
///
/// The deepest chain any curved family has been asked for. The middle fuse takes
/// the ball back out of the answer the first one built, and what stops it is not
/// two coincident faces but [`WHICH_SIDE_OF_ITS_TRIMMING_A_FACE_LIES_ON`] — the
/// dome the union left is a face whose every corner sits on the ball's skin, so
/// nothing in the model says which side of it the face is on. §30.8 item 10, on
/// 61 of 61.
///
/// Two laws blocked by one refusal and a third blocked by another is a sharper
/// statement than *"the laws do not run over a ball"*, and it says the ball
/// needs both closed and not either.
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
    tally.nothing_was_compared_because_it_all_refused(
        "((A ∪ B) \\ B) ∪ B = A ∪ B over a ball, which refuses",
        WHICH_SIDE_OF_ITS_TRIMMING_A_FACE_LIES_ON,
    );
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
/// **Compared with [`invariants_of`] and not with [`answer_of`]**, because the
/// two groupings may legitimately cut the answer into different faces —
/// roadmap §11.2's point, and the reason this file picks between the two
/// comparisons per property rather than once.
///
/// # It does not run today, and what stops it is not the trio
///
/// **One of the four fuses this property runs per case refuses**, and that is
/// enough to leave it comparing nothing, so this pins the refusal rather than
/// reporting a green run over a property that compared nothing (§14.7).
/// Measured: **39 of 156, exactly a quarter**, all naming
/// [`CANNOT_TELL_THE_CELLS_OF_A_FACE_APART`]. That quartering is asserted below
/// and not merely printed.
///
/// The one that stops is the **second** fuse of the grouping that sinks the ball
/// into the block first. Its floor then has a round *hole* in it, so one of that
/// face's rim wires is a single closed edge; `how_often_the_rim_winds_round` fans
/// a rim wire's corners into a polygon, a one-corner wire fans into the segment
/// from that corner to itself, and no turn off it is ever `Pos` or `Neg`. The
/// bar's closed-cut lump then gets no winding at all, and a walk wound zero is
/// joined to a walk whose winding is unknown. `a_ball_and_a_bar_in_one_block.rs`
/// carries the measured cycle table.
///
/// # What this pin used to say, and why it changed
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
/// The old note that the obvious way round is itself refused still holds: putting
/// the ball in a *side* wall would leave each plane one round crossing, but a
/// vertical plane through a ball cuts both of its poles —
/// `degenerate_curved_fuses.rs`'s `a_ball_cut_by_a_plane_down_its_polar_axis_is_refused_by_name`.
#[test]
fn uniting_a_ball_and_a_bar_with_a_block_does_not_depend_on_which_two_go_first() {
    let coordinates = Coordinates::LieOnARoundWallAndMayCoincide;
    let tally = over(LONGER_CASES, a_ball_a_bar_and_a_block(), |trio, tally| {
        let (model, keys) = trio.build();
        let (block, ball, bar) = (keys[0], keys[1], keys[2]);
        tally.meaning_to_compare();

        // Both groupings are asked for, every case, and neither short-circuits
        // the other. What this pins is that *each* of them stops, so a run that
        // only ever tried the first would be pinning half the claim.
        let left = united_in_turn(tally, coordinates, &model, &[block], &[ball], &[bar])?;
        let right = united_in_turn(tally, coordinates, &model, &[ball], &[bar], &[block])?;
        let (Some(left), Some(right)) = (left, right) else {
            return Ok(());
        };

        tally.comparing();
        prop_assert_eq!(
            invariants_of(&left.0, &left.1),
            invariants_of(&right.0, &right.1),
            "uniting a block, a ball and a bar does not depend on which two go first"
        );
        Ok(())
    });
    tally.nothing_was_compared_because_it_all_refused(
        "associativity of union over a ball and a bar in a block, which refuses",
        CANNOT_TELL_THE_CELLS_OF_A_FACE_APART,
    );
    assert_eq!(
        4 * tally.refused.get(),
        tally.fuses.get(),
        "the pin is that exactly one of the four fuses this property runs per case refuses — \
         the second of the grouping that sinks the ball into the block first, which leaves a \
         round hole in the floor for the bar to be cut against: {} of {} refused",
        tally.refused.get(),
        tally.fuses.get()
    );
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
/// brief said so, `how_often_a_fuse_with_a_cone_refuses.rs` measures its own
/// frustum family at 6 answers out of 96, and §47.2 says all six of those are
/// the row where the block misses the frustum altogether.
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
        A_FRUSTUM_FAMILY_STILL_COMPARES_THIS_MUCH,
    );
}

/// The same law over the eighty-four-configuration grid, which the family
/// cannot draw.
///
/// **This grid used to pin a rule, and the rule is dead.** §49.6 published *the
/// fuse answers if and only if the cone, carried up to the block's roof, is
/// still narrower than the distance to the nearest side wall*, and this file
/// asserted it over these eighty-four. `arrange`'s third `HowItRuns` variant
/// puts points in order along a hyperbola now, so fifteen of the forty-four the
/// rule turned down answer instead.
/// `tests/ordering_points_along_an_endless_bend.rs` is where that death is
/// measured — none refused for want of an order, fifty-three answering, and the
/// fifteen the rule was wrong about — and re-asserting any of it here would be a
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

/// The predicate that settles which of two points on a crossing comes first.
///
/// Named here because [`how_often_a_certified_fuse_of_a_skew_pair_refuses`]
/// asserts on it rather than only printing it, and a bare string in an
/// assertion is a thing that goes stale silently.
///
/// It was `"order along a line"` until the arrangement learned to draw a curve
/// that closes, and the same question about a straight crossing is still the
/// same question — the count this family asserts on did not move.
const ORDER_ALONG_A_LINE: &str = "order along a crossing";

/// What the fuse gives up on when it is handed two curved faces lying on the
/// same surface.
///
/// §36.16's row. `a_bar_with_a_copy_of_itself_taken_away_is_nothing_at_all` is
/// the plainest shape that reaches it, and every law that puts an answer back
/// against one of its own inputs reaches it too.
const A_POINT_INSIDE_A_PIECE_OF_A_FACE: &str = "a point inside a piece of a face";

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
/// stops at.
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
    for rule in [Rule::Union, Rule::Intersection] {
        tally.meaning_to_compare();
        let (model, keys) = configuration.build();
        let (one, other) = (keys[0], keys[1]);

        let forwards = fused(tally, coordinates, &model, &[one], &[other], rule)?;
        let backwards = fused(tally, coordinates, &model, &[other], &[one], rule)?;
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

/// Ask all three rules of one configuration and keep nothing but the tally.
fn every_rule(
    tally: &Tally,
    coordinates: Coordinates,
    configuration: Configuration,
) -> Result<(), TestCaseError> {
    let (model, keys) = configuration.build();
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        fused(tally, coordinates, &model, &[keys[0]], &[keys[1]], rule)?;
    }
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
                | Coordinates::LieOnAConesWall => Ok(Fused::Refused),
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
                | Coordinates::LieOnAConesWall => Ok(Fused::NotASolid),
                Coordinates::LieOnARoundWall => Err(TestCaseError::fail(format!(
                    "a {rule:?} over a family that has never handed back four faces along \
                     one edge did, and that is not something this kernel can hold"
                ))),
            }
        }
        Err(why) => match (coordinates, a_curved_refusal(&why)) {
            (
                Coordinates::LieOnARoundWallAndMayCoincide | Coordinates::LieOnAConesWall,
                Some(named),
            ) => {
                tally.refused.set(tally.refused.get() + 1);
                *tally.gave_up.borrow_mut().entry(named).or_default() += 1;
                Ok(Fused::Refused)
            }
            _ => Err(TestCaseError::fail(format!("{rule:?} went wrong: {why}"))),
        },
    }
}

/// The refusals a curved fuse is allowed to hit today, by name.
///
/// Every one is a gap the kernel's own error documentation already owns: §30.8
/// item 10 for the first, §33.1 for the second, §31.2 row 8 for the third, and
/// §47.2 for the last two. Anything else is a bug, so this hands back `None` for
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
/// **It counts on every family and fails on the round one.** Turning it on found
/// the disagreement immediately on the flat families — on the turned pairs, on
/// the skew pairs, and, once the shape was shrunk, on the lattice pairs, where
/// nothing rounds at all. The lattice end of that is fixed and now says so
/// ([`a_lattice_pair_unites_to_the_same_solid_whichever_way_round_it_is_handed_over`]);
/// what is left on the turned end is pinned by hand in
/// [`turning_a_pair_makes_what_the_two_share_depend_on_which_one_comes_first`],
/// which names the one cause still open.
///
/// The fuse is not this lane's to fix (§48.4), and a property that is red for a
/// reason already written down stops catching the reasons that are not. So what
/// the flat families get is a count in their report and a pin with their name
/// on it, and what the curved families get is a failure — they have never
/// handed back an answer no `Model` can hold at all, so a disagreement there
/// would be new.
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
        Coordinates::AreExactlyRepresentable
        | Coordinates::AreUnderAGeneralRotation
        | Coordinates::SitOnTiltedWalls => Ok(()),
        Coordinates::LieOnARoundWall
        | Coordinates::LieOnARoundWallAndMayCoincide
        | Coordinates::LieOnAConesWall => Err(TestCaseError::fail(format!(
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
    /// §48.2 item 5's blind spot, now a number. Zero is the only right value and
    /// it is not zero today — see
    /// [`both_agree_about_whether_there_is_an_answer`] for which families it
    /// fails on and which it only counts on.
    disagreed_about_being_a_solid: Cell<usize>,
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
    /// One family needs it, and needs it because
    /// [`SOME_COMPARISONS_MUST_HAPPEN`] was sized against the properties that
    /// existed when it was written — the worst of which reached 50% to 69% —
    /// and the frustum family sits below all of them. A floor a property brushes
    /// is a flake and not a guard, and moving the shared number to suit one
    /// family would blunt it for the other fourteen.
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

    /// Report, and pin that the property compared **nothing**, because every
    /// second fuse in it refused and named `predicate`.
    ///
    /// The counterpart of [`Tally::report_what_it_compared`] for a law that
    /// cannot run yet. A property that has stopped comparing anything must not
    /// read as a pass (§14.7), and the honest form of that is not a floor it
    /// fails but an assertion that says out loud what today's kernel does — the
    /// shape [`a_bar_with_a_copy_of_itself_taken_away_is_nothing_at_all`]
    /// already uses at one hand-written pair, over a whole family.
    ///
    /// **The day the refusal closes, this fails**, and what replaces it is
    /// [`Tally::report_what_it_compared`] and the law asserted for real.
    fn nothing_was_compared_because_it_all_refused(&self, what: &str, predicate: &str) {
        self.report(what);
        let meant_to = self.meant_to_compare.get();
        assert!(meant_to > 0, "{what} never tried to compare anything");
        assert_eq!(
            self.compared.get(),
            0,
            "{what}: {} of {meant_to} comparisons got both sides, so this law has started \
             running and should be asserted rather than pinned",
            self.compared.get()
        );
        self.the_cause_named_most_often_is(predicate);
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
            "{} of {} fuses came back not a solid ({:.1}%), outside the {:.2}%–{:.2}% \
             this family has always given",
            self.not_a_solid.get(),
            self.fuses.get(),
            100.0 * share,
            100.0 * band.start(),
            100.0 * band.end()
        );
    }
}

/// Run `property` over `cases` configurations from a fresh seed, and hand back
/// what its fuses did.
fn over<T: core::fmt::Debug>(
    cases: u32,
    strategy: impl Strategy<Value = T>,
    property: impl Fn(T, &Tally) -> Result<(), TestCaseError>,
) -> Tally {
    run(TestRunner::new(configured(cases)), strategy, property)
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
/// If the variable is set to something that is not a whole number above zero, or
/// to one whose product is more than a run can count. A soak that silently ran
/// the default corpus would be worse than one that stopped, and a product that
/// wrapped runs **fewer** cases than the shipped corpus while the operator reads
/// the number as the wide one (§48.2, §50.9). Measured: at a factor of
/// 33 554 432 this used to hand proptest a corpus of nought, and the control row
/// went green over the 15 replayed regression seeds — 45 fuses where the shipped
/// corpus is 429.
fn measured_cases() -> u32 {
    let Ok(factor) = std::env::var(SOAK_FACTOR) else {
        return MEASURED_CASES;
    };
    let factor: u32 = factor
        .parse()
        .unwrap_or_else(|_| panic!("{SOAK_FACTOR} has to be a whole number, not {factor:?}"));
    assert!(factor > 0, "{SOAK_FACTOR} has to be at least one");
    MEASURED_CASES.checked_mul(factor).unwrap_or_else(|| {
        panic!("{MEASURED_CASES} cases times {factor} is more than one run can count")
    })
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

/// Insist two answers are the same one, bit for bit.
fn same_answer(
    expected: &Answer,
    found: &Answer,
    rule: Rule,
    what_changed: &str,
) -> Result<(), TestCaseError> {
    if expected.canonical() == found.canonical() {
        return Ok(());
    }

    let mut why = format!("{what_changed} changed the answer to a {rule:?}:\n");
    let _ = writeln!(
        why,
        "  {} solids and {} faces one way, {} solids and {} faces the other",
        expected.solids.len(),
        expected.face_count(),
        found.solids.len(),
        found.face_count()
    );
    for (position, (one, other)) in expected
        .canonical()
        .iter()
        .zip(found.canonical().iter())
        .enumerate()
    {
        for (face, (mine, theirs)) in one.iter().zip(other.iter()).enumerate() {
            if mine != theirs {
                let _ = writeln!(why, "  solid {position} face {face} differs");
                return Err(TestCaseError::fail(why));
            }
        }
    }
    Err(TestCaseError::fail(why))
}

/// The geometry of an answer, and nothing about which slot anything landed in.
#[derive(Clone, Debug)]
struct Answer {
    solids: Vec<Vec<Face>>,
}

/// One face: the surface it sits on, which way round its solid uses it, and its
/// rings of corners.
///
/// **Which way round rather than which way it faces**, because a curved face has
/// no one normal to read. For a flat face the two say the same thing: the
/// outward normal is the stored plane's normal times the orientation's sign, so
/// two faces with the same plane bits agree about the normal exactly when they
/// agree about the orientation. That stays true under a [`Movement`] as well,
/// since a quarter turn is a swap and a negation and multiplying by `±1`
/// commutes with both.
#[derive(Clone, Debug)]
struct Face {
    sits: WhatItSitsOn,
    reversed: bool,
    wires: Vec<Vec<[f64; 3]>>,
}

/// What a face sits on, kept as the numbers that name it.
///
/// Numbers rather than a [`Surface`] because [`Answer::moved_by`] has to move a
/// plane's four coefficients, and because the whole point of the comparison is
/// that nothing about those numbers changed.
#[derive(Clone, Debug)]
enum WhatItSitsOn {
    /// A plane, as its four coefficients.
    Flat([f64; 4]),
    /// A ball's skin: where its middle is and how big it is.
    Ball([f64; 3], f64),
    /// A pipe's skin: a point on its axis, the way that axis points, and how
    /// big it is.
    Pipe([f64; 3], [f64; 3], f64),
    /// A cone's skin: its apex, the way its axis points, and how fast it
    /// widens.
    Cone([f64; 3], [f64; 3], f64),
    /// A ring's skin: its centre, the way its axis points, and its two radii.
    Ring([f64; 3], [f64; 3], f64, f64),
}

impl WhatItSitsOn {
    fn of(surface: &Surface) -> WhatItSitsOn {
        match surface {
            Surface::Plane(flat) => WhatItSitsOn::Flat(flat.coefficients()),
            Surface::Sphere(ball) => WhatItSitsOn::Ball(ball.centre().coordinates(), ball.radius()),
            Surface::Cylinder(pipe) => WhatItSitsOn::Pipe(
                pipe.axis().origin().coordinates(),
                pipe.axis().direction().components(),
                pipe.radius(),
            ),
            Surface::Cone(cone) => WhatItSitsOn::Cone(
                cone.apex().coordinates(),
                cone.axis().direction().components(),
                cone.slope(),
            ),
            Surface::Torus(ring) => WhatItSitsOn::Ring(
                ring.centre().coordinates(),
                ring.axis().direction().components(),
                ring.ring_radius(),
                ring.tube_radius(),
            ),
            other => panic!("this file does not know how to write {other:?} down"),
        }
    }

    /// The numbers, with a tag so two different kinds of surface can never
    /// write themselves out the same way.
    fn written_out(&self) -> Vec<u64> {
        match self {
            WhatItSitsOn::Flat(coefficients) => {
                core::iter::once(0).chain(coefficients.map(bits)).collect()
            }
            WhatItSitsOn::Ball(centre, radius) => core::iter::once(1)
                .chain(centre.map(bits))
                .chain([bits(*radius)])
                .collect(),
            WhatItSitsOn::Pipe(origin, direction, radius) => core::iter::once(2)
                .chain(origin.map(bits))
                .chain(direction.map(bits))
                .chain([bits(*radius)])
                .collect(),
            WhatItSitsOn::Cone(apex, direction, slope) => core::iter::once(3)
                .chain(apex.map(bits))
                .chain(direction.map(bits))
                .chain([bits(*slope)])
                .collect(),
            WhatItSitsOn::Ring(centre, direction, big, small) => core::iter::once(4)
                .chain(centre.map(bits))
                .chain(direction.map(bits))
                .chain([bits(*big), bits(*small)])
                .collect(),
        }
    }
}

fn answer_of(model: &Model, solids: &[SolidKey]) -> Answer {
    Answer {
        solids: solids
            .iter()
            .map(|&solid| {
                model
                    .solid(solid)
                    .expect("this model made the key")
                    .faces()
                    .map(|face| Face {
                        sits: WhatItSitsOn::of(&face.surface()),
                        reversed: face.orientation() == Orientation::Reversed,
                        wires: face
                            .wires()
                            .map(|wire| wire.points().map(|at| at.coordinates()).collect())
                            .collect(),
                    })
                    .collect()
            })
            .collect(),
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
                            sits: match face.sits {
                                WhatItSitsOn::Flat(coefficients) => {
                                    WhatItSitsOn::Flat(movement.plane(coefficients))
                                }
                                WhatItSitsOn::Ball(centre, radius) => {
                                    WhatItSitsOn::Ball(movement.point(centre), radius)
                                }
                                WhatItSitsOn::Pipe(origin, way, radius) => WhatItSitsOn::Pipe(
                                    movement.point(origin),
                                    movement.direction(way),
                                    radius,
                                ),
                                WhatItSitsOn::Cone(apex, way, slope) => WhatItSitsOn::Cone(
                                    movement.point(apex),
                                    movement.direction(way),
                                    slope,
                                ),
                                WhatItSitsOn::Ring(centre, way, big, small) => WhatItSitsOn::Ring(
                                    movement.point(centre),
                                    movement.direction(way),
                                    big,
                                    small,
                                ),
                            },
                            reversed: face.reversed,
                            wires: face
                                .wires
                                .iter()
                                .map(|wire| wire.iter().map(|&at| movement.point(at)).collect())
                                .collect(),
                        })
                        .collect()
                })
                .collect(),
        }
    }

    /// The answer written out in one fixed order, so that two answers agree
    /// exactly when the arena order is the only thing between them.
    ///
    /// A ring of corners is turned to start at its own smallest corner rather
    /// than sorted: which corner comes first is arbitrary, but which corner
    /// follows which is the winding, and the winding is part of the answer.
    fn canonical(&self) -> Vec<Vec<Vec<u64>>> {
        let mut solids: Vec<Vec<Vec<u64>>> = self
            .solids
            .iter()
            .map(|faces| {
                let mut faces: Vec<Vec<u64>> = faces.iter().map(written_out).collect();
                faces.sort_unstable();
                faces
            })
            .collect();
        solids.sort_unstable();
        solids
    }

    fn face_count(&self) -> usize {
        self.solids.iter().map(Vec::len).sum()
    }
}

/// One face as numbers: its surface, which way round it is used, its outline,
/// and then its holes in sorted order.
///
/// **The outline stays first and only the holes are sorted.** Which wire is
/// first is part of the face in this kernel, and a round wire has no winding
/// to carry it instead (§163).
fn written_out(face: &Face) -> Vec<u64> {
    let mut written = face.sits.written_out();
    written.push(u64::from(face.reversed));

    let mut wires = face.wires.iter().map(|wire| turned_to_its_smallest(wire));
    let outline = wires.next().expect("a face has at least one wire");
    let mut holes: Vec<Vec<u64>> = wires.collect();
    holes.sort_unstable();
    written.push(face.wires.len() as u64);
    for wire in core::iter::once(outline).chain(holes) {
        written.push(wire.len() as u64);
        written.extend(wire);
    }
    written
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
    let washer = |wires: Vec<Vec<[f64; 3]>>| Answer {
        solids: vec![vec![Face {
            sits: WhatItSitsOn::Flat([0.0, 0.0, 1.0, 0.0]),
            reversed: false,
            wires,
        }]],
    };
    let rim = vec![[2.0, 0.0, 0.0]];
    let hole = vec![[1.0, 0.0, 0.0]];
    assert_ne!(
        washer(vec![rim.clone(), hole.clone()]).canonical(),
        washer(vec![hole, rim]).canonical(),
        "a washer with its rim and its hole swapped wrote out as the same face"
    );
}

/// What the comparison still forgets, pinned so that it is said rather than
/// found: **a ring of two corners reads the same walked either way.**
///
/// Turning a ring to its smallest rotation keeps which corner follows which,
/// and with two corners the ring walked backwards *is* a rotation of itself.
/// A face bounded by two arcs between the same two corners is such a ring, and
/// so is a one-corner round wire, which has no order at all. The edges' own
/// runs are not written down either. `docs/decisions.md` §163.4 says why this
/// is left open rather than fixed.
#[test]
fn a_ring_of_two_corners_reads_the_same_walked_either_way() {
    let (one, other) = ([1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]);
    assert_eq!(
        turned_to_its_smallest(&[one, other]),
        turned_to_its_smallest(&[other, one])
    );
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
        vec![
            [x, y, 0.0],
            [x + side, y, 0.0],
            [x + side, y + side, 0.0],
            [x, y + side, 0.0],
        ]
    };
    let backwards = |mut ring: Vec<[f64; 3]>| {
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
                    wires: vec![vec![
                        [9.0, 0.0, 0.0],
                        [9.0, 9.0, 0.0],
                        [9.0, 9.0, 1.0],
                        [9.0, 0.0, 1.0],
                    ]],
                },
                Face {
                    sits: WhatItSitsOn::Ball([4.0, 4.0, 4.0], 1.0),
                    reversed: true,
                    wires: vec![vec![[4.0, 4.0, 5.0]]],
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
                    wires: vec![vec![[20.0, 0.0, 0.0], [21.0, 0.0, 0.0], [21.0, 0.0, 1.0]]],
                },
            ],
        ],
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
                    let mut wires: Vec<Vec<[f64; 3]>> = face
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
/// The ring below goes through the origin twice, which is what a turned pair
/// does when both inputs have a corner at the point the turn holds still.
#[test]
fn a_ring_that_goes_through_one_corner_twice_reads_the_same_from_either_side() {
    let here = [0.0, 0.0, 0.0];
    let ring = [here, [1.0, 0.0, 0.0], here, [0.0, 1.0, 0.0]];
    let from_the_other_side = [here, [0.0, 1.0, 0.0], here, [1.0, 0.0, 0.0]];
    assert_eq!(
        turned_to_its_smallest(&ring),
        turned_to_its_smallest(&from_the_other_side),
        "two walks that reach the repeated corner from different sides wrote \
         the same ring down as two different lists"
    );
}

/// A ring written down from whichever of its corners makes the smallest list.
///
/// **The smallest rotation, not the first smallest corner.** Starting at the
/// first corner equal to the smallest one is a canonical form only while no
/// ring visits a corner twice. A face ring that does — and a turned pair makes
/// one, through the point the turn holds still — is written down as its two
/// halves by the two orders, and the comparison above then reports a face that
/// differs when the two answers are the same solid. `docs/decisions.md` §149
/// is the counterexample that cost a session, §151 is this.
fn turned_to_its_smallest(wire: &[[f64; 3]]) -> Vec<u64> {
    let corners: Vec<[u64; 3]> = wire.iter().map(|&at| at.map(bits)).collect();
    (0..corners.len())
        .map(|start| -> Vec<u64> {
            (0..corners.len())
                .flat_map(|step| corners[(start + step) % corners.len()])
                .collect()
        })
        .min()
        .unwrap_or_default()
}

/// A number's bits, with the two zeroes counted as one.
///
/// `-0.0` and `0.0` name the same point and the same plane, and which one an
/// arithmetic path lands on is not part of the answer. Everything else is
/// compared bit for bit.
fn bits(number: f64) -> u64 {
    if number == 0.0 {
        0u64
    } else {
        number.to_bits()
    }
}

/// The deflection the measurements here are asked for.
///
/// Every solid in this file is a polyhedron, and a polyhedron ignores it: its
/// own fan is the answer rather than a picture of one (`docs/decisions.md`
/// §25.4).
fn a_deflection() -> Deflection {
    Deflection::of(0.1).expect("a tenth of a millimetre is a length")
}
