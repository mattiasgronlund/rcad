## 227. Why the cone-or-torus fuzz target fell 70x, and why the fuse gets fewer runs forked — 2026-09-25

*Lane T1 of `docs/test-plan.md`, argued in §226.3. Two drops in a night's run
count, each asked "why": `where_a_cone_or_a_torus_crosses` from 160 M runs on
2026-09-20 to 2.3 M on 2026-09-21 with the same flags, and
`two_solids_fused_by_a_stranger` from 3.0 M on 2026-09-18 on one core to 2.0 M
on 2026-09-25 under `-fork=4`. Read-and-measure only: nothing in the kernel or
the night changed.*

  227.1   Which commit each night fuzzed
  227.2   The cone or torus target fell during 09-20, not between the nights
  227.3   The fuse: each run got dearer, and fork mode is not why
  227.4   What was run, and how to run it again
  227.5   What is left

### 227.1 Which commit each night fuzzed

`watermark.log` is empty on every one of these nights, so it names nothing. The
commit comes from the reflog of the tree the night checks out, read without
writing to it: `git -C ~/git/mattiasgronlund/rcad-nightly-fuzz reflog
--date=iso`. Each night's `checkout: moving … to <sha>` entry at 00:10 is that
night's commit:

| night      | commit     |
|------------|------------|
| 2026-09-18 | `74bd5573` |
| 2026-09-20 | `146d0d62` |
| 2026-09-21 | `ae124a29` |
| 2026-09-25 | `6664fef8` |

The reflog also shows a hand checkout of `a3f8ffe9` at 21:11 on 09-20, between
two nights; the 00:10 checkout on 09-21 replaced it before that night built.

### 227.2 The cone or torus target fell during 09-20, not between the nights

**The 160 M is almost all from the first three hours of 09-20.** Read from
`~/rcad-fuzz-logs/2026-09-20/fuzz-cones-and-tori.log`: the count stood at
155 344 035 at 10 494 s, running about 3 600 inputs a second. The next job
line, at 10 566 s, has coverage jump from 2 431 to 2 985 edges in one step, and
from there to the end of the night the rate is 10 to 50 a second. The last
five hours added about 5 M runs. The fuzzer had found a new part of the code,
and the inputs that reach it are slow.

**How slow, read from the night's own timing record**
(`timings/where_a_cone_or_a_torus_crosses/report`, which `rcad-fuzz-timing`
writes): the 64 slowest inputs of 09-20 took 3.1 to 3.4 seconds each, and every
one of them is dated 03:16 or later — after the jump. The median input took
8 microseconds. So one of these inputs costs as much as about 400 000 ordinary
ones.

**The inputs it found are what 09-21 started from.** The night writes what it
finds into `~/rcad-fuzz-corpus/where_a_cone_or_a_torus_crosses`, and the next
night reads that directory back as its seed. 09-21's log opens at coverage
3 006 — where 09-20 ended, not where it began — and its very first job lines
run at 1 to 6 inputs a second. Its timing record has p98 at 1 141 ms and p99 at
2 282 ms, where 09-20's p98 was 1.1 ms: on 09-21 one input in fifty took over a
second.

**The kernel did not get slower, because the code did not change.** Between
`146d0d62` and `ae124a29` no file under `crates/rcad-geom`,
`crates/rcad-predicates`, `crates/rcad-fuzz-timing` or `fuzz/` changed, nor
`Cargo.lock` (`git diff --stat 146d0d62 ae124a29 -- …` lists only the
`justfile`). The 09-21 build did not even recompile `rcad-geom`
(`2026-09-21/build.log`), and both nights' binaries report the same 18 311
coverage counters. Two inputs appear among the slowest on both nights under
the same name, `…55ef55b72ea716d8` and `…80c9abddbf1e1f22`, at 3.2 to 3.3
seconds each on both.

**The flags did not change either.** Both nights ran `-max_len=81 -fork=4
-timeout=10 -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1`; only
`-max_total_time` differs, by nine seconds (read from the `Running` line of
each log).

So of the four candidates: (a) the kernel got slower — ruled out, the target's
code is byte for byte the same; (c) the recipe changed — ruled out, the command
line is the same; (d) is about the fuse only. What is left is (b), and it
happened inside one night rather than between two: **09-20 found a slow region
three hours in, and every night since starts inside it.** §227.4 checks both
legs with runs: the two nights' binaries take the same time over the same
inputs, and the inputs 09-20 added take about 5 000 times as long as the ones
it started from.

