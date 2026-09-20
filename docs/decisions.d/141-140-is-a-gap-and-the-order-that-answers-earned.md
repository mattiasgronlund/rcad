## 141. §140 is a gap, and the order that answers earned it, diagnosed 2026-09-19

*§140.5 left one question open on purpose: whether the refusal was the defect or
the answer was. It is the refusal. Exact rational arithmetic puts both crossing
conditions positive by `5.4531438193293825e146`, so the answer exists; and the
order that gives it certifies a sign through `inari`'s directed rounding, so it
is **sound**. §16.10's verdict is **gap**. The fix is named and not built here;
an `#[ignore]`d reproducer is on `main` as `d49c7068`.*

### 141.1 How it was asked

Three lanes off `75da5fc1`, none of them allowed to write `crates/*/src/` or this
file. One committed a single `#[ignore]`d test; the other two committed nothing.

| lane | question | what came back |
| --- | --- | --- |
| exact (Opus) | limit or gap, and did the answering order earn it | both, demonstrated; the reproducer |
| site (Sonnet) | where the two orders part company, and since when | `intersect.rs:1650`, and not a regression |
| family (Sonnet) | how wide the disagreeing region is | a region, and its four boundaries |

Splitting it three ways paid for itself twice, and both times by **contradiction
rather than by coverage**. The site lane's reading of the mechanism was corrected
by the exact lane's arithmetic (§141.4); an apparent conflict between the exact
lane's ladder and the family lane's sweep turned out to be a resolution gap
(§141.5), found only because two independent sweeps existed to disagree.

### 141.2 The answer exists, by exact rational arithmetic

Both axes run along `x`, so the whole question is two circles in `(y, z)`: one of
radius `r1 = 5.047645337156084e153` at the origin, one of radius
`r2 = 5.4016709327697754e-8` at `(r1, 41581179)` — the y-offset is exactly `r1`.
They cut each other twice exactly when `(r1−r2)² < d² < (r1+r2)²` for
`d² = r1² + z²`.

In exact rational arithmetic on the `f64` values — `fractions.Fraction`, no
rounding anywhere — `d² − (r1−r2)²` and `(r1+r2)² − d²` both come to
`5.4531438193293825e146`. They are not the same number: subtracting one from the
other leaves `2(z² − r2²)`, about `3.46e15`, which is 131 decades below the
values themselves and so invisible at `f64` display width. That is algebra, not a
second measurement, and it is written here so a later reader does not read two
identical figures as a slip.

Both strictly positive, against `z² = 1728994447030041.0`. The thin pipe's axis
misses the fat pipe's skin by `1.712674258532773e-139`, which is `3.17e-132` of
its own radius: the fat pipe cuts the thin one near enough through the middle.
**Not a knife edge, and not a limit.**

### 141.3 The answering order earned its answer

§140.5's worse branch — that the answering order is unsound and claims two curves
it has not got — is **dead**, on three counts.

The sign that settles the shape is `sign(E² − 4·r1²·r2²·W₁²)`, with
`E = A − W₁(r1²+r2²)` and `A = ‖(o₂−o₁)×w₁‖²`, in
`how_deeply_two_side_by_side_pipes_overlap_exactly`
(`crates/rcad-geom/src/intersect.rs:1549`). **`r1+r2` and `r1−r2` are never
formed** — only squares, and every intermediate stays in range:
`r1² = 2.547872344971356e307`, `E = z² − r2² ≈ 1.729e15`,
`4·r1²·r2² ≈ 2.97e293`. Both orders print an **empty `ProvenanceLog`**, this
repo's signal that nothing was fudged. And the certified sign agrees with exact
rational arithmetic on the factored form `W₁²(δ²−(r1+r2)²)(δ²−(r1−r2)²)`.

The second certificate, `side[0].lo() > 0.0 && side[1].hi() < 0.0` at
`intersect.rs:1647`, rests on the enclosures being honest, and that was checked
rather than assumed: `Interval` is `inari`-backed with directed rounding
(`crates/rcad-predicates/src/interval.rs:13`, `lo()` is `inf()`, `hi()` is
`sup()`), and `Interval::exactly` enters at zero width. So `lo() > 0.0` is a
rigorous certificate: the test can refuse what is true, never certify what is
false.

One caveat worth carrying into any change here: `Interval::exactly` returns
`ENTIRE` for a non-finite input, and `ENTIRE * [0,0] = [0,0]` annihilates an
unknown. It bites nothing in this pair — `normal.x` is exactly `0.0` and the true
x-contribution is exactly zero either way — but a reformulation that multiplies a
possibly-`ENTIRE` term by an exact zero would be relying on that.

### 141.4 Where the sign is lost — and the mechanism that is *not* it

The refusal is `intersect.rs:1650`, the either-side separation test, and **not**
the squareness check four lines above. Demonstrated, not inferred: the picker's
normal prints as `(0.0, 41581179.0, -5.047645337156084e153)` and the direction as
exactly `(1,0,0)`, so `Exact::dot_of(normal, along)` is exactly zero and 1644
passes in both orders.

