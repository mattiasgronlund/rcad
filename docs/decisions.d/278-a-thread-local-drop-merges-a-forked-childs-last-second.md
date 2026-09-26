## 278. A thread-local `Drop` merges a forked child's last second, and it runs inside a real libFuzzer child — 2026-09-26

*Q164. The row: "A forked child that ends normally, at -max_total_time or on
the stop file, loses up to a second of timings, because exit(0) never reaches
a merge; §267.7 showed a thread_local whose Drop runs at exit(0) under
forbid(unsafe_code). Merge from that Drop and show it runs inside a real
libFuzzer child. At most 0.46% of the reader's runs on 2026-09-26." §267.6
§267.7.*

  278.1   The fix and its test, on `fac68b59`
  278.2   The test, before and after
  278.3   The exit(0) merge, run inside a real libFuzzer child
  278.4   Which thread ends a child, and whether it is the one holding the thread-local
  278.5   `-fork=2`: the same mechanism, and where a short sample stops being useful evidence
  278.6   Verdict, and what is left

### 278.1 The fix and its test, on `fac68b59`

`MergeWhenThisThreadEnds` (`crates/rcad-fuzz-timing/src/lib.rs`) is a
zero-sized `thread_local!` whose `Drop` merges from `WATCH`. It is touched
once, inside the same `OnceLock::get_or_init` that makes `WATCH`
(`start_the_clock`), so it costs nothing on every other draw and nothing at
all where `RCAD_FUZZ_TIMINGS` is unset. `#![forbid(unsafe_code)]` holds: no C
call is made to register it, and nothing here is `unsafe`. This closes the gap
267.7 named: a child that ends normally never returns from the draw it is in,
so a run counted since the last due merge used to stay stuck in that
process's memory until the process was gone.

The commit is `fac68b59`, built by the previous run of this row
(`lane-q164-drop-merges`) and fast-forwarded onto this lane's branch first
thing.

### 278.2 The test, before and after

`a_child_that_exits_before_a_merge_is_due_still_leaves_its_run_in_totals`
(`crates/rcad-fuzz-timing/tests/a_child_that_exits_normally_still_merges_its_last_second.rs`)
re-execs its own test binary as a child with `RCAD_FUZZ_TIMINGS` set, has the
child do exactly one draw through `timed()` and call `std::process::exit(0)`
itself well inside the one-second merge interval, then checks the parent
finds that run in `totals`.

Command: `mise exec -- just t a_child_that_exits_before_a_merge_is_due_still_leaves_its_run_in_totals`

- On `8b0748af7338cca915f030eadb00e2f7cb7a2482` (`crates/rcad-fuzz-timing/src/lib.rs`
  reverted to that commit, everything else unchanged): **FAILED** —
  `the child's one draw never reached totals, though it ended with exit(0): ""`.
