//! §144.9: what the provenance log costs the nights that never read one.
//!
//! # The claim this is here to settle
//!
//! `ProvenanceLog`'s own doc comment says an empty log does not allocate, "so a
//! certified operation pays nothing for carrying one". True, and §144.9's
//! objection is that it optimises the path a fuzz run never takes: `Record` owns
//! a `Vec<Point>`, so every *recorded* decision allocates, and finding refusals
//! is the whole point of a fuzz target. The cost the design avoids is avoided
//! where it was never going to be paid and met in full where the machine runs
//! for eight hours.
//!
//! The fix on offer there is a stub `ProvenanceLog` behind a default-on feature
//! — the first `[features]` in this workspace, and a second build configuration
//! the gate would have to run or let rot. §144.9 refuses to buy that on an
//! argument: "logging is expensive here" is a claim, and §16.10 says a claim
//! owes a demonstration. **This is the demonstration.** It asserts nothing, the
//! way §12.5 and `what_a_marched_point_costs` say a human-read figure should;
//! it prints, and `docs/decisions.md` §162 records what it said on the day.
//!
//! # What is measured, and what the number means
//!
//! Two passes over each target's checked-in corpus — the closest thing on disk to
//! the input distribution a night sees — and, for the two surface targets, over a
//! 200 000-draw soak as well, because a corpus is coverage-minimized and a night
//! spends most of itself on mutations that never earn a place in one.
//!
//! - **fuzzing** is the draw run exactly as the fuzz target runs it, through the
//!   same shared harness file the target reaches into. Nothing is stubbed and no
//!   log is read, because no fuzz target has ever read one.
//! - **recording** is every record that draw made, built and kept and freed
//!   again — the `Vec<Point>` per record, the log's own growth, and dropping
//!   both. That is precisely the work a stub `push` would not do, so the share
//!   of the two is **the whole of what the feature could buy**, not an estimate
//!   of it.
//!
//! The share is an **upper bound on the saving and a lower bound on the cost**,
//! and both directions are worth saying out loud. Upper bound on the saving,
//! because a stub cannot give back less than the work it removes and the
//! predicates underneath it still run. Lower bound on the cost, because a
//! million live allocations push the rest of the kernel out of cache, and a
//! replay into a warm allocator does not charge for that. If the share comes
//! back at a tenth of a percent the second reading cannot rescue the feature; if
//! it comes back at a fifth of the run, the feature is worth its second build.
//!
//! **Memory is reported beside time because it is the reading that can fail a
//! night outright.** libFuzzer stops a target at `-rss_limit_mb`, 2048 by
//! default, and a single draw whose walk records a point at a time is the shape
//! that reaches it. A share of the wall clock says nothing about that.
//!
//! # Why this one bench reaches into three crates' test directories
//!
//! The workloads are the fuzz targets' own bodies:
//! `rcad-geom/tests/fuzzing/mod.rs` for the two surface targets,
//! `rcad-topo/tests/fuzzing/fuse.rs` for the fuse and
//! `rcad-step/tests/fuzzing/reader.rs` for the reader. Copying any of them would
//! measure a draw nobody fuzzes, so this reaches them with `#[path]` — the same
//! trick `fuzz/fuzz_targets/` uses to reach the same three files. `rcad-step` is
//! the home because it is the lowest crate that can see all three.
//!
//! **Only the writer is left out, and it is the one target that cannot record.**
//! `rcad-step/tests/fuzzing/writer.rs` names no `Policy` and no log, and every
//! door that records in this kernel takes a `&mut ProvenanceLog` — so a harness
//! that never mentions one cannot reach a record.
//!
//! The reader was left out on that same reasoning at first and it was wrong: it
//! imports and then sweeps, both through a log. Its row is here, and it comes
//! back at zero for a reason worth keeping apart from the writer's — none of its
//! 909 seeds imports at all (`reader.rs` says so: 135 of 907 parsed and none
//! imported), so no draw gets deep enough to record. **A measured zero and not an
//! assumed one**, which is the difference this paragraph exists to keep: the
//! writer's zero is a property of the code and the reader's is a property of
//! today's corpus, and a seed that imports would move it.
//!
//! That also leaves the reader's door the one seam no test can hold. The other
//! two are pinned by `a_fuzz_draw_hands_its_provenance_back_through_the_door.rs`
//! in `rcad-geom` and `rcad-topo`, which assert that the log comes back filled —
//! and there is no such assertion to make here while every draw records nothing.
//! `every_file_the_reader_fuzzer_kept_is_read_again.rs` still covers the body,
//! because the front door delegates to it; what is unheld is only the threading
//! of the log, and the day a seed imports is the day that becomes checkable.

