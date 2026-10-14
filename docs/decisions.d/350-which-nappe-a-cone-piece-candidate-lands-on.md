## 350. Which nappe a cone piece's candidate lands on — 2026-09-28

*Queue row Q205, from §282.7. Lane `lane-q205-which-nappe-a-cone-piece-holds`,
off `2beb47d3`. §282.7 noted that the curved arm of `the_piece_holds` never
asks which half of a cone a point is on, and that no candidate reached the
other half in its runs. **One does, on every cone whose radius is more than
its height** — a half-angle over 45°. A
step of one radius towards the apex goes past it, and the trip back onto the
cone keeps the distance along the axis sign and all, so the step lands on the
other nappe. This lane took the first of the brief's two ways: a fuse through
the public API whose candidate lands there. A pointed cone of slope 2 inside
a block had its face placed by a point under the apex, and all four rules
refused `ResultIsNotManifold`. No wrong `Ok` was seen. The fix is two halves of
`select.rs`: a cone piece holds only points on its own nappe, and a new
candidate half way from the corners' middle to the apex is on that nappe
whatever the slope. Over a throwaway sweep of 368 fuses of a slope-2 cone,
72 refusals became right answers and no family lost one.*

  350.1   Where a candidate crosses to the other nappe
  350.2   What it gave
  350.3   The fix, in two halves
  350.4   The sweep
  350.5   What this leaves

### 350.1 Where a candidate crosses to the other nappe

`back_onto_the_surface` puts every candidate for a curved piece back onto its
surface through `Surface::coordinates_of` and `point_at`. For a cone the second
coordinate is `along = (p − apex) · w`, read straight off the candidate and
never made positive, and `Cone::point_at` walks a negative `along` onto the
other nappe (its doc says so). So **a candidate lands on the other nappe
exactly when its own `along` is below zero**, and the question is which of
`candidate_points`'s families can make one. For a curved piece there are three:

1. **Blends of corners.** Each is a mix of corners with weights that add to
   one. Every corner of a piece is on its face's nappe or at the apex, so
   `along` is at least zero. It can round below zero only when every corner in
   the mix is within a rounding of the apex's plane.
2. **Six steps from the corners' middle `M`**, one along each of `±x`, `±y`,
   `±z`, of length `reach = slope · along(M)` (`how_big_the_surface_is`).
   After a step along `±e_k`, `along = along(M) · (1 ± slope · w_k)`. That goes
   below zero when `slope · |w_k| > 1`. For a cone about a coordinate axis it
   is the step down the axis, and it crosses exactly when `slope > 1`.
3. **The step towards every flat side's material**, a unit direction `t` of
   the same length: `along = along(M) · (1 + slope · (t · w))`, below zero when
   `slope · (−t · w) > 1`.

So families 2 and 3 reach the other nappe on a cone whose half-angle is over
45°, and family 1 never does beyond a rounding at the apex. **§282's runs were
all slope 1/2**, where both steps keep at least half of `along(M)`. That is
why none crossed.

At `slope = 1` the step down the axis of a cone about `z` lands on the apex
itself, by the same formula. That is arithmetic and was not run.

**All of this takes the cone to be named about the axis pointing into its
face.** Named about the other way, every corner of the face has `along` below
zero (§357.6), so the face's own nappe is the one with negative `along`, and
"lands on the other nappe" means `along` coming out above zero. Family 1's
premise fails there, and the three families were not worked through again for
that case. Until Q560, `Cone::point_at` also put every candidate on such a
face half a turn round; it now keeps the direction across on either nappe.

### 350.2 What it gave

**A cone swallowed by a block.** The pointed cone of slope 2 up to `z = 8`, and
a block `40 × 40 × 11` from `(−20, −20, −1)` that holds all of it. The cone's
face is bounded by its rim alone, so it has one corner, `(16, 0, 8)`. The
blend and the steps along `±x` and `±y` land back on the rim, and `+z` lands
above the cap. The `−z` step and the step towards the cap's kept side both land
at `(16, 0, −8)`, which has `along = −8`, and are put back onto the cone at
`(−16, 0, −8)`. That is below the cap, which was all the curved arm asked, so
it placed the face: **outside** the block, which stops at `z = −1`. A throwaway
`eprintln` in `place_the_piece`, printing each piece's surface, the point that
placed it and where, showed exactly that on `2beb47d3` (taken out). The face
was dropped where it should be kept and kept where it should be dropped, and
all four rules refused `ResultIsNotManifold`.

