## 29. What M3's second stage cost, recorded 2026-09-01

*The two rows whose crossing is a degree-four space curve, plus the marcher that
draws what no conic stands in for. §28 is the model for the shape. Longer than
§28 is, because §24.17 asks this stage to make the milestone's largest claim
stand up and a claim of that size does not fit in a paragraph.*

### 29.1 The pair table is finished, and one of the six rows is finished twice

| Pair | What it answers | Where |
|---|---|---|
| plane ∩ plane, plane ∩ ball, plane ∩ pipe, ball ∩ ball | named exactly | stage one |
| ball ∩ pipe | **named exactly, in all five shapes**; the quartic ones marched | here |
| pipe ∩ pipe, axes along one direction | **named exactly** | here |
| pipe ∩ pipe, axes leaning | **named as far as an enclosure reaches** | here |

The ball-and-pipe row came out stronger than §24.5 wrote it. That section files
it as *"special cases named; the rest marched"*, and the special case it had in
mind is the ball threaded on the axis, which cuts two circles. What is actually
true is that **every** ball-and-pipe crossing is named exactly, quartic or not:
§27.1's eighth-degree discriminant settles which of five shapes it is, and the
marcher is needed only to *draw* the two that do not lie in a plane. Naming and
drawing came apart exactly where §24.3 said they would.

The five shapes, and the one that has no name:

```text
A = ‖(c−o)×w‖²    V = A + r² − W·R²    Δ = V² − 4r²A
```

`W(R² − (ρ∓a)²) = −V ± 2r√A`, so the pair `(sign V, sign Δ)` settles both
tangency discriminants without ever taking the root that stands between them.
Nothing, one touching point, one closed loop, two closed loops — and, where the
two loops touch, **one loop that runs through itself**, which
`EdgeRun` has no variant for and this stage refuses by name.

### 29.2 Two leaning pipes, and where the exact level stops

Two pipes whose axes lean against each other come down to one clean statement.
Walk the thinner pipe's cross-section round by an angle `θ` and ask how far
along its own axis the other pipe reaches; that is a quadratic, and its
discriminant is

```text
sin²α · [ ρ₂² − (δ + ρ₁·cos θ)² ]
```

with `α` the angle between the axes, `δ` the distance between them and `ρᵢ` the
certified radii. **The whole shape of the crossing is where `ρ₂` sits against
`δ ± ρ₁`** — above both and the fat pipe swallows the thin one's whole
cross-section and leaves two loops; between them, one; below both, two again
with the pipes' roles the other way round; past `δ − ρ₁` on the far side,
nothing at all. Three comparisons, and each is a **length against a length**.

That is why the exact level runs out here and nowhere else in the file. Those
lengths are `rᵢ/√Wᵢ` and `|T|/√K` for polynomials `T` and `K` in the stored
numbers — three square roots in three different metrics. Clearing them costs
eight degrees, and §29.6 measures what happens then. So the comparison is
**enclosed** instead, which decides it wherever the two sides are not on a knife
edge, and the exact level runs where the two stored directions have exactly the
same squared length — which is every pair whose axes are coordinate directions,
and so the whole of criterion 5's control.

### 29.3 The marcher, and why its bound is measured rather than derived

The closed form knows its per-point bound before it starts: a count of roundings
along a chain, published as `POINT_ALONG_A_CROSSING_ROUNDOFF` and multiplied by
the size of the shapes. The marcher cannot work that way, because how far one of
its points sits off the curve is something it *finds out* by settling the point.

So it measures. After each point is settled onto both surfaces by Newton in the
plane the two normals span, an enclosure of how far it really sits from each of
them is worked out, and the two are divided by the sine of the angle between the
surfaces. That is the radius of the region a point that close to both surfaces
can be in, and the largest of them along a run is what `walk` publishes.

**Which is why near-tangency refuses rather than drifting.** As two surfaces come
parallel that sine goes to nothing and the bound goes up without limit, so the
marcher does not quietly hand back points nobody can judge — it says how far out
they could be and lets the policy decide, and past the point where the sine
cannot be told from zero it refuses by name. Roadmap §6.3 calls near-tangential
contact the place kernels produce silent garbage; that is the branch §24.1 built
this to be able to watch, and it is the branch criterion 7 measures.

