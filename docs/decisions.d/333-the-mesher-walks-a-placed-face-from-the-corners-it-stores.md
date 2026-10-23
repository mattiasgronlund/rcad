## 333. The mesher walks a placed face from the corners it stores — 2026-09-28

*Queue row Q107, from §249.3's side finding and §249.7's third item. Sent twice:
the first lane, `lane-q107-walk-from-the-stored-corners`, made the change in
`88a3f5ac`. It was not merged because a test outside its write set moved. The
second lane, `lane-q107-walk-from-the-stored-corners-2`, off `96da3252`, merged
that commit unchanged, moved the test, and wrote this section. **The answer to
§249.7 is yes.** `BuildingAMesh::walk` now starts a placed face's walk from the
corners the arena stores. Every point of a placed bar is then the unplaced
bar's point moved out, bit for bit. The triangles are not, because `cut_up`
still moves points out and back. Fixing `cut_up` the same way makes the
triangles match too, but it turns a far-off ball's right refusal into a wrong
mesh, so that fix is left for the next row (333.6).*

### 333.1 What changed

For a placed face, the frame the mesher works in is the placement's own
inverse (`the_frame_this_face_is_meshed_in`). `walk` used to hand
`rcad_geom::walk` each end of an edge as the **located** corner moved back into
that frame. That is the placement and then its inverse, so the corner is rounded
twice: `(3, 0, 0)` turned an eighth about `z` and back is `(3, −4.4e−16, 0)`
(§249.3). A walk that starts from a slightly different corner puts its points
somewhere slightly different.

Now `walk` reads the corner the arena stores, through
`where_this_corner_is_in_the_frame_it_is_meshed_in` in `mesh.rs` and a new
`pub(crate)` `VertexView::stored_point` in `view.rs`. This is the same reason
`the_surface_this_face_is_meshed_on` already reads the surface out of the arena.
It is right because a view hands its location down unchanged to every wire,
coedge, edge and vertex under it, and the edge came out of the face's own wires.
So the stored corner is exactly the point the located one stands for. The helper
asserts that the corner's location is the face's, rather than assuming it. A
face at no placement keeps its exact shift, as before.

`crates/rcad-topo/tests/a_placed_bar_is_meshed_from_the_corners_it_stores.rs`
pins it.

### 333.2 What it reaches: every point

The bar is radius 3 and height 5, meshed at `0.005` under `Policy::certified()`.
It is compared with the same bar at no placement, moved out point by point, bit
for bit. The twelve placements are those of the pinning test. Measured with a
temporary probe, `crates/rcad-topo/tests/zz_q107_probe.rs`, run as
`mise exec -- just t -p rcad-topo --test zz_q107_probe probe_bar_rows --no-capture`.
It is deleted again, and a copy is kept at
`/tmp/claude-1000/-home-mattias-git-mattiasgronlund-rcad-worktrees-dispatch-next/0819adaf-2b35-4695-9192-2958341eef17/scratchpad/zz_q107_probe.rs`.
"Old corners" is the same tree with `walk`'s `ends` put back to
`back.apply_to_point(end.point())` for the run, then restored.

"Triangles" counts a placed triangle whose three point indices, sorted, are not
among the unplaced bar's triangles. Every mesh has 252 points and 500 triangles.

| placement | points off, old corners | points off, now | triangles off, old corners | triangles off, now |
|---|---|---|---|---|
| an eighth about `z` | 75 | 0 | 193 | 174 |
| the same, then `(3, −5, 7)` | 184 | 0 | 227 | 191 |
| the same, then `(0.25, −0.5, 0.125)` | 75 | 0 | 200 | 174 |
| the same, then `(0.1, 0.2, 0.3)` | 150 | 0 | 242 | 182 |
| 0.7 about `(1, 2, 3)` | 0 | 0 | 237 | 237 |
| the same, then `(3, 5, 7)` | 71 | 0 | 315 | 300 |
| 2.5 about `(1, 2, 3)` | 0 | 0 | 297 | 297 |
| the same, then `(3, 5, 7)` | 172 | 0 | 307 | 312 |
| 0.7 about `(−3, 1, 0.5)` | 0 | 0 | 236 | 236 |
| the same, then `(3, 5, 7)` | 118 | 0 | 272 | 271 |
| 2.5 about `(−3, 1, 0.5)` | 0 | 0 | 288 | 288 |
| the same, then `(3, 5, 7)` | 240 | 0 | 315 | 287 |

