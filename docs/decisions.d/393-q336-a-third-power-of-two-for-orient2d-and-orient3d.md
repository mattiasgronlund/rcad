## 393. Q336: a third power of two for orient2d and orient3d, lifting instead of scaling down — 2026-10-03

*Q336, `f3e84cf9`, on `crates/rcad-predicates`. From §324 line 105.*

### 393.1 What §324 line 105 left refused

§324.5 named
`a_determinant_whose_products_fall_off_the_bottom_of_the_range_is_not_certified`
as still refusing after the second exponent: both of §324's exponents scale
its `2^1000` coordinate down, pushing `2^-538` out of the normal range.

Q336 adds a third exponent. Where the first two divide — by the largest
difference, then by the largest coordinate — the third multiplies: it reads
every product level 3 forms and lifts until the largest of them reaches a
measured ceiling, so a tiny product that the first two exponents would push
further down instead comes up. The test is renamed
`a_determinant_whose_products_fall_off_the_bottom_of_the_range_is_lifted_to_its_true_sign`
(`crates/rcad-predicates/src/orient3d.rs`): the third exponent it tries is
`Some(23)`, and `orient3d` on the unscaled points now returns `Decision::Neg`,
the sign an oracle built by rescaling the same determinant until it fits
confirms.

### 393.2 The ceilings, measured

Level 3 overflows if a product, or a sum of products, runs off the top of the
range once every coordinate is lifted into the chosen binade. Each ceiling
below is "the highest a measured sum of factor binades can reach while every
one of the sign patterns that makes every factor as large as its binade
allows still leaves level 3 within range" — found by running all of them,
not derived:

- **orient2d**: `LARGEST_SUM_OF_FACTOR_BINADES = 1020`
  (`crates/rcad-predicates/src/orient2d.rs`). Pinned by
  `the_highest_sum_of_factor_binades_overflows_nothing_in_level_three`, which
  asserts 0 of the 64 sign patterns overflow at `1020` and some do at `1021`.
- **orient3d**, a minor's two-factor products: `LARGEST_SUM_OF_TWO_FACTOR_BINADES
  = 1021` (`crates/rcad-predicates/src/orient3d.rs`). Pinned by
  `the_highest_sums_of_factor_binades_overflow_nothing_in_level_three`, which
  asserts 0 of 4096 sign patterns overflow with the two factors' binades
  summing to `1021`, and some do at `1022`.
- **orient3d**, those products times the third coordinate that scales them:
  `LARGEST_SUM_OF_THREE_FACTOR_BINADES = 1017`, pinned by the same test: 0 of
  4096 overflow with all three binades summing to `1017`, and some do at
  `1018`.

### 393.3 The 3D gap at `2^-995` that only `2^511` decides, and the limit below it

§324.7 found that a spike below a spread of about `2^994` stays refused; Q336
closes part of that for a specific shape. `a_spike_at_the_origin_under_a_unit_cube`
has two tiny corners at `(t, 3t, 0)` and `(t, t, 2t)`, and two corners of the
cube. At `t = 2^-995`, level 3 needs the tiny corners' `t * t` lifted to
`2^-969` or more — so by `2^511` or more — and `1 * 1` lifted by `2^512`
already reaches `2^1024`. So `2^511` is the only power of two that decides
these turns, and the third exponent as the binade rule reads it stops one
short, at `2^510`: two factors in binade `511` could sum to `2^1024` between
them, and the rule cannot tell that these two, each exactly `2^511`, do not
overflow. `a_spike_2_to_the_minus_995_wide_is_refused_on_turns_only_2_to_the_511_decides`
pins exactly this: the third exponent tried is `Some(510)`, `orient3d`
on the unscaled points is not certified, and of every exponent from `-1100`
to `1100`, only `511` decides it (matching an oracle's sign).

Below that spread the gap becomes a limit.
`below_2_to_the_minus_995_no_power_of_two_decides_those_turns` runs the same
two tiny corners and two cube corners at `t = 2^-1000` and at `f64::from_bits(1)`
(the smallest subnormal) and finds that no exponent in `-1100..=1100` decides
either: lifting `t * t` to level 3's floor takes `1 * 1` past the top of the
range, so the two requirements cannot be met by one scaling at once.

### 393.4 What this is, and what it leaves

This is a gap closed by reach, not a change in what is certified: nothing
decided before moves, and the renamed test's own doc comment says the first
two exponents still fail on its input for the reasons §324 gave. §324.7's
open row — the second exponent assuming the largest product is the largest
coordinate cubed, which is false for a spike — is unaffected; Q336 adds a
third exponent rather than changing the second. Whether the spread below
`2^-995` is a limit for every shape in this family, not just the two corners
`below_2_to_the_minus_995_no_power_of_two_decides_those_turns` tries, is not
traced here.
