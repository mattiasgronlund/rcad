## 379. What keeps a far-off ball refused once `cut_up` reads its points as made: measured, not landed — 2026-10-03

*Queue row Q378, from §333.7. Lane
`lane-q378-a-placed-bar-meshed-like-the-unplaced-bar`, off `3d403339`. **Not
fixed.** This section records the table on that base, one question that keeps
the ball at `x = EVEN` refused, and the two reasons the change did not land:
the refusal it needs has no `Error` variant in this lane's files, and the
first lane's hunks, adapted to today's `mesh.rs`, no longer make the bar's
triangles match. `mesh.rs` is unchanged on this branch.*

  379.1   The table on the base: only the wall differs
  379.2   The question: does the mesh still enclose space where it lands
  379.3   What it refuses, measured
  379.4   Why it did not land
  379.5   What this leaves

### 379.1 The table on the base: only the wall differs

§333.2's twelve placements of the bar of radius 3 and height 5, meshed at
`0.005`, compared with the unplaced bar's triangles as sorted sets of point
indices, and split by whether the triangle's face is flat. The test that prints
it is `a_placed_bar_is_cut_into_the_unplaced_bars_triangles`, kept at
`/tmp/claude-1000/-home-mattias-git-mattiasgronlund-rcad-worktrees-ci-verdict-cache/361883b9-99a3-4c0e-97d8-a972c39b393b/scratchpad/a_placed_bar_is_meshed_from_the_corners_it_stores.rs`.
Copied over the tracked file and run as
`mise exec -- just t -p rcad-topo --test a_placed_bar_is_meshed_from_the_corners_it_stores --no-capture`:

| placement | triangles off of 500 | on the caps | on the wall |
|---|---|---|---|
| an eighth about `z` | 156 | 0 | 156 |
| the same, then `(3, −5, 7)` | 175 | 0 | 175 |
| the same, then `(0.25, −0.5, 0.125)` | 157 | 0 | 157 |
| the same, then `(0.1, 0.2, 0.3)` | 155 | 0 | 155 |
| 0.7 about `(1, 2, 3)` | 253 | 0 | 253 |
| the same, then `(3, 5, 7)` | 291 | 0 | 291 |
| 2.5 about `(1, 2, 3)` | 299 | 0 | 299 |
| the same, then `(3, 5, 7)` | 309 | 0 | 309 |
| 0.7 about `(−3, 1, 0.5)` | 273 | 0 | 273 |
| the same, then `(3, 5, 7)` | 269 | 0 | 269 |
| 2.5 about `(−3, 1, 0.5)` | 283 | 0 | 283 |
| the same, then `(3, 5, 7)` | 291 | 0 | 291 |

**No cap triangle differs in any row.** §362 cut the caps where the solid sits,
and in these twelve rows that happens to give the unplaced caps' triangles; it
does not promise to (§362.2, "a tie can go the other way"). Everything left is
the wall, which is still cut in the stored frame from points moved out and back.

*Wrong, found by Q781 (379.5): the test read the caps' keys off a second model,
so no triangle could count as a cap's. Each row's total is right; split
properly, the caps hold 141 to 199 of it and the wall 12 to 120.*

### 379.2 The question: does the mesh still enclose space where it lands

§333.6 measured what goes wrong at `x = EVEN` with the first lane's hunks: the
mesh made in the stored frame is a sound octahedron, and every point of it
lands on the plane `x = EVEN`, so the mesh handed out encloses nothing and each
triangle lies on a twin. Of the things a mesh of a solid promises, that breaks
exactly one: **it encloses space.** It does not break distance (every point is
within the `43.87` the mesh claims), it does not break the topology (every side
is still used twice), and it does not break the corners being apart (the
seam's middle and its antipode land `2.2e−16` apart, kept apart only by
rounding left in `y` and `z`).

Three per-triangle questions were thought through and turned down, without a
run:

- **Does each triangle face the way the placement turns its made twin?** At
  `EVEN` it does: flattening onto `x = c` keeps the sign of each normal's `x`.
- **Does the crease at each side keep its sign** (`orient3d` of the two
  triangles' four corners)? A wall quad's four corners are coplanar where they
  are made, so a placement rounds that crease to either sign on a sound bar.
- **Do the loop's corners stay apart on the located surface?** It refuses
  `EVEN`, but only because the seam's middle lands at the ball's centre, where
  the chart is undefined and reads a pole only because the rounding left over is
  in `z`. That is the frame mix-up's accident again.

So the question is the whole mesh's: **where it was made it enclosed space, and
where it lands it encloses none.** Both signs are `measure::signed_volume_over`,
which settles its sign exactly when the rounded sum cannot. It is asked only of
a placed solid, since a face at no placement is moved by a shift that loses
nothing. Asking only "is the volume positive" is not enough on its own: a mesh
inside out where it was made is `Invalid::VolumeIsNotPositive` for `validate`
to say, and `validate` already re-meshes finer when a coarse mesh encloses
exactly nothing (`how_much_space_a_curved_solid_encloses`).

### 379.3 What it refuses, measured

The probe is `q378-volume-probe.diff` beside the test above: the first lane's
hunks redone against today's `mesh.rs` (each table point also remembers where
it was made, and `walk_a_wire_round` reads that for a placed curved face), the
question in `SolidView::mesh`, and a sweep test in the ball file. The question
refuses with a stand-in error, `Invalid::DynamicRangeTooWide { ratio: NaN }`,
only so that it can be told apart. Applied with `git apply` and run as
`mise exec -- just t -p rcad-topo --test how_far_out_a_ball_can_sit_and_still_mesh --test a_placed_bar_is_meshed_from_the_corners_it_stores --no-fail-fast --no-capture`:

- At `x = EVEN` along `x` the volume where made is `1.3333333333333333`, and
  where it lands `0`. Refused. Up `z` at `EVEN`, refused too. `ODD` meshes on
  both axes.
- A unit ball along `x`, four placements from the bottom of each binade:
  every one meshes from `2^50` to `2^52`; from `2^54` to `2^55`, every one is
  refused; in `[2^53, 2^54)` the odd ones mesh and the even ones are refused,
  **except `2^53` itself, which meshes.**
- No bar row is refused.

**`2^53` is where it parts from the base, and the new answer looks right.** At
`2^53` the ulp below is 1 and above is 2, so `c − 1` is a double and the near
side of the ball lands where it was made, while the far side rounds onto `c`.
The mesh that comes back is half flattened and still encloses space. That is the
same kind of answer the file accepts at `ODD`: inside the deflection asked for
and closed. The base refuses it with `CornersCoincide`, for the reading in the
stored chart. Radius 0.25 at `2^51` parts the same way. So
`the_answer_alternates_at_every_ulp_across_the_band` and
`each_radius_has_its_own_band_and_they_all_alternate` would have to start one
ulp above the bottom of each band. The probe did not check the half-flattened
mesh's volume or distance, so that last claim is measured only as "it meshed".

### 379.4 Why it did not land

1. **There is no honest refusal for it among the variants this lane could
   use.** `crates/rcad-topo/src/error.rs` was not in the write set.
   `Invalid::VolumeIsNotPositive` calls a sound ball malformed, which is what
   §333.6 objects to. `CannotMeshAFace` with `CornersCoincide`, and
   `AWireWillNotLieFlatOnItsFace`, name things that did not happen.
   `DynamicRangeTooWide` has a true message (the ball is far outside §3.5's
   `1e11`), but its `ratio` would need `validate`'s smallest-feature sweep,
   which `mesh.rs` cannot reach. The claim that a lost volume always means a
   ratio above `1e11` would also need its own demonstration.