Nothing canonicalises which surface anchors the frame. `intersect`'s pipe-pipe
row (`intersect.rs:188-190`) passes `(first, second)` straight through, and
`two_lines_between_two_pipes` (`intersect.rs:1002-1003`) takes `here` and `along`
from its first argument. So the first-named surface sets the frame — which is
**how** the two orders come to differ, but it is not **why** one of them fails.

The tempting explanation, and the one this section's first draft carried, is that
the fat pipe's clean `(0,0,0)` against the thin pipe's `~5.05e153` origin sets the
enclosure width. That is false. **The anchor enters as `Interval::exactly(here[axis])`
— zero width — so its magnitude costs nothing by itself.**

What drives it is `across`, the step along the axis-to-axis line, which is a wide
interval in both orders because it carries the `apart·apart − r1²` cancellation.
What differs is its size:

| named first | `across` | `across / out` |
| --- | --- | ---: |
| fat pipe | `5.047645337156084e153` | `9.34e160` |
| thin pipe | `1.712674258532773e-139` | `3.17e-132` |

with `out = 5.4016709327697754e-8`, a wanted signal of
`out·δ = 2.7265719096646913e146`, and terms of `z·r1 = 2.0988704429280249e161`
that have to cancel to reach it. Asked fat-pipe-first the test must resolve a
term 161 decades above its answer; asked thin-pipe-first the same term sits 132
decades below it. **The driver is `across` relative to `out`, not the anchor's
distance from the origin.**

That distinction is not pedantry — it decides the fix. Canonicalising by radius,
the obvious repair once you believe the anchor story, picks the order whose
`across` happens to be small in *this* configuration, over a quantity that is not
the radius.

### 141.5 The ladder, and that the kernel is never wrong on it

Sweeping `r2` at the found geometry in powers of two against exact truth, 460
rows:

| `r2` | exact truth | fat pipe first | thin pipe first |
| --- | --- | --- | --- |
| below `1.712674258532773e-139` | nothing | `Undecidable`, enclosure `inf` | `Undecidable`, enclosure `inf` |
| `3.044037611241337e-139` up | curves 2 | `Undecidable`, enclosure `inf` | `Undecidable`, enclosure `inf` |
| `1.2822983070604231e-73` up | curves 2 | `CannotName…` | `CannotName…` |
| `2.7008354663848877e-8` up | curves 2 | `CannotName…` | **curves 2** |
| above `5.4016709327697754e-8`, below `8e-8` | curves 2 | curves 2 | curves 2 |

**In all 460 rows the kernel is never wrong.** It answers `curves 2` only where
the truth is `curves 2`, and otherwise refuses. So these are not tolerances being
crossed, they are **precisions**: the point where the enclosure becomes tight
enough to prove what is true. A tolerance would eventually answer something false;
this never does.

The last row is a correction to the ladder by a second sweep, and worth recording
as a method note. The powers-of-two grid put the return to agreement at
`1.0803341865539551e-7`, which is simply the next grid point after the found
value — the grid never sampled between them. An independent sweep at `8e-8`
found both orders agreeing, so the disagreement band's upper edge lies between
`5.4016709327697754e-8` and `8e-8`, and the band is **narrower than the ladder
alone could show**. Two suspicions about why the sweeps disagreed were both
wrong: `bytes_for`'s five-byte encoding does quantise, but `8e-8` round-trips to
`8.000000040198074e-8`, a relative error of ~5e-15 and nowhere near enough to
matter.

So the gap runs from about `3.04e-139` to just under `8e-8` — roughly 131 decades
— of which the commutativity failure §140 was found through is the **top sliver**,
from `2.70e-8` to under `8e-8`. The oracle that caught this catches disagreement,
and disagreement is the narrowest part of the defect.

The surrounding region was swept on four axes independently: `r1` disagrees
contiguously from `1e30` to `1e150`; `z` within about `3e7` to `8e7`; and the thin
axis has to sit within about `1e-12` of `r1` relatively — on the fat pipe's skin,
not merely near it. A region, not a point. The drawable window is bounded either
way: `bytes_for` refuses `r1 = 1e300` and `r2 = 1e-300` outright.

### 141.6 A second band, undiagnosed and named as such

Row two of the ladder is **not** §140's defect. From about `3.04e-139` to about
`1.28e-73` — some 66 decades — the true answer is two curves and both orders
return `Error::Undecidable` with an **infinite enclosure**, which is the shape
question giving up rather than the picker test refusing.

What makes it its own finding is the first row beside it: the truth flips from
"nothing" to "two curves" between rows one and two, and the kernel's answer does
not change across that flip. It was returning the same "nothing is known" on both
sides of a real geometric transition.

It was found while sweeping for something else and **was not diagnosed**. The lane
that found it said so rather than folding it into the verdict, which is the
behaviour this record wants. It owes its own look, and has a lane running.

*That lane reported. **§143** has it: a gap, its edge predicted by
`product_is_exact`'s representability floor to the digit, and two mechanisms
already in this repo that reach the sign. Still unfixed.*

### 141.7 The family of fixes, and what is not ruled out

