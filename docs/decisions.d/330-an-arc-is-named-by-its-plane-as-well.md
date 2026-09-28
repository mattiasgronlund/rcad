## 330. An arc is named by its plane as well — 2026-09-28

*Q114, from §247.6 and §251.6. Lane `lane-q114-name-the-arcs-plane`, off
`09543b7e`. `which_arc_of_a_circle_this_is` in
`crates/rcad-topo/src/fuse/assemble.rs` now names an arc of a closed crossing
by the plane it lies on as well as by its curved surface, both folded either
way round. The ball less a block over one corner gets past `assemble` and stops
in the mesher; the cone's halved-slope control answers and measures right; 32
cases of `a_narrow_lune_from_a_pole_corner.rs` and the three refused fuses of
the exact `z = y` plane in `a_plane_leaning_through_a_bars_rim_start.rs` now
answer, and every one was held to its closed form. Those two files are not
this lane's, so their pins are left for the merge (330.5).*

  330.1   The change
  330.2   The two shapes the row names
  330.3   What the plane costs here, and why it is not what §245 said it cost
  330.4   Whether a spelling can pair four faces crosswise
  330.5   Pins in other files that moved, and why each new one is right
  330.6   What this leaves

### 330.1 The change

The third part of `EdgeName` went from `Option<([u64; 9], bool)>` to
`Option<([[u64; 9]; 2], bool)>`: the curved surface's name and the plane's,
each `name_of_the_surface_either_way_round`, beside the sign that says which of
the crossing's two arcs it is. It is `None` in exactly the cases it was before,
so no side that did not carry the curved surface carries the plane.

§247.2 kept the plane out on the ground that every arc on one plane has that
plane. That is true of two crossings on one plane (§245.4) and says nothing
about two crossings on one curved surface, which is the case here.

### 330.2 The two shapes the row names

**The ball less the block.** A ball of radius five at the origin less the
block `x ∈ [−10, −3]`, `y ∈ [−10, 10]`, `z ∈ [0, 10]`. The planes `x = −3` and
`z = 0` cut the ball in two circles through `(−3, ±4, 0)`, and an arc of each
runs between those corners on the skin. On the base the difference refuses
`ResultIsNotManifold { count: 4 }` (run on the base tree with
`mise exec -- just t -p rcad-topo --test a_ball_less_a_wedge`). With the change
it refuses `ACurvedFaceGoesRightRoundWithNoSeam` on the answer's `FaceSlot(2v1)`,
in the sweep `Assembly::commit` runs over a throwaway model.

That refusal is the mesher's, and a temporary print (since removed) in
`mesh.rs` showed why. The skin comes out as three pieces of the ball, which the
joiner never joins: above the equator beyond `x = −3`, and below it either side
of `x = −3`. The first goes right round and holds the north pole, so
`cut_off_what_goes_too_far_round` cuts it as a cap, and
`which_pole_this_cap_reaches` reads the pole off the surface across the cap's
**first** coedge. That coedge is the equator's long arc, `EdgeSlot(3v1)`, and
both faces along it are the ball:

```
PROBE cap FaceSlot(2v1) first coedge's edge EdgeSlot(3v1) carries [(Sphere r=5, true), (Sphere r=5, true)]
```

So `find(|surface| surface != mine)` is `None`, and the cap is refused as one
with no seam. `mesh.rs` is not this lane's, so
`a_ball_less_a_block_over_one_corner_of_it_stops_in_the_mesher_on_a_cap_with_the_ball_both_sides_of_its_first_edge`
in `crates/rcad-topo/tests/a_ball_less_a_wedge.rs` pins that refusal, and its
doc carries the answer it stands in for: `500π/3 − 26π/3 = 158π`. The same
file's other test holds the three fuses that answered before (both
intersections, `26π/3`, and the block less the ball, `1400 − 26π/3`), each
validated and within its round wall's area times the deflection, on the side
the mesh must fall.

Reading the second coedge would not be a fix on its own. Its partner is
`x = −3`, and both poles are on the material's side of that plane, so the
rule's premise — that the partner puts the two poles on different sides — does
not hold there either.

