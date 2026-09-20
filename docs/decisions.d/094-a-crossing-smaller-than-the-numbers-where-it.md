## 94. A crossing smaller than the numbers where it sits, fixed 2026-09-13

§91.3 landed a corpus seed that cost `just gate` **63 of its 65 seconds**, said
it was "§89's shape a third time and at ten times the cost", and called it the
next thing to chase. It is neither §89's shape nor a gap. The row now reads
0.069 s.

### 94.1 Where the 63 seconds went, traced rather than guessed

`intersect` is not where the time is: it names the pair in **43 µs** and comes
back with `Curves` and two runs. All of the 63 seconds is in `walk`, and in the
*coarse* walk of the two — the one the fuzz harness asks for at `f64::MAX / 4.0`
to find out how big a crossing is before asking for the real one. Each run spent
31 seconds, walked 1 048 577 points and came back
`TooManyPointsAlongOneCrossing { most: 1048576 }`.

The pair: a pipe of radius `2147482112` standing along `z` through
`x = 1191182336`, and a pipe of radius `4.3601508761683463e-106` lying along `y`
through the origin. The thin one passes clean through the fat one, so they cross
in **two closed loops, each about as wide as the thin pipe**, sitting at
`y = ±1786830731.7039194`.

Instrumenting the marcher's loop is what settled it. The accepted chord fell
geometrically — `5.91e-8`, `5.71e-8`, `3.94e-8`, `2.34e-8`, `1.36e-8`, … down to
`1e-18` by the fortieth step — so `travelled` converged on `2.0285948e-7` and
the walk converged on a point `1.32479e-7` from where it started. The
loop-closing test is `travelled > 3·taken && ‖here − start‖ ≤ taken`, and with
`taken` shrinking to nothing under a `‖here − start‖` that had stopped moving, it
could never fire again. Nothing but `most` was ever going to stop it.

**That is not §89's shape.** §89.2's walk took one enormous step and fell into an
orbit `1.0066e8` across; this one takes steps that shrink. Both end at the same
cap, which is what made them look alike from the outside, and the fix for §89 —
`SAGITTA_MAY_TAKE_OF_ITS_CHORD`, measured — moved this seed by 0.46 s.

The hole the crawl goes through is in `march`'s own termination argument, which
says "every accepted step moves the point somewhere it was not, so a curve of
finite length is walked in finitely many steps". Steps that shrink
geometrically have lengths that **sum to something finite**, so the walk
converges on a point instead of traversing a curve, and the argument does not
cover it. The doc comment always said the argument was about the curve rather
than the arithmetic and leaned on `most` for the rest; this is what that leaning
cost.

### 94.2 Why this is a limit and not a gap

§16.10: a refusal is a **limit** when the answer does not exist and a **gap**
when it does and the arithmetic could not reach it.

Each loop is `4.36e-106` wide. The numbers at `y = 1.787e9` are
`2.384185791015625e-7` apart — `(1786830731.7039194).next_up()` minus itself.
The loop is narrower than one step of the grid it lives on by **ninety-nine
orders of magnitude**, so its whole `y`-extent lies between two consecutive
`f64` values, and the point the marcher settles onto is `1.32e-6` off the thin
pipe with the arithmetic able to place it `1e-23` off. There is no answer to
reach.

And the demonstration rather than the paragraph, which §16.10 asks for: nothing
about the pair changes but the thin pipe's radius. Swept over powers of two,

| thin radius | what `walk` does |
|---|---|
| `4.36e-106` … `2⁻⁶⁰` | §94's refusal, 37 µs |
| `2⁻⁴⁰` … `2⁻⁵` | `CannotFollowACrossingWhereItsSurfacesAreTangent` |
| `2⁻⁴` = 0.0625 | **walks**, 10 points, deviation `1.64e-6` |
| `2⁻³`, `2⁰` | walks, 10 points |

At `0.0625` the loop spans about a quarter of a million grid steps rather than
`1e-99` of one, and the same call walks it. That is
`how_small_a_crossing_can_be_before_no_point_sits_on_it.rs`'s second test.

