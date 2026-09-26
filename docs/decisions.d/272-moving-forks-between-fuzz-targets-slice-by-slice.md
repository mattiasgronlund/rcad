## 272. TITLE — 2026-09-26

*Queue row Q148, which §265.7 queued: split the night's cores between the
targets slice by slice instead of evenly, and build it only if it beats the
even split on runs and coverage. This lane measured it on one commit, both ways
at once, for eight hours of daytime. It changed no code. Nothing in the night's
corpus or the nightly tree was written, and every fuzz process ran between
12:23Z and 20:31Z, outside 21:50–06:00Z.*

  272.1   What ran
  272.2   What a restart costs
  272.3   Runs and coverage, per target and per arm
  272.4   What the rule did with its forks
  272.5   Verdict
  272.6   What this leaves

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
