//! How often naming a crossing gives up, family by family.
//!
//! §24.18's criteria 5 and 6, and §17.7's shape for both: the families are
//! reported separately and never blended into one percentage.
//!
//! # The control is the whole argument
//!
//! [`a_lattice_crossing`] is exactly representable end to end, so
//! `Policy::certified()` has to decide all of it. **One refusal there is one
//! bug**, and without that row the turned rate could not tell a timid filter
//! from genuine undecidability (§24.10).
//!
//! # What §27.1 predicted, and what this measures
//!
//! §24.10 wrote the turned rate down as "the headline" and expected it to be
//! above zero. §27.1 then sharpened the claim before any of this was built:
//! `intersect` takes two `Surface`s, a `Surface` is made of `f64`, and level 3
//! of the cascade decides any polynomial in exact `f64` — so the naming half has
//! **no constructed input at all**, and turning a pair does not give it one. The
//! prediction is therefore that the turned rate is zero as well, and that the
//! only thing that moves either rate is a number too large or too small to
//! multiply.
//!
//! That is what [`a_crossing_somewhere_else`] is for. It is the only family here
//! that refuses, it refuses for the one reason §27.1 named, and the refusal is a
//! **gap**: the same pair written back near the origin is decided.
//!
//! # A rate on its own says nothing
//!
//! Every row also counts what the answers *were*. A regression that made every
//! crossing come back `Nothing` would leave every refusal count at zero and
//! every row green, so each one insists a real share of its pairs came back
//! naming a curve.

#![forbid(unsafe_code)]

mod crossings;

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, RngAlgorithm, TestRng, TestRunner};
use rcad_geom::{Error, SurfaceMeet, intersect};
use rcad_predicates::{Policy, ProvenanceLog};

use crossings::{
    Crossing, a_crossing_somewhere_else, a_lattice_crossing, a_leaning_crossing, a_turned_crossing,
};

/// How many pairs each measured row crosses.
///
/// Two thousand and forty-eight. A crossing is a handful of sign decisions
/// rather than a fuse, so the whole file runs in well under a second and the
/// corpus can be an order of magnitude wider than the one next door in
/// `rcad-topo` without costing anything.
const MEASURED_CASES: u32 = 2048;

/// The environment variable that multiplies [`MEASURED_CASES`] for a soak.
///
/// ```text
/// RCAD_SOAK_FACTOR=200 cargo nextest run -p rcad-geom --no-capture \
///     how_often_a_certified_naming_of_a_turned_crossing_refuses
/// ```
const SOAK_FACTOR: &str = "RCAD_SOAK_FACTOR";

/// The environment variable that reseeds the measured rows.
///
/// Unset — which is how the suite always runs it — the seed is fixed and the
/// number reproduces.
const SOAK_SEED: &str = "RCAD_SOAK_SEED";

/// **§24.18 criterion 6's threshold, and it is zero.**
///
/// §24.10 asked for "a rate measured, asserted below a threshold recorded in
/// this file" and expected a rate above zero. §27.1 argued before the work that
/// there is nothing for a turned pair to be undecided *about*: both surfaces are
/// still made of exact `f64`, every discriminant is still a polynomial in those,
/// and level 3 still decides. The measurement agrees, and §28 records it.
///
/// A threshold of zero is a strong assertion and it is meant to be. If a change
/// ever makes a turned pair refuse, the honest response is to find which product
/// left the range rather than to raise this number.
const A_TURNED_CROSSING_MAY_REFUSE: f64 = 0.0;

/// How much of the leaning family may refuse, for the same reason as above.
const A_LEANING_CROSSING_MAY_REFUSE: f64 = 0.0;

/// How much of the somewhere-else family may refuse.
///
/// This is the one family that reaches the naming half's only refusal, and every
/// one of those refusals is a **gap** (§16.10): the answer exists, and the very
/// same two surfaces written back on the lattice are named. The scales are drawn
/// evenly across a range deliberately wider than the one `f64` can square, so
/// what this number measures is how much of that range is past the edge — and it
/// moves if the range does.
///
/// Measured on the shipped corpus: 10.5%, and every one of those 215 refusals is
/// named once the same two surfaces are written back on the lattice. Twenty-five
/// percent is well above it and well below "most of them", which is what a
/// filter refusing things it could decide would look like.
const A_CROSSING_SOMEWHERE_ELSE_MAY_REFUSE: f64 = 0.25;

/// How much of a family may refuse for want of a half-space telling two lines
/// apart.
///
/// The *other* refusal, counted separately because it gives up on a different
/// question: not "what is the sign of this discriminant" but "which of these two
/// lines is this". A plane exactly along a pipe's axis and inside it cuts two
/// lines, and telling them apart needs a unit normal exactly across both the
/// axis and the plane's normal, which `f64` does not always have. A **gap**, and
/// its fix is the fourteenth-degree separation test §27.1 hands to the second
/// stage.
///
/// Two percent. Measured on the shipped corpus: 0.54% turned, 0.63% leaning,
/// none at all on the lattice, where that direction is a coordinate axis.
const A_CROSSING_MAY_LACK_A_HALF_SPACE: f64 = 0.02;