**Every point now matches in every row.** The four rows at 0 under the old
corners are placements whose round trip gave the bar's two corners back exactly.
The pinning test also asks, for each placement, whether the round trip moves a
corner, and needs at least half of them to, so that it can tell the two walks
apart.

### 333.3 What it does not reach: the triangles

The triangles still differ in every row, between 174 and 312 of 500. `cut_up`
hands the triangulator each wire in the face's own two coordinates, and
`walk_a_wire_round` works those out from points moved out and back again. It
reads a walked point from the point table, where it sits moved out, and moves it
back with `back.apply_to_point(self.points[at])`. It moves a corner back the
same way, in the `at_a_pole` test, at a wire step's start corner, and at a pole
step. That is the double rounding of 333.1, one step later. The points agree bit
for bit, but the numbers the triangulator decides on do not, and a cap of
consecutive points on one circle is full of ties (Q358's finding). So
the triangulation changes.

`which_pole_this_cap_reaches` (`mesh.rs`, the two lines that set `at` and
`step`) also moves the loop's first two table points back. It was left alone: it
reads only which side of the partner surface a step goes, not where the triangles
go.

### 333.4 It frees nothing in the charge for moving a face

`how_far_moving_this_face_rounds` is `how_far_placing_this_face_rounds` plus
`how_far_shifting_this_face_rounds`. The placing term pays for carrying a mesh
point **out** by the placement, and every point is still carried out. The
shifting term is zero for a placed face, because only a face at no placement is
shifted (`the_frame_this_face_is_meshed_in`). Moving the corners back in was
never charged, so the change leaves nothing to take off.

### 333.5 A refusal moves: the ball up the z axis

`how_far_out_a_ball_can_sit_and_still_mesh.rs` places a unit ball at
`EVEN = 1.0808639105689192e16` and at `ODD = 1.170935903116329e16`, along `x`
and up `z`, at a deflection of `1e5`. The parity law still holds: the whole file
passes, 13 of 13 (`mise exec -- just t -p rcad-topo --test merged -E 'test(/^how_far_out_a_ball_can_sit_and_still_mesh::/)'`).
Up `z`, `EVEN` still refuses, but with a different error. Before this change it
was `Geometry(CannotNameWhichPieceOfACrossingThisIs)` from the walk. Now it is
`AWireWillNotLieFlatOnItsFace` from `cut_up`.

**The cause, checked and not guessed.** Temporary `eprintln!`s in `walk` and
`walk_a_wire_round` printed what each step read, with
`mise exec -- just t -p rcad-topo --test merged -E 'test(~how_far_out_a_ball_can_sit_and_still_mesh::the_law_travels)' --no-capture`.
They are removed again, and the diff is kept at `…/scratchpad/probe-z-axis.diff`
beside the probe. Up `z` at `EVEN`, with this change:

```
PROBE walk: located ends [Point([0.0, 0.0, 1.0808639105689192e16]), Point([0.0, 0.0, 1.0808639105689192e16])]; moved back [Point([0.0, 0.0, 0.0]), Point([0.0, 0.0, 0.0])]; stored [Point([0.0, 0.0, 1.0]), Point([0.0, 0.0, -1.0])]
PROBE walk made 1 points: [(Point([1.0, 0.0, 1.8369701987210297e-16]), Point([1.0, 0.0, 1.0808639105689192e16]))]
PROBE cut_up corner 1: located Point([0.0, 0.0, 1.0808639105689192e16]) moved back Point([0.0, 0.0, 0.0]) at (0.0, 0.0)
PROBE cut_up walked point 0: table Point([1.0, 0.0, 1.0808639105689192e16]) moved back Point([1.0, 0.0, 0.0]) at (0.0, 0.0) pole false
PROBE cut_up pole 2: located Point([0.0, 0.0, 1.0808639105689192e16]) moved back Point([0.0, 0.0, 0.0]) at (0.0, 0.0)
PROBE cut_up wire lies flat: false
```

And with `walk`'s ends put back to the old form, on the same run:

```
PROBE walk: located ends [...1.0808639105689192e16], [...1.0808639105689192e16]; moved back [Point([0.0, 0.0, 0.0]), Point([0.0, 0.0, 0.0])]; ...
PROBE walk refused: CannotNameWhichPieceOfACrossingThisIs
```

So the first lane's account holds. Up `z`, both poles `c ± 1` round onto `c`.
The walk used to be handed both ends already collapsed onto the centre, and the
geometry layer refused. Now it is handed the stored poles `(0, 0, ±1)` and walks.
`cut_up` still moves every point out and back, and gets the poles and the seam's
middle all at height 0. All three read as `(0, 0)` in the ball's two
coordinates, so the wire will not lie flat.

Along `x` the collapse is in a coordinate the poles do not use. The same probe
at `EVEN` along `x`: the poles come back as `(0, 0, ±1)`, and the seam's middle
comes back as `(0, 0, 1.8e−16)`, on the axis between them. The wire lies flat,
and the triangulator refuses `CornersCoincide`. That is unchanged by this
section. The test file now names `AWireWillNotLieFlatOnItsFace` in
`what_it_refused` instead of letting it fall to the catch-all. It checks the
three points' round trips with the kernel's own `Isometry`, and its doc gives
this cause in place of "recorded and not explained".

### 333.6 Reading cut_up's points where they were made: a wrong mesh

The first lane also tried the matching fix in `cut_up`. `walk` records, for each
point it walks, the frame it was walked in and the point as walked, and
`walk_a_wire_round` reads that point when the frame is the face's own. It reads
the corners through `where_this_corner_is_in_the_frame_it_is_meshed_in` at all
three corner sites. Only those hunks of the first lane's diff were applied here,
to measure, and then removed. They are kept at `…/scratchpad/cut-up-hunks.diff`.

**It makes every triangle match.** With the probe of 333.2, all twelve rows came
out with 0 of 252 points and 0 of 500 triangles differing, whether triangles are
counted as sorted sets or by position.

**And it turns the ball at `x = EVEN` from a refusal into a mesh.** Measured with
`mise exec -- just t -p rcad-topo --test zz_q107_probe probe_ball_out_there --no-capture`.
The probe also calls `Model::validate` and `SolidView::measure` at the same
deflection. Without the hunks, all three refuse:

```
BALL EVEN at x = 1.0808639105689192e16: refused CannotMeshAFace { ..., why: CornersCoincide { position: 2 } }
BALL EVEN validate: Err(CannotMeshAFace { ..., why: CornersCoincide { position: 2 } })
BALL EVEN measure refused: CannotMeshAFace { ..., why: CornersCoincide { position: 2 } }
```

With them:

```
BALL EVEN at x = 1.0808639105689192e16: 6 points, 8 triangles, deviation Approximation { magnitude: 43.866666666666674, cause: FlatStandIn }
  point 0: minus the centre Vector([[0.0, 0.0, 1.8369701987210297e-16]])
  point 1: minus the centre Vector([[0.0, 0.0, 1.0]])
  point 2: minus the centre Vector([[0.0, 0.0, -1.0]])
  point 3: minus the centre Vector([[0.0, 1.2246467991473532e-16, 0.0]])
  point 4: minus the centre Vector([[0.0, 1.0, 0.0]])
  point 5: minus the centre Vector([[0.0, -1.0, 0.0]])
  triangle [3, 1, 4] [3, 4, 2] [0, 2, 4] [0, 4, 1] [3, 2, 5] [3, 5, 1] [0, 1, 5] [0, 5, 2]: area 0.5 each
  triangles with no area: 0; volume 0 (the ball's is 4.1887902047863905)
  x spans 0 to 0 about the centre
  furthest a sampled point of the ball sits from the mesh: 1
BALL EVEN validate: Err(Invalid(VolumeIsNotPositive { volume: 0.0 }))
BALL EVEN measure refused: Invalid(VolumeIsNotPositive { volume: 0.0 })
```

**It is a wrong answer, and the file's "limit, not a gap" stands.** What the
measurement shows:

- The mesh is flat. Every point has the centre's `x`. In the stored frame the
  six points are an octahedron, and its two corners on the `x` axis, `(±1, 0, 0)`
  to within a rounding, both come out on the centre: points 0 and 3 are
  `2.2e−16` apart where the ball's are 2 apart. They are distinct only by the
  roundings left in `y` and `z`.
- So each triangle lies on top of another with its corners the other way round:
  `[3, 1, 4]` and `[0, 4, 1]`, `[3, 4, 2]` and `[0, 2, 4]`, and so on. The mesh
  encloses exactly nothing. The sum over it is `0.0`.
- The kernel's own checks then call a sound ball malformed. `Model::validate`
  and `measure` both answer `Invalid::VolumeIsNotPositive { volume: 0.0 }`, which
  says the solid "encloses a negative amount of space, or none". The ball is the
  kernel's own `add_ball`, which checked it when it was built. The same ball at
  `ODD` is refused by `validate` for its placement,
  `DynamicRangeTooWide { ratio: 1.17e16 }`, and `measure` hands back a volume
  of `2.67` with an error bound of `inf`.
- By distance alone it would pass. Every sampled point of the ball is within 1 of
  the mesh, inside the `43.87` it claims and the `1e5` asked. Distance is not
  the only thing a mesh of a solid promises, though: the file's own `ODD` mesh,
  which it accepts, is a closed octahedron 4 units across with volume `2.67`,
  and this one has none.

So this is not a right answer that the refusal withheld. The refusal is the
right answer, and the fix in `cut_up` has to keep it. It also changes one pinned
number elsewhere. Running
`mise exec -- just t -p rcad-topo --test merged -E 'test(/^how_far_out_a_ball_can_sit_and_still_mesh::/) or test(/^whether_the_floor_predicts_the_step_cap::/) or test(/^a_placed_bar_is_meshed_from_the_corners_it_stores::/)' --no-fail-fast`
with the hunks gave 10 failures of 16. Nine are in the ball file: every test
that expects a refusal inside or above the band now meshes, including this
section's `z` pin. One is
`whether_the_floor_predicts_the_step_cap.rs`'s
`the_fallback_a_bar_a_billion_units_out_already_had_says_the_same_thing`,
`5.993841838603952e-6` against a pinned `5.994025313141081e-6`.

**Nor does it reach Q358.** A solid read back from a STEP file stands at no
placement and is meshed in the world frame, shifted exactly. The `cut_up` fix
reads a point as walked only when the frame matches, so the read-back's flat caps
still move their points back from the table. Q358's own fix, the located plane
coordinates for a flat face, is the one that reaches it.

### 333.7 What this leaves

- Read `cut_up`'s points and corners in the frame they were made in, so a placed bar's triangles are the unplaced bar's too, **without** meshing the collapsed ball at `x = EVEN` (333.6). Whatever keeps it refused has to be named and measured, not found by trial. The table in 333.2 is the test, and `how_far_out_a_ball_can_sit_and_still_mesh.rs` must stay green. The work is in `crates/rcad-topo/src/mesh.rs`, `BuildingAMesh::walk_a_wire_round`.
- Correct the pinned figure in `whether_the_floor_predicts_the_step_cap.rs` if that change moves it, with its cause.

*2026-10-03, Q378: not fixed. §379 has the table on `3d403339`, where only the
wall differs because §362 now cuts the caps where the solid sits. It names the
question that keeps the ball refused (does the mesh enclose space where it lands,
when it did where it was made) and measures it. It also gives the two reasons
the change did not land: no `Error` variant for that refusal in the write set,
and the adapted hunks still leave 141 to 199 wall triangles different.*

*2026-10-03, Q781: the wall and the caps were the other way round. That table
read the caps' keys off a second model; split properly, the 141 to 199 are all
on the caps, which §362 cuts where the solid sits, and the wall matches in every
row since Q780 (§379.5).*
