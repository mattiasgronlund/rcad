## 364. A wire that passes one corner twice is cut into the loops that touch there — 2026-10-04

*Queue row Q686, which replaced Q411 and Q659. Lane
`lane-q686-a-wire-through-one-corner-twice-split-in-two`, off `de799a3b`, and
its second run with queue row Q884, named in 364.5. Every figure below was run
in one of the two, on its base or on its branch, by the test or probe named
beside it.*

  364.1   What refused, and why the triangulator was right to
  364.2   The cut, on a flat face and on a ball
  364.3   What it answers
  364.4   The exception to §193.2
  364.5   What else it moved
  364.6   What this leaves

### 364.1 What refused, and why the triangulator was right to

The prism is the triangle `(0, 0)`, `(4, 3)`, `(−3, 3)` stood 32 tall, so its
upright edge on the `z` axis has wall 1 (`0.6x − 0.8y = 0`) on one side and
wall 2 (`x + y = 0`) on the other. A ball of radius 1 at `(1, 0, 16)` touches
that edge exactly at `v = (0, 0, 16)`, a shape a designer makes on purpose;
queue row Q74's ball, radius `1 + 2⁻⁵¹` at `(1, 2⁻²⁵, 16)`, crosses it by far
less than a rounding.

Wall 1 cuts a cap `0.4` deep off the ball, and its circle lies in the wall and
touches the edge at `v`. Wall 2's plane cuts the ball in a circle that lies
outside that wall and touches it only at `v`. Both places the edge meets the
ball are the one vertex `v`, and both circles start and end there. So the
ball's face between the two circles is bounded by one wire that is a figure
eight through `v`, and in the union wall 1's wire runs down the edge to `v`,
round its circle and on down the edge. Each names `v` twice.

At the base, a probe running both balls under `Policy::certified()` and
`Policy::tolerant(25.0)`, each rule and either solid first, gave:

- **the touching ball**, under either policy: union and difference refused
  `CannotMeshAFace { CornersCoincide { position: 2 } }` with the ball first
  and `{ position: 8 }` with the prism first; the intersection answered
  `0.43527313408418705 ± 0.0025124898426150894`, the cap being `0.4356342`.
- **Q74's ball**: under `tolerant(25)` the same two refusals and an
  intersection of `0.43528013214842426 ± 0.0025147016622234972`; under
  `certified` every rule, either way round, `Undecidable` on *order along a
  crossing*. That last is the fuse, not the mesher, and is pinned as it
  answers.

**The fuse was right.** Handed the figure eight whole, the triangulator refuses
a loop that names one point twice, and must: it fills by crossings, so the
patch it would draw for the ball's face is the insides of the two circles,
which is the opposite of the face. So `rcad-mesh` stays as it is, and the cut
belongs in the mesher, which knows what the wire was.

### 364.2 The cut, on a flat face and on a ball

`BuildingAMesh::cut_up` now cuts each walked wire apart at a corner of the
model that **starts two of its coedges**
(`corners_a_wire_sets_off_from_twice`), and only there — never at a point that
merely turns up twice in a patch. The cut (`apart_at`) is taken only where the
wire passes that point exactly twice, at **one corner of the chart** both
times, with three corners or more each side, and only where **each part passes
every point once** (`apart_where_it_passes_a_corner_twice`). Each condition
keeps out a shape that already had a meaning:

- a **seam**'s corner starts two coedges, but its two visits are a whole turn
  apart in the chart;
- a **slit** runs out along one edge and back from a corner that starts two
  coedges, and the part out and back passes every point of the slit twice, so
  it stays whole for §193.1's `open_a_slit`;
- a part with a **pole side** passes the pole's point at both ends of the side;
- a wire that goes **right round** either way is not cut at all.

The part that holds the most of the chart goes first, so a split outer wire
keeps its outline in front.

**On a flat face** the parts are an outline and a hole touching it at `v`.
The triangulator lets two loops meet at a corner both name
(`refuse_loops_that_touch`) and fills by crossings, so the hole comes out of
the patch.

**On the ball** the parts are two holes touching at `v`, and the row asked
`open_the_ball_along_a_seam` and `cut_a_hole_up_to_both_poles` to take two.
They did not need to: §402's `open_the_ball_into_strips` (Q451) already takes
a ball face whose every loop is a hole, and two that overlap in longitude are
cut side by side (`cut_holes_side_by_side_up_to_both_poles`). Two more steps
were needed there:

- **The piece between two touching holes is pinched shut at the point they
  share**: one piece above it and one below, which that function walks round as
  one loop through `v` twice. `apart_where_two_holes_touch` cuts it there into
  the two, each with the material inside it, before any is cut narrower; a part
  without is left whole and refused as before. It fires only at a point two of
  the face's loops share.
- **Touching holes go in one run of the strips.** From `−y` (below) the
  measure at deflection `0.001` was refused
  `CurvedFaceWithAHoleGoesTooFarRound`: the longitude the two holes share was
  read off each from the seam on its own, the end of the one as its start plus
  its width and the start of the other as itself, and the two came out an ulp
  apart. The holes landed in two runs with a cut through `v`, and neither lay
  strictly inside its strip. From `+x` the ulp fell the other way.

