## 296. A full hour confirms C's ~1.5× cost but shows no further shift toward it — 2026-09-27

*Runs queue row Q175's second attempt, with §268.5's changes: a binary built
in the lane's own scratchpad, both arms sampled every 300 s, and how `-fork`
sizes a job read before reading a trend. The first attempt built the binary and
started both arms, then was cut off eleven minutes in and had to report with
the arms still running; this run picks up the same two arms and lets them run
to their own end. The run fell between 15:31:52Z and 16:32:13Z, outside
21:50–06:00Z, but not free of other load: the 1-minute load average ranged
from about 6 to about 39 over the hour, from other sessions sharing the
machine. This lane only read and measured. Nothing in the kernel, the nightly
tree or the corpus was written.*

  296.1   What ran, and how it differs from §268
  296.2   How `-fork` sizes a job
  296.3   The table, this time for the whole hour
  296.4   Whether the mutant rate shifts toward C
  296.5   What this still doesn't show

### 296.1 What ran, and how it differs from §268

**The binary is built fresh in this lane's own scratchpad**, not borrowed from
another lane's `/tmp` tree the way §268's was (and lost, mid-run, §268.2).
Same commit, `055aa4a9`, same target, `two_solids_fused_by_a_stranger`.

**The two corpora are §254's and §268's**, copied (`cp -al`, hard-linked, not
moved) into this lane's own `run_withC` and `run_withoutC` before each run:
11 077 raw files with C, 10 154 without — the same counts §254.1 and §268.1
give, confirmed by listing this lane's own `withC/` and `withoutC/` copies.

**Both arms ran at once**, on disjoint CPU sets, for a fixed hour each:

```
taskset -c 6-9  env RCAD_FUZZ_TIMINGS=<withC dir>    <binary> -fork=4 -max_total_time=3600 \
  -timeout=10 -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1 -print_final_stats=1 <copy>
taskset -c 10-13 env RCAD_FUZZ_TIMINGS=<withoutC dir> <binary> -fork=4 -max_total_time=3600 \
  -timeout=10 -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1 -print_final_stats=1 <copy>
```

**Both arms ended on their own, cleanly.** With C printed `INFO: fuzzed for
3600 seconds, wrapping up soon` / `exiting: 0 time: 3601s`; without C printed
the same at `3618s` / `3619s`. No crash, timeout or OOM in either log's
`oom/timeout/crash` column, which stayed at `0/0/0` throughout both logs.
Unlike §268, the binary was never removed from under the run.

