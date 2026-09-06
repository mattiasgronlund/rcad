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
//! **The answer is that it buys nothing, and the reason is not the one it
//! looks like.** The corpus does leave undecided decisions behind — every one
//! of them on a run that was refused — and on the widest budget here they sit
//! four thousand million times inside it and are refused anyway.
//! [`assert_the_budget_was_not_what_stopped_them`] is that measurement, and
//! says where the policy stops being threaded.
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
//!   present — some files import, some do not — and that the log-reading half
//!   of [`WhatARunSaw`] is live on the corpus and not only in the controls.
//!
//! Three tolerant budgets, the widest a ten-thousand-millionfold more slack
//! than any caller would set, so a null result is not a null result about one
//! budget.
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
/// A square with a spike `1e-150` from the origin, seen from corners a whole
/// unit away — the exact level declines to multiply numbers that small and the
/// two cheap levels cannot separate the products, so `orient2d` comes back
/// undecided. A certified policy stops there; a tolerant one reads the
/// enclosure as zero and files a [`Cause::UndecidableResolved`] record.
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
    let tiny = 1e-150;
    let corners = vec![
        [0.0, 0.0],
        [tiny, 3.0 * tiny],
        [2.0 * tiny, 0.0],
        [1.0, -1.0],
        [1.0, 1.0],
        [-1.0, 1.0],
        [-1.0, -1.0],
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
        "nothing in f64 places that spike, and a certified build said {:?}",
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

/// The measurement: every file in both corpora, under a certified policy and
/// under three tolerant budgets, compared whole.
#[test]
fn no_tolerant_budget_reads_a_corpus_file_a_certified_policy_refuses() {
    let root = corpus();
    let mut differences: Vec<String> = Vec::new();
    let mut compared = 0;
    let mut records_seen_on_the_corpus = 0;
    let mut left_undecided_on_the_widest_budget = 0;
    let mut worst_wobble_on_the_widest_budget = 0.0_f64;

    for source in ["nist", "kicad"] {
        let files = files_under(&root.join(source));
        assert!(
            !files.is_empty(),
            "{source} holds no STEP files, so this test would pass by doing nothing"
        );
        let mut certified = WhatACorpusCameTo::default();
        let mut tolerant: BTreeMap<&str, WhatACorpusCameTo> = BTreeMap::new();

        for path in &files {
            let bytes = fs::read(path).expect("a file the walk just listed");
            for (what, working_size) in BUDGETS {
                let strict = import_sweep_and_mesh(&bytes, &Policy::certified(), working_size);
                let loose =
                    import_sweep_and_mesh(&bytes, &Policy::tolerant(working_size), working_size);
                if what == BUDGETS[0].0 {
                    certified.add(&strict);
                }
                tolerant.entry(what).or_default().add(&loose);
                if let Some(said) = how_they_differ(&strict, &loose) {
                    differences.push(format!(
                        "{source}/{} under tolerant({working_size}, {what}): {said}",
                        name_of(path)
                    ));
                }
                compared += 1;
            }
        }

        println!("=== {source}: {} files", files.len());
        println!("    certified:                    {certified}");
        for (what, working_size) in BUDGETS {
            println!(
                "    tolerant({working_size:e}, {what}), budget {:e} mm: {}",
                Policy::tolerant(working_size).max_wobble(),
                tolerant[what]
            );
        }
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
        records_seen_on_the_corpus += certified.records();

        let widest = &tolerant[BUDGETS[2].0];
        left_undecided_on_the_widest_budget += widest.records_on_a_refusal.values().sum::<usize>();
        worst_wobble_on_the_widest_budget =
            worst_wobble_on_the_widest_budget.max(widest.worst_wobble);
    }

    assert_the_budget_was_not_what_stopped_them(
        left_undecided_on_the_widest_budget,
        worst_wobble_on_the_widest_budget,
    );
    assert!(
        records_seen_on_the_corpus > 0,
        "not one record came off either corpus under either policy, so the half of this \
         comparison that reads a provenance log never ran on a real file and a difference in \
         it would have gone unnoticed"
    );
    assert!(
        compared >= 3 * 367,
        "only {compared} pairs of runs were compared, which is too few for 367 files at three \
         budgets"
    );
    println!(
        "{compared} pairs of runs compared, {} different",
        differences.len()
    );
    assert!(
        differences.is_empty(),
        "{} of {compared} corpus runs came out differently under a tolerant policy: \
         {differences:#?}",
        differences.len()
    );
}

/// **Why tolerance bought nothing here is not that the budget was too small.**
///
/// The corpus leaves records behind, all of them on runs that were refused, and
/// on the widest budget every one of them has a wobble the budget covers many
/// times over. A tolerant caller who has said it will accept a point a
/// hundredth of a millimetre out of place is refused over a decision worth
/// picometres, and the record it gets carries no approximation — nothing
/// resolved it, and the outcome is the same one the certified run got.
///
/// **A gap and not a limit** (`docs/decisions.md` §16.10): the answer is inside
/// `f64`, inside the budget, and the arithmetic did not reach it. The reason is
/// in the signature of `which_side_of` in `rcad-geom/src/walk.rs`, which pushes
/// its own [`Record`](rcad_predicates::Record) and hands back
/// `Decision::Undecidable` without taking a `&Policy` at all — so
/// `resolve_under_policy`, the one place a budget is ever spent, is never
/// reached from there. The families of fix considered: a wider budget (measured
/// here across a ten-thousand-millionfold range, and none of them changed a
/// single outcome), a better filter (would move which decisions are undecided,
/// not what a policy does with the ones that are left), and threading the
/// policy through. Only the third one can change anything, and it is outside
/// this file to make.
///
/// This assertion is what would go red when it is made, and that would be the
/// fix arriving rather than a regression.
fn assert_the_budget_was_not_what_stopped_them(left_undecided: usize, worst_wobble: f64) {
    let budget = Policy::tolerant(BUDGETS[2].1).max_wobble();
    assert!(
        left_undecided > 0,
        "no decision anywhere in either corpus was left undecided, so this says nothing about \
         why a budget did not settle one"
    );
    assert!(
        worst_wobble > 0.0 && worst_wobble < budget,
        "{left_undecided} decisions were left undecided on a budget of {budget:e} mm and the \
         worst of them was only {worst_wobble:e} mm out of place, or it was not: the whole \
         point of this measurement is that the budget covers them and they were refused anyway"
    );
    println!(
        "{left_undecided} decisions refused on a {budget:e} mm budget, worst {worst_wobble:e} mm \
         — a factor of {:.0} inside it",
        budget / worst_wobble
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
                        match model.solid(*key).mesh(deflection, policy, &mut log) {
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

    fn records(&self) -> usize {
        self.records_coming_through.values().sum::<usize>()
            + self.records_on_a_refusal.values().sum::<usize>()
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