**Making the arithmetic exact would not have helped, and that is what makes this
a limit rather than a §93.** §93's cure was to go back to the stored numbers
where the quantity is an exact polynomial. Here the obstacle is not the
arithmetic but the grid: the thin pipe pins `|x|, |z|` to about `4e-106`, and at
a grid `y` the fat pipe then wants an `x` up to `1.5e-7` away. Exact distances
move the point along the fat pipe's normal by the same `1e-7` and destroy the
thin pipe's residual just as the rounded ones do.

### 94.3 Four criteria measured, three of them refuted

The whole test suite was instrumented and every `march` call reported on — **991
of them**, 986 closing their loop and 5 reaching the cap — because the question
"what separates this crawl from a walk that works" is not answerable by looking
at the crawl alone. Three plausible answers are wrong:

| criterion | worst legitimate | the crawl | separates? |
|---|---|---|---|
| step against the deviation of its own ends | `2.46e-301` | `1.27e-13` | **no** — legitimate walks are far worse |
| distance travelled against ground covered | `5.32` | `1.54` | **no** — the crawl is the tamer of the two |
| every point certifiably on both surfaces | fails at point 1 of nearly every walk | fails | **no** — too strict everywhere |
| **the settled seed's residual in its own enclosure widths** | **12.8** | **1.67e15** | **yes**, by fourteen orders |

The first is worth keeping because it is so nearly convincing: a step shorter
than the uncertainty of the two points it joins carries no information, and the
crawl's first step is already 27 times shorter than that. But two walks in the
suite close their loops with steps `1e-301` of their deviation, so the rule would
refuse walks that work. The second was measured because §89.2 pointed at it —
"stayed `1.0066e8` from start while a step covered `5.6e5`" — and it is §89's
shape and not this one: this crawl converges on a point, so it never mills
about. The third is the certified statement one wants to be able to make and it
is simply not true of a settled point: an enclosure of the distance is a few
ulps wide and whether it happens to straddle zero is a coin toss.

### 94.4 The fix, and the one constant in it

`the_settling_did_not_land_on_both`, asked of the seed once, before the first
step. The settling moves a point onto both surfaces as far as `f64` allows, and
how far that is is the width of `how_far_off_enclosing`'s own interval — the
narrowest the arithmetic at that point can make the answer. A residual of a few
widths is what a settled point looks like; a residual of a great many is a point
the settling could not move.

`WIDTHS_A_SETTLED_SEED_MAY_SIT_OFF = 1e6`, from the table above: eighty thousand
times above the 12.8 a working walk has ever needed, a billion times below the
`1.7e15` of the failure. Both margins are as large as the measurement allows.

**Asked of the seed and of nothing else**, and that is measured too: the first
*marched* point of a walk that goes on to close its loop sits up to `4.8e10`
widths off, so the same question asked of every point would refuse walks that
work. The seed is the one point the marcher has spent eight Newton rounds on
before anything else exists, and everything downstream is built on it.

**An enclosure of no width is let through.** No width says every operation was
exact, and a residual beside it would be a real distance with no scale to ask
this question against. Of the 3 333 seed checks the suite makes, **580** come
back with no width and every one of those has a residual of exactly zero.

`how_far_off_enclosing` had to stop taking the magnitude of its own answer for
this to be askable. It now returns the **signed** enclosure and
`how_far_off_the_crossing` applies `furthest_from_zero` at the one call site that
wanted it. Two callers, two different things wanted out of one interval: the
magnitude, and the width.

The refusal is `Error::ACrossingTooSmallForAnyPointToSitOnBothItsSurfaces`.
`TooManyPointsAlongOneCrossing` was a true sentence about what the marcher did
and told nobody why.

### 94.5 What moved

The corpus row of `every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly`
goes **63.2 s → 0.069 s**, which is §91.3's own 0.072 s measurement of the row
without the seed. The whole `just gate` test run is 24.9 s at 1857 tests, two of
them new.

**Not one count moved**: points 4235, orders 549, scalings 1097, the same three
numbers §93 published. The seed refused before and refuses now — it brought one
order and two scalings and no points then, and it brings the same now. So the
seed still buys no coverage, and what it earns its place with instead is that
the refusal is cheap and named: the new test fails without this change, with
`TooManyPointsAlongOneCrossing` after 35 seconds.

The new refusal is reachable from the public API and not only from a fuzzer — two
pipes and a radius ratio is all it takes — so it is a published limit like any
other, with the sweep in §94.2 behind it.

