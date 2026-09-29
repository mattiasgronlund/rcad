## 199. The half ball's last two pins answer, and a plane steeper than a cone cuts it in two branches — 2026-09-22

*One batch, dispatched by §175's skill off `127fab3b`: workflow run
`wf_b826c0d2-5c3`, three Opus lanes. The two lanes that changed `src/` were each
checked by a Sonnet verifier. The cone lane changed only a doc comment, so it had
no test that fails without it, and the workflow skipped its verifier; its diff was
read at the merge. `just gate quiet` run here: on `127fab3b` before the batch,
PASS, 2235 run, 2235 passed, 6 skipped; on the merge `4193d550`, PASS, 2242 run,
2242 passed, 6 skipped. A number below is a lane's unless it names a command run
here. Both verifiers re-ran every number they could and found none that
disagreed. The loop areas in 199.1 came from a probe that was since removed, so no
verifier could re-run them.*

### 199.1 A ball's skin with a round hole is cut from the hole up to each pole

`lane-half-ball-hole-slit` (`1572c61d`) closes §198.3's first item. The half
ball `x ≥ 0` cut by `x ≤ 0.5` now fuses, validates, meshes and measures against
`a_balls_volume()/2 − a_caps_volume(0.5)`. The pin is now the answer test
`a_half_ball_cut_square_to_its_flat_face_measures_half_a_ball_without_its_side_cap`.

A probe printed the skin in the ball's chart. The outer loop is the rectangle
`lon ∈ [−π/2, π/2]`, with the poles as its top and bottom sides: 6 corners,
area +9.87. The hole is an island round `(0, 0)` with its one corner at
`(0, π/3)`: 4 corners, area −2.19. `open_a_slit`'s single slit would pass the
same points twice, and the triangulator refuses that. So the new
`cut_a_hole_up_to_both_poles` cuts twice, each cut along one meridian. One runs
from the hole's highest corner up to the top pole side, the other from its
lowest corner down to the bottom one. That leaves two pieces of one loop each.
Anything it cannot prove stays `CurvedFaceWithAHoleGoesTooFarRound`:

- a surface that is not a ball, or anything but two loops;
- loop areas that do not have opposite signs;
- a cut whose nearest outer side is not a pole side, or that runs along a side;
- a hole with a side of no length, or top and bottom corners at one longitude;
- a piece that encloses no area.

The verifier found the last two guards in the code where the lane's summary
listed four.

`a_half_ball_skin_with_a_round_hole.rs` checks that the mesh is closed: every
side is used once each way, and points minus sides plus triangles is 2. It also
checks the volume against the closed form. The cut is tried at 5 places, 3
deflections, and 7 leaning or turned placements. No pinned count or rate moved.

**A new gap**, out of the lane's reach. The same cut, leaned 20° about `y` and
then turned −0.4 rad about `z`, is refused `Error::Undecidable` inside the fuse,
at "a point inside a piece of a face", whose inputs include both poles. The same
lean turned by 0 or 0.3, and leans of 0.3, 0.35 and 0.4, all answer and measure.
It is pinned, with its closed form, as
`a_half_ball_cut_by_a_plane_leaning_twenty_degrees_and_turned_back_is_refused_undecided`.

The cut has one limit, as written. It does not bring the hole round into the
outer loop's turn of longitude. A hole stored a whole turn off would miss the
pole sides and be refused rather than cut. That would be a wrong refusal, not a
wrong answer, and nothing built here reaches it.

### 199.2 A hole walked as two arcs along one ball is one circle

`lane-half-ball-round-hole-wire` (`c1668520`) closes §198.3's second item. The
brief's diagnosis was one step off. The face never reached
`what_a_round_wire_encloses`. Its two arcs are parts of a crossing, so they went
down §167's arc path, which refuses a ball arc on a flat face that keeps the
outside of the ball. `is_one_circle_cut_in_two` now proves the wire is one
circle before leaving it out of that path. The wire must have exactly two sides.
Both must be parts of a crossing with the same stored ball across them, in the
face's own frame. And `the_way_into_an_arc` must point opposite ways off the
chord, so that they are the circle's two arcs and not one arc twice.

