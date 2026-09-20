## 28. What M3's first stage cost, recorded 2026-09-01

*The four pairs whose crossing lies in a plane, named exactly, plus the
evaluator that puts certified points along one. §27 is the model for the shape:
a stage and not a milestone, so it is short. The marcher, the fuzz target, the
cost benchmark and the claim audit are the second stage's and the closing
gate's.*

### 28.1 §27.1's prediction was right, and it moves the headline to zero

§24.10 called the rotated refusal rate "the headline" and expected it above
zero. §27.1 argued, **before any of this was written**, that there is nothing
for a turned pair to be undecided about: `intersect` takes two `Surface`s, a
`Surface` is made of `f64`, every discriminant is a polynomial in those, and
level 3 decides. Measured, on 2048 pairs per family:

| Family | `Error::Undecidable` | Second refusal | Threshold recorded |
|---|---|---|---|
| lattice — the control | **0.00%** | 0.00% | zero, asserted |
| turned — the headline | **0.00%** | 0.54% | zero, asserted |
| leaning whole-number directions | 0.00% | 0.63% | zero, asserted |
| written at `2^k`, `k` from −620 to 620 | 10.50% | 0.00% | 25% |

**Criterion 6's threshold is zero**, recorded here and asserted in
`how_often_naming_a_crossing_refuses.rs`. That is a strong assertion on purpose:
if a change ever makes a turned pair refuse, the answer is to find which product
left the range and not to raise the number.

Three predictions were scored before this stage and two were right; this is the
fourth, the second made in advance, and it holds.

### 28.2 There are two refusals and §24 predicted one of them

The first is the one §27.1 named: exact arithmetic declining a product that
would round its low half onto the subnormal grid, or overflow. Measured, one
pair of each kind: every family answers from `2^-538` to `2^509`, and two planes
answer everywhere, because their discriminants multiply two direction components
or two offsets rather than a radius by itself. A **gap**, demonstrated and not
argued — all 215 refusals in the shipped corpus are named the moment the same two
surfaces are written back on the lattice.

The second was not on anyone's list. **A plane exactly along a pipe's axis and
inside it cuts two lines, and telling them apart needs a `Plane` whose unit
normal is exactly square to both the axis and the cutting plane's normal.**
`Plane` scales every normal to length one and that division rounds, so the
normal we can write down is exactly square to both only sometimes — and a *line*
is endless, so a half-space it only stays inside near the origin has not picked
it at all. `Error::CannotNameWhichPieceOfACrossingThisIs`, 0.5% of turned pairs
and none at all on the lattice, where that direction is a coordinate axis.

Also a **gap**, and the families of fixes weighed: a `Plane` that stores its
normal unscaled, which is a second kind of plane for every predicate in the
kernel to be certified about and so M4's business; a curved half-space, which
buys nothing because `Cylinder` normalizes its axis the same way; and testing
the separation instead of arranging it, which compares the picking plane's own
offset rounding against the half-distance between the two lines and is a sign
question of about the fourteenth degree. **The third is the fix and it needs the
second stage's expansion buffer**, so it lands there.

### 28.3 Two ends is the wrong shape for a crossing of two surfaces

§24.3 says a crossing comes back as "two surfaces, the half-spaces that pick one
piece, and two ends", and §24.9 gives every `NamedRun` a pair of ends. Writing
`intersect` found that **a pair of surfaces names a curve and nothing on it**.
The circle where a plane cuts a ball has no start; the line where two planes
cross has no end at either side. Ends are what *topology* adds — the vertices
that cut a curve into edges — and an intersector handed two surfaces has none to
give.

Two of the four pairs answer with an endless line, and one of those is criterion
4's own row ("a plane tangent to a cylinder gives one line"), so this is not a
corner case the type could be left silent about.

`NamedRun::the_whole_of` is the answer and it widened nothing: both ends come
back as the box that says *nothing at all is known about where this is*. That
can never be confused with an end a predicate gave up on, because §24.10 refuses
such an answer whole — an undecided end comes back as `Err` and never inside a
run. `EdgeRun` did not grow a variant and the format did not move.

### 28.4 "Certified ON" has to mean "never certified OFF"

