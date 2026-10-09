## 272. Moving forks between fuzz targets slice by slice did not beat the even split, and is not built — 2026-09-26

*Queue row Q148, which §265.7 queued: split the night's cores between the
targets slice by slice instead of evenly, and build it only if it beats the
even split on runs and coverage. This lane measured it on one commit, both ways
at once, for eight hours of daytime. It changed no code. Nothing in the night's
corpus or the nightly tree was written. Every fuzz process this lane started
ran between 12:20Z and 21:20Z, outside 21:50–06:00Z; the last was gone by
21:07:14Z (`final.log` line 12). The lane died on the weekly usage limit
before the last three end measurements were written in. A second lane folded
them in from the first lane's scratchpad. It ran no fuzz binary. A third lane
(Q212) copied the files this section cites out of that scratchpad, before a
reboot could wipe it, into
`/home/mattias/rcad-fuzz-logs/q148-forks-slice-by-slice-2026-09-26`, which
every file named below is relative to; it checked each copy against the
scratchpad's file with `sha256sum`. The corpora and the five binaries were
left behind in the scratchpad, since nothing below cites them by path.*

  272.1   What ran
  272.2   What a restart costs
  272.3   The coverage the rule reads counts the timing crate's own code
  272.4   Runs and coverage, per target and per arm
  272.5   What the rule did with its forks
  272.6   Verdict
  272.7   What this leaves
  272.8   Two identical arms side by side, one slice — 2026-10-08
  272.9   The two arms' coverage with the timing directory unset — 2026-10-09

### 272.1 What ran

**Two arms of the night, side by side, on disjoint cores.** Both ran the five
targets on base `3319f838` with ten forks each, half a night's twenty, so both
fit on the machine at once. The machine has 12 cores and 24 threads; thread
*n* and *n* + 12 share a core. Arm E ran on threads 0,1,3,6,9 and their
siblings, arm R on 2,4,5,7,10 and theirs, so each arm had five whole cores and
each shared the four L3 caches the same way. `taskset` pins a fork parent, and
its jobs inherit that. Both ran under `nice -n 10` and `ionice -c 3`, which is
what `rcad-fuzz.service` gives the night.

- **E, the even split,** ran each target once for the whole window at
  `-fork=2`, with no restarts. This is `fuzz-overnight`'s night at half the
  size.
- **R, the rule,** started the same way and restarted all five targets at each
  slice boundary, each with the fork count the rule gave it. Each restart
  reloads the corpus.

**The rule.** At each boundary, a target whose `cov:` did not rise during the
slice drops to one fork. The other targets share the remaining forks equally,
and any spare forks go to the targets that gained the most. If no target rose,
the split is even. So the rule only differs from E when some target has
stopped rising. The coverage is the `cov:` on the fork parent's `#N` lines,
from the first line of the slice to the last. `cov:` counts edges, and it is
the number §265.4 read.

**What was not measured.** The row's other half says a target whose code
changed since the last night starts with more forks. On one commit no target's
code changes between slices, so that half never fires here. If it is built, it
ships unmeasured.

**Each target ran the way `fuzz-target` runs it at this base.** The flags were
`-fork=N -timeout=10 -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1`, plus
`-max_len=71` for `where_two_surfaces_cross`, `-max_len=81` for the cone-or-torus
target, and `-dict=step.dict` for the reader. The environment had
`ASAN_OPTIONS=detect_odr_violation=0`, which `cargo fuzz run` adds, and
`RCAD_FUZZ_TIMINGS` pointed at the arm's own directory. The first corpus was
the arm's own `cp -al` copy of `~/rcad-fuzz-corpus/<target>`. The second,
read-only, was the tracked `fuzz/corpus/<target>`. **No STEP corpus was passed
to the reader,** because `fuzz-reader` passes none at this base. §255's third
corpus argument is on a branch, not on `main`. So the 256 to 282 s the row
quotes for the reader's reload does not apply here. The reader's reload is
§272.2's.

