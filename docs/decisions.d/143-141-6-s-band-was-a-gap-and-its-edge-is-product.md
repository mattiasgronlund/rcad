## 143. §141.6's band was a gap, and its edge is `product_is_exact`'s floor, fixed 2026-09-19

*The second refusal band §141.6 recorded is `Error::Undecidable` with an infinite
enclosure, 66 decades wide, where the answer exists. It is a gap. The infinity
comes from squaring a catastrophic cancellation, the band's top edge is the exact
arm's own representability floor — predicted, and landing on the measured edge to
the digit — and two mechanisms already in this repo reach the sign. **Fixed on
2026-09-19 by the second of those two; §143.9 has the fix and the three
amendments it forced on this section.***

### 143.1 The lead that was wrong

The dispatcher pointed the lane at `quotient_or_everything`
(`crates/rcad-geom/src/enclosure.rs:102`), which returns `Interval::ENTIRE` on a
divisor that straddles zero. **There is no division anywhere in the refusing
path.** `two_lines_between_two_pipes`, which does divide, is never reached. The
lead cost two probes, and is written down because §141.9's point is that a
hypothesis handed to a lane is worth what its falsification instruction is worth.

### 143.2 Where the infinity is born

`crates/rcad-geom/src/intersect.rs:1533`, the `room * room` in
`how_deeply_two_side_by_side_pipes_overlap_enclosing`, term by term:

```
A                = [2.547872344971356e307, 2.5478723449713564e307]
W1*(r1^2+r2^2)   = [2.547872344971356e307, 2.5478723449713564e307]
room = A - that  = [-4.9896007738368e291, 4.9896007738368e291]
room*room        = [-inf, inf]
```

`E = A − W₁(r₁²+r₂²)` is truly `Z² − r₂² ≈ 1.728994447030041e15`, but `Z²` sits
292 decades below `ulp(R1²)` and is annihilated; squaring what is left overflows.
The refusing record is the `edge` question at `intersect.rs:945-951`, whose `?` at
951 becomes the `Error::Undecidable`.

### 143.3 Why the band ends exactly where it does

The exact arm is a **second, separate** failure, and it is what sets the edge.
`how_much_room_two_side_by_side_pipes_leave_exactly` succeeds — `R1²` cancels
exactly, leaving the two-entry expansion `[-r₂², Z²]`. What refuses is
`room.times(&room)` at `intersect.rs:1550`: `Exact::times` (`exact.rs:275-287`)
requires `product_is_exact` of every pair, which wants
`|a·b| ≥ f64::MIN_POSITIVE · 2^53 = 2.004168360008973e-292`
(`crates/rcad-predicates/src/expansion.rs:377-384`).

The smallest entry squared is `r₂⁴`, so the arm needs
`r2 ≥ 1.1898262617302253e-73`. The two representable radii straddling that are
`6.411491535302116e-74`, which refuses, and `1.2822983070604231e-73`, which
decides — **the measured edge, to the digit.** A predicted boundary and not a
fitted one, which is the strongest evidence in this section.

### 143.4 That it is a gap

Written as `E = Z² − r₂²`, in the same `Interval` type, the enclosure is **0.2
wide** against the current arm's ±`4.99e291` — 292 decades of information thrown
away — and the sign is decided at every one of 320 swept radii. The Pos→Neg flip
lands at `3.044037611241337e-139`, which is §141.5's row-1/row-2 boundary, and the
analytic flip `Z²/(2·R1) = 1.712674258532773e-139` matches row 1's top to all 16
digits. §16.10: **gap**.

### 143.5 Two families that reach it, both already in this repo

1. **`Exact::square_minus`** (`exact.rs:222-226`) — `self² − already_squared`
   with both sides slid into the squarable window.
   `how_deeply_two_side_by_side_pipes_overlap_exactly` hand-rolls
   `room.times(&room).minus(…)` instead. Computed across the sweep it succeeds
   from `r2 = 7.431732449319671e-143` up: the whole band, with four decades over.
   Already called at `torus.rs:447` and `three_surfaces.rs:1781`.
