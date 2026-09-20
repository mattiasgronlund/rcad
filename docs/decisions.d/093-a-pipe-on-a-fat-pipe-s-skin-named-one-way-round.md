## 93. A pipe on a fat pipe's skin, named one way round only, fixed 2026-09-13

§91.2 recorded the bug and did not fix it. This is the fix.

Two pipes whose axes run along one direction, the thin one's axis lying on the
fat one's skin. `intersect(fat, thin)` returned
`CannotNameWhichPieceOfACrossingThisIs`; `intersect(thin, fat)` returned `Curves`
with two runs. The crossing is two endless straight lines and both orders now
name it.

This was a **gap** and not a limit (§16.10), which §91.2 settled by observation
rather than argument: the other order produced the answer, so the answer exists
in `f64`. What was left was to find where the bits went.

### 93.1 Where the bits went

**Not where the input suggests.** The failing seed carries a coordinate of
5.04e153, and it is a red herring: it sits along the axis direction, so it names
the same line as any other point on it, and `x = 0` refuses identically. §91.2
measured that and this section does not revisit it.

The trigger is the **radius ratio alone** and it is scale-free. Fat-first stopped
deciding at `2²⁷·r₂` for `r₂ = 1` and at `2²⁶·r₂` for `r₂ = 1.984375` and for
`r₂ = 1000`. Re-measured here on the unfixed tree, the same three numbers came
back, so §91.2's table is reproduced and not taken on trust.

**The refusal was not the shape question.** The discriminant decided the pair
crosses in two lines. What then failed was *naming* the two halves, in
`two_lines_told_apart`, and at its second check rather than its first — the two
lines could not be shown to lie either side of the plane through both axes. The
two checks were instrumented separately to establish that; §91.2 had not said
which.

The arithmetic underneath it was `X = √(ρ₁² − across²)`, with `across =
(δ² + ρ₁² − ρ₂²)/(2δ)` placing both lines along the line between the axes. With
`δ = ρ₁` that is `across = ρ₁ − ρ₂²/(2ρ₁)`, so `ρ₁² − across²` is `ρ₂²` arrived at
by subtracting two numbers that agree to `ρ₂²/ρ₁²` of themselves. At `ρ₁ = 2²⁶·ρ₂`
`f64` has 53 bits and the answer needs the 54th.

Measured in one build, `r₂ = 1`:

| ratio | order | `ρ₁² − across²` | true value |
|---|---|---|---|
| 2²⁷ | fat-first | `[-0.0, 4.0]` | 1 |
| 2²⁷ | thin-first | `[0.9999999999999998, 1.0]` | 1 |
| 2⁶⁰ | fat-first | `[-0.0, 2.95e20]` | 1 |
| 2⁶⁰ | thin-first | `[-16383.0, 1.0]` | 1 |

The second row is the whole of why this looked like an **order** bug: asked
thin-first the same pair has `across ≈ 0` and no cancellation at all. The fourth
row says the order-dependence was a *window* rather than the shape of the thing —
past about 2⁶⁰ both orders refused, and nobody had looked that far.

### 93.2 The discriminant already knew

Heron's factorisation of `4δ²X² = 4δ²ρ₁² − (δ²+ρ₁²−ρ₂²)²` is
`−(δ²−(ρ₁+ρ₂)²)(δ²−(ρ₁−ρ₂)²)`, and that product is the twelfth-degree
discriminant §27.1 wrote down, divided by `(W₁W₂)²`. With `δ² = A/W₁`:

```text
X²  =  −(E² − 4r₁²r₂²W₁W₂)  /  (4·A·W₁·W₂²)
```

in the stored numbers alone, with no subtraction of two nearly equal large
numbers in it. The numerator is **the same expression the shape question asked**,
which is the point: a pair decided to cross in two lines is a pair whose two
lines can be told apart, and the two steps can no longer disagree. It is
extracted into one function, `how_deeply_two_side_by_side_pipes_overlap_*`, so
that is structural rather than a coincidence two call sites have to maintain.

**The first version of this fix was not enough, and the reason is worth keeping.**
Written with interval arithmetic alone it fixed fat-first at every ratio to 2¹⁰⁰
and left thin-first refusing from 2⁶⁰ — because `E = A·W₂ − r₁²W₂ − r₂²W₁`
cancels in its own right when `δ ≈ ρ₂`, which is the same geometry asked the other
way round. Summing the exact expansion instead of enclosing it fixes that, and
`Exact::enclosure`'s own doc comment already names this case: a quantity whose
interval enclosure reaches down to zero is known to full relative accuracy when
it is summed through the expansion. So the numerator and `A` both take the exact
path where it runs, and fall back to the enclosure where it does not.

The refusal that is left is **near-tangency**, where the two lines genuinely
approach each other and `X → 0`. Refusing there is honest.

**`root_enclosing` asserts its argument is not wholly below zero, and that assert
cannot fire here.** Not by inspection of today's callers but because the two sites
share one function: where `how_deeply_two_side_by_side_pipes_overlap_exactly`
runs, the branch's sign is certain and the numerator is truly above zero; where it
returns `None`, the sign came from the interval level or from a tolerant policy,
and `X` falls back to the same interval, which cannot be wholly the wrong side of
zero without the branch not having been taken. A tolerant policy was then walked
through both tangencies, inside and out, a couple of ulps either side of each, and
answered identically to the certified one on all ten pairs — which is the
demonstration §16.10 asks for in place of the paragraph above it.

### 93.3 What moved, measured

Both orders now decide, `2²⁰` through `2¹⁰⁰`, for `r₂` of 1, 1.984375 and 1000.
The artifact that exited 77 at `main`'s tip exits 0.

The corpus row moved **twice over, and only one of the two is the seed**. The fix
alone took orders 547 → 548 and scalings 1093 → 1095 with nothing added, which is
one pair in the checked-in corpus becoming decidable both ways round. It has to be
a side-by-side pipe pair: the only arithmetic this changed sits in
`two_lines_between_two_pipes`, which nothing else reaches. The seed then takes them
to 549 and 1097. Measured the way §91.3 and
`b7bda36` measure, by adding it and taking it away again, it brings one seed, one
pair, one order, two scalings and **no points** — two endless lines are named
rather than walked — and no measurable time: 63.15 s against 63.61 s, which is
§91.3's slow seed and not this one.

The seed's kinds byte is `0x0a`, like §85's, §88's, §89's and both of §91's, so
the repack counts of 92 and 42 are unmoved and the paragraph in
`every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs` that says why now
covers six seeds without having been rewritten for any of them.

**A count in `how_often_naming_a_crossing_refuses.rs` was two out, and it is not
this change's doing.** Its `a_crossing_somewhere_else` row published "10.5%, 215
refusals" measured 2026-09-01; it reads 10.58% and 217 today, with ninety-one
commits to `rcad-geom/src/` in between. §93 cannot be either of the two: the
counter is `Error::Undecidable`, and this diff adds and removes no `ask` call
anywhere — the discriminant question it moved into a function asks the same two
closures in the same order. Re-measured rather than left wrong, because a measured
number nobody re-measures is the thing §77.6 is about.

**A sentence in that file was wrong and is corrected here rather than quietly.**
It promised that "when §91's gap is closed this row goes back under a tenth of a
second". §91 recorded two gaps; this closed §91.2's and the row moved by 0.46 s,
inside the noise. The 63 seconds belongs to §91.3's leaning pair, which is still
open and is the next job. The sentence now names the subsection.

§94 then closed §91.3 and the prediction came half true: the row did go back
under a tenth of a second, and it was not a gap that was closed.

