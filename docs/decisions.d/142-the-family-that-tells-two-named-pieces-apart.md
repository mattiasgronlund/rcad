## 142. The family that tells two named pieces apart has no exact level under it, swept 2026-09-19

*§140 was not a one-off. Every ordinary shape decision in `intersect.rs` goes
through `ask(...)`, the three-level cascade ending in exact arithmetic — which is
why the file tolerates cancellation at all. The functions that decide **which of
two already-named pieces a point falls into** do not call `ask()`. They test a raw
`Interval` against zero with nothing underneath. §140 is that family's first
measured casualty, and the sweep found four more sites with its shape.*

### 142.1 The shape, stated so it can be searched for

Not "a point built far from the origin" — that was the dispatcher's first
version and §141.4 falsified it. The shape is:

> a point or vector assembled from an intermediate that already carries a
> cancellation, fed into a sign test against a plane or half-space, with nothing
> bounding the intermediate's size relative to the quantity whose sign is wanted.

The anchor is harmless because it enters as `Interval::exactly`, at zero width.
What is not harmless is a *wide* intermediate multiplied into the same dot
product, and the ratio that matters is that intermediate against the signal.

### 142.2 The family, and why it is the family

`two_lines_told_apart`, `two_loops_either_side_of_a_pipe` and
`two_loops_either_side_of_a_ball` all decide which of two already-named pieces a
point falls into, and **none of them calls `ask()`**. There is no exact level
under them: a straddling enclosure is a refusal and there is nothing to escalate
to. That is the structural reason §140 existed.

Two siblings in the same family are hardened **by construction** and are the
precedent for what right looks like here: `two_circles_round_a_shared_axis` and
`two_meridian_circles`, where either the sideways contribution is proved exactly
zero, or the margin is a plain linear product with no squared difference in it.

### 142.3 The four sites

| site | the wide intermediate, and what cancels | how far it was checked |
| --- | --- | --- |
| `intersect.rs:1016` `across` in `two_lines_between_two_pipes` | `(apart² + r₁² − r₂²)/(2·apart)`, cancelling when the radii are at very different scales | **measured** — the §140 reference row |
| `intersect.rs:440` `out` in `two_lines_along_a_pipe` | `root_enclosing(skin² − in_front²)`, cancelling near tangency | **measured**, §142.8 — a gap |
| `intersect.rs:1198` `clear` in `two_loops_either_side_of_a_pipe` | `root_enclosing(fat_radius² − reach²)`, `reach = apart + thin_radius` | **measured**, §142.8 — a gap, in a band one ulp wide |
| `intersect.rs:677-689` `reach`/`D₋` in `how_far_apart_two_loops_are` | three terms of comparable size, with no exact pre-summation — where the sibling at `:1303-1318` explicitly has one | **measured**, §142.8 — a gap |

**This table said "read, not measured" for three of its four rows until
2026-09-19**, and the distinction was the sweep's own, kept because a row that
says verified and was not is worse than no row. §142.8 closed all three, and
every one of them refuses. The caution was right to be there and turned out to
be cheap: the rows were true, they were just not yet earned.

### 142.4 What the sweep could not see

It read every row of `intersect`'s surface-pair match down to the leaves rather
than grepping, and every `dot_enclosing(...) - Interval::exactly(...)` in
`rcad-geom` and `rcad-topo` — `dot_enclosing` is defined in both
(`enclosure.rs:110`, `classify.rs:745`).

What it did not open: the ray-parameter solve feeding `march.rs`'s and
`classify.rs`'s marched points. `where_it_lands`'s `t` (`classify.rs:653`) is
solved outside `intersect.rs`, and whether *that* solve has a cancelling
intermediate is unchecked. It is named here rather than left as a silent edge of
the search.

`along_a_crossing.rs` and `which_way_a_closed_crossing_turns.rs` were checked and
are **not** in the family: both fall back from interval to exact
(`both_levels_of_the_rule`), so a straddling enclosure there is caught.

### 142.5 §140's fix, and the one row that moved

`33ea2c0a`. `two_lines_told_apart` takes a `TwoLinesEitherSideOfThePicker`
rather than two whole coordinates and forms the reading in pieces —
`off_the_picker + leaning·how_far ± out·(normal·up)` — with the two small dots
summed by `Exact` and an honest `dot_enclosing` fallback when `Exact` declines a
product. Algebraically identical to what it replaced; only the grouping and the
level changed.

