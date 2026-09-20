## 155. §145.6's annihilation has no caller that can reach it, answered 2026-09-19

*§145.6's third item: is there a pair reaching §142.7's annihilation — a
`[0,0] * how_far` swallowing an `ENTIRE` — with no second route? The answer is
a demonstration that no caller can, per §16.10, and the hypothesis narrowed on
the way. `enclosure.rs:102` needed no change and got none.*

### 155.1 The safe-making property is narrower than §145.6 guessed

§145.6 said the second route exists whenever `leaning` is an exact zero and
`how_far` is `ENTIRE`. What actually makes the site safe is smaller:

> The swallow `[0,0] * how_far` can only be **wrong** where the true `how_far`
> is not a finite real. `how_far` is a quotient by a stored length, so that
> needs a divisor that is **truly zero** — not merely a box around zero — and a
> truly-zero divisor is exactly what forces the second route.

That is a statement about what the true quotient is, so it covers the public
API and not today's callers, which is what §16.10 asks for.

- `two_lines_along_a_pipe`: the divisor is a `Plane` normal's squared length,
  and a `Plane` has no zero normal to offer. The true quotient is always finite
  and the swallowed term is genuinely zero. §145.4's demonstration is untouched
  by §148, which rewrote `the_step_across_to_the_other_axis` in the *other*
  caller.
- `two_lines_between_two_pipes`: the divisor is `‖towards‖`, truly zero exactly
  when `towards == [0,0,0]`. Then `apart` is `[0,0]`, every `sideways` component
  is `ENTIRE`, and `dot(normal, up)` is `ENTIRE` for every picking normal the
  kernel can build — a normal exactly square to `along` cannot sit on the one
  axis where `(along × sideways)` annihilates.

### 155.2 Two routes to the annihilation, both shut at the door

Both empty `how_far` with `up` left tight *in the arithmetic*, and both are shut
by the crossing question, which must be **certified negative** before the caller
is reached.

1. **Overflowing squared radius.** `r ≥ √f64::MAX` makes `apart² + r₁² − r₂²`
   come back `[-inf, inf]` with `apart`, `sideways` and `up` exact — the
   annihilation with no second route. The same overflow empties
   `E² − 4r₁²r₂²W₁²`. Measured through the public API at radii `1e155` to
   `1e300`: all `Err(Undecidable)` at `level: Interval`. The gate stops naming
   pipes between `1e77` and `1e78`, and that boundary is pinned.
2. **Overflowing quotient.** `E²` staying inside `f64` caps `r₁²+r₂²` at
   `√f64::MAX`, and `apart` missing zero floors the step at ~`2.2e-162`. The two
   together reach `5.022696596185628e299` against `f64::MAX`'s `1.797e308` —
   eight and a half decades of room.

**The second half is a margin and not a proof for all time**, and the file says
so: if a later change raises what the crossing question can certify, that half
must be re-run. The first half need not.

### 155.3 What this session checked and what it took on the lane's word

The eight-and-a-half decades, the 156 pickers and the `1e77`/`1e78` boundary are
**asserted by tests in `a_pair_that_annihilates_with_no_second_route.rs`**, and
that file ran green in the merged tree's gate — so they are carried by a test
and not by a report. What was not independently re-derived here is the choice of
those two routes as the only two; that rests on §155.1's argument about the true
quotient, which is the part to attack if this is ever reopened.