§24.7 calls criterion 3 one sentence: *every point the evaluator produces must be
certified ON both surfaces by `SideOfSurface`*. Taken literally it is
unreachable, and not because the evaluator is weak. A point on the circle where
a plane cuts a ball is at coordinates no `f64` can name, so `Decision::Zero`
about it is not something any amount of precision reaches; handed over as an
exact triple it gets a certified `Pos` or `Neg`, which is §2.5's exact answer to
the wrong question.

The reachable form is the one
`a_point_exactly_on_a_circle_is_never_certified_off_it.rs` already argues one
milestone down, and it has two halves rather than one: handed over as **the box
it is really known to be inside**, no point may be certified *off* either
surface, and separately every point must be certified *off* a surface it is
plainly not on. The second half is what stops the first passing for an evaluator
whose bound is a mile wide.

Measured over about thirty thousand points on three families: the furthest any
point sits off its own curve is **1.2% of the bound published for it**.

### 28.5 The published bound needed two steps that are not roundings

`POINT_ALONG_A_CROSSING_ROUNDOFF` is `256 · f64::EPSILON` of the largest number
involved, against a derivation of about `34 · f64::EPSILON` — the same seven-fold
room `Surface::ROUND_TRIP_ROUNDOFF` leaves next door, and for the same reason,
that Rust does not say how accurate `sin`, `cos` and `atan2` are.

Two steps of the chain are things that had to be *put there* rather than counted,
and without either the constant is not a bound at all:

1. **A nearly-tangent circle's radius comes from a difference of two large
   squares.** `n·n · R² − (n·c − d)²` in `f64` cancels to noise, and the radius
   that comes out is wrong by about `ε·L²/radius`, which grows without bound as
   the circle shrinks. Summed through the exact expansion instead — the very
   discriminant that named the crossing — it is right to a rounding or two and
   the bound stays *relative*.
2. **`axis × normal` for a plane nearly square to a pipe is tilted out of the
   plane** by about `ε / sin`, so every point built on it misses the plane by the
   radius times that. What gets used is the part of it lying in the plane, which
   leaves a tilt of order `ε²`.

Neither would have been caught by the drawn corpus. The first needs a tangency
approached rather than hit; the second needs two directions nearly but not
exactly parallel, which the lattice family cannot draw — turning a plane and a
pipe that were exactly parallel turns them by the *same* rotation and leaves them
exactly parallel, so the corpus has the two cases and not the band between them.
That is worth writing down as a shape rather than as an incident: **a generator
built out of exact configurations and one rotation reaches the on-case and the
far-off case and skips the near-case**, and §17.7 already says a nudge of `1e-9`
is not the answer.

### 28.6 Corrections to §24 and §27

- **§24.10, criterion 9.** "The same pair under a general rotation names the
  same kind of crossing" is not true of a pair on a knife edge, and should not
  be: every predicate here is exact about the numbers *as stored* (§17.1), and
  turning rounds them, so a plane that rested exactly on a ball genuinely no
  longer does. The knife edges are named, skipped and counted. The surprising
  one is not the tangency: **a plane not parallel to an endless pipe cuts an
  ellipse however far away it is**, so "parallel and well clear of it" answers
  `Nothing`, and the smallest tilt turns that into an enormous ellipse. There the
  knife edge is the whole of the `Nothing` answer rather than a near miss. 1485
  of 2048 pairs are compared and 564 skipped, which is high because the control's
  directions are all coordinate axes.
- **§27.2's fourth "weighed and not taken".** It declines a `Measures` arm for a
  dimensionless value on the grounds that "level 3 always decides such a
  question, so no record is ever written". It can be written: a direction with a
  component down among the subnormals underflows its own product, and "is this
  plane parallel to this axis" then gives up.
  `a_direction_with_a_subnormal_component_cannot_be_told_from_square_to_a_plane`
  is the demonstration. It is filed as `Measures::Length`, whose conversion is
  the identity, which is the *right* conversion for a value with no units; the
  **name** is wrong, and `Measures::Dimensionless` is one line at M4's gate.
  The freeze held: nothing in `rcad-predicates` moved.
