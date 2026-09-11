//! How long each fuzz run took, and which inputs were the slow ones.
//!
//! # What libFuzzer leaves behind, and why it is not enough
//!
//! libFuzzer keeps no per-run timings. Its only timing output is the `Slowest
//! unit:` line, and that **ratchets**: a run is reported only when it beats the
//! previous record by a tenth *and* clears `-report_slow_units`, which is whole
//! seconds and ten of them by default. So a night that ran a thousand slow
//! inputs leaves one file behind, and nothing at all says how the rest of the
//! runs were spread.
//!
//! That mattered on 2026-09-11. A sixty-minute run of `where_two_surfaces_cross`
//! did 93 584 runs at 31 a second where the night before had done 262, and the
//! only evidence of where the time went was a single 19-second artifact.
//! Measured afterwards over the corpus instead, the spread turned out to be a
//! cliff and not a tail — p95 at 0.474 ms, p98 at 7.5 ms, p99 at 3 103 ms — with
//! **twenty seeds of 996 taking 99.8% of the time**. None of that is visible
//! from what libFuzzer writes.
//!
//! # What this keeps
//!
//! Two things, because they answer two different questions.
//!
//! A **histogram**, sixteen buckets to the octave, so a percentile can be read
//! off at the end. It is fixed-size and costs one add per run, which is what
//! lets it run over the reader's quarter of a billion draws.
//!
//! The **slowest runs themselves**, kept in full so they can be handed straight
//! back to `cargo fuzz run`. A percentile names how slow the slow ones were; it
//! cannot name *which*, and which is the thing somebody then goes and reads.
//!
//! # It is off unless a directory is named
//!
//! `RCAD_FUZZ_TIMINGS` is a directory, and nothing here runs without it — not
//! even the clock. `RCAD_FUZZ_SLOWEST` is how many of the slowest runs to keep,
//! and defaults to [`HOW_MANY_SLOW_RUNS_TO_KEEP`].

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// How many of the slowest runs are kept, where `RCAD_FUZZ_SLOWEST` says nothing.
pub const HOW_MANY_SLOW_RUNS_TO_KEEP: usize = 64;

/// The one watch this process keeps, however many kinds of run are timed.
///
/// **Out here and not inside [`timed`]**, which is where it reads better and
/// would be wrong: a `static` declared in a generic function is duplicated per
/// monomorphisation, so a target that timed a closure returning something other
/// than `()` would quietly get a second watch writing over the first one's
/// report.
static WATCH: OnceLock<Option<Mutex<Watch>>> = OnceLock::new();

/// Run one draw of a fuzz target, timing it where a directory has been named.
///
/// Pass `env!("CARGO_BIN_NAME")` as `target` — it is the fuzz target's own name
/// and cannot drift from the one `cargo fuzz` was asked for.
///
/// With `RCAD_FUZZ_TIMINGS` unset this is the body and one already-resolved
/// `OnceLock` read, so the three targets that nobody is measuring pay nothing.
pub fn timed<T>(target: &str, bytes: &[u8], run: impl FnOnce() -> T) -> T {
    let Some(watch) = WATCH.get_or_init(|| Watch::asked_for(target).map(Mutex::new)) else {
        return run();
    };

    // The clock is stopped before the lock is taken, and `run` is called outside
    // it, so a draw that panics — which is what a fuzz target is for — neither
    // poisons the lock nor counts its own half-finished time.
    let started = Instant::now();
    let answer = run();
    let took = started.elapsed();
    let mut watch = watch
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    watch.record(took, bytes);
    answer
}

/// How long to leave between rewrites of the report.
///
/// The report is rewritten *while the run is going*, and not at the end, because
/// there is no end a fuzz target is guaranteed to reach: libFuzzer calls `exit`
/// on a crash and the timer kills a target that overruns its slice.
const HOW_OFTEN_TO_WRITE: Duration = Duration::from_secs(30);

/// What one target's process has seen so far.
struct Watch {
    into: PathBuf,
    took: HowLongRunsTook,
    slowest: TheSlowestRuns,
    written: Option<Instant>,
}