**The recipe's two "analytically zero" claims were both false in general**, and
the lane measured them rather than writing them down. `dot(normal, here) −
offset` is the picker's own offset rounding. `dot(normal, sideways)` is zero only
when `through_a_point_square_to` used the asked-for `along × towards`; it has
three fallback normals square to `along` and nothing else. On §140's pair both
*are* exactly zero — that is the configuration, not the construction. Computing
both is also what made the **second caller** safe: `two_lines_along_a_pipe`
shares this callee and was converted in the same pass, rather than the callee
being changed on one caller's assumptions.

**The family property is preserved on purpose.** `two_lines_told_apart` still
tests a raw `Interval` against zero with no cascade under it. What changed is
what feeds the test, so it reaches further without certifying anything it has not
got — §142.2's gap is narrowed here and not closed.

One corner, recorded because it is a place the new code certifies where the old
refused: `leaning * how_far` can now be `[0,0] * ENTIRE = [0,0]`. Sound — an
exactly-zero real contribution stays zero against any finite multiple, and a
truly zero divisor is a different branch — but it is a widening and not a
rearrangement.

Measured on both trees with the same command, half-space refusals on the
"somewhere else" row of `how_often_naming_a_crossing_refuses` went **81 (3.95%)
to 0**, every other count in the file byte-identical, including that row's 217
want-of-a-sign refusals, which are a different question. `just gate` PASS at 2035
tests on the merged tree, re-run by the dispatcher rather than taken from the
lane, and the refusal row re-derived independently.

### 142.6 Two published numbers that are wrong, and are nobody's lane yet

Both in `crates/rcad-geom/tests/how_often_naming_a_crossing_refuses.rs`, both
pre-existing, both surfaced by the fix lane and outside its write set:

- `A_CROSSING_MAY_LACK_A_HALF_SPACE` publishes **"0.10% turned"** where both the
  base and the fixed tree measure **0.05%** (1 of 2052).
- Its prose calls the remainder "unclassified, not a limit" and does not mention
  the "somewhere else" row at all — which is the row that just went to zero.

§15.6 says a published bound owes a test. These are the inverse: a test whose
prose has drifted from what it measures. Recorded rather than fixed, because
changing a published sentence is not a thing to do in passing.

**Audited and closed, and the audit found a third.** Every published figure in
the file was re-measured against both trees rather than the two that were
noticed. `A_QUARTIC_CROSSING_MAY_LACK_A_HALF_SPACE` publishes **"2.24% turned"**
and measures **2.10%** (43 of 2052) on the base tree and the fixed one alike —
the same drift as the row above it, missed because the fix lane was reading the
rows §140 moved and this is a row §140 does not touch. Both numbers are
corrected to what the test prints.

The rest of the file's figures hold exactly: the "somewhere else" row's 10.58%
and its 217 refusals, "none at all leaning", "none on the lattice", and every
`…_MAY_REFUSE` constant's sign column at 0%. **Which quantity a constant gates
is what makes this checkable** — the `…_MAY_REFUSE` names gate the sign column
and the `…_LACK_A_HALF_SPACE` names gate the half-space column, and reading a
5% threshold against the wrong column is how a correct number gets reported as
a wrong one.

Both wrong figures were *higher* than measured, so the thresholds went on
passing and nothing failed — which is the whole reason prose drifts here and a
bound does not. Nothing new guards it: a gate arm over published percentages
would be the `§NNN` grep again, candidates everywhere and almost no hits.

**§0's own headline was re-measured and holds.** The "somewhere else" row's
half-space refusals are **81 of 2052 (3.95%) on `d67f958e` and 0 on `33ea2c0a`**
— that claim was written by the session that wrote the fix, which §140.8 names
as the species most worth checking, and it survives.

### 142.7 §140's fix falsified a promise `quotient_or_everything` makes, exactly as §141.3 said it would

`quotient_or_everything` (`enclosure.rs:102`) returns `ENTIRE` where its divisor
touches zero, and its doc comment says what happens next: what comes back "is a
box that says nothing is known, **which travels through the arithmetic above it
and turns into a refusal** (§4.1)".