The step rule is the same shape. **How far it bends** is measured and not
derived: the middle of each straight piece is settled onto the curve and how far
it moved is that piece's sagitta, and a step that spends more than three
quarters of the deflection on it is halved and tried again. **How far it turns**
is capped at a quarter turn, because past a half turn a step cannot be told from
one the other way. Termination: every accepted step moves the point at least one
step along a curve of finite length, so a run takes finitely many — and because
that argument is about the curve rather than about the arithmetic, it is not
left to stand alone. `MOST_POINTS_ALONG_ONE_CROSSING` caps the count outright
and reaching it is a refusal rather than a truncated answer (§24.10).

### 29.4 Criterion 7, the load-bearing one: 0.577, and the same at every scale

§24.5 promised the oracle would exist by construction, and it does. A ball
threaded on a pipe's axis **is a ball and a pipe**, so the evaluator marches it
like any other degree-four crossing and nothing in it knows the answer is
special — and the answer is two circles of the pipe's own certified radius at
`±√(R² − ρ²)` along the axis, which is a formula anybody can write down. The
marcher is therefore differenced against a written-down answer and not against
another of our own implementations.

| | worst share of its own published bound |
|---|---|
| marched, drawn where they stand | **0.577** |
| marched, written at `2^k` for `k` from −200 to 200 | **0.577** |
| the same circles parametrized instead | 0.008 |

**Criterion 7's ceiling is three quarters**, recorded here and asserted in
`how_far_the_marcher_strays_from_a_crossing_that_is_a_circle.rs`. One would be
the point at which the published bound stops being a bound.

The gap between the two arms is not a weakness and it is worth saying why, or
the next person will try to close it. The parametrized arm publishes `256·ε`
against a derivation of about `34·ε`, so a drawn case uses under a hundredth of
it. The marcher publishes what it measured, and what it measured is
`(d₁ + d₂)/sin ψ` where the true distance to the crossing is
`√(d₁² + d₂² − 2d₁d₂cos ψ)/sin ψ`. For two surfaces meeting square on with equal
residuals those differ by exactly `1/√2 = 0.707`, and the measured 0.577 sits
just under it. **A ratio near 0.58 is the bound being nearly tight. A ratio near
zero would have meant the marcher was publishing a number it had not earned.**

That the figure is *identical* across four hundred binary orders of magnitude is
§16.8's argument for a ratio arriving as a measurement.

### 29.5 The numbers, family by family

Criterion 5's control and criterion 6's headline, on 2048 pairs per family.
Reported apart from stage one's four planar pairs for §17.7's reason: the two
kinds of pair refuse for reasons the other does not have.

| Family | `Undecidable` | no half-space | runs through itself | threshold |
|---|---|---|---|---|
| lattice, planar pairs — the control | 0.00% | 0.00% | 0.00% | zero, asserted |
| lattice, quartic pairs — **the control** | **0.00%** | **0.00%** | 4.25% | zero, asserted |
| turned, planar pairs | 0.00% | 0.44% | 0.00% | zero / 2% |
| turned, quartic pairs | **4.64%** | **10.93%** | 0.00% | 8% / 15% |
| leaning | 0.00% | 0.24% | 0.00% | zero / 2% |
| **the band** — drawn tangent, moved a few last bits off it | **0.00%** | 10.79% | 1.07% | 5% |
| written at `2^k` | 10.49% | 3.95% | 0.00% | 25% |

Criterion 3, over about a hundred and thirty thousand points:

| Family | furthest a point sat off its own curve |
|---|---|
| lattice / leaning / turned, parametrized | 0.006 / 0.008 / 0.012 of the bound |
| lattice / turned, quartic pairs, **marched** | 0.738 / 0.355 of the bound |
| the band | 0.410 of the bound |

Three things in those tables are worth reading twice.

**The quartic control is 0% and it did not have to be.** Two leaning pipes are
the one pair whose comparison is not a polynomial, and on the lattice the exact
level still reaches every one of them — because there both stored directions
have the same squared length and the awkward root is rational.

**4.64% of turned quartic pairs refuse, and every one of them is a knife edge
the turn made.** The generator draws radii up to five and places up to six apart
in whole numbers, so a lattice pipe pair is exactly tangent surprisingly often —
five apart with radii of two and three is one — and §28.6 already recorded that
turning such a pair leaves it genuinely no longer tangent, by a few last bits.
`every_turned_quartic_crossing_that_refuses_was_exactly_tangent_before_the_turn`
is the demonstration and not the paragraph: all 95 of them are settled by the
same code before the turn. (That property is **retired at §119**, together with
the refusal it measured: the turned quartic family refuses no signs at all now,
so the name above is history and not a test you can run.)

