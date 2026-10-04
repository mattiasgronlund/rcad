//! Does [`Policy::tolerant`] read, sweep or mesh a real STEP file that
//! [`Policy::certified`] refuses?
//!
//! A downstream viewport measured, over its own pin of these two corpora, that
//! a tolerant policy drew exactly the solids a certified one drew and that no
//! solid carried a provenance record either way — "tolerance bought nothing".
//! That is a claim about *this* kernel, so it is measured here, in this tree,
//! at this commit, over the whole path a viewport takes: read the file, sweep
//! the model, mesh every solid in it.
//!
//! `crates/rcad-topo/tests/what_a_tolerant_policy_costs_a_mesh.rs` is the same
//! question asked of twelve hand-built bars, where the answer is that tolerance
//! costs the caller a worse account of a refusal it gets anyway. This file asks
//! it of files nobody here wrote.
//!
//! **On this corpus the answer is that it buys nothing at all** — not a file,
//! and no longer even an account. It used to buy an account: the corpus left
//! undecided decisions behind, every one of them a half-space on a run of two
//! equal-radius cylinders whose axes cross, reached through the quartic
//! crossing route, and a budget wide enough resolved each of them. `walk.rs`
//! now answers that family in closed form, so the site those decisions came
//! from is decided before a policy is ever consulted and there is nothing left
//! for a budget to be spent on. Both lists this test builds are empty, and
//! both are asserted empty rather than described.
//!
//! **What a budget does with a decision that is still undecided is measured
//! elsewhere**, on geometry built by hand so that it cannot be emptied out from
//! under the measurement:
//! `crates/rcad-geom/tests/whether_a_tolerant_policy_reaches_the_half_spaces_on_a_run.rs`.
//! That is where the floor that used to stand here now stands.
//!
//! # A null result is what a broken measurement gives too
//!
//! So nothing below is allowed to pass by looking for something and not finding
//! it.
//!
//! - **No assertion names a predicate, a cause or a refusal and checks it is
//!   absent.** Every run is boiled down to a [`WhatARunSaw`] built out of what
//!   the run produced — solid count, triangle count, refusal text, and every
//!   record's own `predicate()` and cause — and the two runs are compared
//!   whole. A renamed constant cannot make that comparison quieter; it can only
//!   make it noisier.
//! - **Two controls** run that same comparison over inputs the two policies
//!   *must* disagree about, and fail if it reports them equal:
//!   - [`a_step_file_the_two_policies_disagree_about`] goes through the very
//!     function a corpus file goes through, with a block whose dynamic range is
//!     too wide for `f64`. Certified fails the sweep; tolerant warns, carries
//!     on, and leaves records behind.
//!   - [`a_shape_whose_corners_f64_cannot_place`] is the other half a corpus
//!     file could differ by — a predicate that could not decide, resolved to
//!     zero by the tolerant budget and logged as
//!     [`Cause::UndecidableResolved`].
//! - The same block, at ordinary proportions, must compare **equal**. A
//!   comparison that separates any two runs separates the corpus's too, and the
//!   corpus result would mean nothing.
//! - The corpus sweep itself asserts that both halves of the outcome are
//!   present — some files import, some do not — so the comparison is run over
//!   files that were accepted and over files that were refused.
//!
//! The log-reading half of [`WhatARunSaw`] is live **only in the controls**
//! now, because the corpus leaves no record behind under any policy. That is
//! the load the two controls carry: they are the whole of the evidence that a
//! difference in the records would have been seen, and neither may be deleted
//! on the grounds that the corpus does not exercise it.
//!
//! Three tolerant budgets, each in a test of its own, the widest a
//! ten-thousand-millionfold more slack than any caller would set, so a null
//! result is not a null result about one budget.
//!
//! Neither corpus is in the tree; point `RCAD_STEP_CORPUS` at a directory
//! holding `nist/` and `kicad/` and this runs. Without it this **fails** rather
//! than skips — green has to mean the corpus ran (`docs/decisions.md` §44.10).
//! The two corpora are reported apart and never added together (§17.7).

mod written_by_hand;

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};

