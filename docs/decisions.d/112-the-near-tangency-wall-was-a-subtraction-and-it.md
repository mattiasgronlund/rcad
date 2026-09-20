## 112. The near-tangency wall was a subtraction, and it was a gap, fixed 2026-09-17

### 112.1 The measurement, at the one point three lanes have been pointed at

§110.2 left one question: the marcher's reachable near-tangency refusal is
`settled`'s, raised where `1 − c·c` is exactly zero — **and was it right to give
up?** §16.10 wants a demonstration either way. This is it, and the answer is
**gap**: the number `settled` needed was representable the whole time and the
spelling it was written in could not reach it.

The point is the one §109.6 and §110.2 both stopped at. A ball one rounding of
radius bigger than the pipe it is centred on, leaning `2.1e-8`; the seed settles
exactly; `one_step` is asked for a step of `0.01`; and the first trial point is
`here + tangent·0.01`. There, printed by a probe in `march.rs` and reproduced by
the assertion that replaced it:

```text
step   1.00e-2   s1 4.99988e-5  s2 4.99988e-5   c − 1 = 0.00000e0
                 1 − c·c = 0.00000e0     |u×v|² = 4.44045e-16
step   5.00e-3   c − 1 = −2.22045e-16
                 1 − c·c = 4.44089e-16   |u×v|² = 4.44078e-16
step   1.00e-6   c − 1 = −3.33067e-16
                 1 − c·c = 6.66134e-16   |u×v|² = 4.44089e-16
```

Three things in that table, and each is a separate claim:

- **`c` rounds to exactly one at the first trial.** The dot product of two unit
  normals leaning `2.1e-8` apart is `1 − 2.2e-16` in exact arithmetic, one
  rounding below one, and the rounding of the dot product itself is enough to
  take it the rest of the way. So `1 − c·c` is exactly zero, `across <= 0.0`
  holds, and the settling refuses. **This confirms §110.2's "on its first
  trial"** rather than asserting it: the first trial's own arithmetic is what is
  shown above, and `settled` was propagated out of `one_step` with a `?`, so no
  second shortening could exist. §110.2 is earned, not corrected.
- **The truth is `4.41e-16`** — `sin²` of the lean the test measures — and
  `|u×v|²` returns it to three digits where the subtraction returns zero. §109.6
  recorded this as an untested hypothesis. It is now tested.
- **A refusal was not even the worst of it.** See §112.2, which is the finding
  this unit did not go looking for.

### 112.2 The refusal was the half that announced itself

Everything above and everything below is about a refusal, because a refusal is
what three lanes were pointed at. **The more serious half of this defect was a
wrong answer**, and nothing in the tree was reporting it.

Read the third row of the table again. At a step of `1e-6` — a lean *less*
parallel than the one that refuses, and squarely inside the range the marcher
walks every day — `1 − c·c` returns `6.66134e-16` where the truth is
`4.44089e-16`. That is **fifty per cent wrong**, in the denominator of both
Newton corrections, and it does not refuse: it hands back a number, the settling
uses it, and the point moves by two thirds of what it should have.

Nothing caught that, and the reason is worth keeping. `1 − c·c` reaching exactly
zero is self-announcing — it trips `across <= 0.0` and becomes an honest refusal,
which is why it is the thing that got looked at. `1 − c·c` being merely half
wrong is silent, and the band where it is silent is **wider** than the band where
it refuses: the subtraction is losing digits from the moment `c` gets within a
few roundings of one, and only falls off the end some way further in. A unit
framed around "why does this refuse" would have measured the wall, fixed nothing,
and left the wrong answers either side of it in place.

The repair is the same one — the cancellation is the same cancellation — so this
section records what the fix is worth rather than asking for another one. What it
changes is which sentence goes at the top: the marcher was not only giving up
where it should not, it was quietly mis-settling points where it did not give up.

Roadmap §6.3 calls near-tangential contact the place kernels produce silent
garbage, and `march.rs`'s own module comment says the certified deviation is what
stops the silence. It does still stop it — the deviation bound is worked out with
enclosing arithmetic and never touched this subtraction. But the *stepping* was
producing exactly the silent garbage §6.3 names, eight powers of ten above where
anyone was looking for it, and what would have caught it was measuring the
arithmetic against a closed form rather than watching for an `Err`.

