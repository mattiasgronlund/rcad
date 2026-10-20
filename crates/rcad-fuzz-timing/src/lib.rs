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
//! Three things, because they answer three different questions.
//!
//! A **histogram**, sixteen buckets to the octave, so a percentile can be read
//! off at the end. It is fixed-size and costs one add per run, which is what
//! lets it run over the reader's quarter of a billion draws.
//!
//! The **slowest runs themselves**, kept in full so they can be handed straight
//! back to `cargo fuzz run`. A percentile names how slow the slow ones were; it
//! cannot name *which*, and which is the thing somebody then goes and reads.
//!
//! One job's own **start time and the gap its last periodic merge closed**,
//! one small file per process. `fuzz.log` records only a job's ending, not its
//! start, and a `-timeout`, a crash or an out-of-memory ends in `_Exit`, which
//! runs no destructor — so without this, a job lost that way left nothing on
//! disk at all. Written at every periodic merge rather than only when
//! [`MergeWhenThisThreadEnds`] runs, so the file still holds the job's last
//! known tick even where that destructor never does (§269.8).
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
//! even the clock. `RCAD_FUZZ_SLOWEST` is how many of the slowest runs to keep,
//! and defaults to [`HOW_MANY_SLOW_RUNS_TO_KEEP`].
//!
//! # With it named, libFuzzer must not be able to tell
//!
//! The night fuzzes with the directory named, so whatever this crate does is
//! part of the process libFuzzer is steering. Four things here used to steer
//! it (§267): code compiled into the fuzz target that ran only when the
//! directory was named, allocations a draw made and kept, a thread of its own,
//! and a `memcmp` of each draw against the slow inputs held. Each is gone, and
//! why is written where it would come back: [`timed`],
//! `TheSlowestRuns::keeping`, `HOW_OFTEN_TO_MERGE` and `SlowRun::is`. Moving
//! the merge into the draw made two more things matter. One is the hash keys
//! of the draw's thread, and `prune_to_the_slowest_kept` says why. The other
//! is how the merge matches words and file names, and `read_back` and
//! `how_slow_the_name_says` say why they do it byte by byte (§273). A third
//! thing came from moving the merge into the draw rather than a thread: a
//! child that ends normally never returns from the draw it is in, so a run
//! since the last merge was stuck in that process's memory. `HOW_OFTEN_TO_MERGE`
//! and `MergeWhenThisThreadEnds` say how that run reaches `totals` anyway
//! (§278).
//!
//! What is left is this crate's own code, which libFuzzer still counts as
//! coverage unless the build leaves it uninstrumented — and that is the
//! build's job, not this file's (§267).

#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock, PoisonError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
/// **Two calls and the draw, and nothing else**, because this function is
/// generic and so is compiled into the fuzz target, whose code libFuzzer counts
/// as coverage. Anything here that went one way with `RCAD_FUZZ_TIMINGS` named
/// and another without it — a lock, a first-call branch, the arithmetic on two
/// clock readings — would be edges and comparisons libFuzzer sees only when the
/// night is measuring. `start_the_clock` and `stop_the_clock` hold all of it,
/// and are neither generic nor inlined, so they are compiled here once.
///
/// With the directory unset those two are a resolved `OnceLock` read and a
/// return, so the targets nobody is measuring pay nothing.
pub fn timed<T>(target: &str, bytes: &[u8], run: impl FnOnce() -> T) -> T {
    let started = start_the_clock(target);
    let answer = run();
    stop_the_clock(started, bytes);
    answer
}

/// When this draw started, where a directory has been named.
#[inline(never)]
fn start_the_clock(target: &str) -> Option<Instant> {
    WATCH
        .get_or_init(|| {
            let watch = Watch::asked_for(target).map(Mutex::new);
            if watch.is_some() {
                MERGE_WHEN_THIS_THREAD_ENDS.with(|_| {});
            }
            watch
        })
        .as_ref()
        .map(|_| Instant::now())
}

/// A merge made once more when this thread ends, so a run counted since the
/// last due merge is not lost when the process ends normally.
///
/// **How a child ends decides whether this can save anything** — see
/// [`HOW_OFTEN_TO_MERGE`]. A `-max_total_time` or the stop file ends a child
/// with `exit(0)`, which runs the calling thread's `thread_local!`
/// destructors, so a merge from here reaches `totals` before the process is
/// gone. A `-timeout`, a crash or an out-of-memory ends in `_Exit`, which
/// runs no destructors, so this closes the first gap and does nothing for
/// the second (§278).
///
/// **Touched once, from inside the same [`OnceLock::get_or_init`] that makes
/// [`WATCH`]** — the empty input libFuzzer runs once at start-up and never
/// checks for a leak — so registering this destructor costs nothing on every
/// other draw, and nothing at all where `RCAD_FUZZ_TIMINGS` is unset.
struct MergeWhenThisThreadEnds;

impl Drop for MergeWhenThisThreadEnds {
    fn drop(&mut self) {
        if let Some(Some(watch)) = WATCH.get() {
            let mut watch = watch.lock().unwrap_or_else(PoisonError::into_inner);
            watch.merge_out();
            watch.forget_job_progress();
        }
    }
}

thread_local! {
    static MERGE_WHEN_THIS_THREAD_ENDS: MergeWhenThisThreadEnds = const { MergeWhenThisThreadEnds };
}

/// Count the draw that `started` began, and merge if one is due.
///
/// Only a draw that returns gets here, so a draw that panics — which is what a
/// fuzz target is for — neither poisons the lock nor counts its own
/// half-finished time.
#[inline(never)]
fn stop_the_clock(started: Option<Instant>, bytes: &[u8]) {
    let Some(started) = started else {
        return;
    };
    let now = Instant::now();
    WATCH
        .get()
        .and_then(Option::as_ref)
        .expect("a clock is only started where the watch it was started for exists")
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .record(now.duration_since(started), bytes, now);
}

