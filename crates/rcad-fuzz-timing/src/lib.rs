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
//! # Many processes write here, and that is the hard part
//!
//! Under `-fork=N` the target is not one process. libFuzzer runs a churn of
//! short-lived children — measured at twelve jobs in 101 seconds at `-fork=2`,
//! so about seventeen seconds apiece, which over an eight-hour night at
//! `-fork=24` is roughly **forty thousand processes per target**.
//!
//! Until 2026-09-18 each of them wrote its own report over the last one's and
//! deleted the slow units it found. Measured: 54 845 runs across 12 jobs left a
//! report reading `1 runs, 0.0 s in all`. Not a partial answer — the whole
//! measurement, gone.
//!
//! A directory per process is no fix either, because forty thousand of them is
//! not a report. So there is **one shared accumulation per target**, guarded by
//! a lock file, and each process merges what it has seen into it. The slowest
//! runs need no accumulation at all: a file's name already carries how slow it
//! was, so the directory listing *is* the merged top, and pruning is a sort.
//!
//! # It is off unless a directory is named
//!
//! `RCAD_FUZZ_TIMINGS` is a directory, and nothing here runs without it — not
//! even the clock, and not the merging thread. `RCAD_FUZZ_SLOWEST` is how many
//! of the slowest runs to keep, and defaults to [`HOW_MANY_SLOW_RUNS_TO_KEEP`].

#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, Once, OnceLock};
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

/// Guards the one merging thread, for the same reason `WATCH` is out here.
static START_MERGING: Once = Once::new();

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

    // The thread is started here rather than in the initialiser above because
    // `get_or_init`'s closure cannot hand itself the `&'static` its thread needs.
    START_MERGING.call_once(|| {
        std::thread::spawn(|| {
            loop {
                std::thread::sleep(HOW_OFTEN_TO_MERGE);
                merge_out(watch);
            }
        });
    });

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

/// How long to leave between merges into the shared accumulation.
///
/// **A second, where the report it replaced was rewritten every thirty.** A
/// process only ever loses what it has seen since its last merge, and under
/// `-fork` a process lives about seventeen seconds — so at thirty seconds most
/// children would exit having merged nothing at all, and the loss would be the
/// whole night rather than the edge of it. libFuzzer kills its children from
/// outside, so there is no exit to hook and no way to make the loss zero.
///
/// The cost is a lock and a small read-modify-write per process per second,
/// which against twenty-four of them is a rounding error next to the fuzzing.
const HOW_OFTEN_TO_MERGE: Duration = Duration::from_secs(1);

/// What this process has seen and not yet merged.
struct Watch {
    into: PathBuf,
    keep: usize,
    took: HowLongRunsTook,
    slowest: TheSlowestRuns,
    /// Slow inputs already on disk, so a steady top is not rewritten every tick.
    written: HashSet<String>,
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
            keep,
            took: HowLongRunsTook::new(),
            slowest: TheSlowestRuns::keeping(keep),
            written: HashSet::new(),
        })
    }

    /// One run, counted and — where it is among the slowest — kept.
    ///
    /// No clock is read and nothing is written. Everything this process has seen
    /// goes out on the merging thread's tick, which is what lets the run path
    /// stay two `Instant::now()`s and an add.
    fn record(&mut self, took: Duration, bytes: &[u8]) {
        self.took.record(took);
        self.slowest.offer(took, bytes);
    }
}

