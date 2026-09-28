//! A fuse checked against what its rule is supposed to mean, not against
//! another computation path.
//!
//! `fuses_that_must_agree.rs`, next door, never asks whether a fuse's answer is
//! *right* — it asks whether two ways of reaching an answer land on the same
//! one. This file asks the other question, the one nothing else in the suite
//! asks: for a point genuinely inside A and genuinely outside B, does a union
//! actually contain it? Does an intersection actually refuse it? That is the
//! rule's own meaning, read out of the dictionary rather than cross-checked
//! against a second run of the same arithmetic.
//!
//! # How a point is judged
//!
//! A point is sampled from a fine offset grid across the box that holds both A
//! and B ([`SolidView::box_around_it`]), classified against A and against B
//! with [`SolidView::classify_point`], and thrown away unless both answers are
//! a decisive [`Containment::Inside`] or [`Containment::Outside`] — a boundary,
//! an undecidable predicate, and a trimming or surface a ray could not be fired
//! at all mean "skip", never "coerce to a side". The grid offset
//! (`n * 0.125 + 0.0625`) is never a whole number, and every plane in the
//! lattice family sits on one, so a surviving point is certifiably clear of
//! every wall of A, B and the fused answer alike — nothing here is coerced,
//! and nothing here needs a tolerance.
//!
//! What survives is checked against the fused answer and the rule's own truth
//! table: union is "in A or in B", intersection is "in A and in B", difference
//! is "in A and not in B". Any refusal — of the fuse itself, or of a
//! classification — skips rather than fails, the same convention
//! `fuses_that_must_agree.rs` uses throughout.
//!
//! # The four-quadrant floor
//!
//! `docs/decisions.md` §62 is about a sample set that could only ever land on
//! one side of a question — a test that would pass whether or not the code
//! under it was right. A generator that only ever drew points inside both
//! solids, say, would pass every assertion below whatever the fuse actually
//! did with the other three cases. So every surviving point is tallied by
//! which of the four combinations it landed in — (in A, in B), (in A, out B),
//! (out A, in B), (out A, out B) — and each test fails outright if any of the
//! four never happened across the whole run.
//!
//! # What is covered, and what is not
//!
//! Only the lattice family and the flush family run here. Both are exactly
//! representable — every plane axis-aligned with a whole-number offset, every
//! corner the fuse invents the meet of three of them — so `classify_point`
//! never comes back [`Containment::Undecidable`] on them, which keeps this
//! file's own correctness simple to reason about. The turned, skew and curved
//! families are a follow-up: a turned or skewed pair's planes are not
//! whole-number offsets, so a sample point can no longer be guaranteed clear
//! of every wall by construction, and a curved face needs
//! [`SolidView::box_around_it`]'s own curved-face reasoning re-read before the
//! grid trick above can be trusted near it.

mod cones;
mod pairs;

use std::cell::Cell;

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, TestCaseError, TestRunner};
use rcad_geom::Point;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Model, Rule, SolidKey};

use pairs::{Configuration, a_flush_pair, a_lattice_pair};

/// How many configurations each test draws.
///
/// Each configuration costs up to three fuses and, on a wide box, a few
/// hundred sample points each classified against A, B and the answer — so this
/// is an order of magnitude below `fuses_that_must_agree.rs`'s own [`u32`]
/// case counts, which cost one fuse or a handful. Large enough that the
/// four-quadrant floor below clears comfortably; see each test's own run for
/// the tally it actually produced.
const CASES: u32 = 96;

#[test]
fn a_lattice_pairs_fuse_means_what_its_rule_says() {
    let tally = over(CASES, a_lattice_pair(), checks_the_rules_meaning);
    tally.report_and_require_all_four_combinations("lattice pairs");
}

#[test]
fn a_flush_pairs_fuse_means_what_its_rule_says() {
    let tally = over(CASES, a_flush_pair(), checks_the_rules_meaning);
    tally.report_and_require_all_four_combinations("pairs sharing coordinates");
}