/// How long to leave between merges into the shared accumulation.
///
/// **A second, where the report it replaced was rewritten every thirty.** A
/// process only ever loses what it has seen since its last merge, and under
/// `-fork` a process lives about seventeen seconds — so at thirty seconds most
/// children would exit having merged nothing at all, and the loss would be the
/// whole night rather than the edge of it.
///
/// **How a child ends decides whether that last second could be saved.**
/// libFuzzer's parent never kills a child. In libfuzzer-sys 0.4.13,
/// `WorkerThread` waits for each one to finish (`FuzzerFork.cpp:303-309`),
/// and at the end of the night `StopJobs` only writes a stop file
/// (`:368-373`). A child that reaches its `-max_total_time` or finds the stop
/// file ends with `exit(0)` (`FuzzerDriver.cpp:930`), which runs the functions
/// registered to run at exit and, on Linux, the exiting thread's thread-local
/// destructors — [`MergeWhenThisThreadEnds`] merges from one of those, so
/// that second is saved (§278). A `-timeout`, a crash or an out-of-memory
/// ends in `_Exit` (`FuzzerLoop.cpp:307`, `:238`, `:135` and `:322`), which
/// runs nothing, so what those lose cannot be saved: that loss is a limit.
///
/// **The merge is made by the draw that finds it due, on libFuzzer's own
/// thread**, and not by a thread of this crate's ticking beside it (§267).
/// libFuzzer re-runs any draw that ends with more `malloc`s than `free`s, to
/// look for a leak, and it counts every thread's: a merge on another thread
/// that allocated during one draw and freed during the next made libFuzzer run
/// inputs again at whatever moments the clock chose. A merge inside a draw
/// frees everything it allocated before the draw ends.
///
/// That costs one thing the thread did not: a process that times out in the
/// middle of a draw loses the runs since its last merge, up to a second of
/// them, where the thread went on merging them while the long draw ran
/// (§262.2). The draw that timed out was never counted either way. §267.6
/// sizes the cost on one night.
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
    last_merged: Instant,
    /// When this process's own watch was made — its job's start, in wall-clock
    /// time, so it can be lined up against `fuzz.log`'s.
    started: SystemTime,
    /// How long the periodic merge that just closed had been waiting since the
    /// one before it. Set in [`Watch::record`], before `last_merged` is moved
    /// on, so it is the gap that merge closed and not the (near-zero) one
    /// still open.
    last_merge_gap: Duration,
}

impl Watch {
    /// The watch `RCAD_FUZZ_TIMINGS` asks for, or `None` where it says nothing.
    fn asked_for(target: &str) -> Option<Self> {
        let root = std::env::var_os("RCAD_FUZZ_TIMINGS")?;
        let keep = std::env::var("RCAD_FUZZ_SLOWEST")
            .ok()
            .and_then(|how_many| how_many.parse().ok())
            .unwrap_or(HOW_MANY_SLOW_RUNS_TO_KEEP);
        Some(Self::writing_into(Path::new(&root).join(target), keep))
    }

    fn writing_into(into: PathBuf, keep: usize) -> Self {
        Self {
            into,
            keep,
            took: HowLongRunsTook::new(),
            slowest: TheSlowestRuns::keeping(keep),
            last_merged: Instant::now(),
            started: SystemTime::now(),
            last_merge_gap: Duration::ZERO,
        }
    }

    /// One run, counted, kept where it is among the slowest, and merged out
    /// with everything since the last merge once [`HOW_OFTEN_TO_MERGE`] has
    /// passed by `now`.
    fn record(&mut self, took: Duration, bytes: &[u8], now: Instant) {
        self.took.record(took);
        self.slowest.offer(took, bytes);
        if now.duration_since(self.last_merged) >= HOW_OFTEN_TO_MERGE {
            self.last_merge_gap = now.duration_since(self.last_merged);
            self.last_merged = now;
            self.merge_out();
        }
    }

    /// Add what this process has seen to the shared accumulation, and rewrite
    /// the report from the result.
    ///
    /// Two failures are recovered rather than swallowed: if the directory
    /// cannot be made or the shared lock cannot be taken, the run count stays
    /// where it is so the next merge tries again, and no slow input is marked
    /// as written — nothing was written, so the next merge offers every one of
    /// them again. Every run still waiting then counts as lost — *set*, not
    /// added to, so a run stuck across three failed merges in a row is one run
    /// lost, not three. The same recovery covers a slow input's own write
    /// failing on its own, past that point: only an input [`write_slow_input`]
    /// actually lands under its real name is marked written, so one that did
    /// not is retried on the next merge rather than refused forever.
    ///
    /// Every file here is replaced whole or not at all ([`replace_whole`]), so
    /// a write cut short by a full disk or by the process ending mid-merge
    /// leaves the old file where it was. That matters most for `totals`,
    /// which every process for the target reads back and adds to: written in
    /// place, a process that ended between emptying it and writing it left it
    /// empty, `read_back` read that as no runs, and the next merge started
    /// the whole target's count again from its own runs, `lost` included.
    /// What is still swallowed is a failure to write `totals` or the report:
    /// that loses just that one merge's counts and its report, since the old
    /// `totals` stays until the rename and the slowest are untouched by
    /// either. A fuzz target that stopped fuzzing because its measurements
    /// could not be filed would be measuring the disk.
    ///
    /// Everything this allocates is freed before it returns, which is what
    /// lets it run inside a draw ([`HOW_OFTEN_TO_MERGE`]).
    fn merge_out(&mut self) {
        self.write_job_progress();

        if self.took.runs == 0 && self.slowest.all_written() {
            return;
        }

        let slowest = self.into.join("slowest");
        if std::fs::create_dir_all(&slowest).is_err() {
            self.took.lost = self.took.runs;
            return;
        }
        let Some(lock) = the_shared_report_locked(&self.into) else {
            self.took.lost = self.took.runs;
            return;
        };

        // What is merged is a delta, and it is cleared once it is in, so a
        // process that merges twice does not count its runs twice. The slowest
        // are *not* cleared: they are the process's running top, and dropping
        // them would make every merge re-offer the whole corpus.
        let mut everyones = HowLongRunsTook::read_back(&self.into.join("totals"));
        everyones.add(&self.took);
        let _ = replace_whole(
            &self.into.join(TOTALS_BEING_WRITTEN),
            &self.into.join("totals"),
            everyones.written_out().as_bytes(),
        );
        self.took.clear();

        self.slowest.write_what_is_not_on_disk(&slowest);

        let kept = prune_to_the_slowest_kept(&slowest, self.keep);
        let _ = replace_whole(
            &self.into.join(REPORT_BEING_WRITTEN),
            &self.into.join("report"),
            report_text(&everyones, &kept).as_bytes(),
        );

        let _ = lock.unlock();
    }

