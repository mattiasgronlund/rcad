## 340. A walk whose signs match several walks of one lump is placed, not glued — 2026-09-28

*Queue row Q204, from §282.3, ranked up by Q20's panic. Lane
`lane-q204-two-walks-of-one-lump-in-one-cell`, off `e0760753`.
`arrange.rs`'s `group_the_cycles_into_cells` joined two walks of different
lumps whenever they agreed about every closed curve (§36.3), and through a walk
of a third lump that put two walks of one lump into one cell. On a cone that
was a piece selection could find no point in (§282); on a flat face whose glued
regions were all kept it was a panic in `assemble.rs`. **A gap** (§16.10): the
cells are there, and the arithmetic joined them.*

### 340.1 What the sign rule cannot see

A walk's signs are its sides of the curves drawn in full, and only a closed
curve is drawn in full. Two walks of one lump can be on the same side of every
one of them and told apart only by a curve that runs on for ever: the two halves
of a floor a lid's plane cuts in two, or the patch a leaning plane cuts off a
frustum's face and the rest of that face. A walk of another lump — the circle a
cone's surface cuts in that floor, the frustum's bottom rim — then agrees with
both, was joined to both, and joined them to each other.

On a plane, a pipe, a cone or a ball, one lump's walks each go round a region of
their own and nothing else drawn joins two of them up again, so a cell with two
walks of one lump in it is always two regions glued. A torus is the exception: a
circle round its tube leaves one region.

### 340.2 The rule now

A walk `W` and a walk `V` of another lump are joined only when each is the one
walk of the other's lump it faces. Where `W`'s signs agree with one walk of a
lump, that is the one it faces. Where they agree with several, it is asked which
of them holds it, two ways:

- **On a flat face, by winding**, when every candidate is made of straight
  sides. Their corners are then their whole shape, and the one candidate whose
  corners wind once round a node of `W` holds it. A point no candidate winds
  once round is in the walk round the outside of the lump or in none, and is not
  answered.
- **On a cone or a pipe, by what goes round the axis.** One nappe, or a pipe,
  is a ring. When `W` goes round the axis, its lump cannot sit in a region of
  the candidates' lump whose walk goes round nothing, because that region is a
  disc and nothing in a disc goes round anything. One candidate left standing
  is the answer. How often a walk goes round is counted as crossings of one
  half-plane through the axis, read off exact signs of the node boxes, and it
  rests on every curved arc being cut by a plane, which is what makes the angle
  round the axis only grow or only shrink along it:
  `h = d / (s(n_x cos θ + n_y sin θ) + n_z)` on a cone in its own frame, and the
  same shape on a pipe. Every node of both lumps is held to one nappe first.

What neither settles is refused `CannotTellTheCellsOfAFaceApart`, except for a
walk the face's boundary is known to wind round no times, which is left unjoined:
no piece is made of its cell. A cell that still ends up holding two walks of one
lump is refused the same way, on any surface but a torus.

The frustum of §282 reads: the bottom rim goes round the axis, the patch goes
round nothing, the rest goes round once, so the rim belongs with the rest.

### 340.3 What moved, measured on this branch

Each by `mise exec -- just t -p rcad-topo --test <file>`, and the base figures
from the pins as they stood on `e0760753`.

| where | before | now |
|---|---|---|
| `a_floor_holding_a_circle_cut_in_two.rs`, Q20's two shapes | panic at `assemble.rs:1007` (run on the base with the test alone) | 11 fuses answer, each validated, counted and within its mesh's error of the volume by hand |
| `a_plate_with_a_hole_beside_a_cut.rs`, §282.4's plate with both tools | 4 `ResultIsNotManifold` | 4 answers, within the mesh's error of the volume by hand |
| `a_plane_across_a_cone.rs`, the leaning block's union | refused, a point inside a piece | answers; one cone face with the two wires §282.1 expects; volume within the mesh's error of `42π + 16000 −` the shared segments |
| other nappe, steeper plane (96) | 40 right, 44 no point, 12 ordering | 68 right, 16 no point, 12 ordering |
| other nappe, block below (96) | 56 right, 20 no point, 20 ordering | 62 right, 8 no point, 20 ordering, 4 `ResultIsNotManifold`, 2 answer and do not measure |
| other nappe, the slab | 4 no point | 4 right |
| `how_often_a_fuse_with_a_cone_refuses.rs`, a frustum against a block | 66 of 96, 30 no point | 96 of 96 |
| `what_the_answered_block_fuses_actually_say.rs` | 24 answers add up | 30 answer and add up |