/// Fuse `A` and `B` under every rule that exists today, sample the box around
/// both, and check each surviving point against the rule's own truth table.
fn checks_the_rules_meaning(
    configuration: Configuration,
    tally: &QuadrantTally,
) -> Result<(), TestCaseError> {
    let (model, keys) = configuration.build();
    let (a, b) = (keys[0], keys[1]);

    let Some((low, high)) = sampling_box(&model, a, b) else {
        return Ok(());
    };

    let mut answers = Vec::new();
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        let mut fused_model = model.clone();
        let mut log = ProvenanceLog::new();
        if let Ok(fused) = fused_model.fuse(&[a], &[b], rule, &Policy::certified(), &mut log) {
            answers.push((rule, fused_model, fused.solids().to_vec()));
        }
    }
    if answers.is_empty() {
        return Ok(());
    }

    for point in sample_grid(low, high) {
        let Some(in_a) = decisively_in(&model, &[a], point) else {
            continue;
        };
        let Some(in_b) = decisively_in(&model, &[b], point) else {
            continue;
        };
        tally.saw(in_a, in_b);

        for (rule, answer_model, answer_keys) in &answers {
            let Some(in_answer) = decisively_in(answer_model, answer_keys, point) else {
                continue;
            };
            let expected = match rule {
                Rule::Union => in_a || in_b,
                Rule::Intersection => in_a && in_b,
                Rule::Difference => in_a && !in_b,
                _ => panic!("a fuse rule this test does not know how to check: {rule:?}"),
            };
            prop_assert_eq!(
                in_answer,
                expected,
                "{rule:?} of A and B at {point:?}: should be {expected} (A={in_a}, B={in_b}), \
                 the answer said {in_answer}",
                rule = rule,
                point = point,
                expected = expected,
                in_a = in_a,
                in_b = in_b,
                in_answer = in_answer
            );
        }
    }
    Ok(())
}

/// Whether `point` is decisively inside the union of `keys`' solids, or `None`
/// if any of them could not say.
///
/// `keys` may be empty — a difference whose answer is nothing at all — and
/// that correctly hands back `Some(false)` without a special case: the loop
/// below never runs and no key ever said "inside".
fn decisively_in(model: &Model, keys: &[SolidKey], point: Point) -> Option<bool> {
    let mut any_inside = false;
    for &key in keys {
        let solid = model.solid(key).expect("this model made the key");
        match solid.classify_point(point, &Policy::certified(), &mut ProvenanceLog::new()) {
            Containment::Inside => any_inside = true,
            Containment::Outside => {}
            Containment::OnBoundary
            | Containment::Undecidable { .. }
            | Containment::CannotFollowAFacesTrimming { .. }
            | Containment::CannotFireARayAtAFacesSurface { .. } => return None,
            // `Containment` is `#[non_exhaustive]`; every variant that exists
            // today is named above, and this is only here for one that does not
            // exist yet.
            _ => return None,
        }
    }
    Some(any_inside)
}

/// The box around both `a` and `b`, or `None` if either has no faces to box.
fn sampling_box(model: &Model, a: SolidKey, b: SolidKey) -> Option<(Point, Point)> {
    let (a_low, a_high) = model
        .solid(a)
        .expect("this model made the key")
        .box_around_it()?;
    let (b_low, b_high) = model
        .solid(b)
        .expect("this model made the key")
        .box_around_it()?;
    Some((
        Point::new(
            a_low.x().min(b_low.x()),
            a_low.y().min(b_low.y()),
            a_low.z().min(b_low.z()),
        ),
        Point::new(
            a_high.x().max(b_high.x()),
            a_high.y().max(b_high.y()),
            a_high.z().max(b_high.z()),
        ),
    ))
}

/// How many sample coordinates [`axis_samples`] hands back along one axis, at
/// most.
///
/// The grid trick guarantees a coordinate never lands on a whole number, but
/// not how many candidates there are: two solids five units apart, each up to
/// five units wide, make a box up to fifteen units on a side, which is well
/// over a hundred eighths. This bounds the grid to a few hundred points a
/// configuration rather than the low millions a wide box would otherwise cost,
/// while still landing in every quadrant the box has room for.
const SAMPLES_PER_AXIS: usize = 8;

/// Sample points on a fine offset grid across `[low, high]`, one axis at a
/// time and then combined.
fn sample_grid(low: Point, high: Point) -> Vec<Point> {
    let xs = axis_samples(low.x(), high.x());
    let ys = axis_samples(low.y(), high.y());
    let zs = axis_samples(low.z(), high.z());
    let mut points = Vec::with_capacity(xs.len() * ys.len() * zs.len());
    for &x in &xs {
        for &y in &ys {
            for &z in &zs {
                points.push(Point::new(x, y, z));
            }
        }
    }
    points
}