**The binaries were built at the base and copied into the lane's scratchpad
before the run** (§268.5). The scratchpad turned out to be shared with the
dispatcher's other lanes, so once that was seen, about fifty minutes in, the
binaries and their directory were made read-only. Their sha256 sums, written
to `bin.sha256` when they were copied, were checked then, and `sha256sum -c
bin.sha256` still passed for all five on 09-27, after the last measurement.

**The slice length is 9 736 s, three slices in the window.** It was chosen
from the nights' own reloads. On the nights of 09-22 to 09-26, the first job
finished 1 286 to 2 299 s into the night for `where_two_surfaces_cross`, 588 to
1 334 s for the cone-or-torus target, and 114 to 1 156 s for the fuse. With
one-hour slices, those restarts would have taken up to two-thirds of a slice.
Three slices gives the rule two chances to move forks, and the costliest reload
is at most a quarter of a slice at night rates.

**Runs are counted two ways, because neither is complete.**
- libFuzzer's `#N` is the sum of each finished job's
  `stat::number_of_executed_units`. A job that ends on a timeout or a crash
  prints no final stats, so its runs are missing.
- The timing record (`RCAD_FUZZ_TIMINGS/<target>/totals`) also counts the
  runs of the parent's reload pass, which search nothing. It can also reset to
  zero (§269). A sampler read it every 60 s and added back any value lost to a
  reset.

For R, `#N` is summed over the slices.

**The machine was not the arms' alone.** Other lanes ran gates beside them all
day. The 1-minute load average was 31.85 at the start (12:23Z), 16.30 at the
first boundary (15:06Z), 22.17 at the second (17:48Z) and 12.43 at the end
(20:31Z). Its highest value in the 60-s samples was 57.95. The load hit both
arms at once, which is why they ran side by side.

### 272.2 What a restart costs

**A restart is a new fork parent, and a new parent first runs the whole corpus
once in a single process** (`FuzzerFork.cpp:348`, libfuzzer-sys 0.4.13). Its
other forks wait for that pass. The time to the first `#N` line is that pass,
in seconds:

| target | E, once | R slice 1 | R slice 2 | R slice 3 |
|---|--:|--:|--:|--:|
| where_two_surfaces_cross | 3 124 | 3 122 | 2 687 | 3 256 |
| reader | 37 | 37 | 22 | 52 |
| writer | 190 | 188 | 128 | 270 |
| fuse | 1 752 | 1 758 | 1 337 | 1 603 |
| cones and tori | 2 028 | 2 036 | 1 531 | 1 974 |

These are longer than the nights' (§272.1), because the machine was busy. The
nights paid the first column once. R paid it again at every restart:
between 45 and 54 minutes for `where_two_surfaces_cross`, between 22 and 27
for the fuse, and between 26 and 33 for the cone-or-torus target.

**A restart also starts the jobs short again.** A job runs for `min(300,
JobId)` seconds, and `JobId` starts from 1 in every new parent
(`FuzzerFork.cpp:130` and `:377`). Each job is seeded with about the square
root of the corpus size, picked with a bias to the files found last
(`:139-156`). A restart re-orders that list, so the bias no longer points at
the newest finds. This lane did not measure what either of these costs on its
own.

### 272.3 The coverage the rule reads counts the timing crate's own code

**The `cov:` on the night's `#N` lines includes about 570 to 600 edges of
`rcad-fuzz-timing`.** §267 found this for `where_two_surfaces_cross`: about 640
more edges with `RCAD_FUZZ_TIMINGS` set. The fix is a build change, and it is
not on `main` (§267.6). This lane measured the starting corpus with the same
single pass a fork parent makes on start, reading the first `#N` line's `cov:`.
The target ran with `-fork=1 -max_total_time=1` and a throwaway first corpus,
so the measured directories were read and never written. The reader's
starting corpus, measured four times:

| `RCAD_FUZZ_TIMINGS` | seeds kept | `cov:` | `ft:` |
|---|--:|--:|--:|
| unset | 2 769 | 4 943 | 17 196 |
| unset | 2 774 | 4 943 | 17 195 |
| a fresh directory | 2 805 | 5 507 | 18 329 |
| a fresh directory | 2 818 | 5 498 | 18 306 |