### 112.3 Two subtractions, and neither of them had to be written that way

The Newton correction is `(s₁ − s₂c)/(1 − c²)` along the first normal and
`(s₂ − s₁c)/(1 − c²)` along the second. **Both halves cancel**, and mending the
denominator alone is not enough: with `c` rounded to one and the two distances
equal, `s₁ − s₂c` is *also* exactly zero, so a repaired denominator divides zero
by something and the point sits still.

Both have a spelling that is the same quantity with the cancellation taken out:

```text
1 − c² = (1 − c)(1 + c)    1 − c = ‖u − v‖²/2    1 + c = ‖u + v‖²/2
s₁ − s₂c = (s₁ − s₂) + s₂(1 − c)
```

The two halves of the denominator are **sums of squares**, which cannot cancel at
all. The numerator's subtraction becomes `s₁ − s₂`, the gap between the two
surfaces at the point, which is what the correction is actually made of.

**The family of fixes, and why this member.** §16.10 asks for the family rather
than a sentence, so here is what was tried and what each is worth:

- **A different ordering of the same subtraction** — worth nothing. Both operands
  round to one before any ordering can matter.
- **Scaling.** Ruled out by measurement already, and the measurement is in the
  tree: `scaling_both_normals_by_a_power_of_two_leaves_the_same_bits_behind`
  shows an exact rescale moves only the exponent, and
  `scaling_by_something_that_is_not_a_power_of_two_trades_refusals_for_wrong_answers`
  shows a scale of three rescues some orientations and loses others.
- **A compensated subtraction** — a fused multiply-add for `1 − c·c`. This
  recovers the correctly rounded value of `1 − c·c` *for the `c` it was given*,
  and `c` is the thing that has already lost the information. The same argument
  is already demonstrated one site over, for the cross of the two normals, in
  `taking_the_cancellation_out_of_the_cross_product_buys_nothing_against_the_truth`.
- **An algebraically different form that never forms `c` at all** — which is this
  one, and is why it works. `‖u − v‖²` is built from differences of components
  that are *not* near-equal to each other, so the lean survives into the
  arithmetic instead of being rounded away before it starts.
- **Exact or interval arithmetic.** Would decide the sign, but `settled` is the
  ordinary-`f64` half of the marcher on purpose (`how_far_off_and_which_way`'s
  doc comment): what is certified is worked out again by `how_far_off_enclosing`
  once the point has stopped moving. Making the settling exact is a different and
  much larger change, and it is not needed, because a representable `f64` answer
  exists.

**The sum-and-difference form was tried and is worse, which is worth recording
because it is the obvious one.** `a + b = (s₁ + s₂)/(1 + c)` and
`a − b = (s₁ − s₂)/(1 − c)` has no cancellation anywhere in it, and it fixes the
near-tangent case exactly as well. It fails somewhere else: where the two normals
are square to each other and one distance is far larger than the other, it
reaches `u·s₁ + v·s₂` by cancelling two terms of size `s₁`, and the `s₂` part is
lost. Measured, it broke three tests the smaller form leaves green, the sharpest
being a coarse walk publishing `1.2631839496569128e6` as how far its points may
sit off a crossing drawn on a pipe `1.048576e6` across — **a bound that no longer
says where on the pipe the points are.** `1 − c² = (1 − c)(1 + c)` keeps the
original grouping and touches only the two expressions that cancel.

### 112.4 What it did

The command for every number below is `mise exec -- just t -p rcad-geom
--no-fail-fast`, except the corpus one, which is named where it is used.

- **The crossing that could not be walked is walked.** At a lean of `2.1e-8`,
  `march` draws `43` points where it returned
  `CannotFollowACrossingWhereItsSurfacesAreTangent` before. The points are
  checked against the closed form — a ball threaded on its own pipe's axis
  crosses it in two circles of the pipe's radius at `±√(R² − r²)` — and the
  furthest sits well inside `1e-7` of it, so the test is not merely asserting
  that something came back.