    /// This job's own start time and the gap its last periodic merge closed,
    /// written at every merge rather than only when
    /// [`MergeWhenThisThreadEnds`] runs — which a `-timeout`, a crash or an
    /// out-of-memory's `_Exit` skips entirely, destructor and all (§269.8,
    /// Q2383). `fuzz.log` records only a job's ending; this is what a reader
    /// lines that ending up against, to ask whether a lost job's runs fit
    /// inside one periodic-merge gap or not.
    ///
    /// One file per process, named for its pid, so no two processes ever
    /// write the same name and this needs none of
    /// [`the_shared_report_locked`]'s lock. [`Watch::forget_job_progress`]
    /// removes it once a process reaches that destructor, since a job that
    /// got there is already folded into `totals` in full — only a job this
    /// crate never heard exit again is worth a file.
    fn write_job_progress(&self) {
        let jobs = self.into.join("jobs");
        if std::fs::create_dir_all(&jobs).is_err() {
            return;
        }
        let pid = std::process::id();
        let text = format!(
            "started {}\nlast_merged {}\ngap {}\n",
            nanos_since_epoch(self.started),
            nanos_since_epoch(SystemTime::now()),
            self.last_merge_gap.as_nanos(),
        );
        let _ = replace_whole(
            &jobs.join(format!("{pid}-being-written")),
            &jobs.join(pid.to_string()),
            text.as_bytes(),
        );
    }

    /// Remove this job's own progress file. Called once
    /// [`Watch::merge_out`] has folded its last runs into `totals` on the one
    /// path ([`MergeWhenThisThreadEnds`]'s `Drop`) that can still run it: a
    /// job that reaches its own destructor needs no record of where it was
    /// mid-run, since `totals` already has all of it.
    fn forget_job_progress(&self) {
        let pid = std::process::id();
        let _ = std::fs::remove_file(self.into.join("jobs").join(pid.to_string()));
    }
}

/// Nanoseconds since the Unix epoch, so a job's wall-clock progress lines up
/// with `fuzz.log`'s own clock (§269.8). A clock set before 1970 reads as the
/// epoch itself rather than panicking — not a case a fuzzing machine is in.
fn nanos_since_epoch(when: SystemTime) -> u128 {
    when.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

/// Write one slow input's bytes under its real name, so a reader never sees
/// fewer bytes than the input actually had: `cargo fuzz run` on a short one
/// replays something other than what was slow. §236.6.
///
/// A leftover under the temporary name, from a process that ended mid-write
/// with nothing written after it, is never mistaken for a slow run: it does
/// not start with `slow-`, so `how_slow_the_name_says` does not parse it and
/// [`prune_to_the_slowest_kept`] leaves it alone the same way it leaves any
/// file this module did not write.
fn write_slow_input(slowest: &Path, took: Duration, bytes: &[u8]) -> bool {
    replace_whole(
        &slowest.join(SLOW_INPUT_BEING_WRITTEN),
        &slowest.join(named_for(took, bytes)),
        bytes,
    )
}

/// The temporary name [`write_slow_input`] writes under before renaming.
///
/// Never `slow-...`, the shape every real name has, so a leftover is never
/// read as one.
const SLOW_INPUT_BEING_WRITTEN: &str = "being-written";

/// Put `bytes` under `real`, whole or not at all, by writing them to `temp`
/// in the same directory and renaming that onto `real`.
///
/// `std::fs::write` cannot do this: it empties the file when it opens it and
/// writes after, so a write cut short by a full disk, or a process that ends
/// in between, leaves a short or empty file under the real name. And a
/// process under `-fork` can end at any moment: a `-timeout`, a crash and an
/// out-of-memory all end in `_Exit`, which finishes nothing, and an
/// out-of-memory can come from libFuzzer's own thread while this one is
/// mid-write. A rename within one directory either lands whole or does not
/// happen.
///
/// Each `temp` is a fixed name, not one per call, because every caller holds
/// the one lock every process for this target shares. So there is never
/// more than one writer at a time, and a leftover from a process that ended
/// mid-write is overwritten by the next write rather than piling up one file
/// per crash.
fn replace_whole(temp: &Path, real: &Path, bytes: &[u8]) -> bool {
    write_that_might_be_cut_short(temp, bytes) && std::fs::rename(temp, real).is_ok()
}

/// The temporary name `totals` is written under before renaming.
const TOTALS_BEING_WRITTEN: &str = "totals-being-written";

/// The temporary name the report is written under before renaming.
const REPORT_BEING_WRITTEN: &str = "report-being-written";

/// Write `bytes` to `path` in one call, the way `std::fs::write` does.
#[cfg(not(test))]
fn write_that_might_be_cut_short(path: &Path, bytes: &[u8]) -> bool {
    std::fs::write(path, bytes).is_ok()
}

#[cfg(test)]
thread_local! {
    /// Set by a test to make the next `write_that_might_be_cut_short` act
    /// like a `write(2)` that only got this many bytes out before the
    /// process died or the disk filled — a fault no unit test can cause
    /// for real.
    static FAULT_INJECTED_SHORT_WRITE: std::cell::Cell<Option<usize>> =
        const { std::cell::Cell::new(None) };
}

/// `write_that_might_be_cut_short`, but cut short when a test has set
/// `FAULT_INJECTED_SHORT_WRITE`.
#[cfg(test)]
fn write_that_might_be_cut_short(path: &Path, bytes: &[u8]) -> bool {
    if let Some(budget) = FAULT_INJECTED_SHORT_WRITE.with(std::cell::Cell::take) {
        let _ = std::fs::write(path, &bytes[..budget.min(bytes.len())]);
        return false;
    }
    std::fs::write(path, bytes).is_ok()
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
///
/// That "something else" includes the temporary name [`write_slow_input`]
/// writes under before it renames: `how_slow_the_name_says` never parses it,
/// since it does not start with `slow-`, so it is never counted, never
/// kept, and never named in [`report_text`] — the same as any file this
/// module did not write.
///
/// **Names are sorted and matched by the numbers in them, never compared as
/// text**, for the reason [`SlowRun::is`] gives: this runs inside a draw, and a
/// `memcmp` of two different names would hand both to libFuzzer's mutations.
///
/// **The inputs already kept are held in a `BTreeSet`, never a `HashSet`**, for
/// the same reason. Each `RandomState` made on a thread steps the keys of the
/// next one made there, so one made here would change the order the target's
/// own hash maps iterate in, at a moment the clock picked (§267).
fn prune_to_the_slowest_kept(from: &Path, keep: usize) -> Vec<(Duration, String)> {
    let Ok(listing) = std::fs::read_dir(from) else {
        return Vec::new();
    };
    let mut found: Vec<(Duration, u64, String)> = Vec::new();
    for entry in listing.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Some((took, input)) = how_slow_the_name_says(&name) {
            found.push((took, input, name));
        }
    }
    found.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));

    let mut kept: Vec<(Duration, String)> = Vec::new();
    let mut held: BTreeSet<u64> = BTreeSet::new();
    for (took, input, name) in found {
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
    format!("slow-{:012}us-{:016x}", took.as_micros(), hash_of(bytes))
}

