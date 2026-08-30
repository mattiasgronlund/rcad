//! Fuses that have to give the same answer as each other, whatever that answer
//! turns out to be.
//!
//! §1.7's sixth exit criterion and §14.8's three extra oracles, over the three
//! configuration families §17.7 asks for and reports separately. Nothing here
//! knows what the right answer to any fuse is; every test runs the same question
//! two ways round and insists the two agree.
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
//! 3. **What family of inputs?** Three, reported separately, in `pairs/mod.rs`.
//! 4. **Which gap would this have caught?** §12.3's, twice over. A generator of
//!    one shape family finds one family of bugs, and `random_extrusions` threw
//!    away every input the builder refused, so a wrongly refused input was
//!    invisible. Nothing here filters: every fuse is counted, refusals are
//!    counted separately from answers, and on the two exactly representable
//!    families a refusal fails the test.

mod pairs;

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::fmt::Write as _;

use proptest::prelude::*;
use proptest::test_runner::{
    Config, FileFailurePersistence, RngAlgorithm, TestCaseError, TestRng, TestRunner,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

use pairs::{
    Configuration, Rigid, a_flush_pair, a_lattice_pair, a_lattice_trio, a_rigid_movement,
    a_solid_and_its_copy, a_turned_pair, a_turned_solid_and_its_copy,
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

/// How many the two measurement tests see.
///
/// Larger, because their number is the thing being reported rather than a means
/// to an end, and they run on a fixed seed so the number is the same every time.
const MEASURED_CASES: u32 = 128;

/// How much of the turned family a certified fuse is allowed to refuse.
///
/// **This is §17.7's criterion 8 threshold**, recorded in that section and
/// asserted by [`how_often_a_certified_fuse_of_a_turned_pair_refuses`]. It is
/// also §14.7's other half — that valid input is *accepted* — because a
/// certified fuse that refused most of a family it can plainly answer would pass
/// every property in this file, a property only ever looking at the fuses that
/// came back.
///
/// Thirty-two per cent, chosen from measurement rather than from taste. The
/// fixed-seed rate is 25.4%, and over fourteen fresh seeds the same corpus ran
/// from 19.6% to 28.3%, so no seed reaches this. It sits just under 32.8%, which
/// is what the rate was before the two undeserved refusals of §14.7 were fixed,
/// so either of those coming back trips it.
const TURNED_REFUSALS_MUST_STAY_BELOW: f64 = 0.32;

#[test]
fn what_the_union_and_the_overlap_take_up_is_what_the_two_solids_take_up() {
    let tally = over(CASES, a_lattice_pair(), |configuration, tally| {
        let (model, keys) = configuration.build();
        let (one, other) = (keys[0], keys[1]);

        let Some(united) = fused(
            tally,
            Refusals::AreABug,
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
            Refusals::AreABug,
            &model,
            &[one],
            &[other],
            Rule::Intersection,
        )?
        else {
            return Ok(());
        };

        prop_assert_eq!(
            volume_of(&united.0, &united.1) + volume_of(&shared.0, &shared.1),
            volume_of(&model, &[one]) + volume_of(&model, &[other]),
            "the union and the overlap together are the two solids together"
        );
        Ok(())
    });
    tally.report("inclusion and exclusion, lattice pairs");
}

#[test]
fn uniting_the_first_two_and_then_the_third_is_uniting_the_last_two_and_then_the_first() {
    let tally = over(LONGER_CASES, a_lattice_trio(), |configuration, tally| {
        let (model, keys) = configuration.build();
        let (one, two, three) = (keys[0], keys[1], keys[2]);

        let Some(left) = united_in_turn(tally, &model, &[one], &[two], &[three])? else {
            return Ok(());
        };
        let Some(right) = united_in_turn(tally, &model, &[two], &[three], &[one])? else {
            return Ok(());
        };

        prop_assert_eq!(
            invariants_of(&left.0, &left.1),
            invariants_of(&right.0, &right.1),
            "uniting three solids does not depend on the order they are united in"
        );
        Ok(())
    });
    tally.report("associativity of union, lattice trios");
}

#[test]
fn taking_away_what_is_left_after_taking_the_other_away_leaves_what_the_two_share() {
    let tally = over(LONGER_CASES, a_lattice_pair(), |configuration, tally| {
        let (model, keys) = configuration.build();
        let (one, other) = (keys[0], keys[1]);

        let Some(bitten) = fused(
            tally,
            Refusals::AreABug,
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
            Refusals::AreABug,
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
            Refusals::AreABug,
            &model,
            &[one],
            &[other],
            Rule::Intersection,
        )?
        else {
            return Ok(());
        };

        prop_assert_eq!(
            invariants_of(&twice_bitten.0, &twice_bitten.1),
            invariants_of(&shared.0, &shared.1),
            "A \\ (A \\ B) is A ∩ B"
        );
        Ok(())
    });
    tally.report("A \\ (A \\ B) = A ∩ B, lattice pairs");
}

#[test]
fn turning_a_pair_a_quarter_turn_and_moving_it_a_whole_number_changes_not_one_bit() {
    let tally = over(
        LONGER_CASES,
        (a_lattice_pair(), a_rigid_movement()),
        |(configuration, movement), tally| {
            for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                let (still, keys) = configuration.build();
                let Some(before) = fused(
                    tally,
                    Refusals::AreABug,
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
                    Refusals::AreABug,
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

                let expected = answer_of(&before.0, &before.1).moved_by(&movement);
                same_answer(&expected, &answer_of(&after.0, &after.1), rule, "moving it")?;
            }
            Ok(())
        },
    );
    tally.report("transform invariance, lattice pairs under quarter turns and whole-number shifts");
}

#[test]
fn swapping_the_two_sides_of_a_lattice_pair_changes_not_one_bit() {
    let tally = over(CASES, a_lattice_pair(), |configuration, tally| {
        both_ways_round(tally, Refusals::AreABug, configuration)
    });
    tally.report("commutativity, lattice pairs");
}

#[test]
fn swapping_the_two_sides_of_a_turned_pair_changes_not_one_bit() {
    let tally = over(CASES, a_turned_pair(), |configuration, tally| {
        both_ways_round(tally, Refusals::AreExpected, configuration)
    });
    tally.report("commutativity, turned pairs");
}

#[test]
fn swapping_the_two_sides_of_a_degenerate_pair_changes_not_one_bit() {
    let tally = over(CASES, a_flush_pair(), |configuration, tally| {
        both_ways_round(tally, Refusals::AreABug, configuration)
    });
    tally.report("commutativity, pairs sharing coordinates");
}

#[test]
fn a_solid_united_with_a_copy_of_itself_is_itself() {
    let tally = over(CASES, a_solid_and_its_copy(), |configuration, tally| {
        let (model, keys) = configuration.build();
        let (one, copy) = (keys[0], keys[1]);
        let itself = invariants_of(&model, &[one]);

        let Some(united) = fused(
            tally,
            Refusals::AreABug,
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
            Refusals::AreABug,
            &model,
            &[one],
            &[copy],
            Rule::Intersection,
        )?
        else {
            return Ok(());
        };
        prop_assert_eq!(invariants_of(&shared.0, &shared.1), itself);

        same_answer(
            &answer_of(&united.0, &united.1),
            &answer_of(&shared.0, &shared.1),
            Rule::Union,
            "uniting a solid with a copy of itself against sharing with it",
        )?;
        Ok(())
    });
    tally.report("idempotence, a lattice solid and a copy of it");
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
        let refusals = if name == "turned" {
            Refusals::AreExpected
        } else {
            Refusals::AreABug
        };
        let tally = over(CASES, strategy, |configuration, tally| {
            let (model, keys) = configuration.build();
            let Some(nothing) = fused(
                tally,
                refusals,
                &model,
                &[keys[0]],
                &[keys[1]],
                Rule::Difference,
            )?
            else {
                return Ok(());
            };
            prop_assert!(
                nothing.1.is_empty(),
                "a solid with itself taken away leaves nothing, got {} solids",
                nothing.1.len()
            );
            Ok(())
        });
        tally.report(&format!("A \\ A = ∅, {name} solid and a copy of it"));
    }
}

#[test]
fn a_solid_united_with_what_it_shares_with_another_is_itself() {
    let tally = over(LONGER_CASES, a_lattice_pair(), |configuration, tally| {
        let (model, keys) = configuration.build();
        let (one, other) = (keys[0], keys[1]);

        let Some(shared) = fused(
            tally,
            Refusals::AreABug,
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
            Refusals::AreABug,
            &shared.0,
            &[one],
            &shared.1,
            Rule::Union,
        )?
        else {
            return Ok(());
        };

        prop_assert_eq!(
            invariants_of(&swallowed.0, &swallowed.1),
            invariants_of(&model, &[one]),
            "A ∪ (A ∩ B) is A"
        );
        Ok(())
    });
    tally.report("absorption, lattice pairs");
}

#[test]
fn uniting_three_at_once_is_uniting_two_and_then_the_third() {
    let tally = over(LONGER_CASES, a_lattice_trio(), |configuration, tally| {
        let (model, keys) = configuration.build();

        let Some(at_once) = fused(tally, Refusals::AreABug, &model, &keys, &[], Rule::Union)?
        else {
            return Ok(());
        };
        let Some(first_two) = fused(
            tally,
            Refusals::AreABug,
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
            Refusals::AreABug,
            &first_two.0,
            &then_the_third,
            &[],
            Rule::Union,
        )?
        else {
            return Ok(());
        };

        prop_assert_eq!(
            invariants_of(&at_once.0, &at_once.1),
            invariants_of(&in_steps.0, &in_steps.1),
            "fuse([A, B, C]) is fuse([fuse([A, B]), C])"
        );
        Ok(())
    });
    tally.report("fuse([A,B,C]) = fuse([fuse([A,B]),C]), lattice trios");
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
        every_rule(tally, Refusals::AreABug, configuration)
    });
    tally.report("§17.7 control: lattice pairs, every rule");
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
        every_rule(tally, Refusals::AreABug, configuration)
    });
    tally.report("§17.7 degenerate: pairs sharing coordinates, every rule");
    assert_eq!(
        tally.refused.get(),
        0,
        "two solids sharing coordinates exactly leave nothing that cannot be decided"
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
        every_rule(tally, Refusals::AreExpected, configuration)
    });
    tally.report("§17.7 headline: turned pairs, every rule");
    let refused = tally.refused.get() as f64 / tally.fuses.get() as f64;
    assert!(
        refused < TURNED_REFUSALS_MUST_STAY_BELOW,
        "{:.1}% of {} turned fuses refused, over the {:.0}% §17.7 records",
        refused * 100.0,
        tally.fuses.get(),
        TURNED_REFUSALS_MUST_STAY_BELOW * 100.0
    );
}