2. **`Exact::enclosure()` off the exact expansion** — the exact `room` succeeds
   here, and its enclosure is the tight `[1728994447030040.8,
   1728994447030041.0]`. The `map_or_else(…, Exact::enclosure)` pattern sits 20
   lines away at `intersect.rs:1021-1032`. **The stronger of the two**: it also
   decides the row below the band, where family 1 runs out once `r₂²` goes
   subnormal.

The dispatcher's `z²/(2·r₁)` against `r₂` candidate is correct but is not
minimal — it is the Heron factor `E − 2r₁r₂W₁` divided by `2r₁`, and was verified
against the scaled discriminant on every row rather than argued.

### 143.6 What is not ruled out, listed rather than glossed

- Whether either family is **order-independent**. Only today's answers were
  checked both ways round, not the proposed formulations with the thin pipe
  first, where `W₁` and the gap vector differ.
- Whether family 2 is sound **in general**. The exact arm succeeds *for this
  pair*; if `A` itself overflows there is nothing to take an enclosure from.
- The **general-`W₁` algebra**. Every number here is for `W₁ = 1`.
- The correct answer **below** the band after a fix. Family 2 returns `Pos`,
  which should mean `Nothing`, but that was not followed through to a
  `SurfaceMeet`, and the truth column was taken from a sibling lane on trust.
- Whether `square_minus`'s window claim holds here — its constants were
  recomputed by hand; the function is `pub(crate)` and was not called.
- **Whether this fix and §140's compose.** §140's fix changes
  `two_lines_told_apart`, downstream of a `Decision::Neg` this band never
  reaches. If both land, these rows become `curves 2` rather than a refusal.
  Unchecked.

### 143.7 A decoder fact that will cost the next lane a probe

**The fuzz decoder normalises directions.** `bytes_for` was handed `dx1 = 127`
and the pair comes back with `direction [1,0,0]`, so `W₁ = 1`, not `16129`.
Reasoning from the seed array's raw direction numbers puts every magnitude out by
about four decades, which is what happened before the lane printed the decoded
cylinders.

### 143.8 The evidence, and where it lives

