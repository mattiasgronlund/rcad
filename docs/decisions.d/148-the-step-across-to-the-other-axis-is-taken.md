## 148. The step across to the other axis is taken scaled, and §145's crossing comes back, fixed 2026-09-19

*§145's two pipes are named in two straight runs, both ways round.
`two_lines_between_two_pipes` worked the step from one axis to the other out as
`(c − o) − w·((c − o)·w / w·w)`, which is a subtraction of two numbers that
agree to everything the answer is made of, and for a pair far enough out along
`w` it cancels exactly. The same step multiplied by `w·w` is `(w × (c − o)) × w`
— a polynomial in the stored numbers with nothing to cancel, which [`Exact`]
holds outright — and the scale comes straight back off, one division per
component. Sixteen pairs of eighty-five changed and no others did. §145.6's "do
`up` and `across` need two fixes" is **falsified**: one step serves both. So is
the move that looks obvious, fixing the magnitude alone.*

### 148.1 What was built

`the_step_across_to_the_other_axis` (`intersect.rs`), a private function
replacing the two lines that took the step in plain `f64`. It crosses twice at
the exact level — `Exact::cross_of` the gap against `w`, then that against `w`
again — and divides each component of the result by `along.length_squared()`
once at the end.

It is written as `((o − c) × w) × w` rather than `(w × (c − o)) × w`, which is
the same vector: the only cross product `Exact` offers puts the already-exact
side first, so crossing with `w` twice turns the step round, and starting from
the gap the other way round turns it back.

The plain subtraction survives as the fallback arm, for two coordinates whose
difference times `w·w` leaves `f64` altogether, or a product on the way there
that lands among the subnormals. That is what the row has always done and it is
what is left when the exact level cannot run.

### 148.2 The scale comes off rather than travelling, and why that is the choice

§145.6 floated carrying the scale downstream instead, and said what that costs:
"changes every reader of `sideways`". Rejected, and the reason is that almost no
reader cares. The picker only ranks three normals by which leans nearest
`w × step`; `sideways` and `up` are each divided by the step's own length; and
`leaning · how_far` is the step multiplied by `across` and divided by that
length again. A positive multiple cancels out of all of them.

`across = (δ² + r₁² − r₂²) / 2δ` is the one that cares, because it puts the
step's length up against two radii, and the radii are not scaled. So the scale
has to come off somewhere, and taking it off at the source costs one division
per component — a division by a number near one, which is a benign rounding and
not a cancellation — against touching every reader downstream. What is handed on
is the true step to within an ulp or two of itself however far along `w` the two
axes sit.

**Which way round the step points is not read at all**, and that was measured
rather than argued. Turning it round turns `w × step` with it, the picking
half-space leans along `w × step` and `up` *is* `w × step`, so both flip
together: the two runs come back in the other order and nothing else moves.
Taking the gap the other way round leaves all four tests in
`the_step_between_two_pipe_axes_taken_scaled.rs` green.

### 148.3 Measured

The lane swept eighteen reaches, `1e0` to `1e17`, each with the second origin at
`w` times the reach rounded onto the grid and then nudged along `z` by −2, −1,
0, +1 and +2 ulps, each pair asked both ways round. **Sixteen pairs changed and
no others did**, every one of them a pair whose plain step rounds to exactly
`[0, 0, 0]`, and every one going from `CannotNameWhichPieceOfACrossingThisIs`
both ways round to two straight runs both ways round.

**Re-run here on the merged tree** over seventeen reaches, `1e1` to `1e17`, five
nudges, eighty-five pairs: sixteen have a plain step that cancels to exactly
zero, and **all sixteen are named `Curves(2)` both ways round**. The sixteen are
the nudge-0 row of every reach except `1e11`. That the sixteen *changed* is the
lane's measurement against its own pre-fix base and is not re-run here; that all
sixteen are now named is this session's.

§143's tests and all thirteen refusal-rate tests in
`how_often_naming_a_crossing_refuses.rs` stay green, and every published rate
holds — §145.6 recorded that before the merge and the gate re-ran it. `just gate
quiet` on the merged tree: PASS, 2056 tests run, 2056 passed, 1 skipped
(§136.4's declared one), 25 doc tests.

### 148.4 What §145.6 asked, and the trap it was right to name

Three of its four open items are answered.

- **"Whether an exact `towards` is even representable for this pair."** Not as
  written — the projection is a quotient and `Exact` holds no quotients. Scaling
  by `w·w` is exactly what turns it into a polynomial, which is why the fix has
  that shape and not the `map_or_else(enclosing, Exact::enclosure)` shape §145.6
  offered as its candidate.
- **"Whether `up` and `across` both recover from one fix, or need two."** One.
  Both divide by the length of the one step, so a step that no longer vanishes
  serves both.
- **The converse was checked before any code was written**, and it is the part
  worth keeping: **fixing the magnitude alone would not have worked.**
  `sideways = [0, 0, 0] / apart` is still zero, so `up` is zero, so the
  either-side test fails anyway. Anyone reaching for `squared_gap` at
  `intersect.rs:1023` — which is the obvious first move, because that is where
  the vanished length is read — would have fixed a number nothing was waiting
  on.

### 148.5 The reproducer had to be turned round, and the lane could not do it

`the_step_between_two_pipe_axes_that_rounds_to_nothing.rs` — §145's own
reproducer, landed earlier the same day — **asserted the refusal this fix
removes**. It was not in the lane's write set, so the lane correctly left it
alone and said so; merging the branch on its own left `just gate` red by exactly
that one test, 2050 of 2051.

Its last loop now asserts two straight runs both ways round, and its module doc
stops saying "reported, not fixed". **It keeps its own subject rather than
becoming a second copy of the fix's file**: the plain subtraction still cancels
to exactly nothing, the axes still stay apart, and what makes that harmless now
is only that nothing reads it. §142.7's annihilation has no caller known to
reach it any more.

Recorded because the general case is not rare: **a lane's green gate and the
merged tree's are different questions**, and the difference is whatever the
lane's write set kept it away from.

### 148.6 What this does not close

§145.6's third item is untouched: **is there a pair that reaches the
annihilation with no second route?** This removes one *way of reaching* the
annihilation, not the annihilation. That is still the item on the list that
would cost something, and still the one that would falsify `enclosure.rs:102`
outright, because a pair that annihilated without the second route would be
certifying a sign off `[0, 0] * ENTIRE`.

§150's pair came out of this sweep and is **not** this fix's; it reads the same
before and after.