**The cone's halved-slope control** in `degenerate_curved_fuses.rs`, which §251
moved to `ResultIsNotManifold { count: 4 }`, now answers. The union is held to
the frustum `π s² (28³ − 4³)/3 = 89.5354`, plus the wedge's 96, less what the
two share, worked out in the test without the kernel: each level's disc of
radius `z/16` clipped by the wedge's three slanted faces, added up by Simpson's
rule from `z = 8` to `z = 24`. Measured once with a print, since removed:

```
frustum 89.5353906273091 shared 14.924433028016479 measured 170.59291537217484 at 0.002, 170.4474362889243 at 0.02
```

The closed form is `170.6110`; the file's deflection of 0.02 comes in 0.096%
under it, inside its 0.5%.

### 330.3 What the plane costs here, and why it is not what §245 said it cost

`which_curve_it_is`'s doc records the cost §245.5 and §247.2 quote: with both
surfaces in **every open side's** name, an edge between two solids pressed
flush was split, and the skew family's not-a-solid share rose from under three
in a hundred to three and a half. Two things keep that away from this name.

- **Only a side of a closed crossing between a plane and a ball, a cone or a
  pipe reaches it.** `which_of_them_a_closed_crossing_runs_round` answers
  `None` unless one surface is a plane and the other is cut by it in a curve
  that closes. The skew family, and every other all-flat family the pins in
  `fuses_that_must_agree.rs` are measured on, has no such side, so its names
  are the same bits as before. Those pins all held: the whole crate was run
  with `mise exec -- just t -p rcad-topo --no-fail-fast`, 1084 tests, 1080
  passed, and the four that failed are 330.5's.
- **Those crossings already carry the plane when they close.** A closed edge is
  named by both its surfaces, folded the same way, in `which_curve_it_is`, and
  every circle a plane leaves on a ball, cone or pipe is such an edge when
  nothing cuts it. An arc of that crossing now carries what the whole circle
  already did.

Two faces along one arc spelling its plane with different bits is the hazard
the cost came from. A flat piece carries its own input plane, and the curved
piece along it carries the plane of the face that cut it, which is that same
face's unless `Arrangement::crossing_like` merged a second face on the same
point set into it. `intersect` calls two planes one surface only when their
coefficients are exactly proportional, and the constructors that scale a
normal (`Plane::from_point_and_normal`, `Plane::from_coefficients`) store one
that is a function of its line alone, so two planes built by them on one point
set are one spelling or its exact negation, and the fold takes the negation.
`Plane::transformed` does not scale again, so a plane moved by a turn could be
exactly proportional to another by a factor other than `±1`. No test here
builds one, nothing here checked whether a turn can produce one, and a closed
edge between such a pair would be split the same way.

### 330.4 Whether a spelling can pair four faces crosswise

§247.6 asked whether two spellings the fold leaves apart could pair the four
faces along two arcs crosswise, into two edges that each look sound. **With the
plane in the name, no spelling the fold leaves apart can, and on the base one
could.**

Two faces are paired when their keys agree: the corner pair, the curved name,
the plane name, the sign and the straight flag. Crosswise means a face of one
arc and a face of another arc agreeing. Two different arcs between one corner
pair either lie on two crossings or are the two arcs of one crossing, and that
is the whole family.

- **Two crossings** differ in their curved surface or their plane as point
  sets. Every step of `name_of_the_surface_either_way_round` is exact — it
  negates, writes an axis one way, or slides a pipe's origin only where that is
  exact, and `the_axis_origin_slid_the_one_way`'s doc measures it joining
  nothing that is not one line — so a name fixes its point set, and two
  crossings never share a name. A spelling the fold leaves apart makes two
  names of one surface differ; that splits the one arc it is on into two edges
  with one face each, which is refused, and never makes the faces of two arcs
  agree.
- **One crossing's two arcs** are told apart by the sign alone, which 330.6's
  last item is about.

On the base the plane was not in the key, so the first case was open: take the
ball less the block with the ball's skin spelled one way and both flat faces
naming it another way the fold leaves apart. The skin's two sides would share
one key and the two flat faces' sides another, two edges of two faces each,
each pairing the wrong faces. A ball has no spelling the fold leaves apart —
its centre and radius are all it holds — so the shape needs a slanted pipe
spelled two ways, which reaches `assemble` only where two coincident walls
meet, and §247.3's two tries at that (two half bars stacked on one pipe, two
whole bars stacked with their seams opposite) both stopped before it or never
named an arc. Nothing was built here either; the argument is what closes it.

