## 400. Two solids are cut against each other only where they may meet — 2026-10-04

*Queue row Q456, from Q317's trace. Lane
`lane-q456-a-ball-clear-of-a-block-stays-whole`, off `ef6c6fcd`.
`solids_within_reach_of_each_other` in `crates/rcad-topo/src/fuse/cuts.rs`
joined two solids for cutting whenever their whole-solid boxes touched, and
every plane of the one then cut every face of the other (§222.3 item 1). A
block's plane that crosses a ball clear of the block cut the ball in two, and
nothing folds two pieces of a ball back together. **Now two solids are joined
unless it is proved that no face of the one shares a point with any face of
the other.** A pair the proof cannot settle is joined as before, so an answer or
a refusal moves only where the solids are apart.*

### 400.1 What it did on the base, measured

With the tests of 400.4 run on `ef6c6fcd`:

- The row's shape A — a block from `(-2, -2, 0)` to `(0, 0, 2)` and a ball of
  radius 1 at `(1, 0.5, 1)`, united — came back with the ball in **two** faces,
  a rim along `y = 0` between them. One ulp further out, `x =
  1.0000000000000002`, the boxes stop touching and the ball came back whole.
- The linked shape B — a block from `(-4, -2, 0)` to `(-2, 0, 2)`, a ball at
  `(0, 0.5, 10)` and a ball of radius 4 at `(2, 4, 6)` whose box reaches both —
  came back with the small ball in two faces.
