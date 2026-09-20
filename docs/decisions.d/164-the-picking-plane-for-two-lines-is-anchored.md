## 164. The picking plane for two lines is anchored between them, not on an axis — §142.9's fourth site fixed, 2026-09-19

*§142.9 named a fourth site nothing had measured: the `across` term in
`two_lines_between_two_pipes`, whose doc comment published a `2²⁶` bound. The
bound was about a line of code that is not there. Measuring it found the kernel
giving up **thirty million times sooner**, at a radius ratio as low as `1.32`, for
a reason that is not the `across` arithmetic at all: the picking plane leans, and
it was anchored on the first pipe's axis. Anchoring it between the two lines
removes the term rather than bounding it. Every one of thirty directions is now
told apart at every ratio up to 100, both ways round, where nineteen of thirty
refused.*

### 164.1 What the published bound was right about

`X = √(r₁² − across²)` really does cancel. For a thin pipe whose axis lies on a
fat one's skin, `across = r₁ − r₂²/(2r₁)`, so `r₁² − across²` is `r₂²` reached by
subtracting two numbers of size `r₁²`, and one `f64` step at `r₁²` grows to the
whole of `r₂²` at exactly `r₁ = 2²⁶·r₂`. That arithmetic is checked and kept.

**`two_lines_between_two_pipes` does not do that subtraction.** It takes `X` from
the discriminant, which the rest of the same doc comment says. So `2²⁶` bounded a
line of code that is not there, and nothing about the kernel's behaviour followed
from it. The pair the sentence describes is named at `2²⁶` and on past `2⁵⁰`.

This is the §15.6 failure mode worth naming: a published bound that is *correct
arithmetic about a rejected alternative* reads as a bound on the code, and only a
test can tell the two apart.

### 164.2 What was really deciding it

The picking normal. The routine asks [`HalfSpace::through_a_point_square_to`] for
one along `along × towards`, which separates the two lines perfectly — but that
vector is only **exactly** square to `along` when the pipes run along a coordinate
direction. Everywhere else one of three fallbacks is taken, and the best of those
is bounded at 55° off the direction asked for by its own doc comment.

The two lines sit at `(across, ±X)` in the plane square to the pipes. A plane
anchored on the first pipe's axis, leaning `θ` off the separating direction, holds
both of them on one side once

```text
across·sin θ  ≥  X·cos θ        i.e.   across/X  ≥  cot θ
```

For pipes along `[0.3, 0.5, 0.7]`, `θ` is `26.24°` and `cot θ` is `2.0290`, giving
a threshold ratio of `2.2029765594779596` — measured by bisection, and bracketed
by a test at `2.20` and `2.21`. Over ten pipe directions against three directions
for the step between the axes, **nineteen of thirty refused somewhere under a
ratio of 100, the lowest at `1.3225450275326378`**, and three never refused at
all. Pipes along `z` were named throughout. **The lever was which way the pipes
run, not how the radii compare** — which is why a sweep over radius ratios alone
would never have found it.

### 164.3 A gap and not a limit, in §150's family

At the refusing ratio the two lines read `−1.7507` and `−0.0033` against the
picking normal, both under a plane whose own reading is exactly zero. **Every
`f64` between those two readings separates them**, and the window is `1.7474` wide
on pipes a couple of units across, against enclosures a rounding across. The
margin the kernel was short of is `3.3e-3` — ten trillion times any rounding in
play. Nothing was cancelling; one stored `f64` was the whole of what could not
reach the answer, which is §150's shape exactly.

Three families of fix were weighed, per §16.10:

- **Reorder or rescale the arithmetic.** Refuted by those numbers, and by the
  axis-aligned control deciding at `2⁵⁰` where every enclosure is at its widest.
- **Let the normal lean, and bound the lean.** Refused by `two_lines_told_apart`'s
  own doc comment, and rightly: a line is endless, so a normal off square by any
  rounding reads a different number far enough along it and the half-space holds
  nothing. This is the one that looks like the fix and is not.
- **Move the anchor.** Keep the exactly-square normal and pass the plane through
  the point midway between the lines. This is the one that works.

### 164.4 The fix, and why it is right rather than merely effective

`between_the_two_lines` returns `here + towards · (δ² + r₁² − r₂²) / 2δ²`, which
is `across/δ` of the way along the step, so the `δ` cancels and no square root is
taken. `two_lines_told_apart` then computes `off_the_picker + leaning·how_far`
with the two halves cancelling by construction instead of by luck.

**Where the plane sits between the lines is a choice and not a decision**, in the
same sense as which of three fallback normals gets picked. Whatever the anchor,
`two_lines_told_apart` reads the offset that was actually stored and either proves
the two lines straddle it or refuses. A worse anchor costs refusals and never a
wrong answer — which is why the helper is plain `f64` with no enclosure in it, and
why the cancelling numerator inside it does not matter. It moves the anchor, not
the answer.

What is left of the old term is this anchor's own rounding: an ulp of `across`
against an `X` of about `r₂`. So the radius ratio has to reach about `2⁵²` before
it bites again, where it used to bite at `1.32`.

### 164.5 Measured after

Nowhere under a radius ratio of 100, for any of the thirty directions, either way
round. The first refusal is at `2⁵⁰` — fifteen decimal orders further out than
`1.32`, and past the `2²⁶` the doc comment published.

**Past `2⁵⁴` the pairs stop being the pairs that were asked for**, and this is not
a kernel refusal. The thin pipe's axis is built at `fat` along a unit step; once
one `f64` step at `fat` grows past the thin radius, the rounded point is no longer
on the fat pipe's skin and the pipes genuinely miss. Both orders agree on
`SurfaceMeet::Nothing` there, which is the honest answer to the numbers handed
over. The demonstration that this is the test pair's rounding and not the kernel's
is the axis-aligned control, whose step is exact and which keeps being named to
`2⁷⁰`.

A refusal rate moved **down**, and nothing else moved: `just gate` is green on the
merged tree at 2093 tests, and `RCAD_SOAK_FACTOR=100` over
`every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly`,
`the_same_pair_turned_names_the_same_kind_of_crossing`,
`every_cone_or_torus_pair_the_fuzzer_draws_is_named_honestly` and
`the_degeneracies_a_crossing_has_to_answer` passes.

### 164.6 One test changed its claim

`the_pair_at_1e16_that_answers_one_way_round`'s picker test asserted the stored
offset was **exactly zero**, because §150's pair has its first axis at the origin
and a dot through the origin rounds nothing. That is no longer the reason that
half of the pair is right: the anchor is now a short step off the origin, and what
makes it right is that the step is ordinary-sized whatever the second axis is
doing `1e16` away. The test now pins the offset against the two lines' own
readings averaged, which it matches to within one rounding — a stronger claim than
the zero it replaced, and one that says what the offset is *for*.

§150's own fix is untouched and still exercised: the plane built through the
far-out second axis is still the one that separates where the plain dot's did not.

Write set: `two_lines_between_two_pipes` and the new `between_the_two_lines` in
`crates/rcad-geom/src/intersect.rs`;
`crates/rcad-geom/tests/the_across_term_that_nothing_has_measured.rs` renamed to
`where_two_side_by_side_pipes_stop_being_told_apart.rs` and rewritten around the
fix; and one test in
`crates/rcad-geom/tests/the_pair_at_1e16_that_answers_one_way_round.rs`.

---
