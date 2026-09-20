## 38. The four defects §37 found, fixed, recorded 2026-09-01

Lane 8. `rcad-predicates` is at **0 files changed for the whole milestone**, ten
lanes now, and `rcad-geom` is at 0 files changed for this lane — everything here
is `rcad-topo`, six source files and two test files.

§37.4 found four defects and reported them rather than fixing them, which was
right: a metamorphic suite that edits the kernel until it goes green is worth
nothing. All four are closed. Three of them were panics on solids the public API
builds in two lines, which §16.11 says may not happen, and one was a confident
wrong answer, which §4.1 ranks below both crashing and refusing.

**Two are answered and two are refused by name**, and which of the two is right
differs per defect. The rule this lane read them by is CLAUDE.md's: a refusal is
a *limit* when the answer does not exist and a *gap* when it does and the
arithmetic could not reach it. A gap gets fixed where the fix is the arithmetic;
where the answer exists and this kernel's *representation* cannot hold it, the
gap gets an honest refusal and a demonstration of what it would take.

| §37.4 | what it was | now |
|---|---|---|
| 1. a ball nothing cuts is dropped | the arrangement of a face with no boundary and no cut has no cell to read a piece off | **answered** — the whole surface, cut open along a seam of its own |
| 2. a corner on a curved face panics | two crossings that are one curve, so no order round the node exists | **refused**, `TwoCurvesTouchWithoutCrossing`, a gap |
| 3. a blind hole panics on the way out | a closed edge's one corner read as a corner to dissolve | **answered** — the blind hole and the bubble both |
| 4. a ball cut on its polar axis panics | a closed edge starting at a pole, where no longitude says which way it runs | **refused**, `AClosedEdgeOfTheAnswerWouldStartAtAPole`, a gap |

And §37.5's one unclassified refusal, `ResultIsNotManifold { count: 1 }`, is a
**gap** and is closed with it (§38.5).

### 38.1 A ball that nothing cuts comes back as a ball

`fuse([ball], [], Union)` came back `Ok` with zero solids, no refusal, an empty
log, and a model that validated. §37.4 called it *"a wrong answer, so neither
limit nor gap — the thing both words exist to avoid"*, and it swallowed four
catalogue rows: tangent spheres, a ball in a block under intersection and
difference, and a ball resting on a block.

**The cause is one sentence.** A ball's skin is one face whose one wire is a
seam, `bake` leaves seams out at the door — and it is right to, for the reason
`seams_are_left_out` demonstrates — so the arrangement of that face against no
cuts has no curve drawn on it, no node, no arc, no cell, and therefore no piece.
Nothing was wrong with any step; the empty arrangement simply had no meaning
attached to it.

It has one now: **an arrangement with no arcs at all is the whole of its
surface**, handed back as one piece with `Piece::is_the_whole_surface` set. Three
places read that word and no fourth needs to:

- `select.rs` says such a piece holds every point of its surface. Its wire is a
  seam, and a seam is where the surface's own two coordinates were cut open and
  not where the face stops.
- `assemble.rs` writes it out with a `WireStep::Pole` between the two walks of
  the seam, which is the only thing in the model a fuse had never built and the
  only reason the fuse could not hold a closed face before.
- the commit does nothing special: an uncut closed face shares no edge with
  anything, so it is always a shell on its own, and nesting, volume and meshing
  are the paths a ball already went down.

