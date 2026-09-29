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
- No test pins the sweep. Pinning the corner-up and slab families needs their
  solid counts worked out, which this lane got wrong once.