- **The corpus.** `RCAD_STEP_CORPUS=<corpus> mise exec -- cargo run -q -p
  rcad-step --release --example near_parallel_which_pair` selected **44** files
  on 2026-09-16 (§110.3) and selects **24** now. That example's qualifying set is
  the files whose *only* probe reason is the near-parallel crossing, so **20 of
  the 44 no longer raise that reason at all** — a 45% fall. The 24 that remain
  still stop at the same site, `SettlingAPointOntoBothSurfaces`, so what is left
  is leaner than `2.1e-8` or is a different pair; classifying those is not done
  here.
- **No test moved the other way.** 807 of 807 in `rcad-geom`.

### 112.5 The refusal that is left is the limit, and it is a different one

`settled` still refuses, and now it refuses where `which_way_the_crossing_runs`
does: when `‖u − v‖` or `‖u + v‖` is exactly zero, meaning the two normals are
**bit-for-bit the same vector** or bit-for-bit opposite. That is a lean of about
`1.1e-16`, eight powers of ten below the wall that was there this morning, and
`the_angle_at_which_a_crossing_stops_having_a_direction` already demonstrates it
is a limit and not a gap: one rounding of either normal swings the exactly
computed cross product by a quarter turn there, so the direction is not in the
two `f64` normals for any arithmetic to find.

So the two walls that §109.5 and §110 kept apart have become one wall, at the
place the earlier lanes' arithmetic said it should be all along.

### 112.6 Two claims in `one_step`, one broken and one that had rotted

The lane was briefed to audit one sentence. Both of `one_step`'s are wrong, and
the second only became wrong when the first fix landed.

**`one_step`'s `# Errors` named the wrong variant.** It said the shortening
budget running out raises `CannotFollowACrossingWhereItsSurfacesAreTangent`. It
raises `CannotFollowACrossingBecauseNoStepStaysOnIt`, which §110.1 split off this
morning; the sentence was true yesterday and stayed written down. The rest of it —
"which means the curve is not being followed rather than that the budget was
tight" — is **earned**, by
`a_step_asked_the_wrong_way_round_a_circle_is_refused_as_stepping_and_not_tangency`,
which reaches that path with a step asked backwards round a circle. What that
test's own doc comment says stands: no walk reaches it, because a walk takes its
direction from the point it is standing on.

**`one_step` treated a trial it could not settle as an answer**, propagating the
first such refusal straight out with a `?`. That is wrong for a reason that has
nothing to do with the arithmetic: `march` takes `tangent` from
`which_way_the_crossing_runs` *at `here`*, so the two surfaces are known to cross
transversally at `here` before `one_step` is called at all. A refusal at
`here + tangent·step` is a fact about the trial point, and shortening walks the
trial back towards a point that is known to be fine — which is what the function
already does for every other kind of bad trial. It now shortens, and keeps the
last refusal to hand back if all fifty shorten to nothing, so a genuinely tangent
crossing still says so and still names its site.

**And a refused trial is retried at the square root of the ask, not at half of
it.** This is the one place the fix forced a new rule rather than a repair.
Halving searches the exponent one bit at a time, and the coarse walk starts at
`f64::MAX / 4`: measured on the two pipes of
`the_same_two_pipes_with_a_fatter_thin_one_are_walked`, **49 trials refused in a
row, from `2.2471164185778946e307` down to `9.321014594689857e43`**, every one of
them still absurd. A trial that could not be settled has said nothing about the
curve — not even how long it is, which is what the chord rule reads off a short
trial — so the only thing known is that the ask was too big by an unknown number
of orders of magnitude, and the search for that is over the **exponent**. Eleven
roots take `2.25e307` to about one. It is written `step.sqrt().min(step * 0.5)`
so that it is always a shortening: below an ask of one a root would grow the
step.

That pair is worth keeping as the shape of the thing. The arithmetic fix moved
the marcher's trajectory by a few ulps, and a coarse walk that had been passing
on luck — its first trial settling to *somewhere* finite, then the chord rule
collapsing the step in one go — started landing on a pipe's own axis instead,
where the normal is `NaN`. The old code was no more robust; it was luckier. Both
halves of this section are the fragility, not the luck.

