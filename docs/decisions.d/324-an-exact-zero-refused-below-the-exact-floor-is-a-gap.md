## 324. An exact zero refused because its products sit near `1e-300` is a gap, closed by a second power of two — 2026-09-28

*Q315's lane, on base `ec6803e0`. From §316.6. The fix is in
`crates/rcad-predicates`; the fuse test is
`two_boxes_one_with_a_corner_moved_by_1e-300.rs`.*

### 324.1 What refused

Two `[0, 2]³` boxes built face by face at `Scale::millimetres(100.0)`, the
second with its corner `(2, 0, 0)` stored as `(2, 1e-300, 0)`. On the base,
all four certified fuses (union and intersection, both orders) refuse
`Undecidable` with one record: `orient3d` at `Level::Interval` on `(0,0,0)`,
`(2, 1e-300, 0)`, `(2, 0, 2)`, `(0.5, −0, 0.5)`, enclosure `3.3156184e-316`.
The tolerant fuses answered one box of six faces, but only by writing four
approximations into the log.

The answer exists and is zero: the first point is the origin and the fourth is
exactly a quarter of the third, so three of the four are on one line.

### 324.2 Why no exact stage ran

It was not a level cap. There is no such parameter anywhere in
`rcad-predicates`. The call comes from `classify::decide`, which calls
`orient3d_tolerantly`, which runs the whole cascade and then the retry.
`Level::Interval` is simply what `decide` reports when level 3 declines.

Each level, measured on the base with a scratch probe:

- **Level 1**: the value is exactly `0`, and the bound is `7ε` times a
  permanent of `1.5e-300`, which is subnormal. `decide` hands a subnormal bound
  on.
- **Level 2** straddles zero, `3.3e-316` wide.
- **Level 3** declines. `minor(b, d)` asks `product_is_exact(1e-300, 0.5)`,
  which is `false` because `5e-301` is below the floor of `2^-969`. But
  `two_product(1e-300, 0.5)` has a roundoff of exactly `0`, and so does
  `(1e-300, 2)`. So the floor is conservative. It is a sufficient condition for
  exactness, not a necessary one, and these products are exact anyway.
- **The retry** (`decided_after_normalising`) reads its one exponent off the
  largest *difference*. That difference is `1.5`, already in `[1, 2)`, so the
  exponent is `0` and the retry returns `None` without trying anything.

### 324.3 A gap, shown rather than argued

Every exponent `k` for which multiplying all twelve coordinates by `2^k` is
exact was tried, from `−1100` to `1100`. **659 of them decide it: every `k`
from 15 to 673, all as `Zero` at `Level::Exact`.** So the answer is reachable
in `f64`, and the refusal was a gap (§16.10).

**Why a power of two keeps the answer.** The determinant has degree three in
the coordinates. Multiplying all twelve by `s = 2^k > 0` multiplies it by
`s^3 > 0`, so a sign stays the same sign and a zero stays exactly zero. A
power of two changes only the exponent of each `f64`, never its mantissa, so
the scaled copy is exactly `s` times the input for as long as every result
stays normal. `scaled_by_a_power_of_two` checks that for every coordinate and
refuses the whole copy if one fails. So a sign proved about the copy is a sign
proved about the input. Nothing in this depends on how `k` was picked. The
choice of `k` decides only *how far* the retry reaches.

**orient2d has the same gap.** `(0,0)`, `(3, 1e-300)`, `(6, 2e-300)` are on
one line and refused the same way. Lifted by `2^500` they decide `Zero` at
`Exact`.

### 324.4 The fix: a second exponent, read off the coordinates

Level 3 expands the determinant over the raw coordinates, not over the
differences. Every product it forms grows as `k` grows, and the only thing
that stops `k` growing is the largest product running off the top. So the
exponent that takes the largest coordinate as high as the top allows lifts
every smaller product as far off the bottom as any exponent could without
risking overflow. `exponent_that_lifts_the_largest_to` computes it.
`HIGHEST_BINADE_LEVEL_THREE_TAKES` in each predicate names that binade:
`[2^338, 2^339)` for `orient3d` and `[2^509, 2^510)` for `orient2d`, derived
in their doc comments. `the_highest_binade_the_retry_lifts_into_overflows_nothing`
runs every sign pattern at the top of the binade and finds no overflow. At
one binade higher it finds some, so each constant is the highest that holds.

The first exponent, from the largest difference, is still tried first, and the
second only if the first settled nothing. So no answer the old code gave has
moved, and the change can only turn a refusal into an answer. On the Q315
input the second exponent is `337`, inside the 15-to-673 window.

**The family not taken**: a sharper `product_is_exact`. It would accept a
product whenever its roundoff is representable, for example when the two
factors' lowest set bits multiply to at least `2^-1074`. That also reaches
this input, and it would reach further in the exact arithmetic of
`rcad-geom`'s `Exact` too. But that floor sits under every exact routine in
the workspace, and refusal counts pinned against it live outside this lane's
crate (§143.9's residue among them). It is a separate change, with its own
soundness argument about expansion arithmetic on subnormal entries. It is left
open, not ruled out.

### 324.5 What moved

- **The fuse.** All four certified fuses now answer one box of six faces with
  an empty log. The tolerant ones answer the same, now also with an empty log.
  The fuse does not move on to a different refusal.
- **`orient3d_matches_robust.rs`** pinned the refusal of
  `(1.5,1.5,1.5)`, `(−1.5,−1.5,−1.5)`, `(3,3,3)`, `(−3.5e-323, 1, 0)`. That
  determinant is exactly zero, and `robust` answers `−1.5e-323`. The second
  exponent, `2^337`, lifts `−3.5e-323` to a normal number without losing a
  bit, so the answer is now `Zero`. The test is renamed
  `coordinates_down_among_the_subnormals_are_lifted_and_decided_rather_than_guessed`.
  It still fails if a sign ever comes back. **A refusal rate went down here,
  and this is the one pin in `rcad-predicates` that moved.**
- `a_determinant_whose_products_fall_off_the_bottom_of_the_range_is_not_certified`
  still refuses. Both exponents scale its `2^1000` coordinate down, and both
  push `2^-538` out of the normal range. Its doc now names both exponents.