**The same cone apart from a block that holds `(−16, 0, −8)`**: a block
`[−20, −12] × [−2, 2] × [−12, 1]`. The block's planes cut the double cone, and
the face comes apart into eight pieces. One of them, near the apex, was placed
by `(−2, 0, −1)`, which is also on the other nappe. It was placed **outside**,
which is right by luck, because the block holds nothing of the cone. Two rules
answered right, and the union and the cone-less-block refused
`ACurvedFaceGoesRightRoundWithNoSeam` both before this change and after it.
That refusal is something else and was not traced.

**No wrong `Ok` was seen.** The 368 fuses of 350.4 each hold their answer to
the model's own check, the volume and a grid of points, and on the base none
failed them. That is a count of what was run, not a claim that none can.

### 350.3 The fix, in two halves

**`the_piece_holds` asks the nappe** (`is_on_the_nappe_the_piece_is_on`, below
it). The piece's corners say which half it is on: the sign of
`(corner − apex) · w` over each corner's box, the rule `holds.rs`'s
`is_on_the_nappe_the_face_is_on` reads a face by. A corner at the apex, or too
near its plane to tell, says nothing. Corners that disagree, or none that says,
give `None`. A point on the other half is `Some(false)`. A point at the apex,
or one whose side is `Undecidable`, is `None`, since the apex is on both halves.
The caller then tries the next candidate, as it always did. The sign is taken
over boxes and is a `Decision`, matched with `let … else` so that the wildcard
count does not move.

This also turns away the apex itself, which on a slope-1 cone the step down
the axis landed on (350.1). It used to be accepted, since it is below the cap.

**`candidate_points` adds half way from the corners' middle to the apex.**
Its `along` is `along(M) / 2`, on the corners' own half for any slope. It goes
**last**, so every earlier candidate keeps its place and no piece that found a
point before finds a different one now.

`a_cone_whose_radius_is_twice_its_height_inside_a_block_answers_every_rule_right`, in
`tests/a_plane_cut_along_a_cones_other_nappe.rs`, is the swallowed cone. Run
four ways, each with an environment switch around one half (taken out):

| `select.rs` | the four rules |
|---|---|
| `2beb47d3` | `ResultIsNotManifold` ×4 |
| the nappe check alone | `a point inside a piece of a face` ×4 |
| the new candidate alone | `ResultIsNotManifold` ×4 |
| both | right ×4 |

With the candidate alone, the point under the apex still comes first and still
places the face. With the check alone, nothing is left inside the face. So the
test fails without either half.

