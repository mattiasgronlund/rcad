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
//! **Every other row's sign threshold is zero, and the last of them got there on
//! 2026-09-17.** The quartic turned family was the holdout, at two tenths of a
//! percent; §118.11 took the stored direction's length out of the comparison two
//! leaning pipes ask and the rate went to nothing.
//! [`A_TURNED_QUARTIC_CROSSING_MAY_REFUSE`] carries the arithmetic and the
//! property that retired with the rate.
//!
//! # A rate on its own says nothing
//!
//! Every row also counts what the answers *were*. A regression that made every
//! crossing come back `Nothing` would leave every refusal count at zero and
//! every row green, so each one insists a real share of its pairs came back
//! naming a curve.

#![forbid(unsafe_code)]

mod crossings;
mod soak;

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, TestRunner};
use rcad_geom::{Error, SurfaceMeet, intersect};
use rcad_predicates::{Policy, ProvenanceLog};

use crossings::{
    Crossing, a_crossing_a_hair_from_the_knife_edge, a_crossing_somewhere_else, a_lattice_crossing,
    a_leaning_crossing, a_quartic_lattice_crossing, a_quartic_turned_crossing, a_turned_crossing,
};

/// How many pairs each measured row crosses.
///
/// Two thousand and forty-eight. A crossing is a handful of sign decisions
/// rather than a fuse, so the whole file runs in well under a second and the
/// corpus can be an order of magnitude wider than the one next door in
/// `rcad-topo` without costing anything.
const MEASURED_CASES: u32 = 2048;

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
/// Measured on the shipped corpus: 10.58%, and every one of those 217 refusals is
/// named once the same two surfaces are written back on the lattice. The count
/// read 215 and 10.5% from 2026-09-01 until §93 re-measured it; ninety-one commits
/// to `rcad-geom/src/` landed in between, and §93 is not one of the two, which
/// adds and removes no sign question at all. Twenty-five
/// percent is well above it and well below "most of them", which is what a
/// filter refusing things it could decide would look like.
const A_CROSSING_SOMEWHERE_ELSE_MAY_REFUSE: f64 = 0.25;

/// How much of a family may refuse for want of a half-space telling two lines
/// apart.
///
/// The *other* refusal, counted separately because it gives up on a different
/// question: not "what is the sign of this discriminant" but "which of these two
/// lines is this". A plane exactly along a pipe's axis and inside it cuts two
/// lines, and telling them apart needs a picking normal exactly square to the
/// axis. A line is endless, so a normal that leans into its direction by even
/// one rounding reads a different number a long way along it and the half-space
/// holds nothing — the squareness has to be arranged and cannot be tested
/// (§29.8).
///
/// **Arranging it is what M4 did**, and the arm that could not arrange it now
/// fires nowhere in this file. A picker is a `HalfSpace` and stores whatever
/// normal it is handed, and a caller can always hand over one exactly square to
/// a direction it holds — that direction crossed with a coordinate axis. §33.
///
/// Half a percent, and nothing reaches it. Measured on the shipped corpus:
/// **0.00% turned**, down from 0.44% and then from 0.05%; **none at all
/// leaning**, down from 0.20%; and none on the lattice, where the direction
/// across a pipe and a plane is a coordinate axis.
///
/// The one turned pair that still refused was §142.8's site one, and it was a
/// gap: `√(r² − (n·o − d)²/n·n)` was worked out again in intervals when the
/// caller had already summed the whole of `n·n·r² − (n·o − d)²` exactly, and
/// for an axis far from the origin the reading is wider than the pipe is thick.
/// It is read off that sum now.
///
/// **The bound stays at half a percent, because a draw is a sample of a family
/// and not the family.** The draw is fixed — `soak` seeds a deterministic
/// `ChaCha`, so these rates reproduce to the pair — but zero of *these* 2052 is
/// what one of these 2052 became, and one pair is what 0.05% was. A rate this
/// low is a pair or no pair, so a bound of zero would assert a structural fact
/// the measurement does not carry, and would break on the next change to a
/// generator rather than on a regression. `A_TURNED_CROSSING_MAY_REFUSE` next
/// door may be zero because §118.11 removed the arithmetic; site one's fix
/// re-routed the arithmetic here and did not remove it.
///
/// **What the bounded half does is still unclassified, not a limit**, and this
/// sentence says so because a rate with no verdict beside it reads as settled.
/// It no longer fires in *these* two families, but the same question — where
/// two lines sit relative to the flat surface separating them, on a plane so
/// close to a pipe's wall that they nearly meet — is what the band still
/// refuses at 2.68%. §93 fixed the same shape of arithmetic one row over and
/// §29.8 records what is untried.
const A_CROSSING_MAY_LACK_A_HALF_SPACE: f64 = 0.005;