/// Add what this process has seen to the shared accumulation, and rewrite the
/// report from the result.
///
/// Two failures are recovered rather than swallowed: if the directory cannot
/// be made or the shared lock cannot be taken, the run count just collected
/// goes back to the live watch so the next tick tries again. Everything past
/// that point is still swallowed — a write that fails after the lock is held
/// loses just that one merge's slow-run bytes and report text, not a night's
/// worth of runs. A fuzz target that stopped fuzzing because its measurements
/// could not be filed would be measuring the disk.
fn merge_out(watch: &Mutex<Watch>) {
    let (into, keep, mine, to_write) = {
        let mut watch = watch
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // The histogram is handed over and reset, so what is merged is a delta
        // and a process that merges twice does not count its runs twice. The
        // slowest are *not* reset: they are the process's running top, and
        // dropping them would make every tick re-offer the whole corpus.
        let mine = std::mem::replace(&mut watch.took, HowLongRunsTook::new());
        let to_write: Vec<(Duration, Vec<u8>)> = watch
            .slowest
            .slowest_first()
            .filter(|(took, bytes)| !watch.written.contains(&named_for(*took, bytes)))
            .map(|(took, bytes)| (took, bytes.to_vec()))
            .collect();
        for (took, bytes) in &to_write {
            watch.written.insert(named_for(*took, bytes));
        }
        (watch.into.clone(), watch.keep, mine, to_write)
    };

    if mine.runs == 0 && to_write.is_empty() {
        return;
    }

    let slowest = into.join("slowest");
    if std::fs::create_dir_all(&slowest).is_err() {
        hand_back_what_could_not_be_written(watch, mine);
        return;
    }
    let Some(lock) = the_shared_report_locked(&into) else {
        hand_back_what_could_not_be_written(watch, mine);
        return;
    };

    let mut everyones = HowLongRunsTook::read_back(&into.join("totals"));
    everyones.add(&mine);
    let _ = std::fs::write(into.join("totals"), everyones.written_out());

    for (took, bytes) in &to_write {
        let _ = std::fs::write(slowest.join(named_for(*took, bytes)), bytes);
    }
    let kept = prune_to_the_slowest_kept(&slowest, keep);
    let _ = std::fs::write(into.join("report"), report_text(&everyones, &kept));

    let _ = lock.unlock();
}

/// Put a run count that a merge could not write back into the live watch, so
/// the next tick tries again instead of the runs vanishing with this one.
///
/// Every run in `mine` just failed to write, whether or not some of them had
/// already failed on an earlier tick — `mine.lost` is *set* to `mine.runs`
/// rather than added to, so a run stuck across three failed ticks in a row is
/// counted as one run lost, not three. The lock is taken fresh here rather
/// than reusing the caller's guard, because another thread's [`Watch::record`]
/// may have added new runs to the watch in the meantime; a fresh lock merges
/// onto whatever is there now instead of overwriting it.
fn hand_back_what_could_not_be_written(watch: &Mutex<Watch>, mut mine: HowLongRunsTook) {
    mine.lost = mine.runs;
    watch
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .took
        .add(&mine);
}

/// The lock every process takes before touching the shared accumulation.
///
/// A plain advisory lock on a file of its own. It is held for a read, an add and
/// two small writes, so a process waits on it for microseconds even with all
/// twenty-four forks ticking at once.
fn the_shared_report_locked(into: &Path) -> Option<std::fs::File> {
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(into.join("lock"))
        .ok()?;
    lock.lock().ok()?;
    Some(lock)
}

/// Cut `slowest/` back to the slowest `keep` runs, and say what is left.
///
/// **This is where the merging of the slow runs happens**, and it needs no
/// accumulation because [`named_for`] puts the time in the name: whatever
/// process wrote a file, its name says how slow it was and which input it was,
/// so the listing sorted by time is the top across every process at once.
///
/// One input found by two processes is two names with the same hash and
/// different times, and only the slowest is kept — the same rule
/// [`TheSlowestRuns::offer`] applies inside one process, applied across them.
///
/// Only files this module writes are touched, so pointing `RCAD_FUZZ_TIMINGS` at
/// a directory with something else in it cannot lose that something else.
fn prune_to_the_slowest_kept(from: &Path, keep: usize) -> Vec<(Duration, String)> {
    let Ok(listing) = std::fs::read_dir(from) else {
        return Vec::new();
    };
    let mut found: Vec<(Duration, String)> = Vec::new();
    for entry in listing.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Some((took, _)) = how_slow_the_name_says(&name) {
            found.push((took, name));
        }
    }
    found.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));

    let mut kept: Vec<(Duration, String)> = Vec::new();
    let mut held: HashSet<String> = HashSet::new();
    for (took, name) in found {
        let input = how_slow_the_name_says(&name)
            .map(|(_, input)| input.to_string())
            .unwrap_or_default();
        if kept.len() < keep && held.insert(input) {
            kept.push((took, name));
        } else {
            let _ = std::fs::remove_file(from.join(&name));
        }
    }
    kept
}