With the variable unset, the pass repeats. With it set, the same corpus gives
about 560 more edges, and two passes differ by 9. The other four targets show
the same gap between the arms' first `#N` line and this lane's pass with the
variable unset: 573 and 585 edges for `where_two_surfaces_cross`, 594 for the
writer, 592 and 596 for the fuse, 577 and 582 for the cone-or-torus target.

**This is what the rule acts on, and a restart moves it.** At R's first
restart, all five targets came back with fewer `cov:` edges than they had
ended slice 1 with: 16 fewer for `where_two_surfaces_cross`, 31 for the
reader, 44 for the writer, 40 for the fuse and 27 for the cone-or-torus
target. No corpus file was removed: fork mode deletes only its own job
directories and control files (`FuzzerFork.cpp:80-84`, `:183`, `:332`,
`:486`). In slice 2 all five then "rose", by 1 to
25 edges, and not one of them got back to where it had ended slice 1. So at
the second boundary the rule saw every target rising, and each of those rises
was ground R already had.

**So coverage is compared here with the timing directory unset.** Each arm's
corpus was hard-linked at the second boundary and at the end, and measured
with the same single pass, `RCAD_FUZZ_TIMINGS` unset, as the starting corpus.
That pass is not free of noise either. `where_two_surfaces_cross`'s starting
corpus measured 3 283, and both arms' corpora at the second boundary measured
3 280, although they held every starting file. At the end both measured 3 283
again (`final.log` lines 10–11).

### 272.4 Runs and coverage, per target and per arm

**Coverage, with the timing directory unset** (`cov:` edges):

| target | start | E, boundary 2 | R, boundary 2 | E, end | R, end |
|---|--:|--:|--:|--:|--:|
| where_two_surfaces_cross | 3 283 | 3 280 | 3 280 | 3 283 | 3 283 |
| reader | 4 943 | 4 953 | 4 943 | 4 953 | 4 944 |
| writer | 10 795 | 10 889 | 10 894 | 10 892 | 10 899 |
| fuse | 19 859 | 19 859 | 19 860 | 19 859 | 19 861 |
| cones and tori | 2 530 | 2 530 | 2 534 | 2 530 | 2 534 |

The start column is `measure/results.txt` lines 3–8, the boundary-2 columns
are its lines 9–18, and the end columns are `final.log` lines 2–11, which
`measure/results.txt` lines 19–28 repeat. Each corpus holds every file of the
one to its left: no starting file is missing from either arm's corpus at the
second boundary or at the end, and no second-boundary file is missing from the
same arm's end corpus, for any target.

**Coverage as the fork parents logged it** (`cov:`, timing crate included).
For E, the first line, the value at each boundary, and the last line. For R,
each slice's first and last line:

| target | E | R slice 1 | R slice 2 | R slice 3 |
|---|---|---|---|---|
| where_two_surfaces_cross | 3 856, 3 867, 3 867, 3 872 | 3 868 → 3 868 | 3 852 → 3 853 | 3 841 → 3 851 |
| reader | 5 512, 5 520, 5 528, 5 528 | 5 512 → 5 513 | 5 482 → 5 491 | 5 490 → 5 491 |
| writer | 11 389, 11 477, 11 486, 11 490 | 11 389 → 11 482 | 11 438 → 11 463 | 11 467 → 11 479 |
| fuse | 20 451, 20 453, 20 453, 20 453 | 20 455 → 20 455 | 20 415 → 20 419 | 20 435 → 20 438 |
| cones and tori | 3 107, 3 108, 3 108, 3 108 | 3 112 → 3 113 | 3 086 → 3 103 | 3 111 → 3 112 |

**Runs**, over the whole eight hours:

| target | E, libFuzzer | R, libFuzzer | R ÷ E | E, timing record | R, timing record | R ÷ E |
|---|--:|--:|--:|--:|--:|--:|
| where_two_surfaces_cross | 765 104 | 635 394 | 0.83 | 734 060 | 617 126 | 0.84 |
| reader | 190 141 951 | 222 845 740 | 1.17 | 190 400 525 | 222 846 793 | 1.17 |
| writer | 1 559 546 | 1 868 300 | 1.20 | 1 146 658 | 1 386 522 | 1.21 |
| fuse | 1 322 983 | 1 182 407 | 0.89 | 1 332 620 | 1 217 120 | 0.91 |
| cones and tori | 1 621 494 | 3 865 783 | 2.38 | 1 636 026 | 3 878 563 | 2.37 |

