## 37. Roadmap §7.4 over curved solids, recorded 2026-09-01

Lanes 6 and 7, test-only. `rcad-predicates` is at **0 files changed for the
whole milestone**, nine lanes now, and `rcad-geom` and `rcad-topo`'s `src/` are
untouched by these two.

**The headline is not the table.** Four defects came out of writing it, three of
them panics on solids a caller can build through the public API and one of them a
confident wrong answer, and §4.1 ranks a wrong answer below both. They are pinned
by tests that say in their names that they are wrong, and nothing here blesses
any of them.

### 37.1 The catalogue, as a table

`degenerate_curved_fuses.rs`, and both tables are printed by the tests that
assert them, so this section cannot drift away from what the kernel does without
something going red. Criterion 14's four first.

| §7.4 entry | the pair | rule | outcome |
|---|---|---|---|
| tangent spheres | two balls of radius two, centres four apart | Union | **answered with nothing, which is wrong** |
| tangent spheres, the control | the same two balls, three apart, really crossing | Union | refused by name |
| tangent cylinder–plane | a bar of radius two and a wall on `y = -2` | Difference | refused, `TwoCurvesTouchWithoutCrossing` |
| seam-crossing intersections | a bar notched across its seam | Difference | **answered** |
| seam-crossing intersections | a ball cut by a plane meeting its seam once | Difference | **answered** |
| seam-crossing intersections | a ball cut by a plane meeting its seam twice | Union | refused, `ACurvedFaceGoesRightRoundWithNoSeam` |
| seam-crossing intersections | a ball cut by the very plane its seam lies in | Union | **panics** |
| closed periodic surfaces | a block with a round hole right through it | Difference | **answered** |

And §7.4's other entries, asked again with a ball or a bar in them.

| §7.4 entry | the pair | rule | outcome |
|---|---|---|---|
| coincident curved faces | one bar built twice, in the same place | Union | refused, `Undecidable` on *a point inside a piece of a face* |
| vertex on a curved face | a `3 × 4` block corner on a bar of radius five | Union | **panics** |
| vertex on a curved face | the same corner on a ball of radius five | Union | **panics** |
| zero-length edges | two tools meeting on a plane that cuts a round wall | Difference | refused, `Undecidable` on *a point inside a piece of a face* |
| sliver faces | a bar cut by a slab a millionth thick | Intersection | **answered** |
| fully contained solids | a bar in the middle of a block, united | Union | **answered** |
| fully contained solids | the same, hollowed out | Difference | **panics** |
| fully contained solids | a ball in the middle of a block | Intersection | **answered with nothing, which is wrong** |
| touching, not overlapping | a ball resting on a block's top at one point | Union | **the ball is dropped, which is wrong** |
| touching, not overlapping | a block sitting on a bar's cap | Union | refused, `TwoCurvesTouchWithoutCrossing` |
| self-intersecting inputs | nothing in the public API builds a curved one | Union | discharged by the planar row |

*2026-10-08, Q1341: the row no longer holds, as public API goes. `Import::read`
of a written model builds a self-intersecting curved solid, and
`Model::validate` passes it, since §17.4 keeps that check out of `validate`.
The example is Q1216's unplaced bar of radius 8, cut by `z = 14` and by
`z = 2y`: its bottom ellipse reaches `z = 16`, above the top cap, and its two
edges cross where `sin t = 7/8`. The fuse still refuses it
`ArgumentIntersectsItself` on 12 of 12 — every rule, both orders, beside a far
bar and beside an overlapping ball — which is the designed refusal, so the
row's verdict column stands. These numbers are Q1216's lane's own measurement,
not re-measured here; its probe is `target/q1216-probe` in the dispatcher's
tree (`src/main.rs`, `src/plate.rs`, `model.json`, `unplaced.json`). Re-filed
from Q1317, which the Q1318 bundle (`1742a230`) closed without writing this
note.*

### 37.2 How each entry is known to reach what it is about

