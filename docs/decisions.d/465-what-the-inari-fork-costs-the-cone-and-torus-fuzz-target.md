## 465. What the `inari` fork costs the cone-and-torus fuzz target — 2026-10-24

*Mattias asked why the night of 10-24 fuzzed fewer runs than the nights before.
The drop came on 10-22, the first night that fuzzed §460's fork, and it is one
target's. Built twice on one commit, with and without the fork, the slow inputs
of that target cost twice the CPU with it. §460 took the fork for speed.*

The time goes to the fork's `a * b` on numbers near 2^-1006. Those products
take the fork's one-lane fallback, which in a benchmark with no rcad code costs
three times what 2.0.0's multiplication does (465.3). rcad's release build pays
nothing for it on a core to itself, and 21% when the core's other hardware
thread is busy (465.4). The fuzz build pays most, because it instruments that
fallback (465.5). What went to the fork's maintainer is in 465.6. The products
are that small because one step count in rcad's walk underflows, and a parabola
smaller than about 1e-161 is walked in steps 2^-1022 wide (465.10).

### 465.1 What the nights show

`where_a_cone_or_a_torus_crosses` ran 212, 235, 286, 206 and 225 million times on
the nights of 10-17 to 10-21, then 175, 165 and 168 million on 10-22 to 10-24
(`just fuzz-yield 2026-10-17 2026-10-24`). The other four targets did not drop.
The night of 10-21 fuzzed 3b0fb34e and 10-22 fuzzed 330d6728 (the nightly tree's
reflog). 10-22's `build.log` is the first that compiles `inari` from the fork.
The target's coverage went from 3789 to 3984 edges that night.

The night's own timing reports
(`~/rcad-fuzz-logs/<night>/timings/where_a_cone_or_a_torus_crosses/report`):

| night | runs | seconds in runs | mean per run | p50 | p90 |
|---|---|---|---|---|---|
| 10-21 | 223 518 701 | 101 097 | 0.452 ms | 0.007 ms | 0.623 ms |
| 10-22 | 173 668 221 | 94 079 | 0.542 ms | 0.015 ms | 0.786 ms |

These are wall times on a shared machine, and the commits differ in more than
`inari`: changes to the march and the walk (Q2581, among others) landed between
them too. So
they say when, not what.

### 465.2 One commit, built with and without the fork

Two worktrees at 330d6728. In one, `Cargo.toml`, `Cargo.lock`, `fuzz/Cargo.lock`
and `.cargo/config.toml` were checked out from 9feb4fc0^, which puts crates.io
`inari` 2.0.0 back and changes nothing else. Each was built with the command
`fuzz-overnight` uses: `CI=1 RUSTC=<tree>/fuzz/rustc-stand-in.sh RUSTFLAGS="-C
target-cpu=x86-64-v3" cargo +nightly fuzz build where_a_cone_or_a_torus_crosses`.
The build logs show `Compiling inari v2.0.0` from the fork in one and from
crates.io in the other.

Inputs were picked from `~/rcad-fuzz-corpus/where_a_cone_or_a_torus_crosses`.
Each input went through both binaries at once, pinned to cores 10 and 11 (two
physical cores of the 3900X), then again with the cores swapped. The measure is
user plus system CPU seconds from `/usr/bin/time`, so the load from other
sessions mostly drops out. RCAD_FUZZ_TIMINGS was unset.

Sixteen slow inputs, one run each, CPU seconds (fork / crates.io, first pass and
second pass):