**The seam the answer carries is not the input's, and that is the load-bearing
choice.** The answer's face is written with no placement of its own, so its two
coordinates are read off the surface standing where it is — and a ball has no
frame of its own (`Sphere::point_at`'s own doc says so), so its poles are
`centre ± (0, 0, r)` in the world's axes and its longitude runs from `+x`
whatever the input was turned by. Carrying the input's seam through a rotation
would name a meridian of some other sphere, which is §17.1's mistake — re-using a
cached sample instead of re-deriving — one surface up. The poles are built by
interval arithmetic and collapse onto an exact triple when the sum does not
round, because handing back a rounded triple as an exact one is §34.6's mistake.

#### What is not load-bearing, measured rather than argued

Two things looked like they were and are not, and saying so is §16.10's rule
about "no ordering helps" applied to its own opposite.

- **Which half of the meridian the seam is.** Both halves run pole to pole, so
  both cut the surface open along a meridian and the patch is the same rectangle
  shifted. Flipping `material_side` on the two seam sides leaves all 368 tests of
  `rcad-topo` green.
- **Where the poles are** needs no shape of its own. Putting them on the equator
  instead — still on the ball, still on the meridian plane, so every check about
  *where* they are passes — is caught by the plainest ball there is: the
  triangulator refuses a patch whose corner stands on an earlier one, and every
  ball test in the file goes red at once.

#### The shape where the error does not cancel

A ball fused **on its own** exercises none of the three things that can go wrong,
and that is worth writing down because it is the obvious test to reach for.
`place_the_piece` returns `Outside` without asking anything when there is no
other solid, so selection never looks at the piece; nothing turns it round; and
its shell is never nested. Two shapes were found by perturbing the fix and
measuring which rows went red:

- **A ball wholly inside a block, hollowed out.** Making a whole-surface piece
  never turn round leaves every other test in the crate green and fails exactly
  this one, on two solids where there should be one: the ball comes back as a
  separate positive solid instead of a void, so the block gains its volume
  instead of losing it. `a_ball_wholly_inside_a_block_answers_all_three_rules`.
  This is also the shape §38.4's bubble is measured against, and the only row in
  the crate where a closed shell's winding is told apart by a volume rather than
  by an assert.
- **A ball touching a wall at `centre + (0, r, 0)`.** A whole sphere has one
  corner pair, so selection can offer nine candidates — three blends of the two
  poles, all of them the centre, and six spread a radius along each axis — and
  eight of the nine sit on the seam's own meridian plane or the wrong side of it.
  The ninth is that one. Put a wall exactly there and it comes back `OnBoundary`, which is a
  candidate that says nothing and is abandoned — so reading the seam as a
  trimming leaves selection with no candidate at all and the fuse refuses
  `Undecidable` on *a point inside a piece of a face* on a pair it can plainly
  answer. Removing the whole-surface rule from `the_piece_holds` fails this row
  and only this row.
  `a_ball_touching_a_wall_at_the_one_point_that_can_place_it_keeps_both`.

### 38.2 Two arcs that lie on top of each other

A `3 × 4` block corner on a bar of radius five panicked in `arrange.rs` on *"two
arcs leave one node the same way, so they lie on top of each other"*, and so did
the same corner on a ball of radius five. `3.9` and `4.1` were answered or
refused cleanly either side; only the exact `4.0` panicked.

**They do lie on top of each other.** The plane `x = 3` cuts a bar of radius five
in the two rulings `(3, ±4, z)` and the plane `y = 4` cuts it in `(±3, 4, z)`, so
both hold `(3, 4, z)` and the arrangement is carrying one line as two crossings.
There is no anticlockwise order round that node to be found, because the two
directions out of it along the two crossings are the same direction.

That is exactly what `Error::TwoCurvesTouchWithoutCrossing` already says, and the
same geometry one function away already refused with it: `half_of_the_turn`
returns it when the turn against the reference crossing is zero. The walk
asserted instead, on the belief that the first comparison had ruled the case out.
It had not — **a pair both anticlockwise of the reference is never compared
against it**, and that is the whole of the hole. The assert becomes the same
refusal, three lines, and the two sites now give one answer to one question
(§23.8).

**A gap and not a limit.** The answer exists and is the bar: a `3 × 4` rectangle
measured from the axis is inside a circle of radius five, so the block is wholly
inside the bar and the union is the bar unchanged.

#### The family of fixes, and why it is the only one (§16.10)

The arrangement's contract is **one crossing per curve**. Two objects for one
curve breaks the anticlockwise order at every node on that curve, so the only
family that closes this is *folding two crossings that are the same curve when
their partner surfaces differ*. `crossing_like` already folds a pair — for three
planes, by the rank of a three-by-four matrix of stored coefficients, and for
anything else by asking whether the two **partners** are one surface. Neither
reaches here: the two partners are two different planes, and what coincides is
one branch of each pair's crossing with the cylinder. Closing it needs a
predicate that says whether one branch of `S ∩ Q₁` is the same curve as one
branch of `S ∩ Q₂` for a cylinder and two planes, which is new arithmetic in
`rcad-geom` and is not this lane's to write in the same afternoon as four panics.

Nothing short of folding helps, and that is a statement about the shape rather
than about the arithmetic: the tie is exact — the determinant of the three
gradients is `det((3,4,0), (1,0,0), (0,1,0))`, which is zero in whole numbers,
not nearly zero — so no ordering, no scaling and no budget reaches it. Measured:
`Policy::tolerant(100.0)` refuses it the same way.

### 38.3 A closed edge that starts at a pole

A ball cut by `x = 0` panicked in the arena on *"the two coedges of an edge must
run opposite ways"*.

**§37.4's description of it is wrong and the correction matters.** It calls
`x = 0` "the plane the ball's seam lies in"; the seam lies in `y = 0`. What the
two planes have in common is the thing that does the damage: **both contain the
ball's polar axis**, so the great circle each of them cuts runs through both
poles, and the one corner the arrangement puts on that closed crossing is a pole.

A closed edge starts and ends at the one corner it has, so its ends say nothing
about which way it runs and only the surface's own parametrisation does — the way
its longitude takes the points along it, which is
`a_closed_crossing_is_walked_the_way_it_is_stored` and is the rule the fuse writes
with and `Model::validate` reads back with. At a pole every longitude runs
through the one point, so `ẑ × ∇C` is **exactly** the zero vector and the sign of
a dot against it is not a fact about the shape. Both faces along the edge came
back walking it the same way.

The rule now says it has nothing to read. The fuse turns that into
`Error::AClosedEdgeOfTheAnswerWouldStartAtAPole` and `Model::validate` skips it,
the same way it already skips a face with no corner off its partner. Only a
sphere reaches it: a cylinder's gradient on its own surface is radial and never
parallel to its axis, so its longitude never vanishes.

#### A gap, and the demonstration that the direction rule is not what makes it one

Handed a direction the rule cannot derive — patched in for the measurement and
taken straight back out — the same fuse gets two steps further and refuses
anyway:

| rule | with a direction handed to it |
|---|---|
| `Union` | `ACurvedFaceGoesRightRoundWithNoSeam` |
| `Intersection` | `CannotMeshAFace`, `CornersCoincide` |
| `Difference` | `ACurvedFaceGoesRightRoundWithNoSeam` |

The piece is **half a ball bounded by one curve right round its skin with the
poles on that curve rather than inside it**. `cut_a_cap_up_to_its_pole` cuts a cap
into four patches up to the pole *inside* it, and there is no pole inside this
one. So the answer exists as a shape and no face of this kernel holds it, which
makes this a gap of the same kind as §37.5's `ACurvedFaceGoesRightRoundWithNoSeam`
row. Closing it takes the other half of what §38.1 built: a *cut* closed face
written out with a seam and poles of its own, where §38.1 only had to do it for an
uncut one.

#### The controls, and they are what say the entry reaches its subject

The same ball cut at its **equator** is answered — the great circle runs through
no pole, so each half is a cap with a pole inside it — and so is the same ball cut
by a plane through its middle leaning `0.4` of a radian. Two answers either side
of one incidence, with the same ball and the same knife moved, says this is about
the polar axis and not about cutting through the centre.

### 38.4 A closed edge has one corner, and it is not a corner to dissolve

§36.15's own blind hole panicked in `assemble.rs` on *"a run of edges that became
one edge has two ends left"*, and so did a bar wholly inside a block asked for the
difference. §36.15 lists the blind hole among the shapes that work; it works in
the arrangement and had never been committed, and §37.7 already corrected that
claim.

**A closed edge starts and ends at its one corner**, so it appears *twice* in the
list of edges leaving that corner. The joiner read a list of two as "two edges
meet here and nothing else does", dissolved the corner, and left the joined edge
with no ends at all. The word the code was missing is **different**: two
*different* edges meeting at a corner and nothing else is a run to fold into one;
one edge written down twice is a curve that closes.

Three more assertions downstream said the same straight-line thing in three more
places, and each is one word out in the same direction:

- *an edge whose two ends are one corner has no length* — true of a straight side,
  and a cap circle and the rim of a round hole are the counter-example;
- *a ring of a face has at least three sides* — `Model::validate`'s own rule is
  three **or one that closes**;
- *which way a face walks an edge is which of its two corners the walk starts at*
  — which says nothing at all when the two corners are one.

All four are fixed and none of them is a tolerance. This is a **correct answer**
and not a refusal: the blind hole and the bubble are ordinary shapes, the
arrangement already had them right, and the only thing wrong was the way they
were written down.

#### Where an error here would not cancel, measured

**Both ways of getting a closed edge wrong are caught before a volume is read**,
and that is worth recording because a volume is the obvious thing to lean on.
Dissolving its corner leaves the joined edge with no ends, which is the panic
§37.4 pinned; walking it the same way from both faces trips the arena's own rule
that two coedges run opposite ways. Measured: putting the corner test back leaves
every other test in the crate green and panics on the bubble and on the blind
hole, without ever reaching a mesh.

So the closed form is not what tells a right winding from a wrong one here — it
is what says the answer is the **right** solid rather than merely a valid one.
The blind hole is `512 − 3π` and a hole all the way through would be `512 − 8π`,
and both are solids. The shape where a winding *does* have to be told apart by a
volume is §38.1's bubble, and §38.1 records that measurement.

### 38.5 A chord is not the arc it spans

§37.5 reached `ResultIsNotManifold { count: 1 }` — an edge used **once** rather
than four times, which is a hole in the skin and not §17.4's four faces along an
edge — and left it unclassified because nothing had worked out whether the answer
exists.

It exists. The shape is a bar of radius two with a block on `1 ≤ x ≤ 3`,
`−1 ≤ y ≤ 7`, `3 ≤ z ≤ 5`, so the block's two inner walls cut the bar and the rest
of it sticks out: a bar with a rectangular tab. All three rules now measure it
against a closed form, and the shared piece has a tidy one — the disc of radius
two with `x ≥ 1` has area `4π/3 − √3`, the corner of it with `y ≤ −1` has area
`1 − √3 + π/3`, and the difference is exactly `π − 1`.

**It is a gap, and it is §36.28's chord-and-arc gap arriving as a wrong
classification instead of a refusal.** The block's top at `z = 5` comes apart into
the bit inside the bar and the bit outside it, and the outside bit has one *arc*
among its sides. A piece is kept by classifying a point provably inside it, and
the winding count that proves it reads the corners as a **polygon** — so the
middle of the chord across that arc counted as inside the piece, when it is on the
far side of the arc and inside the bar. The piece was classified `Inside`,
dropped, and the arc left with one face along it. The manifold check caught the
hole, so what came out was a refusal and not a corrupt solid; what it was refusing
was a wrong classification.

What rules the chord's middle out is a sign the arrangement had already written
down and this branch was not reading: `FaceMeet::material_side`, which the
**curved** branch of the very same function has always read. A flat piece now
reads it too, for every side of it whose partner is curved, before winding
anything.

**Candidates are only ever ruled out, never in**, so nothing is now proved inside
a piece that was not before — which is what makes this safe to do late. Measured:
no number in `fuses_that_must_agree.rs` moved by one fuse, and
`a_bar_cut_by_one_plane_off_its_axis_is_refused_for_want_of_a_point_inside_a_piece`
still refuses.

### 38.6 Every refusal this lane leaves, classified

§37.5's rows stand except where this lane moved them. These are the changes.

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `TwoCurvesTouchWithoutCrossing` | a `3 × 4` block corner on a bar or a ball of radius five, so two crossings are one curve | **gap** | `a_block_corner_exactly_on_a_bars_wall_is_refused_by_name`. The answer is the bar. Tolerant refuses it too, and the determinant is exactly zero, so it is not a budget question. §38.2 has the family of fixes |
| `AClosedEdgeOfTheAnswerWouldStartAtAPole` | a ball cut by a plane containing its polar axis | **gap** | `a_ball_cut_by_a_plane_down_its_polar_axis_is_refused_by_name`, with the equator and a leaning plane answered as controls. §38.3 measures what a direction handed to it buys, which is a different refusal two steps on |
| `ResultIsNotManifold { count: 1 }` | ~~**withdrawn.** It was a chord standing in for an arc, and the pair it was reached by is answered~~ — **§48 corrects this: withdrawing the row was right about *that pair* and wrong about the refusal. It is live and it is a gap**, reached by turned pairs and by a trio fused two-against-one, and by three axis-aligned boxes on the integer grid where no curve and no rounding is involved | **gap** | `a_bar_with_a_tab_on_it_is_answered_rather_than_left_with_a_hole_in_its_skin` for the withdrawn pair; `an_edge_with_one_face_along_it.rs` for the live refusal |
| `ACurvedFaceGoesRightRoundWithNoSeam` | now also a face with **no** boundary at all on a surface that does not pinch — a plane or a pipe with nothing trimming it | **limit** | unreachable from the public API and reachable from `Deserialize`, which is why it is a refusal and not an assert (§16.11). No answer exists: such a face is unbounded |

The four rows §37.5 marked `gap` for coincident bars, two tools on one plane, the
quartic and the turned notched bar are all unchanged and all still open.

### 38.7 The numbers

| | at §37.6 | now |
|---|---|---|
| tests (`cargo nextest`, whole workspace) | 1102 | **1106** |
| suite duration | 2.9 s | **2.9 s** |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 3 | **3, unchanged** |
| `rcad-predicates` files changed, whole milestone | 0 | **0** |
| `rcad-geom` files changed, this lane | — | **0** |
| fuzz corpus | 815 | 815 |

**No measured number moved**, family by family and never blended (§17.7), and
these are the same numbers §37.6 recorded against a tree four fixes ago:

| family | fuses | refused | not a solid |
|---|---|---|---|
| §17.7 control, lattice pairs | 423 | 0 (0.0%) | 20 (4.7%) |
| §17.7 degenerate, sharing coordinates | 423 | 0 (0.0%) | 72 (17.0%) |
| §17.7 control, somewhere else in `f64` | 423 | 0 (0.0%) | 17 (4.0%) |
| §17.7 headline, turned pairs | 423 | 96 (22.7%) | 17 (4.0%) |
| §22 headline, skew pairs | 423 | 51 (12.1%) | 8 (1.9%) |
| §22 control, both leaning alike | 423 | 24 (5.7%) | 17 (4.0%) |
| §37 control, a bar through a block | 423 | 0 (0.0%) | 0 (0.0%) |

**`just fuzz 180` was re-run because this lane touches kernel code**: 5 282 767
executions in 181 seconds at about 29 200 a second, and **no defect at all**. The
working corpus grew by fifteen inputs and coverage by seven edges. None of it is
checked in — this lane changed **no `rcad-geom` file**, so the arithmetic the
fuzzer draws over is the one §36.27 already soaked, and §30.7's rule holds — an
unminimised handful is not worth checking in. The shipped corpus stays at 815.

That every measured rate is unchanged is the number to read. Four fixes went into
the arrangement, the assembly and selection, and the one that could plausibly have
moved a rate — §38.5, which makes classification *stricter* — moved none of them,
because it only ever rules a candidate out and another candidate answers.

### 38.8 The six thresholds, re-derived

§37.7's last bullet: the doc comments on `fuses_that_must_agree.rs`'s thresholds
quoted numbers the tree no longer gives. Every threshold still held with room, so
these were stale prose and not failures — but a doc comment is a claim.

| quoted | measured |
|---|---|
| 420 fuses a family | **423** |
| turned refusing 20.71% | **22.70%** |
| skew refusing 14.29% | **12.06%** |
| lattice not a solid 4.8% | **4.7%** |
| flush not a solid 17.4% | **17.0%** |
| elsewhere not a solid 4.1% | **4.0%** |
| turned not a solid 3.9% | **4.0%** |
| skew not a solid 1.2% | **1.9%** |
| leaning alike not a solid 3.1% | **4.0%** |

Two derived claims went with them: `MEASURED_CASES`'s doc said each family is 414
fuses and the suite runs in about ten seconds, and both are now what they measure.

`SKEW_REFUSALS_MUST_STAY_BELOW` also still carried the literal placeholder
`SKEW_SOAK_PROSE`. It is filled in with a soak measured here — six fresh seeds at
ten times the corpus, giving **12.6%, 13.8%, 14.5%, 15.3%, 16.2% and 16.3%**, none
of them over the 18% threshold and the worst within 1.7 points of it. The turned
family's own soak was re-measured while the numbers were out: **23.4%, 23.4%,
23.9%, 24.0%, 24.0% and 24.7%**, so four of six now sit above its threshold where
the old prose said two of six did.

**One number is worth reading twice.** The turned family's headroom is now 1.3
points, which is six fuses, where the prose said 3.3 points and fourteen. That is
the measurement having moved and not the threshold slipping — it read 20.71% of
420 when it was written and reads 22.70% of 423 now — but six fuses is thin, and
the doc says so where it used to say fourteen.

### 38.9 Corrections to §31 through §37

- **§36.15's third shape works now.** §37.7 corrected it to "does not work"; the
  blind hole is committed as well as arranged, and so are the two other shapes
  §36.15 names. Both corrections stand in order: §36.15 was wrong when written,
  §37.7 was right when written, and §38.4 is what makes §36.15 true.
- **§37.4's fourth defect is misnamed.** *"A ball cut by the very plane its seam
  lies in"* — the seam lies in `y = 0` and the defect is reached by `x = 0` too.
  What the two have in common is the polar axis, not the seam, and the difference
  is not pedantry: a plane through the ball's centre that does **not** contain the
  polar axis is answered.
- **§37.4's third defect no longer reaches its third shape.** It lists "a bar cut
  a sliver in from its own cap" alongside the blind hole and the bubble. A knife
  whose face is exactly the bar's own cap plane refuses `Undecidable` on *a point
  inside a piece of a face* — §36.28's coincident-face gap, and it does so with
  §38.5's change taken back out too, so it is not this lane's doing. A knife that
  slices a sliver *in from* the cap without sharing its plane is answered, at
  `π r² h` to a part in ten thousand.
- **§37.4's first defect is understated in one direction and overstated in
  another.** It says a ball's classification is correct while its fuse is wrong,
  which is right; it does not say that `fuse([ball], [], Union)` — the smallest
  case — exercises neither selection nor nesting, so the obvious test for it
  proves the least (§38.1).
- **§37.5's `ResultIsNotManifold { count: 1 }` row is withdrawn.** It is §36.28's
  chord-and-arc gap, and the pair that reached it is answered (§38.5).
- **§37.8's ordering was right.** "The dropped ball is a wrong answer and is worth
  more than the other three together" is what this lane found: §38.1 is the only
  one of the four that took a new concept — a seam the fuse writes rather than
  reads — and the other three are one word each.
- **`Case::Panics` is gone from `degenerate_curved_fuses.rs`**, along with
  `Outcome::Nothing`. An outcome no row reaches is a row nobody can read, and both
  are §37 artefacts of the state this lane closed.

### 38.10 What the closing gate has to be told

- **The milestone's exit risk is closed.** §37.8 named three panics and a wrong
  answer as it. There is no panic left on public-API input in the curved
  catalogue, and no `Ok` with the wrong answer in it — the table the tests print
  has two outcomes again, answered and refused, and `Outcome::Nothing` no longer
  compiles.
- **Two of the four are refusals and both are gaps**, so neither is finished
  business. §38.2 wants a predicate that says whether one branch of `S ∩ Q₁` is
  the same curve as one branch of `S ∩ Q₂`; §38.3 wants the other half of what
  §38.1 built, a cut closed face written out with its own seam and poles. Both
  are `rcad-geom` and `rcad-topo` work of a size no lane should start after
  lunch.
- **§38.1 is the first time this fuse writes a `WireStep::Pole`.** Everything
  about a closed face the fuse could not hold before now has half a road built to
  it, and §38.3 is the shape waiting at the other end of it.
- **The turned family's threshold has six fuses of headroom** (§38.8). It is not
  wrong and it is not a target, but it is the number most likely to go red next
  for a reason that is nobody's regression.