### 330.5 Pins in other files that moved, and why each new one is right

Neither file is in this lane's write set, so both need their pins changed where
this lands. Each move is from a refusal to an answer, or from one count to
another, and each new answer was checked before being called right.

- **`crates/rcad-topo/tests/a_narrow_lune_from_a_pole_corner.rs`.** 32 cases
  pinned as refused now answer: in
  `a_wedge_cut_by_one_prism_whose_edge_is_the_polar_axis_is_answered_or_refused_by_name`,
  Difference and Union at `(90, 1)`, `(90, 1000)`, `(45, 1)`, `(45, 1000)`,
  `(45, 3)`, `(10, 1)`, `(10, 0.001)`, `(10, 1000)`, `(10, 3)`, `(170, 1)`; in
  `a_wedge_cut_from_a_ball_by_two_knives_through_its_polar_axis_is_answered_or_refused_by_name`,
  Difference and Union at `(10, 1)`, `(10, 3)`, `(170, 1)`, `(170, 0.001)`,
  `(170, 1000)`, `(170, 3)`. **Right because** an uncommitted copy of the file,
  whose moved-case branch validated the model and ran the file's own
  `measures` against the case's closed form instead of reporting it, passed all
  six tests, and printed 32 volumes each under its closed form within
  `3δ/r` — for instance `98.9098` against `98.9602`, `3.13966` against `π`.
  No case moved from answered to refused.
- **`crates/rcad-topo/tests/a_plane_leaning_through_a_bars_rim_start.rs`.**
  `a_plane_exactly_through_a_bars_rim_start_is_ordered_and_two_arcs_on_its_wall_get_one_name`:
  both unions and the bar less the prism answered `count: 4` because an arc of
  the cap circle and an arc of the ellipse on `z = y` join the same corners on
  the wall — this row's shape on a pipe. All six fuses now answer under both
  policies, validate, and are within `measure`'s own `volume_error` of the
  file's by-hand volumes: the unions `32701.787 ± 2.641` against `32702.065`,
  the bar less the prism `701.787 ± 2.641` against `702.065`, the prism less
  the bar `31916.715 ± 0.496` against `31916.667`, the intersections
  `83.284 ± 0.496` against `83.333`.
  `under_a_tolerant_policy_the_turned_block_is_ordered_and_its_faces_disagree_on_a_corner`:
  the two fuses pinned `count: 3` (the union and the difference with the bar
  as the argument) now refuse `count: 1`, like the other four. That fits the
  four-face edge being split into two good ones and the corner the faces
  disagree on being what is left, but no print confirmed it.

The row in 330.6 names the exact edits.

### 330.6 What this leaves

- **Pin edits at merge**, outside this lane's write set:
  `a_narrow_lune_from_a_pole_corner.rs` adds the 32 cases of 330.5 to the two
  tests' answered lists and to the file header's counts;
  `a_plane_leaning_through_a_bars_rim_start.rs` measures all six exact-plane
  fuses instead of expecting `count: 4` for four of them, rewrites that test's
  doc and the file header's paragraph that says they refuse, and changes the
  tolerant test's counts from `[3, 1, 1, 1, 3, 1]` to all `1`.
- **The ball less the block, in the mesher.** `which_pole_this_cap_reaches` in
  `crates/rcad-topo/src/mesh.rs` needs a partner that puts the two poles on
  different sides, read off the coedge it takes its step along, and not the
  first coedge's. When it has one, the pin in
  `crates/rcad-topo/tests/a_ball_less_a_wedge.rs` becomes a measurement against
  `158π`.
- **The sign reads the plane's orientation as each face spells it.**
  `which_arc_of_a_circle_this_is` decides which arc a side is by
  `∇round × ∇plane`, and the plane's gradient turns round with its spelling, so
  a face naming the plane facing the other way gets the other arc's sign. That
  was so before this change and the fold of the plane's name does not touch it.
  If two faces along one arc can spell the plane opposite ways — two flat input
  faces on one plane facing apart, the curved piece cut first by one and
  bordered in the answer by the other — the arc is split and refused; if both
  arcs of one crossing have such a pair, the four faces pair crosswise. No
  shape was built for it. `crates/rcad-topo/src/fuse/assemble.rs`, and a test
  file of its own.