| input | pass 1 | pass 2 |
|---|---|---|
| 0228ef21441e0c74 | 1.34 / 0.68 | 1.35 / 0.67 |
| 150ecee9bd3e8cb9 | 2.96 / 1.52 | 3.01 / 1.48 |
| 347a8978c42b8417 | 3.95 / 1.95 | 3.92 / 1.99 |
| 3f55cacf39d486de | 1.06 / 0.53 | 1.07 / 0.55 |
| 481232f8ca623cdf | 3.05 / 2.03 | 2.91 / 2.02 |
| 54fca6e9a90e3676 | 1.13 / 0.57 | 1.10 / 0.56 |
| 610079a822406451 | 3.77 / 1.90 | 3.80 / 1.87 |
| 70fbff12c2df4845 | 2.91 / 1.39 | 2.81 / 1.42 |
| 76257c9bf05fc689 | 1.10 / 0.53 | 1.02 / 0.55 |
| 8453cc06b9117f26 | 2.83 / 1.34 | 2.60 / 1.37 |
| 97eb031891d0f007 | 3.02 / 1.49 | 2.86 / 1.54 |
| a30d66fdc060be16 | 3.54 / 1.74 | 3.56 / 1.86 |
| add80c4dbccf99c8 | 1.41 / 0.65 | 1.39 / 0.71 |
| c0ec6b288ba0cf99 | 3.03 / 1.36 | 3.26 / 1.32 |
| c8d1a511cd777a0d | 1.15 / 0.58 | 1.21 / 0.54 |
| ddb9a9a0090cf04c | 2.88 / 1.45 | 3.26 / 1.41 |
| **all sixteen** | **39.13 / 19.71** | **39.13 / 19.86** |

The fork costs 1.99 and 1.97 times as much over all sixteen. Thirty of the 32
pairs fall between 1.85 and 2.31. The outliers are 481232f8 (1.50 and 1.44) and
c0ec6b28 on its second pass (2.47).

On cheap inputs the two builds are close. Twelve inputs that take 1 to 10 ms for
three runs, run 300 times each, came to 2.03 s for the fork and 1.90 s for
crates.io, with each process's start-up counted in. Eight inputs of 10 to 100 ms,
run 100 times each, came to 1.43 / 1.30 and 1.56 / 1.25. One of them, 1e428323,
is 0.28 / 0.18 on both passes.

So the cost shows in the slow inputs. The timing report gives runs by percentile,
not time by percentile, so this does not say how much of a night's CPU the slow
runs take, nor how much of 10-22's drop is the fork's.

### 465.3 Where the time goes: multiplying tiny numbers

A copy of the 330d6728 tree, built in release with the fork, counted every call
rcad made into `inari` and, inside the fork, which path each took. Over one run
of each of the 16 slow inputs (`rcad-runs/counts-slow16.txt` in the directory of
465.6):

| call | count |
|---|---|
| `Interval::try_from((lo, hi))` | 99 414 900 |
| `a * b` | 120 308 448 |
| `a + b` | 50 128 832 |
| `a - b` | 40 102 624 |
| `a.wid()` | 15 038 400 |
| `contains(0.0)`, `inf()`, `sup()`, `is_empty()` | 25 063 840, 27 148 794, 29 233 460, 10 025 824 |
| `a / b`, `-a`, `a.sqrt()` | 0 |

80 726 121 of the products reach the fork's AVX+FMA `mul_ru`, and 69 979 910 of
those (87%) have a lane that leaves it for the one-lane fallback of §460.1:
134 947 052 lanes go through `generic::mul1_ru_slow`. None of the 90 231 456
`add_ru` calls left its fast path.

The operands are tiny. Of the endpoints passed to `try_from`, `+`, `-`, `*` and
`wid`, 29.4% are exact zero, 18.8% subnormal, 27.2% normal below 2^-1000, 5.0%
between 2^-1000 and 2^-200, 7.5% in [2^-10, 1) and 12.2% in [1, 4). None is 4
or more, none is infinite, and none lies between 2^-620 and 2^-53. The nonzero
exponents are -1022 to -1005, most of them -1006 (157 million) to -1008; -668,
-667 and -622 (53 million together); and -1 and 0 (210 million). A number near
2^-1006 times a number near 1 is below the 2^-968 the fork's `mul_ru` accepts,
so the product goes the slow way (lines at a6650cbe):

1. `src/simd/x86_64/fma.rs`, `mul_ru` (line 100): `two_prod_is_exact(p)`
   (line 162) is `exponent_in(p, -968, 1023)`. It fails for a product below
   2^-968 unless a factor is zero or not finite.
2. `src/simd/x86_64/lanewise.rs`, `or_generic` (line 154): if either lane
   fails, both go to the portable backend.
3. `src/simd/generic.rs`, `lanewise2` (line 328): each lane tries
   `mul1_ru_fast` (line 550) again, and one that fails goes to the `#[cold]`
   `mul1_ru_slow` (line 692), which splits both factors into exponent and
   mantissa, multiplies the mantissas as a `u128` and rounds up by hand.