- On `fac68b59` (this lane's base, `crates/rcad-fuzz-timing/src/lib.rs` restored):
  **1 passed**.

### 278.3 The exit(0) merge, run inside a real libFuzzer child

The test above proves the mechanism with a re-exec of the test binary, not
with `cargo fuzz run`. This section runs the real thing: `two_solids_fused_by_a_stranger`,
the smallest target and the only one that reads no `RCAD_STEP_CORPUS`.

Command (`corpus_root` under the scratchpad, so libFuzzer's second, read-only
argument is the tracked `fuzz/corpus/two_solids_fused_by_a_stranger` and
nothing is written there — the recipe's own design, §81):

```
RCAD_FUZZ_TIMINGS=<scratchpad>/t mise exec -- just fuzz-target \
    two_solids_fused_by_a_stranger 8 <scratchpad>/c "" 0 ""
```

`-max_total_time=8` is short enough that the run ends while still walking the
1751-file seed corpus, so libFuzzer's own count and the crate's `totals` can
be read off the same short run.

| lib.rs | libFuzzer's own last line | `totals` after | gap |
|---|---|---|---|
| `fac68b59` (fixed) | `#1752 DONE ...` / `Done 1752 runs in 17 second(s)` | `runs 1752` | 0 |
| `8b0748af` (pre-fix) | `#1752 DONE ...` / `Done 1752 runs in 9 second(s)` | `runs 1709` | 43 runs, 2.45% |

Both runs end the same way: libFuzzer reaches `-max_total_time`, calls
`FuzzerDriver`'s `exit(0)` (`FuzzerDriver.cpp:930`, following `F->Loop`
directly at `:923` — no thread is spawned for this), and the process is
`cargo fuzz run`'s own child, not a subprocess of the test binary. With the
fix, `totals` matches libFuzzer's own count exactly. Without it, 43 of the
1752 runs — the ones the process had done since its last periodic,
once-a-second merge — never reached `totals`, which is the loss 267.6 and
267.7 described and bounded.

The old `lib.rs` was restored with `git checkout HEAD -- crates/rcad-fuzz-timing/src/lib.rs`
between the two runs; `git status --short` shows the file clean at the end of
this section's work.

### 278.4 Which thread ends a child, and whether it is the one holding the thread-local

`FuzzerDriver()` (`FuzzerDriver.cpp`) calls `F->Loop(CorporaFiles)` at line
923 and `exit(0)` at line 930, both directly, on whatever thread called
`FuzzerDriver` — no thread is created for either. `Loop` is where libFuzzer
calls `LLVMFuzzerTestOneInput` for every draw, which is where the fuzz
target's own code calls `rcad_fuzz_timing::timed`, which is where
`start_the_clock` touches `MERGE_WHEN_THIS_THREAD_ENDS`
(`crates/rcad-fuzz-timing/src/lib.rs`). So the thread that calls `exit(0)` at
`-max_total_time` is the same thread that ran every draw and the same thread
that registered the thread-local: **the process's one and only thread**,
libFuzzer never spawns a second one for the fuzzing loop itself. That is why
a `Drop` on that thread's thread-local reaches the same `WATCH` the draws
were writing to, with no synchronization beyond the `Mutex` `WATCH` already
uses.

A `-timeout`, a crash or an out-of-memory still end in `_Exit`
(`FuzzerLoop.cpp:307`, `:238`, `:135` and `:322`, as 267.7 found), which runs
no destructors on any thread. The fix saves nothing on that path, and 278.3's
table only exercises `-max_total_time`, the path the row names. Nothing in
this lane touched or re-measured the `_Exit` paths; they remain a limit, not
a gap.

### 278.5 `-fork=2`: the same mechanism, and where a short sample stops being useful evidence

Under `-fork`, libFuzzer's parent runs `ExecuteCommand`, which on Linux is
`system(CmdLine.c_str())` (`FuzzerUtilLinux.cpp:25,27`) — a real child
process, exec'd fresh, not a thread. Each such worker is given its own,
ramping `-max_total_time`: `Cmd.addFlag("max_total_time",
std::to_string(std::min((size_t)300, JobId)))` (`FuzzerFork.cpp:130`). So a
`-fork` worker that reaches its own `-max_total_time` runs the identical
`FuzzerDriver` → `Loop` → `exit(0)` path as 278.3 and 278.4, on its own single
thread, in its own process. Nothing about being forked changes which thread
runs `exit()`'s destructors.

Run, same target, `-fork=2`, `-max_total_time=8`, `TMPDIR` scoped to the
scratchpad:

```
TMPDIR=<scratchpad>/tmpdir RCAD_FUZZ_TIMINGS=<scratchpad>/t mise exec -- just \
    fuzz-target two_solids_fused_by_a_stranger 8 <scratchpad>/c "" 2 ""
```

Fixed `lib.rs`: one job (`job: 1`) finished and had its stats merged by the
`-fork` orchestrator before the 8 s budget ran out, printed as `#502: cov:
... job: 1` — that `502` is `Stats.number_of_executed_units`, read back from
that one job's own log (`FuzzerFork.cpp:201`, printed at `:223`) — while the
crate's own `totals` for the same run read `runs 3479`. Pre-fix `lib.rs`, a
second, separate `-fork=2` run of the same target and duration: the
orchestrator's own line read `#4779: ... job: 1`, `totals` read `runs 4950`.

Neither pair is usable as a before/after measurement of the loss, and saying
so is the honest reading of it, not a gap to paper over:

- The orchestrator's own `#N` is **per completed-and-merged job only**
  (`NumRuns += Stats.number_of_executed_units` at the moment one job's log is
  parsed, `FuzzerFork.cpp:201`). A second worker still running, or one whose
  log was never parsed before the whole `-fork` session's global clock ran
  out, never adds to it — so `totals`, which every worker merges into
  directly, reading *higher* than the orchestrator's own `#N` (3479 vs 502,
  4950 vs 4779) is expected and is not evidence of anything gained or lost.
- The two runs' own job counts (502 vs 4779) differ by nearly 10x on the same
  target and duration, which is job-scheduling and machine-load noise (this
  machine was also running Q148's nightly fuzz session throughout), not
  signal — an 8-second sample dominated by how far one job's own corpus load
  got is too small and too noisy to isolate a loss the row itself bounds at
  under half a percent.

278.3's `-fork=0` pair is the clean measurement: there, the *entire* run is
one job, libFuzzer's own final count is the whole run's ground truth, and the
loss (43 runs of 1752) is the full effect of one `-max_total_time` ending,
undiluted by a second worker's own merges landing in the same `totals`. Since
278.4 shows a `-fork` worker takes the identical code path on its own thread,
that measurement carries over: a `-fork` night loses the same up-to-a-second
per worker ending, `-fork` just runs many of them.

### 278.6 Verdict, and what is left

**Built:** `MergeWhenThisThreadEnds`, merging from a thread-local `Drop`, no
`unsafe`, costing nothing where timings are off (278.1).

**Shown, with a test that fails without the change and passes with it
(278.2):** the mechanism, isolated.

**Shown, with the real fuzz target and real `cargo fuzz run` (278.3):** the
same mechanism, inside a genuine libFuzzer child, at `-max_total_time`,
recovering the full 43 runs (2.45% of that short run) the pre-fix `lib.rs`
lost.

**Shown, from source (278.4):** the thread that calls `exit(0)` at
`-max_total_time` is the one and only thread the fuzzing loop and every draw
run on, so it is necessarily the thread holding `MERGE_WHEN_THIS_THREAD_ENDS`.

**Shown, from source, and demonstrated to be the identical path (278.5):** a
`-fork` worker is a real, separate, exec'd process with its own ramping
`-max_total_time`, ending the same way. **Not shown:** a numeric before/after
of the loss under `-fork` itself — two 8-second samples were both too small
and too noisy (job-count variance of nearly 10x between two runs of the same
target and duration, on a machine also running a nightly fuzz session) to
read a sub-1% effect off directly, and that noise, not a flaw in the fix, is
why. 267.6's 0.46% bound was read off a full night's `totals` against
libFuzzer's own count for a single-job target; reproducing that read on
`-fork` would need a run long enough for many jobs to complete, which was out
of this lane's budget.

**Left, low value, per 267.6:** the `_Exit` paths (`-timeout`, crash, OOM)
still lose their runs, unchanged and unmeasured by this lane; measuring
`-fork`'s own per-worker loss on a run long enough to average out job-count
noise, if the 0.46% bound is ever worth tightening.