No timing record reset during the run. The sampler saw no drop in any of the
ten.

**Slice 1 is the same run twice,** since both arms ran at two forks apiece from
the same corpus. It shows how far two identical runs drift apart. At the first
boundary, libFuzzer's runs stood at 168 358 (E) and 193 426 (R) for
`where_two_surfaces_cross`, 61 597 415 and 61 519 207 for the reader, 608 437
and 581 571 for the writer, 407 030 and 450 063 for the fuse, and 392 260 and
507 432 for the cone-or-torus target. That is up to 29% apart with nothing
different between them.

**At the end, R's coverage stands 0 to 9 edges from E's, and the lane never
measured how far two identical runs drift apart on that count.** With the
timing directory unset, R ends level with E on `where_two_surfaces_cross`, 9
behind on the reader, 7 ahead on the writer, 2 ahead on the fuse and 4 ahead
on the cone-or-torus target (`final.log` lines 2–11). The spread that would
settle whether those gaps mean anything is two identical runs measured the same
way. Slice 1 was two identical runs, but no corpus was snapshotted at the
first boundary, so it has no figure on this count. What the lane did measure
comes close from three sides:

- **The measuring pass on its own.** Over the reader's starting corpus it gave
  4 943 twice (`measure/results.txt` lines 3–4). On `where_two_surfaces_cross`
  it gave 3 283 for the starting corpus (line 8), 3 280 for both arms at the
  second boundary (lines 15–16), and 3 283 for both at the end (`final.log`
  lines 10–11), although each corpus holds every file of the one before. So the
  pass alone moves by 3 edges on that target.
- **Two identical runs, on the count the fork parents log.** That count
  includes the timing crate. At the end of slice 1, R stood 1 ahead of E on
  `where_two_surfaces_cross`, 7 behind on the reader, 5 ahead on the writer, 2
  ahead on the fuse and 5 ahead on the cone-or-torus target
  (`boundaries.jsonl` line 1, each target's `E.last.cov` against
  `R.last.cov`). On their first lines, from the same corpus, they already
  stood 12 apart on `where_two_surfaces_cross`: 3 856
  (`E/logs/where_two_surfaces_cross.whole.log` line 9) and 3 868 (R's
  `first.cov` in `boundaries.jsonl` line 1).
- **Slice 3, where both arms ran two forks a target** and differed only in R's
  restart. It moved R's gap to E by at most 2 edges on any target, with the
  timing directory unset (the boundary-2 columns of the table above against
  its end columns).

Each end gap is within 2 edges of the gap the same target showed after the
identical slice 1, and on four of the five it has the same sign: 0 against 1,
−9 against −7, +7 against +5, +2 against +2, and +4 against +5. So the gaps at
the end were about there before the rule moved a fork. That comparison mixes
two counts, the one without the timing crate and the one with it, so it is not
a measured tie. It is a gap this lane cannot tell from the drift between two
identical runs.

**`ft:` tells the same story.** `ft:` counts features, which is finer than
edges. With the timing directory unset, R ends 20 ahead of E on
`where_two_surfaces_cross`, 27 behind on the reader, 213 ahead on the writer,
10 ahead on the fuse and 14 ahead on the cone-or-torus target (`final.log`
lines 2–11). The writer's 213 is the only gap above 30. After the identical
slice 1, the two arms already stood 182 apart on the writer's logged `ft:`,
55 298 for E and 55 480 for R (`boundaries.jsonl` line 1).

### 272.5 What the rule did with its forks

| boundary | `cov:` gains it read (surfaces, reader, writer, fuse, cones) | forks for the next slice |
|---|---|---|
| start | — | 2, 2, 2, 2, 2 |
| 1 | 0, 1, 93, 0, 1 | 1, 3, 3, 1, 2 |
| 2 | 1, 9, 25, 4, 17 | 2, 2, 2, 2, 2 |