2.0.0 multiplies in hardware with MXCSR set to round upward, the same at any
exponent.

A benchmark with no rcad code in it (465.6) replays the recorded calls of four
slow inputs against both versions, pinned to one core, best of five passes.
`a * b` in ns per call, fork / 2.0.0, on the recorded operands and with every
finite nonzero exponent set to 0 (`--ordinary`):

| input | products | `mul_ru` calls with a lane off the fast path | recorded | exponents 0 |
|---|---|---|---|---|
| 0228ef21 | 4 096 758 | 2 560 395 of 2 731 130 | 33.65 / 10.92 (3.08) | 5.84 / 7.53 (0.78) |
| 150ecee9 | 9 174 774 | 5 734 155 of 6 116 474 | 32.87 / 10.94 (3.01) | 5.86 / 7.55 (0.78) |
| 347a8978 | 11 971 854 | 7 482 330 of 7 981 194 | 33.33 / 10.78 (3.09) | 5.93 / 7.58 (0.78) |
| 481232f8 | 12 507 990 | 2 605 805 of 8 859 779 | 15.53 / 9.09 (1.71) | 9.73 / 7.92 (1.23) |

With exponents 0 the fork wins, as its own benchmarks say: those build every
interval from numbers in [1, 3) and never reach this path. On the recorded
operands `+` and `-` take 0.57 to 0.61 of 2.0.0's time, `wid` 0.26 to 0.28, and
`try_from` the same. The two versions gave bit-identical results on every call.
The benchmark's fallback count, worked out from the operands, matches the
in-rcad count for 0228ef21 exactly.

### 465.4 rcad as it ships

A probe binary in each tree of 465.2 runs the fuzz target's body on a corpus file
a given number of times, built in release with `target-cpu=x86-64-v3`: no ASan,
no coverage counters. CPU seconds over the 16 slow inputs, fork / 2.0.0:

| how | CPU seconds | ratio |
|---|---|---|
| paired as in 465.2, three runs per process | 32.70 / 30.73 | 1.064 |
| one core, CPU 22 idle | 43.44 / 44.60 | 0.974 |
| one core, `sha256sum /dev/zero` on CPU 22 | 55.28 / 45.55 | 1.214 |
| one core, CPU 22 not watched | 57.85 / 46.68 | 1.239 |

"One core" ran three runs per process, fork then 2.0.0, three rounds, all on
CPU 10, while `mpstat` watched CPU 22, the other hardware thread of that core. A
spread of 30 corpus files, 8 of them slow, paired, gives 22.86 / 20.67, 1.106.

On a core to itself, 15 of the 16 inputs are between 0.98 and 1.04, and
481232f8 is 0.64. Loading the other hardware thread made the fork's runs 27%
slower and 2.0.0's 2%; every input but 481232f8 is then between 1.20 and 1.33,
and 481232f8 is 0.81. The unwatched run reads like that load arriving part-way:
its first inputs are near 1. The paired runs left CPUs 22 and 23 unwatched,
which is why their per-input ratios scatter from 0.76 to 1.36. The fuzz nights
run `nproc / 5`, 4 workers for each of five targets, 20 on 24 hardware threads,
so most of them share a core.

70fbff12, 8453cc06, c0ec6b28 and ddb9a9a0 make identical counts of every call
and every fallback. They are not different operands, as this said first: three
of the files decode to the same cone and plane, and the fourth moves the plane
in its last bits (465.10). Paired, 70fbff12 was 1.02 and
ddb9a9a0 1.18; on a core to themselves they are 1.00 and 1.01, and their
streams bench alike (`a * b` 3.05 and 3.02). The spread was the core, not the
stream.

**The benchmark overstates what the fork costs inside rcad.** With CPU 22 idle,
it replays one run of 70fbff12's calls, in their own order, in 345 ms with the
fork and 185 ms with 2.0.0. A whole run of rcad on that input takes about 350 ms
with either (3.15 s and 3.14 s for nine runs). So inside rcad the fork's
multiplications cannot cost the 160 ms more the benchmark measures.

### 465.5 The fuzz build

Paired as in 465.2, one run per process, CPU seconds over the 16 slow inputs:

| fuzz build | CPU seconds | ratio |
|---|---|---|
| as the nights build it: ASan, coverage counters, debug assertions | 46.95 / 26.53 | 1.770 |
| no ASan (`cargo fuzz build -s none`) | 40.61 / 14.98 | 2.711 |
| ASan, `inari` without coverage counters | 39.71 / 28.40 | 1.398 |
| no ASan, `inari` without coverage counters | 11.61 / 9.31 | 1.247 |
| ASan, `inari` without coverage counters, debug assertions or overflow checks | 27.70 / 20.69 | 1.339 |
| no ASan, `inari` likewise | 11.02 / 9.64 | 1.143 |

On the spread of 30, the nights' build is 1.887 (40.42 / 21.42) and the fifth
row 1.342 (20.06 / 14.95). Two runs of one build differed by up to 0.1 in ratio:
the nights' build gave 1.835 and 1.770, the no-ASan one 2.567 and 2.711.

`inari` was left uninstrumented the way `fuzz/rustc-stand-in.sh` leaves
`rcad-fuzz-timing` (§267): a copy of the script in each probe tree caught
`--crate-name inari` and appended `-Cllvm-args=-sanitizer-coverage-level=0`, and
for the last two rows `-Cdebug-assertions=off -Coverflow-checks=off`, after
cargo-fuzz's own flags. The later flag wins: `rustc -Cdebug-assertions
-Cdebug-assertions=off --print cfg -` prints no `debug_assertions`. Every other
crate kept cargo-fuzz's flags, and ASan still covers `inari`'s code.

So most of the fuzz build's gap is the instrumented fallback. The fork's ASan
build falls from 46.95 to 27.70 s, near the 26.53 s that 2.0.0 cost fully
instrumented, which is what the nights paid before 10-22. That is rcad's to
change, not the fork's.

### 465.6 What went to the fork's maintainer

`~/rcad-inari-fork-cost-2026-10-24/`, beside `~/rcad-fuzz-logs`:

- `README.md`, for someone who knows `inari` and not rcad: the operation and the
  path that cost the time, the tables of 465.3 to 465.5, how the streams differ
  from the fork's own benchmarks, the stream format, and a benchmark it suggests
  but did not run: one factor in [2^-1010, 2^-1005), the other in [1, 3).
- `streams/`: six recorded streams, zstd-compressed (91 KB to 35 MB), and four
  of them unpacked (300 MB to 990 MB, 2.8 GB in all, which can go).
- `bench/`: `inari-stream-bench`, which replays a stream against 2.0.0 and the
  fork at a6650cbe (`stats`, `time`, `time --ordinary`).
- `results/`: its prints, some with CPU 22 idle and busy beside them.
- `rcad-runs/` and `recorder/`: rcad's side, the CPU tables, the call counts,
  the scripts that built and timed the probes, and the patches that counted and
  recorded the calls in a copy of 330d6728.

### 465.7 What is not known

- **Why the benchmark overstates the cost inside rcad** (465.4). One reading,
  not tested: 2.0.0's MXCSR writes cost more among rcad's own instructions than
  in the benchmark's loop, and that hides the fork's slower multiplications. It
  would fit 481232f8, 0.64 in rcad and 1.07 in the benchmark's in-order replay.
  No profile was taken.
- **Why a busy sibling thread costs the fork 27% and 2.0.0 2%.**
- **How much of the nights' drop this is.** The fuzz-build ratios of 465.5 were
  taken with the sibling threads unwatched, and the nights run with them busy.
- **Which rcad loop makes the tiny products.** Answered in 465.10: the fuzz
  target's own loop over walked points, and not a loop with a fixed cap.
- 481232f8's `a * b` is 1.23 times 2.0.0's even with every exponent set to 0.
- Whether rcad's own tests see any of it. Daytime `just t` medians did not rise
  after 10-21 (`just row-time --since 2026-10-15 --json`), but those tests seldom
  use extreme coordinates.

### 465.8 What this leaves

- Build `inari` in the fuzz builds without coverage counters, debug assertions
  or overflow checks, through `fuzz/rustc-stand-in.sh`. Q2731, merged at
  8f5eafa8 with `check-rustc-stand-in-selftest` to hold it.
- Find which rcad loop makes the tiny products, and whether it should stop
  sooner. Q2732 found it and says it should (465.10); the fix is Q2733, and
  Q2734 replays the other slow inputs after it.
- Nothing for the fork in rcad's queue: its fix belongs in the fork's
  repository, and moving the pin waits on that.

### 465.9 Why `inari`'s own edges need not steer libFuzzer

267.3's first finding was why `rcad-fuzz-timing`'s edges were safe to drop
from coverage: that crate decided nothing the fuzz target's own behavior
turned on. `inari` looks different on its face, since the target's whole
geometric computation runs through it, but the one place its own code forks
on the 16 slow inputs — `mul_ru`'s AVX fast path against the one-lane
`mul1_ru_slow` fallback (465.3) — is a seam with no say over the answer.
465.3's stream replay ran every recorded call through both the fork and
2.0.0, which take that fork oppositely (2.0.0 has no fallback to fall into),
and "the two versions gave bit-identical results on every call". Whichever
path a product takes inside `inari`, rcad gets the same `Interval` back, and
every branch rcad's own code takes on that interval — every `Decision`,
every certified-or-not split — is instrumented regardless, since the stand-in
catches only `rcad-fuzz-timing` and `inari`. Two inputs that differ only in
which of `inari`'s internal paths they take cannot differ in which of rcad's
edges they reach, so an edge inside `inari`'s dispatch cannot mark a corpus
input "new" in a way no rcad edge already marks it.

What this does not rule out: an input that reaches a magnitude new to
`inari`'s dispatch while every rcad-level branch it passes through is one
already seen at other magnitudes, so no rcad edge marks it new either — the
case where coverage-guided search would have kept the input for `inari`'s
sake alone, not rcad's. No measurement here says how often that happens on
this target; 465.7's "whether rcad's own tests see any of it" is the nearest
existing measurement, and it is about test medians, not corpus growth, so it
does not answer this either. Unmeasured, not argued away.

### 465.10 Which loop makes the tiny products

Q2732, read-only, at 422b5468. A scratch copy of rcad counted every `Interval`
product, sampled the call stack of one tiny product in 2000, and replayed the
fuzz target's body on one corpus file at a time.

- **The products come from the fuzz target's own loop over walked points**,
  `how_every_crossing_was_walked_keeping_the_provenance` in
  `crates/rcad-geom/tests/fuzzing/mod.rs` (lines 465-478 at the base). For each
  point the walk draws, it asks both surfaces which side the point is on: the
  cone through `on_intervals` (`cone.rs:409`) and the plane (`plane.rs:622`).
  That is 24 products and 3 `wid()` calls a point. All 2630 stacks sampled on
  70fbff12 were in that loop.
- **The four inputs with identical counts are one shape, not a cap.** A cone
  of slope 1 with its apex at the origin and its axis along (0, 1, 1), cut by
  the plane z = -1.950951127282395e-303. Three of the files decode to those
  bits; ddb9a9a0's plane is at -1.9509509219834527e-303. The crossing is a
  parabola about 4e-303 across.
- **The walk is long because a step count underflows.**
  `how_many_pieces_an_endless_bend_takes` (`walk.rs:2293-2294`) takes
  `(room / stands_off_a_chord).sqrt()`. For a parabola `room` is a length and
  `stands_off_a_chord` is one over a length, so the ratio is a length squared:
  about 1e-305 over 6e301, which is 0 in f64. `widest.max(f64::MIN_POSITIVE)`
  then makes every step 2^-1022 wide. `ceil(2*linear_term/2^-1022)+1` gives
  350 723 points for the four inputs and 170 693 for 0228ef21 (its plane at
  z = -9.495e-304), and the walks printed exactly those. Each of the four makes
  1 052 175 `wid()` calls in `counts-slow16.txt` (465.3), which is 3 × 350 723
  + 6. `MOST_POINTS_ALONG_ONE_CROSSING`
  is never reached, and the march never ran on 70fbff12.
- **It should stop much sooner.** About 20 points would meet the deflection,
  by the lane's working, which it did not run. Q2733 works the count out from
  ratios of like lengths and checks the walk's two arc callers of the same
  function for the same underflow. Q2734, after it, replays the other 11 slow
  inputs: Q2732 traced only these five, and 481232f8's counts are shaped
  differently.
