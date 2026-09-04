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
//!   normal, and its wires as rings of corners, each ring turned to start at its
//!   own smallest corner and the whole lot sorted. Used where the two runs must
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

mod pairs;

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::ops::RangeInclusive;

use proptest::prelude::*;
use proptest::test_runner::{
    Config, FileFailurePersistence, RngAlgorithm, TestCaseError, TestRng, TestRunner,
};
use rcad_geom::{Deflection, Orientation, Plane, Point, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

use pairs::{
    BallAndBlock, Configuration, CurvedPair, Movement, Rigid, a_chain_of_fuses, a_curved_pair,
    a_flush_pair, a_lattice_pair, a_lattice_trio, a_pair_leaning_the_same_way,
    a_pair_somewhere_else, a_rigid_movement, a_skew_pair, a_solid_and_its_copy, a_turned_pair,
    a_turned_solid_and_its_copy, a_turned_solid_and_several_copies,
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
/// statistics. At this count each family is 423 fuses, the whole workspace suite
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
/// seed, so the shipped corpus of 423 turned fuses refuses 96 of them, 22.70%,
/// every run on every machine. There is no sampling noise for the threshold to
/// have to clear, so what it has to clear instead is ordinary drift — 1.3 points
/// is six fuses — and what it has to catch is a regression. The three undeserved
/// refusals recorded so far are each worth several times that on their own: the
/// two §17.7 names are about 3.5 points apiece, and §19.1's plane reader was
/// worth 4.4, which is what took this number from 28 to 24 when it was fixed.
///
/// **Six fuses is thin, and §39 says where the room went.** §38.8 read the
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
/// still fires on anything 1.3 points above the measurement, which is well
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
/// of 423 skew fuses refuses 51 of them, 12.06%, every run on every machine.
/// There is no sampling noise to clear, so what the gap has to allow is ordinary
/// drift — 5.9 points is twenty-five fuses, four times the headroom
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

/// How much of the lattice control comes back as something a `Model` cannot
/// hold.
///
/// Measured at 4.7% over the shipped corpus — 20 of 423 — and the band is
/// roughly half to double that. Every one of them is the case §17.7's catalogue
/// records by hand: two solids meeting along a whole edge would put four faces
/// along it, and an edge belongs to exactly two.
const A_LATTICE_PAIR_IS_NOT_A_SOLID: RangeInclusive<f64> = 0.02..=0.10;

/// The same for the family built to share coordinates, which is full of them.
///
/// Measured at 17.0% — 72 of 423 — which is what "deliberately degenerate"
/// buys: the offsets are whole extents, so edge-on-edge and corner-on-corner
/// are the ordinary case here rather than the rare one.
const A_FLUSH_PAIR_IS_NOT_A_SOLID: RangeInclusive<f64> = 0.10..=0.28;

/// The same for the lattice pairs written somewhere else in `f64`.
///
/// Measured at 4.0% — 17 of 423 — against the lattice family's 4.7%, and the two
/// are the same figure rather than two. The configurations are drawn from the
/// same generator and the same seed, but this family draws two more numbers per
/// case, so the two corpora are not the same 128 configurations. What is exactly
/// the same is the reason: moving everything by a power of two cannot turn an
/// answer into one a `Model` can hold or the other way about. Same band.
const A_PAIR_ELSEWHERE_IS_NOT_A_SOLID: RangeInclusive<f64> = 0.02..=0.10;

/// The same for the turned family.
///
/// Measured at 4.0% — 17 of 423 — lower than the lattice family's 4.7%, which it
/// is a rotation of, because getting on for a quarter of that family's fuses
/// refuse before they get far enough to find out whether the answer is a
/// solid.
const A_TURNED_PAIR_IS_NOT_A_SOLID: RangeInclusive<f64> = 0.015..=0.09;

/// The same for the family whose outlines lean over.
///
/// Measured at 1.9% over the shipped corpus — 8 of 423 — which is **two fifths**
/// of the lattice family's 4.7%, and the reason is the point of the family. An
/// answer is not a solid here almost always because two solids meet along a
/// whole edge and four faces would run along it, and that needs the two edges to
/// be the same edge exactly. Leaning the two solids by different amounts is
/// precisely what stops them sharing an edge. It is still the lowest of the six
/// families by some way, and it is the only one where the ratio to the lattice
/// family is what the family was built to be about.
const A_SKEW_PAIR_IS_NOT_A_SOLID: RangeInclusive<f64> = 0.005..=0.03;

/// The same for its control, where both solids lean the same way.
///
/// Measured at 4.0% — 17 of 423 — and it is the lattice family's 4.7% rather
/// than the skew family's 1.9%, which is the arithmetic saying what the control
/// claims. Leaning both solids by the same amount is an affine map of the whole
/// pair, and an affine map preserves incidence exactly, so the edges that
/// coincided still coincide and the answers that were not solids still are not.
/// It does not land on 4.7% exactly because this family draws one more number
/// per case, so the two corpora are not the same 128 configurations — the same
/// caveat [`A_PAIR_ELSEWHERE_IS_NOT_A_SOLID`] carries — and because a fuse that
/// refuses never gets far enough to find out whether its answer was a solid.
const A_PAIR_LEANING_ALIKE_IS_NOT_A_SOLID: RangeInclusive<f64> = 0.012..=0.08;

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
        else {
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
        else {
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

        let Some(left) = united_in_turn(tally, &model, &[one], &[two], &[three])? else {
            return Ok(());
        };
        let Some(right) = united_in_turn(tally, &model, &[two], &[three], &[one])? else {
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
        else {
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
        else {
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
        else {
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
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        tally.meaning_to_compare();
        let (still, keys) = configuration.build();
        let Some(before) = fused(
            tally,
            Coordinates::AreExactlyRepresentable,
            &still,
            &[keys[0]],
            &[keys[1]],
            rule,
        )?
        else {
            continue;
        };

        let (moved, keys) = configuration.built_after(&movement);
        let Some(after) = fused(
            tally,
            Coordinates::AreExactlyRepresentable,
            &moved,
            &[keys[0]],
            &[keys[1]],
            rule,
        )?
        else {
            return Err(TestCaseError::fail(format!(
                "{rule:?} came back with an answer where the moved pair did not"
            )));
        };

        tally.comparing();
        let expected = answer_of(&before.0, &before.1)
            .moved_by(&movement.in_the_world_of(configuration.elsewhere));
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
        else {
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
        else {
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
            else {
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
        else {
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
        else {
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
        else {
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
        else {
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
        else {
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
        else {
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
        else {
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
        else {
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
        else {
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
            else {
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
            else {
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
                else {
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
            else {
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
            else {
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
/// The constructor now divides by the largest-magnitude component first, so the
/// answer depends on the ratios alone. This is the topology-side guard on that;
/// `rcad-geom` has the arithmetic in
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
    // Both land on the same number, and it is one ulp under the correctly
    // rounded 1/√2. Canonical is not the same as closest: normalizing divides
    // by a square root that has already been rounded, and no ordering of that
    // division recovers the last bit. What §22.6 was about is the two walls
    // agreeing, and they do.
    let canonical = 0.707_106_781_186_547_5_f64;
    assert_eq!(plane_of(1.0)[0], canonical);
    assert_eq!(
        canonical.to_bits() + 1,
        core::f64::consts::FRAC_1_SQRT_2.to_bits(),
        "one ulp under, and stated here so a change to it is a change somebody \
         has to look at"
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

        let Some(forwards) = fused(tally, coordinates, &model, &[one], &[other], rule)? else {
            continue;
        };
        let Some(backwards) = fused(tally, coordinates, &model, &[other], &[one], rule)? else {
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
    model: &Model,
    first: &[SolidKey],
    second: &[SolidKey],
    third: &[SolidKey],
) -> Result<Option<(Model, Vec<SolidKey>)>, TestCaseError> {
    let Some(pair) = fused(
        tally,
        Coordinates::AreExactlyRepresentable,
        model,
        first,
        second,
        Rule::Union,
    )?
    else {
        return Ok(None);
    };
    fused(
        tally,
        Coordinates::AreExactlyRepresentable,
        &pair.0,
        &pair.1,
        third,
        Rule::Union,
    )
}

/// Fuse a copy of `model`, counting what came back.
///
/// A copy, so that every fuse in a property starts from the same model rather
/// than from whatever the last one left behind. `None` means the answer was not
/// a solid this kernel can hold, or was refused where refusals are expected —
/// both counted, neither swept away.
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
) -> Result<Option<(Model, Vec<SolidKey>)>, TestCaseError> {
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
            Ok(Some((model, solids)))
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
                Coordinates::AreUnderAGeneralRotation | Coordinates::SitOnTiltedWalls => Ok(None),
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
            Ok(None)
        }
        Err(why) => Err(TestCaseError::fail(format!("{rule:?} went wrong: {why}"))),
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
        for face in model.solid(solid).faces() {
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
        for face in model.solid(solid).faces() {
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
            self.compared.get() as f64 >= SOME_COMPARISONS_MUST_HAPPEN * meant_to as f64,
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
            "{} of {} fuses came back not a solid ({:.1}%), outside the {:.0}%–{:.0}% \
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
/// If the variable is set to something that is not a whole number above zero. A
/// soak that silently ran the default corpus would be worse than one that
/// stopped.
fn measured_cases() -> u32 {
    let Ok(factor) = std::env::var(SOAK_FACTOR) else {
        return MEASURED_CASES;
    };
    let factor: u32 = factor
        .parse()
        .unwrap_or_else(|_| panic!("{SOAK_FACTOR} has to be a whole number, not {factor:?}"));
    assert!(factor > 0, "{SOAK_FACTOR} has to be at least one");
    MEASURED_CASES * factor
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
    solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .measure(
                    a_deflection(),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
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
            let solid = model.solid(solid);
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
    /// # Panics
    ///
    /// If a face of the answer is curved. Only the rectilinear families are
    /// moved and compared — a [`Movement`] is a quarter turn and a whole-number
    /// shift, which is exact on a plane's four coefficients and on a corner, and
    /// nothing here has worked out what it is on a pipe's axis.
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
                                _ => panic!(
                                    "only the rectilinear families move an answer, and this one \
                                     has a curved face in it"
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

fn written_out(face: &Face) -> Vec<u64> {
    let mut written = face.sits.written_out();
    written.push(u64::from(face.reversed));

    let mut wires: Vec<Vec<u64>> = face
        .wires
        .iter()
        .map(|wire| turned_to_its_smallest(wire))
        .collect();
    wires.sort_unstable();
    written.push(wires.len() as u64);
    for wire in wires {
        written.push(wire.len() as u64);
        written.extend(wire);
    }
    written
}

fn turned_to_its_smallest(wire: &[[f64; 3]]) -> Vec<u64> {
    let corners: Vec<[u64; 3]> = wire.iter().map(|&at| at.map(bits)).collect();
    let smallest = corners
        .iter()
        .enumerate()
        .min_by_key(|(_, corner)| **corner)
        .map_or(0, |(position, _)| position);
    (0..corners.len())
        .flat_map(|step| corners[(smallest + step) % corners.len()])
        .collect()
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
