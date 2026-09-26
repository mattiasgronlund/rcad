## 259. Whether a seeded fuzz run repeats — 2026-09-26

*Queue row Q134, from §257.6: every A/B measurement lane in this repo assumes
that a run with a fixed seed repeats, and §257.2 showed one run of the old
writer target that did not, on a byte-identical corpus copy. This section
answers the question directly for today's five targets, on a rebuilt binary
with no probe of its own, and names two independent, confirmed causes.*

### 259.1 Method

Each of the five targets under `fuzz/fuzz_targets/` was built fresh from this
lane's tree with `RUSTFLAGS="-C target-cpu=x86-64-v3" mise exec -- cargo
+nightly fuzz build <target>` (the `RUSTFLAGS` repeats what the `fuzz-target`
recipe sets — without it `inari` fails to build under `cargo-fuzz`'s own
flags, unrelated to this question). Every run below is that binary, in
non-fork mode (no `-fork` flag), against a fresh `cp -r` of the tracked
`fuzz/corpus/<target>` made into a scratch directory outside the repo, never
the tracked directory itself, with `-seed=42 -print_final_stats=1` and a
fixed `-runs=N`.

One early trial run (before this method settled) was pointed at the tracked
corpus directly instead of a scratch copy, and left about eighty new files
under the tracked `fuzz/corpus/where_a_cone_or_a_torus_crosses/` plus four
loose `crash-`/`slow-unit-` files elsewhere in the tree. All of it was
untracked (`git status --short`, all `??`) and was removed with `git clean
-fd` before any of the numbers below were taken; nothing tracked was touched.

### 259.2 Corpus replay alone already differs, seed and corpus held fixed

Two runs of `no_step_file_makes_the_reader_panic`, `-seed=42 -runs=5000`,
`RCAD_FUZZ_TIMINGS` unset, each on its own fresh copy of the 909-file
tracked corpus:

| | `INITED` cov/ft | `INITED` corp | final cov/ft | new units added |
|---|---|---|---|---|
| Run 1 | 3860 / 7791 | 875 | 3862 / 7794 | 2 |
| Run 2 | 3862 / 7796 | 875 | 3863 / 7808 | 5 |

Both runs read "909 files found" from a byte-identical copy (`diff -rq`
against the pristine tracked corpus, run once, found no content
differences) and hit `INITED` before either has mutated anything, same as
§257.2's writer measurement. The two `cov`/`ft` figures already differ. A
second target, `no_model_makes_the_writer_panic` (`-seed=42 -runs=20000`,
1190-file corpus), shows the same shape at `INITED`: 8879/16867 against
8883/16873, `corp` equal at 1158 both times. A third,
`two_solids_fused_by_a_stranger` (`-seed=42 -runs=5000`, 1751-file corpus):
18476/84963 against 18472/84965, `corp` equal at 1548 both times. **`-seed=`
does not make corpus replay alone repeat**, on three of today's five
targets, confirming §257.2's finding still holds after §241's mutator
rewrite.

(The writer pair above also hit the same crash both times, at execution
`#1229` exactly, tallying the same `new_units_added: 18` first — the known
roundtrip mismatch of §249, "a turned bar comes back a few ulps off," not a
new defect. It does not bear on this section's question and was not
investigated further here.)

### 259.3 The cause: std's per-process HashMap/HashSet seed

