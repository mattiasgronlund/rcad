## 145. The step between two axes rounds to nothing, and takes the crossing with it, found 2026-09-19

*Two ordinary pipes, both one wide, their axes `6.7e-7` apart and `1e10` from the
origin, refuse both ways round. Move either origin one `f64` step and the same
pair is named in two straight runs. The step between the axes is worked out in
plain `f64` with no exact arm under it, and for this pair it cancels to exactly
zero. §16.10 **gap**. It is also the pair that reaches §142.7's annihilation —
and it shows that annihilation certifies nothing, because the same vanished
length refuses by a second route anyway.*

***Fixed by §148**, the same day: the step is taken multiplied by `w·w`, where
nothing cancels, and this pair is named in two straight runs both ways round.
Everything below describes the tree before that. §145.6's open list is answered
there too, except its third item — a pair reaching the annihilation with **no**
second route — which stands.*

### 145.1 The pair

Both pipes one wide, both axes along `Vector::new(0.3, 0.5, 0.7)`. The first
origin is `[0, 0, 0]`; the second is `along * 1e10` rounded componentwise onto
the `f64` grid:

```
[3292927799.69071, 5488212999.484517, 7683498199.278325]
```

Built through the public API alone — `Axis::new`, `Cylinder::new`, `intersect` —
and `intersect` gives `Err(CannotNameWhichPieceOfACrossingThisIs)` in **both**
orders. The two axes are genuinely apart: `between × along` comes out
`[-4.76837158203125e-7, 4.76837158203125e-7, 0.0]`, a separation of
`6.743495761743046e-7`. Nothing about this pair is degenerate, and nothing about
it is small.

That the exact level *certified* that separation is **inferred, not read**. A
fully certified outcome logs nothing by design, so the provenance log for this
pair comes back empty; what stands in for a reading is that the pair gets
**past** the crossing question, to
`CannotNameWhichPieceOfACrossingThisIs`, where every pair whose axes really are
within a rounding comes back `Err(Undecidable)` from that question instead. That
is evidence and not a certificate, and it is written as the weaker of the two.

### 145.2 Why the step vanishes

`intersect.rs:1003` works out the step from one axis to the other square to the
shared direction:

```text
towards = between − along · (between·along / along·along)
```

Every term there is plain `f64`. For this pair the projection lands exactly back
on `between`, so `towards` is exactly `[0.0, 0.0, 0.0]` — not small, not
straddling, *zero*. Rounding the second origin onto the grid is what arranges it:
the rounded origin is still a point the caller is entitled to write down, and it
is the nearest representable point to one that really does lie along `along`.

The vanished length is then divided by twice over. `apart =
root_enclosing(squared_length_enclosing([0,0,0]))` is the zero-width box at
zero, so `quotient_or_everything` takes its `ENTIRE` branch for `across` and for
`how_far` alike.

### 145.3 Why the annihilation rides on it, and certifies nothing

§142.7 asked whether any drawable pair reaches `how_far` `ENTIRE` and `leaning`
exactly zero at the same time. This pair does, and the two conditions are not
independent — which is the part §142.7 could not see. `lines.sideways` **is**
`towards.components()`, so `leaning = Exact::dot_of(picker_normal, [0,0,0])` is
an exact zero whatever the picker turned out to be.

In this caller `how_far == ENTIRE` therefore *implies* `leaning == 0` — **for
everything that gets here**, which is not the same as for everything the code
allows. `quotient_or_everything` takes its `ENTIRE` branch whenever `apart`
merely *contains* zero, and a `towards` whose components underflow does that
without being exactly zero, which would leave `leaning` a small non-zero box.
What closes the hole is §145.5's dead end rather than this paragraph: an
underflowing `towards` never reaches `two_lines_between_two_pipes` at all,
demonstrated on thirteen same-width pairs with gaps from `1e-100` down to
`5e-324`, every one of which comes back `Err(Undecidable)` from the crossing
question, against the same pairs at `1e-8`, `1e-16` and `1e-30` all named in two
runs. The implication holds; it holds because of §145.5, and saying so is the
difference between a demonstration and a sentence that happens to be true. §142.7's H2 — that the exact zero would be the
hard half, a perpendicularity coincidence the fuzzer would never hit — is
falsified: the exact zero is the easy half, and it is common away from the case
as well. One step either side of this pair the picker normal is
`[5.233967780575292e-7, -3.140380668345175e-7, 0]` against a `towards` of
`[0, 0, 9.5367431640625e-7]`, dot exactly zero, with `how_far` perfectly finite.

