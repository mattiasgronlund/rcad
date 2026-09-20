## 120. `Axis::new`'s normalize is one ulp out, and that is a gap, booked 2026-09-17

*Carried as a line in four handoffs, measured during §119, and given a number
here so it stops being carried. Mattias chose booking over fixing it inside §119
on 2026-09-17, off the table in §120.3.*

### 120.1 What was measured

A measurement-only lane, against an **exact** reference — integer triples from
`as_integer_ratio`, rounded by comparing exact rationals against exact midpoints,
with no float arithmetic deciding anything — over 200 037 directions and 600 111
components:

- today's `scaled_to_length_one_from_anywhere` is correctly rounded on **90.07%**
  of components, and is out by up to **3 ulp** on the rest;
- the divide-by-largest-component is **not** what costs the bit at 45°: `(1,1,0)`
  has `widest == 1.0`, so that divide is the identity there. `sqrt`-then-divide
  costs it;
- the divide is **not** redundant either. Removing it silently flushes **693 of
  600 111 components to zero**, because `power_of_two_that_keeps_the_squares_in
  _range` keeps the *squares* in range, which is not the job of keeping the
  *ratios* in range. Worked case: `(2⁻⁴⁸³, 2⁵¹⁶, 0)`, whose true unit `x` is an
  ordinary `1.87e-301`;
- **canonical and correctly rounded are not in tension.** A correctly rounded
  normalize is a function of the exact ray, so two exactly proportional spellings
  round to the same double *by construction* — 100% on every same-ray bucket,
  against today's 100%. §22.6's property is kept, not traded;
- an exact power-of-two rescale plus a double-double length is correctly rounded
  on **every component above `2⁻¹⁰¹²`, zero exceptions**, and off by at most 1 ulp
  below it (1 422 components, all subnormal-adjacent). It loses no components. It
  costs ~42 ns against ~7 ns, and `Axis::new` is in no inner loop.

**If this bound is ever quoted, quote it as "correctly rounded above `2⁻¹⁰¹²`"**,
not as "correctly rounded" flat. §15.6.

### 120.2 So it is a gap, and §16.10 says fix a gap

The answer exists and this arithmetic reaches it. That makes it a **gap**, not a
limit, and the standing rule is defer a limit and fix a gap.

The hypothesis the lane was sent with was wrong on both counts — it blamed the
divide, and it expected canonicality and correct rounding to be in tension. The
lane said so in its first line, which is why the question that came out of it was
the right one. **Ask a measurement lane to destroy your hypothesis, not to
confirm it.**

### 120.3 Why it is booked rather than fixed in §119

Sequencing, and nothing else. Correcting the normalize changes the stored bits of
**every plane and axis this kernel holds** — the kicad and nist counts, the fuzz
corpus, and the tests that pin today's bits. §119 had a blast radius of its own,
and two in one merge is how a wrong literal gets in.

It is a `§` rather than a handoff line because a handoff line is what `C1` was:
booked, fixed, and still listed as open in four documents afterwards. A section
number is the cure for that.

### 120.4 The claim this replaces

`fuses_that_must_agree.rs` said, beside the pinned literal: *"Canonical is not
the same as closest: normalizing divides by a square root that has already been
rounded, and no ordering of that division recovers the last bit."* The first
clause is **false** — they coincide, as §120.1 shows. The second is true and
beside the point, and it is exactly §16.10's shape: "no ordering helps" was a
claim about orderings and silent about **carrying more bits**, which is the
family that does reach it. The comment now says so, and names the literal that
the fix will move by one ulp.

### 120.5 What the fix owes when it runs

- The rescale-plus-double-double normalize, with the bound of §120.1 published
  where §15.6 wants it and a test that fails without it.
- The bit-pinning sites updated in one commit with the change, not after it:
  `the_same_wall_built_from_a_longer_edge_is_the_same_plane` is one, and its
  assertion against `FRAC_1_SQRT_2.to_bits()` is the tell.
- Re-measured corpus counts, because the stored bits of every plane move. A count
  that does not move is a result; a count nobody re-ran is not.
- §22.6's same-ray property re-checked rather than assumed, even though §120.1
  says it is kept by construction.

### 120.6 What ran, 2026-09-18

`Vector::scaled_to_length_one_from_anywhere` is an exact power-of-two rescale
that puts the largest component in `[1, 2)`, a sum of squares and a square root
carried as two `f64` that add to about a hundred bits, and one division per
component that rounds once instead of twice. `AHundredBits` and the four little
functions under it are all of the double-double arithmetic there is; they live in
`point.rs` under their one caller.