At the first boundary the rule took a fork each from `where_two_surfaces_cross`
and the fuse, and gave them to the reader and the writer. At the second,
every target had risen, so the split went back to even (§272.3 says what those
rises were). R therefore differed from E in the forks it had for one slice
of three, and in two restarts.

**The runs follow the forks, except for one target.** One slice of three with
three forks instead of two is 1.17 times the fork-seconds, and with one
instead of two it is 0.83 times. The reader came out at 1.17 and the writer at
1.20. `where_two_surfaces_cross` came out at 0.83 and the fuse at 0.89. So
moving forks moved runs about in proportion, and bought nothing on top.

**The cone-or-torus target ran 2.4 times E's runs with the same two forks in
every slice.** That is not the rule, since the rule never changed its forks.
E's jobs for it ended on the 10 s timeout in 168 of 373 jobs. R's did so in
48 of 169, 22 of 168 and 53 of 169 in its three slices. This lane did not find
why. Slice 1, with nothing different between the arms, already had R 29% ahead
on this target.

### 272.6 Verdict

**R does not beat E, so the rule is not built.** The row asks for the rule to
win before it is built, and it did not. On coverage, with the timing directory
unset, the two arms end within 9 edges of each other on every target. Those
gaps are about the ones two identical runs already had after slice 1, and this
lane cannot tell them from drift (§272.4). On runs, R is ahead where it had
more forks and behind where it had fewer, in about the proportion of
fork-seconds moved. Its one large gain, on the cone-or-torus target, was not
from its forks. So Q148 is dropped with these numbers, as the row says to do
when the rule does not win.

**Nor do the gaps point the rule's way.** The rule moved forks in slice 2 only
(§272.5). Of the two targets it gave a third fork, R ends 7 ahead on the writer
and 9 behind on the reader, where it was already 7 behind after slice 1. Of
the two it cut to one fork, R ends level on `where_two_surfaces_cross` and 2
ahead on the fuse. Had the moved forks bought coverage, the first two would
end ahead and the second two behind.

There was little for the rule to find. In eight hours on one commit, the
coverage that did not come from the timing crate rose by 97 edges on the
writer in E and 104 in R, by 10 on the reader in E and 1 in R, and by at most
4 on the other three: 0 in both arms on `where_two_surfaces_cross`, 0 and 2 on
the fuse, 0 and 4 on the cone-or-torus target (§272.4's table, its start
column against its end columns). That matches §265.4: coverage creeps. A rule
that moves forks toward rising coverage has nothing to steer by when four of
five targets barely move.

**It would read the wrong number anyway.** Until §267's build change lands,
the `cov:` the night logs counts the timing crate's own edges, and two passes
over the same corpus differ by about 9 of them (§272.3). The rule acts on rises
of 0 to 25 edges. After a restart, it read re-found edges as new ground.

### 272.7 What this leaves

- **The "code changed since the last night" half is unmeasured.** On one commit
  it never fires. Measuring it needs nights on different commits. Q146's
  report over many nights is the place for that, and §265.4 already reads the
  big steps between nights as new code arriving, not as finds.
- **Any coverage-steered rule needs §267's build change first.** Without it,
  the signal includes 570 to 600 edges of timing code per target, and they
  move between passes and across restarts.
- **A rerun should snapshot both corpora at the first boundary too.** Slice 1
  is two identical runs, and measured with the timing directory unset it would
  give the spread that §272.4 had to approach from three sides. It costs one
  more measuring pass per target and arm. On the starting corpus the longest,
  `where_two_surfaces_cross`'s, took 2 577 s (`measure/results.txt` line 8).
- **Two cheaper restarts were not measured.** Restarting only the targets whose
  count changed would have skipped all of R's third-slice restarts, since that
  split was even. And `-keep_seed=1` makes a fork parent skip the start pass
  (`FuzzerFork.cpp:342-345`). The parent then starts with no features, so it
  cannot tell a job's new input from one the corpus already had. Neither is
  worth measuring until a rule has a signal to act on.