§14.7's vacuity rule, and lane 5c's finding that the turned family reads 0.00%
because its bar passes clear of every wall. Every entry above is built so the
degeneracy is **exact** — shared whole numbers, a Pythagorean triple, or a plane
through a stored radius — and every one of them is paired with a control a tenth
of a millimetre off it. Where the two come back the same way the entry is not
reaching its subject and the row below says so rather than claiming a pass.

| entry | how it is known to reach it |
|---|---|
| tangent spheres | **it does not.** The two balls touch and neither cuts the other, so both are dropped, and the same nothing comes back when they are a mile apart. §37.4's first defect is the whole of this row |
| tangent cylinder–plane | moving the tangent wall from `y = -2` to `y = -1.9` changes the refusal to a different one **by name**. Two named outcomes either side of one number |
| seam-crossing | the notch across the seam and its exact mirror image on the far side agree to seven parts in `10^17`. A reflection carries the bar to itself and the notch to the other notch, and carries the seam nowhere — so any difference between the two would be the seam and there is none |
| seam-crossing, the ball | the crossing circle meets a ball's seam once for a cut by `z = c` and twice for a cut by `x = c`. One is answered and the other refused, and the two cuts differ in the axis and in nothing else |
| closed periodic | the answer is asserted to contain closed edges — edges that come back to the corner they left because they go right round the cylinder |
| coincident curved faces | the refusal names `(2, 0, 0)`, which is the bar's own seam corner, with an infinite enclosure: nothing was nearly decided |
| vertex on a curved face | `3² + 4² = 5²` puts the corner *provably* on the wall, and the same corner at `3.9` and `4.1` is answered either side on a bar and refused either side on a ball. Only the exact incidence panics |
| zero-length edges | the refusal names `(0, ±2, 0)`, which are the two rulings where the shared plane touches the cylinder |
| sliver faces | the slab is `2^-20` thick, exactly representable, so this is a sliver and not a rounding; the disc it leaves measures `4π · 2^-20` to a quarter of a per cent |
| fully contained | the bar in the same position gets two of the three rules right, so the ball's two wrong answers are the ball's shape and not containment |

### 37.3 The metamorphic results, family by family

`fuses_that_must_agree.rs`, and never blended (§17.7). The curved family is a
block with a round bar through it, the bar a whole unit clear of every wall,
generated by `a_curved_pair` — **a family of its own and not an arm of
`Shape`**, because an arm would have changed the corpus every measured family
draws from and every number §17.7 and §22 record with it.

| property | fuses | refused | not a solid | compared |
|---|---|---|---|---|
| a bar through a block, every rule, fixed seed | 423 | **0** | **0** | — |
| commutativity, a bar through a block | 244 | 0 | 0 | **122 of 122, 100%** |

**One hundred per cent is the number to read**, not the pass. It is the highest
reach of any property in the file — on the run recorded here the flat families
came in between 62% and 100%, with the depth-four chain the floor-setter, and
those run from a fresh seed so they move — and it says the property really
compared a curved answer bit for bit on every case it drew. What makes that
possible is the row above it: nothing in this family refuses, so both sides of
every comparison arrive.

Three of §31.7 criterion 15's names, scored:

| | |
|---|---|
| `swapping_the_two_sides_of_a_curved_pair_changes_not_one_bit` | **met**, at 100% reach |
| `uniting_a_block_and_a_bar_the_other_way_round_gives_the_same_solid` | **met**, at one fixed pair so a failure names a shape rather than a seed |
| `a_bar_with_a_copy_of_itself_taken_away_is_nothing_at_all` | **cannot be met today.** `A \ A` over a bar is refused — `Undecidable` naming a point inside a piece of a face, at the bar's seam corner. The test asserts the refusal by name, and the control is the same question over a block, which answers and leaves nothing |

**Union and intersection only, never difference.** `A \ B` and `B \ A` are
different questions; the first draft of the commutativity property asked all
three and the suite caught it on the second case it drew.

#### The properties that are *not* run over the curved family, and why