/// How much of a quartic family may refuse for want of a half-space.
///
/// **Four percent, down from fifteen, and still the largest refusal rate in this
/// file.** Two loops of a leaning-pipe crossing are told apart by a half-space
/// through the fat pipe's axis, and a loop runs a long way *along* that axis —
/// so unlike the bounded pieces a ball leaves, the picking normal here has to be
/// exactly square to the axis or it reads a different number at either end.
///
/// That much is now arranged and never refused. What is left is the price of
/// arranging it: **the three normals `f64` can hold exactly square to a
/// direction are not the direction that separates the two loops**, and the best
/// of them can be up to 45° off it. Writing `n̂ = α·â + β·b̂`, a loop is picked
/// when `α·√(r₂² − (δ+r₁)²)` beats `|β|·(δ + r₁)` for the two pipes' radii and
/// the distance `δ` between their axes, and a fat pipe barely wider than the
/// thin one inside it does not clear that.
///
/// A **gap** and not a limit: the two loops are there, they are tellable apart,
/// and what runs out is `f64`'s supply of vectors exactly square to a given
/// direction. §33.
///
/// Measured on the shipped corpus: 0% on the lattice, **2.00% turned** — 41 of
/// 2052 — down from 11.86%.
///
/// **That figure is not this bound, and it drifted once already.** It read
/// 2.10% here and in §142.9 while the tree measured 2.05%: the constant below
/// is an upper bound, the test asserts against the bound, and nothing asserts
/// the rate the prose publishes. §142.8's site three took it from 2.05% to
/// 2.00%, and **site two will move it again** — re-measure rather than reason
/// from this line. `just t -E 'test(how_often_a_certified_naming_of_a_turned_quartic_crossing_refuses)' --no-capture`
/// prints it; the draw is seeded, so it reproduces to the pair.
const A_QUARTIC_CROSSING_MAY_LACK_A_HALF_SPACE: f64 = 0.04;

/// How much of the quartic pairs' turned family may refuse for want of a sign.
///
/// **Zero, down from two tenths of a percent, and §118.11 is what finished it.**
/// Deciding which of the four shapes two leaning pipes cross in compares
/// `ρ₁ ± ρ₂` against `δ`, and those used to be `rᵢ/√Wᵢ` and `|T|/√K` — three
/// square roots in three metrics, and clearing them took squaring an expansion
/// five hundred binary orders wide. §32 slid that comparison up the range before
/// squaring it and got the rate to 0.00% on the shipped corpus and 0.03% over
/// twenty times as many draws. §118.11 then made a pipe `r` wide rather than
/// `r/‖w‖` wide, so `ρᵢ = rᵢ`, the `√Wᵢ` leave the comparison altogether, and the
/// expansion whose own spread was past the window a slide has to land in stops
/// existing.
///
/// **This row is now the strong assertion its planar sibling above is**, for the
/// same §27.1 reason and with the same response owed if it ever fires: find
/// which product left the range, do not raise this number.
///
/// Measured on the shipped corpus: 0% on the lattice, **0% turned**, and 0 over
/// 409 604 draws on each of two seeds — 820 000 turned quartic pairs without one
/// sign refusal between them.
///
/// # The property that went with the rate
///
/// `every_turned_quartic_crossing_that_refuses_was_exactly_tangent_before_the_turn`
/// used to stand here. It took each refusal, untouched the turn, and showed the
/// original was settled — which is what told a knife edge a rotation made apart
/// from a timid filter, and what earned the 0.002 above. It drew twenty times
/// what the rows around it do because the refusal had already fallen to 0.03%.
///
/// **It is retired rather than re-aimed, because what it measured is gone.** The
/// family it drew from refuses no signs at all now, so its own vacuity guard —
/// `refused > 0` — is what failed. The only family in this file still refusing
/// for want of a sign is `a_crossing_somewhere_else`, and
/// `every_crossing_a_scaling_refuses_is_one_the_same_shapes_on_the_lattice_can_name`
/// already asks exactly this question of exactly those 217 refusals: take away
/// the thing that caused the refusal, and the answer comes back. Pointing a
/// second property at the same family would be one property written twice
/// (§13 rule 5), and two suites that go green together for one reason are one
/// suite.
///
/// What stands in its place is this constant at zero. A threshold of zero says
/// more than "under 0.2%, and each of the few was a knife edge" ever did.
const A_TURNED_QUARTIC_CROSSING_MAY_REFUSE: f64 = 0.0;

