## 133. The fork-mode timings were not a path problem, measured 2026-09-18

*§129's item 1 is `-fork=N -ignore_crashes=1`, so that one crash stops ending a
target's night. Its blocker was written down as "the report path has no pid in
it". Measured, fork mode is a churn of about forty thousand short-lived
processes a night, the old report kept `1 runs` out of 54 845, and a pid in the
path would have written forty thousand directories. It is one shared
accumulation under a lock instead.*

### 133.1 What `-fork` actually does

`cargo fuzz run` with `-fork=N` does not run N long-lived children. libFuzzer's
parent orchestrates and runs nothing itself; each job is a fresh process spawned
through `/bin/sh -c`, given a slice of the corpus, and exited.

Measured on `where_two_surfaces_cross`, `-fork=2`, 101 seconds, sampling the
process tree twice a second: **12 jobs**, 35 distinct pids below the root, and
the pids arriving in pairs because each job is a shell and a binary. That is
about seventeen seconds a job. Over an eight-hour window at `-fork=24` it is
roughly **forty thousand processes for one target**, and there are five targets.

The sampling misses anything that lived under half a second, so 35 is a floor
and the conclusion only gets stronger.

### 133.2 What the old report did under it

Every one of those processes started a fresh empty `Watch`, wrote
`RCAD_FUZZ_TIMINGS/<target>/report` over the last one's, and ran
`clear_stale_slow_runs`, which deleted every `slow-*` file in the directory
whether or not this process had written it.

The run above executed 54 845 inputs across its 12 jobs. The report it left:

```
1 runs, 0.0 s in all
```

One slow unit kept of sixty-four. This is not a degraded measurement, it is the
absence of one, and it is what the surviving child happened to hold when it
died. §129.1's table — the thing that made the `where_two_surfaces_cross`
finding visible at all — could not have been written from a fork night.

### 133.3 Why a directory per process is not the fix

The obvious repair is `RCAD_FUZZ_TIMINGS/<target>/<pid>/`. It is correct and it
is useless: forty thousand directories per target per night, each holding a few
thousand runs of one job, and the percentile table split across all of them. The
report exists to be read in the morning by a person, and forty thousand reports
is not a report.

**The reason this was not obvious from the source is that the source does not
say how many processes there are.** The handoff that raised the blocker read
`lib.rs` correctly and inferred "N children" from the flag's name. Reading the
code answered what happens per process; only running it answered how many there
are. Same shape as §77.6 — a claim that is true of what was examined and silent
about what was not.

### 133.4 One accumulation, a lock, and a thread on a second

`RCAD_FUZZ_TIMINGS/<target>/totals` holds the histogram every process has merged
into, guarded by an advisory lock on a `lock` file beside it
(`std::fs::File::lock`, stable on the pinned 1.96.0, so no dependency and
`#![forbid(unsafe_code)]` is untouched). A process merges a **delta** and resets
its own counters, so merging twice does not count twice.

The run path no longer writes anything. A background thread ticks once a second
and does the merging, which matters because of §133.1: at the old thirty-second
interval most children would exit having merged nothing at all, and the loss
would be the night rather than its edge. libFuzzer kills its children from
outside, so there is no exit to hook — a process loses at most its last second,
and that is the honest cost. Percentiles are unaffected by it; the run total is
a slight undercount.

A merge is a lock, a read, an add and two small writes against a `totals` file
measured at 2 477 bytes, so twenty-four processes ticking once a second is a
rounding error beside the fuzzing.

### 133.5 The slowest runs need no accumulation at all

`named_for` already writes the time and a hash of the input into the file name.
So the directory listing sorted by time **is** the top across every process, and
pruning is a sort — no shared state, no lock contention beyond the one already
held, and the same one-input-once rule `TheSlowestRuns::offer` applies within a
process now applies across them by keeping only the slowest name per hash.

### 133.6 A mutant that passed, and the test that was wrong

Four mutants were run against the new tests and each fails its own:

| mutant | what fires |
| --- | --- |
| the accumulation dropped | `1 runs, 0.0 s in all` — the live defect's exact string |
| the old delete-everything prune restored | the other process's slow unit is gone |
| the dedupe dropped | one input kept under two names |
| the delta not reset | two runs counted as four |

The second one **passed** the first time it was run, and the test was at fault
rather than the code. Both of that test's slow runs came from the same process,
so a prune that deleted everything it had not itself found still left exactly
the pair being asserted. It now requires the top two to span the two processes —
the slowest found by one and the second slowest by the other — and the reason is
written in the test, because the next person will write the first version.

This is the §15.6 rule in a new place: a test that passes against the defect it
names is prose. Run the mutant.

### 133.7 Exit 70 is not a crash, and the recipe half has to know

`FuzzerFork.cpp:488` — *"Use the exit code from the last child process"* —
`exit(ExitCode)`, where `ExitCode` is whatever the last collected job returned.
With `-ignore_timeouts=1` the loop carries on but does not reset it, and 70 is
libFuzzer's default `-timeout_exitcode`.

Two runs of the same command, same corpus, differing only in whether the final
job happened to time out, exited **70** and **0**. `fuzz-overnight` maps any
non-zero to `crash`, so as it stands fork mode would report crashes that did not
happen — and a night nobody believes is the failure §91 exists to prevent.

Confirmed in the same runs, and it is item 2's premise: with
`-ignore_crashes=1 -ignore_timeouts=1` the target walked past the six landed
`slow-unit-*` seeds of §91.3, kept fuzzing, and wrote every timeout into
`artifact_prefix` — `oom/timeout/crash: 0/1/0` with the job counter still
climbing.

The recipe half of item 1 is not written yet and lands after the night of
2026-09-18 has proved §129's concurrency change on its own.

Commands run to reach this: a `-fork=2` run of the built
`where_two_surfaces_cross` against a copy of its corpus, twice — once against
the old timing crate and once against the new — with the process tree sampled
twice a second; `Read` of `crates/rcad-fuzz-timing/src/lib.rs`; `grep` of
`libfuzzer-sys-0.4.13/libfuzzer/FuzzerFork.cpp`; a `rustc` probe for
`std::fs::File::lock` on the pinned toolchain; `mise exec -- just t -p
rcad-fuzz-timing` against the tree and against four mutants;
`mise exec -- just fmt`, `lint`, `test`.