/// Sample coordinates between `lo` and `hi`, each an odd multiple of a
/// sixteenth: `n * 0.125 + 0.0625` for a whole number `n`.
///
/// That value's fractional part is always one of `0.0625, 0.1875, …, 0.9375` —
/// never zero, whatever `lo`, `hi` or `n` are — so it never lands on the
/// whole-number plane a lattice-family wall sits on, and neither does adding
/// the whole-number `lo` to it. [`SAMPLES_PER_AXIS`] bounds how many of them
/// come back by striding through the candidates rather than taking every one.
fn axis_samples(lo: f64, hi: f64) -> Vec<f64> {
    let candidates = (((hi - lo) / 0.125).floor() as i64 + 1).max(1);
    let stride = (candidates / SAMPLES_PER_AXIS as i64).max(1);
    let mut coords = Vec::new();
    let mut n = 0i64;
    while n < candidates {
        let v = lo + n as f64 * 0.125 + 0.0625;
        if v <= hi {
            coords.push(v);
        }
        n += stride;
    }
    coords
}

/// How many surviving sample points landed in each of the four combinations of
/// (inside/outside A) and (inside/outside B).
///
/// `docs/decisions.md` §62's guard, specialised to this file: a sample set that
/// only ever drew points from one or two of the four combinations would pass
/// every assertion above regardless of whether the fuse was right about the
/// other combinations, so a pass is only worth what [`Self::saw`] actually saw.
#[derive(Default)]
struct QuadrantTally {
    in_a_in_b: Cell<usize>,
    in_a_out_b: Cell<usize>,
    out_a_in_b: Cell<usize>,
    out_a_out_b: Cell<usize>,
}

impl QuadrantTally {
    fn saw(&self, in_a: bool, in_b: bool) {
        let cell = match (in_a, in_b) {
            (true, true) => &self.in_a_in_b,
            (true, false) => &self.in_a_out_b,
            (false, true) => &self.out_a_in_b,
            (false, false) => &self.out_a_out_b,
        };
        cell.set(cell.get() + 1);
    }

    /// Print the tally, and fail unless every one of the four combinations
    /// happened at least once across the whole run.
    fn report_and_require_all_four_combinations(&self, what: &str) {
        println!(
            "{what}: (in A, in B) {}, (in A, out B) {}, (out A, in B) {}, (out A, out B) {}",
            self.in_a_in_b.get(),
            self.in_a_out_b.get(),
            self.out_a_in_b.get(),
            self.out_a_out_b.get(),
        );
        assert!(
            self.in_a_in_b.get() > 0,
            "{what}: no surviving point ever landed inside both A and B"
        );
        assert!(
            self.in_a_out_b.get() > 0,
            "{what}: no surviving point ever landed inside A and outside B"
        );
        assert!(
            self.out_a_in_b.get() > 0,
            "{what}: no surviving point ever landed outside A and inside B"
        );
        assert!(
            self.out_a_out_b.get() > 0,
            "{what}: no surviving point ever landed outside both A and B"
        );
    }
}

/// Run `property` over `cases` configurations from the seed `PROPTEST_RNG_SEED`
/// names — `.cargo/config.toml`'s, unless a soak names its own (§339) — and
/// hand back what it saw.
fn over<T: core::fmt::Debug>(
    cases: u32,
    strategy: impl Strategy<Value = T>,
    property: impl Fn(T, &QuadrantTally) -> Result<(), TestCaseError>,
) -> QuadrantTally {
    let tally = QuadrantTally::default();
    let mut runner = TestRunner::new(Config {
        cases,
        source_file: Some(file!()),
        // Said out loud rather than left to the default, which looks for the
        // crate root above the test file, does not find one, and warns on
        // every run — the same reason `fuses_that_must_agree.rs` sets it.
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
            "proptest-regressions",
        ))),
        ..Config::default()
    });
    if let Err(why) = runner.run(&strategy, |value| property(value, &tally)) {
        panic!("{why}");
    }
    tally
}
