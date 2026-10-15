## 330. An arc is named by its plane as well — 2026-09-28

*Q114, from §247.6 and §251.6. Lane `lane-q114-name-the-arcs-plane`, off
`09543b7e`. `which_arc_of_a_circle_this_is` in
`crates/rcad-topo/src/fuse/assemble.rs` now names an arc of a closed crossing
by the plane it lies on as well as by its curved surface, both folded either
way round. The ball less a block over one corner gets past `assemble` and stops
in the mesher; the cone's halved-slope control answers and measures right; 32
cases of `a_narrow_lune_from_a_pole_corner.rs` and the three refused fuses of
the exact `z = y` plane in `a_plane_leaning_through_a_bars_rim_start.rs` now
answer, and every one was held to its closed form. Those two files were not
that lane's, so a second lane, `lane-q114-name-the-arcs-plane-2`, off
`740ae21a` with the first merged in, changed their pins and printed what the
tolerant turned block's refusal now is (330.5, 330.6).*

  330.1   The change
  330.2   The two shapes the row names
  330.3   What the plane costs here, and why it is not what §245 said it cost
  330.4   Whether a spelling can pair four faces crosswise
  330.5   Pins in other files that moved, and why each new one is right
  330.6   What this leaves
  330.7   Which of Q210's two questions this answers

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
`a_ball_less_or_united_with_a_block_over_one_corner_of_it_is_measured_against_the_piece`
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

**2026-10-05 note (Q1085, fixed by Q555, `c34513e6`): a turn does produce one.**
A block turned about `z` by `0.29` then `0.4953981633974484`, and one turned by
`0.45` then `0.33539816339744805`, both put a face on `x = y`, spelling it
`(−c, c, 0, 0)` and `(−c′, c′, 0, 0)` with `c` and `c′` an ulp apart
(`a_turned_plane_exactly_proportional_to_another.rs`). The arc's name now
folds any ratio, not only the negation
(`name_of_the_plane_at_any_scale` in `assemble.rs`), so the twelve fuses this
paragraph worried about answer. Two blocks facing apart across such a pair are
still refused — by their corners, not the arc's name — and a closed edge
between such a pair, rather than an arc of one, is still untested; neither is
this note's to close.

**2026-10-07 note (Q1222, bringing the note above up to date): a closed edge
between a round solid and such a pair is tested now, by Q1084
(`044d82d2`).** A ball turned about `x` so neither pole is on `x = y`, and a
bar of radius two square to the plane, each cut in one whole circle by
`x = y`, with one block's face holding the whole circle spelled `c` and a
second block clear of every round solid spelled `c′`. Every one of the twelve
fuses of each was refused `ResultIsNotManifold { count: 1 }` until
`name_of_the_closed_crossing_through` named the plane with
`name_of_the_plane_at_any_scale` there too, and now each answers its closed
form
(`a_ball_cut_whole_by_a_plane_two_blocks_spell_an_ulp_apart_in_scale`,
`a_bar_cut_whole_by_a_plane_two_blocks_spell_an_ulp_apart_in_scale`, in
`crates/rcad-topo/tests/a_turned_plane_exactly_proportional_to_another.rs`).
Two blocks on opposite sides of the plane are still refused, by their
corners rather than the arc's name
(`two_blocks_facing_apart_across_a_plane_spelled_an_ulp_apart_in_scale_are_still_refused`),
and two overlapping flat blocks on such a plane, with no closed crossing in
it, are still refused one way round with nobody having traced it
(`two_blocks_overlapping_on_a_plane_spelled_an_ulp_apart_in_scale_are_refused_one_way_round`),
filed by Q1084's lane as Q1221.

**2026-10-09 note (Q1223, fixed by `27157509`): a rim's two names join at any
scale too.** `the_name_each_closed_curve_goes_by` joined a rim's two halves
only where `one_surface` called their planes one bit for bit; it now folds the
scale there as well. A ball on the cap of a bar of its own radius, the cap
turned about `x` onto `y = z`, and a block clear of both turned onto the other
spelling, was refused `ResultIsNotManifold { count: 1 }` in every fuse with
the bar spelling the plane `c′` before the fix — the rim's two names, the cap
with the bar's wall and the plane with the ball, only joined where the two
planes were spelled alike or negated
(`a_ball_on_a_bars_cap_a_block_spells_an_ulp_apart_in_scale`). Every fuse
answers now.