That last clause is no longer true of `two_lines_told_apart`. `middle =
off_the_picker + leaning * lines.how_far` (`intersect.rs:1699`) feeds it a
`how_far` that is exactly such an `ENTIRE` — both callers build it with
`quotient_or_everything` (`:444`, `:1040`) — and multiplies it by `leaning`, an
exactly-summed dot that is analytically zero. Measured, not reasoned:

| product | result |
| --- | --- |
| `Interval::exactly(0.0) * Interval::ENTIRE` | `[-0, 0]` |
| `[1,1] + exactly(0.0) * ENTIRE` | `[1, 1]` — a sign certified with a term unknown |
| `[-1,1] * ENTIRE` | `[-inf, inf]` — a merely zero-*containing* box still refuses |

So the annihilation needs `leaning` to be an **exact** zero; the enclosure
fallback at the same site does not annihilate and does still refuse.

**§141.3 wrote this down before the fix existed** — "a reformulation that
multiplies a possibly-`ENTIRE` term by an exact zero would be relying on that" —
and §140's fix is that reformulation. The caveat was recorded against a
formulation nobody had written yet, and then the thing it warned about was
built. That is the §140.8 species running forwards instead of backwards, and it
is the reason the caveat earned its place.

**The argument that it is sound, offered as an argument and not a proof.** An
exact zero from `Exact` is a rigorous certificate, so the true term is
`0 × how_far_true`; and wherever the picker was built at all, its normal came
from a cross product with `towards`, so `towards ≠ 0` and the true `how_far` is
a finite real. `0 × finite` is `0`, which is what the annihilation returns. On
that reading IEEE 1788's rule is being used correctly rather than got away with.

**What is not demonstrated, and is not written here as if it were.** Whether any
drawable pair actually reaches the case — `how_far` `ENTIRE` and `leaning`
exactly zero together. In both callers the quantity that sends `how_far` to
`ENTIRE` also travels into `out` by a second route, which would refuse on its
own; whether the exact arm can make `out` tight while the enclosure arm leaves
the divisor straddling zero was not tried. §16.10 forbids the sentence "no
caller hits this" without the demonstration, so it is not written.

What is owed is small and is not a lane: either the doc comment at
`enclosure.rs:102` loses its "turns into a refusal" clause, or it keeps it and
names the one site that annihilates instead.

**Settled the same day, and both halves came out the other way.** §145 has the
pair: the case is reachable, so the clause could not simply be dropped for being
unreachable — and the annihilation certifies nothing anyway, because the same
vanished length refuses through `up` instead. The doc comment keeps the clause
and now names both the exception and the second route. The amendment that stood
here for part of a day said "no refusal follows", which was measured to be false;
what §142.7 got right was refusing to write "no caller hits this" without a
demonstration, and the demonstration is what overturned the guess that followed
it.

**The soundness argument above is falsified at its premise, too.** It rests on
"wherever the picker was built at all, its normal came from a cross product with
`towards`, so `towards ≠ 0`". §145's pair has `towards` exactly zero and a picker
built anyway, from one of `through_a_point_square_to`'s fallbacks. The site is
still safe, but by the second route in §145.3 and not by this paragraph's
reasoning — and the paragraph was careful to offer itself "as an argument and
not a proof", which is the only reason this reads as a correction rather than as
a defect.

### 142.8 The three unmeasured rows, measured — all three refuse, all three are gaps

§142.3's caution is discharged. Each row below is a pair built through the public
API, the printed enclosure at the refusing step, and the reason the answer exists
anyway. In all three the certified cascade had **already decided the very thing
the failing step needs**, which is what makes each a §16.10 gap rather than a
limit.

**`intersect.rs:440` — `out` in `two_lines_along_a_pipe`.** The plane `x + y = 0`
against a pipe `1e-8` wide whose axis runs through `(1e10, −1e10, 0)` along
`(1,−1,0)` — an axis lying *in* the plane. `intersect` gives
`Err(CannotNameWhichPieceOfACrossingThisIs)`. `n·o − d` encloses
`[-9.5367431640625e-7, 9.5367431640625e-7]` where the true sum is zero, and
`r² − in_front²` encloses `[-9.093947017729283e-13, 9.095947017729284e-13]`, so
`out.lo()` clamps to zero and neither line is provably on its own side. The
certified `ask` had already proved `n·n·r² − (n·o−d)² > 0` before this point, and
the two lines really are at `±1e-8`. The same plane and pipe at the origin are
named as two lines.