2. **The adapted hunks no longer make the bar's triangles match.** With the
   probe applied, the bar's points all still match bit for bit, but between 141
   and 199 of 500 triangles still differ, all on the wall. The probe prints them
   per row. Under §333.6's older `mesh.rs` the same idea gave 0 in every row. So
   on this base something other than the walked points and corners still
   differs between a placed wall and an unplaced one on the way to the
   triangulator. This lane did not trace it. *(Q781, 379.5: nothing did. The
   141 to 199 are all on the caps, and the wall matched.)*

The pinned `5.994025313141081e-6` in
`whether_the_floor_predicts_the_step_cap.rs` fails under the probe, as it did
under §333.6's hunks. This lane did not read the new value. The pin is
unchanged here because nothing landed.

### 379.5 What this leaves

- Trace what still differs on the wall with 379.3's probe applied (141 to 199
  triangles), and make all twelve rows match. `mesh.rs`.
- Add an `Error` variant for "placing this solid took away the space its mesh
  encloses", ask 379.2's question in `SolidView::mesh`, and move the ball
  file's pins: the refusal's name along `x` and `z`, and the bottom of each
  band (379.3). Needs `crates/rcad-topo/src/error.rs` in the write set.
- Then correct the floor file's pinned deviation with its cause.

**2026-10-03, Q780: the refusal and the hunks landed.** Lane
`lane-q780-a-placed-solid-that-loses-its-space-is-refused`, off `6ac09257`.
The second and third items above are done; the first stays open (Q781).
*(Q781, below: closed; the wall never differed.)*

- `Error::PlacingItTookAwayTheSpaceItsMeshEncloses { enclosed_where_made,
  enclosed_where_placed }` is the refusal, raised at the end of
  `SolidView::mesh` for a placed solid only, when 379.2's question is asked
  and answered no. It is asked only when the volume where the mesh lands is
  not positive, so a sound placed solid pays for one sum. The probe's hunks
  landed with it, with one change: a face at no placement records its points
  as they land, not in the frame it was shifted to, so that the table of
  points as made is in one frame per shell. A flat face is asserted to get no
  point inside it, which the triangulator's own rule guarantees and which
  keeps that table honest.
- The unit ball at `EVEN` is refused by that name through `mesh`, `validate`
  and `measure` alike, `4/3` where made and `0` where it lands, along `x` and
  up `z`:
  `mise exec -- just t -p rcad-topo --test how_far_out_a_ball_can_sit_and_still_mesh`.
- **Both band edges are honest meshes, and their pins moved to the answer.**
  The unit ball at `2^53` and the ball of radius `0.25` at `2^51` mesh as a
  closed square pyramid: the octahedron with its far corner pressed onto the
  centre. Volume `0.6666666666666666` and `0.010416666666666666`, half the
  octahedron's in each case (the balls' own are `4.19` and `0.0654`); the
  furthest of 8 320 sampled points of the ball sits `1` and `0.25` from the
  mesh, against claimed deviations of `36.67` and `9.17`; no triangle has no
  area and no two lie on the same three places.
  `at_the_bottom_of_the_band_the_ball_meshes_half_flattened` checks all three
  and prints the numbers. The alternation tests now start one ulp above the
  bottom of each band, and pin the bottom as meshing.
- The bar's wall still differs: 141 to 199 of 500 triangles across the twelve
  rows, none on the caps — the same as 379.4 item 2. *(Q781, below: all on
  the caps, none on the wall.)*
- The floor file's deviation moved from `5.994025313141081e-6` to
  `5.993841838603952e-6`, still 5134 points. With the one read in
  `walk_a_wire_round` put back to the round trip and everything else kept, the
  old value comes back, so that read is the cause.
- **One floor outside this lane's files fails, because a placed mesh got
  better.** `measuring_one_face.rs`'s
  `the_middle_of_each_face_is_within_its_wobble` floors the closest any
  centroid comes to its wobble at `0.002`, and its doc says `0.0022`. That
  `0.0022` was the bend turned and a million units out, at `0.03`:
  `0.0021706256371892296` with the round trip, against `0.0018130049876618794`
  for the same bend at no placement. With the read as made the placed bend
  comes to `0.0018130046725639277`, the unplaced bend's own figure to seven
  digits, so the closest is now `0.0018130049876618794` and under the floor.
  Every face is still inside its wobble. The floor and the doc's `0.0022` need
  moving to the unplaced bend's `0.0018`; that file was not this lane's.