/// How much of a family has to come back naming an actual curve.
///
/// A third. Two surfaces drawn independently miss each other often, and the
/// generators are built to reach every arm rather than to maximise this — but a
/// family where almost nothing crossed would make every refusal count above
/// vacuous.
const SOME_PAIRS_MUST_CROSS: f64 = 1.0 / 3.0;

/// The control. Exactly representable end to end, so nothing may refuse.
#[test]
fn a_certified_naming_of_a_lattice_crossing_never_refuses() {
    let tally = measured(a_lattice_crossing());
    tally.report("the control: whole numbers, planes square to an axis");
    assert_eq!(
        tally.undecidable.get(),
        0,
        "every one of these is exactly representable, so a refusal is a bug in our filters \
         and not a fact about geometry"
    );
    assert_eq!(
        tally.no_half_space.get(),
        0,
        "on the lattice the direction across a pipe and a plane is a coordinate axis, so \
         writing the half-space down rounds nothing"
    );
    tally.enough_of_them_crossed();
}

/// The headline family.
#[test]
fn how_often_a_certified_naming_of_a_turned_crossing_refuses() {
    let tally = measured(a_turned_crossing());
    tally.report("the headline: the control under one general rotation");
    tally.share_that_refused_is_within(A_TURNED_CROSSING_MAY_REFUSE);
    tally.share_that_lacked_a_half_space_is_within(A_CROSSING_MAY_LACK_A_HALF_SPACE);
    tally.enough_of_them_crossed();
}

/// Planes and pipes leaning over by whole-number slopes, so what is stored is a
/// rounding of what was asked for and an ellipse is an ordinary answer.
#[test]
fn how_often_a_certified_naming_of_a_leaning_crossing_refuses() {
    let tally = measured(a_leaning_crossing());
    tally.report("leaning: whole-number directions, scaled to length one and rounded");
    tally.share_that_refused_is_within(A_LEANING_CROSSING_MAY_REFUSE);
    tally.share_that_lacked_a_half_space_is_within(A_CROSSING_MAY_LACK_A_HALF_SPACE);
    tally.enough_of_them_crossed();
}

/// The only family that reaches the naming half's one refusal.
#[test]
fn how_often_a_certified_naming_of_a_crossing_somewhere_else_refuses() {
    let tally = measured(a_crossing_somewhere_else());
    tally.report("somewhere else: the control at 2^k, moved by 2^m");
    tally.share_that_refused_is_within(A_CROSSING_SOMEWHERE_ELSE_MAY_REFUSE);
    tally.enough_of_them_crossed();
}