**The band refuses nothing for want of a sign, which was not the expectation.**
§29.7 says what closed it.

### 29.6 §24.17's structural claim, and what it turns out to be

§24.17 asks for the largest of M3's limit-or-gap claims to be made to stand up:
*"the general quartic has no exact naming"*. It is not one claim. It is three,
they have three different answers, and only one of them is a limit.

**One: naming which of the five shapes a degree-four crossing takes.** Not a
limit at all — §29.1 does it exactly for a ball and a pipe, at the eighth degree,
and §29.2 does it for two pipes wherever the arithmetic reaches. §24.17 expected
this to be the hard part and it was the easy one.

**Two: reaching that answer for two leaning pipes at any scale.** A **gap**, and
this is the one with a measurement behind it. The comparison needs
`M = r₁²W₂K + r₂²W₁K − T²W₁W₂` squared. Measured on two perfectly ordinary
turned directions and an offset of order one: `M` comes to **22 entries spanning
505 binary orders**, and squaring it asks for the product of its smallest entry
with itself, which lands a thousand orders down — past the bottom of `f64`, so
`Exact::times` declines and level 3 cannot run.

That is a different kind of refusal from every other one in this milestone, and
the difference is the point. §28.2's range refusals are cured by a rescale
because the *whole number* is too small; here the number is fine and its own
internal spread is too wide, and a spread is invariant under scaling. What a
scaling can still do is slide the doubled spread into the window `f64` leaves,
and `the_question_two_leaning_pipes_ask_is_too_wide_to_square_where_it_stands`
shows it doing exactly that: the same expansion written `2^300` higher squares
without complaint. So the answer exists, this arithmetic at this scale could not
reach it, and the fix is the working frame §24.8 defers for want of a figure to
beat. **A gap, and now the first thing with a figure asking for that frame.**

Weighed and put down, with the reason each was put down:

- **The pencil's characteristic polynomial and the Segre classification.**
  `det(λM₁ + μM₂)` for two quadric matrices is a degree-four binary form whose
  real root structure classifies the intersection, and its coefficients *are*
  polynomials in the stored numbers. Put down because reading a real
  intersection type off it is the Dupont–Lazard–Lazard–Petitjean machinery, and
  because the determinant of a 4×4 whose entries are already degree four is
  degree sixteen before anything is compared — the same wall as above, reached
  by a longer road.
- **Sturm chains on the trigonometric discriminant.** §29.2's `Dsc(θ)` is a
  trigonometric polynomial of degree two, so counting its sign changes is
  counting the real roots of a quartic in `tan(θ/2)`, which is textbook. Put
  down because writing that quartic down needs an orthonormal frame across the
  pipe, and `Cylinder::basis` normalizes — so its coefficients are not
  polynomials in the stored numbers and §27.1's load-bearing rule would be
  broken to get them. This one is worth remembering: it is not that the algebra
  is hard, it is that the frame is not exact.
- **An exact algebraic representation of the curve.** Would make the naming
  question trivial and the *evaluation* question no easier, because a mesher
  still needs points and points still round. §24.3 already refuses to carry a
  curve beside the surfaces that name it, and this would be that with more
  degrees.
- **The Steiner special cases.** Two of the three are built — coaxial and
  parallel axes are §29.1's row. The third is not, and see below.

**Three: a crossing that runs through itself.** Two equal pipes whose axes meet
leave two ellipses that cross each other twice; a ball reaching exactly across a
pipe's whole width leaves two loops that pinch together at one point. Both come
back `Error::ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo`, and classifying
that refusal took two goes, because the obvious answer is wrong.

The obvious answer is that `EdgeRun` cannot say it. That is **false**, and saying
so and moving on would have been the paragraph §24.17 warns against. The two
loops of a pinch are exactly `{n̂·(p − c) ≥ 0}` and its opposite for the plane
through the ball's centre square to the axis, which is one `picked_by` each; the
two ellipses are exactly the parts lying *in* a bisector plane, which is a pair
of opposite half-spaces, and `picked_by` takes as many as it is given. The
vocabulary is sufficient. What is not reachable is writing either plane down —
the same `Plane`-stores-a-unit-normal gap §29.8 is about, and on the lattice,
where those directions are coordinate axes, even that is reachable.