use rcad_geom::Deflection;
use rcad_predicates::{Cause, Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::ProfileBuilder;
use written_by_hand::a_block;

/// The working sizes the tolerant runs are given, in millimetres, and what
/// each is meant to be.
///
/// The corpus parts are a few millimetres to a few tens across, so 100 mm is
/// already the generous reading of "the working size". The other two are there
/// so that a null result cannot be answered with "you did not give tolerance
/// enough room": at `1e12` the budget is `64 · f64::EPSILON · 1e12`, about a
/// **hundredth of a millimetre** of slop allowed on a part two millimetres
/// wide.
const BUDGETS: [(&str, f64); 3] = [
    ("the part", 100.0),
    ("ten thousand parts", 1e6),
    ("absurd", 1e12),
];

/// **The control that makes the corpus sweep worth reading**, and it goes
/// through [`import_sweep_and_mesh`] exactly as a corpus file does.
///
/// A block one millimetre by one by a **picometre**. Nothing about its
/// coordinates is hard — they are 0, 1 and `1e-12`, every one of them exactly
/// an `f64` — but the largest of them over the smallest feature is `1e12`, past
/// the `1e11` where `f64` runs out of digits to hold both
/// (`docs/decisions.md` §3.5). A certified policy fails the sweep for it and a
/// tolerant one warns and carries on, so the two runs must differ and
/// [`how_they_differ`] has to say so.
///
/// The same block at ordinary proportions must compare **equal**, or this is
/// not a control: a comparison that separates any two runs separates the
/// corpus's runs too, and the corpus result would mean nothing.
#[test]
fn a_step_file_the_two_policies_disagree_about() {
    let sound = a_block([2.0, 4.0, 8.0], ".MILLI.");
    let certified = import_sweep_and_mesh(sound.as_bytes(), &Policy::certified(), 100.0);
    assert!(
        certified.outcome.is_ok(),
        "an ordinary block has to import, sweep and mesh, or this control is about something \
         else: {:?}",
        certified.outcome
    );
    assert_eq!(
        how_they_differ(
            &certified,
            &import_sweep_and_mesh(sound.as_bytes(), &Policy::tolerant(100.0), 100.0)
        ),
        None,
        "the two policies must agree about an ordinary block, or this comparison separates \
         everything and separating the corpus's runs would prove nothing"
    );

    let too_wide = a_block([1.0, 1.0, 1e-12], ".MILLI.");
    let certified = import_sweep_and_mesh(too_wide.as_bytes(), &Policy::certified(), 100.0);
    let tolerant = import_sweep_and_mesh(too_wide.as_bytes(), &Policy::tolerant(100.0), 100.0);

    assert!(
        certified.outcome.is_err(),
        "a range of 1e12 is past what f64 holds and a certified sweep refuses it, and this one \
         said {:?}",
        certified.outcome
    );
    assert!(
        tolerant.outcome.is_ok(),
        "a tolerant policy warns about that range rather than refusing it, and this one said \
         {:?}",
        tolerant.outcome
    );
    assert!(
        certified.records.is_empty(),
        "a refusal is not a record: {:?}",
        certified.records
    );
    assert!(
        !tolerant.records.is_empty(),
        "the warning the tolerant run was given is what a corpus file would have carried, and \
         there is none"
    );
    assert!(tolerant.worst_wobble > 0.0);

    let difference = how_they_differ(&certified, &tolerant)
        .expect("THE COMPARISON IS BLIND: two runs that must differ were reported the same");
    println!("control, a block 1e12 wide: {difference}");
}

/// The other half a corpus file could differ by: a predicate that could not
/// decide.
///
/// A square with one corner cut off, the cut running from `(1, -t)` through
/// `(0.5, 0.5)` to `(t, 1)` with `t = 1e-318`, so that the turn in the middle
/// is exactly `-t^2`. The two cheap levels cannot separate the products the
/// bend's turn is the difference of, and the exact level holds `1 * 1` and
/// `t * t` in one determinant: every power of two that keeps `1 * 1` in range
/// leaves the whole turn below what `t * t` loses falling off the bottom
/// (`crates/rcad-topo/tests/when_it_cannot_tell.rs` tries them all). So
/// `orient2d` comes back undecided. A certified policy stops there; a tolerant
/// one reads the enclosure as zero and files a [`Cause::UndecidableResolved`]
/// record.
///
/// **Not a STEP file, and that is this lane's finding rather than a hole in
/// it.** Every coordinate the reader takes is an exact `f64` written in the
/// text, and the exact level of the cascade decides those; the corpus sweep
/// below is what measures whether any real file gets past that. What this test
/// is for is the comparison: it proves [`WhatARunSaw`] and [`how_they_differ`]
/// see a difference in the **records**, which the block above does not exercise
/// at the predicate level.
#[test]
fn a_shape_whose_corners_f64_cannot_place() {
    let bend = 1e-318;
    let corners = vec![
        [-1.0, -1.0],
        [1.0, -1.0],
        [1.0, -bend],
        [0.5, 0.5],
        [bend, 1.0],
        [-1.0, 1.0],
    ];

    let mut runs = Vec::new();
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        let mut log = ProvenanceLog::new();
        let outcome = match ProfileBuilder::new(corners.clone()).build(&policy, &mut log) {
            Ok(profile) => {
                assert!(
                    profile.approximation().is_some(),
                    "a profile built out of corners nothing could place is approximate"
                );
                Ok((1, 0))
            }
            Err(why) => Err(first_line(&why)),
        };
        runs.push(WhatARunSaw::of(outcome, &log, 100.0));
    }
    let [certified, tolerant] = <[WhatARunSaw; 2]>::try_from(runs).expect("two policies, two runs");

    assert!(
        certified.outcome.is_err(),
        "nothing in f64 sees which way that cut bends, and a certified build said {:?}",
        certified.outcome
    );
    assert!(
        tolerant.outcome.is_ok(),
        "a tolerant policy reads those enclosures as zero and builds the profile, and this one \
         said {:?}",
        tolerant.outcome
    );
    assert!(
        certified.records.is_empty(),
        "a certified run that refused logged nothing: {:?}",
        certified.records
    );
    let resolved_under_a_budget = format!("{:?}", Cause::UndecidableResolved);
    let resolved: usize = tolerant
        .records
        .iter()
        .filter(|((_, cause), _)| *cause == resolved_under_a_budget)
        .map(|(_, count)| count)
        .sum();
    assert!(
        resolved > 0,
        "a tolerant policy that accepted this resolved something: {:?}",
        tolerant.records
    );

    let difference = how_they_differ(&certified, &tolerant)
        .expect("THE COMPARISON IS BLIND: a resolved predicate is a difference and it missed it");
    println!("control, a corner f64 cannot place: {difference}");
}