/// The report, as it is written to disk.
fn report_text(everyones: &HowLongRunsTook, kept: &[(Duration, String)]) -> String {
    let mut text = format!(
        "{} runs, {:.1} s in all\n",
        everyones.runs,
        everyones.total.as_secs_f64()
    );
    if everyones.lost > 0 {
        text.push_str(&format!(
            "{} of those runs were lost to a write failure and picked up again on a later merge\n",
            everyones.lost
        ));
    }
    text.push_str("\nhow long one run took, as the low end of its bucket\n");
    for share in [50.0, 90.0, 95.0, 98.0, 99.0, 100.0] {
        text.push_str(&format!(
            "  p{share:<5} {:>14.3} ms\n",
            everyones.at_percentile(share).as_secs_f64() * 1e3
        ));
    }
    let cut = everyones.at_percentile(98.0);
    text.push_str(&format!(
        "\n{} runs at or over p98, of which the slowest {} are kept in slowest/:\n",
        everyones.how_many_took_at_least(cut),
        kept.len(),
    ));
    for (took, name) in kept {
        text.push_str(&format!(
            "  {:>12.3} ms  {}\n",
            took.as_secs_f64() * 1e3,
            name
        ));
    }
    text
}

/// What one slow run's file is called.
///
/// The time comes first so a listing sorts by it, and the hash last so the same
/// input keeps the same name across rewrites, across nights and across the many
/// processes a `-fork` run is made of.
fn named_for(took: Duration, bytes: &[u8]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("slow-{:012}us-{hash:016x}", took.as_micros())
}

/// How slow a slow run's file name says it was, and which input it names.
///
/// The inverse of [`named_for`], and the reason the slow runs need no shared
/// accumulation of their own.
fn how_slow_the_name_says(name: &str) -> Option<(Duration, &str)> {
    let rest = name.strip_prefix("slow-")?;
    let (micros, input) = rest.split_once("us-")?;
    Some((Duration::from_micros(micros.parse().ok()?), input))
}

/// How many runs landed in each bucket of nanoseconds.
struct HowLongRunsTook {
    runs: u64,
    total: Duration,
    buckets: Box<[u64; HOW_MANY_BUCKETS]>,
    /// Of `runs`, how many needed more than one merge because an earlier one
    /// failed to write. Not a count of runs still missing — every run counted
    /// here is also counted in `runs`, since [`hand_back_what_could_not_be_written`]
    /// is what sets this before the retry that eventually succeeds.
    lost: u64,
}

impl HowLongRunsTook {
    fn new() -> Self {
        Self {
            runs: 0,
            total: Duration::ZERO,
            buckets: Box::new([0; HOW_MANY_BUCKETS]),
            lost: 0,
        }
    }

    fn record(&mut self, took: Duration) {
        self.runs += 1;
        self.total += took;
        let nanos = u64::try_from(took.as_nanos()).unwrap_or(u64::MAX);
        self.buckets[bucket_of(nanos)] += 1;
    }

    /// Add everything another process saw to this.
    fn add(&mut self, other: &Self) {
        self.runs += other.runs;
        self.total += other.total;
        for (mine, theirs) in self.buckets.iter_mut().zip(other.buckets.iter()) {
            *mine += theirs;
        }
        self.lost += other.lost;
    }

    /// The accumulation as it is stored, which only this module reads.
    ///
    /// Empty buckets are left out. A histogram of a real night touches a few
    /// hundred of the 976, so the file stays a few kilobytes however long the
    /// night runs, which is what makes a merge every second cheap.
    fn written_out(&self) -> String {
        let mut text = format!(
            "runs {}\nnanos {}\nlost {}\n",
            self.runs,
            self.total.as_nanos(),
            self.lost
        );
        for (bucket, count) in self.buckets.iter().enumerate() {
            if *count > 0 {
                text.push_str(&format!("b {bucket} {count}\n"));
            }
        }
        text
    }