That exposed a second refusal. The outline walk asked
`orient3d(aim, aim, target, point)`, which is zero for any point. It still gave
no sign when the point is a box that straddles `x = 0`. That corner is now read as
zero without asking.

Both of §197.2's union-twin pins answer. `face_holds_the_point` on the joined
solid's face `x = 0` gives `NotHeld` in the hole and `Held` on the block. On the
rim, the poles and the block's edge it gives `OnItsBoundary`, and past the block
`NotHeld`. The joined-and-cut fuse validates and gives one solid, which measures
within `HOW_FAR_UNDER` of `the_blocks_part + the_balls_part`. The verifier
switched each fix off in turn. Without the corner identity 2 of 5 tests fail;
without `is_one_circle_cut_in_two` 3 of 5 do. `fuses_that_touch.rs`'s ball
resting on a block it overhangs still refuses `AWireWillNotLieFlatOnItsFace`.

Two things are not tested. A joined solid that has been turned or moved (§168):
the lane's argument is that it reads stored surfaces in the face's frame, but it
did not run one. And the "one arc twice" case that `the_way_into_an_arc` rejects:
the lane calls it unreachable from the public API, and under §16.10 that is a
claim without a demonstration. It stays a claim until someone builds it.

With both lanes merged, all three of `a_half_ball_cut_again_away_from_its_poles.rs`'s
cuts answer. The merge dropped `refused_or_measured` and `which_refusal`, which
nothing called any more (`4193d550`).

### 199.3 A plane steeper than a cone cuts it in two branches, and the fuse follows both

`lane-cone-two-branches-claim` (`0084cda8`) answers what §198.2 flagged under
§16.10. The claim in `in_the_bulge_of`'s doc comment **holds**, and it is more
exact now. A cone face lies on one nappe. The fuse still cuts a plane face along
the whole double cone, and `Cone::side_of` calls both nappes inside.
`a_plane_across_a_cone.rs` shows this with a frustum (slope 1/2, `z` from 2 to 8)
and a block whose face `x = 3/8·(z+1)` is steeper than the cone's side:

1. `Union(block, cone)` refuses "a point inside a piece of a face". The corners it
   names sit at `z ≈ 18.7`, far above the frustum's top: the fuse cut the face
   where the cone has no face.
2. On the block's wall `y = −10`, the cone's sign is `Neg` at `z = 22`, `Pos` at
   `z = 0` and `Neg` at `z = −22`. So "inside" is two regions there.
3. The point where the axis crosses the leaning face, `(0, 0, −1)`, is inside by
   the sign and outside the frustum. That is the point `half_way_into_either_arc`
   builds its candidates round.

The lane also tried counting cones, as a probe that was never committed. The
frustum-against-block family then went from 6 / 60 / 9 of 96 to 15 / 30 / 24 of
96 (answered / a point inside a piece / four faces along one edge), and two
corner-up-cube fuses became four faces along one edge. No finished fuse gave a
wrong answer, so the demonstration is the hazard taken step by step, not a wrong
solid. Counting cones would need two things first: the pipe's edge-naming fix, and
a check that keeps a bulge point on its branch's own nappe.

**Point 1 is a finding in its own right**: the fuse cuts faces along parts of a
cone's surface that no face of the frustum covers. Here that ends in a refusal. The
lane showed no case where it ends in a wrong answer, and it is written down here,
not claimed either way.

### 199.4 What is left

- The half ball leaned 20° and turned −0.4, refused undecided in the fuse
  (199.1). It needs `fuse/select.rs` work, and its pin is in
  `a_half_ball_skin_with_a_round_hole.rs`.
- The fuse cutting a plane along a cone's other nappe and beyond its face
  (199.3, point 1).