#[test]
fn a_tolerant_budget_the_size_of_the_part_reads_no_corpus_file_a_certified_policy_refuses() {
    no_corpus_file_a_certified_policy_refuses_is_read_under(BUDGETS[0]);
}

#[test]
fn a_tolerant_budget_ten_thousand_parts_wide_reads_no_corpus_file_a_certified_policy_refuses() {
    no_corpus_file_a_certified_policy_refuses_is_read_under(BUDGETS[1]);
}

#[test]
fn an_absurd_tolerant_budget_reads_no_corpus_file_a_certified_policy_refuses() {
    no_corpus_file_a_certified_policy_refuses_is_read_under(BUDGETS[2]);
}

/// The measurement: every file in both corpora, under a certified policy and
/// under one tolerant budget, compared whole. Each budget is a test of its own
/// so that no one test of the three sets the slow tier's wall
/// (`docs/decisions.md` §397.4).
///
/// Two lists come out of it and they say different things. A run whose
/// **outcome** moved is a file a tolerant policy read that a certified one
/// refused. A run whose **records** moved is a decision the budget was spent
/// on. Both are asserted empty, because on this corpus a tolerant policy is
/// told exactly what a certified one is told; the two controls above are what
/// stop either emptiness being the comparison having gone blind.
fn no_corpus_file_a_certified_policy_refuses_is_read_under((what, working_size): (&str, f64)) {
    let root = corpus();
    let mut outcomes_that_moved: Vec<String> = Vec::new();
    let mut records_that_moved: Vec<String> = Vec::new();
    let mut compared = 0;

    for source in ["nist", "kicad"] {
        let files = files_under(&root.join(source));
        assert!(
            !files.is_empty(),
            "{source} holds no STEP files, so this test would pass by doing nothing"
        );
        let mut certified = WhatACorpusCameTo::default();
        let mut tolerant = WhatACorpusCameTo::default();

        for path in &files {
            let bytes = fs::read(path).expect("a file the walk just listed");
            let strict = import_sweep_and_mesh(&bytes, &Policy::certified(), working_size);
            let loose =
                import_sweep_and_mesh(&bytes, &Policy::tolerant(working_size), working_size);
            certified.add(&strict);
            tolerant.add(&loose);
            if let Some(said) = how_they_differ(&strict, &loose) {
                let said = format!(
                    "{source}/{} under tolerant({working_size}, {what}): {said}",
                    name_of(path)
                );
                if strict.outcome == loose.outcome {
                    records_that_moved.push(format!("{said}; both came to {:?}", strict.outcome));
                } else {
                    outcomes_that_moved.push(said);
                }
            }
            compared += 1;
        }

        println!("=== {source}: {} files", files.len());
        println!("    certified:                    {certified}");
        println!(
            "    tolerant({working_size:e}, {what}), budget {:e} mm: {tolerant}",
            Policy::tolerant(working_size).max_wobble(),
        );
        assert!(
            certified.accepted > 0 && certified.accepted < files.len(),
            "{} of {} {source} files came all the way through under a certified policy, and \
             both halves have to be there or this is not measuring a difference between \
             accepting and refusing",
            certified.accepted,
            files.len()
        );
        assert!(
            certified.records_coming_through.is_empty(),
            "a fully certified operation records nothing (docs/decisions.md §5), and the \
             {source} files that came all the way through under a certified policy left \
             {:?} behind",
            certified.records_coming_through
        );
    }

    assert!(
        compared >= 367,
        "only {compared} pairs of runs were compared under tolerant({working_size}, {what}), \
         which is too few for 367 files"
    );
    println!(
        "{compared} pairs of runs compared, {} whose outcome moved, {} whose records moved",
        outcomes_that_moved.len(),
        records_that_moved.len()
    );
    assert!(
        outcomes_that_moved.is_empty(),
        "{} of {compared} corpus runs read, swept or meshed differently under a tolerant \
         policy: {outcomes_that_moved:#?}",
        outcomes_that_moved.len()
    );
    assert!(
        records_that_moved.is_empty(),
        "{} of {compared} corpus runs were told something different under a tolerant policy, so \
         a budget is being spent somewhere on this corpus and this file no longer describes it: \
         {records_that_moved:#?}",
        records_that_moved.len()
    );
}

