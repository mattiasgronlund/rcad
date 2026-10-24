## 465. What the `inari` fork costs the cone-and-torus fuzz target — 2026-10-24

*Mattias asked why the night of 10-24 fuzzed fewer runs than the nights before.
The drop came on 10-22, the first night that fuzzed §460's fork, and it is one
target's. Built twice on one commit, with and without the fork, the slow inputs
of that target cost twice the CPU with it. §460 took the fork for speed.*

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

### 465.3 What is not known

- Whether the gap is the fork's own, or comes from the fuzz build. That build
  has ASan and coverage counters on every branch, and the kernel as shipped has
  neither. 9feb4fc0's message quotes the fork's own benchmarks: a 3x3 interval
  determinant at 0.33 of 2.0.0's time, on a 3900X.
- Which `inari` operations the slow inputs call, and how often. Nor how many of
  those take the fork's one-lane fallback, which §460.1 says it uses for operands
  near overflow or underflow; fuzz inputs are full of those.
- Whether rcad's own tests see any of it. Daytime `just t` medians did not rise
  after 10-21 (`just row-time --since 2026-10-15 --json`), but those tests seldom
  use extreme coordinates.

The agent that maintains the fork can only fix what its benchmarks show. So the
answer has to arrive in a form it can run without rcad: the operations and
operands the slow inputs make, and a benchmark that replays them against both
versions.