The probes are committed on their own **unmerged** lane branches, because each
would break `check-ignored` on `main` and because the worktrees that held them are
reclaimable: `48824380` on `lane-140-undecidable` (this section's four probes),
`322feb25` on `lane-140-family` (§141.5's boundaries), `cdafe474` on
`lane-140-site` (the site probe that pinned §140 to `intersect.rs:1650`). Take
what a fix needs from them and turn it into a real test; do not merge them as
they stand.

### 143.9 The fix, and the three things this section had wrong

`how_deeply_two_side_by_side_pipes_overlap_enclosing` now takes `E` from the
exact arm whenever the exact arm can form it, and falls back to the old interval
`E` when it cannot — §143.5's family 2, in the
`map_or_else(…, Exact::enclosure)` shape `two_lines_between_two_pipes` already
used twenty lines below it. The test is
`a_pipe_too_thin_to_square_still_crosses_the_one_it_lies_on.rs`, four tests, none
ignored. Reverting the one `room` line turns all four into assertion failures,
the first of them at `r₂ = 6.411491535302116e-74` — §143.3's *predicted* refusing
radius, one representable step below the band's top edge.

The band decides both ways round from `3.044037611241337e-139` to
`5.4016709327697754e-8`. Below it the answer is `SurfaceMeet::Nothing`, and the
kernel's flip to it sits within one step of a 6-bit mantissa of the exact
cross-section criterion — `2·R₁·r₂ + r₂² − Z²` crosses from `−7.029884024601e12`
to `+6.215997229287e12` at `r₂ = 1.718831582209548e-139`, against the analytic
`Z²/(2·R₁) = 1.712674258532773e-139`. Every sweep was run in both orders and no
radius decided one way and refused the other.

**What still refuses, named rather than rounded off.** Over 2 000 log-uniform
radii asked both ways, 62 of 4 000 still refuse — 1.55% — and all 62 are the
same thing: `r₂²`'s roundoff half is nonzero and below `Exact::times`'s floor of
`2.004168360008973e-292`, the largest of them `1.8467223357979438e-292`, so the
exact `E` cannot be formed at all and the old interval arm is what answers.
Across all 6 000 asks the file makes, the kernel was never *wrong*, only silent:
every ask that was not a refusal came back `curves 2`. That residue is the floor
itself, not a new defect.

*Those figures read "146 of 8 000 — 1.83%" and "roughly 13 000 asks" for the
length of one push.* The audit reproduced `146 of 8 000` exactly — by doubling
the sample to 4 000 radii, which is not what the committed test draws. A number
that is right about a run nobody kept is the hardest kind to catch, because it
survives every check except re-running the thing it claims to describe. §142.6
is the same failure one section earlier, and §142.3's caution is why this
section was audited at all.

**Amendment one — §143.5 understated family 1.** It is not the weaker of the two;
it is **unusable at this site**. `Exact::square_minus` returns "a sign and never a
value", and `how_deeply_two_side_by_side_pipes_overlap_exactly` has two callers:
the `edge` question at `intersect.rs:945`, which reads a sign, and
`intersect.rs:1017`, where its enclosure becomes `-overlap`, the numerator of
`out` — the distance between the two named lines. Family 1 there would silently
scale that distance by a power of two. It cannot be used without first splitting
the function into a sign-only and a value version. §143.6's fifth unruled item,
whether `square_minus`'s window claim holds here, is moot for the same reason.
*This was the dispatcher's hypothesis, handed over labelled as one, and the lane
confirmed it by reading the callers rather than by agreeing with it.*

**Amendment two — §143.6's fourth item expected too much.** The fix's reach below
the band is bounded by the same `Exact::times` floor, not universal: at
`r₂ = 1e-140`, whose roundoff half is `9.136619874250988e-298`, the pair still
refuses. The truth column §143 took from a sibling lane on trust is right; what
was wrong was the unstated assumption that a fix reaching the band reaches
everything under it.

**Amendment three — §140's fix is load-bearing here, not merely compatible.**
§143.6 asked only whether the two compose. They do, and more: with both in place
the band rows come back `Curves` with two runs, and reversing `33ea2c0a`'s
`intersect.rs` hunk on top of this fix turns three of the four new tests red —
including at `r₂ = 5.4016709327697754e-8`, §140's own pair.

**The fourth survives, and naming why took two goes.** It is not, as this
section first said, that there is "no step along the axis-to-axis line to
cancel" — in that test the step along the axis-to-axis line is `R₁`, the largest
number in the file. What is missing is the step along the **pipe's own axis**.
In the leaning test the fat axis is `(1,1,0)/√2` and the thin one sits at
`(0, 0, R₁)`, so the picking normal is `(±0.7071, ∓0.7071, 0)` with its `z`
component exactly zero: the big term lands entirely in the one coordinate slot
the normal reads as nothing, and the small `±up·out` term lands where nothing
large sits. In the other three the thin axis is at `(0, R₁, Z)` with
`Z = 41581179`, the normal is `(0, 0, ±1)`, and `Z` shares its slot with `±out`
— which is exactly where §140's defect lived. Right conclusion, wrong name for
it, corrected by an audit that read the geometry rather than the sentence.

**General `W₁`, and what the public API can actually reach.** `Axis::new`
normalises, so the only non-unit `W₁` a caller can produce is one plus or minus a
few ulps; `(1,1,0)` gives `W₁ − 1 = 2.220446049250313e-16`. On that pair the band
is decided from `2.4352300889930697e-138` up — a higher floor than the
axis-aligned pair's, because `W₁`'s own roundoff entry must also clear
`Exact::times` when multiplied by `r₂²`. **A leaning direction costs a few
decades of reach**, which is worth knowing before anyone reads §143's numbers as
holding everywhere.