`crates/rcad-step/src/read.rs` and `graph.rs` key a `HashMap<EntityId,
usize>`; `crates/rcad-topo/src/validate.rs`, `overlap.rs`, `measure.rs` and
`fuse/mod.rs` do the same for entity and edge lookups. Rust's
`std::collections::HashMap` draws a fresh per-process random key from one
`getrandom(2)` call the first time a `RandomState` is created (confirmed
with `strace -f -e trace=getrandom`, which shows exactly one such call
before the reader target's first crash-unrelated exit). Two processes draw
two different keys, so two runs that reach the same code but iterate a
`HashMap`/`HashSet` in a way that feeds back into control flow (an
early-exit loop, an order-dependent count) can legitimately execute
different instruction paths, and sancov's per-edge hit-count buckets can
then record different coverage from the same bytes.

Tested directly by fixing that one syscall. A five-line `LD_PRELOAD` shim
(`getrandom(3)` returns a fixed byte pattern; never linked into the fuzz
target, loaded only at run time) was compiled with the system `gcc` and
built once, in the scratchpad. Two runs of the reader target,
`-seed=42 -runs=5000`, both under `LD_PRELOAD=<the shim>.so`:

| | `INITED` cov/ft | `INITED` corp | final cov/ft | corp | new units |
|---|---|---|---|---|---|
| Run 1 | 3861 / 7792 | 875 | 3861 / 7794 | 877 | 2 |
| Run 2 | 3861 / 7792 | 875 | 3861 / 7794 | 877 | 2 |

Byte-for-byte identical, on the same seed and the same fresh corpus copy
that diverged without the shim (259.2). Turning the fix off (259.2's
baseline) makes the pair diverge again. That is this section's demonstration
for candidate (b): std's per-process `HashMap`/`HashSet` seed, unpinned by
`-seed=`, is a real and sufficient cause of two "identical" runs disagreeing
— confirmed on the reader target; not independently re-run on the writer or
fuse target, though both use the same kind of keyed map on the same kind of
lookup (259.7).

### 259.4 Address-space layout is not it

Candidate (c) was ruled out on the same target, the same way: `setarch -R`
disables ASLR for one run without touching `getrandom`. Two runs of the
reader target, `-seed=42 -runs=5000`, both under `setarch -R`, no
`LD_PRELOAD`:

| | `INITED` cov/ft | final cov/ft | new units |
|---|---|---|---|
| Run 1 | 3862 / 7795 | 3862 / 7798 | 3 |
| Run 2 | 3862 / 7792 | 3862 / 7794 | 2 |

Still different — turning ASLR off did **not** make the pair match, where
turning the HashMap seed off (259.3) did. Address-space layout is not the
cause, at least not on its own.

### 259.5 A second, independent cause: the `RCAD_FUZZ_TIMINGS` thread

`crates/rcad-fuzz-timing/src/lib.rs` starts a background thread, once per
process, only when `RCAD_FUZZ_TIMINGS` names a directory (`WATCH.get_or_init`
returns `None` otherwise, and `START_MERGING` is inside that `Some` branch —
so candidates (a), the clock, and (d), the extra thread, cannot be told
apart without changing that file, which is outside this lane's write set).
Turning it on is enough to break a target that otherwise repeats.

`where_two_surfaces_cross`, `-seed=42 -runs=1500`, 834-file corpus, no other
change:

| | `RCAD_FUZZ_TIMINGS` | `INITED` cov/ft | `INITED` corp | stopped at | final cov/ft | corp | new units |
|---|---|---|---|---|---|---|---|
| Run 1 | unset | 3107 / 8779 | 504 | `#1500` | 3112 / 8873 | 526 | 22 |
| Run 2 | unset | 3107 / 8779 | 504 | `#1500` | 3112 / 8873 | 526 | 22 |
| Run 1 | set (scratch dir) | 3756 / 9754 | 505 | `#1500` | 3761 / 9865 | 521 | 16 |
| Run 2 | set (scratch dir) | 3748 / 9794 | 508 | `#1643` | 3755 / 9903 | 530 | 18 |

With the variable unset the two runs are **identical in every figure
`-print_final_stats=1` prints** (only `stat::slowest_unit_time_sec` differs,
0 against 12 seconds — a wall-clock reading, not a correctness figure, and
expected to move with load). With it set, the same seed and the same fresh
corpus copy diverge, and not only in coverage: the two runs do not even stop
at the same total executed-unit count (`#1500` against `#1643`), something
259.2's pairs never showed.

The mechanism is not the clock value itself feeding into any decision the
target makes — `timed()` calls the same closure either way and returns what
it returns. It is that the merging thread's own code is part of the same
`-sanitizer-coverage-*`-instrumented binary, so *its* execution touches the
same process-global 8-bit edge counters libFuzzer reads to decide whether an
input is "new." A thread that runs unrelated instrumented code on its own
250ms clock, asynchronously with whichever input libFuzzer is currently
scoring, can get its own coverage attributed to that input — which is
exactly candidate (d), demonstrated rather than argued: no `HashMap` is
reachable from `where_two_surfaces_cross`'s oracle path (259.6), and this
pair still lands on two different stopping points from the same seed and
corpus, with the only difference being that the thread existed at all.

### 259.6 Two geometry-only targets that hold up through the corpus, then diverge under mutation

`where_two_surfaces_cross` and `where_a_cone_or_a_torus_crosses` call only
into `crates/rcad-geom/tests/fuzzing/mod.rs`, and neither `rcad-geom` nor
`rcad-predicates` has a `HashMap` or `HashSet` anywhere in `src/`
(`grep -rl 'HashMap\|HashSet' crates/rcad-geom/src crates/rcad-predicates/src`
returns nothing). That is consistent with 259.5's clean, fully-matching
`RCAD_FUZZ_TIMINGS`-unset pair for the surface target above.

`where_a_cone_or_a_torus_crosses`, `-seed=42 -runs=50000`, 116 corpus draws,
`RCAD_FUZZ_TIMINGS` unset:

| | `INITED` cov/ft/corp | final cov/ft | corp | new units |
|---|---|---|---|---|
| Run 1 | 1807 / 3604 / 105 | 2254 / 5655 | 459 | 475 |
| Run 2 | 1807 / 3604 / 105 | 2249 / 5751 | 457 | 497 |

`INITED` matches exactly — the tracked 38-seed corpus replays the same way
every time, with the timing thread off — but the two runs still end up
different once libFuzzer starts trying bytes the corpus does not already
hold, 49 884 more of them. The same shape shows up with
`RCAD_FUZZ_TIMINGS` set (`INITED` 1861/3680/107 both times; final
2828/6269/463 against 2826/6550/495). So this target's divergence is not
(259.5)'s thread, and 259.3's shim was not tried against it (259.7), so
its cause is measured but not identified. One plausible route: a
cone/torus draw that reaches meshing goes through
`crates/rcad-mesh/src/triangulate.rs`, which does key a `HashMap` — the 38
tracked seeds may simply not reach that far, where a mutated one can. That
is a hypothesis, not a finding; nothing here traced a specific input
through it.

### 259.7 What is left

- **Measured, not inferred:** every table above is a run made in this lane —
  259.2's three no-timings pairs, 259.3's shim pair and its no-shim
  baseline, 259.4's `setarch -R` pair, 259.5's four-run
  `where_two_surfaces_cross` set, and 259.6's two `where_a_cone_or_a_torus_crosses`
  pairs. The HashMap-seed mechanism (259.3) is confirmed **on the reader
  target only**; the same class of `HashMap` use sits on the writer's and
  the fuse target's own reachable paths (259.2), so the same cause is
  plausible there, but nobody ran the `LD_PRELOAD` shim against either of
  them — that is inferred, not measured, and is the cheapest follow-up if
  anyone wants it closed.
- `two_solids_fused_by_a_stranger` was not paired under
  `RCAD_FUZZ_TIMINGS=set`: the second run of that pair was killed by an
  infrastructure interruption unrelated to the target (this lane's own
  environment restarted mid-run), and it was not worth re-running once
  259.5 had already produced a clean causal pair on a different target.
  The no-timings pair for the fuse target (259.2) already shows the same
  divergence shape the reader target shows, which is the point that
  mattered here.
- `where_a_cone_or_a_torus_crosses`'s post-corpus divergence (259.6) is a
  residual in the same shape as §257.6's own: real, measured, and not
  traced to a specific line. Whoever next touches that target or
  `rcad-mesh::triangulate` and cares about reproducing a specific run
  should start there.
- **What a measurement lane should do.** Do not compare two single runs on a
  shared `-seed=` and call the difference the effect under test — 259.2
  and 259.5 both show two runs of the *unmodified* target landing on
  different numbers. Compare distributions instead (several seeds per arm,
  or several repeats of the same seed to size the noise floor first), and
  when the thing under test is coverage or a "came back the same" count
  rather than a crash, run both arms with `RCAD_FUZZ_TIMINGS` unset —
  259.5 shows that variable alone is enough to move the numbers on a
  target that otherwise reproduces exactly.

### 259.8 Corrected at merge: the thread is not shown to be the cause, and the variable changes how many inputs libFuzzer runs

**Re-run.** At merge the dispatcher re-ran 259.5's four runs on the lane's
own `where_two_surfaces_cross` binary, each on a fresh copy of the 834-file
corpus, `-seed=42 -runs=1500 -print_final_stats=1`, all four at once:

| | `RCAD_FUZZ_TIMINGS` | `INITED` at | `INITED` cov/ft | ended at | final cov/ft | corp | new units | time |
|---|---|---|---|---|---|---|---|---|
| Run 1 | unset | `#835` | 3107 / 8779 | `#1500` | 3112 / 8873 | 526 | 22 | 19 s |
| Run 2 | unset | `#835` | 3107 / 8779 | `#1500` | 3112 / 8873 | 526 | 22 | 19 s |
| Run 1 | set | `#902` | 3745 / 9680 | `#1500` | 3752 / 9837 | 534 | 27 | 37 s |
| Run 2 | set | `#903` | 3746 / 9754 | `#1794` | 3768 / 10063 | 535 | 24 | 51 s |

The unset pair matches 259.5's unset pair in every figure, so that result
stands. The set pair diverges, as 259.5 found.

**The mechanism paragraph in 259.5 is wrong on one leg.** It says the cause
is "not the clock value itself feeding into any decision the target makes"
and calls candidate (d), the thread, "demonstrated rather than argued." The
clock value does feed decisions, not in the target but inside `timed()`
(`crates/rcad-fuzz-timing/src/lib.rs`), on the main thread, on every draw:
`record` puts the elapsed time through `bucket_of`, and `Slowest::offer`
branches on it at `took <= self.runs[0].0`, in its `position` scan and in
`partition_point`. That code is instrumented like the rest of the binary; the
≈640 more edges at `INITED` with the variable set are that crate's own code
being counted. So candidate (a), the clock steering instrumented branches, is
enough on its own to break repeat without any help from the thread, and
259.5's own first paragraph is right that this pair cannot tell (a) from (d).
What 259.5 shows is that **setting the variable** breaks repeat. Both
candidates are code in `rcad-fuzz-timing`, so keeping that crate's code out
of what libFuzzer counts would remove both (Q136).

**The variable also changes how many inputs libFuzzer runs.** Unset, `INITED`
lands at `#835`, one run per corpus file plus the empty input. Set, it lands
67 and 68 runs later, and `-runs=1500` ended at 1643 runs in 259.5 and at
1794 here. Which inputs libFuzzer runs again, and why, is not known. Each
set run took about twice the unset time, with 10 s as the slowest unit in all
four, so the extra runs may include slow inputs run again; that is a
guess.

**Why this matters beyond measurement lanes.** `fuzz-overnight` exports
`RCAD_FUZZ_TIMINGS` for every target, so every night's corpus is chosen with
the timing crate's clock-steered edges counted as coverage. An input can be
kept because it took a new length of time rather than because it reached new
kernel code. How much of a night's new corpus that is has not been measured
(Q136).

**One smaller point.** 259.3's `strace` shows one `getrandom` call per
process. That this call is std's `RandomState` seed is inferred from reading
the code; the shim pins that one call, whatever makes it. The cause 259.3
confirms is therefore "the process's one random draw", and the reading that
names it the HashMap seed is very likely but was not traced.

Rows at merge: Q136 (keep the timing crate out of the coverage signal),
Q137 (the shim on the writer and fuse targets, parked until a lane needs two
runs that repeat exactly rather than 259.7's distributions), Q138 (the
cone/torus residual of 259.6, parked until someone needs to reproduce a run
of that target).
