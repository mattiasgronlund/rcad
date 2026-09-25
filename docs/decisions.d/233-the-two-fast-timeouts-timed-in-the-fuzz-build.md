## 233. The two fast timeouts, timed in the fuzz build: load is the surviving cause — 2026-09-25

*Row Q9 of the queue in §230. Measured on `39b278cc`, `main` at the time, and on
`6664fef8`, the commit the night of 2026-09-25 actually fuzzed, both with
`cargo +nightly fuzz run <target> <file>` from `fuzz/`, which is cargo-fuzz's
default build: optimised, with debug assertions and the address sanitizer.*

  233.1   What was timed, and how
  233.2   The four times
  233.3   Neither candidate holds up as written; load is what is left
  233.4   What is still not measured

### 233.1 What was timed, and how

§228.1's two fast inputs: fuse `timeout-bb8aa3402253563a9eb174d039ce15f4cc885e09`
(4 763 bytes) and cone-or-torus `timeout-694123dc56170724157e39be061b6a3d9fee7c60`
(63 bytes). Both were copied out of `/home/mattias/rcad-fuzz-logs/2026-09-25/artifacts/`
— the night's own copy of its artifacts, made by `fuzz-overnight`'s `copy_out`,
not the live `fuzz/artifacts/` in the read-only `rcad-nightly-fuzz` worktree —
and checked byte-for-byte against that copy with `md5sum` before use.

The night's commit was read off `/home/mattias/rcad-fuzz-logs/latest`, the line
`fuzz-overnight` writes when a night finishes: `6664fef8 (0 days old, 0 behind
main)`. `main` at the time of this lane was `39b278cc`, 59 commits ahead of
`6664fef8` — so the two differ, and both were timed. `0c1346f9`, the commit
§228 measured its debug-build times on, sits 21 commits below `39b278cc` and 38
above `6664fef8`; it was not separately timed here, since the question is what
the night itself ran on, not what `main` was when §228 was written.

Each input was run once with `cargo +nightly fuzz run <target> <file>` — no
`-fork`, no `-timeout`, no `-max_total_time` — which is a single replay, and
libFuzzer reports its own wall time for the one execution (`Executed ... in N
ms`), read off that line rather than off the shell's own clock. `RUSTFLAGS="-C
target-cpu=x86-64-v3"` was kept, matching `fuzz-target`'s recipe. The build for
each target was run to completion before its timed run, so no run's number
includes compile time.

### 233.2 The four times

| input | commit | wall time | load average at start |
|---|---|---|---|
| fuse `bb8aa340…` | `39b278cc` (this lane's base) | 1 629 ms | 12.74 |
| fuse `bb8aa340…` | `6664fef8` (the night's commit) | 1 664 ms | 6.79 |
| cone-or-torus `694123dc…` | `39b278cc` | 4 074 ms | 9.99 |
| cone-or-torus `694123dc…` | `6664fef8` | 3 271 ms | 5.10 |

Load was read from `/proc/loadavg` immediately before each command started;
other sessions were compiling on this machine throughout, and this lane did not
try to stop them. All four numbers exclude `cargo fuzz build`'s own compile
time; each is the bare `Executed ... in N ms` line libFuzzer prints for its one
run.

### 233.3 Neither candidate holds up as written; load is what is left

§228.4 named two candidates and a caveat. All three are now checked:

**"A commit between the tip the night fuzzed and `0c1346f9` made these inputs
faster"** does not hold. Both inputs are fast — 1.6 s and 3.3–4.1 s — already
at `6664fef8`, the commit the night itself fuzzed, run alone. There is no slow
version of either input anywhere between the night's commit and now to find a
fix for.

**"The sanitizer build is slower than §228.4 assumes"** — named there as the
way neither candidate might hold — also does not hold, and in the opposite
direction than the sentence worried about. The fuzz build is faster than the
debug build §228 measured, not slower: 1.6–1.7 s against 9 s for the fuse
input, 3.3–4.1 s against 7 s for the cone-or-torus one. §228.4's reasoning —
optimisation should outrun debug assertions and ASan's overhead — is what the
numbers show.

**"The night's load: five targets, each `-fork=4`, 20 processes on 24
threads"** is what is left standing, by elimination rather than by direct
reproduction. Neither input comes near the night's `-timeout=10` alone, at
either commit, under the load this lane's own runs happened to see (5.1 to
12.7) — nowhere near 20 fuzzing processes contending for 24 threads. This lane
did not build that contention (§133.7's `-fork` machinery is the night's, and
reproducing it was out of this row's scope); the case for it rests on having
ruled out the other two, not on having watched an input cross 10 s under load.

### 233.4 What is still not measured

- The load candidate is inferred, not demonstrated: no run here saw the input
  cross the `-timeout=10` line. Reproducing the night's own contention — five
  targets at `-fork=4` together — would either show one of these inputs
  crossing 10 s, which would close this row, or show it does not, which would
  reopen the question.
- §228.5's other item, profiling the slow refusal (`c0c1f6b0…`, `0d5a11d5…`),
  is row Q8, not this one, and was not touched here.
