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

### 324.6 Eight pins outside `rcad-predicates` that this falsifies

`just t --workspace --no-fail-fast` on this lane's tip: 2590 tests run, 2582
passed, 8 failed. None of the eight is in this lane's write set, so none is
edited here. Each one pinned a refusal that §324 shows was a gap.

**Seven use one shape, the square with a spike at `1e-150`.** That shape is
`a_square_with_a_spike_f64_cannot_place` in `when_it_cannot_tell.rs`, and the
same seven corners appear in `what_the_facade_calls_a_failure.rs` and in
`whether_a_tolerant_policy_buys_anything_a_certified_one_does_not.rs`. Its doc
says level 3 declines because `1e-150 * 1e-150` is below the exact floor. True,
but the largest coordinate is `1`. The second exponent multiplies all of them
by `2^509`, the spike comes up to about `2^11`, and level 3 runs. Measured on
this tip: the certified `ProfileBuilder::build` returns the profile with
`approximation: None` and an empty log, and so does the tolerant one. Extruded,
it is refused by `Model::validate` with `DynamicRangeTooWide { ratio:
3.162277660168379e149 }`, which is §3.5's check. That is a different refusal,
and a deliberate one. The failing tests:

- `rcad::what_the_facade_calls_a_failure::a_tolerant_build_hands_back_both_the_shape_and_what_it_cost`
- `rcad-step::whether_a_tolerant_policy_buys_anything_a_certified_one_does_not::a_shape_whose_corners_f64_cannot_place`
- `rcad-topo::when_it_cannot_tell`:
  `a_corner_f64_cannot_place_stops_a_certified_build`,
  `a_tolerant_policy_with_too_small_a_budget_still_refuses`,
  `a_tolerant_policy_says_in_the_log_what_it_had_to_approximate`,
  `extruding_an_approximated_profile_twice_does_not_inflate_the_slop`,
  `slop_a_tolerant_profile_carried_lands_on_the_solid_built_from_it`

These tests need a spike that is a real limit. Level 3 needs `tiny²` above
the floor of `2^-969` and `far²` below the top, about `2^1020`, *after the
same scaling*. So the spread `far / tiny` has to exceed about `2^994`, a
factor of about `1e299`. The spike at `1e-150` under a unit square is only
`2^498`. A shape that is past `2^994` is also near the edge of `f64` itself,
so it has to be measured before it is written down (§16.10), not taken from
this paragraph. `orient2d.rs`'s `no_power_of_two_at_all_rescues_that_spread`
shows how to measure it: try every exponent.

**The eighth is `rcad-topo::fusing_solids::a_fuse_that_needs_a_decision_nothing_can_prove_refuses`**:
two towers `1e308` tall. The certified fuse still refuses `Undecidable`. But
its `orient3d`, whose products used to overflow, is now decided: the second
exponent takes `1e308` down to `2^338`, where the first exponent had pushed
the `0.5` corners out of range. The record that is left is a different
predicate, `"a point inside a piece of a face"`, at `Level::Interval` with an
infinite enclosure, on `(0.5, −0, 1e308)`, `(1, −0, 1e308)`,
`(1, 0.5, 1e308)`, `(0.5, 0.5, 1e308)`. That predicate is in `rcad-topo`'s
source. This lane did not trace whether it is a limit or a gap.
