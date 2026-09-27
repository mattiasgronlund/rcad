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
  296.6   Seed by seed: C gets more total effort, but not more per seed

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

### 296.6 Seed by seed: C gets more total effort, but not more per seed

*Runs queue row Q244, the seed-level question §296.5 left open: not whether
the two arms' totals diverge, but whether, inside the with-C arm alone,
entropic gives set C's inputs more mutations than their share of the corpus
would predict. Only the with-C arm is run; the question is internal to it.
This lane only read and measured; nothing in the kernel, the nightly tree or
the corpus was written. Every run was pinned to cores 20-23 with `taskset`,
started between 18:31Z and 18:38Z, well outside 21:50–06:00Z.*

**Set C is confirmed the way the brief asked, not assumed.** libFuzzer names
each corpus file after the SHA-1 hash of its contents, so two copies of the
same input have the same name in both directories. Listing this lane's own
`withC/` (11 077 names) and `withoutC/` (10 154 names) and taking the names in
the first but not the second gives **923 names** — the same count §254.1 and
§268.1 give for C, and, since these are hash names, exact proof that this is
the same 923 files, not merely the same count.

**The binary and the corpus §296 used are gone.** §296's own scratchpad
(`/tmp/.../scratchpad/q175/`) still holds its `withC`/`withoutC` corpora and
its logs, but the fuzz binary under `build-055aa4a9/fuzz/target/` had already
been removed by the time this lane read it — nothing under that build tree's
`fuzz/` has a `target/` directory at all, and no file named
`two_solids_fused_by_a_stranger` exists anywhere under it. This lane copied
the two corpora (read-only, `cp -al`) into its own `target/q244/`, confirmed
the counts above against them, and rebuilt the binary itself: `git archive
055aa4a9` extracted into `target/q244/src-055aa4a9/`, then `cargo +nightly
fuzz build two_solids_fused_by_a_stranger` there, with `RUSTFLAGS="-C
target-cpu=x86-64-v3"` set explicitly — the `fuzz-target` recipe's own comment
says this has to be repeated because cargo-fuzz's own `RUSTFLAGS` otherwise
shadows the `.cargo/config.toml` entry `inari` needs, and a first attempt
without it failed to compile `inari` with exactly that symptom (`cannot find
function sub1_ru`). The second attempt built cleanly in 57 s. Both runs below
report `cov: 17539` / `cov: 17540` and `ft: 90695` / `ft: 90687` at `INITED` —
against §254.5's `17539 – 17540` edges and `90685 – 90697` features for the
with-C corpus — so this is the same binary-and-corpus pair those sections
used, not a different build that happens to share a commit.

**This measurement drops `-fork=4` for a single process, and says why.**
Under `-fork`, each of the four children gets its own `Corpus` object seeded
with a small subset of files (§296.2: `min(files, sqrt(files+2))`, about 59 to
start) and its own entropic energy assignment over just that subset; the
parent only merges coverage back in between jobs. Asking "does entropic favor
C's inputs" is a question about *one* scheduler's energy assignment over the
*whole* corpus, and `-fork` never runs one. A single process does: one
`Corpus`, one entropic distribution, over every surviving input at once. The
cost is throughput (300 – 350 execs/s on one core here, against the
800 – 1 900/s the four-way split reached in §268/§296) and wall-clock reach —
this is not a like-for-like replacement for the arm-level throughput
comparison those sections ran, only for the seed-level question this one
asks.

