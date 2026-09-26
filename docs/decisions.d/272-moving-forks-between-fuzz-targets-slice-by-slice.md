## 272. Moving forks between fuzz targets slice by slice found no more coverage than the even split, and is not built — 2026-09-26

*Queue row Q148, which §265.7 queued: split the night's cores between the
targets slice by slice instead of evenly, and build it only if it beats the
even split on runs and coverage. This lane measured it on one commit, both ways
at once, for eight hours of daytime. It changed no code. Nothing in the night's
corpus or the nightly tree was written. Every fuzz process this lane started
ran between 12:20Z and 21:20Z, outside 21:50–06:00Z.*

  272.1   What ran
  272.2   What a restart costs
  272.3   The coverage the rule reads counts the timing crate's own code
  272.4   Runs and coverage, per target and per arm
  272.5   What the rule did with its forks
  272.6   Verdict
  272.7   What this leaves

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
binaries and their directory were made read-only. Their sha256 sums were
checked then and at the end, and they had not changed.

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
3 280, although they held every starting file.

### 272.4 Runs and coverage, per target and per arm

**Coverage, with the timing directory unset** (`cov:` edges):

| target | start | E, boundary 2 | R, boundary 2 | E, end | R, end |
|---|--:|--:|--:|--:|--:|
| where_two_surfaces_cross | 3 283 | 3 280 | 3 280 | ‹S_E› | ‹S_R› |
| reader | 4 943 | 4 953 | 4 943 | 4 953 | 4 944 |
| writer | 10 795 | 10 889 | 10 894 | 10 892 | 10 899 |
| fuse | 19 859 | 19 859 | 19 860 | ‹F_E› | ‹F_R› |
| cones and tori | 2 530 | 2 530 | 2 534 | ‹C_E› | ‹C_R› |

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

**R does not beat E, so the rule is not built.** On coverage, with the timing
directory unset, the two arms end within ‹N› edges of each other on every
target, and within a few edges of where they started, except for the writer.
On runs, R is ahead where it had more forks and behind where it had fewer, in
about the proportion of fork-seconds moved. Its one large gain, on the
cone-or-torus target, was not from its forks.

There was little for the rule to find. In eight hours on one commit, the
coverage that did not come from the timing crate rose by about 100 edges on
the writer, 10 on the reader in E, and ‹…› on the other three. That matches
§265.4: coverage creeps. A rule that moves forks toward rising coverage has
nothing to steer by when four of five targets barely move.

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
- **Two cheaper restarts were not measured.** Restarting only the targets whose
  count changed would have skipped all of R's third-slice restarts, since that
  split was even. And `-keep_seed=1` makes a fork parent skip the start pass
  (`FuzzerFork.cpp:342-345`). The parent then starts with no features, so it
  cannot tell a job's new input from one the corpus already had. Neither is
  worth measuring until a rule has a signal to act on.
- **Why R's cone-or-torus target ran 2.4 times as many inputs** (§272.5) is
  not known. It may be the restart, which resets job length and re-orders the
  seed list. It may be the spread that slice 1 already shows between two
  identical runs. One pair of runs cannot say.