### 364.3 What it answers

`a_ball_touching_the_edge_between_two_tilted_walls.rs`: under `tolerant(25)`
both balls, every rule, either solid first, are one solid within its own
published `volume_error` of the closed form — intersection the cap,
`0.4356342` for the touching ball; ball less prism `4.1887902 − 0.4356342`;
prism less ball `336 − 0.4356342`; union `336 + 4.1887902 − 0.4356342`, each
worked from the ball's own radius and centre. Under `certified` the touching
ball answers the same six, and Q74's ball is refused on *order along a
crossing* as at the base. The probe on the branch's first commit, `8a581763`,
gave the touching ball's
union `339.75157955006347 ± 0.01005535780845121` against `339.753156`, and its
ball less prism `3.751579415079623 ± 0.010055357694861712` against
`3.7531560`.

`a_ball_touching_a_prisms_edge_exactly.rs` (queue row Q666) pinned both refusals; they
move. From `+x`, radius 1 and 2, eight heights, the union and the ball less the
prism measure `336 + ball − cap` and `ball − cap`. From `−y`, where both
circles lie outside both walls and the ball shares only `v` with the prism,
the union is two solids measuring `336 + ball` between them and the ball less
the prism is the whole ball. That is queue row Q679's case, answered here.

### 364.4 The exception to §193.2

§193.2 says no builder-made solid makes `CornersCoincide` happen, and that the
"cannot be cured" claim in `its_sides_cross` rests on the argument and a
measurement. A **fuse-made** solid did, as above, and its cure was not a finer
mesh but a cut the mesher makes before the triangulator sees the wire. The doc
of `its_sides_cross` in `validate.rs` now says so: a wire that sets off from
one corner twice no longer reaches it, and what still meets itself there is a
slit, a loop that passes a pole, or one whose two visits are not one point of
the chart.

### 364.5 What else it moved

`just t --workspace --no-fail-fast` on the branch: 3028 run, 3027 passed, and
the one failure is a pin outside this lane's write set.
`degenerate_curved_fuses.rs`'s
`a_wedge_lying_on_a_cones_side_shares_and_leaves_what_is_worked_out_by_hand`
pinned `CornersCoincide` for the wedge and frustum's union both ways round and
for the wedge less the frustum, under `certified` and `tolerant(100)`. Its own
doc named the union's cause as this one: the wedge's floor less the circle the
frustum cuts in it at `z = 8`, which touches the floor's edge at `(1, 0, 8)`. A
probe on the branch, under both policies: the union, either way round, answers
one solid that validates and measures `403.3953495976202 ± 4.982689802626174`
against the hand-worked `403.7412532358421`, under it as an inscribed mesh is;
the wedge less the frustum is refused `TwoPatchesOfTheMeshDrewTheSameSide {
drawn: 2, used: 4 }` instead, its two slivers of cone skin meeting along a
ruling.

**That pin moved in a second run of the row**, `lane-q686-a-wire-through-one-corner-twice-split-in-two-2`,
off `c11bf21f` with this branch merged in, which took queue row Q884's union
along with it. The test now holds the union, either way round and under both
policies, to `frustum + 96 − shared` within the file's half a per cent and
under it (`Side::Under`, the file's side for a mesh inscribed in a round wall
that bounds the solid), with the certified fuse recording nothing and the model
validating; and it pins the wedge less the frustum
`TwoPatchesOfTheMeshDrewTheSameSide { drawn: 2, used: 4 }` by name under both.
Measured there, at the file's deflection of `0.02`: `403.3953495976202` with
the frustum first and `403.39534959761846` with it second, under either policy,
against `403.7412532358421`, 0.086 % under. A probe walking the union's faces
found one wire that starts two coedges at a corner and is not a seam: the
floor's, five coedges on the plane `z = 8`, through `(1, 0, 8)` twice. The two
cone faces' wires pass a corner twice only at their seams, `(0.5, 0, 4)` and
`(1, 0, 8)` below, `(2.5, 0, 20)` and `(3.5, 0, 28)` above.

The five `CornersCoincide` pins the row named —
`a_block_face_through_a_cones_apex.rs`, `a_face_bounded_by_two_steps.rs`,
`how_far_out_a_ball_can_sit_and_still_mesh.rs`,
`meshing_a_plane_cut_along_a_cones_other_nappe.rs` and
`whether_a_coarser_deflection_always_meshes.rs` — all pass unchanged, and
`just test-slow` runs its 40 and passes them.

### 364.6 What this leaves

- **The wedge less the frustum** in `degenerate_curved_fuses.rs`, 364.5: the
  union is pinned as an answer now, and the difference by its new refusal,
  `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 2, used: 4 }`. Which mesher step
  draws that side twice, and whether that answer is a manifold at all, with a
  demonstration, is the other half of queue row Q884 and is filed as a row of
  its own.
- **Q679** is answered by 364.3's `−y` pin.
- **Q74's ball under `certified`**, refused on *order along a crossing* in
  364.3, is open queue row Q908's shape, which names its cause; no row is filed
  for it here.
- **Queue row Q698's** block edge grazing a ball at `cx = 16`, read by Q663's
  lane as this figure eight, was not run here.