**2026-10-03, Q780's second run: the wobble floor moved.** Lane
`lane-q780-a-placed-solid-that-loses-its-space-is-refused-2` took the first
run's branch whole and moved `the_middle_of_each_face_is_within_its_wobble`'s
floor from `0.002` to `0.0016`. That keeps it about as far under the new
closest, `0.0018`, as the old floor sat under `0.0022`. On this branch the
closest of the 96 faces is the bend's tube at `0.03` where it was built,
`0.0018130049876618794`. The same tube turned and a million units out comes to
`0.0018130046725639277`. The test now names the closest face, so a run with
`--no-capture` prints it. This run did not measure the base again. A face at
no placement is meshed from the same points as before: `read_as_made` is false
there, and `remember_made_on` keeps the point as it lands. So the base's
`0.0022` above `0.002` can only have come from a placed face. Which placed face
it was rests on the first run's figure.

**2026-10-03, Q781: the wall never differed; the caps do, by §362's choice.**
Lane `lane-q781-a-placed-wall-cut-like-the-unplaced-wall`, off `348b9390`.
`mesh.rs` changed only in a doc comment.

- **What made it look like the wall.** The table test took the caps' face
  keys from `the_flat_faces_of_a_bar`, which builds a second `Model`. A key
  names its model (`ModelId::next()` per `Model::new`), so no triangle of the
  meshed bar was ever counted as a cap's, and every differing triangle landed
  in the wall's column. Its `placed.triangles() != unplaced.triangles()` failed
  every row for the same reason, since a `MeshTriangle` carries its face key.
  With the keys read off the meshed model, on this base: 142, 155, 141, 143,
  165, 187, 197, 199, 177, 189, 163 and 183 triangles off, the twelve rows in
  379.1's order, **every one on a cap and none on the wall.**
- **What the triangulator is handed, placed against unplaced.** Dumped from
  `hand_over` for each patch: surface, room, each loop's mesh points and chart
  corners as bits, and what `triangulate` handed back. For an eighth about `z`
  the six wall patches are the same bytes, the room included
  (`3f747ae147ad847b`). For an eighth then `(3, −5, 7)` the wall's room
  differs (`3f747ae147ad067b`), and its corners and triangles are still the
  same bits. The first value that differs is a cap's chart corners, from
  `the_surface_this_face_is_cut_up_on` and `the_frame_this_face_is_cut_up_in`:
  a flat face is read on its located plane from the points where they land.
  That is §362's fix (`333bb79a`, after `96da3252`), which is why §333.6's
  older base matched in every row.
- **The two targets cannot both be met by choosing a frame.** With the caps
  cut where they are stored (those two functions and
  `back_out_of_the_frame_this_face_is_cut_up_in` sent to the meshed-in frame
  for a plane too), all twelve rows come to 0 off and
  `a_bar_moved_a_tenth_has_its_caps_cut_up_the_same_after_its_own_file`
  fails: 21 of 121 triangles of the top cap join other points than its
  read-back's. On the base, the reverse: that test passes and the bar rows
  are as above. So this lane kept §362, and the caps are not compared. A
  rule for flat faces that does not read the frame at all, such as breaking
  a tie by mesh-point index in `crates/rcad-mesh/src/triangulate.rs`, might
  meet both. It was not tried.
- **What Q780 did for the wall, measured.** With `walk_a_wire_round`'s
  `read_as_made` forced false, the wall rows read 14, 20, 16, 12, 88, 104,
  102, 110, 96, 80, 120 and 108 off, and caps plus wall give 379.1's totals in
  every row. So Q780's read took the wall to 0.