#![forbid(unsafe_code)]

// `#[path]` and not `include!`, because all three files open with a module doc
// comment and an inner attribute, and neither can be pasted into another file.
#[path = "../../rcad-topo/tests/fuzzing/fuse.rs"]
mod fuse;
#[path = "../tests/fuzzing/reader.rs"]
mod reader;
#[path = "../../rcad-geom/tests/fuzzing/mod.rs"]
mod surfaces;

use std::hint::black_box;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use rcad_predicates::{Point, ProvenanceLog, Record};

/// How many pseudo-random byte strings the soak draws.
///
/// The same count, the same generator and the same seed as the soak in
/// `rcad-geom/tests/every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs`,
/// so the draws here are the ones that file has already characterized: 199 943
/// of 200 000 decode, into 200 003 planes, 99 925 balls and 99 958 pipes.
const SOAK: usize = 200_000;

/// How many records a timing region tries to hold, so the clock is not the thing
/// being measured.
///
/// Most draws record a handful, and two `Instant::now()` calls around four
/// `Record::new`s measure the timer. The rebuild is repeated to about this many
/// records instead, and divided back.
const RECORDS_A_GO: usize = 4096;

/// How many timing regions each draw gets, fastest kept.
const HOW_MANY_ROUNDS: u32 = 3;

fn main() {
    let mut rows = Vec::with_capacity(TARGETS.len() + 2);
    for target in &TARGETS {
        rows.push(what_one_target_costs(
            target,
            "corpus",
            seeds_of(target.name),
        ));
    }
    // The fuse is left out of the soak because random bytes do not decode as a
    // `Model`: postcard refuses them, the draw stops at the door, and 200 000
    // rows of nothing would say only that.
    for target in &TARGETS {
        if let Some(most_bytes) = target.soak_bytes {
            rows.push(what_one_target_costs(target, "soak", a_soak(most_bytes)));
        }
    }
    report(&rows);
}

/// The four targets that record anything, and the two doors each one has.
const TARGETS: [Target; 4] = [
    Target {
        name: "where_two_surfaces_cross",
        draw: draw_a_pair_of_surfaces,
        draw_keeping_the_provenance: draw_a_pair_of_surfaces_keeping_the_provenance,
        soak_bytes: Some(surfaces::MOST_BYTES_A_PAIR_READS),
    },
    Target {
        name: "where_a_cone_or_a_torus_crosses",
        draw: draw_a_cone_or_torus_pair,
        draw_keeping_the_provenance: draw_a_cone_or_torus_pair_keeping_the_provenance,
        soak_bytes: Some(surfaces::MOST_BYTES_A_CONE_OR_TORUS_PAIR_READS),
    },
    Target {
        name: "two_solids_fused_by_a_stranger",
        draw: draw_two_fused_models,
        draw_keeping_the_provenance: draw_two_fused_models_keeping_the_provenance,
        soak_bytes: None,
    },
    Target {
        name: "no_step_file_makes_the_reader_panic",
        draw: draw_a_step_file,
        draw_keeping_the_provenance: draw_a_step_file_keeping_the_provenance,
        soak_bytes: None,
    },
];

/// One fuzz target, as the three things this bench needs to ask of it.
///
/// `draw` is the body the target runs and `draw_keeping_the_provenance` is the
/// same body with the log handed back, and they are the same lines of code in
/// the harness file — which is the only reason the two passes are comparable at
/// all.
struct Target {
    name: &'static str,
    draw: fn(&[u8]),
    draw_keeping_the_provenance: fn(&[u8], &mut ProvenanceLog),
    /// How long a byte string to soak this target with, where a soak says
    /// anything.
    soak_bytes: Option<usize>,
}

/// Both passes over one set of draws.
fn what_one_target_costs(target: &Target, how: &'static str, draws: Vec<Vec<u8>>) -> Row {
    let mut row = Row {
        name: target.name,
        how,
        draws: draws.len(),
        ..Row::default()
    };

    for bytes in &draws {
        let started = Instant::now();
        (target.draw)(bytes);
        row.fuzzing += started.elapsed();
    }

    for bytes in &draws {
        let mut log = ProvenanceLog::new();
        (target.draw_keeping_the_provenance)(bytes, &mut log);
        row.recording += how_long_recording_it_again_takes(&log);
        row.see(bytes, &log);
    }

    row
}

/// Pseudo-random byte strings of the length this target reads.
///
/// A cheap deterministic generator and not a proptest strategy, for the reason
/// the soak it copies gives: what is being explored is the *bytes*, and a
/// strategy would put a shape on them that the fuzzer's own mutations do not
/// have.
fn a_soak(most_bytes: usize) -> Vec<Vec<u8>> {
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    (0..SOAK)
        .map(|_| {
            (0..most_bytes)
                .map(|_| {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    state as u8
                })
                .collect()
        })
        .collect()
}