### 227.3 The fuse: each run got dearer, and fork mode is not why

The two counts are not the same length of night. Read from the `Running` line
of each `fuzz-fuse.log`: 09-18 ran `-max_total_time=5741` — the five targets ran
one after another then — and 09-25 ran `-max_total_time=28730` with `-fork=4`.
So 09-18 made 2.97 M runs in 5 742 seconds on one process, about 518 a second;
09-25 made 2.05 M in 28 747 seconds on four, about 18 a second each. **Per
process, the fuse runs 29 times slower than it did on 09-18.**

**Fork mode's own overhead is small.** The timing record counts the time spent
inside the fuzz target, so it separates the fuse's own work from what libFuzzer
does around it — restarting jobs, merging the corpus:

| night | runs (timing record) | seconds inside the target | process-seconds | share inside | mean per run |
|-------|---------------------:|--------------------------:|----------------:|-------------:|-------------:|
| 09-18 | 2 917 410 | 5 454.7 | 5 742 (1 × 5 742) | 95% | 1.87 ms |
| 09-25 | 2 066 298 | 104 969.7 | 114 988 (4 × 28 747) | 91% | 50.8 ms |

Read from `timings/two_solids_fused_by_a_stranger/report` of each night. Fork
mode costs four points of the night; the other 25x is inside the fuse.

**Contention and fork mode together cannot be the cause, because 09-22 had both
and made 36.7 M runs.** Same flags, same five targets at once, four forks each:
09-22's timing record reads 36 707 613 runs in 98 766.8 seconds, 2.69 ms a run.
The run count of the fuse swings eighteen-fold from one night to the next under
one recipe:

| night | runs | mean per run | p99 |
|-------|-----:|-------------:|----:|
| 09-19 | 4.49 M | 6.3 ms | 38 ms |
| 09-20 | 3.48 M | 30.1 ms | 59 ms |
| 09-21 | 1.30 M | 70.2 ms | 1 007 ms |
| 09-22 | 36.71 M | 2.7 ms | 48 ms |
| 09-23 | 4.58 M | 23.1 ms | 67 ms |
| 09-24 | 2.13 M | 50.0 ms | 2 684 ms |
| 09-25 | 2.07 M | 50.8 ms | 2 550 ms |

(Timing record of each night. 09-19 ran one process for eight hours, no fork.)
The nights that ran few inputs are the nights whose p99 is in seconds: on 09-25
one input in a hundred took 2.5 seconds or more, and those alone account for at
least half of that night's mean. So the fuse's run count is set by how much of
its night goes on a small tail of very slow inputs — the same shape as §227.2,
and the same shape §91.3 found in `where_two_surfaces_cross` on 2026-09-11.

What the logs cannot say is whether that tail is slow because the fuse got
slower on the same inputs (a) or because the corpus moved toward inputs that
were always slow (b). Between `74bd5573` and `6664fef8` the fuse's code grew
from 86 179 to 104 953 coverage counters, so (a) was open. §227.4 closes it:
**the 09-25 fuse is 17% faster than the 09-18 fuse on 09-18's inputs, and the
inputs the corpus gained in that week are 36 times slower than the ones it
had.** It is (b), the same cause as the cone target's.

(c) did land in the window — `0ac02509` runs the five targets at once,
`22684dad` forks, `52ea7984` cuts one input to ten seconds — but none of it
sets the run count. Fork mode costs four points (the table above); the ten
seconds cut 2 inputs on 09-25 (`oom/timeout/crash: 0/2/0`, last line of
`fuzz-fuse.log`); and 09-19, with the five at once but no fork, already ran
6.3 ms a run against 09-18's 1.87. No night was run with fork mode turned back
off, so "toggling it back reproduces the drop" was not tried; 09-22's 36.7 M
under the same flags is the evidence in its place.

### 227.4 What was run, and how to run it again

**Four binaries**, built in this lane's worktree at each night's commit the way
`fuzz-target` builds them: `cd fuzz && RUSTFLAGS="-C target-cpu=x86-64-v3"
cargo +nightly fuzz build <target>`, then the binary copied out of
`fuzz/target/x86_64-unknown-linux-gnu/release/`. The cone target at `146d0d62`
and `ae124a29`; the fuse at `74bd5573` and `6664fef8`.