**The real reason to refuse is the evaluator, and it is demonstrable.** At the
point where a crossing runs through itself the two surfaces are *tangent*, so the
crossing has no direction there and the marcher has nothing to follow:
`marching_from_where_a_crossing_runs_through_itself_has_no_direction_to_follow`
is that, in four lines. §24.10 says a partly undecidable answer refuses whole,
and naming two pieces the evaluator cannot draw is that shape exactly — so the
refusal happens where it can still be one refusal instead of two half-answers.

So: a **limit** of the naming half taken together with the evaluator, and a
**gap** of the naming half on its own. The family of fixes is a marcher that can
be told to start away from a singular point and stop at it, plus a third
`EdgeRun` variant that hands a fuse the crossing point rather than making it
notice. Both have one consumer and it is M4's fuse.

### 29.7 The expansion buffer cost almost nothing, and the useful thing was elsewhere

§27.1 filed a schedule risk here: the eighth- and twelfth-degree discriminants
need an expansion squared, `square_of_expansion` caps at eight entries, and a
pipe's `‖(c−o)×w‖²` runs to 384 with a worst case around 2.6 MB. §28.7 then
landed the fix a degree early — `Exact` is heap-backed — and left this stage
with nothing to build.

Measured, so that the risk is closed with a number rather than a shrug. For a
ball against a pipe at ordinary sizes: `A` is **8 entries**, `V` is **5**, `V²`
is **16**, `r²A` is **4**. The worst case a fixed buffer would have had to hold
is thousands; the drawn case is tens, because zero elimination keeps expansions
short whenever the terms are of similar size — which §27.1 predicted and nobody
had checked. **The buffer cost one allocation on a level that runs on a minority
of calls, and no refusals at all.**

What did turn out to matter was the opposite move: `Exact::enclosure`, which
sums an expansion's entries in interval arithmetic instead of taking the nearest
`f64`. An expansion's entries do not overlap, so enclosing it costs a rounding
or two **of the answer**, where the same expression built out of intervals from
the start carries a rounding of the *inputs*. Two axes a last bit from parallel
cross into something whose every component cancelled away: enclosed from the
expansion, `‖w₁ × w₂‖²` is known to full relative accuracy; built from
intervals, it reaches down to zero, and the distance between the axes — which
divides by its root — comes back as everything. That one change took the band
family from refusing 0.2% to refusing none.

### 29.8 The half-space gap: half of it closed, and the other half is `Plane`'s

§28.2 left `CannotNameWhichPieceOfACrossingThisIs` with a named fix: *"testing
the separation instead of arranging it"*. Instrumented before the work, stage
one's refusals broke down as: 13 of 24 failed because the picking normal was not
exactly square to the endless direction, and 11 of 24 failed only on the two
conditions that did not have to be exact.

So the fix was applied where it works and it closed that 11. A picking plane no
longer has to be square to the cutting plane's normal, nor to pass through the
axis with nothing rounded; where the two lines sit either side of it is now a
comparison between their own separation `√(ρ² − g²)` and how far the plane
strayed. The same test, in its bounded form, is what tells two loops of a
ball-and-pipe crossing apart — there the pieces sit on a sphere of known radius,
so what could move the reading is bounded by `‖n̂ × ŵ‖·R` and the comparison is
ordinary.

**The other half stays, and it is a gap.** For a piece that runs endlessly along
a direction — two lines down a pipe, two loops running the length of a fat pipe —
the picking normal must be exactly square to that direction or it reads a
different number at either end, and testing cannot stand in for arranging when
the thing being tested is unbounded. Every `Plane` constructor scales its normal
to length one and that division rounds, so **which normals come out exactly
square to a given direction is not something a caller chooses**; on the lattice
it is a coordinate axis and comes out right, and off it, sometimes.
`two_lines_no_f64_half_space_can_tell_apart_are_refused_rather_than_guessed`
is the demonstration that it is a gap and not a limit: the same configuration
written on the lattice is named without complaint.

§28.2's first family is the fix — a plane that stores its normal unscaled — and
it is a change to a type every predicate in the kernel is certified about, so it
is M4's. Measured cost of leaving it: 0.44% of turned planar pairs, 10.9% of
turned quartic ones.