§330.3 also guessed that `which_of_the_pairs_closed_crossings_it_is` could
name one of a ring's two circles on such a plane twice. Three families were
built against a half bend of ring four and tube one to reach it, and none did.
Blocks turned onto `x = y` are refused `ACrossingBetweenAPlaneAndATorusThatIsAQuartic`
before the picker is ever asked, in every fuse, with the two spellings alike
too, so this family cannot show the guess either way; no row follows it. Two
bars cut back to `x ≥ 0` and turned onto `x + y = 0` are refused
`AFaceTrimmedByACurveThisCannotFollow` at a bar's bottom cap before the picker
is asked too, for the same reason, filed by Q1223's lane as **Q1496**. Two
bars cut back to the quarter `x ≤ 0`, `y ≥ 0`, one holding the whole circle at
`3π/4` round the ring on its face, answer every fuse with both spelling the
plane alike, and are refused `Undecidable` on `side of plane` with the two
spellings mixed — a sign that could not be settled, not a circle named twice,
so this does not show the guess and does not show that no shape reaches it
either — filed by Q1223's lane as **Q1495**. Once that answers,
**Q1497** runs every fuse of both mixed pairings to see whether the circle is
shown named twice there.

**2026-10-09 note (Q1392, `1384425d`): the quarter bars are refused two ways
now.** Q1392 places a point on the ring against a plane through its axis
exactly (`side_of_a_plane_holding_a_rings_axis`). With the bend named first,
all 12 mixed fuses are now refused `CannotTellTheCellsOfAFaceApart` on the
bend's ring face: `Arrangement::crossing_like` takes the two spellings for one
surface but their pickers, the ring's axis crossed with each stored normal,
for two, so each circle is drawn twice. That is this section's unscaled
half-space, met in arrange before `which_of_the_pairs_closed_crossings_it_is`
names anything; **Q1538** folds it. With the bars named first, all 12 are still
refused `Undecidable` on `side of plane`, at a corner on a bar's round wall;
Q1495 is re-filed for those alone as **Q1539**, and Q1497 waits on both.

**2026-10-09 note (Q1496, fixed by `bced8a48`): the half bars get past the
cap.** The refusal at a bar's bottom cap was raised in the fuse's arrangement,
`which_way_an_edge_runs_round` in `fuse/arrange.rs`, and not in `holds.rs` where
Q1496's lead put it. The cap's arc is half its circle, so its chord runs
through the centre, and its picker is the plane through that chord. Turned, the
fuse solves both ends into boxes that plane runs through, and the centre is on
the plane and on the chord line, so none of the four ways `the_way_into_an_arc`
had could say which half it is. A fifth way, which asks where the line through
the centre square to the chord meets the circle, reads it (§427). One half bar
turned by `0.3`, `π/4`, `0.7853`, `1` or either of Q555's pairs now answers all
six fuses with the half bend
(`a_half_bar_turned_about_a_half_bends_axis.rs`). Two half bars with the
spellings mixed are refused by name as the quarter bars are, `Undecidable` on
`side of plane` with the bars named first and `CannotTellTheCellsOfAFaceApart`
with the bend named first. Whether by Q1539's and Q1538's causes is not
traced, so they still cannot show the guess either way; §427.4 has what was
run.

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

Neither file was in the first lane's write set; the second lane changed both.
Each move is from a refusal to an answer, or from one count to another, and
each new answer was checked before being called right. With `assemble.rs` put
back to `740ae21a`'s, both files' changed tests fail — every one of the 32
lune cases refused `ResultIsNotManifold { count: 4 }`, the exact plane's union
refused the same, and the tolerant turned block's union refused `count: 3` —
and with it as this section leaves it, all nine tests in the two files pass
(`mise exec -- just t -p rcad-topo --test a_plane_leaning_through_a_bars_rim_start
--test a_narrow_lune_from_a_pole_corner`).

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
  No case moved from answered to refused. The file's answered path,
  `moved_from_its_pin`, already validates and measures every case it lists, so
  the 32 are now in the two `answered` lists and held to their closed forms
  there, and the header's counts are counted off those lists: 29 wedges cut by
  two knives (11 intersections, 12 unions, 6 differences) and 30 by one prism
  (10 of each). The header had said 13 intersections and 4 unions for the
  knives' 17, which were 11 and 6.