- **Why R's cone-or-torus target ran 2.4 times as many inputs** (§272.5) is
  not known. It may be the restart, which resets job length and re-orders the
  seed list. It may be the spread that slice 1 already shows between two
  identical runs. One pair of runs cannot say. §272.8 is a second pair.

### 272.8 Two identical arms side by side, one slice — 2026-10-08

*Queue row Q213, which §272.4 and §272.7 left: run the even split twice side
by side on one commit, and measure how far two identical runs drift apart. A
read-only lane ran it (run `wf_1d172cac-3d2`) and changed no code. Every
number below is from its report, which gives the command behind each; the
report is that run's `journal.jsonl` under session
`7e0b4050-c5e7-4c79-b9c6-f16d95a5d801`. Its logs, timing records and end
corpora are in its tree's `target/q213/` (`wf_1d172cac-3d2-1`) and nowhere
else yet.*

**What ran.** Two arms, A and B, each run as arm E of §272.1, two forks a target
and no restarts, with the five binaries §272 used, built at 3319f838. A ran on
cores 0, 1, 3, 6, 9, 12, 13, 15, 18 and 21, B on 2, 4, 5, 7, 10, 14, 16, 17,
19 and 22, for one slice of 9 736 s, §272.1's length, from 09:43:31Z to
12:27:01Z. Other lanes were building and testing on the machine at the same
time. The load average was 34.6 at the start and 25.1 at the end, on 24 cores.

| target | runs, A | runs, B | B over A |
|---|---|---|---|
| `where_two_surfaces_cross` | 311 780 | 283 017 | 0.91 |
| the reader | 76 146 853 | 76 526 460 | 1.00 |
| the writer | 553 778 | 545 999 | 0.99 |
| the fuse | 497 214 | 529 431 | 1.06 |
| the cone-or-torus target | 1 191 387 | 1 575 680 | 1.32 |

Runs are libFuzzer's `#N` on each fork parent's last line. The timing records'
`runs` give the same ratios to within 0.02.

**Two identical arms drift up to 1.32 times apart in runs in one slice, and
the cone-or-torus target is again the one that drifts most.** Slice 1 of
§272.4, the only other pair of identical runs, had R at 1.29 times E on it.
Two samples, both about 1.3. R's 2.4 times E over three slices (§272.5) is far
past either, so the spread between identical runs does not explain most of it.
That leaves the restart as the likelier cause: R restarted at both boundaries,
and no identical arm ever restarted. That is not shown. Two samples are not a
spread, and nothing here ran a restart alone.

**On the coverage the fork parents log, which counts the timing crate, A and B
end 7, 47, 9, 8 and 1 edges apart**, in the table's order: 3 916 and 3 909,
5 680 and 5 727, 11 513 and 11 504, 20 651 and 20 643, 3 138 and 3 139. The
reader's 47 is larger than any gap §272.6 read between R and E. Those gaps
were 0 to 9 edges, measured with the timing directory unset, which is the
other count. So this does not settle §272.4's question. It does point the way
§272.6 already went: the end gaps are within what two identical runs drift.

**The count the row asked for was not measured.** The lane hard-linked both
arms' end corpora (`target/q213/{A,B}/end-corpus/<target>/`) and began the
measuring pass with the timing directory unset. Its measuring script,
`target/q213/measure.py`, starts libFuzzer in a session of its own. So a pass
it killed or wrapped in `timeout` went on running, and it ended up with two or
three copies of one pass on the same two cores. It judged those numbers
unreliable and reported none. It also reported that nothing it started was
still running. At 12:40Z, fifteen processes with its tree in their command
lines were running, started between 12:34:53Z and 12:36:11Z. The measuring
pass alone, on the corpora already linked, is Q1383.

**A third pair is not run.** The question it would answer, restart or spread,
steers nothing. The rule is not built (§272.6), and §272.7 already says a
cheaper restart is not worth measuring until a rule has a signal to act on.

### 272.9 The two arms' coverage with the timing directory unset — 2026-10-09