    /// What [`Self::written_out`] wrote, or an empty one where there is nothing
    /// to read yet and where what is there cannot be understood.
    ///
    /// A file this cannot parse is treated as empty rather than as an error, for
    /// the same reason every other failure here is swallowed — and losing a
    /// night's counts to a torn file is better than a fuzz target that stops.
    fn read_back(from: &Path) -> Self {
        let mut took = Self::new();
        let Ok(text) = std::fs::read_to_string(from) else {
            return took;
        };
        for line in text.lines() {
            let mut words = line.split_whitespace();
            match (words.next(), words.next(), words.next()) {
                (Some("runs"), Some(runs), None) => took.runs = runs.parse().unwrap_or(0),
                (Some("nanos"), Some(nanos), None) => {
                    took.total = Duration::from_nanos(
                        u64::try_from(nanos.parse::<u128>().unwrap_or(0)).unwrap_or(u64::MAX),
                    );
                }
                (Some("lost"), Some(lost), None) => took.lost = lost.parse().unwrap_or(0),
                (Some("b"), Some(bucket), Some(count)) => {
                    if let Ok(bucket) = bucket.parse::<usize>()
                        && bucket < HOW_MANY_BUCKETS
                    {
                        took.buckets[bucket] = count.parse().unwrap_or(0);
                    }
                }
                _ => {}
            }
        }
        took
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

    /// A directory of this test's own, named so two tests never share one.
    fn somewhere_to_write(what: &str) -> PathBuf {
        let into = std::env::temp_dir().join(format!(
            "rcad-fuzz-timing-{}-{what}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&into);
        into
    }

    /// A watch writing into `into`, as a process under `-fork` would have.
    fn a_process_writing_into(into: &Path, keep: usize) -> Mutex<Watch> {
        Mutex::new(Watch {
            into: into.to_path_buf(),
            keep,
            took: HowLongRunsTook::new(),
            slowest: TheSlowestRuns::keeping(keep),
            written: HashSet::new(),
        })
    }

    #[test]
    fn two_processes_writing_at_once_keep_both_their_runs() {
        // The regression. Under `-fork` the target is forty thousand short-lived
        // processes over a night, every one of them pointed at this directory.
        // Each used to write its own report over the last one's, so a measured
        // 54 845 runs across 12 jobs left a report reading `1 runs`.
        let into = somewhere_to_write("two-processes");
        let first = a_process_writing_into(&into, 4);
        let second = a_process_writing_into(&into, 4);

        for run in 1..=3u64 {
            first
                .lock()
                .unwrap()
                .record(Duration::from_millis(run), &[run as u8]);
        }
        for run in 4..=9u64 {
            second
                .lock()
                .unwrap()
                .record(Duration::from_millis(run), &[run as u8]);
        }
        merge_out(&first);
        merge_out(&second);

        let report = std::fs::read_to_string(into.join("report")).expect("a report");
        assert!(
            report.starts_with("9 runs"),
            "two processes recorded 3 and 6 runs; the report says {:?}",
            report.lines().next()
        );
        let _ = std::fs::remove_dir_all(&into);
    }

    #[test]
    fn a_process_that_merges_twice_does_not_count_its_runs_twice() {
        // The merging thread ticks once a second for the whole life of a
        // process, so all but the first tick of every process merges a delta on
        // top of what that same process already put in.
        let into = somewhere_to_write("merge-twice");
        let watch = a_process_writing_into(&into, 4);

        watch
            .lock()
            .unwrap()
            .record(Duration::from_millis(5), b"once");
        merge_out(&watch);
        merge_out(&watch);
        watch
            .lock()
            .unwrap()
            .record(Duration::from_millis(5), b"twice");
        merge_out(&watch);

        let report = std::fs::read_to_string(into.join("report")).expect("a report");
        assert!(
            report.starts_with("2 runs"),
            "two runs were recorded over three merges; the report says {:?}",
            report.lines().next()
        );
        let _ = std::fs::remove_dir_all(&into);
    }

    #[test]
    fn runs_a_failed_merge_could_not_write_survive_and_are_counted() {
        // §236.6: `merge_out` used to drop a process's run count on the floor
        // when it could not create the shared directory or take the lock. A
        // regular file where a directory needs to go can never be turned into
        // one, so `create_dir_all` fails on every tick until it is removed.
        let root = somewhere_to_write("blocked-by-a-file");
        std::fs::create_dir_all(&root).expect("somewhere to write");
        let in_the_way = root.join("in-the-way");
        std::fs::write(&in_the_way, b"not a directory").expect("a blocking file");
        let into = in_the_way.join("timing");
        let watch = a_process_writing_into(&into, 4);

        watch
            .lock()
            .unwrap()
            .record(Duration::from_millis(3), b"first");
        merge_out(&watch);

        assert!(
            !into.join("report").exists(),
            "a merge that could not even make its directory wrote a report anyway"
        );
        {
            let watch = watch.lock().unwrap();
            assert_eq!(watch.took.runs, 1, "the run was dropped instead of handed back");
            assert_eq!(watch.took.lost, 1, "a run just handed back was not counted as lost");
        }

        watch
            .lock()
            .unwrap()
            .record(Duration::from_millis(4), b"second");
        merge_out(&watch);
        assert_eq!(
            watch.lock().unwrap().took.lost,
            2,
            "a second failed merge should count both runs now stuck, not add to the first count"
        );

        std::fs::remove_file(&in_the_way).expect("clear the way for the directory");
        merge_out(&watch);

        let report = std::fs::read_to_string(into.join("report")).expect("a report");
        assert!(
            report.starts_with("2 runs"),
            "two runs were recorded across two failed merges; the report says {:?}",
            report.lines().next()
        );
        assert!(
            report.contains("2 of those runs were lost to a write failure"),
            "the report does not say what the failed merges cost: {report}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_slow_runs_kept_are_the_slowest_across_every_process() {
        // Each process only ever sees its own draws, so the top has to be taken
        // across the directory rather than within a process. The old rewrite did
        // the opposite: it deleted every slow unit the current process did not
        // itself name.
        // **The two kept have to come one from each process**, or the test is
        // satisfied by a process that deletes everything it did not itself find
        // and happens to hold both. That is how the old rewrite behaved, and a
        // first draft of this test passed against it.
        let into = somewhere_to_write("slowest-across");
        let first = a_process_writing_into(&into, 2);
        let second = a_process_writing_into(&into, 2);

        first
            .lock()
            .unwrap()
            .record(Duration::from_millis(85), b"second slowest of all");
        second
            .lock()
            .unwrap()
            .record(Duration::from_millis(90), b"slowest of all");
        second
            .lock()
            .unwrap()
            .record(Duration::from_millis(10), b"quick");
        merge_out(&first);
        merge_out(&second);

        let kept: HashSet<String> = std::fs::read_dir(into.join("slowest"))
            .expect("a slowest directory")
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(kept.len(), 2, "two are kept, and these are {kept:?}");
        assert!(
            kept.contains(&named_for(Duration::from_millis(90), b"slowest of all")),
            "the slowest run, which the second process found, was not kept: {kept:?}"
        );
        assert!(
            kept.contains(&named_for(
                Duration::from_millis(85),
                b"second slowest of all"
            )),
            "the second slowest run, which the *first* process found, was not kept: {kept:?}"
        );
        let _ = std::fs::remove_dir_all(&into);
    }

    #[test]
    fn one_input_two_processes_both_found_slow_is_kept_once() {
        // Two processes replaying the same corpus entry time it differently, so
        // the same bytes arrive under two names. Only the slower is kept — the
        // rule `TheSlowestRuns::offer` applies inside a process, applied across
        // them.
        let into = somewhere_to_write("same-input");
        let first = a_process_writing_into(&into, 8);
        let second = a_process_writing_into(&into, 8);

        first
            .lock()
            .unwrap()
            .record(Duration::from_millis(30), b"same");
        second
            .lock()
            .unwrap()
            .record(Duration::from_millis(70), b"same");
        merge_out(&first);
        merge_out(&second);

        let kept: Vec<String> = std::fs::read_dir(into.join("slowest"))
            .expect("a slowest directory")
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            kept.len(),
            1,
            "one input was kept under two names: {kept:?}"
        );
        assert_eq!(kept[0], named_for(Duration::from_millis(70), b"same"));
        let _ = std::fs::remove_dir_all(&into);
    }

    #[test]
    fn a_name_says_how_slow_it_was_and_which_input_it_was() {
        // The whole reason the slow runs need no shared accumulation.
        let name = named_for(Duration::from_millis(12), b"bytes");
        let (took, input) = how_slow_the_name_says(&name).expect("a name this module wrote");
        assert_eq!(took, Duration::from_millis(12));
        assert_eq!(
            input,
            how_slow_the_name_says(&named_for(Duration::from_secs(9), b"bytes"))
                .expect("the same input, timed differently")
                .1,
            "the same bytes did not give the same input part"
        );
        assert!(how_slow_the_name_says("something-else").is_none());
    }

    #[test]
    fn what_was_written_out_reads_back_the_same() {
        let mut took = HowLongRunsTook::new();
        for run in [1u64, 1, 5, 900, 6_997_742] {
            took.record(Duration::from_micros(run));
        }
        let into = somewhere_to_write("round-trip");
        std::fs::create_dir_all(&into).expect("somewhere to write");
        std::fs::write(into.join("totals"), took.written_out()).expect("a totals file");

        let back = HowLongRunsTook::read_back(&into.join("totals"));
        assert_eq!(back.runs, took.runs);
        assert_eq!(back.total, took.total);
        assert_eq!(back.buckets, took.buckets);
        let _ = std::fs::remove_dir_all(&into);
    }

    #[test]
    fn a_totals_file_that_cannot_be_read_is_empty_rather_than_a_panic() {
        let into = somewhere_to_write("torn");
        std::fs::create_dir_all(&into).expect("somewhere to write");
        std::fs::write(into.join("totals"), "runs\nb 99999999 1\nb x y\nnonsense\n")
            .expect("a torn file");
        assert_eq!(HowLongRunsTook::read_back(&into.join("totals")).runs, 0);
        let _ = std::fs::remove_dir_all(&into);
    }

    #[test]
    fn adding_two_histograms_adds_their_runs_and_their_buckets() {
        let mut first = HowLongRunsTook::new();
        let mut second = HowLongRunsTook::new();
        for run in 1..=50 {
            first.record(Duration::from_millis(run));
        }
        for run in 51..=100 {
            second.record(Duration::from_millis(run));
        }
        first.add(&second);

        let mut all_at_once = HowLongRunsTook::new();
        for run in 1..=100 {
            all_at_once.record(Duration::from_millis(run));
        }
        assert_eq!(first.runs, all_at_once.runs);
        assert_eq!(first.total, all_at_once.total);
        assert_eq!(first.buckets, all_at_once.buckets);
    }

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
        let into = somewhere_to_write("names-what-it-wrote");
        let watch = a_process_writing_into(&into, 2);
        for run in [1u64, 40, 90] {
            watch
                .lock()
                .unwrap()
                .record(Duration::from_millis(run), &[run as u8]);
        }
        merge_out(&watch);

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
    fn a_file_this_module_did_not_write_is_left_alone() {
        let into = somewhere_to_write("a-stranger");
        let slowest = into.join("slowest");
        std::fs::create_dir_all(&slowest).expect("somewhere to write");
        std::fs::write(slowest.join("something-else"), b"not ours").expect("a stranger");

        let watch = a_process_writing_into(&into, 1);
        watch
            .lock()
            .unwrap()
            .record(Duration::from_millis(9), b"new");
        merge_out(&watch);

        assert!(
            slowest.join("something-else").exists(),
            "a file this module did not write was removed"
        );
        let _ = std::fs::remove_dir_all(&into);
    }
}
