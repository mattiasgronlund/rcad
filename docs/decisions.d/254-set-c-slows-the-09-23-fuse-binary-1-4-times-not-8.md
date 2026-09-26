## 254. Set C slows the 09-23 fuse binary 1.4 times over two minutes, not the nights' 8 — 2026-09-26

*Runs queue row Q61. It tests §239.6's untested reading: that 09-23's 4.6 M
runs against 09-22's 36.7 M come from the 09-23 night starting with set C in
its corpus. The 09-23 binary ran for a fixed 120 s from its corpus, with C and
without it, three times each, taking turns. This lane only read and measured.
Nothing in the kernel, the nightly tree or the corpus was written, and every run
fell between 07:57Z and 08:15Z, outside 21:50–06:00Z.*

  254.1   What ran
  254.2   The table
  254.3   Where the difference goes
  254.4   What it shows for §239.6
  254.5   What C carries
  254.6   What was not run

### 254.1 What ran

**The binary.** `two_solids_fused_by_a_stranger`, built at `055aa4a9`, the
09-23 night's commit (§239.1), in a detached worktree at
`/tmp/q61-build-055aa4a9` with `cargo +nightly fuzz build`. Its build id is
`222ec625…`, not §239.1's `049fac09…`. So this is the same commit but a
different build, and the difference was not looked into.

**The two corpora.** Both were copied, never moved, out of the two directories
§239.2 names, and split by mtime at the same UTC times:

- **with C**: every file older than `2026-09-22 22:10:28 UTC`, which is A, B
  and C together. 11 077 names.
- **without C**: the same list minus C's 923 names (`-newermt '2026-09-21
  22:10:28 UTC' ! -newermt '2026-09-22 22:10:28 UTC'`). 10 154 names.

Names were de-duplicated across the two directories. That is why the without-C
arm is one short of §239.2's A + B (9 986 + 169). Each run's log confirms its
size, `seed corpus: files: 11077` or `files: 10154`, and no C name is in the
without-C copy.

**Each run** started on a fresh hard-linked copy of its arm's corpus:

```
<binary> -max_total_time=120 -timeout=10 -ignore_crashes=1 -ignore_timeouts=1 \
  -ignore_ooms=1 -print_final_stats=1 <copy>
```

That is one process with no `-fork`, so the three `-ignore_*` flags, which only
apply in fork mode, did nothing. Runs were counted as
`stat::number_of_executed_units`. The order was with, without, with, without,
with, without. No run crashed, timed out or stopped early.

### 254.2 The table

| pair | runs with C | runs without C | without ÷ with | 1-min load, with run | 1-min load, without run |
|------|------------:|---------------:|---------------:|----------------------|-------------------------|
| 1 | 39 194 | 57 862 | 1.48 | 3.33 → 3.35 | 3.35 → 2.28 |
| 2 | 37 799 | 49 040 | 1.30 | 2.28 → 4.92 | 4.92 → 8.34 |
| 3 | 40 150 | 57 275 | 1.43 | 8.34 → 2.99 | 2.99 → 2.03 |
| mean | 39 048 | 54 726 | 1.40 | | |

Pair 2's without-C run had the heaviest load of the six, from other sessions
building. It is that arm's low run, and it is still above every with-C run.

### 254.3 Where the difference goes

libFuzzer prints `INITED` when it has finished running the corpus files. Before
that line the process is running the files, and after it, the mutants. Every
log has `#N INITED … exec/s` and `#N DONE … exec/s` lines, and exec/s is runs
divided by whole seconds since start. That puts the switch at about a second's
precision:

| arm | time loading the corpus | mutants per second after |
|-----|------------------------:|-------------------------:|
| with C | 66 – 71 s | 494 – 559 (mean 527) |
| without C | 49 – 50 s | 547 – 663 (mean 624) |

**C costs about 17 – 21 s of loading.** That is close to 923 × 17 ms = 15.7 s,
using §239's per-file figure. **The mutants then run about 15% slower per
second with C in the corpus.** Loading takes up 40 – 60% of a 120 s run, so
most of the 1.40 is loading. If the mutant rates held, one long process would
lose about 15% of its runs to C, not the 29% it loses over two minutes.

### 254.4 What it shows for §239.6

**Starting from C is measurably slower, but by 1.3 – 1.5 times over two
minutes, not the 8 times between the nights (36.7 M ÷ 4.6 M).** This test used
one process, 120 s and the 09-23 binary on both arms. In that setup C explains
a gap five to six times smaller than the one it was put forward for, and the
part of the gap that is not loading is smaller still.

Loading alone cannot close that distance. At the measured 6.4 ms a file with C
and 4.9 ms without, a run that did nothing but load files would lose about a
quarter of its runs to C. For C to cost eight times, a night has to spend its
time on C's inputs, or on their mutants, far beyond their 8% share of the
files.

So this neither confirms nor breaks §239.6's leg (b), "what the night ran". The
nights did not run the way this test did (254.6).

### 254.5 What C carries

C is not dead weight:

- **With C**, `INITED` reports 17 539 – 17 540 edges and 90 685 – 90 697
  features.
- **Without C**, it reports 16 901 – 16 904 edges and 84 208 – 84 225 features.

None of the without-C runs found C's 638 edges and roughly 6 470 features
again. In 120 s they gained 0 – 1 edges and 170 – 184 features, against 0 – 10
and 25 – 82 for the with-C runs. Dropping C to win runs back would drop
coverage that two minutes of fuzzing does not rebuild.

C holds the corpus's rarest features, and that gives a way it could cost more
over a night than over two minutes. libFuzzer's default scheduler (entropic)
gives more mutations to inputs with rare features, so it has a reason to spend
a long run on C's slow inputs well past their 8% share. **This is a hypothesis
and was not measured.**

### 254.6 What was not run

- **`-fork=4`, which is how the nights ran (§234.4).** In fork mode libFuzzer
  gives each child job part of the corpus to run as seeds before it mutates.
  Loading is then paid once per job rather than once per process, at a cost
  that depends on how long each job runs and how many files it gets, and
  neither was measured. The next measurement is the same two arms under
  `-fork=4`, for a fixed time much longer than two minutes, still outside
  21:50 – 06:00Z.
- **Anything longer than 120 s.** Nobody knows whether the mutant rate holds
  after two minutes or shifts toward C as 254.5 suggests.
- **The 09-22 binary and the nights' own mutants**, as in §239.8.
- **Load was not removed, only spread by taking turns.** The 1-minute load
  average ran from 2.03 to 8.34.
