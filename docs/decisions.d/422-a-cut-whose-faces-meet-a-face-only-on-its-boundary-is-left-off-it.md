## 422. A cut whose faces meet a face only on its boundary is left off it — 2026-10-09

*Queue row Q1470, `refuses/B`, which finishes Q1453. Two runs wrote the code:
the first on `lane-q1470-a-bar-cut-through-the-rings-axis` (`6a8790a4`,
`c37e0875`, `7d0a181f`), merged by `2e1c1c8f` into the second,
`lane-q1470-the-half-spaces-a-face-lies-in` (`4bdc944d`, `bf71a10c`,
`ee1a4500`, `12153478`, `ee6a2789`). A third run,
`lane-q1470-the-boundary-cut-written-down`, off `a90cd02d`, wrote this section
and the doc comments.*

### 422.1 The shape

A bar of a bend's tube radius butted onto the bend's `+x` cap, its far end cut
by the plane `x + y = 0` through the ring's axis, rings 3/1 and 5/2
(`a_bar_cut_through_the_rings_axis_on_a_bends_cap_by_every_rule_against_the_closed_forms`
in `rcad-topo`'s `tests/fuses_that_touch.rs`). The bar's pipe meets the ring
in the cap's rim and in a loop, `x = R − y²/4R`, that crosses the rim at
`(R, 0, ±r)`. Q1453 named the loop in two halves, one each side of the cap's
plane `y = 0` (`where_a_ring_and_a_pipe_along_its_core_cross` in `rcad-geom`'s
`intersect.rs`). The half with `y < 0` lies on the bar's pipe face, past the
bend's cap, and the half with `y > 0` on the bend's ring face, past the bar's
cap. No edge of any answer runs along either.

On the base the bar's pipe face drew its half, with two nodes where the far
plane crosses it at `x = −y = 2R(√2 − 1)`, `(2.485, −2.485, ±0.857)` for ring
3/1. Points along a quartic cannot be put in order, so all 24 fuses (three
rules both ways round, certified and tolerant) were refused
`ACrossingThisCannotBeOrderedAlong`. Now all 24 give the closed forms the pin
checks and validate. The answer existed and the arithmetic could not reach it,
so the refusal was a gap (§16.10).

### 422.2 The rule

A face `F` is cut by a surface `S` because some face `G` carrying `S` may
cross it, and where it does the crossing is an edge of the answer. **If every
face carrying `S` meets `F` only on `F`'s boundary**, then no point of the
crossing inside `F` is on any of them. Inside `F` the crossing is then no
boundary of either solid, `F` on one side of it is inside or outside each
solid exactly where it is on the other, and leaving it off loses no edge.

`meets_it_only_on_its_boundary` in `fuse/cuts.rs` proves it by a plane that
`G` lies on one closed side of, in one of two ways:

- **(a) `F` lies on the other closed side of the same plane.** Then the two
  share points of that plane alone, and `F`'s inside misses the plane, so what
  they share is on `F`'s boundary. The planes are compared by
  `PlaneAgreement`: the same way round with opposite sides, or opposite ways
  round with the same side.
- **(b) The box round `F` is strictly on the other side.** Then the two share
  nothing at all.

`the_cuts_that_reach_only_its_boundary` keeps a surface only when every
finding of it on `F` says so, so a surface carried by two faces stays on
wherever one of them does not. That is how Q1090's rule, in the same file,
treats a surface carried by two faces.

Q1090's rule leaves off a crossing that shares no point with anything else on
the face. This one leaves off crossings that do, so it needs the guard of
422.4, which Q1090's does not.

**In the pin**, the bend's ring face lies in `y ≥ 0` and `x ≥ 0`, and the
bar's pipe face in `y ≤ 0` and on the bar's side of `x + y = 0`. By (a) on
`y = 0`, the ring is left off the pipe face and the pipe off the ring face.
By (b), the far cap's box is strictly in `y < 0`, so the ring is left off the
far cap as well. That takes off the far cap the circle `x + y = 0` cuts the
ring in, which runs through the same two points of the far rim. Without that,
the rim would be split on one side only, which the row's text warned of. This
paragraph is read off the code; no probe printed it. The probes of 422.7
dropped the same crossings by hand, and 422.5 records the one that was printed.

### 422.3 The sides a face lies in

`BakedFace::lies_in`, filled by `the_sides_of_planes_it_lies_in` in
`fuse/bake.rs` (`bf71a10c`), holds the closed sides of planes the whole face
is proved to lie on. It is filled for one input face and never for a group
joined from several, since a group need not be in one piece: a bar with a
groove round it has its wall in two pieces, one each side of the groove.

- **A pipe's face**: every plane across a closed edge. Such a plane leans to
  the axis, so it crosses the pipe in that one ellipse and nowhere else. The
  face's inside misses the edge, so it misses the plane, and lies on the side
  `BakedEdge::inside_across` names.
- **A ring's face between two planes through its axis**, with no other edges:
  both planes, when each edge's corner is strictly on the kept side of the
  other plane. That is the less-than-a-half-turn test `holds.rs` makes of the
  same face. Each plane crosses the ring in two circles, the edge and its
  mirror across the axis, and the face lies inside both sides only where it
  turns less than a half-turn.
- Every other face: nothing. A ball's or a cone's face between planes would
  be sound to do too, and nothing needs it yet (§13).

`a_round_bars_wall_lies_on_the_side_of_each_cap_the_bar_is_on`, in `bake.rs`'s
tests, pins `inside_across` read as the negative side of the plane across,
on a bar at rest and turned. Read the other way round, the wall would lie
outside both caps.

### 422.4 A node of a cut still splits what runs through it

A crossing left off a face takes its nodes with it, and a node splits every
crossing through it. An edge of the answer along one of those is split there by
the face across too, unless that face leaves the same cut out. So
`leave_out_the_cuts_that_reach_only_the_boundary` in `fuse/arrange.rs`
(`ee1a4500`), which runs after the meets are placed, leaves a crossing off only
where `splits_nothing_kept` says that every node of it is no input corner, and
either:

- is provably off the face (422.5), or
- has no other crossing through it that is drawn and kept, except boundary
  crossings the faces across split there no more than this face will
  (`the_faces_across_split_it_no_more`). Those faces' choices reach this face
  through `BakedEdge::its_neighbour_leaves_out`, which `cuts.rs`'s
  `tell_each_edge_what_its_neighbour_leaves_out` fills.

It runs to a fixed point, since a crossing kept makes its nodes count against
the others through them.

**The guard is measured, not argued.** The second run's last lane cut
`splits_nothing_kept`'s test down to `node.vertex.is_none()` alone, in a probe
it did not commit, and ran `mise exec -- just t -p rcad-topo --no-fail-fast`:
1716 passed and 26 failed. Every failure was a refusal, `ResultIsNotManifold {
count: 1 }` or "an edge would have 1 faces along it", except one tally
assertion in `how_often_a_curved_fuse_refuses.rs`'s
`every_answer_over_a_block_turned_against_a_bar_is_a_solid_that_validates`,
which holds its count of fuses answered to every fuse it makes. No volume was
wrong.

It printed why on `a_bar_standing_against_a_blocks_wall_answers_every_rule`
(`tests/a_bar_lying_along_a_plane_or_a_bar.rs`): a cube of side 4 and a bar of
radius 1 from `z = 1` to `z = 3` whose axis runs through `(5, 2)`, touching the
cube's wall `x = 4` along the line `(4, 2, z)`. The bar's wall lies between its
caps, so the cube's floor `z = 0` and top `z = 4` are strictly past them, and
by (b) the rule alone would leave the pipe off both. On the floor, the pipe's
circle reaches the face only on its boundary, at `(4, 2, 0)`, a node on the
floor's edge along `x = 4`. But the wall `x = 4` is flat, lies in no side of
anything, and keeps the touching line through that node. So the wall splits
its edge with the floor there, and the floor must split that edge there too,
or the edge is in two on the wall's side and whole on the floor's, which is
not a manifold. The same goes for the top at `(4, 2, 4)`. The guard keeps the
circle on both: the node is on the face, and the crossing along `x = 4`
through it is a boundary crossing whose face across, the wall, does not leave
the pipe out.

### 422.5 Off the face by a plane

With the guard as `ee1a4500` first wrote it, a node let its crossing go only
when it was outside the box round the face, and the pin was still refused. The
box round the bend's ring face holds the far rim's corners `(2.485, −2.485,
±0.857)`, so the half of the loop through them was kept on the ring face and
refused ordering there.

A node strictly past a plane the face lies on one side of is off the face just
as surely: the ring face lies in `y ≥ 0`, and the node is at `y = −2.485`.
`is_provably_off_the_face` asks both, and `ASideOfAPlane::past_it` gives the
sign a point past the plane has (`12153478`). That commit's message records the
pin passing.

### 422.6 One surface, two spellings

`crossing_like`, in `arrange.rs`, folds a second spelling of one surface into
the crossing the first drew: a pipe's axis slid along itself or turned round,
or on a flat face another plane through the same line. Before `ee6a2789` it
forgot the second spelling. Two bars stacked on one pipe, the upper wall's
axis slid up to `z = 2` or turned round, cut a box's faces in one curve under
two sets of numbers. The lower wall's spelling meets the box nowhere, so it
reached only the boundary, and the upper's crosses the box. The crossing kept
the lower spelling as its partner and was left out, and three tests in
`tests/two_bars_on_one_pipe_and_a_box_united_in_one_call.rs`
(`OnTheUpperWallOnly` with `OriginSlidUp`, and the window opposite the seam
with `AxisTurnedRound`) were refused `ResultIsNotManifold { count: 1 }`.

Each crossing now keeps every spelling folded into it, in
`Crossing.also_drawn_by`, and `Crossing::every_surface_that_draws_it` gives
the partner, then `also_cut_by`, then `also_drawn_by`. The drop step and
`the_faces_across_split_it_no_more` read that. By the second run's last lane,
those three tests pass on the base, fail at `12153478` and pass at
`ee6a2789`. So they guard this fix, but they are not tests that fail without
the row.

### 422.7 The cap's rim, named once

With the loop halves left off by hand (the probes of `c37e0875`, reverted by
`4bdc944d`), every union was still refused `ResultIsNotManifold { count: 1 }`.
`the_name_each_closed_curve_goes_by`, in `fuse/assemble.rs`, joined no
`[plane, ring]` name, so the cap's rim came out as two closed edges, one face
along each: `[pipe, cap plane]` starting at the bar's seam corner
`(R + r, 0, 0)`, and `[ring, cap plane]` at the bend's `(R − r, 0, 0)`.

Now a `[plane, ring]` name that says which circle it is, for a plane through
the ring's axis, joins a `[plane, pipe]`, `[plane, ball]` or `[plane, cone]`
name on the same plane. It does so when the three surfaces share more than a
point and the half-space that picks the named circle holds the other name's
start (`7d0a181f`). The other conic shares more than four points with the
plane's two circles on the ring, so it is one of them, and the start says
which.

### 422.8 What this leaves

- **A face that drops a run its neighbour keeps is refused, not answered
  wrong**, on the evidence of 422.4: of the 26 tests that failed with the
  guard cut down to the corner test, 25 failed on a refusal and one on its
  count of fuses answered, which it holds to every fuse it makes, and none on
  a wrong volume. That is evidence from those tests,
  not a proof.
- **Not run: a third spelling the face never sees.**
  `its_neighbour_leaves_out` still holds surfaces as the neighbour spelled
  them, and `the_faces_across_split_it_no_more` compares by spelling. If a
  neighbour leaves out one spelling of a surface but keeps its run, because
  another spelling that this face never sees draws the run there too, this
  face could drop a run the neighbour keeps. By 422.4's measurement that would
  be a refusal. Nobody has built the shape. Row.
- **Row, small**: the doc comment of `where_a_ring_and_a_pipe_along_its_core_cross`
  in `rcad-geom`'s `intersect.rs` still says the fuse does not leave the loop's
  halves off and is refused ordering the one on the bar's pipe face. Since this
  section it leaves them off.
- Not a row: a ball's or a cone's face between planes (422.3), with no
  consumer (§13).