/// The FNV-1a hash of an input, which names it on disk and tells two apart.
fn hash_of(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// How slow a slow run's file name says it was, and the hash of its input.
///
/// The inverse of [`named_for`], and the reason the slow runs need no shared
/// accumulation of their own.
///
/// **The `slow-` and the `us-` are found byte by byte**, not with
/// `strip_prefix` and `split_once`. This runs inside a draw, and both of those
/// call `bcmp`, which ASan hands to libFuzzer's mutations whenever it comes out
/// unequal ([`SlowRun::is`] says why that matters). `split_once` did so on
/// every name: before it searches for `us-`, `core` compares `us` with `s-`
/// (§273).
fn how_slow_the_name_says(name: &str) -> Option<(Duration, u64)> {
    let [b's', b'l', b'o', b'w', b'-', ..] = name.as_bytes() else {
        return None;
    };
    let rest = &name["slow-".len()..];
    let at = rest
        .as_bytes()
        .windows(3)
        .position(|three| matches!(three, [b'u', b's', b'-']))?;
    let (micros, input) = (&rest[..at], &rest[at + "us-".len()..]);
    Some((
        Duration::from_micros(micros.parse().ok()?),
        u64::from_str_radix(input, 16).ok()?,
    ))
}

/// How many runs landed in each bucket of nanoseconds.
struct HowLongRunsTook {
    runs: u64,
    total: Duration,
    buckets: Box<[u64; HOW_MANY_BUCKETS]>,
    /// Of `runs`, how many needed more than one merge because an earlier one
    /// failed to write. Not a count of runs still missing — every run counted
    /// here is also counted in `runs`, since a failed [`Watch::merge_out`] is
    /// what sets this before the retry that eventually succeeds.
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

    /// Back to no runs at all, in place: a merge runs inside a draw, and a
    /// fresh histogram would be an allocation it has to free again.
    fn clear(&mut self) {
        self.runs = 0;
        self.total = Duration::ZERO;
        self.buckets.fill(0);
        self.lost = 0;
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
    ///
    /// **Each line's first word is matched byte by byte**, not against a
    /// `&str` pattern. This runs inside a draw, and a `&str` pattern compiles
    /// to a `bcmp`, which ASan hands to libFuzzer's mutations whenever it comes
    /// out unequal ([`SlowRun::is`] says why that matters). The `lost` line was
    /// compared with `"runs"` first, on every merge (§273).
    fn read_back(from: &Path) -> Self {
        let mut took = Self::new();
        let Ok(text) = std::fs::read_to_string(from) else {
            return took;
        };
        for line in text.lines() {
            let mut words = line.split_whitespace();
            match (words.next().map(str::as_bytes), words.next(), words.next()) {
                (Some([b'r', b'u', b'n', b's']), Some(runs), None) => {
                    took.runs = runs.parse().unwrap_or(0);
                }
                (Some([b'n', b'a', b'n', b'o', b's']), Some(nanos), None) => {
                    took.total = Duration::from_nanos(
                        u64::try_from(nanos.parse::<u128>().unwrap_or(0)).unwrap_or(u64::MAX),
                    );
                }
                (Some([b'l', b'o', b's', b't']), Some(lost), None) => {
                    took.lost = lost.parse().unwrap_or(0);
                }
                (Some([b'b']), Some(bucket), Some(count)) => {
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
    runs: Vec<SlowRun>,
    /// A buffer for every run not yet kept, so `runs.len() + spare.len()` is
    /// always `keep`.
    spare: Vec<Vec<u8>>,
}

/// One of the slowest runs.
struct SlowRun {
    took: Duration,
    bytes: Vec<u8>,
    /// [`hash_of`] `bytes`.
    hash: u64,
    /// Whether this run is on disk under the name `took` and `bytes` give it,
    /// so a steady top is not rewritten on every merge.
    written: bool,
}

impl SlowRun {
    /// Whether this run's input is `bytes`, whose hash is `hash`.
    ///
    /// **The hashes are compared first, and the bytes only where they match**,
    /// so two different inputs never go through `memcmp`. ASan's `memcmp`
    /// hands libFuzzer both sides of every comparison that comes out unequal,
    /// from any code in the process, and libFuzzer keeps them to mutate with.
    /// Comparing each draw with the slow inputs held — which ones are held is
    /// the clock's choice — made the mutations depend on the clock (§267).
    fn is(&self, hash: u64, bytes: &[u8]) -> bool {
        self.hash == hash && self.bytes == bytes
    }
}

/// How many bytes each spare buffer starts with room for.
///
/// Anything above zero: see [`TheSlowestRuns::keeping`].
const ROOM_A_SPARE_BUFFER_STARTS_WITH: usize = 64;

impl TheSlowestRuns {
    /// Room for `keep` runs, **every buffer allocated now, in the draw that
    /// made the watch**, and never again after it.
    ///
    /// libFuzzer re-runs a draw that ended with more `malloc`s than `free`s, to
    /// look for a leak, and counts the re-run against `-runs`. Copying a slow
    /// input into a fresh `Vec` is one `malloc` and no `free`, so the first
    /// `keep` slow inputs of every process were each run twice, and only when
    /// the night was measuring (§267). Copying it into a buffer that already
    /// exists is nothing at all, or a `realloc` — one of each — when the input
    /// is longer than the buffer. So each buffer starts with room for
    /// something, because growing an empty `Vec` is a `malloc` alone.
    ///
    /// The draw that makes the watch is the first one, which is the empty
    /// input libFuzzer runs once at start-up and never looks at for leaks.
    fn keeping(keep: usize) -> Self {
        Self {
            keep,
            runs: Vec::with_capacity(keep),
            spare: (0..keep)
                .map(|_| Vec::with_capacity(ROOM_A_SPARE_BUFFER_STARTS_WITH))
                .collect(),
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
        if self.runs.len() == self.keep && took <= self.runs[0].took {
            return false;
        }
        let hash = hash_of(bytes);
        let mut run = if let Some(held) = self.runs.iter().position(|run| run.is(hash, bytes)) {
            if self.runs[held].took >= took {
                return false;
            }
            self.runs.remove(held)
        } else {
            let mut buffer = if self.runs.len() == self.keep {
                self.runs.remove(0).bytes
            } else {
                self.spare
                    .pop()
                    .expect("a spare buffer for every run not yet kept")
            };
            buffer.clear();
            buffer.extend_from_slice(bytes);
            SlowRun {
                took,
                bytes: buffer,
                hash,
                written: false,
            }
        };
        run.took = took;
        run.written = false;
        let at = self.runs.partition_point(|seen| seen.took <= took);
        self.runs.insert(at, run);
        true
    }

    fn all_written(&self) -> bool {
        self.runs.iter().all(|run| run.written)
    }

    /// Write every kept run not yet on disk, and mark the ones that landed.
    fn write_what_is_not_on_disk(&mut self, slowest: &Path) {
        for run in self.runs.iter_mut().rev().filter(|run| !run.written) {
            run.written = write_slow_input(slowest, run.took, &run.bytes);
        }
    }

    /// What is kept, slowest first.
    #[cfg(test)]
    fn slowest_first(&self) -> impl Iterator<Item = (Duration, &[u8])> {
        self.runs
            .iter()
            .rev()
            .map(|run| (run.took, run.bytes.as_slice()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::hash::{BuildHasher, RandomState};

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
    fn a_process_writing_into(into: &Path, keep: usize) -> Watch {
        Watch::writing_into(into.to_path_buf(), keep)
    }

    impl Watch {
        /// One run, counted the way a draw counts it, with no merge due yet.
        fn ran(&mut self, took: Duration, bytes: &[u8]) {
            self.record(took, bytes, self.last_merged);
        }
    }

    #[test]
    fn a_merge_that_is_due_is_made_before_the_draw_that_found_it_returns() {
        // §267: the merge used to be made by a thread of this crate's own,
        // ticking beside libFuzzer. Its allocations landed in whichever draw
        // was running, and libFuzzer re-runs a draw that ends with more
        // `malloc`s than `free`s — so with the timings named, libFuzzer ran
        // inputs again at moments the clock chose. A merge made inside the
        // draw, and finished before it returns, cannot do that.
        let into = somewhere_to_write("merge-in-the-draw");
        let mut watch = a_process_writing_into(&into, 4);
        let later = watch.last_merged + HOW_OFTEN_TO_MERGE;

        watch.record(Duration::from_millis(2), b"not due yet", watch.last_merged);
        assert!(
            !into.join("report").exists(),
            "a merge was made before a second had passed"
        );

        watch.record(Duration::from_millis(3), b"due now", later);
        let report = std::fs::read_to_string(into.join("report"))
            .expect("the draw that found a merge due returned without making it");
        assert!(
            report.starts_with("2 runs"),
            "the merge made in the draw did not carry both runs: {:?}",
            report.lines().next()
        );

        watch.record(Duration::from_millis(4), b"just merged", later);
        assert_eq!(
            watch.took.runs, 1,
            "a second merge was made in the same second as the first"
        );
        let _ = std::fs::remove_dir_all(&into);
    }

    #[test]
    fn a_jobs_start_and_last_merge_are_written_at_every_periodic_merge_not_only_at_exit() {
        // §269.8, Q2383: a `-timeout`, a crash or an out-of-memory ends in
        // `_Exit`, which runs no destructor — `MergeWhenThisThreadEnds`
        // included — so a job lost that way used to leave nothing behind
        // saying when it started or how recently it had last merged.
        // `fuzz.log` records only a job's own ending, and Q2220 found that
        // rebuilding a start from completion order is unreliable. Writing
        // this at every periodic merge, not only at the destructor, means a
        // killed job's file still holds its last tick.
        let into = somewhere_to_write("job-progress");
        let mut watch = a_process_writing_into(&into, 4);
        let job_file = into.join("jobs").join(std::process::id().to_string());
        assert!(
            !job_file.exists(),
            "a job's progress was written before its very first periodic merge"
        );

        let due = watch.last_merged + HOW_OFTEN_TO_MERGE;
        watch.record(Duration::from_millis(2), b"first tick", due);
        let first = std::fs::read_to_string(&job_file).expect(
            "the job's start and last-merge time were not written at its first periodic merge",
        );
        assert!(first.starts_with("started "), "{first}");
        assert!(first.contains("last_merged "), "{first}");
        assert!(first.contains("gap "), "{first}");

        std::thread::sleep(Duration::from_millis(2));
        let due_again = due + HOW_OFTEN_TO_MERGE;
        watch.record(Duration::from_millis(2), b"second tick", due_again);
        let second = std::fs::read_to_string(&job_file)
            .expect("a job progress file after a second periodic merge");
        assert_ne!(
            first, second,
            "the job's last-merge time did not move between two periodic merges, \
             so it was written once rather than continuously"
        );
        let _ = std::fs::remove_dir_all(&into);
    }

    #[test]
    fn a_jobs_progress_file_is_forgotten_once_its_destructor_merges_it_for_the_last_time() {
        // A job that reaches `MergeWhenThisThreadEnds`'s `Drop` is already
        // folded into `totals` in full, so its progress file would do
        // nothing but accumulate — one for every one of the forty thousand
        // processes a `-fork` night runs (§1-§78 of this file's own doc
        // comment) — if nothing ever removed it.
        let into = somewhere_to_write("job-progress-forgotten");
        let watch = a_process_writing_into(&into, 4);
        let job_file = into.join("jobs").join(std::process::id().to_string());
        watch.write_job_progress();
        assert!(
            job_file.exists(),
            "the progress file should exist right after a periodic merge"
        );

        watch.forget_job_progress();
        assert!(
            !job_file.exists(),
            "the progress file should be gone once the job's destructor has merged it for the last time"
        );
        let _ = std::fs::remove_dir_all(&into);
    }

    #[test]
    fn keeping_a_slow_run_uses_a_buffer_set_aside_when_the_watch_was_made() {
        // §267: copying a slow input into a fresh `Vec` is a `malloc` with no
        // `free`, and libFuzzer re-runs any draw that ends that way — so the
        // first sixty-four slow inputs of every process were run twice, and
        // only when the timings were named. Every buffer is made up front
        // instead, and passed from run to run.
        let mut slowest = TheSlowestRuns::keeping(3);
        let set_aside: HashSet<*const u8> =
            slowest.spare.iter().map(|buffer| buffer.as_ptr()).collect();
        assert_eq!(set_aside.len(), 3, "three buffers, each its own");
        assert!(
            slowest.spare.iter().all(|buffer| buffer.capacity() > 0),
            "an empty buffer's first copy would be a malloc alone"
        );

        for (run, bytes) in [
            (5u64, &b"five"[..]),
            (1, b"one"),
            (9, b"nine"),
            (3, b"three"),
            (7, b"seven"),
            (8, b"eight"),
            (9, b"five"),
        ] {
            slowest.offer(Duration::from_millis(run), bytes);
        }

        let kept: Vec<_> = slowest
            .slowest_first()
            .map(|(took, bytes)| (took.as_millis(), bytes.to_vec()))
            .collect();
        assert_eq!(
            kept,
            vec![
                (9, b"five".to_vec()),
                (9, b"nine".to_vec()),
                (8, b"eight".to_vec())
            ]
        );
        for run in &slowest.runs {
            assert!(
                set_aside.contains(&run.bytes.as_ptr()),
                "a kept run's bytes are in a buffer made after the watch was"
            );
        }
        assert_eq!(slowest.runs.len() + slowest.spare.len(), 3);
    }

    #[test]
    fn two_inputs_are_told_apart_by_their_hashes_before_their_bytes() {
        // §267: comparing a draw's bytes with a different held input's bytes
        // is a `memcmp` that comes out unequal, and ASan hands both sides of
        // that to libFuzzer's mutations. A run whose hash does not match is
        // not the same input, whatever its bytes say — so the bytes are never
        // looked at. The hash here is deliberately not the bytes' own, which
        // only a comparison that never reached the bytes can get right.
        let held = SlowRun {
            took: Duration::from_millis(9),
            bytes: b"same bytes".to_vec(),
            hash: hash_of(b"same bytes") ^ 1,
            written: false,
        };
        assert!(
            !held.is(hash_of(b"same bytes"), b"same bytes"),
            "the bytes were compared although the hashes differ"
        );
        assert!(held.is(held.hash, b"same bytes"));
        assert!(!held.is(held.hash, b"other bytes"));
    }

    #[test]
    fn a_draw_that_merges_leaves_the_hash_keys_of_its_thread_where_it_found_them() {
        // §267: every `RandomState` made on a thread steps the keys the next
        // one made there gets, and so the order the target's own hash maps
        // iterate in. The merge runs inside a draw, at a moment the clock
        // picks. If it made a `RandomState`, the clock would pick when the
        // target's hash maps change their order, and only with the timings
        // named. Two names for one input make the merge prune a duplicate.
        let into = somewhere_to_write("hash-keys");
        let slowest = into.join("slowest");
        std::fs::create_dir_all(&slowest).expect("somewhere to write");
        for took in [30, 70] {
            std::fs::write(
                slowest.join(named_for(Duration::from_millis(took), b"same")),
                b"same",
            )
            .expect("a slow input");
        }

        let before = the_hash_key_this_thread_hands_out_next();
        let mut watch = a_process_writing_into(&into, 4);
        let due = watch.last_merged + HOW_OFTEN_TO_MERGE;
        watch.record(Duration::from_millis(5), b"a draw", due);
        let after = the_hash_key_this_thread_hands_out_next();

        assert!(
            !slowest
                .join(named_for(Duration::from_millis(30), b"same"))
                .exists(),
            "the draw did not merge and prune, so this measured nothing"
        );
        assert_eq!(
            after,
            before.wrapping_add(1),
            "the draw made {} `RandomState`s on its thread",
            after.wrapping_sub(before).wrapping_sub(1)
        );
        let _ = std::fs::remove_dir_all(&into);
    }

    /// The first key of the `RandomState` this thread hands out next.
    ///
    /// std keeps two keys per thread and adds one to the first each time a
    /// `RandomState` is made there (`std/src/hash/random.rs`), so the keys of
    /// two readings say how many were made between them. Nothing public
    /// returns the key, but a `DefaultHasher`'s `Debug` prints it.
    fn the_hash_key_this_thread_hands_out_next() -> u64 {
        let hasher = format!("{:?}", RandomState::new().build_hasher());
        hasher
            .split_once("k0: ")
            .and_then(|(_, rest)| rest.split(',').next())
            .and_then(|key| key.parse().ok())
            .unwrap_or_else(|| panic!("a DefaultHasher no longer prints its first key: {hasher}"))
    }

    #[test]
    fn two_processes_writing_at_once_keep_both_their_runs() {
        // The regression. Under `-fork` the target is forty thousand short-lived
        // processes over a night, every one of them pointed at this directory.
        // Each used to write its own report over the last one's, so a measured
        // 54 845 runs across 12 jobs left a report reading `1 runs`.
        let into = somewhere_to_write("two-processes");
        let mut first = a_process_writing_into(&into, 4);
        let mut second = a_process_writing_into(&into, 4);

        for run in 1..=3u64 {
            first.ran(Duration::from_millis(run), &[run as u8]);
        }
        for run in 4..=9u64 {
            second.ran(Duration::from_millis(run), &[run as u8]);
        }
        first.merge_out();
        second.merge_out();

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
        // A merge is made once a second for the whole life of a process, so
        // all but the first merge of every process adds a delta on top of what
        // that same process already put in.
        let into = somewhere_to_write("merge-twice");
        let mut watch = a_process_writing_into(&into, 4);

        watch.ran(Duration::from_millis(5), b"once");
        watch.merge_out();
        watch.merge_out();
        watch.ran(Duration::from_millis(5), b"twice");
        watch.merge_out();

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
        let mut watch = a_process_writing_into(&into, 4);

        watch.ran(Duration::from_millis(3), b"first");
        watch.merge_out();

        assert!(
            !into.join("report").exists(),
            "a merge that could not even make its directory wrote a report anyway"
        );
        assert_eq!(
            watch.took.runs, 1,
            "the run was dropped instead of kept for the next merge"
        );
        assert_eq!(
            watch.took.lost, 1,
            "a run a merge could not write was not counted as lost"
        );

        watch.ran(Duration::from_millis(4), b"second");
        watch.merge_out();
        assert_eq!(
            watch.took.lost, 2,
            "a second failed merge should count both runs now stuck, not add to the first count"
        );

        std::fs::remove_file(&in_the_way).expect("clear the way for the directory");
        watch.merge_out();

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
    fn a_slow_input_a_blocked_merge_could_not_write_is_retried_once_unblocked() {
        // Q64, following Q49 (§236.6): `merge_out` marked a slow input as
        // written the moment it *decided* to write it, not once the write had
        // actually landed. So a merge that failed before ever touching
        // `slowest/` — here, because the directory could not be made — still
        // burned the input's one shot at a retry, and its bytes were never
        // written even after the block cleared.
        let root = somewhere_to_write("blocked-slow-bytes");
        std::fs::create_dir_all(&root).expect("somewhere to write");
        let in_the_way = root.join("in-the-way");
        std::fs::write(&in_the_way, b"not a directory").expect("a blocking file");
        let into = in_the_way.join("timing");
        let mut watch = a_process_writing_into(&into, 4);

        let bytes = b"a slow input";
        watch.ran(Duration::from_millis(50), bytes);
        watch.merge_out();

        assert!(
            !into.join("slowest").exists(),
            "a merge that could not make its directory wrote to it anyway"
        );

        std::fs::remove_file(&in_the_way).expect("clear the way for the directory");
        watch.merge_out();

        let name = named_for(Duration::from_millis(50), bytes);
        assert!(
            into.join("slowest").join(&name).is_file(),
            "the slow input's bytes were never written, even once the block cleared"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_single_slow_file_write_that_fails_is_retried_too() {
        // Q64: the directory and the lock are not the only writes that can
        // fail. One slow input's own `fs::write` can fail on its own, with
        // `slowest/` already there and every other write this merge makes
        // going through — here, because a directory sits where the file
        // needs to go. The same fix has to cover this path too, or a
        // `written` that grew on attempt rather than success would still
        // refuse this input its retry.
        let into = somewhere_to_write("blocked-one-file");
        std::fs::create_dir_all(into.join("slowest")).expect("somewhere to write");

        let bytes = b"blocked by a directory of its own name";
        let name = named_for(Duration::from_millis(40), bytes);
        std::fs::create_dir_all(into.join("slowest").join(&name))
            .expect("a directory where the slow input's file needs to go");

        let mut watch = a_process_writing_into(&into, 4);
        watch.ran(Duration::from_millis(40), bytes);
        watch.merge_out();

        assert!(
            into.join("slowest").join(&name).is_dir(),
            "the write should have failed, leaving the blocking directory in place"
        );

        std::fs::remove_dir_all(into.join("slowest").join(&name)).expect("clear the way");
        watch.merge_out();

        assert!(
            into.join("slowest").join(&name).is_file(),
            "a slow input whose one file write failed was never retried"
        );
        let _ = std::fs::remove_dir_all(&into);
    }

    #[test]
    fn a_write_cut_short_never_leaves_a_short_file_under_the_real_name() {
        // Q67, §236.6: `fs::write` creates the file and then writes, so a
        // write cut short by a full disk or a process killed mid-write used
        // to leave the truncated bytes sitting under the real name — and
        // `cargo fuzz run` on that file would replay a different input from
        // the one that was actually slow.
        let into = somewhere_to_write("cut-short");
        std::fs::create_dir_all(into.join("slowest")).expect("somewhere to write");

        let bytes = b"a slow input long enough that four bytes is visibly short";
        let name = named_for(Duration::from_millis(20), bytes);

        FAULT_INJECTED_SHORT_WRITE.with(|budget| budget.set(Some(4)));
        let wrote = write_slow_input(&into.join("slowest"), Duration::from_millis(20), bytes);

        assert!(!wrote, "a write cut short should be reported as failed");
        assert!(
            !into.join("slowest").join(&name).exists(),
            "a write cut short left a short file under the input's real name"
        );
        let _ = std::fs::remove_dir_all(&into);
    }

    /// Set to the directory a copy of this test binary merges into until it
    /// is killed, by [`a_process_killed_while_it_merges_never_sets_the_totals_back`].
    const MERGE_UNTIL_KILLED: &str = "RCAD_FUZZ_TIMING_MERGE_UNTIL_KILLED";

    /// How many copies that test kills, each at a different moment.
    const KILLS: u64 = 200;

    #[test]
    fn a_process_killed_while_it_merges_never_sets_the_totals_back() {
        // Every process for a target adds its runs to the one `totals`, and
        // any of them can end in the middle of a merge: a `-timeout`, a crash
        // or an out-of-memory is `_Exit`, and an out-of-memory can come from
        // another thread. `std::fs::write` empties the file when it opens it
        // and writes after, so a process that ended between the two left
        // `totals` empty, `read_back` read that as no runs at all, and the
        // next merge started the whole target's count again from its own.
        if let Some(into) = std::env::var_os(MERGE_UNTIL_KILLED) {
            merge_until_killed(Path::new(&into));
            return;
        }
        let into = somewhere_to_write("killed-mid-merge");
        std::fs::create_dir_all(&into).expect("somewhere to write");
        let this_binary = std::env::current_exe().expect("this test's own binary");
        let mut counted = 0;
        for kill in 0..KILLS {
            let mut child = std::process::Command::new(&this_binary)
                .args([
                    "--exact",
                    "tests::a_process_killed_while_it_merges_never_sets_the_totals_back",
                ])
                .env(MERGE_UNTIL_KILLED, &into)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .expect("a copy of this test binary");
            let before = runs_once_merged_past(&into, counted, &mut child);
            std::thread::sleep(Duration::from_micros(kill * 37 % 500));
            let was_running = child.try_wait().expect("the copy's state").is_none();
            child.kill().expect("the copy killed");
            child.wait().expect("the killed copy reaped");
            assert!(was_running, "copy {kill} stopped before it was killed");

            let after = runs_in_totals(&into);
            assert!(
                after >= before,
                "copy {kill} was killed with `totals` at {before} runs, and it read {after} after"
            );
            counted = after;
        }
        let _ = std::fs::remove_dir_all(&into);
    }

    /// Merge one run at a time into `into` until killed, or for ten seconds,
    /// so a copy whose test died before killing it does not run on.
    fn merge_until_killed(into: &Path) {
        let mut watch = a_process_writing_into(into, 0);
        let stop = Instant::now() + Duration::from_secs(10);
        while Instant::now() < stop {
            watch.ran(Duration::from_micros(1), b"");
            watch.merge_out();
        }
    }

    /// The runs in `totals` once `child`'s merges have taken it past `counted`.
    fn runs_once_merged_past(into: &Path, counted: u64, child: &mut std::process::Child) -> u64 {
        let give_up = Instant::now() + Duration::from_secs(10);
        loop {
            let runs = runs_in_totals(into);
            if runs > counted {
                return runs;
            }
            let exited = child.try_wait().expect("the copy's state").is_some();
            if exited || Instant::now() > give_up {
                let _ = child.kill();
                let _ = child.wait();
                panic!("a copy never merged past {counted} runs; it exited: {exited}");
            }
            std::thread::sleep(Duration::from_micros(100));
        }
    }

    /// The runs in `totals`, read under the lock every merge takes.
    fn runs_in_totals(into: &Path) -> u64 {
        let lock = the_shared_report_locked(into).expect("the lock every merge takes");
        let runs = HowLongRunsTook::read_back(&into.join("totals")).runs;
        lock.unlock().expect("the lock released");
        runs
    }

    #[test]
    fn a_totals_file_cut_short_drops_the_torn_line_rather_than_lying() {
        // Q67 asked whether the short-write problem reaches `totals`. A file
        // cut short cannot land under the name any more (see
        // `a_process_killed_while_it_merges_never_sets_the_totals_back`), but
        // `read_back` still treats a line it cannot parse as absent rather
        // than guessing at it, so a torn tail loses what got cut and nothing
        // more — it never reads back as *more* than was written.
        let into = somewhere_to_write("totals-cut-short");
        std::fs::create_dir_all(&into).expect("somewhere to write");
        let mut took = HowLongRunsTook::new();
        for run in [1u64, 500, 12_000] {
            took.record(Duration::from_micros(run));
        }
        let whole = took.written_out();
        let cut = &whole[..whole.len().saturating_sub(5)];
        std::fs::write(into.join("totals"), cut).expect("a torn totals file");

        let back = HowLongRunsTook::read_back(&into.join("totals"));
        assert_eq!(
            back.runs, took.runs,
            "the run count sits on an earlier line; a torn tail should not touch it"
        );
        assert!(
            back.buckets.iter().sum::<u64>() <= took.buckets.iter().sum(),
            "a torn bucket line must be dropped, never counted as more runs than were written"
        );
        let _ = std::fs::remove_dir_all(&into);
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
        let mut first = a_process_writing_into(&into, 2);
        let mut second = a_process_writing_into(&into, 2);

        first.ran(Duration::from_millis(85), b"second slowest of all");
        second.ran(Duration::from_millis(90), b"slowest of all");
        second.ran(Duration::from_millis(10), b"quick");
        first.merge_out();
        second.merge_out();

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
        let mut first = a_process_writing_into(&into, 8);
        let mut second = a_process_writing_into(&into, 8);

        first.ran(Duration::from_millis(30), b"same");
        second.ran(Duration::from_millis(70), b"same");
        first.merge_out();
        second.merge_out();

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
        let mut watch = a_process_writing_into(&into, 2);
        for run in [1u64, 40, 90] {
            watch.ran(Duration::from_millis(run), &[run as u8]);
        }
        watch.merge_out();

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

        let mut watch = a_process_writing_into(&into, 1);
        watch.ran(Duration::from_millis(9), b"new");
        watch.merge_out();

        assert!(
            slowest.join("something-else").exists(),
            "a file this module did not write was removed"
        );
        let _ = std::fs::remove_dir_all(&into);
    }
}