- The test is now `a_placed_bars_wall_is_cut_into_the_unplaced_walls_triangles`:
  the wall's triangles corner for corner and in order, with the caps printed.
  It passes on this base, and fails under the forced-false read above. Run as
  `mise exec -- just t -p rcad-topo --test a_placed_bar_is_meshed_from_the_corners_it_stores --no-capture`;
  the rcad-step side is
  `mise exec -- just t -p rcad-step -E 'binary(merged) & test(a_flat_cap_is_cut_up_the_same_after_its_own_file)' --no-capture`.
- No pin moved, since no code did.

**2026-10-03, Q784: a rule on corner order cuts the caps alike and breaks two
ball lids, so nothing landed.** Lane
`lane-q784-a-placed-bars-caps-cut-like-the-unplaced-caps`, off `ad8189a6`.
Only this entry changed.

- **Q781's two numbers, measured again.** With
  `the_surface_this_face_is_cut_up_on`, `the_frame_this_face_is_cut_up_in`
  and `back_out_of_the_frame_this_face_is_cut_up_in` sent to the meshed-in
  frame for a plane too, all twelve rows print 0 off, caps and wall, and
  `a_bar_moved_a_tenth_has_its_caps_cut_up_the_same_after_its_own_file` fails:
  21 of 121 triangles of the cap on `z = 5.3` join other points. Both are
  Q781's. On the base the rows print 142, 155, 141, 143, 165, 187, 197, 199,
  177, 189, 163 and 183.
- **spade's in-circle test is exact, and the caps' ties are near-ties, so
  breaking ties by index cannot work.** spade 2.15.1 decides a flip with
  `robust::incircle(...) < 0.0` (`contained_in_circumference` in
  `delaunay_core/math.rs`), which is Shewchuk's adaptive exact test, so a tie
  there is an exact zero. Each flat patch was dumped from `triangulate` and
  its sides tested again with `robust` 1.2.0 in a scratch probe. Of the 123
  sides spade drew inside the unplaced bar's cap, 2 test exactly zero and the
  other 121 come under `3.6e-15` of the fourth power of their size. Per
  placed cap, 56 to 98 sides of the unplaced cut are missing. On those, the
  unplaced cap has 1 or 2 exact zeros and the placed cap none, and the sign
  differs on 34 to 59, all under `1.5e-14` of size⁴. A rule on exact zeros
  would touch at most two sides of a cap, and none on the placed side.
- **The read-back hands over the same corners in the same order.** For the bar
  moved a tenth, the placed caps and the read-back caps reach `triangulate`
  with the same corner bits in the same order, so a rule on corner order
  keeps that test.
- **The rule tried reads only corner order and which way each corner turns.**
  It applied to a flat patch with one loop, every corner inside one edge,
  every corner turning the same way by `orient2d`, and `v` climbing then
  falling once round the loop, which makes the loop convex and once round.
  That patch was cut by corner order instead of by spade. A hole, a corner
  between two edges, or a corner turning the other way left the patch to
  spade. On the bar's caps the tightest turn is `5e-2` of its size squared,
  far from a tie. Two shapes of the rule were tried: a fan from corner 0; and
  every other corner cut off, round after round, so that the first round
  draws only sides that skip one corner, which no curved neighbour draws
  (§187.4). **Both meet both pins**: 0 off on caps and wall in all twelve
  rows, with the caps compared corner for corner and in order, and the
  read-back test passes. A disc of 40 corners turned by twelve angles and
  moved is cut the same in all twelve, where spade cut all twelve
  differently. A five-petalled curve and a disc with a round hole stay with
  spade and pass the patch file's `check_everything`.