**`intersect.rs:1198` — `clear` in `two_loops_either_side_of_a_pipe`.** A pipe
`3e-17` wide along `x` through the origin, and a fat pipe along `y` through
`(0,0,1)`, so the axes are exactly one apart. Walking the first twelve `f64`
widths above that, **exactly one refuses** — `r_fat = 1.0000000000000002`, with
the next named one at `1.0000000000000004`. A refusal band one ulp wide.
`δ + r_thin` encloses `[1.0, 1.0000000000000002]`; at the refusing width
`r_fat² − reach²` encloses `[-2.220446049250313e-16, 6.661338147750939e-16]`.
Reaching this function at all requires `ask(r_fat − r_thin − δ)` to have come
back `Pos` under `Policy::certified()`, which is the exact statement that the
crossing is two loops.

**`intersect.rs:677-689` — `reach`/`D₋` in `how_far_apart_two_loops_are`.** A
pipe `1e-3` wide along `(1,1,0)` through the origin, and a ball `1.0005e-3` wide
centred at `(1e10, 1e10, 0)` — exactly on that axis. `A = ‖(c−o)×w‖²` encloses
`[-9.094947017729282e-13, 9.094947017729282e-13]` where the true value is zero;
`2r√(AW)` encloses `[-0.0, 1.9073486328125e-9]`, nearly twice the whole of `D₋`,
which encloses `[-9.080081275143498e-10, 1.00115949470212e-9]`. `reach.lo() == 0`
is sufficient on its own — the caller's margin subtracts two terms that are never
below zero. The certified policy had already decided `A = 0`, which is the only
way this pair reaches the ball-on-the-axis row at all.

**Two hypotheses died here, and the third changed shape.** That §140's fix had
moved the first row was checked by running, not reasoned: the same test file on
`d67f958e`, the commit before `33ea2c0a`, passes with numbers identical to the
last digit. §140's fix changed how two lines are handed to `two_lines_told_apart`
and this site loses them earlier, in the `out` the caller works out first. And
the shape is not "radii at very different scales" — site 3 refuses at `1e-3`
against `1.0005e-3`, site 2 at two lengths both about one. Only site 1 has a wide
ratio at all, so **the radius ratio is not the lever**, and that much is clean.

**What the lever *is* does not generalise, and this paragraph claimed it did.**
For sites 1 and 3 it is distance from the origin, and each has a control in the
test file: the same plane and pipe, and the same ball and pipe, moved to the
origin, are both named. Site 2 has no such control and could not have one —
**its refusing pair is already at the origin**, a thin pipe through it against a
fat axis one away. Site 2 is a standing counterexample to the general sentence,
and it was caught by an audit asking what the evidence covered rather than
whether it was true.

**Where the fix comes from is not the sibling.** §142.3 guessed site 3 would be
fixed by copying the exact pre-summation at `intersect.rs:1303-1318`; the
polynomials differ, and it is not needed. `how_far_off_an_axis_exactly`
(`intersect.rs:851`) already computes exactly the `A` that site 3 loses, and
`where_a_ball_and_a_pipe_cross` **already calls it** one question earlier to
decide the centre is on the axis. `how_far_apart_two_loops_are` then recomputes
`A` in intervals and throws the exact answer away. Site 1 has that same shape.

**Site 2 does not**, and what is different about it is narrower than this
paragraph first claimed. Its sign is decided exactly; what `clear` wants is the
*size* of `r_fat² − reach²`. Sites 1 and 3 are "call the exact routine this file
already has"; site 2 is not, and §142.9 is the answer for it.

### 142.9 A magnitude where only a sign was kept: rationalise, don't factor

*Site 2's answer, and the rule behind it. The family is **four** sites, not
three — the sweep turned up one nobody had named. Factoring the difference does
not work and the reason is worth keeping. The fix is to rationalise it, so the
expansion the kernel already builds becomes a numerator that cannot cancel.*