The rescale does the job §120.1 said the old divide was doing and does it
exactly: a power of two moves no mantissa bit, so the answer is a function of the
exact ray rather than of a rounded one. **The `693` components the naive removal
flushed to zero are not lost** — the worked case `(2⁻⁴⁸³, 2⁵¹⁶, 0)` comes back
with an `x` of `1.87e-301`, because the scale is chosen from the largest
*component* and not from the largest square.

`a_unit_component_is_the_nearest_f64_to_the_exact_ratio` is the test §15.6 asks
for, and it is a measurement rather than a memory:

| | as it ran | after §121's widening |
| --- | --- | --- |
| components checked against an exact reference | 14 880 | 16 680 |
| not the nearest `f64` | **0** | **0** |
| not the nearest `f64`, dividing by the largest component and normalizing | 2 753, **18.50%** | 2 759, **16.54%** |
| comparisons too wide for the exact arithmetic to carry | 213 | 213 |
| the smallest component the sweep checked | `2⁻⁷⁹¹` | `2⁻⁹⁰⁰` |

The right-hand column is the same test after §121 widened its inputs; the rate
fell because the 1 800 components it added are ones the old arithmetic gets
right, not because the defect moved. Both columns are the same tree's output for
the same code under test.

The reference decides nothing in `f64`. "`answer` is at least as close to `c/√Q`
as `neighbour` is" becomes `2c` against `(answer + neighbour)·√Q`, and squaring
both sides turns the root into `Q` and leaves an expansion whose sign
`Exact::sign` names. Both scalings it needs are powers of two, so the sign it
names is the sign of the unscaled question.

**The old arithmetic is still in the test**, three lines of it, which is what
makes 18.50% a number this tree produces rather than one it remembers. It is
higher than §120.1's 9.93% because this sweep draws exponents across eight
hundred binades and §120.1 drew directions; both are measurements of the same
defect over different populations, and neither corrects the other.

**This paragraph was wrong three ways when it was written, and §121 re-measured
every part of it.** It said the test reached `2⁻⁹¹⁵`; it reached `2⁻⁷⁹¹`, which
the test now prints. It called the shortfall to `2⁻¹⁰¹²` "twenty-odd binades",
which was `1012 − 995` — the published bound minus `WIDEST_SQUARABLE_SPREAD`
treated as a floor on a magnitude, when what that constant bounds is the *spread*
inside one expansion. And it blamed `WIDEST_SQUARABLE_SPREAD` for the wall, which
it cannot be: that constant is `#[cfg(test)]`, lives in `exact.rs`, and is read by
two tests inside that same file and by nothing else in the tree.

What it is instead, measured: the exact reference is the test's own `ExactFrame`,
which lifts a triple's smallest component to `2⁻⁴⁰⁰` and gives up on a lift past
500. So it carries a triple **900 binades wide and no wider**, and the test now
asserts exactly that by handing it `(1, 2⁻⁹⁰⁰, 0)` and `(1, 2⁻⁹⁰¹, 0)` and
checking that it takes the first and refuses the second. The largest component of
a unit vector is never below about `1/√3`, so 900 binades of spread is also the
floor on any component this reference can check.

After the widening the test reaches that floor: hand-written `(1, 2⁻ᵉ, 0)`
directions claim every binade down to `2⁻⁹⁰⁰`, where before the drawn directions
alone stopped 109 binades short, their exponents coming out of a range 800 wide.
**So 112 binades of the published bound — `2⁻⁹⁰⁰` down to `2⁻¹⁰¹²` — still rest on
§120.1's integer-triple measurement**, which used exact rationals rather than this
crate's expansions and so has no such wall.

The one remaining family for those 112 binades is retuning `ExactFrame`'s two
constants, and it runs out short of the bound. The lift target and the lift cap
are not free of each other: the squared length's lowest entry sits near
`2^(2·smallest − 53)` and the comparison multiplies it by a square whose lowest
entry is near `2⁻¹⁰⁴`, and that product has to stay above the `2⁻⁹⁶⁹`
`product_is_exact` wants — which is what fixes the target at `2⁻⁴⁰⁰` and buys the
900. Moving the target down buys spread at the top and loses it at the bottom one
for one. Past that the comparison has to stop being built out of `Exact` at all —
an exact-rational reference like §120.1's, which is a different piece of
machinery and not a retune. The bottom of the bound is pinned another way instead:
`a_tiny_third_component_comes_back_untouched_into_the_subnormals` sweeps
`(1, 0, 2⁻ᵉ)` from `e = 600` to `e = 1074` and asserts the unit is `(1.0, 0.0,
2⁻ᵉ)` itself, needing no expansion arithmetic at all. It checks the first 301 of
those against `ExactFrame` where the reference still carries, which is the same
900-binade wall arriving from the other side. **It does not fail without §120** —
the old arithmetic hands back `(1, 0, t)` there too, and the test asserts that
rather than leaving it to be assumed — so it is the half of §15.6 that says the
bound has a test, and the sweep above is the half that says the bound belongs to
the new arithmetic.