impl Watch {
    /// The watch `RCAD_FUZZ_TIMINGS` asks for, or `None` where it says nothing.
    fn asked_for(target: &str) -> Option<Self> {
        let root = std::env::var_os("RCAD_FUZZ_TIMINGS")?;
        let keep = std::env::var("RCAD_FUZZ_SLOWEST")
            .ok()
            .and_then(|how_many| how_many.parse().ok())
            .unwrap_or(HOW_MANY_SLOW_RUNS_TO_KEEP);
        Some(Self {
            into: Path::new(&root).join(target),
            took: HowLongRunsTook::new(),
            slowest: TheSlowestRuns::keeping(keep),
            written: None,
        })
    }

    /// One run, counted and — where it is among the slowest — kept.
    fn record(&mut self, took: Duration, bytes: &[u8]) {
        self.took.record(took);

        // The clock is read only for a run worth keeping, which after the first
        // seconds is a few in a million. Asking whether a rewrite is due on
        // every run instead costs a third `Instant::now()` on a path that two
        // are already most of.
        if !self.slowest.offer(took, bytes) {
            return;
        }
        if self
            .written
            .is_none_or(|written| written.elapsed() >= HOW_OFTEN_TO_WRITE)
        {
            self.write_the_report();
            self.written = Some(Instant::now());
        }
    }

    /// The report and the slow inputs beside it.
    ///
    /// Every failure here is swallowed. A fuzz target that stopped fuzzing
    /// because its measurements could not be filed would be measuring the disk.
    fn write_the_report(&self) {
        let slowest = self.into.join("slowest");
        if std::fs::create_dir_all(&slowest).is_err() {
            return;
        }
        clear_stale_slow_runs(&slowest);
        for (took, bytes) in self.slowest.slowest_first() {
            let _ = std::fs::write(slowest.join(named_for(took, bytes)), bytes);
        }
        let _ = std::fs::write(self.into.join("report"), self.report_text());
    }

    /// The report, as it is written to disk.
    fn report_text(&self) -> String {
        let mut text = format!(
            "{} runs, {:.1} s in all\n\nhow long one run took, as the low end of its bucket\n",
            self.took.runs,
            self.took.total.as_secs_f64()
        );
        for share in [50.0, 90.0, 95.0, 98.0, 99.0, 100.0] {
            text.push_str(&format!(
                "  p{share:<5} {:>14.3} ms\n",
                self.took.at_percentile(share).as_secs_f64() * 1e3
            ));
        }
        let cut = self.took.at_percentile(98.0);
        text.push_str(&format!(
            "\n{} runs at or over p98, of which the slowest {} are kept in slowest/:\n",
            self.took.how_many_took_at_least(cut),
            self.slowest.runs.len(),
        ));
        for (took, bytes) in self.slowest.slowest_first() {
            text.push_str(&format!(
                "  {:>12.3} ms  {}\n",
                took.as_secs_f64() * 1e3,
                named_for(took, bytes)
            ));
        }
        text
    }
}

/// What one slow run's file is called.
///
/// The time comes first so a listing sorts by it, and the hash last so the same
/// input keeps the same name across rewrites and across nights.
fn named_for(took: Duration, bytes: &[u8]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("slow-{:012}us-{hash:016x}", took.as_micros())
}