/// Every seed checked in beside one fuzz target.
fn seeds_of(target: &str) -> Vec<Vec<u8>> {
    let corpus = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fuzz/corpus")
        .join(target);
    let listing = std::fs::read_dir(&corpus)
        .unwrap_or_else(|why| panic!("no corpus at {}: {why}", corpus.display()));
    let mut seeds: Vec<Vec<u8>> = listing
        .map(|entry| {
            let path = entry.expect("a corpus entry").path();
            std::fs::read(&path).unwrap_or_else(|why| panic!("{}: {why}", path.display()))
        })
        .collect();
    // Sorted so two runs of this bench read the same seeds in the same order and
    // the worst-draw row below names the same draw twice.
    seeds.sort();
    seeds
}

/// How long it takes to build, keep and free this log's records again.
///
/// **The same records and not records like them.** Every field is read back off
/// the real one, so the predicate names, the input counts and the log's growth
/// are the draw's own rather than a guess at their shape — which is what makes
/// the share a measurement instead of a model.
fn how_long_recording_it_again_takes(log: &ProvenanceLog) -> Duration {
    if log.is_empty() {
        return Duration::ZERO;
    }
    let goes = u32::try_from((RECORDS_A_GO / log.len()).max(1)).expect("a log fits in a u32");
    let mut fastest = Duration::MAX;
    for _ in 0..HOW_MANY_ROUNDS {
        let started = Instant::now();
        for _ in 0..goes {
            let mut again = ProvenanceLog::new();
            for record in log.records() {
                again.push(Record::new(
                    record.predicate(),
                    record.inputs(),
                    record.level(),
                    record.enclosure(),
                    record.measures(),
                ));
            }
            drop(black_box(again));
        }
        fastest = fastest.min(started.elapsed() / goes);
    }
    fastest
}

/// What one target's draws said.
#[derive(Default)]
struct Row {
    name: &'static str,
    /// Where the draws came from: the checked-in corpus, or the soak.
    how: &'static str,
    draws: usize,
    /// The draw as the fuzz target runs it, summed over the corpus.
    fuzzing: Duration,
    /// Every record that draw made, built and freed again.
    recording: Duration,
    /// Draws that recorded at least one decision.
    draws_that_recorded: usize,
    records: u64,
    /// What those records hold, counting the heap behind each one's inputs.
    bytes: u64,
    /// The single worst draw, which is the one a `-rss_limit_mb` would stop.
    worst_records: usize,
    worst_bytes: usize,
    /// The draw itself, because a count cannot be replayed and these bytes can.
    ///
    /// The same reason `rcad-fuzz-timing` keeps the slowest runs in full beside
    /// its histogram: a percentile names how bad the worst was and cannot name
    /// *which*, and which is the thing somebody then goes and pins in a test.
    worst_draw: Vec<u8>,
}

impl Row {
    /// Take in one draw's log.
    fn see(&mut self, bytes_drawn: &[u8], log: &ProvenanceLog) {
        if log.is_empty() {
            return;
        }
        let bytes = how_much_a_log_holds(log);
        self.draws_that_recorded += 1;
        self.records += log.len() as u64;
        self.bytes += bytes as u64;
        if log.len() > self.worst_records {
            self.worst_records = log.len();
            self.worst_bytes = bytes;
            self.worst_draw = bytes_drawn.to_vec();
        }
    }

    /// What share of the fuzzing the recording was.
    fn share(&self) -> f64 {
        if self.fuzzing.is_zero() {
            return f64::NAN;
        }
        self.recording.as_secs_f64() / self.fuzzing.as_secs_f64()
    }

    /// Records a second of fuzzing makes.
    fn records_a_second(&self) -> f64 {
        if self.fuzzing.is_zero() {
            return f64::NAN;
        }
        self.records as f64 / self.fuzzing.as_secs_f64()
    }
}

/// How much heap one log is holding, counting what its records point at.
///
/// A floor rather than a figure: `Vec` rounds its capacity up as it grows and
/// the allocator rounds again, so the real resident cost is above this. It is
/// still the number that says whether a draw can reach a `-rss_limit_mb`.
fn how_much_a_log_holds(log: &ProvenanceLog) -> usize {
    let inputs: usize = log
        .records()
        .iter()
        .map(|record| record.inputs().len())
        .sum();
    log.len() * size_of::<Record>() + inputs * size_of::<Point>()
}

