## 460. `inari` from Mattias's fork, which rounds without switching MXCSR — 2026-10-21

*Mattias asked to move `inari` to the latest commit of his fork,
`github.com/mattiasgronlund/inari`. §2.4 pinned an exact crates.io version; this
pins an exact commit instead.*

### 460.1 What the fork changes

The fork is `inari` 2.0.0 plus one later upstream commit and fourteen of
Mattias's, from 2026-09-22 to 09-24. Its `Cargo.toml` still says 2.0.0. It is
pinned at `a6650cbef99a64424ddfa1ac9cc9090471e0fc4d`, the head of its `main` on
2026-10-21.

- **The AVX+FMA backend, the one rcad builds, no longer switches MXCSR.** It
  rounds to nearest, then works out exactly how far that is from the true
  answer, by TwoSum for addition and by the FMA for multiplication and
  division. It steps one ulp outward when the true answer lies beyond. It does
  both lanes at once. An input near overflow or underflow, where that shortcut is not safe,
  goes to a portable backend that does the same thing one lane at a time.
- **New backends:** a portable one for any architecture (and for x86-64, through
  the `force-generic` feature), one for SSE4.2 without AVX, and one for wasm32
  with `simd128`. No x86-64 backend switches the rounding mode any more.
- **`sqrt`** takes both ends in one vector square root.
- **Upstream's hand-written `Debug`** for `inari::Interval`. rcad's `Interval`
  writes its own `Debug` and never prints `inari`'s, so no output moves.

### 460.2 Why

Earlier the same day, a CPU-time sampler in another session put interval
arithmetic at 24.3% of the CPU in the fuse corpus replay and 32.6% in the
rcad-geom suite. It put the rounding-mode switches at 74–78% of the time spent
inside `inari`. That run is not repeated here.

The fork's `BENCHMARKS.md` was measured on a Ryzen 9 3900X (Zen 2), one core,
at load average 1 to 3. Against 2.0.0's MXCSR backend, its `CHANGELOG.md` sums
it up as:

- interval add and subtract take 0.47 of the time;
- multiply takes 0.49 to 0.62;
- divide takes 0.67 to 0.69;
- a 3×3 determinant takes 0.33 (192.51 ns to 64.00 ns, spread 17%).

A chain of dependent `add_ru`, `div_ru` or `mul_add_ru` operations, each waiting
on the one before, got 3 to 12% slower.

**What this does to rcad's own run times is not measured.** The machine was at
load average 30 when this landed, and a timing taken there would be noise.

### 460.3 Why the answers cannot move, and what checks that

Rounding one operation toward +∞, or toward −∞, has exactly one right answer.
So a backend that gets it right gives the same bits as one that switches MXCSR.
Every interval rcad builds stays what it was, bit for bit, unless the fork has a
bug. That is why this change brings no test that fails before it and passes
after it: the only thing a test could tell apart is time.

Two things check that the fork gets it right:

- **The fork's own unit tests**, at the pinned commit, built with rcad's
  toolchain and `-C target-cpu=x86-64-v3` so that they run the AVX+FMA backend:
  34 of 35 pass. Among the 34 are its differential tests, which compare that
  backend bit for bit with the portable one on random and hand-picked inputs.
  The one failure is `numeric::tests::rad`, which wants the radius of
  `[0, -0]` to be `+0`. **Upstream 2.0.0 fails it the same way** under the
  same toolchain and flags (31 of 32 pass), so the fork did not bring it. rcad
  never calls `rad`. It calls only construction, `inf`, `sup`, `wid`,
  `contains`, `is_empty`, `sqrt`, negation and the four arithmetic operators.
- **rcad's gate** runs every answer the suite pins through the new backend. On
  `9feb4fc0`, with every step run fresh and none served, it passed. That was
  4,435 tests in the main tier, 77 in the slow tier, 3 more and 30 doc tests.
  `deny` accepted the git source, and `check-fuzz-builds` built the fuzz crate
  against its lockfile.

### 460.4 How it is pinned

`Cargo.toml` names the fork by its https URL and the full commit sha. It uses
https and not ssh because the repository is public and CI fetches it with no
key. `deny.toml` allows a git source from that one URL. `unknown-git = "deny"`
still refuses any other.

Two lockfiles moved. One is the workspace's. The other is `fuzz/Cargo.lock`,
which the fuzz crate keeps for itself because it is not a workspace member.
Cargo fetches the fork's `ITF1788` submodule along with it. That submodule holds
the IEEE 1788 test vectors, and nothing in rcad builds from it.

To move to a newer commit of the fork, change `rev` in `Cargo.toml`. Then run
`cargo update -p inari`, and run it again with `--manifest-path fuzz/Cargo.toml`.
The first move, to 63f2237, is §467.

### 460.5 Haswell stays the floor

The fork builds on any x86-64 with SSE4.2 (Nehalem, 2008). 2.0.0 needed AVX and
FMA. Even so, `.cargo/config.toml` still sets `-C target-cpu=x86-64-v3`, for
three reasons:

- At `x86-64-v2`, `inari` would pick its SSE4.2 backend, which none of rcad's
  tests run against.
- Nothing in rcad has asked to run on a CPU older than Haswell. Lowering the
  floor is a feature with no consumer, so §13 says it is not built.
- Baseline x86-64, with no SSE4.2, still fails to build unless `force-generic`
  is on.