**And the annihilation buys nothing — which is as well, because the argument
that it is sound does not survive this pair.** §142.7 reasoned that the swallow
was safe because "wherever the picker was built at all, its normal came from a
cross product with `towards`, so `towards ≠ 0`" and the true quotient was
therefore a finite real. Here `towards` **is** exactly zero, `along × towards` is
zero with it, and `HalfSpace::through_a_point_square_to` builds the picker from
one of its three fallbacks instead — its doc says `towards` "may be anything at
all, zero included". So the picker exists and the premise does not, the divisor
handed in is exactly zero, and the quotient the expression asks for is not a
finite real at all. **§142.7's argument is falsified at its premise by the very
pair that first reaches the case it was written about.**

What saves the site is a second route, not the argument. The same vanished
length is what `sideways` is normalised by, so `up` is `ENTIRE`, `either_way` is
`ENTIRE`, and neither side of the picker can be signed. `out` stays tight —
exactly the asymmetry §142.7 predicted would be the opening — and it does not
matter. `quotient_or_everything`'s "turns into a refusal" survives here by that
second route, and `enclosure.rs` now says so in those terms. This is why
§145.6's "is there a pair with no second route" is the item on that list that
would cost something.

### 145.4 The other caller cannot reach it, demonstrated

`two_lines_told_apart` is private with exactly two call sites, beginning at
`intersect.rs:438` and `intersect.rs:1034`, both reached from the public
`intersect`. That is the whole family, so §16.10's "check the public API" is
discharged by checking two functions.

The other one cannot reach the case at all. In `two_lines_along_a_pipe`,
`how_far`'s divisor is `squared_length_enclosing(exactly(flat.normal()))`, and a
`Plane`'s normal comes from `scaled_to_length_one_from_anywhere`
(`plane.rs:190`), so that enclosure is a box around one and never contains zero.
Demonstrated rather than argued: over a thousand planes built from coefficient
triples spanning `0`, `±1`, `3`, `1e-8`, `1e±160`, `1e±300`, `f64::MIN_POSITIVE`
and `5e-324`, the squared-length enclosure never contains zero.

### 145.5 That it is a gap, and the dead end that looks like the way in

Move the second origin one step along `z`, or two, either way: the same pair is
**named in two straight runs, both orders, all four nudges**. The answer exists,
the pair is ordinary, and the arithmetic lost it only because one `f64`
subtraction cancelled exactly. §16.10: **gap**, and the same family as §140 one
step earlier — §140 fixed the *reading* of two lines once they had been handed
over as whole coordinates; this is the step between the axes being worked out in
plain `f64` before anything exact runs at all.

**The dead end, written down because it is the obvious first move.** Making the
two axes genuinely close does not get here. `apart`'s divisor only straddles zero
on underflow, which wants the separation below about `2.2e-162`; but then
`how_far_off_an_axis_exactly` (`intersect.rs:851`) needs `product_is_exact` on
expansion entries around `1e-200`, returns `None`, the enclosure arm underflows
too, and `WHERE_TWO_PIPES_CROSS` is `Undecidable` — so the pair never reaches
`two_lines_between_two_pipes` at all. `Exact` is a Shewchuk expansion
(`exact.rs:99`) and `squared_length_of` has no power-of-two slide to save it.

### 145.6 What a fix would be, and what is not ruled out

`towards` is the one quantity on this path with no exact arm under it, where
`out` and `squared_gap` twenty lines below both have one
(`intersect.rs:1017-1032`). Giving it the same `map_or_else(enclosing,
Exact::enclosure)` shape is the candidate, and it is a candidate and not a plan.

Not ruled out, listed rather than glossed:

- Whether an exact `towards` is even representable for this pair — the
  projection is a quotient, and `Exact` holds no quotients. The honest form may
  be to scale the step by `along·along` and carry the scale, which changes every
  reader of `sideways` downstream.
- Whether `up` and `across` both recover from one fix, or need two. `up` divides
  by a *different* root of the same vanished length.
- Whether any pair reaches the annihilation **without** the second route. That is
  the case that would falsify `enclosure.rs:102` outright, and nobody has looked
  for it.
- ~~What this does to the refusal rates in
  `how_often_naming_a_crossing_refuses.rs`.~~ **Measured, and nothing moved.**
  All thirteen tests pass on the fixed tree and every published rate holds: the
  turned headline's half-space column at 0.05%, the quartic turned at 2.10%, the
  band row at 3.17%, the somewhere-else row's sign column at 10.58%, and the
  rest at zero. The two figures corrected in `023d3f57` were corrected before
  this fix and survive it.