**The bounded half above is written as closed, and it is not classified.** Added
by §93, which found the same shape of arithmetic one row over and fixed it there.
`two_lines_along_a_pipe` works out the separation as `√(skin² − in_front²)` from
`ρ/√W` and `(n·o − d)/√N` — two numbers that have **already been rounded by a
division**, so the loss is in the operands before the subtraction happens and no
regrouping of the subtraction recovers it. On the lattice this costs nothing: a
plane walked in to one half-ulp of a pipe's wall, at radius 1 and at radius 1024
with a coordinate normal, names two lines every time. What refuses is the rotated
pairs, where `√N` rounds and `in_front` stops being exact — the band family's
3.12% and the turned quartic family's 2.24%.

The untried technique is §93's: stop working from normalised lengths and go back
to the stored numbers, where the same quantity is an exact polynomial,
`out² = (ρ²N − W(n·o−d)²)/(WN)`. Whether that is enough is a separate question,
because what refuses is `on_each_line` projected on the picking normal and the
other enclosures in that point may dominate — which is something to measure and
not to argue. So this is recorded as **unclassified**, not as a limit: §16.10's
rule is that the sentence "the answer is not reachable" needs a demonstration,
and nobody has written one for this half. Mattias took (a) over a lane on
2026-09-13; the lane is worth having after §91.3, whose slow seed costs the gate
63 of its 65 seconds.

### 29.9 Criterion 10: what runs in CI and what does not, said plainly

`cargo-fuzz` builds with a sanitizer that needs a nightly compiler and
`rust-toolchain.toml` pins a stable one, so **`just default` does not run the
fuzz target**. `just fuzz` does, on a compiler the repo does not pin, and
`mise.toml` now pins `cargo-fuzz` itself. Two things had to be worked around and
both are written into the recipe: `cargo-fuzz` sets `RUSTFLAGS` as an
environment variable, which shadows the `.cargo/config.toml` entry `inari` needs
to build at all (§2.4), and the shared property module is reached with `#[path]`
rather than `include!` because it opens with a module doc comment.

What runs on every commit is the same property over the same decoder:
`every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs` replays the
checked-in corpus and soaks two hundred thousand pseudo-random byte strings,
about seven hundred thousand points, in five seconds.

**Run duration: 901 seconds, 23 037 787 executions at about 25 600 a second**,
corpus grown and then minimised to 562 inputs — 530 the fuzzer kept and 32
written out by hand and named for the configuration each one is. It found two
things in its first five minutes, both real, and nothing in the fourteen minutes
after them:

1. **A conic drawn near the top of `f64`'s range published a bound of infinity**,
   and `Approximation::new` asserts a magnitude is a real amount. Now
   `Error::CannotWorkOutHowBigACrossingIs`, which is the same refusal the naming
   half has one step earlier.
2. **`where_two_balls_cut` put the circle's centre nowhere near its own
   published bound**, for two balls of almost the same size whose centres are
   almost in the same place. `D² + R₁² − R₂²` cancels to a millionth of either
   square and is then divided by a `D²` smaller still, so a relative rounding of
   the difference comes out of the division magnified by however many times `D`
   goes into the radius. Summed through `Exact` like the radius beside it, which
   is what §28.5's second finding had already done for the radius and nobody had
   done for the centre. `two_balls_of_almost_one_size_almost_in_one_place` is
   the seed, checked in.

Both are stage one's, both are in the band §28.5 named, and neither was
reachable by any generator in the suite. That is the argument for criterion 10
having been worth the trouble, and it is a measurement rather than a hope.

A third came out of the soak before the fuzzer ran: `quotient_enclosing`
asserts its divisor is away from zero, which is right for the spline evaluator
where every caller has proved it and wrong for an intersector whose divisors are
lengths a caller handed over. A direction whose squared length underflows is bad
input and not a broken invariant, so it now divides into a box that says nothing
is known and turns into a refusal (§4.1).

### 29.10 Criterion 11: the divide is 4% of a marched point, and §24.12 was wrong

Human-read, no target, and labelled as such — §24.16 said this should not need
writing a third time and this is the third time.

