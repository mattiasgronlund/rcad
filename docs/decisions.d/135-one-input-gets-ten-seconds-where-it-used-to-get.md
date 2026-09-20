## 135. One input gets ten seconds, where it used to get twenty, measured 2026-09-18

*§129's item 2. `where_two_surfaces_cross` fuzzed 259 136 inputs on the night of
2026-09-18 while the writer target fuzzed 94 897 507 — 366 times as many in the
same window. Its own timing report says why: **99% of its inputs account for at
most 7% of its night**, and the 64 slowest kept are every one of them over
seventeen seconds. `-timeout=10` bounds that, and only under `-fork`, where a
timeout is survivable.*

### 135.1 What one target's night was actually spent on

The five reports from `~/rcad-fuzz-logs/2026-09-18/timings/`, the last one-core
night, each target having had about 5 000 seconds:

| target | runs | p90 | p99 | p100 |
| --- | ---: | ---: | ---: | ---: |
| `where_two_surfaces_cross` | 259 136 | 1.05 ms | 7.86 ms | **19 327 ms** |
| `two_solids_fused_by_a_stranger` | 2 917 410 | 5.24 ms | 31.5 ms | 88 ms |
| `where_a_cone_or_a_torus_crosses` | 29 415 705 | 0.52 ms | 0.89 ms | 151 ms |
| `no_step_file_makes_the_reader_panic` | 44 789 305 | 0.30 ms | 1.02 ms | 21 ms |
| `no_model_makes_the_writer_panic` | 94 897 507 | 0.04 ms | 1.38 ms | 17 ms |

The first row is four orders of magnitude off the last, and the tail is where it
goes. Bounding from the percentiles, which is all they support:

- the bottom 90% — 233 222 runs at no more than 1.049 ms — is **at most 245 s**
- the p90-to-p99 band — 23 322 runs at no more than 7.864 ms — adds **at most 183 s**
- so 99% of its inputs cost **at most 428 s of 5 774**, and the remaining 2 592
  runs carry **at least 5 346 s: about 93% of the target's night**, averaging
  2.07 s each

And at the very end, the 64 runs the report keeps are all between 17.06 s and
19.59 s, so at least 1 091 s — **19% of the night in 0.025% of the inputs**.

This is the first time that question could be asked. §129's timing crate is what
produced these tables, and §133 is what made them survive fork mode; before them
there was libFuzzer's one ratcheted `Slowest unit:` line and no spread at all.

### 135.2 Why ten and not one

Ten seconds is 1 270 times that target's p99 of 7.86 ms. A far tighter bound was
available and would have paid better — capping at one second would leave those
2 592 runs costing at most 2 592 s instead of 5 346, recovering something like
half the target's window and roughly doubling its run count.

It was refused, and the reason is what the slow inputs *are*. **An input killed
by `-timeout` never enters the corpus.** On this target the slow inputs are the
nearly-parallel crossings — §0's whole unit is "a crossing too nearly parallel to
follow", and §89 records fine deflection costing ~2.9 s as legitimate work, not a
defect. A one-second bound would buy throughput by excluding from the corpus the
exact family this kernel is being worked on, and would do it silently.

So the bound is deliberately loose: it is there to stop one input eating twenty
seconds, not to make the fuzzer fast. The 1-to-10 second band, where the
interesting geometry lives, is untouched.

Per-target numbers were considered and refused under §13. Every other target's
p100 is under 160 ms, so for four of the five any value at or above one second is
invisible; four more literals with no consumer is what §13 exists to prevent.

### 135.3 The flag is only safe inside fork mode

`-timeout=10` is set in the same branch as `-fork`, and never on its own.

`ignore_timeouts` is a fork-mode flag — `FuzzerFlags.def:66` says so in its help
text — so outside fork mode a timeout kills the process with
`timeout_exitcode`. And `fuzz/corpus/where_two_surfaces_cross` is tracked, 834
files, and holds the very inputs measured above at 17 to 19 seconds. A hand-run
`just fuzz 60` carrying this flag would therefore die during its corpus replay,
**before fuzzing a single new input**, and would do it every time.

That is arm Fifteen, and it is the one arm here that exercises `fuzz-target`
through `fuzz` rather than through `fuzz-overnight`, because the hand path is the
one call site the night never takes.

### 135.4 `timeout-` stops being a finding

§134.3 put `timeout-` in the set of artifact names that fail a night, on the
grounds that a timeout then meant an input running past libFuzzer's 1200-second
default. `-timeout=10` changes what the name means, and Q8 was answered (a) on
2026-09-18 to move it out of that set when this landed.

With this bound a `timeout-` file on `where_two_surfaces_cross` is a nightly
certainty and not news: the corpus replay reaches those 17-to-19 second inputs
before any fuzzing starts, so one gets written every night by construction. A
night that failed on it would fail every night, and a night that always fails is
a night nobody reads — §91's failure reached by crying wolf rather than by
staying quiet.

So `timeout-` joins `slow-unit-` as a measurement rather than a fault. **Neither
is ignored**: both are still copied out of the worktree by §91's `copy_out` and
still counted in the summary line, which is how a new one gets noticed. What
failed a night is now exactly `crash-`, `oom-` and `leak-`.

### 135.5 The mutants

Three, on top of §134.6's six:

| mutation | arm | what it said |
| --- | --- | --- |
| `-timeout=10` dropped | One | a target runs with no per-input timeout, so one input can still take twenty seconds of its night |
| `timeout-` put back in the failing set | Fourteen | a timed-out input failed the night, so every night fails from here on |
| `-timeout=10` moved outside the `-fork` branch | Fifteen | a hand run was given a per-input timeout, which without `-fork` kills it on its own corpus replay |

The third is the one worth having. It is the mistake a reader makes on seeing a
per-input timeout — that it is a property of the target rather than of fork mode
— and the arm names the consequence rather than the difference.

Commands run to reach this: `cat` of the five `report` files under
`~/rcad-fuzz-logs/2026-09-18/timings/`; `ls` of
`fuzz/corpus/where_two_surfaces_cross/`; `mise exec -- just
check-fuzz-overnight-selftest` against the tree and against three mutations;
`mise exec -- just gate`.

**Not measured: what this is worth.** The arithmetic above says what the bound
costs the tail, not what the night gains, and the first night to run it will be
the first evidence either way. `where_two_surfaces_cross`'s run count in
`~/rcad-fuzz-logs/` is the number to read, against 259 136.