**The family is four.** A paren-balanced sweep of `crates/rcad-geom/src/` for a
`root_enclosing` or `quotient_or_everything` whose whole argument holds a binary
minus gives 8 hits, of which four are this shape — a magnitude of a cancelling
difference whose *sign* a predicate settles: `intersect.rs:436` (§142.8's site
1), `:1193` (site 2), `:680-685` (site 3, which the syntactic sweep misses
because the difference is bound to a `let` first), and **`:1012`, the `across`
term in `two_lines_between_two_pipes`, which nothing has measured**. That last
is the same `T² + (r₁²−r₂²)K` expansion family and is new work.

**Factoring alone is refuted, and the refutation is the useful part.** The
obvious move is `r_fat² − reach² = (r_fat − reach)(r_fat + reach)`, the first
factor being what the caller already certified positive. It fails on the real
code's own printed numbers: at the refusing width `reach.hi()` and `fat_radius`
are **the same `f64`**, so `fat_radius − reach` is `[0, 2.220446049250313e-16]`
with its low end exactly zero, and the product's is too. `reach` encloses a
number that is not representable, and its irreducible one-ulp width at magnitude
one, `2.22e-16`, is *larger than the true answer* `1.92e-16`. **No rearrangement
of `f64` interval arithmetic reaches a positive lower bound from `reach`** —
§16.10's "name the family of fixes and say why it is the only one", discharged
against the family the obvious move belongs to.

**Rationalising does work.** With `S = r_fat − r_thin`, `δ = |T|/√K`:

```text
m  =  S − |T|/√K  =  (S²K − T²) / (K·S + |T|·√K)
```

The single root is gone from the numerator, which is the expansion
`whether_two_radii_out_reach_a_gap_exactly` (`intersect.rs:1495`) already
builds; the denominator is two positive terms with nothing to cancel. Then
`clear = √(m · (r_fat + reach))`, and `r_fat + reach ≈ 2` is well conditioned.

**What makes it complete is a dichotomy, not the formula.** `ask` runs the
filter first and only builds the expansion when the filter straddles zero. The
filter's interval is the same three numbers as `m` in a different association
order. So where the filter decided `Pos`, `m`'s interval is already positive and
the cheap form suffices; where it did not, the exact numerator exists by
construction. Each arm alone has a hole; together they have none, and nothing
new is computed in the common case.

**Modelled, not measured, and the distinction is kept.** Over 423 synthetic
pairs sitting on the two-loops boundary, a rational model of the routes gives a
positive lower bound on 185/423 today, 240/423 factored, and **423/423**
rationalised. That is a `Fraction` model with one-ulp outward rounding, which is
*wider* than the code's directed rounding — so a positive low end in the model
implies one in the code, and the first two figures understate the real kernel by
a couple of ulps. **No kernel measurement stands behind the 423/423 yet**; the
lane that builds this owes it.

**What would break it, and why none of it corrupts.** `Exact::times`'s subnormal
floor can refuse the degree-six numerator — then fall back to today's formula,
as `the_size_of_the_discriminant` (`three_surfaces.rs:1799`) already does, so no
regression. Overflow in `K·S` or `|T|√K` widens the quotient to everything,
which is today's refusal and not a wrong answer. `S > 0` is guaranteed by the
`ask` that got the caller here; a future caller arriving without it gets a
straddling denominator, so `quotient_or_everything` returns everything and the
kernel refuses. Worth an assert, never a wrong answer.

**The rule, one sentence.** When a predicate needs a magnitude and the exact
level only certifies a sign, rationalise the difference so the exact expansion
becomes the numerator of a quotient that cannot cancel, and hand it back through
`Exact::enclosure` — **the kernel needs no new "magnitude from a sign" facility;
it needs the routine that already built the expansion to stop wrapping it in
`Exact::of(±1.0)`.**

*The sentence that stood here said "no exact level in this kernel hands a size
back". **That is false**, and `Exact::enclosure` exists to do exactly it — 38
call sites in `crates/rcad-geom/src/` outside `exact.rs`, with
`the_size_of_the_discriminant` (`three_surfaces.rs:1799`) as the finished
precedent. What is true is much smaller: one routine,
`how_two_radii_and_a_gap_compare_exactly` (`intersect.rs:1481-1486`), computes
the expansion site 2 needs and then throws it away in `Exact::of(±1.0)`. A
sentence that generalised one routine's habit into a property of the kernel is
what closed this question prematurely, and §142.9 reopened it.*