/// Everything one run under one policy came to, in a form two runs can be
/// compared by.
///
/// Built out of what the run produced and never out of what it was searched
/// for. `records` keys on each record's own `predicate()` and the debug form of
/// its cause, so a record under a name this file has never heard of still shows
/// up as a difference rather than being filtered away.
#[derive(Clone, Debug, PartialEq)]
struct WhatARunSaw {
    /// `Ok((solids, triangles))` when the file imported, swept and meshed;
    /// `Err(why)` at the first of those that refused.
    outcome: Result<(usize, usize), String>,
    /// `(predicate, cause)` counted. A record with no approximation on it — one
    /// given up on rather than resolved — files under `"gave up"`.
    records: BTreeMap<(String, String), usize>,
    /// The furthest out of place any of those decisions could have put a point.
    worst_wobble: f64,
}

impl WhatARunSaw {
    fn of(outcome: Result<(usize, usize), String>, log: &ProvenanceLog, working_size: f64) -> Self {
        let mut records: BTreeMap<(String, String), usize> = BTreeMap::new();
        for record in log.records() {
            let cause = record.approximation().map_or_else(
                || "gave up".to_owned(),
                |approximation| format!("{:?}", approximation.cause()),
            );
            *records
                .entry((record.predicate().to_owned(), cause))
                .or_default() += 1;
        }
        Self {
            outcome,
            records,
            worst_wobble: log.worst_wobble(working_size),
        }
    }
}

/// Read a STEP file, sweep the model it makes and mesh every solid in it, all
/// three under `policy`.
///
/// The whole path a viewport takes, because the claim being checked is about
/// what a viewport draws. Each stage matters on its own: the reader asks
/// predicates about the corners it takes, the sweep is where the dynamic-range
/// check reads the policy without asking a predicate anything, and the mesher
/// is where `rcad-geom`'s `room_left_over` waves a tolerant caller through.
fn import_sweep_and_mesh(bytes: &[u8], policy: &Policy, working_size: f64) -> WhatARunSaw {
    let file = StepFile::read(bytes).expect("every file this runs on parses");
    let mut log = ProvenanceLog::new();
    let outcome = match Import::read(&file, policy, &mut log) {
        Ok(import) => {
            let model = import.model();
            let deflection = Deflection::for_scale(model.scale());
            match model.validate(policy, &mut log) {
                Ok(()) => {
                    let mut triangles = 0;
                    let mut refused = None;
                    for (_, key) in import.solids() {
                        match model
                            .solid(*key)
                            .expect("this model made the key")
                            .mesh(deflection, policy, &mut log)
                        {
                            Ok(mesh) => triangles += mesh.triangles().len(),
                            Err(why) => {
                                refused = Some(format!("the mesher: {}", first_line(&why)));
                                break;
                            }
                        }
                    }
                    refused.map_or(Ok((import.solids().len(), triangles)), Err)
                }
                Err(why) => Err(format!("the sweep: {}", first_line(&why))),
            }
        }
        Err(why) => Err(first_line(&why)),
    };
    WhatARunSaw::of(outcome, &log, working_size)
}