- **Both break two ball lids.** The whole suite under the fan, `mise exec --
  just t --no-fail-fast`: 2867 of 2869 pass. The two that fail:
  - `a_rim_cornered_on_the_pole_builds_one_way_round_and_that_way_measures`
    (`a_closed_edge_neither_direction_rule_answers.rs`) is refused
    `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }` for `TheCap`,
    rim named `ItsTwoFaces`, lid listed first `false`, deflection `0.5`. The
    lid draws a side between two rim points that the ball's patch also draws.
    Every-other-corner is refused the same way.
  - `no_sliver_cut_through_a_balls_pole_folds_at_any_deflection`
    (`a_mesh_that_folds_at_one_deflection_and_not_the_next.rs`) pins 38 folds
    in 1440 meshes, every one cured at a later rung. The fan gives 96 and
    every-other-corner 68, all cured at a later rung.
  Every-other-corner was run on those two tests and the four new and pinned
  ones only, not the whole suite. Landing either rule would trade the caps'
  ties for a new refusal and more folds, so the lane kept the base.
- **What a next lane needs.** The triangulator sees one face at a time, so it
  cannot tell a bar's cap from a ball's lid. A bar's wall never draws a side
  between two rim points, because the rim is a straight line in the wall's
  chart. A ball's patch does draw one. `mesh.rs` knows the face across the
  rim. So a rule on corner order, offered only where no face across the rim
  can draw such a side, should meet both pins and leave the lids to spade.
  That has not been run. It needs `crates/rcad-mesh/src/domain.rs` (a way to
  ask for the rule) and `crates/rcad-topo/src/mesh.rs` (asking for it) in the
  write set, beside this lane's three files. Not measured: whether a tube's
  flat end, a ring between two circles, has the same near-ties.

**2026-10-03, Q796: a bar's caps are cut by corner order, asked for only where
the face across reads the rim at one height.** Lane
`lane-q796-a-placed-bars-caps-cut-by-corner-order`, off `01b6eb41`. All twelve
rows are 0 off on caps and wall, the read-back test passes, and both ball lid
tests come back as pinned.

- **Q784's numbers, measured again with its rule rebuilt** and applied to every
  flat patch with one loop, every corner inside one edge, every corner turning
  the same way and `v` climbing then falling once. Every other corner cut off,
  round after round: 0 off on caps and wall in all twelve rows, the read-back
  test passes, `a_rim_cornered_on_the_pole_builds_one_way_round_and_that_way_measures`
  is refused `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }` for
  `TheCap`, rim named `ItsTwoFaces`, lid listed first `false`, deflection
  `0.5`, and `no_sliver_cut_through_a_balls_pole_folds_at_any_deflection`
  counts 68 folds in 1440, all cured at a later rung. A fan from corner 0: the
  same, with 96 folds. Every number is Q784's. Run as
  `mise exec -- just t -E 'binary(a_placed_bar_is_meshed_from_the_corners_it_stores) | binary(a_flat_cap_is_cut_up_the_same_after_its_own_file) | test(=a_rim_cornered_on_the_pole_builds_one_way_round_and_that_way_measures) | test(=no_sliver_cut_through_a_balls_pole_folds_at_any_deflection)' --no-capture --no-fail-fast`.
- **What landed.** `TrimmedDomain::cut_by_corner_order` is the way to ask. The
  triangulator honours it for a flat patch with no holes whose outline turns
  the same way at every corner by `orient2d` and climbs then falls once, and
  cuts every other corner off, round after round; any other patch asked is cut
  by spade as though it had not been asked. Every refusal is still asked
  first. `mesh.rs` asks for a flat face bounded by one closed edge when every
  face across that edge is curved, its second coordinate does not wrap, and it
  reads every point of the edge at the same bits of that coordinate — read by
  the same `HowAFaceReadsThePointsRoundIt` its own walk uses
  (`BuildingAMesh::nothing_across_its_rim_joins_two_of_its_points`).
- **Leg (a), decidable without a tolerance: yes.** The face across hands its
  triangulator those points on one straight line, the walk passes a second
  coordinate that does not wrap through as it came, and a side between two of
  them not next to each other along the edge runs through the points between,
  which no triangulation draws. The points strictly along the edge lie on no
  other face. So equal bits are enough, and the test is bit equality.