**What ran.** Two independent runs, both from a fresh copy of the with-C
corpus, both pinned with `taskset -c 20-23` (cores 0-19 were other lanes' and
a concurrent `just gate`'s):

```
taskset -c 20-23 <binary> -print_final_stats=1 -print_corpus_stats=1 -timeout=10 \
  -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1 -max_total_time=300 <copy>
taskset -c 20-23 <binary> -print_final_stats=1 -print_corpus_stats=1 -timeout=10 \
  -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1 -max_total_time=900 <copy>
```

`-print_corpus_stats=1` is libFuzzer's own per-input dump
(`FuzzerCorpus.h:326`, `Corpus.PrintStats()`, called from
`PrintFinalStats` when this flag is set): one line per surviving input, its
SHA-1 name, size, and **`NumExecutedMutations`** — the count this section
reads as "effort". Both runs replayed the full 11 077-file seed corpus before
mutating (`#11078 INITED`), which is where the two corpora's edge and feature
counts above come from, and both ended cleanly at their full duration (`DONE`
at `#103084`/342 exec/s and `#318086`/353 exec/s — 301 s and 901 s).

**Not every raw file becomes a seed entropic can pick.** Loading discards any
file that adds no coverage beyond what an earlier one already gave; only the
survivors get a `NumExecutedMutations` line at all. That reduction, not
anything scheduled afterward, is most of C's over-representation:

| | 300 s run | 900 s run |
|--|---:|---:|
| C's share of the 11 077 raw files (923 names) | 8.33% | 8.33% |
| C's share of the retained corpus at exit (`grep -c '^\s*\['`) | 561/3500 = **16.03%** | 512/3567 = **14.35%** |
| C's share of every executed mutation (`sum(runs)` over C ÷ over all) | 14 343/91 696 = **15.64%** | 43 518/306 702 = **14.19%** |
| mean mutations per C input | 25.57 | 85.00 |
| mean mutations per non-C input | 26.32 | 86.15 |
| median, both groups | 25.0 | 85.0 |
| Welch's t (C mean − non-C mean, in units of its own standard error) | −1.46 | −1.21 |

(`target/q244/analyze.py`, run against `run1_withC.log` and `run2_withC.log`;
the retained-corpus and per-input figures come from parsing every
`[ idx sha1] sz: … runs: … succ: … focus: …` line and matching `sha1` against
the 923 names in `setC_names.txt`.)

**C's share of the raw files (923 of 11 077) roughly doubles by the time it
reaches the retained corpus** — 16.03% and 14.35% against 8.33% — which is
consistent with §254.5's own reading, that C carries coverage the rest of the
corpus does not, so a much larger fraction of C's files survive the
redundancy check that most of the corpus fails. **But C's share of executed
mutations tracks its share of the retained corpus, not its share of the raw
files**: 15.64% of effort against 16.03% retained (300 s), 14.19% against
14.35% (900 s) — both within half a percentage point, both *slightly below*
parity rather than above it. The mean mutations per input for C and for
everything else differ by less than 5% in either run (25.57 vs 26.32; 85.00
vs 86.15), and Welch's t stays under 1.5 in absolute value both times — far
short of the roughly 2 a real difference would need to clear at these sample
sizes (n = 561/2939 and 512/3055). **Once a C input is in the corpus at all,
entropic gives it the same ration as everything else next to it, not more.**

**Nor does the gap open up as the run goes on**, which is the part of
§254.5's hypothesis this section can speak to that §296.4 could not: the
300 s and 900 s runs (three times the length) give almost the same retained
share (16.03% → 14.35%) and the same effort-tracks-retained pattern, not a
widening one. The two runs used different random seeds (unpinned, as in every
run in this file) and are two separate one-shot snapshots, not the same run
sampled twice — libFuzzer's per-input counts only print at exit, so a single
continuous run cannot be read at intermediate checkpoints without changing the
binary. That is a real limit on how well "settled" can be shown here: two
points, three-to-one in length, agreeing with each other is evidence against a
growing effect, not proof there is none at any length.

**This does not support §254.5's hypothesis as stated.** The hypothesis was
that entropic spends a *growing* share of mutation effort on C's inputs
*because* they carry rare features — an active preference, compounding with
run length. What this section finds instead: C's inputs do end up with a
larger share of total effort than their file count would suggest, but that
gap is explained by which of C's files survive the initial coverage check,
not by entropic favoring the survivors once they are in the corpus, and the
size of that gap does not grow between a 5-minute and a 15-minute run. The
arm-level total in §254.5 (C costs 1.4 to 1.7 times, depending on `-fork`) is
still real and still unexplained by loading time alone (§254.4) — this
section says the seed-level mechanism proposed for it is not the one running.

### 296.7 A third point, nine times the first run's length, and the gap shrinks rather than grows

*Runs queue row Q268: a third with-C-only single-process run, at
`-max_total_time=2700` (45 minutes) — three times §296.6's longer run and
nine times its shorter one — to say whether §296.6's no-growth finding still
holds at a longer length. This lane only read and measured; nothing in the
kernel, the nightly tree or the corpus was written. The run was pinned to
cores 20-23 with `taskset`, started at 20:05:28Z and finished at 20:50:30Z,
outside 21:50–06:00Z.*

**Binary and corpus are the same pair §296.6 used, checked the same way.**
§296.6's own build and corpus copy were themselves temporary (its own
`target/q244/` scratch build); this lane repeated the same steps
independently: `git archive 055aa4a9` into `target/q268/src-055aa4a9/`, then
`cargo +nightly fuzz build two_solids_fused_by_a_stranger` there with
`RUSTFLAGS="-C target-cpu=x86-64-v3"` set explicitly (`target/q268/build.sh`),
which built in 55.4 s — close to §296.6's 57 s. The with-C corpus is Q175's
own, at Q175's session scratchpad, read-only to this lane: `cp -al`'d into
`target/q268/withC` (11 077 files, matching §296.6's count), and this run's
own `INITED` line reads `cov: 17539 ft: 90696` — inside §254.5's `17539–17540`
/ `90685–90697` range and inside §296.6's own two runs' figures — so this is
the same binary-and-corpus pair, not a coincidentally similar one. Set C was
recomputed independently too, not read from §296.6: sorting `withC`'s and
`withoutC`'s file names and taking the with-only names (`target/q268/setc.sh`)
gives **923 names**, the same count as §296.6.

**What ran**, one process, pinned:

```
taskset -c 20-23 /usr/bin/time -v two_solids_fused_by_a_stranger \
  -print_final_stats=1 -print_corpus_stats=1 -timeout=10 \
  -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1 \
  -max_total_time=2700 target/q268/withC
```

(the full command and environment are the first four lines of
`target/q268/run3_withC.log`). It ran its full 2700 s and exited 0: `DONE` at
`#930409` (344 exec/s), wall clock 45:01.66 per `/usr/bin/time -v`, `uptime`
read load average 10.25 (5 min) before the run at 20:05:23Z and 11.70 after it
at 20:50:43Z — this machine had other work on it throughout, as it did for
§296.6's runs.

**The comparison, alongside §296.6's two points** (`target/q268/analyze.py`,
run against `run3_withC.log` and this lane's own `setC_names.txt`; it parses
every `[ idx sha1] sz: … runs: … succ: … focus: …` corpus-stats line, matches
`sha1` against the 923 set-C names, and sums the `runs` field — the same
`NumExecutedMutations` reading §296.6 used):

| | 300 s (§296.6) | 900 s (§296.6) | 2700 s (this run) |
|--|---:|---:|---:|
| C's share of the 11 077 raw files | 8.33% | 8.33% | 8.33% |
| C's share of the retained corpus | 16.03% | 14.35% | **13.06%** |
| C's share of executed mutations | 15.64% | 14.19% | **13.09%** |
| gap (mutation share − retained share) | −0.39 pp | −0.16 pp | **+0.03 pp** |
| mean mutations/input, C | 25.57 | 85.00 | **253.77** |
| mean mutations/input, other | 26.32 | 86.15 | **253.16** |
| relative difference | 2.85% | 1.34% | **0.24%** |
| Welch's t | −1.46 | −1.21 | **0.34** |
| n (C / other) | 561 / 2939 | 512 / 3055 | 474 / 3155 |

**This backs §296.6's no-growth finding, and more strongly than its own two
points did.** Every pattern §296.6 reported holds a third time: C's share of
executed mutations tracks its share of the retained corpus almost exactly
(13.09% against 13.06%, 0.03 percentage points apart — tighter than either of
§296.6's own runs), the mean mutations per input for C and for the rest
differ by a quarter of one percent, and Welch's t (0.34) is the smallest of
the three, well inside the noise a sample this size would produce with no
real difference at all. Nothing here shows entropic spending a growing share
of effort on C's inputs as the run gets longer.

**If anything, the numbers move the opposite way from what growth would
predict.** C's share of the retained corpus falls monotonically as the run
gets longer — 16.03% → 14.35% → 13.06% — and the size of the C-vs-other mean
gap shrinks in the same order — 2.85% → 1.34% → 0.24%. §296.6 was careful to
call its own two points "evidence against a growing effect, not proof there
is none at any length," because two unpinned-seed one-shot snapshots
agreeing could still be coincidence. A third point, at three times the
longer run and in the same direction, is harder to read as coincidence: all
three quantities that a growing preference would have to move — retained
share, the mutation/retained gap, and the mean-mutations gap — are smaller at
2700 s than at 900 s than at 300 s, not larger. This lane cannot rule out
that a length far past 2700 s reverses that trend; what it can say is that
nothing in three points spanning a nine-to-one range points toward growth,
and the direction they do point is toward less divergence, not more.

`mise exec -- just gate quiet` on this section's own commit (`eba0b15a`) was
green: 2554 tests passed (3 slow), 12 skipped; 3 more tests and 25 doc tests
passed; `gate: PASS`.