`just t -p rcad-topo` with both halves on `bd525f4e`: 1134 run, 1134 passed, as
the verifier counted it twice (the lane's own count was 1135). No pin moved in
`rcad-topo`.

### 350.4 The sweep

A throwaway test (taken out) ran three families round the pointed cone of
slope 2 up to `z = 3`, every rule and both differences, through the file's
`fuse_and_check`: the leaning block of `a_leaning_block` at leans
`0.25, 0.375, 1, 4`, crossings `0, ±0.5, 1.5` and three turns; the corner-up
cube of sides `2, 4, 6`, middles `0, 1, 2.5, 3.5` and two turns; and twenty
slabs `14` deep. **The solid count was left out** for the pairs that share
volume. It was the one number I had to work out by hand, and I got it wrong
for a cube whose top corner pokes out of the cap, where the block less the
cone is two solids. The volume and the grid of points still held every answer.

| family | fuses | right on `2beb47d3` | right with both halves | what moved |
|---|---|---|---|---|
| leaning block | 192 | 98 | 98 | nothing |
| corner-up cube | 96 | 12 | 72 | 44 `a point inside a piece of a face` and 16 `ResultIsNotManifold` became right |
| slab | 80 | 28 | 40 | 12 `ResultIsNotManifold` became right |

With the new candidate alone, the corner-up cube was 56 right: its 44 moved
and its 16 `ResultIsNotManifold` did not. So the 44 wanted a point on the
right half and the 28 were placed from the wrong one. Every other refusal
count in the table was the same on all three runs.

### 350.5 What this leaves

- **The steps of families 2 and 3 still land on the other nappe** on a cone
  whose radius is more than its height. They are turned away now and not
  used. Capping the step so it cannot pass the apex would change which point
  places a piece on every such cone. With the check turning those points away
  and the new candidate after them, no refusal in the sweep asked for it, so
  it was not done.
- **The leaning block's 56 `a point inside a piece of a face`** on the wide
  cone did not move with either half. Not traced.
- **`ACurvedFaceGoesRightRoundWithNoSeam`** on the union and the
  cone-less-block of the wide cone apart from a block (350.2), and 12 and 8 of
  it in the corner-up and slab families. Not traced.
- §282.7's last bullet says this was unmeasured. It is outside this lane's
  files, and this section is the answer to it.
- **Q467 pins the sweep**, in `a_plane_cut_along_a_cones_other_nappe.rs`,
  round §350.4's cone (`THE_STEEP_CONE`) and round a cone of slope 1 up to
  `z = 3` (`THE_UNIT_CONE`). §350.4 wrote down neither its turns nor its slabs'
  heights, so the pins choose them: turns `0`, `1` and `3` for the leaning
  block, `0` and `0.7` for the cube, and slabs from `−14`, `−8`, `−3`, `0` and
  `2`, turned `0`, `0.5`, `1` and `1.5`. Every solid count is worked out by hand
  (`a_leaning_blocks_solids`, `a_corner_up_cubes_solids`, `a_slabs_solids`),
  none read off the kernel; the cube whose top pokes out of the cap is two
  solids there. Round the steep cone:

  | family | fuses | right in §350.4 | right in the pins | the others |
  |---|---|---|---|---|
  | leaning block | 192 | 98 | 178 | 12 `CannotTellTheCellsOfAFaceApart`, 2 `CannotMeshAFace` |
  | corner-up cube | 96 | 72 | 88 | 8 `order along a crossing` |
  | slab | 80 | 40 | 64 | 16 `ACurvedFaceGoesRightRoundWithNoSeam` |

  The two columns are not known to hold the same cases, since the turns and
  heights may not be §350.4's, and the base is ten days of fixes on from
  `bd525f4e`. Which change moved which count was not traced. At the turns
  pinned here, none of the leaning block's 56 `a point inside a piece of a
  face` of the bullet above is left, and the corner-up cube refuses no
  `ACurvedFaceGoesRightRoundWithNoSeam`. **2026-10-11, Q1742 (Q1733, §35.6):**
  the slab is now 50 right of 80, refusing 16 `ACurvedFaceGoesRightRoundWithNoSeam`
  and 14 `side of a half-space`, and no more `a point inside a piece of a
  face`. **2026-10-14, Q1871 (Q1837):** the two slab pins in
  `a_plane_cut_along_a_cones_other_nappe.rs` now hold 64 right and 16
  `ACurvedFaceGoesRightRoundWithNoSeam`; Q1743's 14 `side of a half-space`
  answer right since Q1837's fix to `walk.rs`. Turned `1.5` the slab, as the
  model stores it, leaves a lens of the cone past its end at `−y`, `3.6e−16`
  inside the rim.
- **Round the unit cone**, the leaning block is 169 right of 192, the cube 88 of
  96 and the slab 80 of 80 (Q1733, §35.6; was 68 of 80, naming 12 `a point
  inside a piece of a face` in the slab from `−8`). The leaning block refuses 8
  `order along a crossing` and 2 `AWireWillNotLieFlatOnItsFace`, 5 answers do not measure (4
  `CannotMeshAFace`, 1 `TwoPatchesOfTheMeshDrewTheSameSide`), and 8 more fuses
  are Q1562's, below. Q1461's cone bands moved eight, each a block that misses
  the cone, and all eight answer right since: leaned `1`, crossing `−0.5`,
  turned `1`, the four rules refused `order along a crossing`; and leaned `4`,
  crossing `0`, turned `3`, where the turn rounds the face clear of the apex,
  they refused `ResultIsNotManifold`. Before Q1461 the leaning block was 161
  right and refused 12 `order along a crossing` and 4 `ResultIsNotManifold`.
  Which of its two steps moved them was not traced. None of the unit cone's
  refusals names the apex. Nobody traced whether its step down the axis lands
  on the apex, as 350.1 works out, and is turned away there, as 350.3 says.
- **The unit cone's `a point inside a piece of a face`** are 8 in the cube
  (side `2`, middle `1`, both turns). An untraced lead. **2026-10-11, Q1742
  (Q1733, §35.6):** the 12 that used to be in the slab (from `−8`, turned
  `0.5`, `1` and `1.5`) are gone; the slab is 80 of 80 now.
- **Leaned `1` at crossing `0`, the face touches the unit cone along the ruling
  `x = z`.** Turned `1`, the face as the model stores it passes `4.4e−16` beyond
  the apex, so it cuts a real sliver from the cone, at most `6e−8` across
  (`2a3486d6`'s message has the exact arithmetic). The kernel's intersection is
  that sliver, one solid, so it answers the shape the floats describe and is not
  a defect. It does not measure, and the union and the cone less the block
  refuse `AWireWillNotLieFlatOnItsFace`.
- **Q1562, read again.** Leaned `4` at crossing `0`, the stored face puts the
  apex inside the block turned `0` and `1`, by `2.2e−16` and `4.4e−16`, so a
  union of one solid holding a tip of the cone about `3e−16` tall is the right
  shape. The defect is that `fuse` answers `Ok` with a model its own `validate`
  refuses (`DynamicRangeTooWide`), where it should refuse.
  `a_unit_cone_union_a_lean_four_block_passes_fuse_but_fails_validate` pinned
  it until Q1562 made the fuse refuse it; the note at the end of this section
  says how.
- **The pins check every second point of the grid, and are cut into pieces of
  about 20 s.** On the whole grid they cost too much for either tier, and the
  cost was the check, not the fuse. The corner-up cube round the steep cone,
  run one fuse at a time with nothing else of the lane's running, at a load of
  34 to 69 on 24 cores, took 953.6 s: 1.7 s in `model.fuse` and 944.5 s in
  `fuse_and_check`'s grid, which asks `classify_point` of up to 30 625 points.
  One case took 110.6 s alone, and its union 55.2 s, 54.7 s of it the grid, so
  no cut by lean, turn, side or case brought a piece to §397.4's 20 s. Timed
  point by point at a load of about 61, that union (side `2`, middle `0`,
  unturned) took 25.9 s over its 30 222 clear points, and 24.5 s of it went on
  the 29 492 outside both the cone and the cube, at 0.83 ms each; the 668
  inside the cone cost 1.9 ms each. Q850's cube families round the frustum and
  the slope-½ pointed cone took 54.8 s and 51.2 s alone, whole, under the same
  check (§397.4), and why one answer round the steep cone costs so much more
  to ask about is not traced.

  So the pins round both cones check every second point along each axis
  (`tally_on_every_second_point`): 4 225 points of the same lattice, a step of
  `1` where the whole grid's is `0.5`. Every family keeps the tally above
  there. The floor on points decided drops from 5 500 to 1 500 where the
  answer measures and from 7 000 to 4 000 where it does not, under the fewest
  these fuses decide there, 1 553 (the steep cone's union with the block
  leaned a quarter, crossing `1.5`, turned `3`) and 4 159. A row filed with
  this section asks for a cheaper `classify_point` on points outside both
  shapes, and a row after it puts these pins back on the whole grid.

  Each family is cut the way Q850 cut its cone families: the leaning block by
  lean and turn round the steep cone and by lean round the unit cone, the cube
  by middle and turn, and the slab into the slabs from below the apex and
  those from it up. Each piece pins its own counts, and a family's pieces add
  up to its tally. Each piece's wall alone, one at a time with nothing else of
  the lane's running (`just t -E '…' --test-threads 1`), in seconds:

  | family | pieces | load |
  |---|---|---|
  | steep, leaning block, turned `0`, `1`, `3` | lean `0.25`: 11.0, 11.4, 11.4; `0.375`: 22.7, 10.7, 18.4; `1`: 9.4, 11.6, 10.3; `4`: 14.4, 5.7, 8.9 | 24 to 48 |
  | steep, corner-up cube, turned `0`, `0.7` | middle `0`: 9.0, 20.5; `1`: 13.3, 18.6; `2.5`: 29.3, 14.2; `3.5`: 22.4, 9.1 | 46 to 43 |
  | steep, slab | from below the apex 16.7, from it up 25.2 | 46 to 43 |
  | unit, leaning block | lean `0.25` 14.2, `0.375` 13.0, `1` 9.8, `4` away from Q1562 7.8 | 30 to 24 |
  | unit, corner-up cube, turned `0`, `0.7` | middle `0`: 6.8, 6.7; `1`: 4.7, 5.2; `2.5`: 7.5, 7.4; `3.5`: 12.9, 13.4 | 30 to 24 |
  | unit, slab | from below the apex 11.0, from it up 13.5 | 30 to 24 |

  The five over 20 s took 11.8, 9.2, 17.4, 8.8 and 13.7 s run again at a load
  of 38 to 32, so the load put them there and they are not cut further. All 36
  are in `slow_tests`, and the Q1562 pin stays in `test`. `just test-slow`
  alone took 219.8 s with them, at a load of 30 at the start and 56 at the
  end, against 252.1 s on `e7271921` without them, at 29 and 72. Its wall is
  still the frustum's families of up to 181 s; the 36 run beside them. The
  tests are new, so they have caught nothing yet, and their score for
  `slow_tests` is cost alone.

> **2026-10-10 note (Q1562): the eight fuses are refused now, by name, and it
> is a limit.** Each of them, lean `4`, crossing `0`, turns `0` and `1`, every
> rule, answered `Ok` with one solid, and `Model::validate` then refused the
> model `DynamicRangeTooWide`: `5.364581909073752e16` for the union and the
> block less the cone turned `0`, `5.364581909073751e16` for the other two,
> and `2.6822909545368416e16` for all four turned `1`. It was the same on
> `cde12ea7`, Q1288's parent, so it is not Q1288's doing and not Q1560's
> cause (the pin's body as a scratch test in a worktree there, `just
> --working-directory target/q1562-parent --justfile
> target/q1562-parent/justfile t -p rcad-topo --test scratch_q1562
> --no-capture`).
>
> **Why the fuse let it out**, read with prints since removed:
> `the_inputs_fit_in_f64` said the inputs fit, for all eight, so
> `Assembly::check_dynamic_range` ran. It measured chords only: `13.097` over
> a chord of `10` for the union and the block less the cone, and nothing at
> all for the intersection and the cone less the block, whose answers have no
> open edge. `Model::check_dynamic_range` also takes, on each cone face, the
> cone's radius at each corner that is not a pole at the apex, and a fuse
> writes no pole on a cone face. Every answer has a cone face bounded by an
> ellipse, a closed edge with one corner, at `(6.1e−17, −2.4e−16, 2.4e−16)`
> turned `0` and `(4.6e−16, −1.5e−16, 4.9e−16)` turned `1`: where the block's
> leaning face, stored a rounding past the apex, cuts the cone. The radius
> there is `2.44e−16` (`4.88e−16`), and `13.097` over it is validate's ratio.
> The fuse's doc said every radius in its answer is one an input already
> carried. That holds for a ball, a pipe and a ring, whose radius is a number
> of the surface, and not for a cone, whose radius at a corner depends on
> where the corner is. Q1560's corner was far out, which the chord measure
> catches; this one is a closed edge, whose chord of zero is skipped.
>
> **It is a limit.** The answer the stored floats describe has that ellipse:
> the face passes the apex by `2.2e−16` (`4.4e−16`), so a tip of the cone is
> inside the block. Leaving it out answers a different shape, a union that
> touches the cone at one point and is not manifold. The ratio is the largest
> coordinate over the smallest feature, so a fix has to change one of the two:
> scaling changes both alike; moving the origin leaves the block `13` long, so
> some corner at least `6.5` from it and the ratio above `1e16`; and only
> another shape changes the feature. None of the three makes it fit.
>
> **The fix is two parts** (`fuse/assemble.rs`, `fuse/mod.rs`). (a)
> `Assembly::check_dynamic_range` takes the cone's radius at every corner of
> each cone face, above zero, as validate does. (b) It measures the answer's
> smallest feature against the larger of the answer's largest coordinate and
> the inputs', which `the_inputs_fit_in_f64` now hands it when the inputs fit.
> The intersection is the tip alone, about `2.4e−16` across and from the
> origin, so its own range fits; validate refuses the model because the inputs
> stay in it beside the answer, and `Model` has no way to take them out. With
> (a) alone, the fold started at zero, both intersections still answered `Ok`
> and the cone less the block was refused at its own `1.2288187693549286e16`
> and `6144093846774564.0`. With both, all eight are refused at exactly
> validate's ratios above, both inputs validate before each fuse, and the
> model validates after each refusal:
> `a_unit_cone_and_a_lean_four_block_through_its_apex_are_refused_as_too_wide_a_range`
> pins it, in place of the pin named in the bullet above (`just t -p rcad-topo
> --test a_plane_cut_along_a_cones_other_nappe
> a_unit_cone_and_a_lean_four_block_through_its_apex_are_refused_as_too_wide_a_range`).
> Behind the guard, (b) refuses only a feature finer than any the inputs could
> carry, and it asks of the inputs and the answer together what validate asks
> of the model the caller holds after the fuse. The check still runs where it
> did; Q1614 moves it ahead of `nest_the_shells`.
>
> **Q1630's wedge on this fix**: `a_pointed_cone(0.5, 8)` and the wedge from
> `[0,0], [6,0], [6,0.1]`, `z` from `−2` to `10`. Both inputs validate, and
> all six fuses (the union and the intersection both ways round, both
> differences) are now refused `DynamicRangeTooWide { ratio:
> 6.002298719014579e16 }`, the ratio validate gave its `Ok` models (a scratch
> test since removed, `just t -p rcad-topo --test scratch_q1630
> --no-capture`). Whether that refusal is a limit or a gap is Q1630's.