/// The table that goes into the written record.
fn report(rows: &[Row]) {
    println!("\nwhat the provenance log costs a fuzz night");
    println!(
        "one Record is {} bytes and one Point is {}, before the heap behind the inputs\n",
        size_of::<Record>(),
        size_of::<Point>()
    );
    println!(
        "{:<36} {:>6} {:>7} {:>8} {:>10} {:>9} {:>9} {:>11} {:>8}",
        "target", "how", "draws", "recorded", "fuzzing", "records", "rec/s", "recording", "share"
    );
    for row in rows {
        println!(
            "{:<36} {:>6} {:>7} {:>8} {:>9.3}s {:>9} {:>9.0} {:>9.3}ms {:>7.4}%",
            row.name,
            row.how,
            row.draws,
            row.draws_that_recorded,
            row.fuzzing.as_secs_f64(),
            row.records,
            row.records_a_second(),
            row.recording.as_secs_f64() * 1e3,
            row.share() * 100.0,
        );
    }

    println!("\nthe worst single draw, which is the one an rss limit would stop");
    println!(
        "{:<36} {:>6} {:>10} {:>14}",
        "target", "how", "records", "at least"
    );
    for row in rows {
        println!(
            "{:<36} {:>6} {:>10} {:>11.1} kB",
            row.name,
            row.how,
            row.worst_records,
            row.worst_bytes as f64 / 1024.0,
        );
    }

    println!("\nand the draws themselves, written out so they can be replayed");
    for row in rows {
        if row.worst_draw.is_empty() {
            continue;
        }
        println!(
            "{:>8} bytes  {}",
            row.worst_draw.len(),
            where_a_draw_was_written(row).display()
        );
    }
    println!();
}

/// Write one row's worst draw out, and say where it went.
///
/// A file and not a hex string in the terminal, because the fuse's worst draw is
/// four kilobytes of `Model` — and because a file is what `cargo fuzz run` takes
/// as an argument. Under `target/`, so `just clean-targets` reclaims it and no
/// corpus is quietly added to.
///
/// # Panics
///
/// If the file cannot be written, which would leave the report naming a draw
/// nobody can get at.
fn where_a_draw_was_written(row: &Row) -> PathBuf {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/provenance-cost");
    std::fs::create_dir_all(&directory)
        .unwrap_or_else(|why| panic!("{}: {why}", directory.display()));
    let path = directory.join(format!("{}.{}", row.name, row.how));
    std::fs::write(&path, &row.worst_draw)
        .unwrap_or_else(|why| panic!("{}: {why}", path.display()));
    path
}

/// A pair of surfaces, exactly as `where_two_surfaces_cross` draws one.
fn draw_a_pair_of_surfaces(bytes: &[u8]) {
    if let Some(pair) = surfaces::two_surfaces_from(bytes) {
        let _ = surfaces::every_oracle_holds_of(pair);
    }
}

fn draw_a_pair_of_surfaces_keeping_the_provenance(bytes: &[u8], log: &mut ProvenanceLog) {
    if let Some(pair) = surfaces::two_surfaces_from(bytes) {
        let _ = surfaces::every_oracle_holds_of_keeping_the_provenance(pair, log);
    }
}

/// A pair with a cone or a torus in it, as `where_a_cone_or_a_torus_crosses`
/// draws one.
fn draw_a_cone_or_torus_pair(bytes: &[u8]) {
    if let Some(pair) = surfaces::two_surfaces_with_a_cone_or_a_torus_from(bytes) {
        let _ = surfaces::every_oracle_holds_of(pair);
    }
}

fn draw_a_cone_or_torus_pair_keeping_the_provenance(bytes: &[u8], log: &mut ProvenanceLog) {
    if let Some(pair) = surfaces::two_surfaces_with_a_cone_or_a_torus_from(bytes) {
        let _ = surfaces::every_oracle_holds_of_keeping_the_provenance(pair, log);
    }
}

/// Two models split and fused, as `two_solids_fused_by_a_stranger` draws them.
fn draw_two_fused_models(bytes: &[u8]) {
    let _ = fuse::two_models_split_and_fused(bytes);
}

fn draw_two_fused_models_keeping_the_provenance(bytes: &[u8], log: &mut ProvenanceLog) {
    let _ = fuse::two_models_split_and_fused_keeping_the_provenance(bytes, log);
}

/// A Part 21 file read and walked, as `no_step_file_makes_the_reader_panic`
/// draws one.
fn draw_a_step_file(bytes: &[u8]) {
    let _ = reader::everything_one_file_is_put_through(bytes);
}

fn draw_a_step_file_keeping_the_provenance(bytes: &[u8], log: &mut ProvenanceLog) {
    let _ = reader::everything_one_file_is_put_through_keeping_the_provenance(bytes, log);
}
