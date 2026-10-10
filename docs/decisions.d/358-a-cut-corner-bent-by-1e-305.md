## 358. A cut corner bent by `1e-305` was a gap: orient2d's exact level now keeps a product that falls off the bottom — 2026-09-29

*Q337's second lane, `lane-q337-keep-the-product-that-fell-below`, on
`d0d669f0`. From §324.7. The fix is in `crates/rcad-predicates/src/orient2d.rs`.*

### 358.1 What refused, and why it was a gap

§324.7 pinned a square with one corner cut off, the cut running from `(1, t)`
through `(0.5, 0.5)` to `(t, 1)` with `t = 1e-305`, as a turn nothing in `f64`
can see. Seven tests in three crates stood on it.

The turn at the middle corner has determinant `-t (1 - t)`, about `-1e-305`.
That is an ordinary normal `f64`, and it is negative, so the turn is clockwise.
On the base, `orient2d` refused it `Undecidable` at `Level::Interval`, enclosure
`1.6653345369377348e-16`, in both orders the builder asks. Level 3 did not run
because one of its six products, `t * t = 1e-610`, fails `product_is_exact`, and
`cross` then gave up on the whole determinant. The other five products are
exact, and the one that fell off is worth `1e-610`, against an answer of
`1e-305`.

§324.7 tried every exponent and found none that decides it. That is true of the
cascade that gives up at the first product below the floor, and it said so
only of that cascade. It said nothing about keeping that product. So the answer
exists and is a normal number, and the arithmetic could reach it. That makes it
a gap (§16.10).

### 358.2 The fix: keep the product and draw a box round what it lost

`cross` now keeps a product that is finite but fails `product_is_exact`, and
counts it. `exact_determinant` hands back the held sum `H` and the count `n`.
A product that overflows still refuses the whole level.

**Why the box is `n * 2^-1074` wide.** `two_product`'s high half is the product
rounded. Its low half comes from a fused multiply-add, which rounds only when
that low half lands among the subnormals, onto a grid `2^-1074` apart. So the
pair misses the true product by at most `2^-1075`. A product flushed to zero is
covered too, because its low half is then the product rounded onto that same
grid. `expansion_sum` is exact, subnormal entries included. So `H` misses the
true determinant by less than `n * 2^-1074`. `rcad-geom`'s
`Exact::times_holding_on` rests on the same bound, and §322.3 checks it there.

`sign_clear_of_what_fell_off` forms `H - n * 2^-1074` and `H + n * 2^-1074`
exactly. When both are positive the answer is `Pos`, when both are negative it
is `Neg`, and otherwise level 3 declines. `Zero` still needs `n = 0`, because a
box of any width round zero holds both signs. A sign decided this way is still
reported at `Level::Exact`. It is certified in the same sense as any other.

**Demonstrated, not only argued.**
`the_box_round_what_level_three_held_holds_the_true_determinant` builds 20 000
triangles with coordinates between `2^-612` and `2^-400`, half of them with
small whole-number mantissas so that some determinants cancel down to almost
nothing. For each triangle, the truth is the same triangle lifted by `2^600`,
where nothing falls off. The test asserts that the lifted box holds the truth,
and that every sign decided at the bottom is the truth's. It also asserts that
more than 10 000 of the triangles dropped a product and that more than 5 000 of
those were decided anyway. `a_cut_corner_bent_by_1e_305_turns_clockwise` pins
the turn itself as `(Neg, Exact)`.

### 358.3 What moved

This lane put a temporary probe in `orient2d_reporting_level`. It computed every
answer twice, once with a dropped product refusing as before, and logged the
call whenever the two differed. It then ran `mise exec -- just t --workspace
--no-fail-fast`. The probe logged 27 calls, in 11 tests. They were three
distinct triples, and all three moved from `Undecidable` at `Interval` to `Neg`
at `Exact`:

- the cut corner's middle turn, in two orders: 25 calls across the eight tests
  on the cut corner, plus `a_cut_corner_bent_by_1e_305_turns_clockwise`;
- the spread `(2^-600, 2^-600)`, `(2^-600, 2^-600 + 2^-650)`,
  `(2^1000, 2^1000)`: 2 calls.

Nothing else in the workspace moved. No fuse, mesh or build answer changed, and
no level changed on an answer that was already decided.

**The spread was a gap too.** `orient2d.rs` kept it as the refusal that had to
survive, with the argument that no power of two lifts `2^-600 * 2^-600` above
the floor without taking `2^1000 * 2^1000` off the top. But level 3 never
multiplies `2^1000` by itself here. Its determinant is `2^-1250 - 2^350`. The
two tiny products fall off, the `2^400` products cancel exactly, and `-2^350`
settles the sign. It is now
`a_spread_whose_far_products_cancel_is_decided_by_what_level_three_holds`. The
same triple was pinned as undecidable in
`crates/rcad/tests/asking_a_predicate_from_the_front_door.rs`
(`a_turn_spread_wider_than_f64_holds_at_once_names_itself_in_the_log`). That
file was outside this lane's write set, so the merge swapped that pin to the
turn in 358.4.