Not an oversight in any of the three cases, and each is the same shape of
argument §17.7 uses to keep inclusion and exclusion off the turned family.

- **Inclusion and exclusion, absorption, associativity, the chains.** All of them
  compare `invariants_of`, which is a volume, an area and a centroid read off a
  **mesh** for a curved solid. Two decompositions of one solid mesh differently,
  so the comparison is not exactly true and a bound on it needs a bound on how
  far two meshes of one shape can be from each other — which nothing here has.
  A tolerance guessed instead would make the test measure the tolerance (§14.1
  question 1).
- **Transform invariance.** A `Movement` is a quarter turn and a whole-number
  shift, exact on a plane's four coefficients and on a corner. Nobody has worked
  out what it is on a pipe's axis, and `Answer::moved_by` panics rather than
  guess.
- **`A ∪ A ∪ A ∪ …`.** Two bit-identical bars refuse, so it would compare
  nothing. That is the row above rather than a second finding.

Three solids at once was tried and is not here: a block with **two** bars through
it refuses with `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` before anything is
selected, so a curved trio property would assert nothing at all. Named here so
the next lane does not spend an afternoon finding out.

### 37.4 Four defects this lane found and did not fix

A metamorphic suite that edits the kernel until it goes green is worth nothing,
so none of these is fixed. Each is pinned by a test whose name says it is wrong,
and each has a control.

**1. A ball that nothing cuts is dropped by the fuse.** `fuse([ball], [], Union)`
comes back `Ok` with **zero solids**, no refusal, an empty log, and a model that
validates. A bar in the same position comes back whole. It is not classification:
the ball answers `Inside`, `OnBoundary` and `Outside` correctly for points at its
middle, on its skin and outside it. A ball is one face, its boundary is a seam
with a pole at each end, and the arrangement of that face against no cuts yields
no piece its own boundary winds once round.

Everything about a ball that does not cross its partner inherits it: two tangent
balls, a ball inside a ball, a ball inside a block asked what they share or what
is left, a ball resting on a block, a ball tangent to a bar. **This is a wrong
answer and not a refusal**, so §16.10's two words do not fit it — it is neither
a limit nor a gap but the thing both of them exist to avoid.
`a_ball_that_nothing_cuts_is_dropped_by_the_fuse`.

**2. A corner exactly on a curved face panics.** `arrange.rs`, *"two arcs leave
one node the same way, so they lie on top of each other"*. A `3 × 4` block corner
on a bar of radius five, and the same corner on a ball of radius five. The
controls are the same corner at `3.9` and `4.1`: on the bar both are answered,
on the ball both are refused by name, and only the exact incidence panics. A
panic on public input, which §16.11 says may not happen.
`a_block_corner_exactly_on_a_bars_wall_panics`,
`a_block_corner_exactly_on_a_balls_skin_panics`.

**3. A bar with a cap inside another solid panics on the way out.**
`assemble.rs`, *"a run of edges that became one edge has two ends left"*. It
reaches a bar wholly inside a block asked for the difference, a bar with one cap
inside a block under union and difference, and a bar cut a sliver in from its own
cap. **It is §36.15's own blind hole**, which that section lists among the shapes
that work — and it does work, in the arrangement, which is all
`a_blind_hole_cuts_the_face_it_stops_short_of_as_well` ever asked it.
`hollowing_a_block_out_with_a_bar_inside_it_panics`,
`a_blind_hole_panics_when_it_is_committed_rather_than_only_arranged`.

**4. A ball cut by the very plane its seam lies in panics.** *"the two coedges of
an edge must run opposite ways"* — `Invalid::CoedgesRunTheSameWay` arriving as an
assertion rather than as the refusal it is elsewhere. The same ball cut by any
other plane parallel to that one is refused by name, and cut square to it is
answered. `a_ball_cut_by_the_very_plane_its_seam_lies_in_panics`.

### 37.5 Every refusal these two lanes found, classified