/// `A ∪ B` and `B ∪ A`, then `A ∩ B` and `B ∩ A`, bit for bit.
///
/// Swapping the two sides is the sharpest question this suite asks, because a
/// splitter that cut a face by one line and then cut the pieces by the next
/// would compute a genuinely different partition each way round and there would
/// be nothing to fix (§17.4). Comparing volumes would not see it.
fn both_ways_round(
    tally: &Tally,
    refusals: Refusals,
    configuration: Configuration,
) -> Result<(), TestCaseError> {
    for rule in [Rule::Union, Rule::Intersection] {
        let (model, keys) = configuration.build();
        let (one, other) = (keys[0], keys[1]);

        let Some(forwards) = fused(tally, refusals, &model, &[one], &[other], rule)? else {
            continue;
        };
        let Some(backwards) = fused(tally, refusals, &model, &[other], &[one], rule)? else {
            continue;
        };

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
    refusals: Refusals,
    configuration: Configuration,
) -> Result<(), TestCaseError> {
    let (model, keys) = configuration.build();
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        fused(tally, refusals, &model, &[keys[0]], &[keys[1]], rule)?;
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
    let Some(pair) = fused(tally, Refusals::AreABug, model, first, second, Rule::Union)? else {
        return Ok(None);
    };
    fused(
        tally,
        Refusals::AreABug,
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
fn fused(
    tally: &Tally,
    refusals: Refusals,
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
            match refusals {
                Refusals::AreExpected => Ok(None),
                Refusals::AreABug => Err(TestCaseError::fail(format!(
                    "a {rule:?} over exactly representable coordinates refused, with {} records",
                    records.len()
                ))),
            }
        }
        Err(Error::ResultIsNotManifold { .. }) => {
            tally.not_a_solid.set(tally.not_a_solid.get() + 1);
            Ok(None)
        }
        Err(why) => Err(TestCaseError::fail(format!("{rule:?} went wrong: {why}"))),
    }
}

/// Whether a refusal is an outcome to count or a test failure.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Refusals {
    /// The configuration is exactly representable, so a certified policy has to
    /// decide it (§17.7's control row).
    AreABug,
    /// The configuration is under a general rotation, where a refusal is the
    /// thing being measured.
    AreExpected,
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
    /// Which predicate each refusal named, and how often.
    ///
    /// Criterion 8 asks for a rate, and a rate on its own cannot tell a timid
    /// filter from genuine undecidability (§14.7). The name of the predicate
    /// that gave up is the cheapest thing that can.
    gave_up: RefCell<BTreeMap<&'static str, usize>>,
}