**Two expectations were wrong and nothing read them while every rule refused.**
The slab's cone-less-slab is two solids, not one: the slab takes the whole cone
from `z = 1` to `z = 2`, so the tip comes away. And one block of the family
"below the frustum", leaned at `2` and reaching up to `1.5`, shares `2.917` with
the frustum by the file's own column sum. Both files now say so and work the
second out from the shapes.

The 16 and 8 that still find no point were asked about by trace for the slab
and the leaning block only; that they are single cells is read from the rule,
not measured one by one.

### 340.4 What this leaves

- **Four pins outside this lane's files** moved and fail until their owner
  moves them: `how_often_a_fuse_with_a_cone_refuses.rs` (96 of 96 now),
  `what_the_answered_block_fuses_actually_say.rs` (30 now),
  `a_ball_and_a_bar_in_one_block.rs` and `fuses_that_must_agree.rs` (below).
- **The ball-then-bar union of `a_ball_and_a_bar_in_one_block.rs` refuses one
  step on.** Its floor's bar circle agreed with the inside and the outside of
  the floor's square, and the old glue refused. The square's corners now place
  it inside, and every face's cells were printed and are the right regions —
  the floor with two round holes, the bar's disc, three tubes. The fuse then
  refuses its own answer `Invalid(FacesDoNotAgreeWhichWayTheyFace)`: its
  coarsest mesh uses a side more often one way than the other. That is after
  `arrange.rs`, and where is not found. The same pair fused all at once answers.
  §340.5 found it, and it is in `arrange.rs` after all.
- **Two answers do not measure.** The union and a difference of the block that
  reaches into the frustum keep a piece of the cone bounded by an arc of the
  parabola the block's steep face cuts; they hold every grid point asked (7327)
  and `measure` gives up with `DeflectionBelowWhatAMeshCanReach`.
- **Four refuse `ResultIsNotManifold`** in the block-below family, not looked
  into.
- **The refusal of §340.2 has no shape that reaches it** in the suites. It is
  what a walk that goes round nothing, a ball, or a curved side on a plane would
  get.

**2026-10-05, queue row Q424: two shapes reach it, and the suites already had
one.** `a_walk_neither_winding_nor_the_axis_places.rs` pins
`CannotTellTheCellsOfAFaceApart` under every rule with the arguments both ways
round for two shapes, and a printed probe at each of the five places
`arrange.rs` raises it, since removed, named
`the_walk_of_each_other_lump_each_walk_faces` for all twelve fuses.

- **On a ball**: a ball of radius 2 less a small ball on its skin, against a
  square rod through it. The rod's four walls cut the ball in four circles that
  are one lump, with a patch inside all four at each end of the rod. The dimple
  sits in one patch and is on the same side of every circle as both.
- **A curved side on a plane**: a plate with a round notch at one corner, cut in
  two by a holed slab's wall, with the hole's circle in the half away from the
  notch. Winding by corners is not asked, because the other half has the
  notch's arc.