The rows §36.16 and §36.26 already carry are unchanged. These are the ones the
catalogue reached that no section had placed.

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `TwoCurvesTouchWithoutCrossing` | a cut plane is tangent to a bar's wall, so two curves on that wall touch and do not cross | **limit** | `a_wall_that_touches_a_bar_without_crossing_it_is_refused_by_name`. There is no curve to walk: the two arcs meet and there is nothing between them, and the same wall a tenth off tangent refuses for a different reason by name |
| `TwoCurvesTouchWithoutCrossing` | a block sits exactly on a bar's cap, so its wall touches the cap circle | **limit**, the same one | `a_block_sitting_on_a_bars_cap_is_refused_by_name` |
| `ACurvedFaceGoesRightRoundWithNoSeam` | a cut meets a ball's seam twice, leaving a piece bounded by one closed curve and no seam | **gap** | `a_ball_cut_by_a_plane_that_crosses_its_seam_twice_is_refused_by_name`. The answer exists — it is two caps of a ball — and §36.18 closed the pinching case without closing this one |
| `Undecidable`, *a point inside a piece of a face* | two coincident curved faces | **gap** | `a_bar_and_a_copy_of_it_in_the_same_place_are_refused_for_want_of_a_point_inside_a_piece`. The answer is the bar. Under `Policy::tolerant` as well |
| `Undecidable`, *a point inside a piece of a face* | two tools meeting on one plane that cuts a round wall | **gap** | `two_tools_meeting_on_a_plane_that_cuts_a_round_wall_are_refused_by_name`. One tool making the same cut is answered, so the answer exists and the arithmetic could not reach it |
| `Invalid::VolumeIsNotPositive` | two balls crossing far enough apart that the answer's shell measures nothing | **not a refusal about arithmetic** — this crate saying it cannot assemble what it cut, before writing anything | `two_balls_that_really_cross_are_refused_by_name` |
| `ACrossingThisCannotBeOrderedAlong` | a ball with a pipe through it, §36.20's quartic | **gap**, unchanged from §36.20 | `a_ball_with_a_pipe_through_it_is_refused_by_name`, and **it is not a budget question**: `Policy::tolerant` refuses it too, which §36.20 did not measure |
| `Undecidable`, *order along a crossing* | a turned notched bar | **gap**, unchanged from §36.28 | `a_turned_notched_bar_is_refused_for_want_of_an_order_along_a_crossing`, tolerant too, and the same shape unturned is decided and records nothing |
| `Undecidable`, *a point inside a piece of a face* | a bar cut by one plane off its axis | **gap**, unchanged from §36.28 | `a_bar_cut_by_one_plane_off_its_axis_is_refused_for_want_of_a_point_inside_a_piece` |

`ResultIsNotManifold { count: 1 }` was reached once, by a notch whose wall cuts
right across a bar, and it is **an edge used once rather than four times** — a
hole in the skin rather than §17.4's four-faces-along-an-edge. It is not in the
catalogue because nothing here has worked out whether the answer exists, and
guessing which of the two words it is would be the paragraph §16.10 forbids.
Named so the closing gate can decide.

### 37.6 The numbers

| | at §36.27 | now |
|---|---|---|
| tests (`cargo nextest`, whole workspace) | 1070 | **1102** |
| suite duration | 2.9 s | **2.9 s** |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 3 | 3 |
| fuzz corpus | 815 | 815, untouched |

Thirty-two tests: twenty-seven in the catalogue and five metamorphic.
`just default` is green, `cargo-deny` included. `just fuzz` was not run — neither
lane touches `rcad-geom`, so a longer soak would measure the same arithmetic
§36.27 already soaked.

**No measured number moved.** Lane 7 changed how an answer is written down for
comparison, which touches every family in `fuses_that_must_agree.rs`, so the six
fixed-seed rows were re-measured against a clone of the tree at `3b0e1ef` and
come back identical fuse for fuse: 423 fuses each; the three control rows at 0
refused; both leaning alike at 24; skew at 51; turned at 96; and the
not-a-solid shares at 4.7%, 17.0%, 4.0%, 4.0%, 1.9% and 4.0%. The rows that do
differ between the two runs are exactly the ones drawn from a **fresh seed every
run** by design, and they differ from one run of the same tree to the next.