*Queue row Q1383, which §272.8 left: measure Q213's two end corpora with the
timing directory unset, one pass per target and arm. A read-only lane ran it
(run `wf_728a8da1-296`, its report in that run's `journal.jsonl` under session
`c5002611-b420-4273-a11d-b3ee7ccfd39e`) and changed no code. It ran Q213's
`measure.py` with the five binaries built at 3319f838, on threads 8 and 20,
one pass at a time, A then B for each target, from 13:35Z to 17:15Z. Before
the first pass and after the last, `pgrep -af target/q213` showed no process
but its own. The dispatcher checked every figure below against
`measure/results.txt`, then copied Q213's logs, timing records and every
measuring pass, 2026-10-08's and these, into
`/home/mattias/rcad-fuzz-logs/q213-two-identical-arms-2026-10-08`, which every
file named in this subsection is relative to, and removed Q213's tree. The
corpora and the crash artifacts were not copied, since nothing below cites
them by path.*

| target | cov, A | cov, B | gap | ft, A | ft, B | timeouts, A | timeouts, B |
|---|--:|--:|--:|--:|--:|--:|--:|
| where_two_surfaces_cross | 3 072 | 3 075 | B +3 | 12 267 | 12 281 | 69 | 23 |
| reader | 5 102 | 5 162 | B +60 | 17 644 | 17 679 | 0 | 0 |
| writer | 10 870 | 10 860 | A +10 | 54 978 | 55 004 | 0 | 0 |
| fuse | 19 968 | 19 967 | A +1 | 109 235 | 109 205 | 15 | 13 |
| cones and tori | 2 548 | 2 548 | 0 | 7 638 | 7 639 | 0 | 1 |

The coverage is `measure/results.txt` lines 4–13. The timeouts are the
`timeout-*` files in each pass's own `measure/q1383-*` directory.

**Two identical runs end up to 60 edges apart after one slice, so the 0 to 9
edges §272.6 read between R and E are not a signal.** R against E was 0, −9,
+7, +2 and +4 in the table's order (§272.4); A against B is +3, +60, −10, −1
and 0. On the reader and the writer, the two targets the rule gave a third
fork, identical arms drifted further than R from E did: 60 against 9 and 10
against 7. On the fuse and the cone-or-torus target this one pair drifted less
than R from E, 1 against 2 and 0 against 4. One pair is one sample a target,
and neither of those gaps points the rule's way (§272.6), so they change
nothing. On the count the fork parents log, the two arms were 7, 47, 9, 8 and 1
edges apart (§272.8). The reader is the largest gap on both counts, with B
ahead on both.

**One pass over a corpus repeats to within 2 edges.** Q213's passes of
2026-10-08 measured three of A's end corpora while two or three copies of a
pass shared the cores (§272.8). They read 5 102, 10 870 and 3 070 for the
reader, the writer and `where_two_surfaces_cross`; these passes read 5 102,
10 870 and 3 072 (`measure/results.txt` lines 1–3 against 4, 6 and 12). So the
reader's 60 and the writer's 10 come from the corpora, not from the measuring.

**Timeouts move this count by little.** The 2026-10-08 pass over A's
`where_two_surfaces_cross` corpus left 215 timeout files and read 3 070; this
one left 69 and read 3 072 (`measure/end-A-zg3c9pzs`,
`measure/q1383-A-hcs_lasv`). A timed out three times as often as B on that
target, on inputs that were mostly different (7 names in common), and ends 3
edges behind. On the reader and the writer, where the gaps are largest, neither
arm timed out.

**The crash files are the 2026-09-26 harness's own asserts, and the same in
both writer arms.** Each writer pass left 335 `crash-*` files, the same 335
names in A and B and in 2026-10-08's pass over A; the fuse passes left 121 and
123. Replayed on the 3319f838 binary by the dispatcher, nineteen of the first
twenty writer files stop at `writer.rs:262`, a face read back with a different
surface, and one at `writer.rs:186`, a written file the reader refuses. The
first fuse file stops at `fuse.rs:490`, where `both_ways_round` returns an
error.
They are the old build's answers and say nothing about today's code. No row is
filed for them.

**Verdict: §272.6 stands.** The rule is not built. Two identical runs,
measured with the timing directory unset, end up to 60 edges apart, and R and
E ended at most 9 apart.
No row is filed.