**The spike no longer needs the third exponent.** §324.7's spike at the origin
had eight turns that only the third exponent settled. Level 3 now settles them
at the input's own scale or at the second exponent, down to a spike the
smallest subnormal wide, because their `4t` terms outweigh the `t * t` that
falls off. The third exponent is still needed where the `t * 1` terms cancel
and only `t * t` is left. `(1, 0)`, `(0, t)`, `(t, t)` at the smallest
subnormal is one such turn, and
`a_turn_left_with_nothing_but_tiny_times_tiny_still_needs_the_third_exponent`
pins it.

### 358.4 What is still refused, and the family that makes it a limit

The seven pins now use the cut `(1, -t)`, `(0.5, 0.5)`, `(t, 1)` with
`t = 1e-318`, a subnormal (`202402 * 2^-1074`). Its middle turn is exactly
`-t^2`, so it is still clockwise. `orient2d.rs` keeps the same turn as its own
limit test.

**Powers of two, all of them run.** Level 3's sum holds `1 * 1` and `t * t`,
and everything except `t * t` cancels. Multiplying every coordinate by `2^k`
multiplies every product by `2^2k`. `1 * 1` runs off the top past `k = 511`.
At `k = 511` the whole answer is `2^1022 * t^2`, about `4.5e-329`, which is
below `2^-1075`, the least a product falling off may lose. So at every scale
the answer sits inside the box, and level 3 declines.
`no_power_of_two_lets_f64_see_which_way_the_cut_bends` (rcad-topo) and
`no_power_of_two_at_all_rescues_that_turn` (orient2d.rs) try every exponent
from `-1100` to `1100`. They count exactness by moving back and comparing,
because `t` is a subnormal and the old helper, which asked for a normal result,
left out the input itself. Each turn scales exactly at 1025 exponents, from
`-1` to `1023`, and none of them is decided. The control
`a_cut_corner_bent_by_1e_305_or_1e_290_is_decided` builds the same cut, certified,
at both bends. At `1e-305` the turn is `-1e-610`, which `2^1018` lifts to about
`-3e-304`.

The pins still assert what they asserted before, and the slop bound
`< 16 * EPSILON / 100` still holds on the new shape.

**Two more families, run too** (Q534; this paragraph argued them and ran
neither until then):

- **A power of two per axis.** Every product level 3 forms is one x times one
  y. Scaling x by `2^i` and y by `2^j` moves every product, and so the answer
  and `1 * 1` together, by the same `2^(i+j)`. The only thing this adds is an
  odd total. At `2^1023`, the most `1 * 1` allows, the answer is about
  `9e-329`, still below `2^-1075`. `no_power_of_two_per_axis_rescues_that_turn`
  (orient2d.rs) tries every `i` and `j` from `-1100` to `1100`. Each axis holds
  the same three sizes, `1`, `0.5` and `t`, so each moves exactly at the same
  1025 exponents as above, and neither limits the other: it asserts
  `1025 * 1025` exact pairs, 1 050 625, and that none of them is decided. It
  takes about 2 s.
- **An exact translation.** Moving every point by `(dx, dy)` keeps the
  determinant. But each axis holds both `1` and `t`, and `1 + dx` and `t + dx`
  are both exact only for `dx = 0`: `1 + dx` needs `dx` to have no bit below
  `2^-53`, and then `t + dx` needs its 53 bits to reach from `2^-1073` up to
  about `dx`, unless the sum cancels. `only_moving_that_turn_by_nothing_keeps_it_exact`
  (orient2d.rs) cannot walk every `f64`, so it tries the moves that stand for
  them: every power of two and its negative, which puts the lowest bit of `dx`
  at every place an `f64` has one, the ulps of `1` and of `t` among them; and
  `±1`, `±0.5` and `±t` with one ulp either side, the moves where a sum
  cancels. It asserts that `0` is the only exact move on each axis, so one
  pair, and that it is not decided. It asks exactness of `two_sum`'s lost part
  and not by moving back: `1 + 2^-1074` rounds to `1`, and taking `2^-1074`
  away gives `1` again, so moving back would have admitted a move that shifts
  `t` and leaves `1` where it was.

**What would reach it** is a wider exponent: big integers, rationals, or a
float that carries its own exponent. The answer is about `2^-2113` in a sum
whose largest term is `1`. That ratio is wider than the whole `f64` exponent
range. It is §3.5's dynamic-range territory, and it is a limit of `f64`, not
of this arithmetic.

### 358.5 orient3d has the same gap, not fixed here

`orient3d`'s level 3 refuses the whole determinant when any product in a minor,
or any product of a minor's entry with a coordinate, fails `product_is_exact`.
This was measured on this lane's tip with a temporary test, not committed. The
cut corner lifted into 3D, `(1, 1e-305, 0)`, `(0.5, 0.5, 0)`, `(1e-305, 1, 0)`,
`(0, 0, 1)`, comes back `Undecidable` at `Level::Interval`, with enclosure
`2.7755575615628914e-16`. Its determinant is `t - t^2`, which is positive.

The fix there is not a copy of this one. `orient3d` multiplies each held minor
by a coordinate, so what the minor lost is multiplied by that coordinate too.
The products of that scaling can also fall off. That is the shape of
`rcad-geom`'s `Nearly`, a box carried through a further product. It is left as
a row.