- **§24.9's four arms are enough.** `Nothing`, `TouchAtOnePoint`, `Curves` and
  `SameSurface` cover every answer the four planar pairs give, including one
  ball swallowed by another and a plane clear of a pipe it runs alongside. The
  `Vec` in `Curves` is never longer than two.
- **§24.5's row for plane ∩ cylinder** lists "an ellipse; a circle, one line or
  two lines in special position". The circle needs no naming of its own — a
  circle and an ellipse are both one closed curve and neither is drawn, so they
  are the same answer. What does need its own path is the *evaluator*, where a
  plane square to the axis has no `axis × normal` to take a direction from at
  all, and the pipe's own `basis` is what stands in.

### 28.7 What went into the code, and what it cost

Four error variants in `rcad-geom` beyond the gate's `Undecidable`:
`CrossingRunsToAQuartic`, `CannotNameWhichPieceOfACrossingThisIs`,
`CannotPlaceAPointThatCloseToACrossing` and `CannotWorkOutHowBigACrossingIs`.
The first is a deferral said out loud, which is not the same thing as a wrong
answer; the middle two are the two refusals of §28.2 and §28.4's policy
question; the last is the gap again, met while working out how big a conic is.

**Level 1 of the cascade is deliberately not implemented for any of the new
questions.** A forward error bound is a derived claim about a polynomial and
§15.6 says a published bound needs a test; deriving and testing one for each of
seven questions buys speed on a path nothing has measured, which is the shape
§1.4 and §24.16 both turn down. Level 2 decides everything level 1 would have,
by construction rather than by derivation. If M4 measures the intersector and
wants the filter, the derivations are the work and they are not large.

**`Exact` is a heap-backed expansion**, which is §27.1's fix (i) landing one
degree below where §27.1 assigned it. `n·n · r² − w·w · (n·o − d)²` multiplies a
six-entry expansion by a ninety-eight-entry one, and a fixed buffer for that is
over a thousand `f64` on a level that runs on a small minority of calls. The
existing `square_of_expansion` is untouched: the two predicates that call it know
their own sizes, which is exactly the case `expansion`'s crate doc describes.

### 28.8 The numbers

| | at §27 | now |
|---|---|---|
| tests (`cargo nextest`) | 878 | 931 |
| suite duration | 16.6 s | 16.9 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 2 | 2 |

The suite grew by fifty-three tests and by three tenths of a second, because a
crossing is a handful of sign decisions rather than a fuse — which is why the
measured rows here run 2048 configurations where `rcad-topo`'s run 128.

### 28.9 What the second stage inherits

1. **`Exact` is already there.** The eighth- and twelfth-degree discriminants of
   §27.1 can be written straight into it with no buffer arithmetic anywhere.
2. **The arms are in place.** `intersect` and the evaluator's
   `the_conic_between` each have a `CrossingRunsToAQuartic` arm holding the two
   quartic pairs, so the second stage fills them in rather than restructuring
   what the first shipped.
3. **`EdgeRun::RightRoundWhereItsFacesCross` still says "one closed curve"**,
   which is true of every crossing the four planar pairs produce and false of a
   quartic that leaves two loops. §27.3 named the two families of fixes and
   built neither; the second stage is the consumer that decides.
4. **The two-lines gap of §28.2** is fixed by a fourteenth-degree separation
   test, which needs the buffer the second stage is building anyway.
5. **The mesher does not call `walk` yet.** §27.5 settled who walks an edge and
   this stage built the evaluator, but swapping the mesher over changes how many
   points every curved mesh gets and moves the pole and seam handling down a
   layer. `every_curved_edge_the_kernel_builds_is_walked_into_points_on_both_its_faces`
   is the evidence that what is left is wiring: every curved edge the kernel
   builds already crosses that seam and comes back with points on both faces.
   Criterion 8 needs the wiring and a solid whose edges are computed, and both
   are the closing gate's.
6. **The marcher's own interval divide** is still the thing to measure, per
   §27.4, and `Interval::divided_by` is still `pub(crate)` in a frozen crate
   while `quotient_enclosing` in `rcad-geom` is now `pub(crate)` to that whole
   crate rather than to one module. That was the move §27.2 said would cost the
   freeze nothing, and it did not.

---