**Writing an answer down by its surface rather than by its plane loses nothing
for a flat family.** A face's outward normal is its stored plane's normal times
the orientation's sign, so two faces agreeing about the plane bits agree about
the normal exactly when they agree about the orientation — and under a
`Movement`, which is a quarter turn and a whole-number shift, multiplying by `±1`
commutes with a swap and a negation. The comparison is the same one, one field
narrower and three surfaces wider.

### 37.7 Corrections to §31 through §36

- **§36.15's third shape does not work.** "A block with a round hole through it,
  a bar cut square across by a block, and a blind hole are the first" — the blind
  hole is arranged and never committed, and committing it panics (§37.4's third
  defect). The two others do work, end to end, and are in the catalogue.
- **§36.23 and §36.28 on two balls are right about one sub-family and not the
  shape.** "Measured, two balls united refuse at
  `ACurvedFaceGoesRightRoundWithNoSeam`" holds for two balls of radius two up to
  two apart. At three apart they refuse `Invalid::VolumeIsNotPositive`, and once
  they stop crossing at all — tangent, nested, or far apart — they come back
  `Ok` with nothing. So the sentence §36.28 hangs on it, that whoever makes two
  balls fuseable closes that refusal first, is only the first thing they close.
- **§36.20 left the quartic unmeasured under a tolerant policy.** It refuses
  there too, so it is not a budget question and no amount of slack reaches it.
- **§31.7 criterion 15's second name cannot be met**, and §37.3 says what stands
  in the way. The criterion is not withdrawn; the test is written to fail the day
  it becomes meetable.
- **§36.22's fourth bullet is withdrawn by §36.24 and by this lane both.** It
  warned that a hand-built curved solid might mesh a cap to the wrong pole
  without `validate` catching it; §36.24 made the rule a check, and every curved
  solid this lane builds goes through `Model::validate` after every fuse.
- **The doc comments on `fuses_that_must_agree.rs`'s six thresholds are stale**,
  and were before this lane: they quote 420 fuses, 20.71% turned, 14.29% skew,
  4.8%, 17.4%, 4.1%, 3.9%, 1.2% and 3.1%, against a tree that gives 423, 22.7%,
  12.1%, 4.7%, 17.0%, 4.0%, 4.0%, 1.9% and 4.0%. Every threshold still holds with
  room. `SKEW_REFUSALS_MUST_STAY_BELOW`'s doc also still carries the literal
  placeholder `SKEW_SOAK_PROSE`. Not touched here, because a number in a doc
  comment is somebody's derivation and re-deriving six of them is not this lane's
  work — but the closing gate inherits them.

### 37.8 What the closing gate has to be told

- **Three panics and a wrong answer are the milestone's exit risk**, not the
  refusals. §16.11 says a caller's bytes may not panic, and all four defects in
  §37.4 arrive from `Model::fuse` on solids the public API builds in two lines.
- **The order to close them in is the order they cost.** The dropped ball is a
  wrong answer and is worth more than the other three together; the corner on a
  curved face is the one an ordinary part reaches by accident; the blind hole is
  the one a user will hit first.
- **Two entries of roadmap §7.4 are not measuring what they are named.** Tangent
  spheres measures the dropped ball, and vertex-on-a-curved-face measures a
  panic. Neither is a hole in the catalogue — both are recorded — and only five of the nineteen rows are
  answered at all, so a closing gate counting rows will get the wrong number
  either way.
- **The curved metamorphic family is the strongest row in the file and the
  narrowest.** One shape, at 100% reach, and every degenerate arrangement of it
  is hand-written in the catalogue instead. Widening the generator to reach
  tangency or coincidence would take the reach to nothing, which is what §37.2's
  first row is about.