**The sampling has a nine-minute gap, filled from the logs.** One sample was
taken at 15:43:01Z by the first attempt's sampler (`samples_first_lane.txt`),
which then died with that attempt at about 15:45Z. The dispatcher restarted
the same script (`q175_monitor.sh`) at 15:53:33Z; it took eight samples, every
300 s, ending at 16:33:33Z (`samples_resumed.txt`). **No sample exists near
15:48Z.** §296.3's table does not use the sampler at all for that reason —
every row comes from the fuzz logs' own `#N: … time: Ns` lines, which are
continuous, and the sampler's numbers are used here only as a cross-check
(they agree with the logs' final totals to the run, see below).

### 296.2 How `-fork` sizes a job

Read from `FuzzerFork.cpp` in `libfuzzer-sys` 0.4.13, checked before use as
the brief asked:

- **The parent merges the corpus down first.** Before any job runs,
  `CrashResistantMerge` reduces the raw seed files to the ones that carry
  distinct coverage. This run's two `INFO: -fork=4: N seed inputs` lines give
  3 521 with C and 3 061 without — close to but not identical to §268's 3 518
  and 3 062, which is expected: it is the same merge algorithm run again on a
  fresh copy, not the same output guaranteed byte-for-byte.
- **Each job gets `min(files, sqrt(files + 2))` seeds**, drawn one at a time by
  `Rand->SkewTowardsLast(Files.size())` — a skew toward the *end* of the
  (size-sorted, then merge-appended) file list, not a uniform sample. For 3 521
  files that is 59 seeds a job; the merged corpus grows slightly as jobs find
  new coverage, so the seed count per job grows too, slowly.
- **Job `J` runs for `min(300, J)` seconds** (`Cmd.addFlag("max_total_time",
  min(300, JobId))`). Job 1 runs 1 s, job 50 runs 50 s, job 300 and every job
  after it runs the full 300 s. **This run never reached that cap**: with C
  finished 163 jobs, without C finished 164 (`grep -c '^#[0-9]+:'` on each
  log), so job lengths were still climbing linearly through the whole hour,
  not flat at 300 s as they would be past job 300.
- **The four workers pull jobs off one queue, and the corpus merge is
  serial** (one thread pops `MergeQ`, one job's stats and corpus additions at
  a time), so `JobId` climbs roughly in step with wall time on either arm
  regardless of which of the four child processes actually ran it.

### 296.3 The table, this time for the whole hour

Cumulative runs from each log's own `#N: … time: Ns` lines, nearest at or
before each mark (`grep -E '^#[0-9]+:' log_*.txt`, then the nearest `time:`
to the target):

| t | runs with C | runs without C | window ratio (without ÷ with) |
|--:|------------:|---------------:|---------------:|
| 300 s | 253 821 | 397 569 | 1.57 |
| 600 s | 552 422 | 856 729 | 1.54 |
| 900 s | 931 278 | 1 445 190 | 1.55 |
| 1 200 s | 1 214 695 | 1 888 553 | 1.56 |
| 1 500 s | 1 535 524 | 2 336 562 | 1.40 |
| 1 800 s | 1 795 931 | 2 836 080 | 1.92 |
| 2 100 s | 1 976 737 | 3 197 201 | 2.00 |
| 2 400 s | 2 203 989 | 3 536 291 | 1.49 |
| 2 700 s | 2 558 289 | 3 988 285 | 1.28 |
| 3 000 s | 2 969 283 | 4 564 064 | 1.40 |
| 3 300 s | 3 341 453 | 5 138 793 | 1.54 |
| 3 600 s | 3 763 842 | 5 611 169 | 1.12 |
| final (3 601 s / 3 619 s) | 3 843 228 | 5 739 897 | — |

"Window ratio" is this row's runs minus the row above's, divided the same way
(so it is the throughput ratio for that one 300 s slice, not the cumulative
one). The final row is each arm's own `rcad-fuzz-timing` `totals` file
(`grep '^runs '`), which counts every completed run including the last job or
two still merging when the process exited — it agrees with the resumed
sampler's last sample (i=8, 16:33:33Z) to the run: 3 843 228 and 5 739 897
both places.

**Total time spent inside the target, the whole hour**: `nanos ÷ 1e9` from
each `totals` file — 11 001.7 s with C, 10 860.0 s without, a ratio of 1.013.
As in §268.3's five minutes, the two arms get almost exactly the same CPU
over the whole hour; the runs gap is entirely a per-run cost gap, not a
scheduling one.

**Mean time per run, the whole hour**: `nanos ÷ runs` — 2.862 ms with C,
1.892 ms without, a ratio of **1.51**. Against §268.3's five-minute figure of
**1.67**, this went down, not up.

**p90 and p99, the whole hour**, read off each arm's own bucket histogram
(`grep '^b ' totals`, `rcad-fuzz-timing`'s own bucketing: 16 buckets per
octave of nanoseconds, `bucket_of`/`nanos_at` in `crates/rcad-fuzz-timing/src/lib.rs`):

| | with C | without C | ratio |
|--|-------:|----------:|------:|
| p90 | 3.932 ms | 0.508 ms | 7.74 |
| p99 | 46.14 ms | 35.65 ms | 1.29 |

Against §268.3's five-minute p90 of 9.437 ms / 1.180 ms (ratio 8.00) and p99
of 52.429 ms / 39.846 ms (ratio 1.32), both ratios are very slightly *lower*
over the full hour than they were in the first five minutes.

### 296.4 Whether the mutant rate shifts toward C

**No. Every metric here is flat or slightly softer over the hour than §268's
five-minute snapshot, not shifted further toward C:**

- the aggregate runs ratio at the end of the hour, 5 739 897 ÷ 3 843 228 =
  1.49, is below §268.3's five-minute 1.70–1.72;
- the mean-time-per-run ratio, 1.51, is below §268.3's 1.67;
- p90's ratio, 7.74, is below §268.3's 8.00;
- p99's ratio, 1.29, is below §268.3's 1.32.

**The twelve 300 s windows do not climb toward the end, either.** They run
1.57, 1.54, 1.55, 1.56, 1.40, 1.92, 2.00, 1.49, 1.28, 1.40, 1.54, 1.12, in
order from the run's first five minutes to its last. The two highest windows
sit back to back in the middle third, 1.92 for 1 500 s–1 800 s and 2.00 for
1 800 s–2 100 s, and the single lowest window of the whole hour, 1.12, is the
very last one, 3 300 s–3 600 s.
If §254.5's hypothesis held at the scale these two arms measure it — that
entropic spends a growing share of mutation effort on C's rare features as a
run goes on — the window ratios should trend up toward the end. They do not:
a line fit by eye through those twelve points is flat to falling, not rising.

**This is likely mostly load, not the scheduler.** The samples in
`samples_resumed.txt` carry the machine's own 1-minute load average alongside
each pair of run counts, and it swings from 6.05 to 39.24 across the hour,
uncorrelated with elapsed time (it peaks at i=1–2, around 15:58–16:03Z, then
falls, then partly recovers) — other sessions' work, not this run's. A
window's ratio moving with that kind of shared-machine noise, on one pair of
arms, with unpinned seeds, is not enough to separate a real scheduler effect
from noise, and the swings here (1.12 to 2.00) are wide enough to explain the
whole spread without invoking one.

**§254.5's hypothesis is not supported by an hour of data at `-fork=4`.**
It predicted the cost would grow with run length; measured over 5 minutes and
then over the full hour on the same pair of corpora, it did not grow — if
anything it eased slightly on every metric checked.

### 296.5 What this still doesn't show

- **This is still one pair of runs**, unpinned seeds, sharing a machine with
  other sessions. A second hour, on quiet cores, would tell whether the
  slight softening seen here is real or itself noise.
- **The comparison is still arm-level**, with-C-corpus against without-C-corpus,
  not seed-level. It says nothing about how much of entropic's attention any
  individual seed drawn from C gets inside the with-C arm — only that the two
  arms' aggregate costs did not diverge further as the hour went on.
- **The nights still run far longer than an hour** (§239's 09-23 night was
  measured at 4.6 M runs against 09-22's 36.7 M, an 8× gap this hour's ~1.5×
  does not approach). Whatever produces that 8× gap on a real night, an hour
  at `-fork=4` on two static corpora does not reproduce it, and this lane does
  not know what does.