- All twelve of Q316's clear-apart fuses — a block of side 4 from `(-2, -4,
  -1)` and a unit ball at `(2.75, 0.75, 1)`, `(2.9, 0.6, 1)`, `(2.75, -2,
  3.75)` or `(2.75, 0.75, 3.75)`, under union both ways round and as the ball
  less the block — refused: three `Invalid(FacesDoNotAgreeWhichWayTheyFace)`,
  three `CurvedFaceWithAHoleGoesTooFarRound`, six
  `ACurvedFaceGoesRightRoundWithNoSeam`.
- Shape A's answer fused again with a cube far from both refused
  `AFaceMeetsItsNeighbourOnItsOwnSurface`.

On the branch every one of those answers: the block and the ball apart, `64`
or `8` exactly for the block, and the ball in one face measuring `4.1692` to
`4.1694` give or take the `0.126` it publishes, against `4π/3 = 4.18879`.

### 400.2 The proof

Two faces share no point when any of these holds, each a proof:

1. their two boxes miss;
2. one surface misses the part of space both boxes hold — a plane whose
   equation keeps one sign over that box, a ball's skin that is wholly outside
   it or holds it wholly inside;
3. the two surfaces do not meet at all (`intersect` answers `Nothing`);
4. they touch at one point (`TouchAtOnePoint`) and that point is outside the
   part of space both boxes hold;
5. one surface misses the band round an axis the other face lies in — how far
   up that axis and how far out from it — a ball's skin wherever its centre
   is;
6. one surface is a ball that holds every corner's own box of the other face,
   a flat face whose every side is straight, strictly inside it.

The boxes for these are built here and rounded outward: a ball's face is inside
the box round the ball, and a flat face whose sides are straight, or on a ball,
is inside the box round its corners' own boxes and those balls' boxes, since a
flat face is inside the hull of its boundary. A face on a pipe, a cone or a
torus, or a flat face with a side on one, has no box here, and then only the
other face's box is asked about. All of it runs in interval arithmetic and
`intersect` runs under `Policy::certified()` whatever the fuse runs under, its
records dropped: a tolerant policy settling a close call would skip a cut on a
guess. A pair this cannot settle is joined, and the cutting that follows asks
the same surfaces again under the fuse's own policy and log.

**Since Q896, item 5.** A face on a pipe or a cone whose every side is a circle
where a plane square to its axis crosses it, or a flat face square to an axis
whose every side is straight or such a circle, lies in a band: how far up the
axis, `(p − origin)·w`, and how far out from it, `|(p − origin) × w|²`, both
worked in interval arithmetic on the stored numbers
(`ABandRoundAnAxis`/`band_a_face_is_inside` in `cuts.rs`). A pipe or cone on
that axis's line, a ball centred on it, or a plane square to it, is then one
expression in up and out, and a proved sign of that expression over the band is
proof 5. This is what joins a frustum inside a bar, a pointed cone inside a
bar, and a bar inside a ball apart without a box test on the round faces, none
of which proof 2's boxes reach.

**Since Q2252, item 5 asks a ball off the axis too.** Write `c` for the ball's
centre, and `a = (p − origin) × w`, `b = (c − origin) × w` for a point `p` of
the band. Lagrange's identity makes `(|p − c|² − radius²)·|w|²` equal to
`near − 2a·b`, with `near = (up − c's up)² + out + c's out − radius²·|w|²`, and
`2a·b` is never farther from nought than `2√(out·c's out)`. So the skin misses
the band where `near` keeps one sign over it and `near² > 4·out·c's out` —
below nought the band is inside the ball, above it outside — both in interval
arithmetic on the stored numbers (`a_ball_misses_a_band` in `cuts.rs`). On the
axis `b` is exactly nought and this is the old test. Off it, the band's point
farthest from `c` is on the circle at its top or bottom and its greatest
`out`, which for a pipe face is one of its sides and for a flat face the circle
through its farthest corner, so the ball holds the face when it holds the
circles its sides run on, and the test asks no more than that. A ball of radius
1 less a bar of radius 0.2 and height 0.6 standing at `(0.4, 0)` inside it had
no proof — nothing bounds a pipe face, or a flat face with a side on a pipe,
in a box, item 6 asks every side straight, and the band asked a ball centred on
the bar's axis — and the answer had the ball's skin in 5 faces, 8 in all, where
the bar on the ball's axis left it in one; united with the ball it left the
ball in 5. `a_ball_hollowed_by_an_off_centre_bar_keeps_its_skin_whole.rs` pins
4 faces with the volume for the bar on the axis, at `(0.4, 0)`, and at
`(0.75, 0)`, whose caps' rims are `√0.9925` out, and the ball alone after the
union either way round.

**Since Q2247, item 6.** A flat face whose every side is straight lies in the
hull of its corners, and a ball holds the hull of any points it holds, so the
ball holding each corner's own box strictly inside it — the squared distance
from its centre to the box's furthest point less its squared radius proved
negative, in interval arithmetic on the stored numbers — proves its skin misses
the face (`a_ball_holds_a_flat_face` in `cuts.rs`). Proof 2 asks the one box
round all the corners, and that box can leave the ball at a corner of its own
that no corner of the face is near: the top of a 12-sided prism of radius `r`
at height `h` has its box reach `(r, r, h)`. A ball of radius 1 less a cube of
side 0.9 turned an eighth turn about `z` was joined to the cube for that, every
plane of the cube cut the ball's face in a circle, and the answer had the
ball's skin in 26 faces where the cube square to the axes left it in one; a
12-sided prism of radius 0.81 and height 1 left it in 98, under union as well
as difference. Q2119 measured the edge: a 12-sided prism of height 0.6 went
from 15 faces to 136 between `r = 0.674` and `0.675`, where `2r² + 0.09 = 1`,
and started a twenty-fourth of a turn round between `0.698` and `0.699`.
`a_ball_hollowed_by_a_turned_cube_keeps_its_skin_whole.rs` pins 15 faces on
both sides of both edges and at `r = 0.95`, whose corners are `0.996` out, the
cube at 7, and the volumes within what each measure publishes. A side on
anything but a plane may bulge past its corners, so a flat face with such a
side is left to the other proofs.

**No face pair sharing a point is enough to leave two solids uncut against each
other**, whether they stand apart or one is wholly inside the other: no edge of
the answer then has a face of the one on one side and a face of the other on
the other, which is all §17.4's dicing argument asks of their planes, and
selection places each uncut face against the other solid as a whole. The join
stays a chain (§222.3 item 1): two solids that each meet a third are cut
against each other, as before.

**Why not the prototype.** Q317's prototype asked whether one face's surface
splits the other face, by counting pieces. A surface that touches a face only
along its boundary splits nothing, and that is two solids meeting at an edge or
a corner, called apart. It answered `a_narrow_lune_from_a_pole_corner.rs` and
`a_mesh_that_folds_at_one_deflection_and_not_the_next.rs` with no solid at
all. Both pass here; a touch is never a proof of anything above.

### 400.3 The one test that is not a proof

Where one of the two faces has no box of 400.2's — a face on a pipe, a cone or
a torus, or a flat face with a side on one — `bake`'s own face boxes are asked
whether they miss, before the four. Those are read off stored corners in plain
`f64` (§35.4), grown by what writing a corner down costs (Q694, §340.10), but
still plain `f64`, so a miss between them is still not a proof. The whole-solid
boxes are those same boxes taken together, and a miss between those has always
been taken as two solids apart. Face by face it is a little weaker than that:
two solids whose whole boxes overlap but which touch only where both faces'
boxes miss, grown or not, were joined, and are now left uncut. Asked only
where no proof is to be had, it is weaker only for those faces; bounding those
faces soundly would retire it.

It is there because without it an associativity pin moved.
`fuse_laws_over_a_cone.rs`'s
`a_frustum_a_block_and_a_cube_inside_the_block_unite_one_way_and_not_the_other`
compares `block ∪ (frustum ∪ cube)` with `(block ∪ frustum) ∪ cube`, the cube
wholly inside the block and clear of the frustum. In the first grouping the cube
is proved apart from the block, all of it flat. In the second it is asked
against `block ∪ frustum`, whose frustum wall is a cone with no box of its own
in 400.2, so it could not be proved apart, was cut by the cube's plane `x =
3.5`, and the two answers' corners differed: `[[3.0, 3.5, 0.0], [4.0, 5.0,
-1.0]]` against `[[3.5, 3.586, 0.0], [3.5, 4.134, -1.0], …]`. With the face
boxes asked for those faces it passes, and no other test moved either way.

### 400.4 What moved, and the tests

`crates/rcad-topo/tests/a_ball_clear_of_a_block_comes_back_whole.rs` is new:
shape A both ways round, one ulp further out, the linked shape B, Q316's twelve,
and shape A's answer fused again with a far cube. Four of its five fail on the
base, with the refusals of 400.1; the one ulp out passes on both, and is there
because the two must agree.

`mise exec -- just t -p rcad-topo` moved three pins and nothing else, each a
draw of a ball **clear of** its block that now comes back whole. None moved to
a worse answer:

- `a_rim_that_names_its_cutting_surface.rs` now draws the ball half a unit off
  `y = 0` on the block's side, where it touches the block's face `x = 0` at one
  point.
- `a_ball_cut_through_its_equator_meshes_each_half_on_its_own_side.rs` drew the
  same clear ball turned about `z`. Turning a ball that *touches* a block does
  not do: over seventeen turns from `0.1` to `6`, rounding put the ball clear
  of the block, and whole, or into it, and the fuse refused `Undecidable` on
  *a point in a piece that every solid could place*. Its upright draws are now
  a ball resting on a notched block, cut by the plane of one wall of the notch,
  on both sides of it; the longitude draw keeps only the touching one.
- `every_model_the_fuse_fuzzer_kept_is_fused_again.rs`'s seed of a ball by a
  block clear of it still answers two solids, and no longer carries a rim; its
  doc says so.

And one that did not move but no longer reaches what it was kept for:
`a_meet_folds_into_a_corner_only_as_far_as_rounding_reaches.rs`'s two blocks
are proved apart, so the §345 fold is never asked about on that draw. The
answer is the same two blocks; the fold stays pinned by
`a_meet_is_folded_when_its_tighter_box_lies_inside_the_corners_room_however_far_cramers_reaches`
in `arrange.rs`, which asks `a_corner_is_this_place` directly. A draw where two
solids meet and reach that fold is left as a row.

### 400.5 Where the rim between two faces of one ball is still reached

The row asked whether a draw where the solids meet still reaches
`EdgeRun::RightRoundWhereItCrosses` (§217.1). **One does: a ball that touches
its block at one point.** A touch is a meet, so the block's planes still cut
the ball, and a plane crossing it outside the block leaves a rim between two
pieces of the one ball. That is what the two files above pin now, the rim
naming the block's plane in both.

Draws where the solids *cross* write the rim and are then refused by the
mesher, each at a site an open row already holds, measured with a throwaway
probe on this branch:

- A block's plane cutting the ball outside a box block that crosses it
  (ball at the origin, block `[0.9, 10.9] × [−5, 5] × [−9.5, 0.5]`) leaves the
  rest of the ball a ring round its pole with the block's cut as an island:
  `AWireWillNotLieFlatOnItsFace`, Q453's site. With an axis-aligned box this
  is forced: the piece on the block's side holds the block's cut and the rim
  both, and holds a pole whenever the rim's plane leaves the centre on the
  block's side. *Since Q920 (Q453's cut) this draw answers: a cap on a ball
  is cut up to its pole with the island in it (§212.6), and
  `tests/a_block_poking_into_a_ball_whose_top_plane_cuts_the_rest.rs` measures
  every rule both ways round against the closed forms.*
- Through a chain of three, so that the rim's piece is not the one the block
  crosses: a horizontal rim leaves a cap holding a pole between two pieces of
  one ball, `ACurvedFaceGoesRightRoundWithNoSeam` (Q452's case (B), dropped
  under §330.2); an upright rim leaves a band of two loops,
  `CurvedFaceWithAHoleGoesTooFarRound`, Q451's site.
- One chain **panics** in `assemble.rs`: a unit ball at the origin, the block
  `[0.9, 5] × [−5, 5] × [−5, 5]` and the block `[0, 3] × [2, 3] × [1.5, 2]`,
  as `[ball] ∪ [both blocks]` — *a run of edges that became one edge has two
  ends left*, at line 1735. Its rim plane `x = 0` runs through both poles. The
  ball meets the first block and the first block overlaps the second, so the
  base's box test put the three in one group as this does; it was not run on
  the base. It is left as a row.

**Note, 2026-10-04, Q871: the panic is closed, and it was not the rim's.** On
`656b9f09` the union panicked both ways round, at line 1860 by then; the
intersection and both differences answered. The plane `x = 0` cuts the ball
into two arcs from pole to pole, cuts that are pieces of no input edge, and
each pole has those two arcs along it and nothing else. The union joins the
pieces the link's planes cut the slab's faces into, so the joining runs, and
it folds every corner with two different edges along it: both poles went, and
the two arcs became one run with no ends. §204.1 keeps one corner of a closed
run only when it is cut from one input edge, and left any other with none.
Where nothing in a fuse is joined, the joining is skipped and such a run stays
in its pieces, which is why the other three rules answered.
`the_corners_each_closed_run_keeps` in `fuse/assemble.rs` now keeps every
corner of such a run, the answer a fuse with nothing to join already gives.
The union is one solid of `4π/3 + 410 + 1.5 − 1.05` less the cap past
`x = 0.9`, inside its measured bound in both orders, and
`tests/a_ball_cut_through_both_poles_by_a_linked_blocks_plane.rs` pins it with
the other four. The union keeps the ball in two faces either side of `x = 0`.

**Note, 2026-10-05, Q575 and Q903: fused again with a far cube, this now
answers.** Since Q575 (`9ff21bfa`) bakes the faces of a curved input as one
face before joining, `the_union_fused_again_with_a_far_cube_is_the_union_and_the_cube`
in `a_ball_cut_through_both_poles_by_a_linked_blocks_plane.rs` pins the union
and the cube, either way round, no longer refused
`AFaceMeetsItsNeighbourOnItsOwnSurface` at `crossing_for` (Q584's site). Q903
found where that refusal still reaches a ball split on itself: the union of a
ball merely *touching* a block also keeps the ball in two faces, and fused
again with any cube it is refused `Undecidable` 'a point in a piece that
every solid could place' instead, because a ball held in two faces cannot
place a point at its own centre.
`a_ball_touching_a_block_united_and_fused_again_with_a_far_cube_is_refused_where_it_places_a_point`
in `a_ball_split_on_itself_united_with_a_block_clear_of_it.rs` pins it, under
every rule both ways round, held by the row after Q977 (Q1024).

**Note, 2026-10-17, Q1384: the ball is kept as one face, and the two pole
arcs are gone.** The last note above still said "the union keeps the ball in
two faces either side of `x = 0`". Since Q1380 (`4039e989`, §229.2) the
joiner's ball arm joins two pieces of one ball used the same way round into
one face, and this union is one of the three cases that move: the plane
`x = 0` still cuts the ball into two arcs from pole to pole that are pieces
of no input edge, but the union now keeps both pieces either side of that
plane as the one face the joiner makes of them, so neither pole has an arc
left along it to fold. `a_ball_cut_through_both_poles_by_a_linked_blocks_plane.rs`
pins it: `the_union_fused_again_with_a_far_cube_is_the_union_and_the_cube`
pins the ball as `1` face (`balls_faces`), where the note above still read
two.

Measured: breaking the else arm of `the_corners_each_closed_run_keeps` in
`fuse/assemble.rs` — the path Q871 added, which keeps every corner of a
closed run cut from no input edge, reached when nothing in a fuse joins it —
and running `just t -p rcad-topo --no-fail-fast` (in a lane's own tree, never
committed) passes all three tests of
`a_ball_cut_through_both_poles_by_a_linked_blocks_plane.rs` even so: none of
them reach that path any more for this union. The path itself is not dead —
43 other tests elsewhere in `rcad-topo` still panic on it, among them several
of `two_equal_bars_leaning_at_each_other` and `fuses_that_must_agree` — so
Q871's answer still holds wherever a fuse joins nothing; this union is just no
longer one of the cases that needs it.

### 400.6 Q452

Q452 was dropped under §330.2 before this landed, and
`a_ball_by_a_blocks_top_edge_clear_of_it.rs` was never written, so nothing of
it moves here.

### 400.7 What this leaves

- The `assemble.rs` panic of 400.5, closed by Q871 (the note there).
- `cuts.rs`'s module header still said two solids are skipped only when no
  chain of overlapping boxes joins them, and `bake.rs`'s `Bounds` says a box
  test changes no answer. **Since Q896, the `cuts.rs` half is fixed**: the
  header now says two solids are skipped only when no chain of solids that may
  meet joins them, which is what the code does. **Since Q872, the `bake.rs`
  half is fixed too**: `BakedSolid::bounds`'s doc now carries 400.3's caveat,
  that a box round a pipe, cone or torus face (or a flat face with a side on
  one) is read off stored corners in plain `f64`, and a miss between two such
  boxes can still leave a pair of faces that really touch uncut.
- A face on a pipe or a cone has no box in 400.2, so a pipe or a cone clear of
  a block whose boxes miss face by face only in `bake`'s boxes is left uncut by
  400.3's standard and not by a proof, and one whose `bake` boxes touch is
  joined and cut as before. Bounding those faces soundly would extend the proof
  to them and retire 400.3. **Since Q896, answered for a pipe or a cone face
  whose every side is a circle square to its axis**, and for a flat face square
  to an axis whose sides are straight or such a circle: item 5's band bounds
  them, and `two_faces_provably_apart` proves them apart by it without
  `bake`'s boxes. A pipe or a cone face with a side running some other way —
  slanted, or with no side at all — is still left to 400.3, and 400.3 still
  stands for it.
- **Since Q2252, a band whose square plane interval arithmetic cannot show
  square.** The same bar at `(0.4, 0)` turned half a radian about `y` has no
  band at all, and the answer is 8 faces again: the far cap's normal is the
  pipe's axis exactly and the near cap's its exact negative, but
  `is_square_to` takes the cross product in intervals, and `n_z·w_x −
  n_x·w_z` comes out a range one rounding either side of nought. Each part
  worked as an exact difference of two products
  (`rcad_predicates::expansion::difference_of_two_products`) gives the bar its
  band and the four faces, and moves the pin of
  `a_turned_bar_through_a_block_cuts_ellipses_and_nothing_has_to_be_approximated`
  in `fuse/arrange.rs`, two block faces cut by 6 surfaces and not 7 with the
  same pieces, which is outside Q2252's files. A row from Q2252 carries it.
  **Since Q2256, closed.** `ABandRoundAnAxis::is_parallel_to` in `cuts.rs`
  works each part so, and `is_square_to` and the direction half of
  `is_on_the_axis` ask it. A part whose product `difference_of_two_products`
  refuses, one that overflows or one down among the smallest `f64`s, is still
  asked in intervals, so nothing parallel before is less so now. The leaning
  bar is four faces by difference and the ball alone by union, both pinned in
  `a_ball_hollowed_by_an_off_centre_bar_keeps_its_skin_whole.rs`, and the
  block's faces at `y = 0` and `y = 8` in the `arrange.rs` pin no longer
  carry the pipe among their cuts.
- A draw where two solids meet that reaches the §345 fold.
- Q451 and Q453 wait on this row and were written around Q316's four draws,
  which no longer reach the mesher.
