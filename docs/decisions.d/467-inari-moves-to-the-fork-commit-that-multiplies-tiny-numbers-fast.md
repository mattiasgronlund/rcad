## 467. `inari` moves to the fork's 63f2237, which multiplies tiny numbers on the fast path — 2026-10-25

*Mattias asked to move `inari` to the head of his fork's `main`, after the fork's
maintainer fixed the slow multiplication §465 found. This is the first move of
the pin, done the way 460.4 says.*

### 467.1 What changed in the fork

The pin moves from `a6650cbe` to `63f2237ef642f9d7719873fa19105fd695270d74`, the
head of the fork's `main` on 2026-10-25, 21 commits later. Two of them change
code that runs, and only one of those changes code rcad calls:

- **f2dec45: `a * b` with a product below 2^-968 stays on the AVX+FMA path.**
  Before, a pair with one such lane went to the portable backend, which rounds
  the exact product in 128-bit integers, one lane at a time (465.3). Now one
  factor is scaled by 2^128, the scaled product is rounded upward on the fast
  path, and the result is scaled back. Scaling by a power of two is exact there,
  and the comment on `mul_ru_tiny` in `src/simd/x86_64/fma.rs` argues why the
  answer is the same. Only a product that overflows still goes to the portable
  backend.
- **19d8446: `rad` of [0, 0] returns +0, not -0.** rcad never calls `rad` (460.3).

The other 19 change no code that runs:

- Rocq proofs that the portable backend's `add_ru`, `sub_ru`, `mul_ru`, `div_ru`,
  `sqrt_rd_ru`, `mul_add_ru`, `floor`, `ceil`, `round`, `round_ties_even` and
  `trunc` round correctly, comments in `src/` that point at them, and CI that
  checks them. **The proofs are of the portable backend only**, as the first
  line of the fork's `rocq/README.md` says. The AVX+FMA backend that rcad builds,
  `mul_ru_tiny` included, is not proved. It is checked against the portable one,
  bit for bit, by the differential test (467.2).
- A corrected comment on `round_up_exact`, lifetimes written out in
  `parse.rs` for Clippy, a benchmark of tiny products, and coverage and MSRV CI.

`git log a6650cbe..63f2237 -- src/simd/x86_64/ src/numeric.rs` names f2dec45
and 19d8446 and nothing else, and the rest of the diff under `src/` is comments
and the three lifetimes.

### 467.2 Why no answer can move, and what checked that

Rounding one operation toward +∞ has one right answer (460.3), so a correct
backend changes no bit of any interval rcad builds. That is why this change,
like §460, brings no test that fails before it and passes after it. Four things
check that the fork gets it right:

- **The fork's unit tests**, in a clone at 63f2237, built as rcad builds it:
  toolchain 1.96.0, `-C target-cpu=x86-64-v3`, no default features. 35 of 35
  pass. At a6650cbe it was 34 of 35, and the one failure, `numeric::tests::rad`,
  passes since 19d8446. With those flags the tests' `super::mul_ru` is
  `fma::mul_ru`, because `src/simd/x86_64.rs` picks `fma` when AVX and FMA are
  both on, so they run the changed code.

      RUSTC_WRAPPER= RUSTFLAGS="-C target-cpu=x86-64-v3" cargo +1.96.0 test --lib --no-default-features

- **The differential test, 20 times deeper than a release build runs it.** It
  compares the AVX+FMA backend with the portable one bit for bit, on hand-picked
  inputs and on random ones. f2dec45 added exponent -1075 to the hand-picked
  ones. With `INARI_DIFF_N=20000000` in release, `mul_ru/fma` and
  `mul_ru/dekker` ran 40,133,128 cases each, and every operation the test covers
  had 0 mismatches. It took 54 s.

      RUSTC_WRAPPER= RUSTFLAGS="-C target-cpu=x86-64-v3" INARI_DIFF_N=20000000 cargo +1.96.0 test --release --lib --no-default-features differential -- --nocapture

- **rcad's own recorded operations.** 465.6's replay bench, copied out of
  `~/rcad-inari-fork-cost-2026-10-24/bench` with its `rev` changed to 63f2237,
  replayed the four unpacked streams: 104,839,968 calls of `try_from`, `+`, `-`,
  `*` and `wid`, every result bit-identical to 2.0.0's.
  `inari-stream-bench time <stream> 1` counts the results that differ in its
  last column. The fork's maintainer reports the same in its `BENCHMARKS.md`.
- **rcad's gate** at `02722335`, which runs every answer the suite pins through
  the new code. It passed with every step run, since a sha starting with `0`
  serves none: 4,554 tests in the main tier, 77 in the slow tier, 3 more and 30
  doc tests. `deny` accepted the git source, and `check-fuzz-builds` built the
  fuzz crate against its lockfile.

### 467.3 What it costs, and what was not measured here

The fork's maintainer measured it, in the fork's `BENCHMARKS.md` at 63f2237. In
465.6's replay bench, `a * b` with the fork took 3.09, 3.00, 3.07 and 1.72 times
2.0.0's time on the four streams before the fix, and 0.99, 1.05, 1.02 and 1.44
after. In the fork's own `mul_tiny` benchmark, where every product lies between
2^-1010 and 2^-1006, it went from 4.7 times 2.0.0's time to 0.82. These numbers
were not taken again here. The replay in 467.2 ran beside the gate, so its times
are not a measurement.

Not measured:

- **The fuzz build, paired as in 465.2.** The nights are the cheaper measure:
  set the first night that fuzzes 63f2237 against 465.1's table. Q2733
  (`4a918d03`) already cut how many tiny products rcad's walk makes (465.10), so
  a night after both shows the two together, not this change alone.
- **rcad's release build.** 465.4 found that it paid nothing for the slow path on
  a core to itself, and 21% with the core's other hardware thread busy.
- **481232f8.** Its `a * b` is still 1.44 times 2.0.0's in the replay, and 1.23
  of that is there with every exponent set to 0 (`--ordinary`), so it is not
  about tiny products. 465.7 already lists this.
- **Why 2.0.0 itself measures 12.2–12.6 ns per `a * b` in the maintainer's rebuilt
  bench and 10.9 in 465's.** The maintainer could not explain it. Against
  10.9 ns, the fork's 12.5–12.7 ns would be 1.14–1.17 times 2.0.0's.