**Fixed input sets**, copied out of the two corpus directories each night reads
(`~/rcad-fuzz-corpus/<target>` and the nightly tree's `fuzz/corpus/<target>`)
and split by when each file arrived — `find … -newermt <from> ! -newermt <to>`,
with the night's 00:10 start as the edge:

- cone target, **before 09-20**: 1 525 files, all run.
- cone target, **added during 09-20**: 163 files, every fourth run (40).
- fuse, **before 09-18**: 8 951 files — the log's seed count that night is
  8 978 — every fifth run (1 790).
- fuse, **added from 09-18 to 09-25**: 2 675 files, every third run (891).

**How each was timed.** Each binary is handed the files by name, which makes
libFuzzer run each one once and print `Executed <file> in N ms`; those lines are
summed. Batches of 10 (the slow cone set) to 500, and inside each batch every
binary ran the same files, **with the order swapped every batch** — A then B,
then B then A — so that a change in machine load falls on both alike. Other
sessions were building on the machine throughout: the one-minute load average
read 3.0 to 15.5 across the runs, logged beside each batch. So only the ratios
below mean anything; the absolute times are not comparable with a night's.

| target | inputs | 09-20 / 09-18 binary | 09-21 / 09-25 binary | ratio |
|--------|--------|---------------------:|---------------------:|------:|
| cone or torus | 1 525 from before 09-20 | 0.14 ms each | 0.20 ms each | — |
| cone or torus | 40 added during 09-20 | 1 083 ms each | 962 ms each | 0.89 |
| fuse | 1 790 from before 09-18 | 5.42 ms each | 4.49 ms each | 0.83 |
| fuse | 891 added 09-18 to 09-25 | 295 ms each | 160 ms each | 0.54 |

Load average during the cone rows 9.7 to 15.5, during the fuse rows 3.0 to 8.6.
The cone target's fast set is too fast for libFuzzer's whole-millisecond
`Executed` line: most inputs print 0 ms, so its two figures are 214 ms and
311 ms spread over 1 525 inputs, and the only claim made from them is that they
are thousands of times below the slow set.

What the table says, leg by leg:

- **(a) is broken for both targets.** The later binary is no slower on the same
  inputs — the cone target's two binaries are the same code and time the same
  within noise, and the 09-25 fuse is 17% *faster* than the 09-18 fuse on
  09-18's own inputs.
- **(b) holds for both.** On one binary, the inputs a night added are far
  slower than the inputs it started from: the cone target's about 5 000 times
  (962 ms against 0.20 ms, both on `ae124a29`), the fuse's 36 times (160 ms
  against 4.49 ms, both on `6664fef8`). The fuse's new inputs are slow on the
  09-18 fuse as well — slower, at 295 ms — so they are slow because of what they
  are, not because the kernel changed under them.

  *Later, §240: a corpus file's mtime can be a later night's rewrite rather
  than the night it arrived, so the "added" sets here may hold older inputs
  too. §240.7 argues that can only make (b)'s gap a floor, never a ceiling.*

The fuse's 36 times per input on this machine and the 27 times per run between
the two nights' timing records (1.87 ms to 50.8 ms, §227.3) are the same size.
They need not match: a night runs mutants of its corpus, not the corpus itself.

### 227.5 What is left

- **What the slow inputs are doing.** Neither target was profiled. The cone
  target's 3-second inputs are kept by name in each night's
  `timings/where_a_cone_or_a_torus_crosses/slowest/` —
  `slow-000003289151us-55ef55b72ea716d8` from 09-21 is one that was slow on both
  nights — and `cargo fuzz run where_a_cone_or_a_torus_crosses <file>` under a
  profiler would name the routine. Whether 3 seconds for one pair of surfaces
  is a gap to fix or a price to pay is the kernel's question, not this lane's.
  Until it is answered, every night on that target spends almost all its time
  in that one region.
- **09-22's 36.7 M fuse runs** under the same flags as its neighbours are
  unexplained. It started from 3 107 seeds against 3 297 to 3 687 on the
  nights round it, which would fit a night that happened not to start among
  the slow inputs, but that was not checked.
- **Contention was not measured on its own.** Five targets at four forks is 20
  processes on this machine's 24 threads. Nothing here separates what that
  costs from the input-driven slowdown, except that (b) alone is big enough to
  account for all of it.
- **The cone target's timing record undercounts under fork mode.** 09-21's
  record reads 1 009 847 runs and 48 527.8 s inside the target, where libFuzzer's
  own count is 2 298 296 and four forks had about 115 000 process-seconds; 09-20's
  record, by contrast, agrees with libFuzzer to half a percent. Seen, not
  looked into. The fuse's records agree with libFuzzer within 2% on both nights
  used here, which is why §227.3 leans on them.