fn first_line(why: &impl Display) -> String {
    let text = why.to_string();
    text.lines().next().unwrap_or(&text).to_owned()
}

/// What is different between two runs, or `None` when nothing is.
///
/// The whole of the comparison this file rests on, and deliberately one
/// function so the controls and the corpus sweep cannot drift apart: a control
/// proving this sees a difference proves it for the corpus too.
fn how_they_differ(certified: &WhatARunSaw, tolerant: &WhatARunSaw) -> Option<String> {
    if certified == tolerant {
        return None;
    }
    let mut said = Vec::new();
    if certified.outcome != tolerant.outcome {
        said.push(format!(
            "certified said {:?} and tolerant said {:?}",
            certified.outcome, tolerant.outcome
        ));
    }
    if certified.records != tolerant.records {
        said.push(format!(
            "certified logged {:?} and tolerant logged {:?}",
            certified.records, tolerant.records
        ));
    }
    if certified.worst_wobble != tolerant.worst_wobble {
        said.push(format!(
            "certified wobbled by {} and tolerant by {}",
            certified.worst_wobble, tolerant.worst_wobble
        ));
    }
    assert!(
        !said.is_empty(),
        "two runs that are not equal have to differ by something this can name"
    );
    Some(said.join("; "))
}

/// What one whole corpus came to under one policy.
///
/// The two record buckets are kept apart because they answer different
/// questions. A record left behind by a run that was **refused** is the honest
/// account of why. A record left behind by a run that came all the way
/// **through** is a claim that the answer it handed back was not fully
/// certified — and under a certified policy there must be none of those at all
/// (`docs/decisions.md` §5).
#[derive(Default)]
struct WhatACorpusCameTo {
    accepted: usize,
    solids: usize,
    triangles: usize,
    records_coming_through: BTreeMap<(String, String), usize>,
    records_on_a_refusal: BTreeMap<(String, String), usize>,
    /// The furthest out of place any one decision anywhere in the corpus could
    /// have put a point. Read against [`Policy::max_wobble`] it says whether
    /// the budget was the thing that stopped a run being resolved.
    worst_wobble: f64,
    refusals: BTreeSet<String>,
}

impl WhatACorpusCameTo {
    fn add(&mut self, run: &WhatARunSaw) {
        let bucket = match &run.outcome {
            Ok((solids, triangles)) => {
                self.accepted += 1;
                self.solids += solids;
                self.triangles += triangles;
                &mut self.records_coming_through
            }
            Err(why) => {
                self.refusals.insert(why.clone());
                &mut self.records_on_a_refusal
            }
        };
        for (what, count) in &run.records {
            *bucket.entry(what.clone()).or_default() += count;
        }
        self.worst_wobble = self.worst_wobble.max(run.worst_wobble);
    }
}

impl Display for WhatACorpusCameTo {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            out,
            "{} through, {} solids, {} triangles, {} kinds of refusal, worst wobble {:e}; \
             coming through {:?}, on a refusal {:?}",
            self.accepted,
            self.solids,
            self.triangles,
            self.refusals.len(),
            self.worst_wobble,
            self.records_coming_through,
            self.records_on_a_refusal
        )
    }
}

fn corpus() -> PathBuf {
    let root = env::var_os("RCAD_STEP_CORPUS").unwrap_or_else(|| {
        panic!(
            "RCAD_STEP_CORPUS is not set. Point it at a directory holding nist/ and kicad/ (see \
             this file's header) — this test does not skip on a missing corpus."
        )
    });
    let root = PathBuf::from(root);
    assert!(
        root.is_dir(),
        "RCAD_STEP_CORPUS points at {}, which is not a directory",
        root.display()
    );
    root
}

fn files_under(directory: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = fs::read_dir(directory) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(files_under(&path));
            continue;
        }
        let is_step = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                extension.eq_ignore_ascii_case("stp") || extension.eq_ignore_ascii_case("step")
            });
        if is_step {
            found.push(path);
        }
    }
    found.sort();
    found
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}