- **`crates/rcad-topo/tests/a_plane_leaning_through_a_bars_rim_start.rs`.**
  `a_plane_exactly_through_a_bars_rim_start_is_ordered_and_two_arcs_on_its_wall_get_one_name`
  — §284.6's name for it — is now
  `a_plane_exactly_through_a_bars_rim_start_answers_all_six_fuses_right_under_both_policies`:
  both unions and the bar less the prism answered `count: 4` because an arc of
  the cap circle and an arc of the ellipse on `z = y` join the same corners on
  the wall — this row's shape on a pipe. All six fuses now answer under both
  policies, validate, and are within `measure`'s own `volume_error` of the
  file's by-hand volumes, which the test works out from the bar and the prism
  and no longer takes as written numbers. A print in the test, since removed,
  gave the same figures under both policies: the unions
  `32701.787 ± 2.641` against `32702.065`, the bar less the prism
  `701.787 ± 2.641` against `702.065`, the prism less the bar
  `31916.715 ± 0.496` against `31916.667`, the intersections
  `83.284 ± 0.496` against `83.333`.
  `under_a_tolerant_policy_the_turned_block_is_ordered_and_its_faces_disagree_on_a_corner`:
  the two fuses pinned `count: 3` (the union and the difference with the bar
  as the argument) now refuse `count: 1`, like the other four. **A print
  confirmed why, and it was not a four-face edge.** A temporary `eprintln!` in
  `darts_along_each_edge` listed every edge not claimed by exactly two faces,
  with its corners and its two surfaces, for all six fuses, once as this
  section leaves `assemble.rs` and once with the arc's name carrying the
  curved surface twice in place of the plane, which is the base's key. The
  print for the union with the bar first, condensed to the edges on the wall:

  ```
  base, union bar first:  edge 0 claimed 3  (5,0,0) → (−5,2.5e-15,0)  z = 0 and wall   faces: cap, wall, wall
  now,  union bar first:  edge 0 claimed 2  (5,0,0) → (−5,2.5e-15,0)  z = 0 and wall   faces: cap, wall
                          edge 3 claimed 1  (5,0,0) → (−5,2.5e-15,0)  wall and turned  faces: wall
  ```

  On the base, the wall's arc of the ellipse ran between the same two corners
  as the cap circle's arc and got its name, so that edge was claimed by the
  cap once and by the wall twice. With the plane in the name they are two
  edges, one sound. What is left, in all six fuses alike, is four edges with
  one face each: the diameter on `z = 0` and the half ellipse on the wall,
  each once from the rim start `(5, 0, 0)` for the cap and the wall, and once
  from the meet `(5, 2.5e-15, 0)` for the turned face, all to
  `(−5, 2.5e-15, 0)`. That is the corner the faces disagree on, which is what
  the test's name already said; its doc now carries this.

### 330.6 What this leaves

- **The tolerant turned block's corner**, what is left of Q210 (330.7): why
  the turned face keeps the meet as a corner of its own where the cap and the
  wall fold it into the rim start. Its check is
  `under_a_tolerant_policy_the_turned_block_is_ordered_and_its_faces_disagree_on_a_corner`
  in `crates/rcad-topo/tests/a_plane_leaning_through_a_bars_rim_start.rs`,
  whose messages carry the volumes.