### 120.7 What moved, and the three witnesses that dissolved

Fourteen tests. Ten are counts, and they are re-measured in the same commit:

| | before | after |
| --- | --- | --- |
| turns of two placed solids that keep the contact / lose it | 10 446 / 11 100 | 10 558 / 10 988 |
| turned box pairs leaving an edge with one face along it (§39.4) | 107 | 104 |
| slanted-line sweep, three candidates | 9 028, 3 271, 7 509 | 8 983, 3 747, 7 615 |
| pipes the rounded moment wrongly folds, and the widest | 113, 3.94e-13 | 114, 2.27e-13 |
| rebuilding an axis from its own direction, of 100 000 | over 1 000 | 992 |
| the two cap measurements at an ordinary chamfer | over 2× | 1.32× |
| a turned bar's mesh, against the untouched one | 566 / 566 | 582 / 566 |

Two M0 measurement pins moved a rounding as well — the turned cube and the turned
plate, both turned about `(1, 1, 1)`, whose stored direction moved.

**Three tests were built on witnesses the change dissolved, and each needed a new
one rather than a new number.** This is the part a count cannot carry:

- `a_conic_the_intersector_names_is_refused_here_because_the_two_ask_it_differently`
  needed a pair whose `f64` dot cancels where the exact one does not. `(1, 0, 3)`
  against `(−12, −12, 4)` stopped cancelling — its `f64` dot is `−2.8e-17` now. A
  sweep of small whole-number pipes against small whole-number planes found
  dozens that still do; `(2, 0, 3)` against `(−12, −12, 8)` is the one pinned.
  **§39's gap did not move with its witness.**
- `a_point_a_radius_out_from_a_pipes_axis_is_measured_as_being_on_the_pipe` and
  `the_two_shapes_store_their_directions_one_above_and_one_below_unit_length`
  both need one direction storing above unit length and one below. `(1, 1, 0)`
  crossed from below to above and `(1, 6, 0)` landed exactly on one, which is no
  witness at all; `(1, 2, 0)` is below and `(1, 1, 0)` is above now.
- `a_ball_that_just_fits_a_pipe_fits_it_down_any_axis` needs a direction that does
  *not* store as a unit triple. `(1, 1, 1)` now does, so `(1, 2, 1)` stands in its
  place.

`the_two_finder_relay_pipes_written_with_one_radius_are_one_width` is the fourth,
and it is the happy one: it pinned the normalize landing **one ulp below**
`FRAC_1_SQRT_2` and now pins it landing on it. Its second claim — that a better
normalize would move that gap rather than close it, because neither candidate
doubles its square to one — was written as a prediction and held.

§120.5 named `the_same_wall_built_from_a_longer_edge_is_the_same_plane` as a
pinning site. **It did not move**: `Plane` does not go through this function.

### 120.8 The corpus did not move, and §120.5's standing rates were stale

`mise exec -- just t how_often_a_real_file_imports --no-capture`:

```
=== nist: 1 / 44 imported = 2.3%
=== kicad: 114 / 323 imported = 35.3%
```

`mise exec -- just t every_file_that_stops_at_the_tangency_wall_names_the_edge_it_was_following --no-capture`:

```
files stopping at the tangency wall: 24
```

**Unmoved, and not merely unmoved by rate.** `assert_pinned_kicad_imports`
compares the importing files against `KICAD_FILES_THAT_IMPORT` as a *set* and
fails both ways (§44.10), and it passed — so the same 114 files import, not
merely as many. A change that moves the stored bits of every axis in the kernel
moved no real file across the line in either direction.

§120.5 was written expecting `110 / 323 = 34.1%`. That figure is this file's own
at §119 and older than the tree: the pinned set has held 114 names since before
this lane started, and §-118's row already records `112/323`. A rate written down
in prose goes stale where a by-name pin cannot, which is §44.10's whole argument
arriving a second time.

§22.6's same-ray property is checked rather than assumed, as §120.5 asks:
`normalizing_alone_stores_a_different_direction_and_reducing_first_does_not`
draws 210 000 pairs of proportional directions and still finds **0** that store
different bits. §120.1 is right that a correctly rounded answer keeps the
property by construction; the test is what says the construction holds.

---