- Still open from §198.3: `mesh.rs:839`, the piece of a ball holding both poles;
  `runs_up`'s short-way sum for a cap whose rim runs through the other pole; the
  narrow lune; and the named refusal for a closed edge that neither direction rule
  answers.

*2026-09-29, Q580: each item checked against `main` at `e342780e`. Four are
closed, one is measured where it answers, and one is still open.*

- **The leaned half ball: closed by §200.1** (`lane-leaned-half-ball`,
  `05206e64`). It was a gap: the cut crosses the flat face, and one more
  candidate point in `candidate_points` answers it. Its pin is now the answer
  test `a_half_ball_cut_by_a_plane_leaning_twenty_degrees_answers_at_every_turn`,
  in the same file.
- **The plane cut along a cone's other nappe: closed for the shape 199.3
  built**, one piece at a time: the block's top face since §211, its wall at
  `z ≈ −20` since §251, and the frustum's own face since §282.
  `a_block_leaning_through_a_frustum_is_cut_along_the_cone_off_the_frustums_own_face`,
  in `a_plane_across_a_cone.rs`, answers and measures against a slice integral.
  Whether a candidate point can land on the other nappe was Q205's: §350
  (`bd525f4e`) found one does on a cone whose radius is more than its height,
  and such a point is now turned away (§350.5). What §350.5 left untraced is
  its own, and Q467 is to pin §350's sweep.
- **`mesh.rs:839`, the piece of a ball holding both poles: closed by §202.1**
  (`lane-ball-both-poles-mesh`, `07d1ffcb` and `7b395d1a`), pinned by
  `a_piece_of_a_ball_holding_both_poles.rs`.
- **`runs_up`'s short-way sum: closed by §200.2** (`lane-runs-up-other-pole`,
  `d1f7eab9` and `76204dff`): 63 caps holding the north pole with the rim
  through the south, at radii 0.5, 1 and 2, in
  `a_cap_whose_rim_runs_through_the_other_pole.rs`. The mirror case, a ball of
  radius 5 less a cap whose rim runs through its north pole, was Q373
  (`e2498262`, done at `938334db`) in `which_pole_this_cap_reaches`, pinned by
  `a_ball_less_a_cap_whose_rim_runs_through_its_north_pole.rs`. Q577 is to pin
  it with the rim's corner on the pole itself.
- **The narrow lune: measured since §200.1 and §330.**
  `a_narrow_lune_from_a_pole_corner.rs` answers 29 wedges cut by two knives
  and 30 cut by one prism, each measured against its closed form, and pins
  every other case as refused by name. Those refusals are still open, and no
  row names them.
- **The named refusal for a closed edge that neither direction rule answers:
  open at the time.** §202.1 did not add it. Its lane found that the only walks
  reaching that code are the caps of
  `a_cap_whose_rim_runs_through_the_other_pole.rs`, and that all of them
  answer. `walk` in `mesh.rs` still keeps the walker's order when neither
  `the_way_a_closed_edge_is_stored_running` nor
  `the_way_a_closed_edge_where_its_faces_touch_runs` answers, and its own
  comment says that gives a cap walked backwards, a wrong mesh rather than a
  refusal.

Q580's lane proposed a row for each of the last two, and its merger filed
them: Q607 traces the narrow lune's refusals, and Q606 is the closed edge.

*2026-09-29, Q606 (`5f6cab39`): no refusal is needed.* Built through
`SolidBuilder`, a ball and a plane cornered exactly on the ball's north pole.
The way round against the walk is refused `FacesDoNotAgreeWhichWayTheyFace` by
`Model::validate`; the other builds, meshes closed and measures at deflections
0.5, 0.05 and 0.005, alone and after a union. Pinned in
`a_closed_edge_neither_direction_rule_answers.rs`. What it leaves: the
cone-apex and torus-axis corners are unbuilt, and the lid-first spelling named
by the lid's plane turned over is refused `AWireDoesNotFanIntoALoopToMesh
{ TooFewCorners }` whichever way round.

Commands: `just decision 198.2`, `198.3`, `197.2`, `167`, `16.10`.