- **Leg (b): the bar's wall answers no and the ball's patch yes.** With a
  print on every flat face bounded by one closed edge, over the four tests
  above: every bar cap was asked to be cut by corner order (26 per bar test, 6
  in the read-back file, which covers the bar read back at no placement), and
  every ball lid was not (140 in the rim test, 2016 in the fold test). The
  print was taken out before the commit.
- **Leg (c): both lid tests come back exactly as pinned**, the fold test at its
  38 of 1440.
- **The pin.** `a_placed_bar_is_cut_into_the_unplaced_bars_triangles`, which
  replaces `a_placed_bars_wall_is_cut_into_the_unplaced_walls_triangles`,
  compares caps and wall corner for corner and in order. With `mesh.rs`'s ask
  forced off it fails, printing 142, 155, 141, 143, 165, 187, 197, 199, 177,
  189, 163 and 183 off on the caps, Q781's numbers:
  `mise exec -- just t -p rcad-topo --test a_placed_bar_is_meshed_from_the_corners_it_stores --no-capture`.
  In `rcad-mesh`, a disc of 40 corners turned by twelve angles and moved is cut
  the same in all twelve when asked, where spade cut 11 of the 11 turned ones
  differently from the first; a clockwise disc comes back anticlockwise; a disc
  with a hole, five petals, a corner that does not turn and a patch of a pipe
  are cut as though not asked; and a seven-pointed star drawn by every second
  point, which turns one way and goes round twice, fails the once-round check:
  `mise exec -- just t -p rcad-mesh -E 'test(/corner_order|once_round/)' --no-capture`.
- **The whole suite**, `mise exec -- just t --no-fail-fast`: 2871 run, 2871
  pass, 9 skipped. No test moved.
- **Left.** A tube's flat end, a ring between two circles, is not asked for:
  it has two loops, so the triangulator leaves it to spade whatever is asked.
  Whether its two circles give the same near-ties, and so whether a placed
  tube's ends are cut into other triangles than an unplaced tube's, was not
  measured: no builder makes a tube without a fuse, and a probe was not cheap.

**2026-10-06, Q797: a pipe's rings do not match; a bar's whole discs still
do.** Lane `lane-q797-a-placed-pipes-flat-ends-meshed-from-stored-corners`,
off `9e6da7e1`.

- **The measurement.** A pipe — a bar of radius 3 and height 5 less a
  coaxial bar of radius 1 that pokes through both ends, built with
  `Model::fuse(Rule::Difference)` since no builder makes one directly — is
  meshed at `0.005` under the same twelve placements §333.2 uses, and each
  flat end's triangles are compared with the unplaced pipe's as sorted sets
  of point indices, the bottom ring and the top ring kept apart. Both rings
  have 796 triangles, bottom and top together, in every placement.
  `crates/rcad-topo/tests/a_placed_pipe_is_meshed_from_the_corners_it_stores.rs`,
  run as
  `mise exec -- just t -p rcad-topo --test a_placed_pipe_is_meshed_from_the_corners_it_stores --no-capture`.

  | placement | bottom ring off | top ring off |
  |---|---|---|
  | an eighth about `z` | 2 | 0 |
  | the same, then `(3, −5, 7)` | 0 | 2 |
  | the same, then `(0.25, −0.5, 0.125)` | 2 | 0 |
  | the same, then `(0.1, 0.2, 0.3)` | 2 | 0 |
  | 0.7 about `(1, 2, 3)` | 2 | 2 |
  | the same, then `(3, 5, 7)` | 2 | 0 |
  | 2.5 about `(1, 2, 3)` | 0 | 0 |
  | the same, then `(3, 5, 7)` | 0 | 0 |
  | 0.7 about `(−3, 1, 0.5)` | 2 | 0 |
  | the same, then `(3, 5, 7)` | 2 | 0 |
  | 2.5 about `(−3, 1, 0.5)` | 0 | 2 |
  | the same, then `(3, 5, 7)` | 0 | 0 |