| | |
|---|---|
| one certified marched point | **2.58 µs** (643 µs for a 249-point run) |
| one parametrized point | 33 ns (2.3 µs for a 70-point run) |
| naming the whole crossing, drawing none of it | 1.22 µs |
| one interval divide | 11.3 ns |
| **the interval divide's share of a marched point** | **9 divides × 11.3 ns = 4%** |

So **§24.12's prediction is scored and it is wrong in both halves.** §27.4
already found the first half misaimed — neither sign predicate divides at any
level. The second half said the money would go to the divide beneath it, and
measured against the marcher's own divide, which is the one §27.4 said to score
it against, it is 4%. The money goes to settling: two Newton corrections per
trial step, each of them six dot products and a two-by-two solve, and a certified
enclosure of the deviation on top.

Marching costs about **eighty times** what parametrizing the same curve costs.
That is the price of criterion 3 holding for a curve nobody can write down, and
it is a number M4 can decide whether to spend.

### 29.11 Corrections to §24, §27 and §28

- **§24.5's ball-and-pipe row** says "special cases named; the rest marched".
  Every ball-and-pipe crossing is *named*; only two of the five shapes need
  marching to be *drawn*. See §29.1.
- **§24.5's pipe-and-pipe row**, same sentence, and here the correction goes the
  other way: the coaxial and parallel-axis special cases are named exactly, and
  the third one §24.5 lists — equal radii with crossing axes — **cannot be
  named at all**, because what it produces is one curve that runs through
  itself. See §29.6.
- **§24.5's claim that the oracle exists by construction** holds, but not for
  the reason it gives. It is the ball threaded on the axis that provides it, not
  the equal-radii pipes.
- **§24.12's prediction is wrong in both halves.** §29.10.
- **§24.17's structural claim splits three ways**, and only one third of it is a
  limit. §29.6.
- **§27.1's schedule risk did not materialise.** The expansions that had to be
  squared came to tens of entries and not thousands. §29.7.
- **§27.2's "a public `Interval` divide" was weighed and not taken**, on the
  grounds that `rcad-geom` has `quotient_enclosing` of its own. True, and it
  turned out that function's *contract* was the thing that needed changing
  rather than its visibility. §29.9.
- **§28.2's fix closes half the gap** and the other half is a limit of `Plane`
  rather than of the arithmetic. §29.8.
- **§28.5's finding is now a generator.** `a_crossing_a_hair_from_the_knife_edge`
  draws seven ways two of these surfaces can rest against each other, exactly
  tangent in whole numbers, then moves the second one a handful of its own last
  bits off it and optionally leans it by `2^-k`. Both halves are relative, for
  §17.7's reason.
- **§28.7's `Exact` is the right shape and was missing one method.** §29.7.

### 29.12 The numbers

| | at §28 | now |
|---|---|---|
| tests (`cargo nextest`) | 931 | 973 |
| suite duration | 16.9 s | 18.1 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 2 | 2 |
| fuzz corpus | — | 562 inputs |

Forty-two tests and a second and a fifth, and most of that second is the
marcher: a marched point costs eighty times a parametrized one, and three of the
measured rows walk tens of thousands of them.

### 29.13 What the closing gate inherits

1. **Criterion 8 is not done**, and §28.9 item 5 already said why: the mesher
   still walks its own edges, and swapping it over to `walk` changes how many
   points every curved mesh gets. A curved solid whose edges are *computed*
   needs that wiring plus a builder that takes two surfaces instead of an edge.
2. **Criterion 12, the claim audit**, has not run. §29.6 is written to be
   audited rather than to pass one.
3. **The two `EdgeRun` gaps are now one.** §27.3 left
   `RightRoundWhereItsFacesCross` saying "one closed curve" with a quartic's two
   loops as the counterexample; those are named as two `PartOfWhereItCrosses`
   runs, so the variant's claim is true again. What is left is the curve that
   runs through itself, and §29.6 says its consumer is M4's fuse.
4. **A working frame now has a figure asking for it**, which §24.8 said was the
   condition. It is the 505-order spread of §29.6, and the ask is specific:
   squaring the two-pipe discriminant needs the expansion slid up the range, not
   the inputs centred.
5. **`Plane` storing an unscaled normal has a measured cost of not doing it**:
   10.9% of turned quartic pairs. That is §28.2's first family with a number
   against it for the first time.
6. **`Measures::Dimensionless` is still one line**, per §28.6, and the leaning
   pipes' three comparisons are lengths, which is the right arm and the right
   name.

---