- **The ball less the block, in the mesher.** `which_pole_this_cap_reaches` in
  `crates/rcad-topo/src/mesh.rs` needs a partner that puts the two poles on
  different sides, read off the coedge it takes its step along, and not the
  first coedge's. When it has one, the pin in
  `crates/rcad-topo/tests/a_ball_less_a_wedge.rs` becomes a measurement against
  `158π`.

  **2026-10-15 note (Q508, `dfa75f1e`): closed, and not by that partner.** The
  box of side two wholly inside a ball of radius three, and this bullet's own
  shape, each have the ball across every edge of the cap that needs a pole, so
  no partner separating the poles exists for either and the fix this bullet
  asked for does not reach them. `which_pole_this_cap_reaches` instead reads
  the pole off the cap's own rim: a wire keeps its face's material on the left
  of the stored surface's own normal, a ball's and a cone's two coordinates
  are right-handed about that normal, so a rim walked the way the longitude
  grows has the material above it, and the walk's own count of the turn it
  gained (`runs_up`) says which pole that is — the same count
  `cut_a_cap_up_to_its_pole` already steps its quarters by, so the two can no
  longer disagree (§41.8 item 5's two doc comments now say one thing). The pin
  here becomes the measurement against `158π` this bullet asked for
  (`a_ball_less_or_united_with_a_block_over_one_corner_of_it_is_measured_against_the_piece`),
  the box wholly inside the ball and the crossing block both answer too
  (`a_cap_whose_pole_no_partner_separates.rs`), and the old reading also took
  the north pole for a cap round the south one — not only the wrong partner
  when one existed, but the wrong pole outright. Two pins outside this fix's
  own write set (`mesh.rs` alone) turned from a refusal to an answer with it:
  the ball resting on a block it overhangs
  (`a_ball_resting_on_a_block_it_overhangs_unites_into_both_solids_as_they_stand`
  in `fuses_that_touch.rs`, measuring `40.648` against `8 + 32π/3`) and the
  ball less, and united either way with, two blocks meeting its equator from
  opposite sides
  (`a_ball_less_or_united_with_two_blocks_meeting_its_equator_from_opposite_sides_keeps_all_of_it_but_the_two_pieces`
  in `two_flat_faces_on_one_plane_facing_apart_along_an_arc.rs`, named below,
  measuring `468.744` against `448π/3` and `3268.744` against `2800 + 448π/3`).
  A third pin the same source change reached, in a different file again,
  moved with its own row rather than Q508's own: Q719 (`afc7bdc5`) renamed
  `a_ball_whose_south_pole_is_moved_fuses_to_the_sound_balls_answer`'s
  `ROUND_THE_SOUTH_POLE` difference, in
  `a_broken_two_surface_corner_read_back_from_bytes.rs`, from
  `ACurvedFaceGoesRightRoundWithNoSeam` to `TheSoundAnswer` — a block round a
  ball's south pole cut out of the ball answers the sound ball's answer to
  the last bit, under both policies and every one of its five pole moves,
  with nothing left in the file to build the old `Outcome` variant, which
  went with its two match arms.
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

**2026-09-29, Q553: closed by Q365** (`lane-q365-a-plane-spelt-both-ways-along-one-arc`,
merged at `04296595`; its own commit is `65a5db7f`, and `5cb3132e` is only a
later formatting pass over the test file it added, not the fix). Two shapes,
both in the new `crates/rcad-topo/tests/two_flat_faces_on_one_plane_facing_apart_along_an_arc.rs`:

- **A ball of radius 5 between two blocks that meet its equator from
  opposite sides** — one under it where `x ≤ −3`, one over it where
  `x ≥ 3`, both crossing `z = 0`. On the base, before Q365, every one of the
  twelve fuses the file builds from this shape refused
  `ResultIsNotManifold { count: 1 }` (Q365's own commit message,
  `65a5db7f`). Q365's fix gets past that: the intersection and the blocks
  less the ball now answer and measure right
  (`a_ball_shares_with_two_blocks_meeting_its_equator_from_opposite_sides_the_two_pieces_they_cut_off`,
  `two_blocks_meeting_a_balls_equator_from_opposite_sides_less_the_ball_lose_a_piece_each`),
  and the ball less the blocks, and the union, now stop one step later, in
  the mesher, on the second bullet above, closed since by Q508
  (`a_ball_less_or_united_with_two_blocks_meeting_its_equator_from_opposite_sides_keeps_all_of_it_but_the_two_pieces`,
  six fuses — `just t --test two_flat_faces_on_one_plane_facing_apart_along_an_arc`
  passes all five tests in the file on this section's base). §39.4 records
  both refusals under their own rows.
- **A lower half ball, split flat side up, under two blocks standing on it
  side by side at `x = 3`.** On the base this paired the four faces along
  the equator's two arcs crosswise and the union was refused
  (`CoedgesRunTheSameWay` or `FacesDoNotAgreeWhichWayTheyFace`, by the order
  of the arguments — Q365's own commit message again). It now answers
  `4000 + 250π/3` under all four orders
  (`a_lower_half_ball_united_with_two_blocks_standing_on_it_side_by_side_is_the_two_together`).

### 330.7 Which of Q210's two questions this answers

Q210 asks first whether this change answers the exact `z = y` plane, and then
why the tolerant turned block refuses. **It answers the first**: all six exact
plane fuses answer, validate and measure under both policies (330.5). **It
narrows the second without answering it**: under the tolerant policy the
turned block no longer has an edge claimed three times, and every one of its
six fuses is refused on the corner that the cap and the wall fold into the
rim start and the turned face keeps as the meet, `2.5e-15` away. Why the
turned face keeps it is the part of Q210 still open, and the row can be cut
down to that.