- **A ring has a hole, so §379.5's corner-order rule leaves it to spade.**
  `mesh.rs` asks for a flat face bounded by one closed edge; a ring is
  bounded by two (the outer circle and the inner one), so it is cut by
  spade's Delaunay test whichever bar it was. That test is near-tied on a
  circle's points the same way it is on a bar's whole disc (§379.5, Q784), and
  nine of the twelve placements move 2 of a ring's 796 triangles onto other
  points, once on both rings at once. A bar's whole discs stay at `0` under
  the same twelve placements, so this is the gap Q796 left: a pipe's ends are
  not cut into the unplaced pipe's triangles the way a bar's are.
- **A row is filed for it** (not yet judged wrong, so untagged): a fix would
  need `TrimmedDomain`'s loop-count check in `crates/rcad-mesh/src/domain.rs`
  widened to a one-hole ring whose outer loop meets §379.5's corner-order
  conditions and whose hole is a circle read at the same height everywhere,
  plus `mesh.rs`'s ask for it — not measured whether that is sufficient, so
  no class.

**2026-10-07, Q1109: a pipe's rings are cut by corner order, and all twelve
placements are 0 off.** Lane `lane-q1109-a-placed-pipes-rings-cut-by-corner-order`,
off `9e02c8f0`, merged at `6710f521`.

- On the base, Q797's test printed its twelve rows as pinned.
- The ring's outline has 126 corners and its hole 73. The outline's corner 0
  is a quarter turn from the hole's corner 0, so a cut that starts both loops
  at corner 0 turns triangles over and falls back to spade (measured with a
  probe).
- **What landed.** `triangulate.rs` cuts a flat patch with one hole that turns
  against its outline by stepping round both loops together
  (`in_step_round_a_ring`). The hole starts at the middle of the run of its
  sides that the outline's corner 0 sees from outside
  (`the_corner_of_the_hole_across_from`, `orient2d` signs only). Which loop
  steps next is a whole-number comparison of equal shares of a turn. The cut
  is taken only when every triangle turns the outline's way by `orient2d`.
  With that, the triangles' sides add up to the two loops, so they cover the
  ring exactly once; any other ring is cut by spade as though not asked.
- `mesh.rs` asks on a flat face of one or two wires. Each must be one closed
  edge that every curved face across reads at one height of a second
  coordinate that does not wrap. For a ring, no face may be across both rims.
- **Results.** All twelve rows are 0 off on both rings, corner for corner and
  in order (`a_placed_pipes_flat_ends_are_cut_into_the_unplaced_pipes_triangles`).
  These pass as pinned: `a_placed_bar_is_cut_into_the_unplaced_bars_triangles`,
  the read-back file,
  `a_rim_cornered_on_the_pole_builds_either_way_round_and_both_ways_measure`,
  and `no_sliver_cut_through_a_balls_pole_folds_at_any_deflection`, which has
  been pinned at 2 folds since Q1027. rcad-mesh, rcad-topo and rcad-step: 1956
  of 1956 pass.
- **The Q796 entry's command no longer runs the rim test**: that test was
  renamed from `…_builds_one_way_round_and_that_way_measures`. The four tests
  run now as
  `mise exec -- just t -E 'binary(a_placed_bar_is_meshed_from_the_corners_it_stores) | binary(a_flat_cap_is_cut_up_the_same_after_its_own_file) | test(=a_rim_cornered_on_the_pole_builds_either_way_round_and_both_ways_measure) | test(=no_sliver_cut_through_a_balls_pole_folds_at_any_deflection)' --no-capture --no-fail-fast`.
- **Left, not run (Q1274).** A ring whose outline's corner 0 lies within a
  rounding of the line through one of its hole's sides could see another run
  of sides once placed, start the hole at another corner, and be cut into
  other triangles than the unplaced ring.