**Named, not built.** Form the side test in displacement coordinates rather than
absolute ones. The picking plane passes through `here`, so `dot(normal, here) −
offset` is analytically zero; `normal = along × towards` is analytically square to
`sideways`. What survives is `side = ±out·δ` — a product of two positives with no
cancellation anywhere — whose sign follows from `out > 0`, which follows from the
discriminant the caller has **already certified exactly**. It removes `across` and
the anchor together. This is the same move §93 made one level up when it took `X`
from the discriminant rather than from `√(r1²−across²)`.

Ruled out **by arithmetic that was run**, not by argument:

- **A power-of-two rescale.** At `2^-128`, `2^-64`, `2^-32` and `2^0` the
  fat-first order gives the identical refusal; at `2^-256`, `2^32`, `2^64` and
  `2^128` the shape question itself goes `Undecidable`. An exact power-of-two
  scale is a similarity, so every relative error is unchanged — it only moves
  which level fails.
- **Pushing an exact level into the side test.** `out` comes through
  `root_enclosing`, a square root, so there is no exact expansion of `place` to
  hand to an exact predicate. The displacement form is what removes the need.
- **Canonicalising by radius.** §141.4: the driver is `across`, which is not the
  radius.

**Not ruled out, and deliberately not written as ruled out:**

- **That the displacement form is the only family that reaches the sign.** Nobody
  showed that. §16.10 asks for the family to be named and it is named; "no other
  rearrangement reaches it" is a sentence this section declines to write, and
  §140.8 says why.
- **The `z²/(2·r1)` against `r2` rearrangement.** Not the fix for §140's refusal —
  the shape question already decides that correctly, with an empty log. It is a
  candidate for §141.6's band instead, and is unverified.

### 141.8 Not a regression: `c4f1872a`'s gap, not reached

`c4f1872a` (2026-09-13) fixed this same either-side check for radius ratios
`2^20` through `2^100`, by removing a subtraction of two nearly equal huge
numbers. This pair is about `2^535`. A lane checked that commit out and re-ran
the identical sweep: byte-for-byte identical results to `75da5fc1`, including the
`HalfSpace` normals. Only two later commits touch the pipe-pipe row — `8811cb8b`,
in the coincident-axis branch, and `35e21ce8`, whose dropped
`squared_theirs_direction` factors are exactly `1.0` under §118.11's unit-direction
convention and so an algebraic no-op.

**That paragraph is the site lane's, and the exact lane did not check it** — it is
recorded here on one lane's word, which is the standard §138 asks for when a claim
has one source. What corroborates it independently is a different measurement: with
the thin axis on the skin and `z = 0`, both orders agree `curves 2` at every ratio
from `2^0` to `2^510`, both going `Undecidable` at `2^511` from overflow. So the
ratio alone is survivable, and this pair needs `z ≠ 0` on top of it — which is
exactly what makes `apart·apart − r1²` cancel.

### 141.9 The dispatcher's hypothesis was right about the geometry and wrong about the mechanism

All three lanes were handed a hand-computed hypothesis, labelled unverified and
with an instruction to falsify it. Both halves are worth the record.

**Right, and it predicted a boundary before it was measured.** The cross-section
reduction, and that the thin axis sits on the fat pipe's skin. If the real test is
whether the thin axis lies within `r2` of the skin — `z²/(2·r1) < r2` — the
geometry should flip from "nothing" to "two curves" at `r1 = z²/(2·r2) ≈ 1.6e22`.
The family lane, sweeping independently and without that number, found agreement
on "nothing" at `r1 = 1e20` and disagreement at `1e30`. A model that calls a
boundary before it is measured is better evidence than the pair that suggested it.

**Wrong about the mechanism, in the direction that wastes a lane.** The hypothesis
said the loss was `r1 ± r2` and `sqrt(r1² + z²)` collapsing onto `r1`, since
`ulp(r1) ≈ 7e137`. Those sums are never formed in the shape question — which is
precisely where the hypothesis pointed. A lane that had taken it on trust would
have gone looking in the wrong function.

**And one reading offered mid-flight and withdrawn.** From the family lane's
threshold ladder the dispatcher suggested two different tolerances in two code
paths. There are no tolerances; there are two precisions (§141.5). It was sent as
a correction before the lane it was sent to reported.

The argument this makes is not "do not hypothesise". It is that a hypothesis is
worth handing to a lane **only** with the instruction to falsify it attached, and
that the instruction has to be real: the lane that was right here is the one that
went and checked what `Interval` actually rounds with.

### 141.10 What it owes

A fix lane, running, against the named family and the reproducer already on
`main`. §141.6's band owes its own diagnosis, and has a lane. Whether the shape at
§141.4 — a wide cancelling intermediate multiplied into a sign test, with nothing
bounding its size against the signal — occurs anywhere else in the kernel is a
sweep, also running. None of the three may write this file.

*Both lanes reported, and both are written up: the sweep as §142, the band as
§143. §140 itself is fixed and on `main` as `33ea2c0a` — §142.5 has the fix and
what it cost.*