/// The refusal that family reaches is a **gap**, demonstrated rather than
/// argued: the same pair moved back up the number line is decided.
///
/// `CLAUDE.md` asks for a demonstration and not a paragraph, and this is it. It
/// is also the one property in this file that would still mean something if
/// every rate above went to zero.
#[test]
fn every_crossing_a_scaling_refuses_is_one_the_same_shapes_on_the_lattice_can_name() {
    let mut runner = TestRunner::new_with_rng(configured(measured_cases()), fresh_seeded_soak());
    let refused = Cell::new(0usize);
    let rescued = Cell::new(0usize);

    let outcome = runner.run(&a_crossing_somewhere_else(), |crossing| {
        let [first, second] = crossing.surfaces();
        if !matches!(
            intersect(
                &first,
                &second,
                &Policy::certified(),
                &mut ProvenanceLog::new()
            ),
            Err(Error::Undecidable { .. })
        ) {
            return Ok(());
        }
        refused.set(refused.get() + 1);

        let mut back_on_the_lattice = crossing;
        back_on_the_lattice.elsewhere.scale = 0;
        let [first, second] = back_on_the_lattice.surfaces();
        let named = intersect(
            &first,
            &second,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        if named.is_ok() {
            rescued.set(rescued.get() + 1);
        }
        Ok(())
    });
    if let Err(why) = outcome {
        panic!("{why}");
    }

    println!(
        "of {} refused crossings, {} were named once the same shapes were written back on the \
         lattice",
        refused.get(),
        rescued.get()
    );
    assert!(
        refused.get() > 0,
        "this property measures nothing unless the scaling family actually refuses something"
    );
    assert_eq!(
        rescued.get(),
        refused.get(),
        "every one of these refusals is a gap and not a limit, so a rescale has to reach \
         the answer for all of them"
    );
}

/// What a family's crossings did.
#[derive(Default)]
struct Tally {
    named: Cell<usize>,
    undecidable: Cell<usize>,
    no_half_space: Cell<usize>,
    crossed: Cell<usize>,
    /// Which answer came back, and how often. A rate on its own cannot tell a
    /// timid filter from a family that stopped crossing at all (§14.7).
    answers: RefCell<BTreeMap<&'static str, usize>>,
    /// Which question each refusal named.
    gave_up: RefCell<BTreeMap<&'static str, usize>>,
}

impl Tally {
    fn saw(&self, crossed: &Result<SurfaceMeet, Error>) {
        self.named.set(self.named.get() + 1);
        let answer = match crossed {
            Ok(SurfaceMeet::Nothing) => "nothing",
            Ok(SurfaceMeet::TouchAtOnePoint { .. }) => "one point",
            Ok(SurfaceMeet::Curves(runs)) => {
                self.crossed.set(self.crossed.get() + 1);
                if runs.len() == 1 {
                    "one curve"
                } else {
                    "two curves"
                }
            }
            Ok(SurfaceMeet::SameSurface) => "the same surface",
            Err(Error::Undecidable { records }) => {
                self.undecidable.set(self.undecidable.get() + 1);
                let mut gave_up = self.gave_up.borrow_mut();
                for record in records.records() {
                    *gave_up.entry(record.predicate()).or_default() += 1;
                }
                "refused: could not settle a sign"
            }
            Err(Error::CannotNameWhichPieceOfACrossingThisIs) => {
                self.no_half_space.set(self.no_half_space.get() + 1);
                "refused: no half-space tells the two pieces apart"
            }
            Err(why) => panic!("a pair of surfaces this generator draws should not give {why}"),
            // `SurfaceMeet` grows as the pair table does, and guessing what a
            // new arm means is worse than stopping.
            Ok(other) => panic!("unhandled crossing: {other:?}"),
        };
        *self.answers.borrow_mut().entry(answer).or_default() += 1;
    }

    fn report(&self, what: &str) {
        let named = self.named.get();
        println!(
            "{what}: {named} pairs, {} refused for want of a sign ({:.2}%), {} for want of a \
             half-space ({:.2}%)",
            self.undecidable.get(),
            100.0 * self.undecidable.get() as f64 / named as f64,
            self.no_half_space.get(),
            100.0 * self.no_half_space.get() as f64 / named as f64
        );
        for (answer, count) in self.answers.borrow().iter() {
            println!(
                "    {count} ({:.1}%) {answer}",
                100.0 * *count as f64 / named as f64
            );
        }
        for (question, count) in self.gave_up.borrow().iter() {
            println!(
                "    a refusal named \"{question}\" {count} times ({:.2}% of all pairs)",
                100.0 * *count as f64 / named as f64
            );
        }
    }

    /// Insist the share that could not tell two lines apart is inside what this
    /// family is allowed.
    fn share_that_lacked_a_half_space_is_within(&self, threshold: f64) {
        let share = self.no_half_space.get() as f64 / self.named.get() as f64;
        assert!(
            share <= threshold,
            "{} of {} crossings came in two pieces no half-space could tell apart ({:.2}%), \
             past the {:.2}% this family is allowed",
            self.no_half_space.get(),
            self.named.get(),
            100.0 * share,
            100.0 * threshold
        );
    }

    /// Insist the share that could not be named is inside what this family is
    /// allowed.
    fn share_that_refused_is_within(&self, threshold: f64) {
        let share = self.undecidable.get() as f64 / self.named.get() as f64;
        assert!(
            share <= threshold,
            "{} of {} crossings could not be named ({:.2}%), past the {:.2}% this family is \
             allowed",
            self.undecidable.get(),
            self.named.get(),
            100.0 * share,
            100.0 * threshold
        );
    }

    /// The vacuity guard. A family where nothing crossed measures nothing.
    fn enough_of_them_crossed(&self) {
        let share = self.crossed.get() as f64 / self.named.get() as f64;
        assert!(
            share >= SOME_PAIRS_MUST_CROSS,
            "only {} of {} pairs came back naming a curve ({:.1}%), so the refusal counts \
             above are barely measuring anything",
            self.crossed.get(),
            self.named.get(),
            100.0 * share
        );
    }
}

/// Cross every pair a strategy draws, from a fixed seed, and hand back what
/// happened.
fn measured(strategy: impl Strategy<Value = Crossing>) -> Tally {
    let mut runner = TestRunner::new_with_rng(configured(measured_cases()), fresh_seeded_soak());
    let tally = Tally::default();
    let outcome = runner.run(&strategy, |crossing| {
        let [first, second] = crossing.surfaces();
        tally.saw(&intersect(
            &first,
            &second,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        ));
        Ok(())
    });
    if let Err(why) = outcome {
        panic!("{why}");
    }
    tally
}

/// [`MEASURED_CASES`], multiplied by whatever [`SOAK_FACTOR`] says.
///
/// # Panics
///
/// If the variable is set to something that is not a whole number above zero.
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

fn configured(cases: u32) -> Config {
    Config {
        cases,
        source_file: Some(file!()),
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
            "proptest-regressions",
        ))),
        ..Config::default()
    }
}