impl Tally {
    fn report(&self, what: &str) {
        let fuses = self.fuses.get();
        println!(
            "{what}: {fuses} fuses, {} refused ({:.1}%), {} not a solid",
            self.refused.get(),
            100.0 * self.refused.get() as f64 / fuses as f64,
            self.not_a_solid.get()
        );
        for (predicate, count) in self.gave_up.borrow().iter() {
            println!(
                "    {count} ({:.1}% of all fuses) named \"{predicate}\"",
                100.0 * *count as f64 / fuses as f64
            );
        }
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
fn measured<T: core::fmt::Debug>(
    strategy: impl Strategy<Value = T>,
    property: impl Fn(T, &Tally) -> Result<(), TestCaseError>,
) -> Tally {
    run(
        TestRunner::new_with_rng(
            configured(MEASURED_CASES),
            TestRng::deterministic_rng(RngAlgorithm::ChaCha),
        ),
        strategy,
        property,
    )
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
        .map(|&solid| model.solid(solid).measure().volume())
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
            let measure = solid.measure();
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

/// One face: the plane it sits on, which way it faces, and its rings of corners.
#[derive(Clone, Debug)]
struct Face {
    plane: [f64; 4],
    outward: [f64; 3],
    wires: Vec<Vec<[f64; 3]>>,
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
                        plane: face.plane().coefficients(),
                        outward: face.outward_normal().components(),
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
    fn moved_by(&self, movement: &Rigid) -> Answer {
        Answer {
            solids: self
                .solids
                .iter()
                .map(|faces| {
                    faces
                        .iter()
                        .map(|face| Face {
                            plane: movement.plane(face.plane),
                            outward: movement.direction(face.outward),
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
    let mut written: Vec<u64> = face
        .plane
        .iter()
        .chain(&face.outward)
        .copied()
        .map(bits)
        .collect();

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