/// The slow inputs a previous rewrite left, which the current top no longer names.
///
/// Only files this module writes are touched, so pointing `RCAD_FUZZ_TIMINGS` at
/// a directory with something else in it cannot lose that something else.
fn clear_stale_slow_runs(from: &Path) {
    let Ok(listing) = std::fs::read_dir(from) else {
        return;
    };
    for entry in listing.flatten() {
        if entry.file_name().to_string_lossy().starts_with("slow-") {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

/// How many runs landed in each bucket of nanoseconds.
struct HowLongRunsTook {
    runs: u64,
    total: Duration,
    buckets: Box<[u64; HOW_MANY_BUCKETS]>,
}

impl HowLongRunsTook {
    fn new() -> Self {
        Self {
            runs: 0,
            total: Duration::ZERO,
            buckets: Box::new([0; HOW_MANY_BUCKETS]),
        }
    }

    fn record(&mut self, took: Duration) {
        self.runs += 1;
        self.total += took;
        let nanos = u64::try_from(took.as_nanos()).unwrap_or(u64::MAX);
        self.buckets[bucket_of(nanos)] += 1;
    }

    /// The value at a percentile by nearest rank, as the low end of its bucket.
    ///
    /// The low end and not the middle, because what a reader does with this is
    /// compare a run against it, and a bucket's low end is the only value in it
    /// that no run in the bucket is below.
    fn at_percentile(&self, share: f64) -> Duration {
        if self.runs == 0 {
            return Duration::ZERO;
        }
        let rank = (share / 100.0 * self.runs as f64).ceil().max(1.0) as u64;
        let mut seen = 0;
        for (bucket, count) in self.buckets.iter().enumerate() {
            seen += count;
            if seen >= rank {
                return Duration::from_nanos(nanos_at(bucket));
            }
        }
        Duration::from_nanos(nanos_at(HOW_MANY_BUCKETS - 1))
    }

    /// How many runs took at least this long, counted from the low end of the
    /// bucket `took` falls in.
    ///
    /// So it is an over-count by up to a bucket's width, and deliberately: what
    /// it is asked for in the report is how many runs the p98 line covers, and
    /// that line is itself the low end of a bucket. Counting from anywhere else
    /// would answer a question about a value the report does not print.
    fn how_many_took_at_least(&self, took: Duration) -> u64 {
        let nanos = u64::try_from(took.as_nanos()).unwrap_or(u64::MAX);
        self.buckets[bucket_of(nanos)..].iter().sum()
    }
}

/// How many buckets one octave of nanoseconds is split into.
const BUCKETS_TO_AN_OCTAVE: u64 = 1 << SPLITTING_BITS;

/// How many bits of a length below its leading one the bucket keeps.
const SPLITTING_BITS: u32 = 4;

/// Enough buckets for every `u64` of nanoseconds, which is 584 years of them.
const HOW_MANY_BUCKETS: usize = 976;

/// Which bucket a length of run falls in.
///
/// Sixteen to the octave, so a bucket is at most 6.25% wide and a percentile read
/// off one is that close. Lengths below sixteen nanoseconds get a bucket each,
/// which keeps the numbering unbroken at the bottom where an octave has fewer
/// values in it than the split asks for.
fn bucket_of(nanos: u64) -> usize {
    if nanos < BUCKETS_TO_AN_OCTAVE {
        return nanos as usize;
    }
    let octave = u64::from(63 - nanos.leading_zeros());
    let splitting = (nanos >> (octave - u64::from(SPLITTING_BITS))) & (BUCKETS_TO_AN_OCTAVE - 1);
    ((octave - u64::from(SPLITTING_BITS) + 1) * BUCKETS_TO_AN_OCTAVE + splitting) as usize
}

/// The shortest run that falls in a bucket.
fn nanos_at(bucket: usize) -> u64 {
    let bucket = bucket as u64;
    if bucket < BUCKETS_TO_AN_OCTAVE {
        return bucket;
    }
    let octave = bucket / BUCKETS_TO_AN_OCTAVE + u64::from(SPLITTING_BITS) - 1;
    let splitting = bucket % BUCKETS_TO_AN_OCTAVE;
    (BUCKETS_TO_AN_OCTAVE + splitting) << (octave - u64::from(SPLITTING_BITS))
}

/// The slowest runs seen, and the bytes that were slow.
struct TheSlowestRuns {
    keep: usize,
    /// Quickest first, so the one to drop is the one at the front.
    runs: Vec<(Duration, Vec<u8>)>,
}

impl TheSlowestRuns {
    fn keeping(keep: usize) -> Self {
        Self {
            keep,
            runs: Vec::new(),
        }
    }

    /// One run, kept where it is among the slowest. Says whether it was kept.
    ///
    /// An input already held is not held twice. libFuzzer runs a corpus entry
    /// again whenever it merges or minimises, and a top filled with one input
    /// sixty-four times names one slow seed where it could have named sixty-four.
    fn offer(&mut self, took: Duration, bytes: &[u8]) -> bool {
        if self.keep == 0 {
            return false;
        }
        if self.runs.len() == self.keep && took <= self.runs[0].0 {
            return false;
        }
        if let Some(held) = self.runs.iter().position(|(_, held)| held == bytes) {
            if self.runs[held].0 >= took {
                return false;
            }
            self.runs.remove(held);
        } else if self.runs.len() == self.keep {
            self.runs.remove(0);
        }
        let at = self.runs.partition_point(|(seen, _)| *seen <= took);
        self.runs.insert(at, (took, bytes.to_vec()));
        true
    }

    /// What is kept, slowest first.
    fn slowest_first(&self) -> impl Iterator<Item = (Duration, &[u8])> {
        self.runs
            .iter()
            .rev()
            .map(|(took, bytes)| (*took, bytes.as_slice()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_length_of_run_is_in_a_bucket_that_starts_at_or_below_it() {
        for nanos in (0..2000).chain([
            1 << 20,
            (1 << 20) + 1,
            1_000_000_000,
            6_997_742_000,
            u64::MAX,
        ]) {
            let bucket = bucket_of(nanos);
            assert!(
                nanos_at(bucket) <= nanos,
                "{nanos} ns landed in a bucket starting at {}",
                nanos_at(bucket)
            );
            assert!(
                bucket + 1 >= HOW_MANY_BUCKETS || nanos_at(bucket + 1) > nanos,
                "{nanos} ns should have been in the next bucket up"
            );
        }
    }

    #[test]
    fn the_buckets_run_from_nothing_to_the_longest_run_a_u64_can_hold() {
        assert_eq!(bucket_of(0), 0);
        assert_eq!(bucket_of(u64::MAX), HOW_MANY_BUCKETS - 1);
        for bucket in 1..HOW_MANY_BUCKETS {
            assert!(
                nanos_at(bucket) > nanos_at(bucket - 1),
                "bucket {bucket} does not start above the one below it"
            );
        }
    }

    #[test]
    fn a_bucket_is_never_more_than_a_sixteenth_wider_than_where_it_starts() {
        for bucket in BUCKETS_TO_AN_OCTAVE as usize..HOW_MANY_BUCKETS - 1 {
            let (starts, next) = (nanos_at(bucket), nanos_at(bucket + 1));
            assert!(
                (next - starts) * BUCKETS_TO_AN_OCTAVE <= starts,
                "bucket {bucket} runs {starts}..{next}, which is wider than a sixteenth"
            );
        }
    }

    #[test]
    fn the_percentile_of_a_known_spread_is_the_value_at_that_rank() {
        let mut took = HowLongRunsTook::new();
        for run in 1..=100 {
            took.record(Duration::from_millis(run));
        }
        // Nearest rank, then rounded down to where its bucket starts, which for
        // a millisecond in the nineties is within 6.25% of the value itself.
        for (share, wanted) in [(50.0, 50), (90.0, 90), (98.0, 98), (100.0, 100)] {
            let at = took.at_percentile(share).as_secs_f64() * 1e3;
            assert!(
                at <= f64::from(wanted) && at > f64::from(wanted) * 0.9375,
                "p{share} of one to a hundred milliseconds came back {at} ms"
            );
        }
    }

    #[test]
    fn a_percentile_of_nothing_at_all_is_nothing_rather_than_a_panic() {
        assert_eq!(HowLongRunsTook::new().at_percentile(98.0), Duration::ZERO);
    }

    #[test]
    fn how_many_took_at_least_counts_the_slow_end_and_not_the_whole_run() {
        let mut took = HowLongRunsTook::new();
        for run in 1..=100 {
            took.record(Duration::from_millis(run));
        }
        assert_eq!(took.how_many_took_at_least(Duration::from_millis(1)), 100);

        // A bucket at a hundred milliseconds runs 96.47 to 100.66, so asking
        // about the hundredth run counts the four that share its bucket. That
        // over-count is the whole of what a sixteenth-of-an-octave bucket costs,
        // and the report prints the same low end it counts from.
        assert_eq!(took.how_many_took_at_least(Duration::from_millis(100)), 4);
        assert_eq!(took.how_many_took_at_least(Duration::from_millis(50)), 52);
    }

    #[test]
    fn the_slowest_runs_kept_are_the_slowest_ones_offered() {
        let mut slowest = TheSlowestRuns::keeping(3);
        for run in [5u64, 1, 9, 3, 7] {
            slowest.offer(Duration::from_millis(run), &[run as u8]);
        }
        let kept: Vec<_> = slowest
            .slowest_first()
            .map(|(took, bytes)| (took.as_millis(), bytes.to_vec()))
            .collect();
        assert_eq!(kept, vec![(9, vec![9]), (7, vec![7]), (5, vec![5])]);
    }

    #[test]
    fn a_run_no_slower_than_the_quickest_kept_is_turned_away() {
        let mut slowest = TheSlowestRuns::keeping(2);
        assert!(slowest.offer(Duration::from_millis(9), b"slow"));
        assert!(slowest.offer(Duration::from_millis(7), b"middling"));
        assert!(!slowest.offer(Duration::from_millis(1), b"quick"));
        assert_eq!(slowest.runs.len(), 2);
    }

    #[test]
    fn one_input_offered_twice_is_held_once_at_its_slowest() {
        let mut slowest = TheSlowestRuns::keeping(4);
        slowest.offer(Duration::from_millis(3), b"same");
        slowest.offer(Duration::from_millis(9), b"same");
        slowest.offer(Duration::from_millis(5), b"same");
        let kept: Vec<_> = slowest
            .slowest_first()
            .map(|(took, _)| took.as_millis())
            .collect();
        assert_eq!(kept, vec![9], "the same bytes were held more than once");
    }

    #[test]
    fn keeping_none_keeps_none_rather_than_reaching_past_the_end() {
        let mut slowest = TheSlowestRuns::keeping(0);
        assert!(!slowest.offer(Duration::from_millis(9), b"slow"));
        assert_eq!(slowest.slowest_first().count(), 0);
    }

    #[test]
    fn one_input_keeps_one_name_however_often_it_is_written() {
        let named = named_for(Duration::from_millis(12), b"bytes");
        assert_eq!(named, named_for(Duration::from_millis(12), b"bytes"));
        assert_ne!(named, named_for(Duration::from_millis(12), b"other"));
        assert!(named.starts_with("slow-000000012000us-"), "{named}");
    }

    #[test]
    fn a_report_names_every_slow_input_it_wrote() {
        let into = std::env::temp_dir().join(format!("rcad-fuzz-timing-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&into);
        let mut watch = Watch {
            into: into.clone(),
            took: HowLongRunsTook::new(),
            slowest: TheSlowestRuns::keeping(2),
            written: None,
        };
        for run in [1u64, 40, 90] {
            watch.record(Duration::from_millis(run), &[run as u8]);
        }
        watch.write_the_report();

        let report = std::fs::read_to_string(into.join("report")).expect("a report");
        assert!(report.contains("3 runs"), "{report}");
        let mut named = 0;
        for line in report.lines().filter(|line| line.contains("slow-")) {
            let file = line.split_whitespace().last().expect("a file name");
            assert!(
                into.join("slowest").join(file).is_file(),
                "the report names {file}, which was not written"
            );
            named += 1;
        }
        assert_eq!(named, 2, "the report named {named} of the two kept");
        let _ = std::fs::remove_dir_all(&into);
    }

    #[test]
    fn a_rewrite_clears_the_slow_runs_the_last_one_left() {
        let into = std::env::temp_dir().join(format!("rcad-fuzz-stale-{}", std::process::id()));
        let slowest = into.join("slowest");
        std::fs::create_dir_all(&slowest).expect("somewhere to write");
        std::fs::write(slowest.join("slow-000000000001us-dead"), b"old").expect("an old run");
        std::fs::write(slowest.join("something-else"), b"not ours").expect("a stranger");

        let mut watch = Watch {
            into: into.clone(),
            took: HowLongRunsTook::new(),
            slowest: TheSlowestRuns::keeping(1),
            written: None,
        };
        watch.record(Duration::from_millis(9), b"new");
        watch.write_the_report();

        assert!(!slowest.join("slow-000000000001us-dead").exists());
        assert!(
            slowest.join("something-else").exists(),
            "a file this module did not write was removed"
        );
        let _ = std::fs::remove_dir_all(&into);
    }
}