### 112.7 What happens to a normal that is not a number, measured rather than hoped

The paragraph above says a trajectory landed somewhere the normal is `NaN`. That
is a correctness question and not a colourful detail, because `NaN` propagating
quietly is neither of this repo's two answers: it is not a panic on a broken
invariant and not an `Err` on bad caller input. Every comparison against one is
`false`, so a guard written as `if x <= 0.0 { refuse }` waves it through.

**It is reachable, and that is new.** `a_normal_that_is_not_a_number_is_reported_as_two_surfaces_being_tangent`
pinned the mis-report and said explicitly that it was *not* fixed because the
lane "could not demonstrate that `march` reaches it — `settled` would have to
leave a point on a ball's centre", and §16.10 wants a demonstration for "no
caller hits this" too. It does reach it. Instrumenting `settled` to print its
iterate each round, on the two pipes of
`the_same_two_pipes_with_a_fatter_thin_one_are_walked`, the last four rounds run

```text
[3.1994025547765454e196, -2.854116061397334e206, -2.2471164185778946e307]
[-4.149515568880993e180, -2.854116061397334e206, 0.0]
[0.0, 0.0, -3.6250807591402947e105]
[0.0, 0.0, 0.0]
```

and `[0, 0, 0]` is that file's thin pipe's axis. So the settling can *land* on the
one place a normal does not exist, from an absurd trial step, and the question
"does anything hand that point back" had to be answered.

**It does not.** `nothing_hands_back_a_point_whose_normal_is_not_a_number`
(`march.rs`) puts a point on a pipe's own axis and checks all five ways in. Every
one refuses, and each names the site it refused at:

| asked | refuses at |
|---|---|
| `settled` | `SettlingAPointOntoBothSurfaces` |
| `how_far_off_the_crossing` | `BoundingHowFarOffTheCrossingAPointIs` |
| `settled_with_a_bound` | `SettlingAPointOntoBothSurfaces` |
| `which_way_the_crossing_runs` | `CrossingTheTwoNormalsForADirection` |
| `march` | `SettlingAPointOntoBothSurfaces` |

So it is a **refusal**, not a number and not a shortening — except inside
`one_step`, where §112.6's change turns `settled`'s refusal into a shortening and
a retry, which is why the gate is green rather than because the `NaN` stopped
happening. The distance to the surface at that point is an *ordinary number*
(`-1.0` for a unit pipe); only the direction is not. A guard that watched the
distance would have missed all of this, which is why the test asserts that too.

**One thing reading the source gets wrong, and the test corrects.**
`how_far_off_the_crossing` looks like it should slip past its first guard, since
`leaning.lo() <= 0.0` is `false` for a `NaN`, and be caught only by
`!reach.hi().is_finite()` further down. It is not: it refuses at the sine guard.
The enclosing arithmetic never hands that comparison a `NaN` to be fooled by — the
interval layer is doing work the plain `f64` layer above it could not. That is
recorded because it was predicted wrongly first, and checked second.

**What is still open.** The test covers the family by the *place* — a pipe's
axis, a ball's centre — and not by the *route*. It does not demonstrate that
`settled` landing on such a point always ends in a refusal for every pair and
every trial length; it demonstrates that the five entry points refuse when handed
one, and that `one_step` shortens rather than dying when a trial produces one. A
`NaN` arriving through some arm of `how_far_off_and_which_way` this test does not
build — the cone's and the torus's both divide by a `reach` that can be zero — is
untested. **That is the honest boundary of what this lane characterised**, and a
lane that wants to close it should go at `how_far_off_and_which_way` and ask which
of its five arms can produce a non-finite normal at a finite point, rather than at
`march`.

The mis-report the older test pins is also still a mis-report: all five of these
say `CannotFollowACrossingWhereItsSurfacesAreTangent`, and a pipe's axis is not a
tangency. Fixing that wants a name for the other case, which is `error.rs` and
another lane's file.