Both answers exist, and the test docs work out their volumes, so both are gaps
(§16.10). The same probe on Q616's
`a_holed_bar_cut_to_a_bow_tie_cap_is_refused_while_a_face_is_cut_into_cells`
(Q639's row) named the same place, on the bar's floor, six times: a curved side
on a plane as well. So the bullet above was stale from Q616 on.

What the file pins that does not reach it, each with what the probe printed: a
ball face bounded by two arcs places its hole, because every face is cut by its
own solid's faces too (`cuts.rs`), so the arcs' circles are drawn in full, and
the fuse is refused later in selection, `AFaceTrimmedByACurveThisCannotFollow`.
A ball against a pipe is refused `ACrossingThisCannotBeOrderedAlong` before any
cell, and that was the one walk going round nothing on a pipe that was tried. **No
cone was tried**, so a walk that goes round nothing is neither shown to reach it
nor shown not to.

**A second gap came up on the way, with the same cause.** A pin through a
plate's round tab is refused `ResultIsNotManifold`: the probe printed the pin's
circle on the plate's floor wound no times, because the face's corners are all on
the plate's square and `say_whether_the_face_holds_each_lump_of_closed_cuts`
counts round the corners alone. Where a side is curved, the corners are not the
face's shape there either.

### 340.5 The half ball the second fuse turns inside out

*Queue row Q412, which replaced Q204 to land this section's fix with the four
pins of §340.4 in its files. Lane `lane-q412-two-walks-of-one-lump-in-one-cell`,
off `e5a7d95a` with `83b8511e` merged.*

The ball-then-bar union of `a_ball_and_a_bar_in_one_block.rs` refuses because
**the half ball hanging under the floor comes out of the arrangement as the
upper half**. Nothing cuts it in the second fuse. Its one wire is the circle
where it meets the floor, and that circle's one corner is on the floor, so
`Arrangement::material_is_left_of_forward` finds no corner off the floor to read
its side from and takes the circle's forward direction. That is §197's guess,
which §197 named and closed for an edge with two ends and left standing for a
whole circle. The lower half lies on the floor plane's outer side, so forward
is the wrong way for it.

Traced with printed probes on this branch, now removed:

- The first fuse's answer has the floor walking the circle forward and the half
  ball walking it backward. The second fuse's pieces have both walking it
  forward. The floor's piece reads `material_side: Pos` off its square's
  corners; the half ball's reads `Neg`, from the guess.
- The four sides the coarsest mesh uses one way only are the four chords of
  that circle, each used once by the floor and once by the half ball.
- **Turning that one guess round for that one face, and nothing else**, the
  union answers, validates, has 11 faces, and measures `357.594` at deflection
  `0.02`. The shape worked out by hand is the block, the lower half ball and
  two bar stubs of two units: `343 + 2π/3 + 4π = 357.661`. The all-at-once
  union measures the same `357.59`.

**This is a gap, and the fix is not in `arrange.rs` alone.** Which way a face's
wire walks a closed edge is in the model: the coedge's direction on the edge,
read with `a_closed_crossing_is_walked_the_way_it_is_stored`, as `holds.rs`
already does for a ring's face. `bake.rs`'s `BakedEdge` does not carry it, and
`split_a_face` gets nothing but the baked face. So the fix is `bake.rs`
carrying that direction and `material_is_left_of_forward` reading it before
it falls back on a corner. Refusing where it now guesses was not taken, because
the guess is also where every round cap goes and it is right for them: a flat
disk lies inside the round surface that cuts it. The bar's two caps reached it
in this very fuse, and with only the half ball turned round the answer
validated.

Until then the union is pinned at that exact `Invalid`. `a_curved_refusal`
still calls it a bug. The one property that meets this configuration,
`uniting_a_ball_and_a_bar_with_a_block_does_not_depend_on_which_two_go_first`,
lets that one `Invalid` through for that one fuse and names it
`A_HALF_BALL_TURNED_INSIDE_OUT`. Its quarter of fuses that do not answer is
unchanged. Q316's block and ball, clear of each other, were not looked at.
§340.7 is the fix.

### 340.6 The two counts, measured on this branch

Each by `mise exec -- just t -p rcad-topo --test <file>`, and the whole crate
by `mise exec -- just t -p rcad-topo --no-fail-fast`: 1117 run, 1117 passed.

| where | before | now |
|---|---|---|
| `how_often_a_fuse_with_a_cone_refuses.rs`, a frustum against a block | 66 of 96, 30 no point | 96 of 96, nothing refused; its floor is 96 |
| `what_the_answered_block_fuses_actually_say.rs`, rows that miss | 24, four rows | 30, all five |
| the same, rows that meet | 7 checked | 11 checked, 66 answers; its floor is 11 |

### 340.7 The edge says which half, where no corner can

*Queue row Q421. Lane `lane-q421-a-half-ball-under-the-floor-faces-the-right-way`,
off `ec4cbc69`.*

**The flip was reproduced first.** With the guess at the end of
`Arrangement::material_is_left_of_forward` turned round for a face on a sphere
and for nothing else, the ball-then-bar union answers, validates, has 11
faces, and measures `357.59371084320134` at deflection `0.02` against
`343 + 2π/3 + 4π = 357.6607657167524` by hand. The guess was reached three
times in the second fuse: the half ball, and the bar's two caps.

**The fix is §340.5's.** `BakedEdge::inside_across` carries, for an edge that
runs right round a closed curve, whether the face lies on the negative side of
the face across it. `bake.rs` reads it off the coedge's direction on its edge
through `a_closed_crossing_is_walked_the_way_it_is_stored`, backwards, the way
`holds.rs` reads a ring's face: in the stored frame, with both gradients as
the solid uses its faces, and turned round where the face across is used
turned round. `material_is_left_of_forward` reads it after the corners and
before the guess, and only where the crossing's partner is the edge's own face
across bit for bit, since a folded crossing's partner can be another spelling
of it whose sides are the other way round. Printed on the way: the half ball's
edge says it lies outside the floor, and both caps' edges say they lie inside
the bar's wall, which is what the guess said of them. The union then answers
with the same 11 faces and the same `357.59371084320134`.

**Bake reads it for every closed edge and refuses what the rule refuses** — a
direction it cannot tell, a closed edge between two planes, a kind of run it
has never seen — rather than only where the arrangement would need it. The one
test outside these files that moved, §340.8's, moved from a refusal to an
answer.

**What moved, all in this lane's files.**
`the_same_three_one_pair_at_a_time_stop_somewhere_else` is
`the_same_three_one_pair_at_a_time_make_the_same_block`, and makes every check
the all-at-once union makes, through one helper. `A_HALF_BALL_TURNED_INSIDE_OUT`
and `the_ball_comes_back_inside_out` are gone from `fuses_that_must_agree.rs`,
and `uniting_a_ball_and_a_bar_with_a_block_does_not_depend_on_which_two_go_first`
compares: 172 fuses, none refused, 43 of 43 comparisons reached. **It failed
the first time it compared**, and not on the shape: the two groupings hand back
the same 11 faces on the same surfaces in a different order, and a whole
solid's measure sums them in that order, so volume, area and centroid differ
in the last place. It compares each face measured on its own now, sorted, bit
for bit, which is the same triangles in either list.

### 340.8 What this leaves

- **`fuse_laws_over_a_cone.rs`'s
  `three_trios_that_refused_both_ways_now_answer_one_way_round` fails**, and the
  file is not this lane's. Its third trio, a pointed cone with its apex a unit
  under a six-block's floor and a cube far away, refused `(block ∪ cone) ∪ cube`
  at the second union with `ACurvedFaceGoesRightRoundWithNoSeam`; it answers
  now. Put among that file's `TRIOS_THAT_AGREE_ABOUT_ANSWERING` for a probe and
  then put back, the associativity there compared 4 trios where it compared 3,
  and held. So the new answer is right, and the trio moves to that list. Why
  this fix reached it was not traced. §340.9 moves it and traces it.
- **The guess is still there** for a closed edge the rule says nothing about:
  one whose run names a third surface, one along which the two surfaces touch,
  and one on a crossing folded into another spelling of its partner. None was
  met here.

### 340.9 The pointed cone's tip is the half ball again

*Queue row Q836, which is Q421 again with `fuse_laws_over_a_cone.rs` in its
files. Lane `lane-q836-a-half-ball-under-the-floor-faces-the-right-way-2`, off
`2ad40445` with `b3e5d2fd` merged, which brings §340.7 onto Q815's, Q665's and
Q821's changes.*

The trio of §340.8 is a block six across, a pointed cone of slope one half and
height four with its apex at `(3, 3, −1)`, a unit under the floor, and a cube
of two at `(20, 20, 20)`. `(block ∪ cone) ∪ cube` stopped at the second union
with `ACurvedFaceGoesRightRoundWithNoSeam`. §340.8 guessed why and did not run
it: the cone's tip hangs under the floor, bounded by one circle with its one
corner on the floor, which is the half ball's shape. **The guess was right.**

Traced with a printed probe on this branch, now removed: each time
`Arrangement::material_is_left_of_forward` got past the corners, it printed the
face, the crossing's partner and the edge's `inside_across`.

- The first union leaves the tip as a cone face whose one wire is one edge, the
  circle of radius one half where the cone meets the floor, with its one
  corner at `(3, 2.5, 0)`. The floor keeps that circle as a hole.
- Nothing cuts the tip in the second union, so it has no corner off the floor.
  The corners gave out there once, for that face against the floor `z = 0`
  with its normal down. Its edge says `inside_across: Some(false)`: the tip lies
  on the floor's outer side, under it. The guess said `true`, the side above.
- Every other time the corners gave out, in either grouping, it was the cone's
  cap and the cone across the cap circle at `z = 3`, and the edge said `true`
  there, as the guess did. So in this trio the fix changes one answer, the
  tip's side.
- With the arrangement put back on the guess and the bake kept, the
  associativity there finds the asymmetry again: `block ∪ (cone ∪ third)
  answered and (block ∪ cone) ∪ third refused at the second union:
  ACurvedFaceGoesRightRoundWithNoSeam`. Where in the second union the
  inside-out tip was refused was not traced.

**Against the shape worked out by hand.** The file compares one grouping with
the other and checks none of its trios against a closed form, so none was
added. The probe measured both groupings anyway: `224.26054293132952` each,
bit for bit, with a bound of `0.011680353250146892` at deflection `0.01`,
against `216 + 8 + π/12 = 224.26179938779916`. They are `1.26e-3` short,
inside the bound.

**What moved, all in this lane's files.** The trio is the fourth of
`TRIOS_THAT_AGREE_ABOUT_ANSWERING` and `TRIOS_THAT_ANSWER_ONE_WAY_ROUND` keeps
two; the associativity floor is 4, and the run prints `associativity, a block,
a cone and a cube: 4 compared, 2 with bit-identical volumes, refusals {}`.
`three_trios_that_refused_both_ways_now_answer_one_way_round` is
`two_trios_that_refused_both_ways_now_answer_one_way_round`.

**That renamed test does not fail without the fix**, though Q836's brief said
it would: once the trio has left its list it holds either way. With
the arrangement back on the guess, the three that fail are
`the_same_three_one_pair_at_a_time_make_the_same_block`,
`uniting_a_ball_and_a_bar_with_a_block_does_not_depend_on_which_two_go_first`
and `uniting_a_block_a_cone_and_a_cube_does_not_depend_on_which_two_go_first`.
The whole workspace, `mise exec -- just t --workspace --no-fail-fast`: 2905
run, 2905 passed, so no test outside these files fails on top of the mesher's
new changes. A floor that a count only rose past would not show there.

### 340.10 A face's box grown by what writing a corner down costs

*Queue row Q694, lane `lane-q694-a-faces-box-grown-by-what-a-corner-cost`, off
`5d28a70c`. The four refusals §340.4 left "not looked into" are the first of
the thirty below.*

`bake.rs`'s box round a face was the box round the face as the model stores
it. The fuse meets the face where the baked surfaces cross, and a stored corner
is that place rounded, so the box could stop a rounding short of the face, and
`arrange.rs`'s `work_out_how_each_crossing_runs` then dropped a cut whose nodes
were all outside it. **The box is grown on every side by
`what_writing_the_corner_down_cost` at its farthest corner now**, on both
paths: the box `view.rs` reads off a face, and the joiner's box round its
derived corners. The doc of `Bounds` says what it allows for, and that it is
still plain `f64` and not grown by the room `Model::validate_solid` grants a
corner off its plane.

**Thirty refusals, one cause, instrumented.** A probe in `provably_misses`,
since removed, printed every point the ungrown and the grown box disagreed
about, with its caller. On the base all thirty fuses below printed, every print
was at `arrange.rs`'s `work_out_how_each_crossing_runs`, and with the growth
applied in `provably_misses` alone the same thirty answered as with it applied
to the boxes themselves. No other fuse of the three tests printed.

- `a_block_below_a_frustum_is_cut_where_the_frustum_has_no_face_and_every_answer_is_right`:
  the four rules of the block leaned 2 from −3 to 1, turned 0. The ridge's two
  meets with the cone at `x` from `1.341640786499873` to `1.3416407864998736`,
  the box from `1.341640786499874`. All four answer right and measure; the
  family goes from 64 right to 68. The union decides 6986 grid points and
  measures `529.0849648866143` with an error of `0.20056639955312652`, so it
  is held to the 5 500 floor of an answer that measures and not the 7 000 of
  one that does not; no floor moved. The intersection decides 17954, the cone
  less the block 16361, the block less the cone 8579. Q422's trial had three
  of them not measuring; Q423 landed between.
- `no_sliver_chopped_by_the_blocks_own_sides_folds_at_any_deflection`: eight
  chopped slivers, leans 1.35 and 1.5 turned 0. 270 answered to 278, 2700
  meshes to 2780, none folded.
- `a_wedge_cut_by_one_prism_whose_edge_is_the_polar_axis_is_answered_or_refused_by_name`:
  eighteen cases, (90, 0.001), (90, 3), (45, 0.001), (170, 0.001),
  (170, 1000) and (170, 3) by every rule. At radius 0.001 a prism face's box
  began at `y = 0` and both poles came out at `y = −1.78e−35`. All 48 cases
  answer, validate and measure against the closed form now.

**The checked-in fuzz corpus moved too**, in a file that is not this lane's.
`every_seed_in_the_checked_in_corpus_is_fused_without_panicking` reaches
`ok: 378, refused: 312` with the growth off and `ok: 426, refused: 264` with it
on, so its floor of 300 refusals fails. Fifty-four seeds changed. Most went from
`ResultIsNotManifold` to an answer, and every answer passes the sweep under its
own policy. One went the other way: seed `f117319f6357` answered and now
refuses `order along a crossing`. The same probe, put on `Bounds::overlap` too
and run on that seed alone, printed once and not in `provably_misses`: at
`cuts.rs`'s whole-solid test, whose two boxes missed in `z` by
`1.344974619049452e-284`, one starting at that height and the other stopping at
`2.78e-308`. Grown, they overlap, and the pair is no longer passed over as
apart. That one is not the dropped cut; it is the growth reaching the box test
§400.3 asked for.

**Landed by Q933 on `707a8956`, on top of Q920 and Q925, and the replay's
refusal floor set again.** On the merged tree the three tests above pass on the
same pins: 68 right; 278 answered and 2780 meshes; all 48 wedge cases. The
replay reads `ok: 428, refused: 262`, and with the growth switched off by a
probe, since removed, `ok: 380, refused: 310`. So the growth moves the same 48
fuses here as on `5d28a70c`, and the two between the branch's `426, 264` and
these came with what landed between, grown or not. The file's rule is a floor
at roughly three quarters of what a run measured, so the refusal floor is 195,
three quarters of 262. Every other floor of the replay is still at most three
quarters of its count: `ok` 428 against 190, `solids_made` 447 against 190,
`ok_that_wrote_something` 411 against 180, `asked_both_ways_round` 108 against
80, `the_same_both_ways_round` 40 against 14.

**Seed `f117319f6357` refusing is the safe outcome, and no row is filed for
it.** On the merged tree it refuses `Undecidable` at `order along a crossing`.
A probe in `cuts.rs`'s `solids_within_reach_of_each_other`, since removed,
printed the two whole-solid boxes. Ungrown, one runs in `z` from
`1.344974619049452e-284` to 10 and the other from −10.08 to
`2.7813423231340017e-308`, so the boxes miss, the pair is passed over and the
fuse answers. Grown, they run from `−1.42e−13` and to `2.13e−13`, so they
overlap. Either way `no_face_of_one_meets_the_other` could not prove the two
solids apart, so the base's answer rested on the plain-`f64` box test alone,
which is not a proof (§400.3). The gap between the two solids is about
`1e−285` of their largest coordinate, 15: a spread of about `1e285`, far past
the cliff §3.5 measures at `2^49` for a constructed point, where §3.5 says a
refusal is the safe outcome and a confident answer is not.

### 340.11 What this leaves

- `cuts.rs`'s doc of `two_faces_provably_apart` and §400.3 say `bake`'s boxes
  can be short by a rounding and that growing them would close that; they are
  grown now.
- The box is not grown by the room `Model::validate_solid` grants a corner off
  its plane, nor rounded outward round a curved side (Q873).