/// How much of the knife-edge family may refuse for want of a sign.
///
/// **The band, and the highest number in this file by a long way.** Every pair
/// here is drawn exactly tangent and then moved a handful of last bits off it,
/// which is precisely where a sign question has nothing left to stand on. A
/// family that refused none of these would not be reaching the band at all,
/// which is why the row below also insists it refuses *something*.
///
/// Five percent. Measured on the shipped corpus: **0%**, which was not the
/// expectation. Summing `T` and `K` through the expansion before enclosing them,
/// and squaring the roots away where `f64` leaves room to, between them reach
/// every pair drawn here — so what this row now measures is that the band is
/// being reached at all, which the half-space and touches-itself counts below it
/// show it is.
const A_CROSSING_ON_A_KNIFE_EDGE_MAY_REFUSE: f64 = 0.05;

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

/// The two rows the second stage added, on the lattice. Exactly representable
/// end to end, so nothing here may refuse either.
#[test]
fn a_certified_naming_of_a_quartic_lattice_crossing_never_refuses() {
    let tally = measured(a_quartic_lattice_crossing());
    tally.report("the quartic control: a ball and a pipe, and two pipes, in whole numbers");
    assert_eq!(
        tally.undecidable.get(),
        0,
        "on the lattice both stored directions have the same squared length, so the exact \
         level runs and a refusal is a bug in our filters"
    );
    // The half of the control that used to go unasserted, and the half that
    // caught this lane's own regression: on the lattice the direction that
    // separates two loops is already exactly square to the axis they run along,
    // so a picker built any other way is a step backwards rather than a fact
    // about geometry.
    assert_eq!(
        tally.no_half_space.get(),
        0,
        "on the lattice the direction separating two loops is exactly square to the axis, \
         so writing the half-space down rounds nothing"
    );
    tally.enough_of_them_crossed();
}

/// The two new rows under one general rotation, which is where the exact level
/// stops being able to back the enclosure up.
#[test]
fn how_often_a_certified_naming_of_a_turned_quartic_crossing_refuses() {
    let tally = measured(a_quartic_turned_crossing());
    tally.report("the quartic headline: the same two rows under one general rotation");
    tally.share_that_refused_is_within(A_TURNED_QUARTIC_CROSSING_MAY_REFUSE);
    tally.share_that_lacked_a_half_space_is_within(A_QUARTIC_CROSSING_MAY_LACK_A_HALF_SPACE);
    tally.enough_of_them_crossed();
}

/// **The band**, and the family §28.5 said the corpus could not reach.
///
/// Every pair here is drawn exactly tangent and then moved a few of its own last
/// bits off it, so the sign questions are settled — or not — a handful of
/// roundings from zero. That it refuses *something* is as much the point as that
/// it refuses little: a family drawn at the knife edge that never gave up would
/// mean the nudge was not landing in the band.
#[test]
fn how_often_a_certified_naming_of_a_crossing_a_hair_from_tangency_refuses() {
    let tally = measured(a_crossing_a_hair_from_the_knife_edge());
    tally.report("the band: drawn exactly tangent, then moved a few last bits off it");
    tally.share_that_refused_is_within(A_CROSSING_ON_A_KNIFE_EDGE_MAY_REFUSE);
    assert!(
        tally.undecidable.get() + tally.no_half_space.get() > 0,
        "a family drawn on the knife edge that never gives up is not reaching the knife edge"
    );
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
    let mut runner =
        TestRunner::new_with_rng(configured(soak::cases(MEASURED_CASES)), soak::seeded());
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
                match runs.len() {
                    1 => "one curve",
                    2 => "two curves",
                    4 => "four curves",
                    _ => "three curves, or more than four",
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
            // A crossing that runs through itself is no refusal of naming's
            // since Q2649: a ball reaching exactly across a pipe and a thin
            // pipe resting on the inside of a wider one are both named as two
            // loops that meet where the walls touch.
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
    let mut runner =
        TestRunner::new_with_rng(configured(soak::cases(MEASURED_CASES)), soak::seeded());
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
